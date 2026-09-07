use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// Packs a block position into the protocol's single long: 26 bits of X, 26 of Z and 12
/// of Y, each truncated to its field rather than range-checked.
///
/// Computed unsigned because the X shift moves bits into the sign position, which Java
/// wraps silently and Rust would otherwise trap on in a debug build.
fn encode_position(x: i64, y: i64, z: i64) -> i64 {
    let packed =
        ((x as u64 & 0x3FF_FFFF) << 38) | ((z as u64 & 0x3FF_FFFF) << 12) | (y as u64 & 0xFFF);

    packed as i64
}

/// Where the compass points, and where the player respawns.
pub struct SpawnPosition<'a> {
    /// Read from 1.21.9, which lets the spawn point live in another dimension.
    pub dimension_key: &'a str,
    pub x: i64,
    pub y: i64,
    pub z: i64,
    pub yaw: f32,
    pub pitch: f32,
}

impl ClientboundPacket for SpawnPosition<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::SpawnPosition
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        if version >= ProtocolVersion::V1_21_9 {
            buffer.write_string(self.dimension_key);
        }
        buffer.put_i64(encode_position(self.x, self.y, self.z));
        buffer.put_f32(self.yaw);
        if version >= ProtocolVersion::V1_21_9 {
            buffer.put_f32(self.pitch);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_the_origin_when_packed_then_only_the_height_survives() {
        assert_eq!(encode_position(0, 400, 0), 400);
    }

    #[test]
    fn given_a_coordinate_that_reaches_the_sign_bit_when_packed_then_it_wraps_like_java() {
        // 0x2000000 is bit 25 of X, which the 38-bit shift moves onto bit 63.
        assert_eq!(encode_position(0x200_0000, 0, 0), i64::MIN);
    }

    #[test]
    fn given_negative_coordinates_when_packed_then_each_field_keeps_its_low_bits() {
        let packed = encode_position(-1, -1, -1);

        assert_eq!(packed >> 38, -1);
        assert_eq!((packed << 26) >> 38, -1);
        assert_eq!((packed << 52) >> 52, -1);
    }

    #[test]
    fn given_a_height_wider_than_twelve_bits_when_packed_then_it_is_truncated() {
        assert_eq!(encode_position(0, 0x1000, 0), 0);
    }
}
