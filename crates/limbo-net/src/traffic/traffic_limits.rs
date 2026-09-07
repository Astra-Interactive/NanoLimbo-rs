use std::time::Duration;

/// The ceilings a connection's incoming traffic is held to.
///
/// A limit of `None` is not enforced. `window` is the span the two rates are measured
/// over; a window of zero switches rate tracking off entirely.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrafficLimits {
    pub max_packet_size: Option<usize>,
    pub window: Duration,
    pub max_packets_per_second: Option<f64>,
    pub max_bytes_per_second: Option<f64>,
}

impl TrafficLimits {
    /// A configured rate, or `None` when the setting disables the limit.
    ///
    /// `NaN` compares false against zero and so disables the limit, which is the safe
    /// reading of a nonsensical setting: enforcing a limit nothing can satisfy would
    /// disconnect every player.
    fn enabled_rate(value: f64) -> Option<f64> {
        (value > 0.0).then_some(value)
    }

    /// Reads the limits as `settings.yml` spells them, where any value at or below zero
    /// turns a limit off.
    pub fn from_settings(
        max_packet_size: i64,
        window: Duration,
        max_packets_per_second: f64,
        max_bytes_per_second: f64,
    ) -> Self {
        Self {
            max_packet_size: usize::try_from(max_packet_size)
                .ok()
                .filter(|size| *size > 0),
            window,
            max_packets_per_second: Self::enabled_rate(max_packets_per_second),
            max_bytes_per_second: Self::enabled_rate(max_bytes_per_second),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
