use bytes::BufMut;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;
use limbo_text::write_component;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// The text above and below the tab list.
pub struct PlayerListHeader<'a> {
    pub header: &'a Component,
    pub footer: &'a Component,
}

impl ClientboundPacket for PlayerListHeader<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::PlayerListHeader
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        write_component(buffer, self.header, version)?;
        write_component(buffer, self.footer, version)?;

        Ok(())
    }
}
