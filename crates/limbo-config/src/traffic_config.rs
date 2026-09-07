use std::time::Duration;

/// Limits on what a single connection may send before it is dropped.
///
/// Each limit is optional because the configuration disables one by giving it a negative
/// value; carrying that as `None` keeps the "is this switched on" question out of the
/// read loop.
#[derive(Debug, Clone, PartialEq)]
pub struct TrafficConfig {
    /// Largest accepted frame, in bytes.
    pub max_packet_size: Option<u32>,
    /// The window the two rates below are averaged over.
    pub interval: Option<Duration>,
    /// Packets per second, averaged over the interval.
    pub max_packet_rate: Option<f64>,
    /// Bytes per second, averaged over the interval.
    pub max_packet_bytes_rate: Option<f64>,
}

impl TrafficConfig {
    /// Whether either rate limit can fire.
    ///
    /// Both rates are measured over the interval, so an absent interval switches them off
    /// however they are configured — the reference implementation only allocated its
    /// counters under exactly this condition.
    pub fn measures_rates(&self) -> bool {
        self.interval.is_some()
            && (self.max_packet_rate.is_some() || self.max_packet_bytes_rate.is_some())
    }
}
