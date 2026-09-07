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

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED: Uuid = Uuid::from_u128(0x29c6_6bf5_7218_3158_9983_b554_b116_9e82);

    #[test]
    fn given_a_hyphenated_uuid_when_parsed_then_it_is_read() {
        let parsed = parse_uuid("29c66bf5-7218-3158-9983-b554b1169e82").unwrap();

        assert_eq!(parsed, EXPECTED);
    }

    #[test]
    fn given_an_unhyphenated_uuid_when_parsed_then_it_is_read() {
        let parsed = parse_uuid("29c66bf5721831589983b554b1169e82").unwrap();

        assert_eq!(parsed, EXPECTED);
    }

    #[test]
    fn given_uppercase_digits_when_parsed_then_they_are_read() {
        let parsed = parse_uuid("29C66BF5-7218-3158-9983-B554B1169E82").unwrap();

        assert_eq!(parsed, EXPECTED);
    }

    #[test]
    fn given_text_of_the_wrong_length_when_parsed_then_the_length_is_reported() {
        assert_eq!(
            parse_uuid(""),
            Err(UuidParseError::UnexpectedLength { length: 0 })
        );
        assert_eq!(
            parse_uuid("29c66bf5721831589983b554b1169e8"),
            Err(UuidParseError::UnexpectedLength { length: 31 })
        );
        assert_eq!(
            parse_uuid("{29c66bf5-7218-3158-9983-b554b1169e82}"),
            Err(UuidParseError::UnexpectedLength { length: 38 })
        );
    }

    #[test]
    fn given_hyphens_in_the_wrong_places_when_parsed_then_it_is_malformed() {
        assert_eq!(
            parse_uuid("29c66bf5-72183158-9983-b554b1169e82-"),
            Err(UuidParseError::Malformed)
        );
    }

    #[test]
    fn given_digits_outside_hexadecimal_when_parsed_then_it_is_malformed() {
        assert_eq!(
            parse_uuid("29c66bf5721831589983b554b1169ezz"),
            Err(UuidParseError::Malformed)
        );
    }

    #[test]
    fn given_multi_byte_characters_filling_the_length_when_parsed_then_it_is_malformed() {
        let padded = "ä".repeat(18);

        assert_eq!(padded.len(), 36);
        assert_eq!(parse_uuid(&padded), Err(UuidParseError::Malformed));
    }
}
