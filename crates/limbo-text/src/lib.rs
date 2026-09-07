//! Chat components and the text formats the server converts between.
//!
//! The Java implementation delegated all of this to Kyori Adventure. No Rust crate covers
//! the same ground — Adventure's per-version JSON profiles and its NBT component encoding
//! in particular have no equivalent — so this crate implements the subset `settings.yml`
//! can express, and is checked against fixtures dumped from Adventure.
//!
//! See `MIGRATION_PLAN.md` sections 5.4 and 7.

pub mod chat;
pub mod component_json;
pub mod component_nbt;
pub mod component_writer;
pub mod json_profile;
pub mod legacy_codes;
pub mod mini_message;
mod mini_message_node;
pub mod text_parser;

pub use component_writer::{to_json_for, write_component};
pub use json_profile::JsonProfile;
pub use legacy_codes::to_legacy_string;
pub use text_parser::parse;
