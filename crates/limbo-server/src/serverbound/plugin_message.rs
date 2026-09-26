use bytes::{Buf, Bytes};
use limbo_protocol::buffer::{PacketDecodeError, ProtocolRead};

/// Channel names are namespaced keys, which the protocol caps well below this.
const MAX_CHANNEL_LENGTH: usize = 256;

/// The largest payload the server will accept on a plugin channel.
const MAX_PAYLOAD_LENGTH: usize = i16::MAX as usize;

/// A message on a named channel. The server reads none of them, but must consume them
/// without losing frame alignment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMessage {
    pub channel: String,
    pub data: Bytes,
}

impl PluginMessage {
    pub fn decode<B>(buffer: &mut B) -> Result<Self, PacketDecodeError>
    where
        B: Buf + ?Sized,
    {
        let channel = buffer.read_string(MAX_CHANNEL_LENGTH)?;

        let remaining = buffer.remaining();
        if remaining > MAX_PAYLOAD_LENGTH {
            return Err(PacketDecodeError::ByteArrayTooLong {
                actual: remaining,
                limit: MAX_PAYLOAD_LENGTH,
            });
        }

        Ok(Self {
            channel,
            data: buffer.copy_to_bytes(remaining),
        })
    }
}
