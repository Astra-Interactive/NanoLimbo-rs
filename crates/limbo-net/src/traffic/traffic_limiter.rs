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
