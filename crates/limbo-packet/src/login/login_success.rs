use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use uuid::Uuid;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;
use crate::write_bool::write_bool;

/// The last packet of the login state: the identity the client will play under.
pub struct LoginSuccess<'a> {
    /// Offline-mode uuid, or the one the proxy forwarded.
    pub uuid: Uuid,
    pub username: &'a str,
    /// Chat session the client signs messages under, read from 26.2.
    ///
    /// Java generates this with `UUID.randomUUID()`; here it is injected so the packet
    /// encodes to the same bytes twice.
    pub session_id: Uuid,
}

impl ClientboundPacket for LoginSuccess<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::LoginSuccess
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        if version >= ProtocolVersion::V1_16 {
            buffer.write_uuid(self.uuid);
        } else if version >= ProtocolVersion::V1_7_6 {
            buffer.write_string(&self.uuid.hyphenated().to_string());
        } else {
            // 1.7.2 reads the uuid without its hyphens.
            buffer.write_string(&self.uuid.simple().to_string());
        }

        buffer.write_string(self.username);

        if version >= ProtocolVersion::V1_19 {
            // Signed profile properties, of which the limbo has none.
            buffer.write_var_int(0);
        }
        // Strict error handling, asked for only by the releases that had the field.
        if (ProtocolVersion::V1_20_5..=ProtocolVersion::V1_21).contains(&version) {
            write_bool(buffer, true);
        }
        if version >= ProtocolVersion::V26_2 {
            buffer.write_uuid(self.session_id);
        }

        Ok(())
    }
}
