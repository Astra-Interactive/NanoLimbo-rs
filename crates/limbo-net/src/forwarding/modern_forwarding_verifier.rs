use std::fmt;

use hmac::{Hmac, Mac};
use limbo_protocol::buffer::ProtocolRead;
use sha2::Sha256;

use crate::forwarding::forwarded_identity::ForwardedIdentity;
use crate::forwarding::forwarded_profile::ForwardedProfile;
use crate::forwarding::modern_forwarding_error::ModernForwardingError;

/// Length of the HMAC-SHA256 signature Velocity puts in front of the payload.
pub(crate) const SIGNATURE_LENGTH: usize = 32;

/// Longest string the Java implementation would read here: `ByteMessage.readString()`
/// defaults to `Short.MAX_VALUE` characters.
const MAX_STRING_CHARS: usize = 32_767;

/// Verifies and reads the payload Velocity answers the player info request with.
///
/// The payload is `signature || version || address || uuid || username`, and everything
/// after the signature is what the signature covers. The comparison is constant time:
/// a signature check that leaks how far it matched is a signature check an attacker can
/// walk one byte at a time.
#[derive(Clone)]
pub struct ModernForwardingVerifier {
    secret: Vec<u8>,
}

impl ModernForwardingVerifier {
    /// Plugin message channel the login plugin request asks on and Velocity answers on.
    pub const PLAYER_INFO_CHANNEL: &str = "velocity:player_info";

    /// Highest forwarding version this server understands, and the one it asks for in
    /// the login plugin request.
    pub const MAX_SUPPORTED_VERSION: u8 = 1;

    pub fn new(secret: Vec<u8>) -> Self {
        Self { secret }
    }

    fn verify_signature(
        &self,
        signature: &[u8],
        signed: &[u8],
    ) -> Result<(), ModernForwardingError> {
        if self.secret.is_empty() {
            return Err(ModernForwardingError::UnusableSecretKey);
        }

        // HMAC accepts a key of any length, so this only guards against a future where
        // it does not.
        let mut mac = Hmac::<Sha256>::new_from_slice(&self.secret)
            .map_err(|_invalid_length| ModernForwardingError::UnusableSecretKey)?;
        mac.update(signed);
        mac.verify_slice(signature)
            .map_err(|_mismatch| ModernForwardingError::SignatureMismatch)
    }

    /// Checks the signature and reads the player info behind it.
    pub fn verify(&self, payload: &[u8]) -> Result<ForwardedProfile, ModernForwardingError> {
        let (signature, mut signed) = payload.split_at_checked(SIGNATURE_LENGTH).ok_or(
            ModernForwardingError::TruncatedSignature {
                length: payload.len(),
            },
        )?;

        self.verify_signature(signature, signed)?;

        let version = signed.read_var_int()?;
        let maximum = i32::from(Self::MAX_SUPPORTED_VERSION);
        if version > maximum {
            return Err(ModernForwardingError::UnsupportedVersion { version, maximum });
        }

        let address = signed.read_string(MAX_STRING_CHARS)?;
        let uuid = signed.read_uuid()?;
        let username = signed.read_string(MAX_STRING_CHARS)?;

        Ok(ForwardedProfile {
            identity: ForwardedIdentity { address, uuid },
            username,
        })
    }
}

/// Keeps the secret out of any log line that formats the verifier.
impl fmt::Debug for ModernForwardingVerifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModernForwardingVerifier")
            .field("secret", &"<redacted>")
            .finish()
    }
}
