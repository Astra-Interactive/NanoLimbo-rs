use uuid::Uuid;

use crate::identity::{UuidParseError, parse_uuid};

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
