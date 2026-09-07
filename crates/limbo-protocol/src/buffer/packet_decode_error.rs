/// Why a byte sequence received from a client could not be interpreted.
///
/// Every variant describes hostile or corrupt input, never a server fault, so callers
/// log at debug level and close the connection rather than treating it as an incident.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PacketDecodeError {
    #[error("expected {needed} more byte(s), only {available} left")]
    UnexpectedEndOfInput { needed: usize, available: usize },

    #[error("varint is longer than the 5 bytes the protocol allows")]
    VarIntTooWide,

    #[error("length {length} is negative")]
    NegativeLength { length: i32 },

    #[error("string of {actual} bytes exceeds the {limit} byte limit")]
    StringTooManyBytes { actual: usize, limit: usize },

    #[error("string of {actual} characters exceeds the {limit} character limit")]
    StringTooManyChars { actual: usize, limit: usize },

    #[error("string is not valid UTF-8")]
    MalformedUtf8,

    #[error("byte array of {actual} bytes exceeds the {limit} byte limit")]
    ByteArrayTooLong { actual: usize, limit: usize },
}
