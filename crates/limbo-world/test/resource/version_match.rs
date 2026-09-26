use limbo_protocol::version::ProtocolVersion;

use crate::resource::VersionMatch;

#[test]
fn given_an_exact_match_when_a_newer_version_is_offered_then_it_is_not_claimed() {
    let rung = VersionMatch::Exactly(ProtocolVersion::V1_21_9);

    assert!(rung.matches(ProtocolVersion::V1_21_9));
    assert!(!rung.matches(ProtocolVersion::V1_21_11));
    assert!(!rung.matches(ProtocolVersion::V1_21_7));
}

#[test]
fn given_a_lower_bound_when_the_release_below_it_is_offered_then_it_is_not_claimed() {
    let rung = VersionMatch::AtLeast(ProtocolVersion::V1_20);

    assert!(rung.matches(ProtocolVersion::V1_20));
    assert!(rung.matches(ProtocolVersion::V26_2));
    assert!(!rung.matches(ProtocolVersion::V1_19_4));
}

#[test]
fn given_the_trailing_rung_when_any_version_is_offered_then_it_is_claimed() {
    for version in ProtocolVersion::all() {
        assert!(VersionMatch::Any.matches(version), "{version}");
    }
}
