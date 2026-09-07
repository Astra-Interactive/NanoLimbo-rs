//! The shape of `settings.yml` as serde reads it, before any of it means anything.
//!
//! These are the one place in the workspace where `Default` is derived or implemented for
//! a struct, as `struct-default-values.md` allows for serde types: a `Default` here is not
//! a convenience, it is the record of what every setting falls back to when a deployment
//! leaves it out, and it holds the numbers the reference implementation passed to its
//! `getInt`/`getBoolean`/`getString` calls. Keeping them together makes the compatibility
//! surface one page instead of scattered literals in the parser.

pub mod bind_dto;
pub mod boss_bar_dto;
pub mod brand_name_dto;
pub mod header_and_footer_dto;
pub mod info_forwarding_dto;
pub mod join_message_dto;
pub mod netty_dto;
pub mod netty_threads_dto;
pub mod ping_dto;
pub mod player_list_dto;
pub mod scalar_text;
pub mod settings_dto;
pub mod title_dto;
pub mod tokens_dto;
pub mod traffic_dto;
