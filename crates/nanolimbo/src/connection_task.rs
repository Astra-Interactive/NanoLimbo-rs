use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bytes::{BufMut, Bytes, BytesMut};
use futures_util::{SinkExt, StreamExt};
use limbo_config::InfoForwarding;
use limbo_net::forwarding::parse_legacy_handshake;
use limbo_net::frame::VarIntFrameCodec;
use limbo_net::identity::offline_mode_uuid;
use limbo_packet::login::{LoginDisconnect, LoginPluginRequest};
use limbo_packet::play::{Disconnect, KeepAlive};
use limbo_packet::status::StatusResponse;
use limbo_packet::{ClientboundPacket, PreEncodedPacket};
use limbo_protocol::buffer::{ProtocolRead, ProtocolWrite};
use limbo_protocol::packet::{ConnectionState, PacketDirection, PacketKind, PacketRoute};
use limbo_protocol::version::ProtocolVersion;
use limbo_server::connected_player::ConnectedPlayer;
use limbo_server::connection_action::ConnectionAction;
use limbo_server::connection_flow::ConnectionFlow;
use limbo_server::connection_id::ConnectionId;
use limbo_server::forwarding_mode::ForwardingMode;
use limbo_server::id_source::IdSource;
use limbo_server::serverbound_packet::ServerBoundPacket;
use limbo_text::chat::Component;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use uuid::Uuid;

use crate::server_context::ServerContext;

/// How often the server asks the client to prove it is still there.
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(5);

/// Channel Velocity forwards the player's identity over.
const PLAYER_INFO_CHANNEL: &str = "velocity:player_info";

/// Clients up to 1.7.6 drop packets arriving with the join game packet, so theirs wait.
const SPAWN_DELAY: Duration = Duration::from_millis(100);

/// Version of Velocity's forwarding format this server understands.
const SUPPORTED_FORWARDING_VERSION: u8 = 1;

type Connection = Framed<TcpStream, VarIntFrameCodec>;

/// Prefixes a payload with the id it travels under on this route.
fn frame_for(route: PacketRoute, kind: PacketKind, payload: &[u8]) -> Option<Bytes> {
    let id = route.id_of(kind)?;
    let mut framed = BytesMut::with_capacity(payload.len() + 5);
    framed.write_var_int(id);
    framed.extend_from_slice(payload);
    Some(framed.freeze())
}

/// Why a connection ended. Every variant is ordinary; none is a server fault.
#[derive(Debug)]
enum Ending {
    ClientClosed,
    Timeout,
    Refused,
    Malformed,
    ServerStopping,
}

struct Session {
    context: Arc<ServerContext>,
    flow: ConnectionFlow,
    address: SocketAddr,
    incoming: PacketRoute,
    outgoing: PacketRoute,
    registered: Option<ConnectionId>,
    forwarded_address: Option<String>,
}

impl Session {
    fn new(context: Arc<ServerContext>, address: SocketAddr) -> Self {
        let policy = context.policy();
        Self {
            context,
            flow: ConnectionFlow::new(policy),
            address,
            incoming: PacketRoute::new(
                ConnectionState::Handshaking,
                PacketDirection::ServerBound,
                ProtocolVersion::MIN,
            ),
            outgoing: PacketRoute::new(
                ConnectionState::Handshaking,
                PacketDirection::ClientBound,
                ProtocolVersion::MIN,
            ),
            registered: None,
            forwarded_address: None,
        }
    }

    /// The address to log and to report, which a proxy may have replaced.
    fn reported_address(&self) -> String {
        if !self.context.config.log_players_ip {
            return "<redacted>".to_owned();
        }
        match &self.forwarded_address {
            Some(forwarded) => forwarded.clone(),
            None => self.address.to_string(),
        }
    }

    async fn send(&self, connection: &mut Connection, kind: PacketKind, payload: &[u8]) {
        let Some(frame) = frame_for(self.outgoing, kind, payload) else {
            tracing::debug!(?kind, version = %self.outgoing.version(), "no id for packet here");
            return;
        };
        if let Err(error) = connection.send(frame).await {
            tracing::debug!(%error, "could not write to the client");
        }
    }

