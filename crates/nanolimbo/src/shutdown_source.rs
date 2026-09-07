use std::sync::Arc;

use tokio::sync::Notify;

/// Where a running server learns that it should stop.
///
/// The binary owns its process, so it can read commands from standard input and install
/// signal handlers. A server embedded in a host process owns neither: standard input
/// belongs to that host's own console, and the signal handlers belong to its runtime.
/// Reading the console there would swallow the host's commands, which is why the two
/// cases are distinct rather than one function with a flag.
pub enum ShutdownSource {
    /// Console commands on standard input, plus Ctrl-C and SIGTERM.
    Standalone,
    /// An explicit request from the host, and nothing else.
    Embedded(Arc<Notify>),
}

impl ShutdownSource {
    /// Whether the server may take standard input for its own console.
    pub const fn owns_console(&self) -> bool {
        matches!(self, Self::Standalone)
    }

    /// Resolves once the server has been asked to stop.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_an_embedded_server_when_asked_about_the_console_then_it_leaves_it_to_the_host() {
        let embedded = ShutdownSource::Embedded(Arc::new(Notify::new()));

        assert!(!embedded.owns_console());
        assert!(ShutdownSource::Standalone.owns_console());
    }

    #[tokio::test]
    async fn given_an_embedded_server_when_the_host_stops_it_then_the_wait_resolves() {
        let notify = Arc::new(Notify::new());
        let source = ShutdownSource::Embedded(Arc::clone(&notify));

        notify.notify_one();

        source.wait().await;
    }

    /// The host may ask the server to stop before the server ever starts waiting. A
    /// stored permit is what keeps that request from being lost, so a proxy that shuts
    /// down while the limbo is still starting does not hang.
    #[tokio::test]
    async fn given_a_stop_requested_before_the_wait_begins_then_the_request_is_not_lost() {
        let notify = Arc::new(Notify::new());
        notify.notify_one();

        let source = ShutdownSource::Embedded(notify);

        tokio::time::timeout(std::time::Duration::from_secs(1), source.wait())
            .await
            .expect("a stop requested earlier must still be observed");
    }
}
