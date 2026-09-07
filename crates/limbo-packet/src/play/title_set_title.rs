use bytes::BufMut;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;
use limbo_text::write_component;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// The large line of a title, from 1.17 where titles split into one packet per part.
///
/// Older clients receive the same payload inside
/// [`TitleLegacy`](crate::play::TitleLegacy).
pub struct TitleSetTitle<'a> {
    pub title: &'a Component,
}

impl ClientboundPacket for TitleSetTitle<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::TitleSetTitle
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        write_component(buffer, self.title, version)?;

        Ok(())
    }
}
