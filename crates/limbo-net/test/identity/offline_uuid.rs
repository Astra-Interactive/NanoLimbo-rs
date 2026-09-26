use uuid::{Variant, Version};

use crate::identity::offline_mode_uuid;

#[test]
fn given_a_username_when_hashed_then_the_uuid_is_name_based_and_rfc_4122() {
    let uuid = offline_mode_uuid("NanoLimbo");

    assert_eq!(uuid.get_version(), Some(Version::Md5));
    assert_eq!(uuid.get_variant(), Variant::RFC4122);
}

#[test]
fn given_the_same_username_when_hashed_twice_then_the_uuid_is_the_same() {
    assert_eq!(offline_mode_uuid("Notch"), offline_mode_uuid("Notch"));
}

#[test]
fn given_usernames_differing_only_in_case_when_hashed_then_the_uuids_differ() {
    assert_ne!(offline_mode_uuid("Notch"), offline_mode_uuid("notch"));
}
