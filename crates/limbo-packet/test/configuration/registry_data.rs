use limbo_protocol::version::ProtocolVersion;
use valence_nbt::{Compound, List, compound};

use crate::configuration::RegistryData;
use crate::encoding::ClientboundPacket;

fn codec_with_one_registry() -> Compound {
    compound! {
        "minecraft:worldgen/biome" => compound! {
            "type" => "minecraft:worldgen/biome",
            "value" => List::Compound(vec![
                compound! {
                    "name" => "minecraft:plains",
                    "id" => 0_i32,
                    "element" => compound! { "has_precipitation" => 1_i8 },
                },
                compound! {
                    "name" => "minecraft:the_void",
                    "id" => 1_i32,
                },
            ]),
        },
    }
}

#[test]
fn given_a_codec_when_split_then_one_packet_per_registry_keeps_the_declared_order() {
    let codec = codec_with_one_registry();

    let packets = RegistryData::split_codec(&codec);

    match packets.first() {
        Some(RegistryData::Registry { key, entries }) => {
            assert_eq!(*key, "minecraft:worldgen/biome");
            assert_eq!(entries.len(), 2);
            assert_eq!(
                entries.first().map(|entry| entry.name),
                Some("minecraft:plains")
            );
        }
        _ => panic!("a split codec yields registry packets"),
    }
}

#[test]
fn given_an_entry_without_an_element_when_encoded_then_only_its_name_is_announced() {
    let codec = codec_with_one_registry();
    let packets = RegistryData::split_codec(&codec);
    let mut buffer = bytes::BytesMut::new();

    packets
        .first()
        .expect("one registry")
        .encode(&mut buffer, ProtocolVersion::V1_20_5)
        .expect("encoding must succeed");

    // The last entry is a bare name followed by a false "has element" flag.
    assert_eq!(buffer.last(), Some(&0x00));
}
