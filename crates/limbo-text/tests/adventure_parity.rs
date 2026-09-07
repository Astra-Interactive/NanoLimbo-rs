//! Checks this crate against the reference implementation across every text input the
//! configuration can carry.
//!
//! The comparison is byte-for-byte on the serialized component, for each of the four
//! profiles the client has read over the years. That is a stronger claim than it first
//! looks: matching the bytes means matching Kyori Adventure's tree compaction as well as
//! its colours and key order, since two trees that render alike still serialize
//! differently.
//!
//! Reaching it took five rules that no specification states and only the reference output
//! reveals — a leading plain-text child folds into its parent, an only child folds with
//! its style, adjacent same-styled text joins, a style equal to the inherited one is
//! dropped, and a wrapper around a single child disappears. Each is documented where it
//! is implemented.
//!
//! See `MIGRATION_PLAN.md` section 7.

use serde_json::Value;

use limbo_protocol::version::ProtocolVersion;
use limbo_text::{parse, to_json_for, to_legacy_string};

const FIXTURE: &str = include_str!("../../../fixtures/text/components.json");

/// The corpus covers every construct the shipped configuration uses plus the edge cases a
/// hand-written parser gets wrong. If it shrinks, the suite stops meaning anything.
const MINIMUM_PAIRS: usize = 100;

fn version_of(protocol: &str) -> Option<ProtocolVersion> {
    ProtocolVersion::from_number(protocol.parse().ok()?)
}

/// The corpus, or an empty list when the fixture is malformed — which the coverage floor
/// in the first test turns into a failure rather than a silent pass.
fn entries(document: &Value) -> &[Value] {
    document["entries"].as_array().map_or(&[], Vec::as_slice)
}

#[test]
fn given_every_configured_text_when_serialized_then_it_matches_the_reference_byte_for_byte() {
    let document: Value = serde_json::from_str(FIXTURE).expect("fixture must parse");
    let mut compared = 0;

    for entry in entries(&document) {
        let input = entry["input"].as_str().expect("input string");
        let component = parse(input);

        for (protocol, expected) in entry["json"].as_object().expect("json profiles") {
            let expected_json = expected.as_str().expect("profile json string");
            let version = version_of(protocol).expect("fixture protocols must be supported");

            assert_eq!(
                to_json_for(&component, version),
                expected_json,
                "\ninput:    {input:?}\nprotocol: {protocol}\n"
            );
            compared += 1;
        }
    }

    assert!(
        compared >= MINIMUM_PAIRS,
        "only {compared} input/profile pairs were compared, expected at least {MINIMUM_PAIRS}"
    );
}

#[test]
fn given_every_configured_text_when_parsed_then_its_plain_rendering_matches_the_reference() {
    let document: Value = serde_json::from_str(FIXTURE).expect("fixture must parse");

    for entry in entries(&document) {
        let input = entry["input"].as_str().expect("input string");
        let expected = entry["plain"].as_str().expect("plain string");

        assert_eq!(parse(input).to_plain_text(), expected, "input {input:?}");
    }
}

#[test]
fn given_every_configured_text_when_parsed_then_its_legacy_rendering_matches_the_reference() {
    let document: Value = serde_json::from_str(FIXTURE).expect("fixture must parse");

    for entry in entries(&document) {
        let input = entry["input"].as_str().expect("input string");
        let expected = entry["legacy"].as_str().expect("legacy string");

        assert_eq!(to_legacy_string(&parse(input)), expected, "input {input:?}");
    }
}
