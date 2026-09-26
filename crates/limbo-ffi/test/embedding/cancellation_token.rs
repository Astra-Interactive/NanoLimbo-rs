use std::sync::Arc;

use tokio::sync::Notify;

use crate::embedding::CancellationToken;

#[tokio::test]
async fn given_a_cancelled_token_when_its_source_is_awaited_then_it_resolves() {
    let token = CancellationToken::new(Arc::new(Notify::new()));
    let source = token.shutdown_source();

    token.cancel();

    tokio::time::timeout(std::time::Duration::from_secs(1), source.wait())
        .await
        .expect("cancelling the token must release a server waiting on its source");
}

#[test]
fn given_a_token_when_it_produces_a_source_then_the_server_leaves_the_console_alone() {
    let token = CancellationToken::new(Arc::new(Notify::new()));

    assert!(!token.shutdown_source().owns_console());
}
