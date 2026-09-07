use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;
use limbo_text::write_component;
use uuid::Uuid;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;
use crate::play::chat_position::ChatPosition;
use crate::write_bool::write_bool;

/// A message from the server, shown in chat, as a system message or on the action bar.
pub struct ChatMessage<'a> {
    pub message: &'a Component,
    pub position: ChatPosition,
    /// Who the message is attributed to, read only between 1.16 and 1.19.
    ///
    /// Java generates it with `UUID.randomUUID()`; here it is injected so the packet is
    /// reproducible.
    pub sender: Uuid,
}

impl ClientboundPacket for ChatMessage<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::ChatMessage
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        write_component(buffer, self.message, version)?;

        if version >= ProtocolVersion::V1_19_1 {
            write_bool(buffer, self.position == ChatPosition::ActionBar);
        } else if version >= ProtocolVersion::V1_19 {
            buffer.write_var_int(self.position.index());
        } else if version >= ProtocolVersion::V1_8 {
            buffer.put_u8(self.position.index() as u8);
        }

        if (ProtocolVersion::V1_16..ProtocolVersion::V1_19).contains(&version) {
            buffer.write_uuid(self.sender);
        }

        Ok(())
    }
}
