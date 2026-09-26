use bytes::BytesMut;
use hmac::{Hmac, Mac};
use limbo_protocol::buffer::{PacketDecodeError, ProtocolWrite};
use sha2::Sha256;
use uuid::Uuid;

use crate::forwarding::modern_forwarding_verifier::SIGNATURE_LENGTH;
use crate::forwarding::{
    ForwardedIdentity, ForwardedProfile, ModernForwardingError, ModernForwardingVerifier,
};

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
