use std::fmt;

use serde_json::Value;
use subtle::{Choice, ConstantTimeEq};

use crate::forwarding::bungee_guard_forwarding_error::BungeeGuardForwardingError;
use crate::forwarding::forwarded_identity::ForwardedIdentity;
use crate::forwarding::handshake_fields::split_forwarded_fields;
use crate::identity::parse_uuid;

/// Name of the profile property BungeeGuard hides its token in.
const TOKEN_PROPERTY: &str = "bungeeguard-token";

/// Compares two byte strings in time that does not depend on where they differ.
///
/// Lengths are compared first and cannot be hidden — they are not secret, and every
/// deployment's tokens are a fixed length anyway.
fn constant_time_equals(left: &[u8], right: &[u8]) -> Choice {
    if left.len() != right.len() {
        return Choice::from(0_u8);
    }
    left.ct_eq(right)
}

/// Checks the shared token BungeeGuard smuggles through the forwarded properties.
///
/// BungeeGuard reuses the legacy handshake layout and puts the token in the fourth
/// field, the json array of profile properties, so a client that copies the first three
/// fields still cannot get in without the token.
#[derive(Clone)]
pub struct BungeeGuardVerifier {
    tokens: Vec<String>,
}

impl BungeeGuardVerifier {
    pub fn new(tokens: Vec<String>) -> Self {
        Self { tokens }
    }

    /// Digs the token out of the forwarded property list.
    ///
    /// A property named for the token whose value is not a string is skipped rather than
    /// taken as the answer, which is what the Java implementation did by only breaking
    /// out of its loop once it had a value.
    fn find_token(properties: &Value) -> Result<&str, BungeeGuardForwardingError> {
        let entries = properties
            .as_array()
            .ok_or(BungeeGuardForwardingError::PropertiesNotAnArray)?;

        entries
            .iter()
            .filter(|entry| entry.get("name").and_then(Value::as_str) == Some(TOKEN_PROPERTY))
            .find_map(|entry| entry.get("value").and_then(Value::as_str))
            .ok_or(BungeeGuardForwardingError::MissingToken)
    }

    /// Whether any configured token matches, compared without leaking where it differs.
    ///
    /// The token is a shared secret checked against attacker-supplied input, so a
    /// byte-by-byte comparison that returns early would let a caller recover it one
    /// character at a time. Upstream uses `List.contains`, which does exactly that —
    /// even though the Velocity path beside it correctly uses `MessageDigest.isEqual`.
    /// Every configured token is examined, so the timing does not reveal which one, or
    /// how many, matched. See MIGRATION_PLAN.md section 3.1b.
    fn accepts(&self, offered: &str) -> bool {
        self.tokens
            .iter()
            .fold(Choice::from(0_u8), |matched, configured| {
                matched | constant_time_equals(configured.as_bytes(), offered.as_bytes())
            })
            .into()
    }

    /// Reads the identity out of a handshake host and accepts it only if the token
    /// travelling with it is one of the configured ones.
    pub fn verify(&self, host: &str) -> Result<ForwardedIdentity, BungeeGuardForwardingError> {
        let fields = split_forwarded_fields(host);
        let [_host, address, uuid_text, properties_text] = fields.as_slice() else {
            return Err(BungeeGuardForwardingError::UnexpectedFieldCount {
                field_count: fields.len(),
            });
        };

        let uuid = parse_uuid(uuid_text)?;
        let properties: Value = serde_json::from_str(properties_text)
            .map_err(|_invalid_json| BungeeGuardForwardingError::MalformedProperties)?;
        let token = Self::find_token(&properties)?;

        if !self.accepts(token) {
            return Err(BungeeGuardForwardingError::UnknownToken);
        }

        Ok(ForwardedIdentity {
            address: (*address).to_owned(),
            uuid,
        })
    }
}

/// Keeps the tokens, which are shared secrets, out of any log line that formats the
/// verifier.
impl fmt::Debug for BungeeGuardVerifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BungeeGuardVerifier")
            .field("configured_tokens", &self.tokens.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::identity::UuidParseError;

    use super::*;

    const PLAYER_UUID: Uuid = Uuid::from_u128(0x29c6_6bf5_7218_3158_9983_b554_b116_9e82);
    const CONFIGURED_TOKEN: &str = "s3cr3t-token";

    fn verifier() -> BungeeGuardVerifier {
        BungeeGuardVerifier::new(vec![
            "another-token".to_owned(),
            CONFIGURED_TOKEN.to_owned(),
        ])
    }

    fn host_with(properties: &str) -> String {
        format!(
            "limbo.example\u{0}198.51.100.7\u{0}29c66bf5721831589983b554b1169e82\u{0}{properties}"
        )
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
            BungeeGuardForwardingError::MalformedUuid(UuidParseError::UnexpectedLength {
                length: 4
            })
        );
    }

    #[test]
    fn given_no_tokens_configured_when_verified_then_nothing_is_accepted() {
        let error = BungeeGuardVerifier::new(Vec::new())
            .verify(&host_with(&properties_with(CONFIGURED_TOKEN)))
            .unwrap_err();

        assert_eq!(error, BungeeGuardForwardingError::UnknownToken);
    }
}
