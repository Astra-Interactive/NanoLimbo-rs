use crate::settings::ConfigWarning;
use crate::settings::LimboConfig;

/// A configuration together with everything about it the operator should hear.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedConfig {
    pub config: LimboConfig,
    pub warnings: Vec<ConfigWarning>,
}
