use std::sync::Arc;

use limbo_config::{InfoForwarding, LimboConfig};
use limbo_net::forwarding::{BungeeGuardVerifier, ModernForwardingVerifier};
use limbo_server::connection_registry::ConnectionRegistry;
use limbo_server::forwarding_mode::ForwardingMode;
use limbo_server::server_policy::ServerPolicy;

use crate::packet_snapshots::PacketSnapshots;
use crate::random_id_source::RandomIdSource;

/// Everything a connection task needs, shared by all of them.
///
/// Assembled once in the composition root and handed down as an `Arc`. Upstream reached
/// the same state through public static fields; passing it explicitly means a test can
/// stand up a second server with different settings in the same process.
pub struct ServerContext {
    pub config: Arc<LimboConfig>,
    pub snapshots: Arc<PacketSnapshots>,
    pub connections: Arc<ConnectionRegistry>,
    pub ids: RandomIdSource,
    pub bungee_guard: Option<BungeeGuardVerifier>,
    pub modern_forwarding: Option<ModernForwardingVerifier>,
}

impl ServerContext {
    pub fn forwarding_mode(&self) -> ForwardingMode {
        match &self.config.info_forwarding {
            InfoForwarding::None => ForwardingMode::None,
            InfoForwarding::Legacy => ForwardingMode::Legacy,
            InfoForwarding::Modern { .. } => ForwardingMode::Modern,
            InfoForwarding::BungeeGuard { .. } => ForwardingMode::BungeeGuard,
        }
    }

    pub fn policy(&self) -> ServerPolicy {
        ServerPolicy::new(self.forwarding_mode(), self.config.max_players)
    }
}
