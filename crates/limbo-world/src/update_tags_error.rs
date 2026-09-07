use limbo_protocol::version::ProtocolVersion;

/// Why a tag resource could not be flattened into the Update Tags packet's payload.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UpdateTagsError {
    #[error("no tag set is defined for {version}")]
    NoTagSet { version: ProtocolVersion },

    #[error("registry `{registry}` is not a compound")]
    RegistryNotCompound { registry: String },

    #[error("tag `{tag}` of registry `{registry}` is not a list of ints")]
    TagNotIntList { registry: String, tag: String },
}
