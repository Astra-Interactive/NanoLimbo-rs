use bytes::BufMut;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// A world-level notification, of which the limbo sends one: "start waiting for chunks",
/// which is what makes a 1.20.3 client draw the world instead of the loading screen.
pub struct GameEvent {
    pub event: u8,
    pub value: f32,
}

impl ClientboundPacket for GameEvent {
    fn kind(&self) -> PacketKind {
        PacketKind::GameEvent
    }

    fn encode<B>(&self, buffer: &mut B, _version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.put_u8(self.event);
        buffer.put_f32(self.value);

        Ok(())
    }
}
