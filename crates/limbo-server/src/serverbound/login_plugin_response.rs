use bytes::{Buf, Bytes};
use limbo_protocol::buffer::{PacketDecodeError, ProtocolRead};

/// The largest payload the server will accept in reply to a login plugin request.
const MAX_PAYLOAD_LENGTH: usize = i16::MAX as usize;

/// A client's answer to a login plugin request. Velocity's forwarded player data
/// arrives this way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginPluginResponse {
    pub message_id: i32,
    /// False when the client did not understand the channel, in which case there is no
    /// data and forwarding cannot proceed.
    pub successful: bool,
    pub data: Bytes,
}

impl LoginPluginResponse {
    pub fn decode<B>(buffer: &mut B) -> Result<Self, PacketDecodeError>
    where
        B: Buf + ?Sized,
    {
        let message_id = buffer.read_var_int()?;
        let successful = buffer.read_bool()?;

        let remaining = buffer.remaining();
        if remaining > MAX_PAYLOAD_LENGTH {
            return Err(PacketDecodeError::ByteArrayTooLong {
                actual: remaining,
                limit: MAX_PAYLOAD_LENGTH,
            });
        }

        Ok(Self {
            message_id,
            successful,
            data: buffer.copy_to_bytes(remaining),
        })
    }
}
