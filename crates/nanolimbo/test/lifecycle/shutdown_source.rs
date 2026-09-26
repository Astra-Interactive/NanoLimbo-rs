use std::sync::Arc;

use tokio::sync::Notify;

use crate::lifecycle::ShutdownSource;

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
