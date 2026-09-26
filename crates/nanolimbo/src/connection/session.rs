use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bytes::{BufMut, Bytes, BytesMut};
use futures_util::SinkExt;
use limbo_net::frame::VarIntFrameCodec;
use limbo_packet::login::{LoginDisconnect, LoginPluginRequest};
use limbo_packet::play::{Disconnect, KeepAlive};
use limbo_packet::status::StatusResponse;
use limbo_packet::{ClientboundPacket, PreEncodedPacket};
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::{ConnectionState, PacketDirection, PacketKind, PacketRoute};
use limbo_protocol::version::ProtocolVersion;
use limbo_server::connection::ConnectionAction;
use limbo_server::connection::ConnectionFlow;
use limbo_server::connection::ForwardingMode;
use limbo_server::id::IdSource;
use limbo_server::player::ConnectedPlayer;
use limbo_server::player::ConnectionId;
use limbo_text::chat::Component;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::connection::Ending;
use crate::di::ServerContext;

pub(crate) type Connection = Framed<TcpStream, VarIntFrameCodec>;

/// One client's connection: where it is in the protocol, and what the server knows about
/// the player behind it.
pub(crate) struct Session {
    pub(crate) context: Arc<ServerContext>,
    pub(crate) flow: ConnectionFlow,
    address: SocketAddr,
    pub(crate) incoming: PacketRoute,
    outgoing: PacketRoute,
    pub(crate) registered: Option<ConnectionId>,
    pub(crate) forwarded_address: Option<String>,
}

impl Session {
    /// Channel Velocity forwards the player's identity over.
    const PLAYER_INFO_CHANNEL: &str = "velocity:player_info";

    /// Clients up to 1.7.6 drop packets arriving with the join game packet, so theirs wait.
    const SPAWN_DELAY: Duration = Duration::from_millis(100);

    /// Version of Velocity's forwarding format this server understands.
    const SUPPORTED_FORWARDING_VERSION: u8 = 1;

    /// Prefixes a payload with the id it travels under on this route.
    fn frame_for(route: PacketRoute, kind: PacketKind, payload: &[u8]) -> Option<Bytes> {
        let id = route.id_of(kind)?;
        let mut framed = BytesMut::with_capacity(payload.len() + 5);
        framed.write_var_int(id);
        framed.extend_from_slice(payload);
        Some(framed.freeze())
    }

    pub(crate) fn new(context: Arc<ServerContext>, address: SocketAddr) -> Self {
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
    pub(crate) fn reported_address(&self) -> String {
        if !self.context.config.log_players_ip {
            return "<redacted>".to_owned();
        }
        match &self.forwarded_address {
            Some(forwarded) => forwarded.clone(),
            None => self.address.to_string(),
        }
    }

    async fn send(&self, connection: &mut Connection, kind: PacketKind, payload: &[u8]) {
        let Some(frame) = Self::frame_for(self.outgoing, kind, payload) else {
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

    pub(crate) async fn send_keep_alive(&self, connection: &mut Connection) {
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
    pub(crate) async fn perform(
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
                        channel: Self::PLAYER_INFO_CHANNEL,
                        data: &[Self::SUPPORTED_FORWARDING_VERSION],
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
                tokio::time::sleep(Self::SPAWN_DELAY).await;
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
