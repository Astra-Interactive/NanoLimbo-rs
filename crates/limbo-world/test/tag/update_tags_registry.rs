use limbo_protocol::version::ProtocolVersion;
use valence_nbt::{List, compound};

use crate::resource::ResourceSelection;
use crate::tag::update_tags_registry::{TAG_LADDER, parse_update_tags};
use crate::tag::{TagEntry, TagRegistry, UpdateTagsError, UpdateTagsRegistry};

fn selected_tag_set_name(version: ProtocolVersion) -> Option<&'static str> {
    ResourceSelection::select(TAG_LADDER, version)
        .and_then(|rung| TAG_LADDER.get(rung))
        .map(ResourceSelection::name)
}

fn tags_at(version: ProtocolVersion) -> Vec<TagRegistry> {
    UpdateTagsRegistry::load()
        .expect("every embedded tag set must load")
        .tags_for(version)
        .expect("every supported version must yield a tag set")
}

fn registry_named<'a>(tags: &'a [TagRegistry], key: &str) -> &'a TagRegistry {
    tags.iter()
        .find(|registry| registry.key() == key)
        .expect("the resource declares this registry")
}

fn tag_named<'a>(registry: &'a TagRegistry, key: &str) -> &'a TagEntry {
    registry
        .tags()
        .iter()
        .find(|tag| tag.key() == key)
        .expect("the resource declares this tag")
}

#[test]
fn given_a_protocol_version_when_a_tag_set_is_selected_then_it_matches_the_java_chain() {
    let expected = [
        (ProtocolVersion::V1_20_5, "tags_1_20_5.nbt"),
        (ProtocolVersion::V1_21, "tags_1_21.nbt"),
        (ProtocolVersion::V1_21_2, "tags_1_21_2.nbt"),
        (ProtocolVersion::V1_21_4, "tags_1_21_4.nbt"),
        (ProtocolVersion::V1_21_5, "tags_1_21_5.nbt"),
        (ProtocolVersion::V1_21_6, "tags_1_21_6.nbt"),
        (ProtocolVersion::V1_21_7, "tags_1_21_7.nbt"),
        (ProtocolVersion::V1_21_9, "tags_1_21_9.nbt"),
        (ProtocolVersion::V1_21_11, "tags_1_21_11.nbt"),
        (ProtocolVersion::V26_1, "tags_26_1.nbt"),
        (ProtocolVersion::V26_2, "tags_26_2.nbt"),
    ];

    for (version, resource) in expected {
        assert_eq!(selected_tag_set_name(version), Some(resource), "{version}");
    }
}

/// Java's trailing `else` hands the 1.20.5 set to anything older, including clients
/// that never receive the packet.
#[test]
fn given_a_version_below_1_20_5_when_a_tag_set_is_selected_then_the_oldest_one_is_used() {
    for version in [ProtocolVersion::V1_7_2, ProtocolVersion::V1_20_3] {
        assert_eq!(selected_tag_set_name(version), Some("tags_1_20_5.nbt"));
    }
}

#[test]
fn given_every_supported_version_when_tags_are_requested_then_a_tag_set_is_returned() {
    let registry = UpdateTagsRegistry::load().expect("every embedded tag set must load");

    for version in ProtocolVersion::all() {
        assert!(registry.tags_for(version).is_ok(), "{version}");
    }
}

/// Ordering is the visible difference from Java, which loses it to `HashMap`. The
/// declaration order of the resource is what a client receives here.
#[test]
fn given_the_1_21_tag_set_when_parsed_then_registries_keep_the_order_of_the_resource() {
    let tags = tags_at(ProtocolVersion::V1_21);

    let keys: Vec<&str> = tags.iter().map(TagRegistry::key).collect();

    assert_eq!(
        keys,
        vec![
            "minecraft:instrument",
            "minecraft:banner_pattern",
            "minecraft:game_event",
            "minecraft:enchantment",
            "minecraft:painting_variant",
            "minecraft:point_of_interest_type",
            "minecraft:damage_type",
            "minecraft:worldgen/biome",
            "minecraft:cat_variant",
            "minecraft:fluid",
            "minecraft:block",
            "minecraft:entity_type",
            "minecraft:item",
        ]
    );
}

#[test]
fn given_a_tag_with_members_when_parsed_then_its_entry_ids_keep_their_order() {
    let tags = tags_at(ProtocolVersion::V1_21);

    let instruments = registry_named(&tags, "minecraft:instrument");

    assert_eq!(
        tag_named(instruments, "minecraft:regular_goat_horns").entry_ids(),
        [0, 1, 2, 3]
    );
    assert_eq!(
        tag_named(instruments, "minecraft:screaming_goat_horns").entry_ids(),
        [4, 5, 6, 7]
    );
}

/// An empty NBT list has no element type, so a members-less tag looks like a list of
/// the wrong kind. Dropping it would shorten the packet the client expects.
#[test]
fn given_a_tag_with_no_members_when_parsed_then_it_survives_with_an_empty_id_list() {
    let tags = tags_at(ProtocolVersion::V1_21);

    let blocks = registry_named(&tags, "minecraft:block");

    assert!(
        tag_named(blocks, "minecraft:incorrect_for_diamond_tool")
            .entry_ids()
            .is_empty()
    );
}

#[test]
fn given_a_registry_that_is_not_a_compound_when_parsed_then_the_shape_is_reported() {
    let malformed = compound! { "minecraft:block" => 1_i32 };

    let parsed = parse_update_tags(&malformed);

    assert_eq!(
        parsed.err(),
        Some(UpdateTagsError::RegistryNotCompound {
            registry: "minecraft:block".to_owned(),
        })
    );
}

#[test]
fn given_a_tag_that_is_not_a_list_of_ints_when_parsed_then_the_shape_is_reported() {
    let malformed = compound! {
        "minecraft:block" => compound! {
            "minecraft:mineable" => List::String(vec!["stone".to_owned()]),
        },
    };

    let parsed = parse_update_tags(&malformed);

    assert_eq!(
        parsed.err(),
        Some(UpdateTagsError::TagNotIntList {
            registry: "minecraft:block".to_owned(),
            tag: "minecraft:mineable".to_owned(),
        })
    );
}
