//! Update tags is the one clientbound packet whose bytes cannot be compared against the
//! Java implementation: it collects into a `HashMap`, so the order it serializes
//! registries and tags in is an artifact of Java's hashing rather than anything the
//! protocol specifies.
//!
//! `fixtures/packets/update_tags.json` therefore records content instead — per-registry
//! counts plus a SHA-256 over registries and tags sorted by name. This test encodes the
//! packet, reads the payload back, and checks the content that comes out, which verifies
//! the encoder rather than the registry it was handed.

// `clippy.toml` sanctions `expect` and `panic` in tests, but clippy only recognises code
// inside a `#[test]` function, which the module-level helpers of an integration-test
// crate are not. Marking the file `#![cfg(test)]` would satisfy clippy and compile the
// whole suite away if that flag were ever absent, which is the one failure this suite
// must never have.
#![allow(clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use bytes::BytesMut;
use limbo_packet::ClientboundPacket;
use limbo_packet::configuration::UpdateTags;
use limbo_protocol::buffer::ProtocolRead;
use limbo_protocol::version::ProtocolVersion;
use limbo_world::UpdateTagsRegistry;
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};

/// The oldest client the server sends update tags to.
const FIRST_TAGGED_VERSION: ProtocolVersion = ProtocolVersion::V1_20_5;

/// How many versions the fixture describes, so a shrunken file fails the run.
const EXPECTED_VERSIONS: usize = 11;

fn load_fixture() -> JsonValue {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/packets/update_tags.json");
    let raw = fs::read_to_string(path).expect("the fixture must exist");

    serde_json::from_str(&raw).expect("the fixture must be valid json")
}

fn json_array<'a>(value: &'a JsonValue, key: &str) -> &'a Vec<JsonValue> {
    value
        .get(key)
        .and_then(JsonValue::as_array)
        .unwrap_or_else(|| panic!("the fixture must carry an array under {key}"))
}

fn summary_digest(entry: &JsonValue) -> &str {
    entry
        .get("digest")
        .and_then(JsonValue::as_str)
        .expect("the fixture must carry a digest")
}

fn json_i64(value: &JsonValue, key: &str) -> i64 {
    value
        .get(key)
        .and_then(JsonValue::as_i64)
        .unwrap_or_else(|| panic!("the fixture must carry a number under {key}"))
}

/// Registries and their tags, as read back off the wire and sorted the way the fixture's
/// digest recipe wants them.
type DecodedTags = BTreeMap<String, BTreeMap<String, Vec<i32>>>;

fn decode_payload(payload: &[u8]) -> DecodedTags {
    let mut cursor: &[u8] = payload;
    let mut registries = DecodedTags::new();

    let registry_count = cursor.read_var_int().expect("a registry count");
    for _ in 0..registry_count {
        let registry_key = cursor.read_string(256).expect("a registry key");
        let tag_count = cursor.read_var_int().expect("a tag count");

        let mut tags = BTreeMap::new();
        for _ in 0..tag_count {
            let tag_key = cursor.read_string(256).expect("a tag key");
            let id_count = cursor.read_var_int().expect("an id count");

            let mut entry_ids = Vec::new();
            for _ in 0..id_count {
                entry_ids.push(cursor.read_var_int().expect("an entry id"));
            }

            assert!(
                tags.insert(tag_key, entry_ids).is_none(),
                "a registry must not name the same tag twice"
            );
        }

        assert!(
            registries.insert(registry_key, tags).is_none(),
            "a payload must not name the same registry twice"
        );
    }

    assert!(cursor.is_empty(), "the payload must be fully consumed");

    registries
}

/// The digest input the fixture documents: for each registry sorted by name, its name
/// and a NUL, then for each tag sorted by name, its name, `=`, Java's `List.toString`
/// of the ids, and a NUL.
fn digest_of(registries: &DecodedTags) -> String {
    let mut canonical = String::new();

    for (registry_key, tags) in registries {
        canonical.push_str(registry_key);
        canonical.push('\0');

        for (tag_key, entry_ids) in tags {
            let rendered: Vec<String> = entry_ids.iter().map(i32::to_string).collect();
            canonical.push_str(tag_key);
            canonical.push('=');
            canonical.push('[');
            canonical.push_str(&rendered.join(", "));
            canonical.push(']');
            canonical.push('\0');
        }
    }

    let digest = Sha256::digest(canonical.as_bytes());

    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn encode_for(registry: &UpdateTagsRegistry, version: ProtocolVersion) -> Vec<u8> {
    let registries = registry
        .tags_for(version)
        .expect("every version from 1.20.5 has a tag set");

    let mut buffer = BytesMut::new();
    UpdateTags {
        registries: &registries,
    }
    .encode(&mut buffer, version)
    .expect("encoding cannot fail");

    buffer.to_vec()
}

#[test]
fn given_the_fixture_when_read_then_it_still_covers_every_version_that_receives_tags() {
    let fixture = load_fixture();
    let entries = json_array(&fixture, "entries");

    assert_eq!(entries.len(), EXPECTED_VERSIONS);

    let covered: Vec<i32> = entries
        .iter()
        .map(|entry| i32::try_from(json_i64(entry, "protocol")).expect("a protocol number"))
        .collect();
    let expected: Vec<i32> = ProtocolVersion::all()
        .filter(|version| *version >= FIRST_TAGGED_VERSION)
        .map(ProtocolVersion::number)
        .collect();

    assert_eq!(covered, expected);
}

#[test]
fn given_the_encoded_packet_when_read_back_then_its_content_matches_the_reference_digest() {
    let registry = UpdateTagsRegistry::load().expect("every embedded tag set must load");
    let fixture = load_fixture();

    for entry in json_array(&fixture, "entries") {
        let version = ProtocolVersion::from_number(
            i32::try_from(json_i64(entry, "protocol")).expect("a protocol number"),
        )
        .expect("the fixture only names supported versions");

        let decoded = decode_payload(&encode_for(&registry, version));

        assert_eq!(
            decoded.len() as i64,
            json_i64(entry, "registryCount"),
            "{version}: registry count"
        );

        let total_ids: usize = decoded
            .values()
            .flat_map(BTreeMap::values)
            .map(Vec::len)
            .sum();
        assert_eq!(
            total_ids as i64,
            json_i64(entry, "totalIds"),
            "{version}: total id count"
        );

        for summary in json_array(entry, "registries") {
            let key = summary
                .get("registry")
                .and_then(JsonValue::as_str)
                .expect("a registry name");
            let tags = decoded
                .get(key)
                .unwrap_or_else(|| panic!("{version}: the payload omits {key}"));

            assert_eq!(
                tags.len() as i64,
                json_i64(summary, "tags"),
                "{version}: tag count of {key}"
            );
            assert_eq!(
                tags.values().map(Vec::len).sum::<usize>() as i64,
                json_i64(summary, "ids"),
                "{version}: id count of {key}"
            );
        }

        assert_eq!(
            digest_of(&decoded),
            summary_digest(entry),
            "{version}: content digest"
        );
    }
}
