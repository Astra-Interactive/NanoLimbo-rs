/// Why an embedded dimension resource could not be turned into an NBT compound.
///
/// Both variants mean the shipped binary itself is corrupt rather than that a client
/// misbehaved, so the only sane response is to refuse to start.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResourceLoadError {
    #[error("embedded resource `{resource}` is not valid gzip: {reason}")]
    Decompress {
        resource: &'static str,
        reason: String,
    },

    #[error("embedded resource `{resource}` is not a valid nbt compound: {reason}")]
    Parse {
        resource: &'static str,
        reason: String,
    },
}
