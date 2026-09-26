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