    async fn send_snapshot(
        &self,
        connection: &mut Connection,
        kind: PacketKind,
        packet: &PreEncodedPacket,
    ) {
        match packet.payload(self.outgoing.version()) {
            Some(payload) => self.send(connection, kind, payload).await,
            None => tracing::debug!(?kind, version = %self.outgoing.version(), "no payload"),
        }
    }

    async fn send_encoded(&self, connection: &mut Connection, packet: &impl ClientboundPacket) {
        let mut payload = BytesMut::new();
        match packet.encode(&mut payload, self.outgoing.version()) {
            Ok(()) => self.send(connection, packet.kind(), &payload).await,
            Err(error) => tracing::warn!(%error, "could not encode a packet"),
        }
    }

    async fn disconnect(&self, connection: &mut Connection, reason: &Component) {
        match self.outgoing.state() {
            ConnectionState::Login => {
                self.send_encoded(connection, &LoginDisconnect { reason })
                    .await;
            }
            ConnectionState::Configuration | ConnectionState::Play => {
                self.send_encoded(connection, &Disconnect { reason }).await;
            }
            // Nothing the client would read as a message, so just hang up.
            ConnectionState::Handshaking | ConnectionState::Status => {}
        }
    }

    fn enter_state(&mut self, state: ConnectionState) {
        self.incoming =
            PacketRoute::new(state, PacketDirection::ServerBound, self.incoming.version());
        self.outgoing =
            PacketRoute::new(state, PacketDirection::ClientBound, self.outgoing.version());
    }

    fn register(&mut self) {
        let username = self
            .flow
            .profile()
            .username
            .clone()
            .unwrap_or_else(|| self.context.snapshots.player_list_username.clone());
        // A player who reached this point without an identity is one the configuration
        // names for us, so they carry the player list entry's identity.
        let uuid = self
            .flow
            .profile()
            .uuid
            .unwrap_or(self.context.snapshots.player_list_uuid);

        self.registered = Some(self.context.connections.add(ConnectedPlayer {
            username: username.clone(),
            uuid,
            address: self.address,
            version: self.outgoing.version(),
        }));

        tracing::info!(
            "Player {username} connected ({}) [{}]",
            self.reported_address(),
            self.outgoing.version()
        );
    }

    /// Sends everything a client needs to consider itself in the world.
    async fn spawn_player(&mut self, connection: &mut Connection) {
        let snapshots = Arc::clone(&self.context.snapshots);
        let version = self.outgoing.version();

        self.send_snapshot(connection, PacketKind::JoinGame, &snapshots.join_game)
            .await;
        self.send_snapshot(
            connection,
            PacketKind::PlayerAbilities,
            &snapshots.player_abilities,
        )
        .await;

        let position = if version < ProtocolVersion::V1_9 {
            &snapshots.position_and_look_legacy
        } else {
            &snapshots.position_and_look
        };
        self.send_snapshot(connection, PacketKind::PlayerPositionAndLook, position)
            .await;

        if version >= ProtocolVersion::V1_19_3 {
            self.send_snapshot(
                connection,
                PacketKind::SpawnPosition,
                &snapshots.spawn_position,
            )
            .await;
        }

        // 1.16.4 crashes without a player list entry whether or not one was asked for.
        if self.context.config.player_list.enabled || version == ProtocolVersion::V1_16_4 {
            self.send_snapshot(connection, PacketKind::PlayerInfo, &snapshots.player_info)
                .await;
        }

        if version >= ProtocolVersion::V1_13 {
            self.send_snapshot(
                connection,
                PacketKind::DeclareCommands,
                &snapshots.declare_commands,
            )
            .await;
            if let Some(brand) = &snapshots.brand {
                self.send_snapshot(connection, PacketKind::PluginMessage, brand)
                    .await;
            }
        }

        if version >= ProtocolVersion::V1_9
            && let Some(boss_bar) = &snapshots.boss_bar
        {
            self.send_snapshot(connection, PacketKind::BossBar, boss_bar)
                .await;
        }

        if let Some(message) = &snapshots.join_message {
            self.send_snapshot(connection, PacketKind::ChatMessage, message)
                .await;
        }

        if version >= ProtocolVersion::V1_8
            && let Some(title) = &snapshots.title
        {
            if version >= ProtocolVersion::V1_17 {
                self.send_snapshot(connection, PacketKind::TitleSetTitle, &title.title)
                    .await;
                self.send_snapshot(connection, PacketKind::TitleSetSubTitle, &title.subtitle)
                    .await;
                self.send_snapshot(connection, PacketKind::TitleTimes, &title.times)
                    .await;
            } else {
                self.send_snapshot(connection, PacketKind::TitleLegacy, &title.legacy_title)
                    .await;
                self.send_snapshot(connection, PacketKind::TitleLegacy, &title.legacy_subtitle)
                    .await;
                self.send_snapshot(connection, PacketKind::TitleLegacy, &title.legacy_times)
                    .await;
            }
        }

        if version >= ProtocolVersion::V1_8
            && let Some(banner) = &snapshots.header_and_footer
        {
            self.send_snapshot(connection, PacketKind::PlayerListHeader, banner)
                .await;
        }

        if version >= ProtocolVersion::V1_20_3 {
            self.send_snapshot(
                connection,
                PacketKind::GameEvent,
                &snapshots.start_waiting_chunks,
            )
            .await;
            for chunk in &snapshots.chunks {
                self.send_snapshot(connection, PacketKind::ChunkWithLight, chunk)
                    .await;
            }
        }

        self.send_keep_alive(connection).await;
    }

