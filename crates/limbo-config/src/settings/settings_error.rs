use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use thiserror::Error;

/// Something in the content of `settings.yml` the server refuses to start with.
///
/// Every variant names the setting it came from, because the only person who can fix a
/// bad configuration is reading the log line this renders into.
#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("settings.yml is not valid YAML: {0}")]
    Malformed(#[from] serde_yaml_ng::Error),

    #[error("bind.ip `{host}` cannot be resolved")]
    BindHostUnresolvable {
        host: String,
        #[source]
        source: io::Error,
    },

    #[error("bind.ip `{host}` resolves to no address")]
    BindHostWithoutAddress { host: String },

    #[error(
        "unknown dimension `{value}`, expected OVERWORLD, THE_NETHER (NETHER) or THE_END (END)"
    )]
    UnknownDimension { value: String },

    #[error(
        "unknown bossBar.color `{value}`, expected PINK, BLUE, RED, GREEN, YELLOW, PURPLE or WHITE"
    )]
    UnknownBossBarColor { value: String },

    #[error(
        "unknown bossBar.division `{value}`, expected SOLID, DASHES_6, DASHES_10, DASHES_12 or DASHES_20"
    )]
    UnknownBossBarDivision { value: String },

    #[error("bossBar.health must be between 0.0 and 1.0, got {health}")]
    BossBarHealthOutOfRange { health: f32 },

    #[error("unknown infoForwarding.type `{value}`, expected NONE, LEGACY, MODERN or BUNGEE_GUARD")]
    UnknownForwardingType { value: String },

    #[error("unknown netty.transportType `{value}`, expected NIO, EPOLL, IO_URING or KQUEUE")]
    UnknownTransportType { value: String },

    #[error("cannot read the file referenced with `@`: {}", path.display())]
    ExternalFileUnreadable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("the file referenced with `@` is not valid UTF-8: {}", path.display())]
    ExternalFileNotUtf8 {
        path: PathBuf,
        #[source]
        source: FromUtf8Error,
    },
}
