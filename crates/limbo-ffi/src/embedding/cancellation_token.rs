use std::sync::Arc;

use nanolimbo::lifecycle::ShutdownSource;
use tokio::sync::Notify;

/// `start_app` blocks for the server's lifetime, so a stop always arrives on another
/// thread. Hence shared, not owned.
pub struct CancellationToken {
    notify: Arc<Notify>,
}

impl CancellationToken {
    pub const fn new(notify: Arc<Notify>) -> Self {
        Self { notify }
    }

    pub fn shutdown_source(&self) -> ShutdownSource {
        ShutdownSource::Embedded(Arc::clone(&self.notify))
    }

    /// Stores a permit, so a stop that arrives before the server starts is not lost.
    pub fn cancel(&self) {
        self.notify.notify_one();
    }
}