    async fn send_keep_alive(&self, connection: &mut Connection) {
        if self.outgoing.state() != ConnectionState::Play {
            return;
        }
        self.send_encoded(
            connection,
            &KeepAlive {
                id: self.context.ids.next_keep_alive_id(),
            },
        )
        .await;
    }

    async fn send_status(&self, connection: &mut Connection) {
        let config = Arc::clone(&self.context.config);
        let version = self.outgoing.version();
        let protocol = config.ping.protocol.unwrap_or_else(|| {
            match self.context.forwarding_mode() == ForwardingMode::None {
                true => version.number(),
                // Behind a proxy the client's own number says nothing useful, so the
                // newest supported one is reported instead.
                false => ProtocolVersion::MAX.number(),
            }
        });

        self.send_encoded(
            connection,
            &StatusResponse {
                version_name: &limbo_text::to_legacy_string(&config.ping.version),
                protocol,
                max_players: config.max_players,
                online_players: self.context.connections.count() as i32,
                sample: &[],
                description: &config.ping.description,
            },
        )
        .await;
    }

    async fn send_registries(&self, connection: &mut Connection) {
        let snapshots = Arc::clone(&self.context.snapshots);
        let version = self.outgoing.version();

        match snapshots.split_registry_data.get(&version) {
            Some(packets) => {
                for packet in packets {
                    self.send_snapshot(connection, PacketKind::RegistryData, packet)
                        .await;
                }
            }
            None => {
                self.send_snapshot(
                    connection,
                    PacketKind::RegistryData,
                    &snapshots.registry_data,
                )
                .await;
            }
        }
    }

