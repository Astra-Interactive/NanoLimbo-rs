use std::time::Duration;

use crate::traffic::TrafficLimits;

#[test]
fn given_non_positive_settings_when_read_then_every_limit_is_disabled() {
    let limits = TrafficLimits::from_settings(0, Duration::from_secs(1), 0.0, -5.0);

    assert_eq!(
        limits,
        TrafficLimits {
            max_packet_size: None,
            window: Duration::from_secs(1),
            max_packets_per_second: None,
            max_bytes_per_second: None,
        }
    );
}

#[test]
fn given_a_negative_packet_size_when_read_then_the_size_limit_is_disabled() {
    let limits = TrafficLimits::from_settings(-1, Duration::from_secs(1), 1.0, 1.0);

    assert_eq!(limits.max_packet_size, None);
}

#[test]
fn given_a_rate_of_not_a_number_when_read_then_the_rate_limit_is_disabled() {
    let limits = TrafficLimits::from_settings(1, Duration::from_secs(1), f64::NAN, f64::NAN);

    assert_eq!(limits.max_packets_per_second, None);
    assert_eq!(limits.max_bytes_per_second, None);
}

#[test]
fn given_positive_settings_when_read_then_every_limit_is_kept() {
    let limits = TrafficLimits::from_settings(2048, Duration::from_secs(3), 60.0, 4096.0);

    assert_eq!(
        limits,
        TrafficLimits {
            max_packet_size: Some(2048),
            window: Duration::from_secs(3),
            max_packets_per_second: Some(60.0),
            max_bytes_per_second: Some(4096.0),
        }
    );
}
