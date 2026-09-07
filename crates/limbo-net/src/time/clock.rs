use std::time::Instant;

/// Reads the current moment.
///
/// Nothing in this crate calls [`Instant::now`] inline: rate limiting is clock-driven,
/// and a test that has to sleep to reach a bucket boundary is a test nobody runs.
pub trait Clock {
    fn now(&self) -> Instant;
}
