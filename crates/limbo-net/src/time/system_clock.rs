use std::time::Instant;

use crate::time::clock::Clock;

/// The monotonic system clock.
///
/// A unit struct, so every connection task can hold its own copy without sharing state
/// or allocating.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}
