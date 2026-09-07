use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_world::{Dimension, VersionedDimension, write_compound};
use valence_nbt::{Compound, Value};

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// Longs a `MOTION_BLOCKING` heightmap occupies: 256 columns of nine bits, packed seven
/// to a long.
const HEIGHTMAP_WORDS: usize = 37;

/// Position of `MOTION_BLOCKING` in the client's heightmap enum, which 1.21.5 started
/// sending instead of the heightmap's name.
const MOTION_BLOCKING_HEIGHTMAP: i32 = 4;

/// Palette format id for a section that is one block type throughout.
const SINGLE_VALUE_PALETTE: u8 = 0;

/// Air, the only block the limbo's chunks contain.
const AIR_BLOCK_STATE: i32 = 0;

/// Sky and block light sections bracket the chunk, adding one below and one above.
const LIGHT_SECTIONS_MARGIN: i32 = 2;

/// The bits of an all-ones mask, as the long array a bit set serializes to.
///
/// Java builds a `BitSet` and calls `toLongArray`, which drops trailing zero words; with
/// every bit set there are none to drop, so this is the same array.
fn filled_bit_set(bits: i32) -> Vec<i64> {
    let bits = bits.max(0) as u32;
    let full_words = (bits / 64) as usize;
    let remainder = bits % 64;

    let mut words = vec![-1_i64; full_words];
    if remainder > 0 {
        words.push(((1_u64 << remainder) - 1) as i64);
    }

    words
}

/// The heightmap compound clients before 1.21.5 read: one named map under a `root` tag.
fn heightmaps_compound() -> Compound {
    let mut heightmaps = Compound::new();
    heightmaps.insert(
        "MOTION_BLOCKING",
        Value::LongArray(vec![0; HEIGHTMAP_WORDS]),
    );

    let mut root = Compound::new();
    root.insert("root", Value::Compound(heightmaps));

    root
}

/// One 16-block-tall slice of empty sky, with a single-value palette for blocks and one
/// for biomes.
fn empty_section(version: ProtocolVersion) -> Vec<u8> {
    let mut section = Vec::new();

    // Non-air block count.
    section.put_i16(0);
    if version >= ProtocolVersion::V26_1 {
        section.put_i16(0);
    }

    write_single_value_palette(&mut section, version);
    write_single_value_palette(&mut section, version);

    section
}

/// A palette holding one value, so the section needs no packed block storage at all.
///
/// 1.21.5 dropped the storage length prefix, having made the length derivable from the
/// palette; before that an empty storage still announced itself as zero longs.
fn write_single_value_palette<B>(buffer: &mut B, version: ProtocolVersion)
where
    B: BufMut + ?Sized,
{
    buffer.put_u8(SINGLE_VALUE_PALETTE);
    buffer.write_var_int(AIR_BLOCK_STATE);

    if version < ProtocolVersion::V1_21_5 {
        buffer.write_var_int(0);
    }
}

/// An empty chunk column with its light data, which is what the limbo's world is made of.
pub struct ChunkWithLight<'a> {
    pub x: i32,
    pub z: i32,
    pub dimension: &'a VersionedDimension,
}

impl ChunkWithLight<'_> {
    fn dimension_for(&self, version: ProtocolVersion) -> Result<&Dimension, PacketEncodeError> {
        self.dimension
            .for_version(version)
            .ok_or_else(|| PacketEncodeError::DimensionUnresolved {
                key: self.dimension.key().to_owned(),
                version,
            })
    }

    fn write_heightmaps<B>(
        buffer: &mut B,
        version: ProtocolVersion,
    ) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        if version < ProtocolVersion::V1_21_5 {
            write_compound(buffer, &heightmaps_compound(), version)?;
            return Ok(());
        }

        buffer.write_var_int(1);
        buffer.write_var_int(MOTION_BLOCKING_HEIGHTMAP);
        buffer.write_var_int(HEIGHTMAP_WORDS as i32);
        for _ in 0..HEIGHTMAP_WORDS {
            buffer.put_i64(0);
        }

        Ok(())
    }

    fn write_sections<B>(buffer: &mut B, version: ProtocolVersion, sections: i32)
    where
        B: BufMut + ?Sized,
    {
        let section = empty_section(version);
        let sections = sections.max(0);

        buffer.write_var_int(section.len() as i32 * sections);
        for _ in 0..sections {
            buffer.put_slice(&section);
        }
    }

    /// Light: nothing is lit and nothing is dark, so only the "block light is empty"
    /// mask carries any bits.
    fn write_light<B>(buffer: &mut B, sections: i32)
    where
        B: BufMut + ?Sized,
    {
        buffer.write_long_array(&[]);
        buffer.write_long_array(&[]);
        buffer.write_long_array(&[]);
        buffer.write_long_array(&filled_bit_set(sections + LIGHT_SECTIONS_MARGIN));

        buffer.write_var_int(0);
        buffer.write_var_int(0);
    }
}

impl ClientboundPacket for ChunkWithLight<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::ChunkWithLight
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        let sections = self.dimension_for(version)?.chunk_sections();

        buffer.put_i32(self.x);
        buffer.put_i32(self.z);

        Self::write_heightmaps(buffer, version)?;
        Self::write_sections(buffer, version, sections);

        // No block entities.
        buffer.write_var_int(0);

        Self::write_light(buffer, sections);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_mask_shorter_than_a_word_when_built_then_one_partly_filled_word_is_emitted() {
        assert_eq!(filled_bit_set(18), vec![0x3_FFFF]);
    }

    #[test]
    fn given_a_mask_of_exactly_one_word_when_built_then_no_empty_word_is_appended() {
        assert_eq!(filled_bit_set(64), vec![-1]);
    }

    #[test]
    fn given_a_mask_spanning_two_words_when_built_then_the_high_word_holds_the_remainder() {
        assert_eq!(filled_bit_set(66), vec![-1, 0b11]);
    }

    #[test]
    fn given_no_bits_when_built_then_the_mask_is_empty() {
        assert_eq!(filled_bit_set(0), Vec::<i64>::new());
    }

    #[test]
    fn given_the_release_that_dropped_the_storage_length_when_a_section_is_built_then_it_shrinks() {
        assert_eq!(empty_section(ProtocolVersion::V1_21_4).len(), 8);
        assert_eq!(empty_section(ProtocolVersion::V1_21_5).len(), 6);
        assert_eq!(empty_section(ProtocolVersion::V26_1).len(), 8);
    }
}
