use std::sync::Arc;

use tokio::sync::Notify;

/// An embedded server owns neither standard input nor the signal handlers: reading the
/// console would swallow its host's own commands.
pub enum ShutdownSource {
    Standalone,
    Embedded(Arc<Notify>),
}

impl ShutdownSource {
    pub const fn owns_console(&self) -> bool {
        matches!(self, Self::Standalone)
    }

    pub async fn wait(&self) {
        match self {
            Self::Standalone => wait_for_signal().await,
            Self::Embedded(notify) => notify.notified().await,
        }
    }
}

/// Waits for whichever signal comes first, so a container stop is as clean as Ctrl-C.
async fn wait_for_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate = match signal(SignalKind::terminate()) {
            Ok(stream) => stream,
            Err(error) => {
                tracing::warn!(%error, "cannot listen for SIGTERM; Ctrl-C still works");
                let _ = tokio::signal::ctrl_c().await;
                return;
            }
        };
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
