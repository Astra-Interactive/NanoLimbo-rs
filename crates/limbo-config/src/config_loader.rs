use std::path::Path;

use crate::config_error::ConfigError;
use crate::config_file_system::ConfigFileSystem;
use crate::config_loader_defaults::{DEFAULT_SETTINGS, SETTINGS_FILE_NAME};
use crate::loaded_config::LoadedConfig;
use crate::settings_parser::parse_settings;

/// Finds, and if necessary creates, the configuration the server runs on.
pub struct ConfigLoader<F> {
    file_system: F,
}

impl<F: ConfigFileSystem> ConfigLoader<F> {
    pub const fn new(file_system: F) -> Self {
        Self { file_system }
    }

    /// Loads `settings.yml` from `directory`, writing the shipped default there first if
    /// the file does not exist yet, so that a first run leaves an editable file behind.
    ///
    /// `@file` credential references in the loaded file resolve against the same
    /// directory. A host name under `bind.ip` is resolved here, which may block.
    pub fn load(&self, directory: &Path) -> Result<LoadedConfig, ConfigError> {
        let path = directory.join(SETTINGS_FILE_NAME);

        if !self.file_system.exists(&path) {
            self.file_system
                .write(&path, DEFAULT_SETTINGS.as_bytes())
                .map_err(|error| ConfigError::Unwritable {
                    path: path.clone(),
                    source: error,
                })?;
        }

        let content = self
            .file_system
            .read(&path)
            .map_err(|error| ConfigError::Unreadable {
                path: path.clone(),
                source: error,
            })?;

        let yaml = String::from_utf8(content).map_err(|error| ConfigError::NotUtf8 {
            path,
            source: error,
        })?;

        Ok(parse_settings(&yaml, directory, &self.file_system)?)
    }
}
