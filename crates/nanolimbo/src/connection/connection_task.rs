use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use futures_util::StreamExt;
use limbo_net::frame::VarIntFrameCodec;
use limbo_net::identity::offline_mode_uuid;
use limbo_net::time::SystemClock;
use limbo_net::traffic::{TrafficLimiter, TrafficLimits, TrafficVerdict};
use limbo_protocol::buffer::ProtocolRead;
use limbo_server::connection::ConnectionAction;
use limbo_server::connection::ForwardingMode;
use limbo_server::serverbound::ServerBoundPacket;
use limbo_text::chat::Component;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use crate::connection::{Connection, Ending, ForwardedFromProxy, Session};
use crate::di::ServerContext;

/// How often the server asks the client to prove it is still there.
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(5);

/// Builds the per-connection limiter, or nothing when limits are switched off.
///
/// Each connection gets its own: the limits are per player, and sharing one would let a
/// busy server throttle a quiet client.
fn build_limiter(context: &ServerContext) -> Option<TrafficLimiter<SystemClock>> {
    let traffic = context.config.traffic.as_ref()?;

    Some(TrafficLimiter::new(
        TrafficLimits {
            max_packet_size: traffic.max_packet_size.map(|size| size as usize),
            window: traffic.interval.unwrap_or(Duration::ZERO),
            max_packets_per_second: traffic.max_packet_rate,
            max_bytes_per_second: traffic.max_packet_bytes_rate,
        },
        SystemClock,
    ))
}

/// Applies the configured traffic limits to one frame.
///
/// Over the limit the connection is dropped without a message, as upstream does: a client
/// flooding the server is not one that will read a kick screen.
fn check_traffic(
    limiter: &mut TrafficLimiter<SystemClock>,
    session: &Session,
    size: usize,
) -> Option<Ending> {
    match limiter.check(size) {
        TrafficVerdict::Allowed => None,
        verdict => {
            tracing::info!(
                "Closed {} due to traffic limits: {verdict:?}",
                session.reported_address()
            );
            Some(Ending::TooMuchTraffic)
        }
    }
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

fn refusal(message: &str) -> ConnectionAction {
    let mut reason = Component::text(message);
    reason.style.color = Some(limbo_text::chat::TextColor::Named(
        limbo_text::chat::NamedColor::Red,
    ));
    ConnectionAction::Disconnect { reason }
}

fn accept_velocity_reply(
    session: &mut Session,
    response: &limbo_server::serverbound::LoginPluginResponse,
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

/// Pulls the player's identity out of whichever packet carries it.
///
/// Returns the extra steps that follow, which for Velocity is the whole of login: its
/// reply is what the server was waiting for.
fn adopt_identity(session: &mut Session, packet: &ServerBoundPacket) -> Vec<ConnectionAction> {
    match packet {
        ServerBoundPacket::Handshake(handshake) => {
            match ForwardedFromProxy::from_handshake(&session.context, &handshake.host) {
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

/// Serves one client until it leaves, misbehaves, or the server stops.
pub(crate) async fn serve(
    stream: TcpStream,
    address: SocketAddr,
    context: Arc<ServerContext>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) {
    if let Err(error) = stream.set_nodelay(true) {
        tracing::debug!(%error, "could not disable Nagle's algorithm");
    }

    let read_timeout = context.config.read_timeout;
    let mut limiter = build_limiter(&context);
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

        if let Some(limiter) = limiter.as_mut()
            && let Some(ending) = check_traffic(limiter, &session, frame.len())
        {
            break ending;
        }

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
