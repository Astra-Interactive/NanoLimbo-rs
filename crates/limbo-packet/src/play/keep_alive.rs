use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::encoding::ClientboundPacket;
use crate::encoding::PacketEncodeError;

/// Proves the connection is alive; the client echoes the id back unchanged.
///
/// The id narrowed twice on its way through the versions, so a 1.7 client sees only the
/// low 32 bits of what a modern one sees.
pub struct KeepAlive {
    /// Java draws this from `ThreadLocalRandom` every five seconds. It arrives as a
    /// field so the connection's timer, not this crate, owns the randomness.
    pub id: i64,
}

impl ClientboundPacket for KeepAlive {
    fn kind(&self) -> PacketKind {
        PacketKind::KeepAlive
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        if version >= ProtocolVersion::V1_12_2 {
            buffer.put_i64(self.id);
        } else if version >= ProtocolVersion::V1_8 {
            buffer.write_var_int(self.id as i32);
        } else {
            buffer.put_i32(self.id as i32);
        }

        Ok(())
    }
}
