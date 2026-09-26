use limbo_protocol::version::ProtocolVersion;
use valence_nbt::Compound;
use valence_nbt::binary::{from_binary, to_binary};

use crate::codec::dimension_registry::CODEC_LADDER;
use crate::resource::ResourceSelection;
use crate::tag::update_tags_registry::TAG_LADDER;

fn every_embedded_resource() -> impl Iterator<Item = &'static ResourceSelection> {
    CODEC_LADDER.iter().chain(TAG_LADDER.iter())
}

#[test]
fn given_every_embedded_resource_when_decoded_then_it_yields_a_compound_with_an_empty_root_name() {
    for selection in every_embedded_resource() {
        let decompressed = selection
            .decompress()
            .expect("every embedded resource must be valid gzip");

        let (compound, root_name): (Compound, String) = from_binary(&mut decompressed.as_slice())
            .expect("every embedded resource must be valid nbt");

        assert_eq!(root_name, "", "{}", selection.name());
        assert!(!compound.is_empty(), "{}", selection.name());
    }
}

/// Re-encoding must reproduce the resource byte for byte. Anything less means the
/// bytes reaching a client differ from the ones the Java implementation ships, which
/// is precisely the class of difference a limbo server cannot afford.
#[test]
fn given_every_embedded_resource_when_re_encoded_then_the_bytes_are_unchanged() {
    for selection in every_embedded_resource() {
        let decompressed = selection
            .decompress()
            .expect("every embedded resource must be valid gzip");
        let (compound, root_name): (Compound, String) = from_binary(&mut decompressed.as_slice())
            .expect("every embedded resource must be valid nbt");

        let mut re_encoded = Vec::with_capacity(decompressed.len());
        to_binary(&compound, &mut re_encoded, &root_name).expect("re-encoding must succeed");

        assert_eq!(re_encoded, decompressed, "{}", selection.name());
    }
}

#[test]
fn given_the_two_ladders_when_counted_then_every_shipped_resource_is_reachable() {
    assert_eq!(CODEC_LADDER.len(), 19);
    assert_eq!(TAG_LADDER.len(), 11);
}

#[test]
fn given_a_ladder_whose_last_rung_matches_anything_when_any_version_is_offered_then_a_rung_is_found()
 {
    for version in ProtocolVersion::all() {
        assert!(
            ResourceSelection::select(CODEC_LADDER, version).is_some(),
            "no codec for {version}"
        );
        assert!(
            ResourceSelection::select(TAG_LADDER, version).is_some(),
            "no tag set for {version}"
        );
    }
}

#[test]
fn given_an_empty_ladder_when_a_version_is_offered_then_no_rung_is_found() {
    assert_eq!(ResourceSelection::select(&[], ProtocolVersion::V1_21), None);
}
