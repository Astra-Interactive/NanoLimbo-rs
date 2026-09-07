//! Chat components and the text formats the server converts between.
//!
//! The Java implementation delegated all of this to Kyori Adventure. No Rust crate covers
//! the same ground — Adventure's per-version JSON profiles and its NBT component encoding
//! in particular have no equivalent — so this crate implements the subset `settings.yml`
//! can express, and is verified byte-for-byte against fixtures dumped from Adventure.
//!
//! See `MIGRATION_PLAN.md` sections 5.4 and 7.

pub mod chat;
