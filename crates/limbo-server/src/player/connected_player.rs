use std::net::SocketAddr;

use limbo_protocol::version::ProtocolVersion;
use uuid::Uuid;

/// What the server remembers about a player who has finished logging in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectedPlayer {
    pub username: String,
    pub uuid: Uuid,
    pub address: SocketAddr,
    pub version: ProtocolVersion,
}
