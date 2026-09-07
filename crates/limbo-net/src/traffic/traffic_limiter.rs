use std::time::Duration;

use crate::time::Clock;
use crate::traffic::packet_bucket::PacketBucket;
use crate::traffic::traffic_limits::TrafficLimits;
use crate::traffic::traffic_sample::TrafficSample;
use crate::traffic::traffic_verdict::TrafficVerdict;

/// Holds one connection to its traffic limits.
///
/// The port of the Java implementation's `ChannelTrafficHandler`, minus the I/O: it
/// answers with a [`TrafficVerdict`] and leaves closing the socket to the caller.
#[derive(Debug)]
pub struct TrafficLimiter<C> {
    limits: TrafficLimits,
    bucket: Option<PacketBucket>,
    clock: C,
}

impl<C: Clock> TrafficLimiter<C> {
    /// Builds a limiter for one connection.
    ///
    /// The window of buckets is the limiter's own state rather than an injected
    /// collaborator: it only exists when a rate limit is configured, and its resolution
    /// follows from the very limits given here. The clock, which is a collaborator, is
    /// injected.
    pub fn new(limits: TrafficLimits, clock: C) -> Self {
        let tracks_rates =
            limits.max_packets_per_second.is_some() || limits.max_bytes_per_second.is_some();
        let bucket = (tracks_rates && limits.window > Duration::ZERO)
            .then(|| PacketBucket::new(limits.window));

        Self {
            limits,
            bucket,
            clock,
        }
    }

    fn rate_per_second(count: u64, window: Duration) -> f64 {
        count as f64 / window.as_secs_f64()
    }