    /// Carries out one decision from the state machine.
    ///
    /// Returns `Some` when the connection is finished, which the caller reports and acts
    /// on rather than deciding for itself.
    async fn perform(
        &mut self,
        connection: &mut Connection,
        action: ConnectionAction,
    ) -> Option<Ending> {
        let snapshots = Arc::clone(&self.context.snapshots);

        match action {
            ConnectionAction::AdoptVersion { version } => {
                self.incoming =
                    PacketRoute::new(self.incoming.state(), PacketDirection::ServerBound, version);
                self.outgoing =
                    PacketRoute::new(self.outgoing.state(), PacketDirection::ClientBound, version);
            }
            ConnectionAction::EnterState { state } => self.enter_state(state),
            ConnectionAction::EnterOutgoingState { state } => {
                self.outgoing =
                    PacketRoute::new(state, PacketDirection::ClientBound, self.outgoing.version());
            }
            ConnectionAction::AdoptForwardedAddress { address } => {
                self.forwarded_address = Some(address);
            }
            ConnectionAction::SendStatusResponse => self.send_status(connection).await,
            ConnectionAction::SendPongAndClose { payload } => {
                let mut body = BytesMut::new();
                body.put_i64(payload);
                self.send(connection, PacketKind::StatusPing, &body).await;
                return Some(Ending::ClientClosed);
            }
            ConnectionAction::RequestForwardedPlayerInfo => {
                let message_id = self.context.ids.next_message_id();
                self.flow.expect_forwarding_reply(message_id);
                self.send_encoded(
                    connection,
                    &LoginPluginRequest {
                        message_id,
                        channel: PLAYER_INFO_CHANNEL,
                        data: &[SUPPORTED_FORWARDING_VERSION],
                    },
                )
                .await;
            }
            ConnectionAction::SendLoginSuccess => {
                self.send_snapshot(
                    connection,
                    PacketKind::LoginSuccess,
                    &snapshots.login_success,
                )
                .await;
            }
            ConnectionAction::RegisterPlayer => self.register(),
            ConnectionAction::SendBrand => {
                if let Some(brand) = &snapshots.brand {
                    self.send_snapshot(connection, PacketKind::PluginMessage, brand)
                        .await;
                }
            }
            ConnectionAction::SendKnownPacks => {
                self.send_snapshot(connection, PacketKind::KnownPacks, &snapshots.known_packs)
                    .await;
            }
            ConnectionAction::SendRegistryData => self.send_registries(connection).await,
            ConnectionAction::SendUpdateTags => {
                self.send_snapshot(connection, PacketKind::UpdateTags, &snapshots.update_tags)
                    .await;
            }
            ConnectionAction::SendFinishConfiguration => {
                self.send_snapshot(
                    connection,
                    PacketKind::FinishConfiguration,
                    &snapshots.finish_configuration,
                )
                .await;
            }
            ConnectionAction::SpawnPlayer => self.spawn_player(connection).await,
            ConnectionAction::SpawnPlayerAfterDelay => {
                tokio::time::sleep(SPAWN_DELAY).await;
                self.spawn_player(connection).await;
            }
            ConnectionAction::Disconnect { reason } => {
                self.disconnect(connection, &reason).await;
                return Some(Ending::Refused);
            }
        }

        None
    }
}

/// Reads a proxy-forwarded identity out of the handshake host field.
///
/// Both proxy formats pack it into that field separated by NUL bytes, so it is available
/// before the client has said anything else.
fn forwarded_from_handshake(context: &ServerContext, host: &str) -> Option<ForwardedFromProxy> {
    match &context.config.info_forwarding {
        InfoForwarding::Legacy => {
            parse_legacy_handshake(host)
                .ok()
                .map(|identity| ForwardedFromProxy {
                    address: identity.address,
                    uuid: identity.uuid,
                })
        }
        InfoForwarding::BungeeGuard { .. } => {
            context
                .bungee_guard
                .as_ref()?
                .verify(host)
                .ok()
                .map(|identity| ForwardedFromProxy {
                    address: identity.address,
                    uuid: identity.uuid,
                })
        }
        InfoForwarding::None | InfoForwarding::Modern { .. } => None,
    }
}

/// What a proxy vouched for in the handshake.
struct ForwardedFromProxy {
    address: String,
    uuid: Uuid,
}

/// Serves one client until it leaves, misbehaves, or the server stops.
pub async fn serve(
    stream: TcpStream,
    address: SocketAddr,
    context: Arc<ServerContext>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) {
    if let Err(error) = stream.set_nodelay(true) {
        tracing::debug!(%error, "could not disable Nagle's algorithm");
    }

    let read_timeout = context.config.read_timeout;
    let mut session = Session::new(Arc::clone(&context), address);
    let mut connection = Framed::new(stream, VarIntFrameCodec);
    let mut keep_alive = tokio::time::interval(KEEP_ALIVE_INTERVAL);
    keep_alive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    let ending = loop {
        let next = tokio::select! {
            frame = read_frame(&mut connection, read_timeout) => frame,
            _ = keep_alive.tick() => {
                session.send_keep_alive(&mut connection).await;
                continue;
            }
            _ = shutdown.recv() => break Ending::ServerStopping,
        };

        let frame = match next {
            Ok(Some(frame)) => frame,
            Ok(None) => break Ending::ClientClosed,
            Err(ending) => break ending,
        };

        if let Some(ending) = handle_frame(&mut session, &mut connection, frame).await {
            break ending;
        }
    };

    if let Some(id) = session.registered
        && let Some(player) = context.connections.remove(id)
    {
        tracing::info!("Player {} disconnected", player.username);
    }
    tracing::debug!(?ending, address = %session.reported_address(), "connection closed");
}

