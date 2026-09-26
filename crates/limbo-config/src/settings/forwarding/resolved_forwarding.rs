use crate::settings::ConfigWarning;
use crate::settings::forwarding::InfoForwarding;

/// The forwarding scheme, and the one thing about it that may be worth saying out loud.
pub struct ResolvedForwarding {
    pub forwarding: InfoForwarding,
    pub warning: Option<ConfigWarning>,
}
