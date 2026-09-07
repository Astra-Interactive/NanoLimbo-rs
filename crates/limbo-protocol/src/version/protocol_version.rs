use std::fmt;

use crate::version::version_descriptor::VersionDescriptor;

/// Every protocol version the server speaks, ordered by ascending protocol number.
///
/// Sorted order is a load-bearing invariant: lookups binary-search this table and
/// [`ProtocolVersion`] ordering is defined by it.
pub static SUPPORTED: &[VersionDescriptor] = &[
    VersionDescriptor::new(ProtocolVersion::V1_7_2, "1.7.2"),
    VersionDescriptor::new(ProtocolVersion::V1_7_6, "1.7.6"),
    VersionDescriptor::new(ProtocolVersion::V1_8, "1.8"),
    VersionDescriptor::new(ProtocolVersion::V1_9, "1.9"),
    VersionDescriptor::new(ProtocolVersion::V1_9_1, "1.9.1"),
    VersionDescriptor::new(ProtocolVersion::V1_9_2, "1.9.2"),
    VersionDescriptor::new(ProtocolVersion::V1_9_4, "1.9.4"),
    VersionDescriptor::new(ProtocolVersion::V1_10, "1.10"),
    VersionDescriptor::new(ProtocolVersion::V1_11, "1.11"),
    VersionDescriptor::new(ProtocolVersion::V1_11_1, "1.11.1"),
    VersionDescriptor::new(ProtocolVersion::V1_12, "1.12"),
    VersionDescriptor::new(ProtocolVersion::V1_12_1, "1.12.1"),
    VersionDescriptor::new(ProtocolVersion::V1_12_2, "1.12.2"),
    VersionDescriptor::new(ProtocolVersion::V1_13, "1.13"),
    VersionDescriptor::new(ProtocolVersion::V1_13_1, "1.13.1"),
    VersionDescriptor::new(ProtocolVersion::V1_13_2, "1.13.2"),
    VersionDescriptor::new(ProtocolVersion::V1_14, "1.14"),
    VersionDescriptor::new(ProtocolVersion::V1_14_1, "1.14.1"),
    VersionDescriptor::new(ProtocolVersion::V1_14_2, "1.14.2"),
    VersionDescriptor::new(ProtocolVersion::V1_14_3, "1.14.3"),
    VersionDescriptor::new(ProtocolVersion::V1_14_4, "1.14.4"),
    VersionDescriptor::new(ProtocolVersion::V1_15, "1.15"),
    VersionDescriptor::new(ProtocolVersion::V1_15_1, "1.15.1"),
    VersionDescriptor::new(ProtocolVersion::V1_15_2, "1.15.2"),
    VersionDescriptor::new(ProtocolVersion::V1_16, "1.16"),
    VersionDescriptor::new(ProtocolVersion::V1_16_1, "1.16.1"),
    VersionDescriptor::new(ProtocolVersion::V1_16_2, "1.16.2"),
    VersionDescriptor::new(ProtocolVersion::V1_16_3, "1.16.3"),
    VersionDescriptor::new(ProtocolVersion::V1_16_4, "1.16.4"),
    VersionDescriptor::new(ProtocolVersion::V1_17, "1.17"),
    VersionDescriptor::new(ProtocolVersion::V1_17_1, "1.17.1"),
    VersionDescriptor::new(ProtocolVersion::V1_18, "1.18"),
    VersionDescriptor::new(ProtocolVersion::V1_18_2, "1.18.2"),
    VersionDescriptor::new(ProtocolVersion::V1_19, "1.19"),
    VersionDescriptor::new(ProtocolVersion::V1_19_1, "1.19.1"),
    VersionDescriptor::new(ProtocolVersion::V1_19_3, "1.19.3"),
    VersionDescriptor::new(ProtocolVersion::V1_19_4, "1.19.4"),
    VersionDescriptor::new(ProtocolVersion::V1_20, "1.20"),
    VersionDescriptor::new(ProtocolVersion::V1_20_2, "1.20.2"),
    VersionDescriptor::new(ProtocolVersion::V1_20_3, "1.20.3"),
    VersionDescriptor::new(ProtocolVersion::V1_20_5, "1.20.5"),
    VersionDescriptor::new(ProtocolVersion::V1_21, "1.21"),
    VersionDescriptor::new(ProtocolVersion::V1_21_2, "1.21.2"),
    VersionDescriptor::new(ProtocolVersion::V1_21_4, "1.21.4"),
    VersionDescriptor::new(ProtocolVersion::V1_21_5, "1.21.5"),
    VersionDescriptor::new(ProtocolVersion::V1_21_6, "1.21.6"),
    VersionDescriptor::new(ProtocolVersion::V1_21_7, "1.21.7"),
    VersionDescriptor::new(ProtocolVersion::V1_21_9, "1.21.9"),
    VersionDescriptor::new(ProtocolVersion::V1_21_11, "1.21.11"),
    VersionDescriptor::new(ProtocolVersion::V26_1, "26.1"),
    VersionDescriptor::new(ProtocolVersion::V26_2, "26.2"),
];

