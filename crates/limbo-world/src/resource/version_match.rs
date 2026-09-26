use limbo_protocol::version::ProtocolVersion;

/// How one rung of a version-selection ladder claims a client's protocol version.
///
/// The Java implementation spells these ladders as `if / else if` chains that mix
/// `version.moreOrEqual(V)` with `version.equals(V)`. The two are not interchangeable:
/// turning an equality rung into a lower bound would hand a client the codec of a
/// different release, so every rung records which comparison it was written with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionMatch {
    /// Java's `version.moreOrEqual(bound)`: claims this release and everything newer
    /// that no earlier rung took.
    AtLeast(ProtocolVersion),
    /// Java's `version.equals(expected)`: claims exactly one release.
    Exactly(ProtocolVersion),
    /// The trailing `else`: claims whatever the rungs above it left over.
    Any,
}

impl VersionMatch {
    pub fn matches(self, version: ProtocolVersion) -> bool {
        match self {
            Self::AtLeast(bound) => version >= bound,
            Self::Exactly(expected) => version == expected,
            Self::Any => true,
        }
    }
}