/// Waits for the next frame, treating silence past the configured timeout as a departure.
async fn read_frame(
    connection: &mut Connection,
    read_timeout: Option<Duration>,
) -> Result<Option<Bytes>, Ending> {
    let next = match read_timeout {
        Some(limit) => match tokio::time::timeout(limit, connection.next()).await {
            Ok(next) => next,
            Err(_elapsed) => return Err(Ending::Timeout),
        },
        None => connection.next().await,
    };

    match next {
        Some(Ok(frame)) => Ok(Some(frame)),
        Some(Err(error)) => {
            tracing::debug!(%error, "malformed frame");
            Err(Ending::Malformed)
        }
        None => Ok(None),
    }
}

/// Decodes one frame and carries out whatever it leads to.
async fn handle_frame(
    session: &mut Session,
    connection: &mut Connection,
    frame: Bytes,
) -> Option<Ending> {
    let mut payload = frame;
    let id = match payload.read_var_int() {
        Ok(id) => id,
        Err(error) => {
            tracing::debug!(%error, "frame without a packet id");
            return Some(Ending::Malformed);
        }
    };

    let packet = match ServerBoundPacket::decode(session.incoming, id, &mut payload) {
        Ok(Some(packet)) => packet,
        // An id the server has no use for. Upstream ignores these too.
        Ok(None) => return None,
        Err(error) => {
            tracing::debug!(%error, id, "could not decode a packet");
            return Some(Ending::Malformed);
        }
    };

    let extra = adopt_identity(session, &packet);
    let online = session.context.connections.count() as i32;

    let mut actions = session.flow.handle(packet, online);
    actions.extend(extra);

    for action in actions {
        if let Some(ending) = session.perform(connection, action).await {
            return Some(ending);
        }
    }
    None
}

/// Pulls the player's identity out of whichever packet carries it.
///
/// Returns the extra steps that follow, which for Velocity is the whole of login: its
/// reply is what the server was waiting for.
fn adopt_identity(session: &mut Session, packet: &ServerBoundPacket) -> Vec<ConnectionAction> {
    match packet {
        ServerBoundPacket::Handshake(handshake) => {
            match forwarded_from_handshake(&session.context, &handshake.host) {
                Some(forwarded) => {
                    session.forwarded_address = Some(forwarded.address);
                    session.flow.adopt_offline_identity(forwarded.uuid);
                    Vec::new()
                }
                None => Vec::new(),
            }
        }
        ServerBoundPacket::LoginStart(login) => {
            // Without a proxy the identity is derived from the name, as it is in
            // offline mode everywhere else.
            if session.context.forwarding_mode() != ForwardingMode::Modern {
                session
                    .flow
                    .adopt_offline_identity(offline_mode_uuid(&login.username));
            }
            Vec::new()
        }
        ServerBoundPacket::LoginPluginResponse(response) => {
            accept_velocity_reply(session, response)
        }
        _ => Vec::new(),
    }
}

fn accept_velocity_reply(
    session: &mut Session,
    response: &limbo_server::login_plugin_response::LoginPluginResponse,
) -> Vec<ConnectionAction> {
    if !session.flow.forwarding_reply_matches(response.message_id) {
        return Vec::new();
    }

    let Some(verifier) = &session.context.modern_forwarding else {
        return Vec::new();
    };
    if !response.successful {
        return vec![refusal("You need to connect with Velocity")];
    }

    match verifier.verify(&response.data) {
        Ok(profile) => session.flow.accept_forwarded_identity(
            profile.username,
            profile.identity.uuid,
            profile.identity.address,
        ),
        Err(error) => {
            tracing::debug!(%error, "rejected forwarded player info");
            vec![refusal("Can't verify forwarded player info")]
        }
    }
}

fn refusal(message: &str) -> ConnectionAction {
    let mut reason = Component::text(message);
    reason.style.color = Some(limbo_text::chat::TextColor::Named(
        limbo_text::chat::NamedColor::Red,
    ));
    ConnectionAction::Disconnect { reason }
}
