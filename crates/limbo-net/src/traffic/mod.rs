//! Per-connection traffic limits.
//!
//! The port of the Java implementation's `ChannelTrafficHandler`: a maximum packet size
//! plus a sliding window over the recent packet and byte rates. [`PacketBucket`] holds
//! the window and does no I/O — it is handed the moment each packet arrived, so the
//! limits are testable without sleeping.

mod packet_bucket;
mod traffic_limiter;
mod traffic_limits;
mod traffic_sample;
mod traffic_verdict;

pub use packet_bucket::PacketBucket;
pub use traffic_limiter::TrafficLimiter;
pub use traffic_limits::TrafficLimits;
pub use traffic_sample::TrafficSample;
pub use traffic_verdict::TrafficVerdict;
