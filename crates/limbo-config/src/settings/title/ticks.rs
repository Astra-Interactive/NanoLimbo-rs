use std::time::Duration;

/// A span of time counted in game ticks, twenty to the second.
///
/// The title timing packet carries the tick count itself, so this stays a count rather
/// than a [`Duration`]: converting in both directions would only round away what the
/// configuration said. [`Ticks::as_duration`] exists for logging and for comparing
/// against wall-clock time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticks(i32);

impl Ticks {
    const MILLIS_PER_TICK: u64 = 50;

    pub const fn new(count: i32) -> Self {
        Self(count)
    }

    /// The count as the protocol carries it, negative values included: the client reads
    /// this field as a signed integer and the reference implementation never clamped it.
    pub const fn count(self) -> i32 {
        self.0
    }

    /// The wall-clock length of this span, or `None` for a negative count.
    pub fn as_duration(self) -> Option<Duration> {
        u64::try_from(self.0)
            .ok()
            .map(|count| Duration::from_millis(count * Self::MILLIS_PER_TICK))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_one_second_of_ticks_when_converted_then_it_is_one_second() {
        assert_eq!(Ticks::new(20).as_duration(), Some(Duration::from_secs(1)));
    }

    #[test]
    fn given_a_single_tick_when_converted_then_it_is_fifty_milliseconds() {
        assert_eq!(Ticks::new(1).as_duration(), Some(Duration::from_millis(50)));
    }

    #[test]
    fn given_a_negative_count_when_converted_then_there_is_no_duration() {
        assert_eq!(Ticks::new(-1).as_duration(), None);
        assert_eq!(Ticks::new(-1).count(), -1);
    }
}
