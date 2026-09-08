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
