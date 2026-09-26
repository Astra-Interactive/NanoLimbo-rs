use limbo_config::InfoForwarding;
use limbo_net::forwarding::parse_legacy_handshake;
use uuid::Uuid;

use crate::di::ServerContext;

/// What a proxy vouched for in the handshake.
pub(crate) struct ForwardedFromProxy {
    pub(crate) address: String,
    pub(crate) uuid: Uuid,
}

impl ForwardedFromProxy {
    /// Reads a proxy-forwarded identity out of the handshake host field.
    ///
    /// Both proxy formats pack it into that field separated by NUL bytes, so it is available
    /// before the client has said anything else.
    pub(crate) fn from_handshake(context: &ServerContext, host: &str) -> Option<Self> {
        match &context.config.info_forwarding {
            InfoForwarding::Legacy => parse_legacy_handshake(host).ok().map(|identity| Self {
                address: identity.address,
                uuid: identity.uuid,
            }),
            InfoForwarding::BungeeGuard { .. } => context
                .bungee_guard
                .as_ref()?
                .verify(host)
                .ok()
                .map(|identity| Self {
                    address: identity.address,
                    uuid: identity.uuid,
                }),
            InfoForwarding::None | InfoForwarding::Modern { .. } => None,
        }
    }
}
