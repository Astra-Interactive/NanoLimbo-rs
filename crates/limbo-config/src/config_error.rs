use std::io;
use std::path::PathBuf;
use std::string::FromUtf8Error;

use thiserror::Error;

use crate::settings_error::SettingsError;

/// Why the server could not obtain a configuration to start from.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot read {}", path.display())]
    Unreadable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("cannot write the default configuration to {}", path.display())]
    Unwritable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("{} is not valid UTF-8", path.display())]
    NotUtf8 {
        path: PathBuf,
        #[source]
        source: FromUtf8Error,
    },

    #[error(transparent)]
    Settings(#[from] SettingsError),
}
