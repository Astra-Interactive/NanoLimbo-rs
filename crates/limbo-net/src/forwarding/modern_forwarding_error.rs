use limbo_protocol::buffer::PacketDecodeError;

/// Why a modern (Velocity) forwarding payload was turned away.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModernForwardingError {
    #[error("forwarding payload of {length} byte(s) is too short to carry a signature")]
    TruncatedSignature { length: usize },

    #[error("no usable velocity secret is configured, so no forwarded info can be trusted")]
    UnusableSecretKey,

    #[error("forwarding payload was not signed with the configured secret")]
    SignatureMismatch,

    #[error("forwarding version {version} is newer than the supported version {maximum}")]
    UnsupportedVersion { version: i32, maximum: i32 },

    #[error("forwarding payload is malformed: {0}")]
    Malformed(#[from] PacketDecodeError),
}
