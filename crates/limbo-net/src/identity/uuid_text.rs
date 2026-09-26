use uuid::Uuid;

use crate::identity::uuid_parse_error::UuidParseError;

/// Length of the canonical `8-4-4-4-12` form.
const HYPHENATED_LENGTH: usize = 36;

/// Length of the same digits without the hyphens, as Mojang's session API returns them.
const PLAIN_LENGTH: usize = 32;

/// Reads the two UUID forms a proxy may put in a handshake.
///
/// The port of the Java implementation's `UUIDUtils.fromString`, which branched on the
/// presence of a hyphen and re-inserted the hyphens itself. This is stricter than the
/// Java original in one direction only: `java.util.UUID.fromString` also accepted groups
/// of the wrong width, such as `1-2-3-4-5`, which no proxy sends. Forms the `uuid` crate
/// accepts but Java does not, braced and URN, are rejected by the length check.
pub fn parse_uuid(text: &str) -> Result<Uuid, UuidParseError> {
    if text.len() != HYPHENATED_LENGTH && text.len() != PLAIN_LENGTH {
        return Err(UuidParseError::UnexpectedLength { length: text.len() });
    }

    Uuid::try_parse(text).map_err(|_malformed| UuidParseError::Malformed)
}
