//! The network layer between a TCP stream and the connection state machine.
//!
//! Four concerns live here, all of them ports of the Java implementation's Netty
//! pipeline and forwarding utilities:
//!
//! - [`frame`] turns a byte stream into length-prefixed frames and back.
//! - [`traffic`] decides whether a connection is sending too much, too fast.
//! - [`forwarding`] recovers the player's real identity from a proxy.
//! - [`identity`] derives the offline-mode UUID and reads the UUID text forms.
//!
//! Everything a client sends is hostile until proven otherwise: no path reachable from
//! network input indexes, slices or unwraps. Time is read through the [`time::Clock`]
//! port so the rate limiter is testable without sleeping.

pub mod forwarding {
    //! Player info forwarding: how a proxy tells the limbo who the player really is.
    //!
    //! Three schemes, ported from the Java implementation's `PacketHandler` and
    //! `ForwardingUtils`:
    //!
    //! - legacy (BungeeCord), which appends the fields to the handshake host;
    //! - BungeeGuard, which adds a shared token to those fields;
    //! - modern (Velocity), which signs a payload with HMAC-SHA256.
    //!
    //! This is the code that decides who gets in, so every way a connection can be turned
    //! away is its own error variant rather than a bare `false`.

    mod bungee_guard_forwarding_error;
    mod bungee_guard_verifier;
    mod forwarded_identity;
    mod forwarded_profile;
    mod handshake_fields;
    mod legacy_forwarding;
    mod legacy_forwarding_error;
    mod modern_forwarding_error;
    mod modern_forwarding_verifier;

    pub use bungee_guard_forwarding_error::BungeeGuardForwardingError;
    pub use bungee_guard_verifier::BungeeGuardVerifier;
    pub use forwarded_identity::ForwardedIdentity;
    pub use forwarded_profile::ForwardedProfile;
    pub use legacy_forwarding::parse_legacy_handshake;
    pub use legacy_forwarding_error::LegacyForwardingError;
    pub use modern_forwarding_error::ModernForwardingError;
    pub use modern_forwarding_verifier::ModernForwardingVerifier;
}

pub mod frame {
    //! Length-prefixed framing.
    //!
    //! Every Minecraft packet travels as `varint(length) || payload`. [`VarIntFrameCodec`]
    //! is the port of the Java implementation's `VarIntFrameDecoder` and
    //! `VarIntLengthEncoder`, and it keeps their quirks: leading `0x00` bytes are skipped,
    //! and the length prefix is capped at 21 bits.

    mod frame_error;
    mod length_prefix;
    mod var_int_frame_codec;

    pub use frame_error::FrameError;
    pub use var_int_frame_codec::VarIntFrameCodec;
}

pub mod identity {
    //! Player identity: the offline-mode UUID and the UUID text a proxy sends.

    mod offline_uuid;
    mod uuid_parse_error;
    mod uuid_text;

    pub use offline_uuid::offline_mode_uuid;
    pub use uuid_parse_error::UuidParseError;
    pub use uuid_text::parse_uuid;
}

pub mod time {
    //! The clock port and the implementation the server runs with.

    mod clock;
    mod system_clock;

    pub use clock::Clock;
    pub use system_clock::SystemClock;
}

pub mod traffic {
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
}