/// A Minecraft protocol version the server is known to support.
///
/// The wrapped number is the value exchanged in the handshake. A value of this type is
/// always one of [`SUPPORTED`]: it can only be built from an associated constant or
/// from [`ProtocolVersion::from_number`], so an arbitrary integer cannot masquerade as
/// a supported version.
///
/// Ordering follows the protocol number, which is also release order, so version gates
/// read as plain comparisons instead of the `moreOrEqual`/`lessOrEqual` helpers the Java
/// implementation needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolVersion(i32);

impl ProtocolVersion {
    /// Minecraft 1.7.2.
    pub const V1_7_2: Self = Self(4);
    /// Minecraft 1.7.6.
    pub const V1_7_6: Self = Self(5);
    /// Minecraft 1.8.
    pub const V1_8: Self = Self(47);
    /// Minecraft 1.9.
    pub const V1_9: Self = Self(107);
    /// Minecraft 1.9.1.
    pub const V1_9_1: Self = Self(108);
    /// Minecraft 1.9.2.
    pub const V1_9_2: Self = Self(109);
    /// Minecraft 1.9.4.
    pub const V1_9_4: Self = Self(110);
    /// Minecraft 1.10.
    pub const V1_10: Self = Self(210);
    /// Minecraft 1.11.
    pub const V1_11: Self = Self(315);
    /// Minecraft 1.11.1.
    pub const V1_11_1: Self = Self(316);
    /// Minecraft 1.12.
    pub const V1_12: Self = Self(335);
    /// Minecraft 1.12.1.
    pub const V1_12_1: Self = Self(338);
    /// Minecraft 1.12.2.
    pub const V1_12_2: Self = Self(340);
    /// Minecraft 1.13.
    pub const V1_13: Self = Self(393);
    /// Minecraft 1.13.1.
    pub const V1_13_1: Self = Self(401);
    /// Minecraft 1.13.2.
    pub const V1_13_2: Self = Self(404);
    /// Minecraft 1.14.
    pub const V1_14: Self = Self(477);
    /// Minecraft 1.14.1.
    pub const V1_14_1: Self = Self(480);
    /// Minecraft 1.14.2.
    pub const V1_14_2: Self = Self(485);
    /// Minecraft 1.14.3.
    pub const V1_14_3: Self = Self(490);
    /// Minecraft 1.14.4.
    pub const V1_14_4: Self = Self(498);
    /// Minecraft 1.15.
    pub const V1_15: Self = Self(573);
    /// Minecraft 1.15.1.
    pub const V1_15_1: Self = Self(575);
    /// Minecraft 1.15.2.
    pub const V1_15_2: Self = Self(578);
    /// Minecraft 1.16.
    pub const V1_16: Self = Self(735);
    /// Minecraft 1.16.1.
    pub const V1_16_1: Self = Self(736);
    /// Minecraft 1.16.2.
    pub const V1_16_2: Self = Self(751);
    /// Minecraft 1.16.3.
    pub const V1_16_3: Self = Self(753);
    /// Minecraft 1.16.4.
    pub const V1_16_4: Self = Self(754);
    /// Minecraft 1.17.
    pub const V1_17: Self = Self(755);
    /// Minecraft 1.17.1.
    pub const V1_17_1: Self = Self(756);
    /// Minecraft 1.18.
    pub const V1_18: Self = Self(757);
    /// Minecraft 1.18.2.
    pub const V1_18_2: Self = Self(758);
    /// Minecraft 1.19.
    pub const V1_19: Self = Self(759);
    /// Minecraft 1.19.1.
    pub const V1_19_1: Self = Self(760);
    /// Minecraft 1.19.3.
    pub const V1_19_3: Self = Self(761);
    /// Minecraft 1.19.4.
    pub const V1_19_4: Self = Self(762);
    /// Minecraft 1.20.
    pub const V1_20: Self = Self(763);
    /// Minecraft 1.20.2.
    pub const V1_20_2: Self = Self(764);
    /// Minecraft 1.20.3.
    pub const V1_20_3: Self = Self(765);
    /// Minecraft 1.20.5.
    pub const V1_20_5: Self = Self(766);
    /// Minecraft 1.21.
    pub const V1_21: Self = Self(767);
    /// Minecraft 1.21.2.
    pub const V1_21_2: Self = Self(768);
    /// Minecraft 1.21.4.
    pub const V1_21_4: Self = Self(769);
    /// Minecraft 1.21.5.
    pub const V1_21_5: Self = Self(770);
    /// Minecraft 1.21.6.
    pub const V1_21_6: Self = Self(771);
    /// Minecraft 1.21.7.
    pub const V1_21_7: Self = Self(772);
    /// Minecraft 1.21.9.
    pub const V1_21_9: Self = Self(773);
    /// Minecraft 1.21.11.
    pub const V1_21_11: Self = Self(774);
    /// Minecraft 26.1.
    pub const V26_1: Self = Self(775);
    /// Minecraft 26.2.
    pub const V26_2: Self = Self(776);

