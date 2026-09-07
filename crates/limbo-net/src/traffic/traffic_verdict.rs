use std::time::Duration;

/// What the traffic limiter decided about one incoming packet.
///
/// The rejection variants carry what the Java implementation logged, so the connection
/// task can report why it is closing the socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficVerdict {
    Allowed,

    PacketTooLarge {
        size: usize,
        limit: usize,
    },

    /// More packets arrived in `window` than the configured rate allows.
    PacketRateExceeded {
        packets: u64,
        window: Duration,
    },

    /// More bytes arrived in `window` than the configured rate allows.
    ByteRateExceeded {
        bytes: u64,
        window: Duration,
    },
}
