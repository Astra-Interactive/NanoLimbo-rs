/// Why a UUID a proxy sent could not be read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum UuidParseError {
    #[error("uuid text of {length} bytes is neither 32 plain nor 36 hyphenated digits")]
    UnexpectedLength { length: usize },

    #[error("uuid text is not made of hexadecimal digits in the expected places")]
    Malformed,
}
