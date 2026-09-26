use limbo_config::{InfoForwarding, LimboConfig};
use limbo_net::forwarding::{BungeeGuardVerifier, ModernForwardingVerifier};

/// The verifier for whichever forwarding scheme is configured, if any.
///
/// At most one is ever present: the schemes are alternatives, and holding them in one
/// value makes that plain where two `Option` parameters would not.
pub struct ForwardingVerifiers {
    pub bungee_guard: Option<BungeeGuardVerifier>,
    pub modern: Option<ModernForwardingVerifier>,
}

impl ForwardingVerifiers {
    pub fn for_config(config: &LimboConfig) -> Self {
        match &config.info_forwarding {
            InfoForwarding::BungeeGuard { tokens } => Self {
                bungee_guard: Some(BungeeGuardVerifier::new(tokens.clone())),
                modern: None,
            },
            InfoForwarding::Modern { secret } => Self {
                bungee_guard: None,
                modern: Some(ModernForwardingVerifier::new(secret.clone())),
            },
            // Reached directly, or the proxy puts everything in the handshake.
            InfoForwarding::None | InfoForwarding::Legacy => Self {
                bungee_guard: None,
                modern: None,
            },
        }
    }
}
