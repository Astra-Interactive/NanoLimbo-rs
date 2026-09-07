//! Checks the offline-mode UUID against the identities the Java implementation derives.
//!
//! `UUID.nameUUIDFromBytes` is reimplemented here rather than borrowed from the `uuid`
//! crate, whose version 3 constructor hashes a namespace the Java method does not, so
//! the fixture is the only thing proving the two agree. It covers plain ASCII, accented
//! and astral-plane usernames, which is where a byte-versus-character mistake would show.

use serde_json::Value;

use limbo_net::identity::{offline_mode_uuid, parse_uuid};

const FIXTURE: &str = include_str!("../../../fixtures/uuid/offline.json");

#[test]
fn given_the_reference_usernames_when_hashed_then_every_offline_uuid_matches() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture is valid json");
    let entries = fixture
        .get("entries")
        .and_then(Value::as_array)
        .expect("fixture lists entries");

    assert!(!entries.is_empty(), "fixture must cover at least one name");

    for entry in entries {
        let username = entry
            .get("username")
            .and_then(Value::as_str)
            .expect("entry has a username");
        let expected = entry
            .get("uuid")
            .and_then(Value::as_str)
            .map(|text| parse_uuid(text).expect("fixture uuid is well formed"))
            .expect("entry has a uuid");

        assert_eq!(
            offline_mode_uuid(username),
            expected,
            "offline uuid for {username:?}"
        );
    }
}
