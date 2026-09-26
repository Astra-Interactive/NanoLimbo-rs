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
    /// how many, matched.
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
