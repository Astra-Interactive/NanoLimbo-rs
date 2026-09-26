use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::encoding::ClientboundPacket;
use crate::encoding::PacketEncodeError;

/// Packs a block position into the protocol's single long: 26 bits of X, 26 of Z and 12
/// of Y, each truncated to its field rather than range-checked.
///
/// Computed unsigned because the X shift moves bits into the sign position, which Java
/// wraps silently and Rust would otherwise trap on in a debug build.
pub(crate) fn encode_position(x: i64, y: i64, z: i64) -> i64 {
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
