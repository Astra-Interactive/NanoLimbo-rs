use crate::transport_type::TransportType;

/// What the `netty` block of `settings.yml` still has to say to a tokio server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Advisory: tokio selects the polling mechanism itself. Kept so the value a
    /// deployment configured can be reported back to it.
    pub transport_type: TransportType,
    /// Worker threads for the runtime, or `None` to let tokio size the pool from the
    /// available parallelism, which is what a thread count of zero meant to Netty.
    pub worker_threads: Option<usize>,
}
