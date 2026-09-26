use md5::{Digest, Md5};
use uuid::Uuid;

/// What Mojang's offline mode hashes together with the username.
const OFFLINE_PLAYER_PREFIX: &[u8] = b"OfflinePlayer:";

/// Version nibble of a name-based (MD5) UUID, RFC 4122 section 4.3.
const VERSION_NAME_BASED_MD5: u8 = 0x30;

/// Variant bits every RFC 4122 UUID carries.
const VARIANT_RFC_4122: u8 = 0x80;

/// Derives the UUID an offline-mode player is known by.
///
/// The port of `UUID.nameUUIDFromBytes("OfflinePlayer:" + name)`, which is a version 3
/// UUID with no namespace: the MD5 digest of the name alone, with the version and
/// variant bits overwritten. The `uuid` crate's `new_v3` cannot express it, because it
/// always hashes a namespace in front of the name.
pub fn offline_mode_uuid(username: &str) -> Uuid {
    let mut digest = Md5::new();
    digest.update(OFFLINE_PLAYER_PREFIX);
    digest.update(username.as_bytes());

    let mut bytes: [u8; 16] = digest.finalize().into();
    bytes[6] = (bytes[6] & 0x0F) | VERSION_NAME_BASED_MD5;
    bytes[8] = (bytes[8] & 0x3F) | VARIANT_RFC_4122;

    Uuid::from_bytes(bytes)
}
