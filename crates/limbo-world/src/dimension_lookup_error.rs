use limbo_protocol::version::ProtocolVersion;

/// Why a dimension could not be resolved against every supported protocol version.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DimensionLookupError {
    #[error("the codec sent to {version} defines no dimension `{key}`")]
    Unresolved {
        key: &'static str,
        version: ProtocolVersion,
    },
}
