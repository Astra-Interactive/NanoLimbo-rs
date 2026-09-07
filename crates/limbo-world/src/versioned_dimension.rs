use std::collections::HashMap;

use limbo_protocol::version::ProtocolVersion;

use crate::dimension::Dimension;

/// One dimension, resolved ahead of time against every protocol version the server
/// speaks.
///
/// Ports Java's `VersionedDimension`. Where Java exposes one getter per field and looks
/// the version up again in each of them, this hands out the whole [`Dimension`] once and
/// lets the caller read the fields it needs from it.
pub struct VersionedDimension {
    key: String,
    legacy_id: i32,
    per_version: HashMap<ProtocolVersion, Dimension>,
}

impl VersionedDimension {
    pub fn new(
        key: String,
        legacy_id: i32,
        per_version: HashMap<ProtocolVersion, Dimension>,
    ) -> Self {
        Self {
            key,
            legacy_id,
            per_version,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    /// Id used before 1.16, when the wire carried a bare dimension number instead of a
    /// namespaced key.
    pub fn legacy_id(&self) -> i32 {
        self.legacy_id
    }

    pub fn for_version(&self, version: ProtocolVersion) -> Option<&Dimension> {
        self.per_version.get(&version)
    }
}
