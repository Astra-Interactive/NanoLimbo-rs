use uuid::Uuid;

use crate::forwarding::{BungeeGuardForwardingError, BungeeGuardVerifier, ForwardedIdentity};
use crate::identity::UuidParseError;

const PLAYER_UUID: Uuid = Uuid::from_u128(0x29c6_6bf5_7218_3158_9983_b554_b116_9e82);
const CONFIGURED_TOKEN: &str = "s3cr3t-token";

fn verifier() -> BungeeGuardVerifier {
    BungeeGuardVerifier::new(vec![
        "another-token".to_owned(),
        CONFIGURED_TOKEN.to_owned(),
    ])
}

fn host_with(properties: &str) -> String {
    format!("limbo.example\u{0}198.51.100.7\u{0}29c66bf5721831589983b554b1169e82\u{0}{properties}")
}

fn properties_with(token: &str) -> String {
    format!("[{{\"name\":\"bungeeguard-token\",\"value\":\"{token}\"}}]")
}

#[test]
fn given_a_configured_token_when_verified_then_the_identity_is_read() {
    let identity = verifier()
        .verify(&host_with(&properties_with(CONFIGURED_TOKEN)))
        .unwrap();

    assert_eq!(
        identity,
        ForwardedIdentity {
            address: "198.51.100.7".to_owned(),
            uuid: PLAYER_UUID,
        }
    );
}

#[test]
fn given_the_token_among_other_properties_when_verified_then_it_is_still_found() {
    let properties = format!(
        "[{{\"name\":\"textures\",\"value\":\"...\"}},{{\"name\":\"bungeeguard-token\",\"value\":\"{CONFIGURED_TOKEN}\"}}]"
    );

    assert!(verifier().verify(&host_with(&properties)).is_ok());
}

#[test]
fn given_an_unknown_token_when_verified_then_the_connection_is_turned_away() {
    let error = verifier()
        .verify(&host_with(&properties_with("forged")))
        .unwrap_err();

    assert_eq!(error, BungeeGuardForwardingError::UnknownToken);
}

#[test]
fn given_no_token_property_when_verified_then_the_connection_is_turned_away() {
    let error = verifier()
        .verify(&host_with("[{\"name\":\"textures\",\"value\":\"...\"}]"))
        .unwrap_err();

    assert_eq!(error, BungeeGuardForwardingError::MissingToken);
}

#[test]
fn given_a_token_property_without_a_string_value_when_verified_then_a_later_one_still_counts() {
    let properties = format!(
        "[{{\"name\":\"bungeeguard-token\"}},{{\"name\":\"bungeeguard-token\",\"value\":\"{CONFIGURED_TOKEN}\"}}]"
    );

    assert!(verifier().verify(&host_with(&properties)).is_ok());
}

#[test]
fn given_properties_that_are_not_json_when_verified_then_the_connection_is_turned_away() {
    let error = verifier()
        .verify(&host_with("not json at all"))
        .unwrap_err();

    assert_eq!(error, BungeeGuardForwardingError::MalformedProperties);
}

#[test]
fn given_properties_that_are_not_an_array_when_verified_then_the_connection_is_turned_away() {
    let error = verifier()
        .verify(&host_with("{\"name\":\"bungeeguard-token\"}"))
        .unwrap_err();

    assert_eq!(error, BungeeGuardForwardingError::PropertiesNotAnArray);
}

#[test]
fn given_a_handshake_without_the_properties_field_when_verified_then_it_is_turned_away() {
    let error = verifier()
        .verify("limbo.example\u{0}198.51.100.7\u{0}29c66bf5721831589983b554b1169e82")
        .unwrap_err();

    assert_eq!(
        error,
        BungeeGuardForwardingError::UnexpectedFieldCount { field_count: 3 }
    );
}

#[test]
fn given_a_malformed_uuid_when_verified_then_the_reason_is_reported() {
    let host = format!(
        "limbo.example\u{0}198.51.100.7\u{0}nope\u{0}{}",
        properties_with(CONFIGURED_TOKEN)
    );

    let error = verifier().verify(&host).unwrap_err();

    assert_eq!(
        error,
        BungeeGuardForwardingError::MalformedUuid(UuidParseError::UnexpectedLength { length: 4 })
    );
}

#[test]
fn given_no_tokens_configured_when_verified_then_nothing_is_accepted() {
    let error = BungeeGuardVerifier::new(Vec::new())
        .verify(&host_with(&properties_with(CONFIGURED_TOKEN)))
        .unwrap_err();

    assert_eq!(error, BungeeGuardForwardingError::UnknownToken);
}
