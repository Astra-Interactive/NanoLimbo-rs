//! `settings.yml`: reading it, validating it, and writing the shipped default on first run.
//!
//! The file is the one part of the server that outlives a release, so the parsing here is
//! deliberately conservative: every key the Java implementation understood is understood,
//! with the same fallbacks and the same validation, and a setting that no longer applies
//! to a tokio server is accepted and reported rather than rejected. A deployment should
//! be able to replace the binary and change nothing else.
//!
//! What comes out is domain types — a resolved [`SocketAddr`](std::net::SocketAddr),
//! parsed [`Component`](limbo_text::chat::Component)s, [`Duration`](std::time::Duration)s
//! and enums — so that nothing downstream has to re-read a string from the configuration.
//!
//! ```no_run
//! use std::path::Path;
//!
//! use limbo_config::{ConfigLoader, DiskFileSystem};
//!
//! # fn main() -> Result<(), limbo_config::ConfigError> {
//! let loaded = ConfigLoader::new(DiskFileSystem).load(Path::new("."))?;
//!
//! for warning in &loaded.warnings {
//!     eprintln!("warning: {warning}");
//! }
//!
//! let address = loaded.config.bind_address;
//! # let _ = address;
//! # Ok(())
//! # }
//! ```

mod bind_address;
mod boss_bar_color;
mod boss_bar_config;
mod boss_bar_division;
mod boss_bar_health;
mod config_error;
mod config_file_system;
mod config_loader;
mod config_loader_defaults;
mod config_warning;
mod dimension_parser;
mod disk_file_system;
mod dto;
mod header_and_footer_config;
mod info_forwarding;
mod info_forwarding_resolver;
mod limbo_config;
mod loaded_config;
mod ping_config;
mod player_list_config;
mod resolved_forwarding;
mod runtime_config;
mod settings_error;
mod settings_parser;
mod ticks;
mod title_config;
mod traffic_config;
mod transport_type;

pub use boss_bar_color::BossBarColor;
pub use boss_bar_config::BossBarConfig;
pub use boss_bar_division::BossBarDivision;
pub use boss_bar_health::BossBarHealth;
pub use config_error::ConfigError;
pub use config_file_system::ConfigFileSystem;
pub use config_loader::ConfigLoader;
pub use config_loader_defaults::{DEFAULT_SETTINGS, SETTINGS_FILE_NAME};
pub use config_warning::ConfigWarning;
pub use disk_file_system::DiskFileSystem;
pub use header_and_footer_config::HeaderAndFooterConfig;
pub use info_forwarding::InfoForwarding;
pub use limbo_config::LimboConfig;
pub use loaded_config::LoadedConfig;
pub use ping_config::PingConfig;
pub use player_list_config::PlayerListConfig;
pub use runtime_config::RuntimeConfig;
pub use settings_error::SettingsError;
pub use settings_parser::parse_settings;
pub use ticks::Ticks;
pub use title_config::TitleConfig;
pub use traffic_config::TrafficConfig;
pub use transport_type::TransportType;