    /// Judges one packet, recording it against the rate window on the way.
    ///
    /// `packet_size` is the framed payload, without its length prefix, which is what the
    /// Java handler measured sitting behind the frame decoder. A packet turned away for
    /// its size is not recorded, so one oversized packet cannot also trip a rate limit.
    pub fn check(&mut self, packet_size: usize) -> TrafficVerdict {
        if let Some(limit) = self.limits.max_packet_size
            && packet_size > limit
        {
            return TrafficVerdict::PacketTooLarge {
                size: packet_size,
                limit,
            };
        }

        let Some(bucket) = self.bucket.as_mut() else {
            return TrafficVerdict::Allowed;
        };
        bucket.record(TrafficSample::single_packet(packet_size), self.clock.now());

        let totals = bucket.totals();
        let window = self.limits.window;

        if let Some(limit) = self.limits.max_packets_per_second
            && Self::rate_per_second(totals.packets, window) > limit
        {
            return TrafficVerdict::PacketRateExceeded {
                packets: totals.packets,
                window,
            };
        }

        if let Some(limit) = self.limits.max_bytes_per_second
            && Self::rate_per_second(totals.bytes, window) > limit
        {
            return TrafficVerdict::ByteRateExceeded {
                bytes: totals.bytes,
                window,
            };
        }

        TrafficVerdict::Allowed
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;
    use std::time::Instant;

    use super::*;

    const WINDOW: Duration = Duration::from_secs(1);

    /// A clock the test moves by hand, shared with the limiter that holds a copy.
    #[derive(Clone)]
    struct SteppedClock {
        now: Rc<Cell<Instant>>,
    }

    impl SteppedClock {
        fn started_now() -> Self {
            Self {
                now: Rc::new(Cell::new(Instant::now())),
            }
        }

        fn advance(&self, by: Duration) {
            self.now.set(self.now.get() + by);
        }
    }

    impl Clock for SteppedClock {
        fn now(&self) -> Instant {
            self.now.get()
        }
    }

    fn limits_of(
        max_packet_size: Option<usize>,
        max_packets_per_second: Option<f64>,
        max_bytes_per_second: Option<f64>,
    ) -> TrafficLimits {
        TrafficLimits {
            max_packet_size,
            window: WINDOW,
            max_packets_per_second,
            max_bytes_per_second,
        }
    }

    #[test]
    fn given_a_packet_over_the_size_limit_when_checked_then_it_is_rejected() {
        let mut limiter =
            TrafficLimiter::new(limits_of(Some(64), None, None), SteppedClock::started_now());

        let verdict = limiter.check(65);

        assert_eq!(
            verdict,
            TrafficVerdict::PacketTooLarge {
                size: 65,
                limit: 64
            }
        );
    }

    #[test]
    fn given_a_packet_exactly_at_the_size_limit_when_checked_then_it_is_allowed() {
        let mut limiter =
            TrafficLimiter::new(limits_of(Some(64), None, None), SteppedClock::started_now());

        assert_eq!(limiter.check(64), TrafficVerdict::Allowed);
    }

    #[test]
    fn given_no_limits_configured_when_checked_then_every_packet_is_allowed() {
        let mut limiter =
            TrafficLimiter::new(limits_of(None, None, None), SteppedClock::started_now());

        for _ in 0..1_000 {
            assert_eq!(limiter.check(4_096), TrafficVerdict::Allowed);
        }
    }

    #[test]
    fn given_more_packets_than_the_rate_allows_when_checked_then_the_burst_is_reported() {
        let clock = SteppedClock::started_now();
        let mut limiter = TrafficLimiter::new(limits_of(None, Some(5.0), None), clock.clone());

        for _ in 0..5 {
            assert_eq!(limiter.check(1), TrafficVerdict::Allowed);
            clock.advance(Duration::from_millis(1));
        }
        let verdict = limiter.check(1);

        assert_eq!(
            verdict,
            TrafficVerdict::PacketRateExceeded {
                packets: 6,
                window: WINDOW
            }
        );
    }

    #[test]
    fn given_the_window_has_passed_when_checked_then_the_connection_recovers() {
        let clock = SteppedClock::started_now();
        let mut limiter = TrafficLimiter::new(limits_of(None, Some(5.0), None), clock.clone());

        for _ in 0..6 {
            limiter.check(1);
        }
        clock.advance(WINDOW);

        assert_eq!(limiter.check(1), TrafficVerdict::Allowed);
    }

    #[test]
    fn given_more_bytes_than_the_rate_allows_when_checked_then_the_volume_is_reported() {
        let clock = SteppedClock::started_now();
        let mut limiter = TrafficLimiter::new(limits_of(None, None, Some(100.0)), clock.clone());

        assert_eq!(limiter.check(40), TrafficVerdict::Allowed);
        clock.advance(Duration::from_millis(10));
        assert_eq!(limiter.check(40), TrafficVerdict::Allowed);
        clock.advance(Duration::from_millis(10));
        let verdict = limiter.check(40);

        assert_eq!(
            verdict,
            TrafficVerdict::ByteRateExceeded {
                bytes: 120,
                window: WINDOW
            }
        );
    }

    #[test]
    fn given_only_a_byte_rate_is_configured_when_many_small_packets_arrive_then_they_pass() {
        let clock = SteppedClock::started_now();
        let mut limiter = TrafficLimiter::new(limits_of(None, None, Some(100.0)), clock.clone());

        for _ in 0..50 {
            assert_eq!(limiter.check(1), TrafficVerdict::Allowed);
            clock.advance(Duration::from_millis(1));
        }
    }

    #[test]
    fn given_a_window_of_zero_when_rates_are_configured_then_they_are_not_enforced() {
        let limits = TrafficLimits {
            max_packet_size: None,
            window: Duration::ZERO,
            max_packets_per_second: Some(1.0),
            max_bytes_per_second: Some(1.0),
        };
        let mut limiter = TrafficLimiter::new(limits, SteppedClock::started_now());

        for _ in 0..10 {
            assert_eq!(limiter.check(1_024), TrafficVerdict::Allowed);
        }
    }
}
