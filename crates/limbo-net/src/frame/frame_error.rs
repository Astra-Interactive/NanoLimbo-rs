/// Why a byte stream could not be split into frames, or a frame could not be written.
///
/// Both decode variants describe hostile or corrupt input rather than a server fault:
/// log at debug level and close the connection. [`FrameError::FrameTooLarge`] is the
/// exception — it means the server tried to send a packet no client could read.
#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("frame length prefix is wider than the 21 bits the protocol allows")]
    LengthPrefixTooWide,

    #[error("frame of {length} bytes exceeds the {limit} byte protocol maximum")]
    FrameTooLarge { length: usize, limit: usize },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
