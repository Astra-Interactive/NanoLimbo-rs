use std::sync::Arc;

use crate::server_context::ServerContext;

/// A server that is fully built but has not opened a port yet.
///
/// Warnings are carried rather than printed, because they are produced before the log
/// subscriber exists — the configuration decides what the log level is.
pub struct PreparedServer {
    pub context: Arc<ServerContext>,
    pub warnings: Vec<String>,
}
