/// A packet count and the bytes those packets carried.
///
/// Used both for a single arrival and for the total held by a window of buckets, which
/// is why the counts are wider than any single packet could need.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrafficSample {
    pub packets: u64,
    pub bytes: u64,
}

impl TrafficSample {
    pub const EMPTY: Self = Self {
        packets: 0,
        bytes: 0,
    };

    /// One packet of `bytes` bytes, as read off the wire.
    pub fn single_packet(bytes: usize) -> Self {
        Self {
            packets: 1,
            bytes: u64::try_from(bytes).unwrap_or(u64::MAX),
        }
    }

    /// Adds `other` into this sample, saturating rather than wrapping.
    ///
    /// A connection would have to survive longer than the universe to reach the
    /// saturation point; saturating simply removes the last arithmetic panic from a path
    /// a client controls.
    pub fn merge(&mut self, other: Self) {
        self.packets = self.packets.saturating_add(other.packets);
        self.bytes = self.bytes.saturating_add(other.bytes);
    }
}
