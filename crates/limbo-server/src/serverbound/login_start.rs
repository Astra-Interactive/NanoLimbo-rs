use bytes::Buf;
use limbo_protocol::buffer::{PacketDecodeError, ProtocolRead};
use limbo_protocol::version::ProtocolVersion;
use uuid::Uuid;

/// Usernames have been capped at sixteen characters since the beginning.
const MAX_USERNAME_LENGTH: usize = 16;

/// Sizes the chat-signing key blob was capped at while it existed.
const MAX_PUBLIC_KEY_LENGTH: usize = 512;
const MAX_KEY_SIGNATURE_LENGTH: usize = 4096;

/// The client asking to join, naming itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginStart {
    pub username: String,
    /// Present from 1.19.1. The server does not trust it — an offline-mode identity is
    /// derived from the username instead, and a proxy overrides both.
    pub uuid: Option<Uuid>,
}

impl LoginStart {
    /// Reads and discards the chat-signing key 1.19 and 1.19.1 clients send.
    ///
    /// A limbo server verifies no chat, but the bytes still have to be consumed or every
    /// field after them is misread.
    fn skip_public_key<B>(buffer: &mut B) -> Result<(), PacketDecodeError>
    where
        B: Buf + ?Sized,
    {
        if !buffer.read_bool()? {
            return Ok(());
        }
        let _expiry = buffer.read_i64()?;
        buffer.read_byte_array(MAX_PUBLIC_KEY_LENGTH)?;
        buffer.read_byte_array(MAX_KEY_SIGNATURE_LENGTH)?;
        Ok(())
    }

    pub fn decode<B>(buffer: &mut B, version: ProtocolVersion) -> Result<Self, PacketDecodeError>
    where
        B: Buf + ?Sized,
    {
        let username = buffer.read_string(MAX_USERNAME_LENGTH)?;

        if (ProtocolVersion::V1_19..=ProtocolVersion::V1_19_1).contains(&version) {
            Self::skip_public_key(buffer)?;
        }

        let uuid = if version >= ProtocolVersion::V1_19_1 {
            // From 1.20.2 the field is unconditional; before that a flag precedes it.
            if version >= ProtocolVersion::V1_20_2 || buffer.read_bool()? {
                Some(buffer.read_uuid()?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self { username, uuid })
    }
}
