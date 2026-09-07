//! Minecraft protocol primitives shared by every layer of the server.
//!
//! This crate is pure domain logic: it performs no I/O and depends on no runtime,
//! so all of it is host-testable.

pub mod buffer;
pub mod packet;
pub mod version;
