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
