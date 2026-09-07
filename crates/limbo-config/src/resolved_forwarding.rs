use crate::config_warning::ConfigWarning;
use crate::info_forwarding::InfoForwarding;

/// The forwarding scheme, and the one thing about it that may be worth saying out loud.
pub struct ResolvedForwarding {
    pub forwarding: InfoForwarding,
    pub warning: Option<ConfigWarning>,
}
