use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;
use limbo_text::to_json_for;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// Refuses a login, saying why.
///
/// The reason travels as a JSON string on every version. The login state predates the
/// NBT component form and never adopted it, so this cannot use
/// [`write_component`](limbo_text::write_component).
pub struct LoginDisconnect<'a> {
    pub reason: &'a Component,
}

impl ClientboundPacket for LoginDisconnect<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::LoginDisconnect
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.write_string(&to_json_for(self.reason, version));

        Ok(())
    }
}
