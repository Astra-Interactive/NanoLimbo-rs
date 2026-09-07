use crate::config_warning::ConfigWarning;
use crate::limbo_config::LimboConfig;

/// A configuration together with everything about it the operator should hear.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedConfig {
    pub config: LimboConfig,
    pub warnings: Vec<ConfigWarning>,
}