    /// Oldest supported release, and the fallback layout used to reject a client
    /// whose protocol number is unknown.
    pub const MIN: Self = Self::V1_7_2;

    /// Newest supported release. Reported in the status response when info forwarding
    /// hides the client's real version.
    pub const MAX: Self = Self::V26_2;

    /// Resolves a protocol number received in a handshake.
    ///
    /// Returns `None` for snapshots, future releases and malformed input. Callers must
    /// decide what to do with an unsupported client rather than silently falling back
    /// to an arbitrary layout.
    pub fn from_number(number: i32) -> Option<Self> {
        SUPPORTED
            .binary_search_by_key(&number, |descriptor| descriptor.version().0)
            .ok()
            .and_then(|index| SUPPORTED.get(index))
            .map(VersionDescriptor::version)
    }

    /// The number carried in the handshake and reported in the status response.
    pub const fn number(self) -> i32 {
        self.0
    }

    pub fn descriptor(self) -> Option<&'static VersionDescriptor> {
        SUPPORTED
            .binary_search_by_key(&self.0, |descriptor| descriptor.version().0)
            .ok()
            .and_then(|index| SUPPORTED.get(index))
    }

    /// Release name as Mojang publishes it, for example `1.21.4`.
    pub fn display_name(self) -> Option<&'static str> {
        self.descriptor().map(VersionDescriptor::display_name)
    }

    /// Every supported version, oldest first.
    pub fn all() -> impl DoubleEndedIterator<Item = Self> + ExactSizeIterator {
        SUPPORTED.iter().map(VersionDescriptor::version)
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.display_name() {
            Some(name) => formatter.write_str(name),
            None => write!(formatter, "protocol {}", self.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_supported_table_when_scanned_then_protocol_numbers_strictly_increase() {
        for pair in SUPPORTED.windows(2) {
            let (Some(lower), Some(higher)) = (pair.first(), pair.last()) else {
                unreachable!("windows(2) always yields two elements");
            };

            assert!(
                lower.version() < higher.version(),
                "{} ({}) must sort before {} ({})",
                lower.display_name(),
                lower.version().number(),
                higher.display_name(),
                higher.version().number(),
            );
        }
    }

    #[test]
    fn given_every_supported_number_when_resolved_then_the_same_version_comes_back() {
        for descriptor in SUPPORTED {
            let resolved = ProtocolVersion::from_number(descriptor.version().number());

            assert_eq!(resolved, Some(descriptor.version()), "{descriptor:?}");
        }
    }

    #[test]
    fn given_a_number_outside_the_table_when_resolved_then_no_version_is_returned() {
        let unsupported = [
            i32::MIN,
            -1,
            0,
            6,           // gap between 1.7.6 (5) and 1.8 (47)
            777,         // one past the newest release
            0x4000_0001, // snapshots set the high bits
            i32::MAX,
        ];

        for number in unsupported {
            assert_eq!(ProtocolVersion::from_number(number), None, "{number}");
        }
    }

    #[test]
    fn given_declared_bounds_when_compared_with_the_table_then_they_match_its_ends() {
        assert_eq!(ProtocolVersion::all().next(), Some(ProtocolVersion::MIN));
        assert_eq!(
            ProtocolVersion::all().next_back(),
            Some(ProtocolVersion::MAX)
        );
    }
}
