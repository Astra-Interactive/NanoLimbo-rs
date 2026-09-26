use std::time::{Duration, Instant};

use crate::traffic::traffic_sample::TrafficSample;

/// Buckets the window is divided into, as in the Java implementation.
const BUCKET_COUNT: usize = 150;

/// A sliding window of recent traffic, divided into equal buckets.
///
/// The newest bucket is always the first: time moves the window by rotating the older
/// buckets along and clearing the ones that were skipped. Nothing here reads a clock —
/// the caller passes the moment a packet arrived — so the whole structure is pure.
#[derive(Debug, Clone)]
pub struct PacketBucket {
    resolution: Duration,
    buckets: [TrafficSample; BUCKET_COUNT],
    newest_started_at: Option<Instant>,
}

impl PacketBucket {
    /// Divides `window` into buckets.
    ///
    /// The resolution is never zero, however short the window, so that stepping the
    /// window can never divide by zero. A window of zero collapses to a single bucket
    /// that is replaced on every arrival.
    pub fn new(window: Duration) -> Self {
        let resolution = window
            .checked_div(BUCKET_COUNT as u32)
            .unwrap_or(window)
            .max(Duration::from_nanos(1));

        Self {
            resolution,
            buckets: [TrafficSample::EMPTY; BUCKET_COUNT],
            newest_started_at: None,
        }
    }

    /// How many buckets the window has moved on in `elapsed`.
    ///
    /// A gap wide enough to overflow the count is reported as a full window, which is
    /// the same outcome: everything remembered has expired.
    fn steps_in(&self, elapsed: Duration) -> usize {
        let steps = elapsed.as_nanos() / self.resolution.as_nanos();
        usize::try_from(steps).unwrap_or(BUCKET_COUNT)
    }

    /// Forgets the whole window and starts a new one at `at`.
    fn restart(&mut self, sample: TrafficSample, at: Instant) {
        self.overwrite_newest(sample, BUCKET_COUNT);
        self.newest_started_at = Some(at);
    }

    /// Writes `sample` into the newest bucket and empties the `count - 1` buckets behind
    /// it, which are the ones no packet arrived in.
    fn overwrite_newest(&mut self, sample: TrafficSample, count: usize) {
        for (offset, bucket) in self.buckets.iter_mut().take(count).enumerate() {
            *bucket = if offset == 0 {
                sample
            } else {
                TrafficSample::EMPTY
            };
        }
    }

    /// Start of the bucket `steps` on from `from`.
    ///
    /// Buckets stay on a fixed grid rather than restarting at each arrival, so a steady
    /// stream cannot drift the window out from under the rate it is measuring.
    fn grid_start(&self, from: Instant, steps: usize) -> Option<Instant> {
        let offset = self.resolution.checked_mul(u32::try_from(steps).ok()?)?;
        from.checked_add(offset)
    }

    /// Records a packet that arrived at `at`.
    pub fn record(&mut self, sample: TrafficSample, at: Instant) {
        let Some(newest_started_at) = self.newest_started_at else {
            self.restart(sample, at);
            return;
        };

        // A clock that ran backwards folds into the newest bucket instead of rewinding
        // the window, which is what the Java implementation's clamp to zero did.
        let elapsed = at.saturating_duration_since(newest_started_at);
        let steps = self.steps_in(elapsed);

        if steps == 0 {
            if let Some(newest) = self.buckets.first_mut() {
                newest.merge(sample);
            }
            return;
        }

        if steps >= BUCKET_COUNT {
            self.restart(sample, at);
            return;
        }

        self.buckets.rotate_right(steps);
        self.overwrite_newest(sample, steps);
        self.newest_started_at = Some(self.grid_start(newest_started_at, steps).unwrap_or(at));
    }

    /// Everything the window still remembers.
    pub fn totals(&self) -> TrafficSample {
        let mut total = TrafficSample::EMPTY;

        for bucket in &self.buckets {
            total.merge(*bucket);
        }

        total
    }
}
