use limbo_protocol::buffer::NbtEncodeError;
use limbo_protocol::version::ProtocolVersion;

/// Why a packet could not be turned into the bytes its client expects.
///
/// Encoding is otherwise infallible: the buffer grows on demand and every value the
/// server sends was validated when it was configured. What remains are the two ways the
/// data behind a packet can be unusable for a particular client.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PacketEncodeError {
    #[error(transparent)]
    Nbt(#[from] NbtEncodeError),

    /// The world was never resolved against this client's version, so there is no codec,
    /// dimension id or build height to send it.
    #[error("dimension {key} is not resolved for protocol {version}")]
    DimensionUnresolved {
        key: String,
        version: ProtocolVersion,
    },
}
