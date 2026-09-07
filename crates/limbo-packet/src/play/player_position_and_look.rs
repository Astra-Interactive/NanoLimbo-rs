use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;
use crate::write_bool::write_bool;

/// Eye height 1.7 clients add to the position themselves, so the server subtracts it.
///
/// Java writes `y + 1.62F` into a double, widening the float, so the value that reaches
/// the wire is the double nearest to the 32-bit constant rather than to 1.62.
const LEGACY_EYE_HEIGHT: f64 = 1.62_f32 as f64;

/// Relative-position flag for the Y axis, the only axis the limbo leaves to the client.
const RELATIVE_Y_FLAG: u8 = 0x08;

/// Puts the player where the limbo wants them, and asks them to confirm the teleport.
pub struct PlayerPositionAndLook {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    /// Java draws this from `ThreadLocalRandom`. It arrives as a field so the packet is
    /// reproducible; the client echoes it back in its confirmation.
    pub teleport_id: i32,
}

impl PlayerPositionAndLook {
    fn encode_legacy<B>(&self, buffer: &mut B, version: ProtocolVersion)
    where
        B: BufMut + ?Sized,
    {
        buffer.put_f64(self.x);
        buffer.put_f64(if version < ProtocolVersion::V1_8 {
            self.y + LEGACY_EYE_HEIGHT
        } else {
            self.y
        });
        buffer.put_f64(self.z);
        buffer.put_f32(self.yaw);
        buffer.put_f32(self.pitch);

        if version >= ProtocolVersion::V1_8 {
            buffer.put_u8(RELATIVE_Y_FLAG);
        } else {
            // 1.7 has a bare "on ground" boolean where the flags byte later went.
            write_bool(buffer, true);
        }

        if version >= ProtocolVersion::V1_9 {
            buffer.write_var_int(self.teleport_id);
        }
        if (ProtocolVersion::V1_17..=ProtocolVersion::V1_19_3).contains(&version) {
            // Dismount vehicle.
            write_bool(buffer, false);
        }
    }

    /// 1.21.2 rebuilt the packet around a movement delta and widened the flags to an int.
    fn encode_modern<B>(&self, buffer: &mut B)
    where
        B: BufMut + ?Sized,
    {
        buffer.write_var_int(self.teleport_id);

        buffer.put_f64(self.x);
        buffer.put_f64(self.y);
        buffer.put_f64(self.z);

        buffer.put_f64(0.0);
        buffer.put_f64(0.0);
        buffer.put_f64(0.0);

        buffer.put_f32(self.yaw);
        buffer.put_f32(self.pitch);

        buffer.put_i32(i32::from(RELATIVE_Y_FLAG));
    }
}

impl ClientboundPacket for PlayerPositionAndLook {
    fn kind(&self) -> PacketKind {
        PacketKind::PlayerPositionAndLook
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        if version >= ProtocolVersion::V1_21_2 {
            self.encode_modern(buffer);
        } else {
            self.encode_legacy(buffer, version);
        }

        Ok(())
    }
}
