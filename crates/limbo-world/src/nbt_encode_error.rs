/// Why a compound could not be turned into the bytes a packet carries.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NbtEncodeError {
    #[error("nbt encoder rejected the compound: {reason}")]
    Rejected { reason: String },

    /// The encoder returned fewer bytes than the header it is contractually required to
    /// write. Unreachable in practice; the alternative to reporting it is a panic.
    #[error("nbt encoder produced a root shorter than its own header")]
    MalformedRootHeader,
}
