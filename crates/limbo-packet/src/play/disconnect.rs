use bytes::BufMut;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;
use limbo_text::write_component;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// Ends a play session, saying why.
///
/// Unlike its login-state counterpart the reason follows the component encoding of the
/// day, so it becomes NBT from 1.20.3.
pub struct Disconnect<'a> {
    pub reason: &'a Component,
}

impl ClientboundPacket for Disconnect<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::Disconnect
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        write_component(buffer, self.reason, version)?;

        Ok(())
    }
}
