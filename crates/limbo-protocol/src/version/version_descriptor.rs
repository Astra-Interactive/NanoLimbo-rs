use crate::version::protocol_version::ProtocolVersion;

/// One entry of the supported-version table: a protocol number paired with the
/// human-readable Minecraft release that introduced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionDescriptor {
    version: ProtocolVersion,
    display_name: &'static str,
}

impl VersionDescriptor {
    pub(crate) const fn new(version: ProtocolVersion, display_name: &'static str) -> Self {
        Self {
            version,
            display_name,
        }
    }

    pub const fn version(&self) -> ProtocolVersion {
        self.version
    }

    /// Release name as Mojang publishes it, for example `1.21.4`.
    ///
    /// Sent to clients verbatim in the known-packs handshake, so it is data, not a label.
    pub const fn display_name(&self) -> &'static str {
        self.display_name
    }
}
