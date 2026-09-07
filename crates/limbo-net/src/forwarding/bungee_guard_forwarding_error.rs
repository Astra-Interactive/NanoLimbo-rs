use crate::identity::UuidParseError;

/// Why a BungeeGuard forwarded handshake was turned away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BungeeGuardForwardingError {
    #[error("handshake host carries {field_count} field(s), expected 4")]
    UnexpectedFieldCount { field_count: usize },

    #[error("forwarded uuid is unreadable: {0}")]
    MalformedUuid(#[from] UuidParseError),

    #[error("forwarded properties are not valid json")]
    MalformedProperties,

    #[error("forwarded properties are not a json array")]
    PropertiesNotAnArray,

    #[error("forwarded properties carry no bungeeguard-token")]
    MissingToken,

    #[error("the bungeeguard-token is not one of the configured tokens")]
    UnknownToken,
}
