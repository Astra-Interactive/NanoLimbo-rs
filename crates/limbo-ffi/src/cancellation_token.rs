use std::sync::Arc;

use nanolimbo::shutdown_source::ShutdownSource;
use tokio::sync::Notify;

/// The handle a host keeps so that it can stop a server it started.
///
/// `start_app` blocks for the lifetime of the server, so the stop request always arrives
/// on a different thread than the one running it. Everything here is therefore shared
/// rather than owned, and the type is `Sync`.
pub struct CancellationToken {
    notify: Arc<Notify>,
}

impl CancellationToken {
    pub const fn new(notify: Arc<Notify>) -> Self {
        Self { notify }
    }

    /// The shutdown source to hand to the server this token controls.
    pub fn shutdown_source(&self) -> ShutdownSource {
        ShutdownSource::Embedded(Arc::clone(&self.notify))
    }

    /// Asks the server to stop.
    ///
    /// A permit is stored rather than broadcast, so a host that stops the server before
    /// it has finished starting is still obeyed instead of being ignored.
    pub fn cancel(&self) {
        self.notify.notify_one();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn given_a_cancelled_token_when_its_source_is_awaited_then_it_resolves() {
        let token = CancellationToken::new(Arc::new(Notify::new()));
        let source = token.shutdown_source();

        token.cancel();

        tokio::time::timeout(std::time::Duration::from_secs(1), source.wait())
            .await
            .expect("cancelling the token must release a server waiting on its source");
    }

    /// A host must never find that the server has quietly taken over its console.
    #[test]
    fn given_a_token_when_it_produces_a_source_then_the_server_leaves_the_console_alone() {
        let token = CancellationToken::new(Arc::new(Notify::new()));

        assert!(!token.shutdown_source().owns_console());
    }
}
