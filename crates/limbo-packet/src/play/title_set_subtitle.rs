use bytes::BufMut;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;
use limbo_text::write_component;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// The small line of a title, from 1.17.
pub struct TitleSetSubTitle<'a> {
    pub subtitle: &'a Component,
}

impl ClientboundPacket for TitleSetSubTitle<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::TitleSetSubTitle
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        write_component(buffer, self.subtitle, version)?;

        Ok(())
    }
}
