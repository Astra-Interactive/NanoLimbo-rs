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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_an_exact_match_when_a_newer_version_is_offered_then_it_is_not_claimed() {
        let rung = VersionMatch::Exactly(ProtocolVersion::V1_21_9);

        assert!(rung.matches(ProtocolVersion::V1_21_9));
        assert!(!rung.matches(ProtocolVersion::V1_21_11));
        assert!(!rung.matches(ProtocolVersion::V1_21_7));
    }

    #[test]
    fn given_a_lower_bound_when_the_release_below_it_is_offered_then_it_is_not_claimed() {
        let rung = VersionMatch::AtLeast(ProtocolVersion::V1_20);

        assert!(rung.matches(ProtocolVersion::V1_20));
        assert!(rung.matches(ProtocolVersion::V26_2));
        assert!(!rung.matches(ProtocolVersion::V1_19_4));
    }

    #[test]
    fn given_the_trailing_rung_when_any_version_is_offered_then_it_is_claimed() {
        for version in ProtocolVersion::all() {
            assert!(VersionMatch::Any.matches(version), "{version}");
        }
    }
}
