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

mod file_system {
    //! The one port through which configuration touches the disk, so loading and the
    //! `@file` references inside it are testable without one.

    mod config_file_system;
    mod disk_file_system;

    pub use config_file_system::ConfigFileSystem;
    pub use disk_file_system::DiskFileSystem;
}

mod loading {
    //! Finding `settings.yml`, writing the shipped default when it is missing, and handing
    //! its text to the parser.

    mod config_error;
    mod config_loader;
    mod config_loader_defaults;

    pub use config_error::ConfigError;
    pub use config_loader::ConfigLoader;
    pub use config_loader_defaults::{DEFAULT_SETTINGS, SETTINGS_FILE_NAME};
}

mod settings {
    //! What `settings.yml` says once it has been validated, and the parser that gets it
    //! there.

    pub(crate) mod boss_bar {
        //! The boss bar shown to every player in the limbo.

        mod boss_bar_color;
        mod boss_bar_config;
        mod boss_bar_division;
        mod boss_bar_health;

        pub use boss_bar_color::BossBarColor;
        pub use boss_bar_config::BossBarConfig;
        pub use boss_bar_division::BossBarDivision;
        pub use boss_bar_health::BossBarHealth;
    }

    pub(crate) mod forwarding {
        //! How a proxy in front of the limbo passes the player's identity on, including the
        //! secrets and tokens that may be read from files next to `settings.yml`.

        mod info_forwarding;
        pub(crate) mod info_forwarding_resolver;
        mod resolved_forwarding;

        pub use info_forwarding::InfoForwarding;
        pub(crate) use info_forwarding_resolver::resolve_info_forwarding;
        pub(crate) use resolved_forwarding::ResolvedForwarding;
    }

    pub(crate) mod network {
        //! Where the server listens, and the runtime that serves the connections.

        mod bind_address;
        mod runtime_config;
        mod transport_type;

        pub(crate) use bind_address::resolve_bind_address;
        pub use runtime_config::RuntimeConfig;
        pub use transport_type::TransportType;
    }

    pub(crate) mod ping {
        //! The answer to a server list ping.

        mod ping_config;

        pub use ping_config::PingConfig;
    }

    pub(crate) mod player_list {
        //! The tab list: the entry for the joining player, and its header and footer.

        mod header_and_footer_config;
        mod player_list_config;

        pub use header_and_footer_config::HeaderAndFooterConfig;
        pub use player_list_config::PlayerListConfig;
    }

    pub(crate) mod schema {
        //! The shape of `settings.yml` as serde reads it, before any of it means anything.
        //!
        //! These are the one place in the workspace where `Default` is derived or implemented for
        //! a struct, as `struct-default-values.md` allows for serde types: a `Default` here is not
        //! a convenience, it is the record of what every setting falls back to when a deployment
        //! leaves it out, and it holds the numbers the reference implementation passed to its
        //! `getInt`/`getBoolean`/`getString` calls. Keeping them together makes the compatibility
        //! surface one page instead of scattered literals in the parser.

        mod bind_dto;
        mod boss_bar_dto;
        mod brand_name_dto;
        mod header_and_footer_dto;
        mod info_forwarding_dto;
        mod join_message_dto;
        mod netty_dto;
        mod netty_threads_dto;
        mod ping_dto;
        mod player_list_dto;
        mod scalar_text;
        mod settings_dto;
        mod title_dto;
        mod tokens_dto;
        mod traffic_dto;

        pub(crate) use bind_dto::BindDto;
        pub(crate) use boss_bar_dto::BossBarDto;
        pub(crate) use brand_name_dto::BrandNameDto;
        pub(crate) use header_and_footer_dto::HeaderAndFooterDto;
        pub(crate) use info_forwarding_dto::InfoForwardingDto;
        pub(crate) use join_message_dto::JoinMessageDto;
        pub(crate) use netty_dto::NettyDto;
        pub(crate) use netty_threads_dto::NettyThreadsDto;
        pub(crate) use ping_dto::PingDto;
        pub(crate) use player_list_dto::PlayerListDto;
        pub(crate) use scalar_text::ScalarText;
        pub(crate) use settings_dto::SettingsDto;
        pub(crate) use title_dto::TitleDto;
        pub(crate) use tokens_dto::TokensDto;
        pub(crate) use traffic_dto::TrafficDto;
    }

    pub(crate) mod title {
        //! The title shown on join, and the tick durations it is timed in.

        mod ticks;
        mod title_config;

        pub use ticks::Ticks;
        pub use title_config::TitleConfig;
    }

    pub(crate) mod traffic {
        //! The per-connection packet size and rate limits.

        mod traffic_config;

        pub use traffic_config::TrafficConfig;
    }

    mod config_warning;
    mod dimension_parser;
    mod limbo_config;
    mod loaded_config;
    mod settings_error;
    pub(crate) mod settings_parser;

    pub use config_warning::ConfigWarning;
    pub(crate) use dimension_parser::parse_dimension_type;
    pub use limbo_config::LimboConfig;
    pub use loaded_config::LoadedConfig;
    pub use settings_error::SettingsError;
    pub use settings_parser::parse_settings;
}

pub use file_system::{ConfigFileSystem, DiskFileSystem};
pub use loading::{ConfigError, ConfigLoader, DEFAULT_SETTINGS, SETTINGS_FILE_NAME};
pub use settings::boss_bar::{BossBarColor, BossBarConfig, BossBarDivision, BossBarHealth};
pub use settings::forwarding::InfoForwarding;
pub use settings::network::{RuntimeConfig, TransportType};
pub use settings::ping::PingConfig;
pub use settings::player_list::{HeaderAndFooterConfig, PlayerListConfig};
pub use settings::title::{Ticks, TitleConfig};
pub use settings::traffic::TrafficConfig;
pub use settings::{ConfigWarning, LimboConfig, LoadedConfig, SettingsError, parse_settings};

#[cfg(test)]
#[path = "../test/lib.rs"]
mod test;
