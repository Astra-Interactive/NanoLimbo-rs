use limbo_protocol::version::ProtocolVersion;
use valence_nbt::Compound;

use crate::codec::DimensionRegistry;
use crate::codec::dimension_registry::{CODEC_LADDER, dimension_entries};
use crate::resource::ResourceSelection;

fn selected_codec_name(version: ProtocolVersion) -> Option<&'static str> {
    ResourceSelection::select(CODEC_LADDER, version)
        .and_then(|rung| CODEC_LADDER.get(rung))
        .map(ResourceSelection::name)
}

fn loaded_registry() -> DimensionRegistry {
    DimensionRegistry::load().expect("every embedded codec must load")
}

/// Every rung of the ladder, plus the release immediately below each lower bound.
/// The pairs that matter most are 1.18 against 1.18.2 and 1.19.3 against 1.19.4:
/// adjacent releases that are served different codecs.
#[test]
fn given_a_protocol_version_when_a_codec_is_selected_then_it_matches_the_java_chain() {
    let expected = [
        (ProtocolVersion::V1_7_2, "codec_1_16.nbt"),
        (ProtocolVersion::V1_15_2, "codec_1_16.nbt"),
        (ProtocolVersion::V1_16, "codec_1_16.nbt"),
        (ProtocolVersion::V1_16_1, "codec_1_16.nbt"),
        (ProtocolVersion::V1_16_2, "codec_1_16_2.nbt"),
        (ProtocolVersion::V1_16_4, "codec_1_16_2.nbt"),
        (ProtocolVersion::V1_17, "codec_1_17.nbt"),
        (ProtocolVersion::V1_18, "codec_1_17.nbt"),
        (ProtocolVersion::V1_18_2, "codec_1_18_2.nbt"),
        (ProtocolVersion::V1_19, "codec_1_19.nbt"),
        (ProtocolVersion::V1_19_1, "codec_1_19_1.nbt"),
        (ProtocolVersion::V1_19_3, "codec_1_19_1.nbt"),
        (ProtocolVersion::V1_19_4, "codec_1_19_4.nbt"),
        (ProtocolVersion::V1_20, "codec_1_20.nbt"),
        (ProtocolVersion::V1_20_2, "codec_1_20.nbt"),
        (ProtocolVersion::V1_20_3, "codec_1_20.nbt"),
        (ProtocolVersion::V1_20_5, "codec_1_20_5.nbt"),
        (ProtocolVersion::V1_21, "codec_1_21.nbt"),
        (ProtocolVersion::V1_21_2, "codec_1_21_2.nbt"),
        (ProtocolVersion::V1_21_4, "codec_1_21_4.nbt"),
        (ProtocolVersion::V1_21_5, "codec_1_21_5.nbt"),
        (ProtocolVersion::V1_21_6, "codec_1_21_6.nbt"),
        (ProtocolVersion::V1_21_7, "codec_1_21_7.nbt"),
        (ProtocolVersion::V1_21_9, "codec_1_21_9.nbt"),
        (ProtocolVersion::V1_21_11, "codec_1_21_11.nbt"),
        (ProtocolVersion::V26_1, "codec_26_1.nbt"),
        (ProtocolVersion::V26_2, "codec_26_2.nbt"),
    ];

    for (version, resource) in expected {
        assert_eq!(selected_codec_name(version), Some(resource), "{version}");
    }
}

#[test]
fn given_every_supported_version_when_a_codec_is_selected_then_one_is_always_available() {
    let registry = loaded_registry();

    for version in ProtocolVersion::all() {
        assert!(registry.codec(version).is_some(), "{version}");
    }
}

#[test]
fn given_the_1_16_codec_when_a_dimension_is_resolved_then_its_id_is_its_position_in_the_list() {
    let registry = loaded_registry();

    let nether = registry
        .find_dimension(ProtocolVersion::V1_16, "minecraft:the_nether")
        .expect("the 1.16 codec defines the nether");

    assert_eq!(nether.id(), 2);
}

/// The legacy list has no `element` wrapper, so the entry itself — `name` field and
/// all — is what a 1.16 client receives as the dimension's properties.
#[test]
fn given_the_1_16_codec_when_a_dimension_is_resolved_then_the_entry_itself_is_its_element() {
    let registry = loaded_registry();

    let overworld = registry
        .find_dimension(ProtocolVersion::V1_16, "minecraft:overworld")
        .expect("the 1.16 codec defines the overworld");

    assert!(overworld.element_codec().contains_key("name"));
    assert!(overworld.element_codec().contains_key("has_skylight"));
}

#[test]
fn given_a_codec_from_1_16_2_when_a_dimension_is_resolved_then_only_the_element_is_returned() {
    let registry = loaded_registry();

    let overworld = registry
        .find_dimension(ProtocolVersion::V1_16_2, "minecraft:overworld")
        .expect("the 1.16.2 codec defines the overworld");

    assert!(!overworld.element_codec().contains_key("name"));
    assert!(overworld.element_codec().contains_key("has_skylight"));
}

/// Java re-applies its search to the first entry when nothing matched, but the
/// search re-checks the name, so an unknown key resolves to nothing rather than
/// silently to the overworld.
#[test]
fn given_a_key_no_codec_defines_when_it_is_resolved_then_nothing_is_returned() {
    let registry = loaded_registry();

    for version in ProtocolVersion::all() {
        assert!(
            registry
                .find_dimension(version, "minecraft:no_such_dimension")
                .is_none(),
            "{version}"
        );
    }
}

#[test]
fn given_a_resolved_dimension_when_its_codec_is_read_then_it_is_the_whole_registry_codec() {
    let registry = loaded_registry();

    let end = registry
        .find_dimension(ProtocolVersion::V1_21, "minecraft:the_end")
        .expect("the 1.21 codec defines the end");

    assert_eq!(Some(end.codec()), registry.codec(ProtocolVersion::V1_21));
}

#[test]
fn given_a_codec_without_a_dimension_list_when_entries_are_read_then_none_are_found() {
    assert!(dimension_entries(&Compound::new()).is_empty());
}
