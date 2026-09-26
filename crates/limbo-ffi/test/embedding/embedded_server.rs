use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;

use crate::embedding::{CancellationToken, StartStatus, run};

fn scratch_directory(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "nanolimbo-ffi-{}-{name}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&root).expect("a working directory");
    root
}

#[test]
fn given_an_unreadable_configuration_when_the_server_runs_then_it_reports_a_startup_failure() {
    let root = scratch_directory("malformed");
    std::fs::write(root.join("settings.yml"), "bind: [this is not a mapping]")
        .expect("a settings file");

    let status = run(&CancellationToken::new(Arc::new(Notify::new())), &root);

    assert_eq!(status, StartStatus::StartupFailed);
}

#[test]
fn given_a_running_server_when_the_host_cancels_the_token_then_it_stops_and_reports_ok() {
    let root = scratch_directory("cancels");
    std::fs::write(
        root.join("settings.yml"),
        "bind:\n  ip: \"127.0.0.1\"\n  port: 0\ndebugLevel: 0\n",
    )
    .expect("a settings file");

    let token = Arc::new(CancellationToken::new(Arc::new(Notify::new())));
    let stopper = Arc::clone(&token);
    let server = std::thread::spawn(move || run(&token, &root));

    std::thread::sleep(Duration::from_millis(200));
    stopper.cancel();

    let status = server.join().expect("the server thread must not panic");

    assert_eq!(status, StartStatus::Ok);
}
