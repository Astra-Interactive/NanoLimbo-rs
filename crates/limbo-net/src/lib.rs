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

pub mod forwarding;
pub mod frame;
pub mod identity;
pub mod time;
pub mod traffic;
