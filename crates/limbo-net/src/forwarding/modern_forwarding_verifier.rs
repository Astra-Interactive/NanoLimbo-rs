use std::fmt;

use hmac::{Hmac, Mac};
use limbo_protocol::buffer::ProtocolRead;
use sha2::Sha256;

use crate::forwarding::forwarded_identity::ForwardedIdentity;
use crate::forwarding::forwarded_profile::ForwardedProfile;
use crate::forwarding::modern_forwarding_error::ModernForwardingError;

/// Length of the HMAC-SHA256 signature Velocity puts in front of the payload.
const SIGNATURE_LENGTH: usize = 32;

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

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use limbo_protocol::buffer::{PacketDecodeError, ProtocolWrite};
    use uuid::Uuid;

    use super::*;

    const SECRET: &[u8] = b"forwarding-secret";
    const PLAYER_UUID: Uuid = Uuid::from_u128(0x29c6_6bf5_7218_3158_9983_b554_b116_9e82);

    /// Signature and payload taken from an independent HMAC-SHA256 implementation over
    /// `version=1, address=127.0.0.1, uuid, username=NanoLimbo`.
    const REFERENCE_PAYLOAD: &str = concat!(
        "e6e6f14154855b8a2b83d954da12d5eed096b843b7f1cc9f6993b118df40d93e",
        "01093132372e302e302e3129c66bf5721831589983b554b1169e82094e616e6f4c696d626f",
    );

    fn from_hex(text: &str) -> Vec<u8> {
        text.as_bytes()
            .chunks(2)
            .map(|pair| {
                let digits = std::str::from_utf8(pair).expect("hex is ascii");
                u8::from_str_radix(digits, 16).expect("hex digits")
            })
            .collect()
    }

    fn forwarding_data(version: i32, address: &str, username: &str) -> Vec<u8> {
        let mut data = BytesMut::new();
        data.write_var_int(version);
        data.write_string(address);
        data.write_uuid(PLAYER_UUID);
        data.write_string(username);
        data.to_vec()
    }

    fn signed_with(secret: &[u8], data: &[u8]) -> Vec<u8> {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret).expect("any key length");
        mac.update(data);

        let mut payload = mac.finalize().into_bytes().to_vec();
        payload.extend_from_slice(data);
        payload
    }

    fn verifier() -> ModernForwardingVerifier {
        ModernForwardingVerifier::new(SECRET.to_vec())
    }

    #[test]
    fn given_a_payload_signed_elsewhere_when_verified_then_the_profile_is_read() {
        let profile = verifier().verify(&from_hex(REFERENCE_PAYLOAD)).unwrap();

        assert_eq!(
            profile,
            ForwardedProfile {
                identity: ForwardedIdentity {
                    address: "127.0.0.1".to_owned(),
                    uuid: PLAYER_UUID,
                },
                username: "NanoLimbo".to_owned(),
            }
        );
    }

    #[test]
    fn given_a_payload_tampered_with_after_signing_when_verified_then_it_is_rejected() {
        let mut payload = from_hex(REFERENCE_PAYLOAD);
        let last = payload.len() - 1;
        payload[last] ^= 0x01;

        let error = verifier().verify(&payload).unwrap_err();

        assert_eq!(error, ModernForwardingError::SignatureMismatch);
    }

    #[test]
    fn given_a_payload_signed_with_another_secret_when_verified_then_it_is_rejected() {
        let data = forwarding_data(1, "127.0.0.1", "NanoLimbo");
        let payload = signed_with(b"someone-elses-secret", &data);

        let error = verifier().verify(&payload).unwrap_err();

        assert_eq!(error, ModernForwardingError::SignatureMismatch);
    }

    #[test]
    fn given_a_signature_cut_short_when_verified_then_it_is_rejected() {
        let payload = from_hex(REFERENCE_PAYLOAD);
        let truncated = payload
            .get(..SIGNATURE_LENGTH - 1)
            .expect("payload is longer");

        let error = verifier().verify(truncated).unwrap_err();

        assert_eq!(
            error,
            ModernForwardingError::TruncatedSignature {
                length: SIGNATURE_LENGTH - 1
            }
        );
    }

    #[test]
    fn given_an_empty_payload_when_verified_then_it_is_rejected() {
        let error = verifier().verify(&[]).unwrap_err();

        assert_eq!(
            error,
            ModernForwardingError::TruncatedSignature { length: 0 }
        );
    }

    #[test]
    fn given_a_forwarding_version_beyond_the_supported_one_when_verified_then_it_is_rejected() {
        let data = forwarding_data(2, "127.0.0.1", "NanoLimbo");
        let payload = signed_with(SECRET, &data);

        let error = verifier().verify(&payload).unwrap_err();

        assert_eq!(
            error,
            ModernForwardingError::UnsupportedVersion {
                version: 2,
                maximum: 1
            }
        );
    }

    #[test]
    fn given_a_signed_but_incomplete_body_when_verified_then_the_decode_error_is_reported() {
        let data = forwarding_data(1, "127.0.0.1", "NanoLimbo");
        let shortened = data.get(..data.len() - 4).expect("data is longer");
        let payload = signed_with(SECRET, shortened);

        let error = verifier().verify(&payload).unwrap_err();

        assert!(matches!(
            error,
            ModernForwardingError::Malformed(PacketDecodeError::UnexpectedEndOfInput { .. })
        ));
    }

    #[test]
    fn given_no_secret_configured_when_verified_then_nothing_is_trusted() {
        let error = ModernForwardingVerifier::new(Vec::new())
            .verify(&from_hex(REFERENCE_PAYLOAD))
            .unwrap_err();

        assert_eq!(error, ModernForwardingError::UnusableSecretKey);
    }

    #[test]
    fn given_profile_properties_after_the_username_when_verified_then_they_are_ignored() {
        let mut data = forwarding_data(1, "127.0.0.1", "NanoLimbo");
        data.extend_from_slice(&[0x01, 0x04, b'l', b'e', b'f', b't']);
        let payload = signed_with(SECRET, &data);

        let profile = verifier().verify(&payload).unwrap();

        assert_eq!(profile.username, "NanoLimbo");
    }
}
