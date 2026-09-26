use crate::version::{ProtocolVersion, SUPPORTED};

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
