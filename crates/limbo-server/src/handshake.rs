use bytes::Buf;
use limbo_protocol::buffer::{PacketDecodeError, ProtocolRead};

use crate::client_intent::ClientIntent;

/// Longest host a client may claim to have connected to.
///
/// Proxies pack forwarded player data into this field separated by NUL bytes, so it runs
/// far longer than a hostname would.
const MAX_HOST_LENGTH: usize = 255;

/// The first packet of every connection, naming the protocol version and what follows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handshake {
    /// Raw protocol number, before checking whether the server supports it. Kept raw so
    /// an unsupported client can be told its own version back.
    pub protocol: i32,
    /// The address the client believes it connected to. Carries forwarded player data
    /// under BungeeCord and BungeeGuard forwarding.
    pub host: String,
    pub port: u16,
    pub intent: Option<ClientIntent>,
}

impl Handshake {
    pub fn decode<B>(buffer: &mut B) -> Result<Self, PacketDecodeError>
    where
        B: Buf + ?Sized,
    {
        Ok(Self {
            protocol: buffer.read_var_int()?,
            host: buffer.read_string(MAX_HOST_LENGTH)?,
            port: buffer.read_u16()?,
            intent: ClientIntent::from_id(buffer.read_var_int()?),
        })
    }
}
