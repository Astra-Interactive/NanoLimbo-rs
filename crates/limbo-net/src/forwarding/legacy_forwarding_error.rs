use crate::identity::UuidParseError;

/// Why a legacy (BungeeCord) forwarded handshake was turned away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum LegacyForwardingError {
    #[error("handshake host carries {field_count} field(s), expected 3 or 4")]
    UnexpectedFieldCount { field_count: usize },

    #[error("forwarded uuid is unreadable: {0}")]
    MalformedUuid(#[from] UuidParseError),
}
