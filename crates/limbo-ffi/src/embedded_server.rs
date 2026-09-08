use std::path::Path;

use nanolimbo::logging;
use nanolimbo::startup::{prepare, serve};
use nanolimbo::startup_error::StartupError;

use crate::cancellation_token::CancellationToken;
use crate::start_status::StartStatus;

/// Blocks the calling thread until `token` is cancelled.
///
/// The directory is passed rather than taken from the process, whose working directory
/// belongs to the host.
pub fn run(token: &CancellationToken, configuration_directory: &Path) -> StartStatus {
    let prepared = match prepare(configuration_directory) {
        Ok(prepared) => prepared,
        Err(error) => {
            // The configuration decides the log level, so there is no subscriber yet.
            eprintln!("Cannot start server: {error}");
            return StartStatus::StartupFailed;
        }
    };
    let context = prepared.context;

    logging::install(context.config.debug_level);
    for warning in prepared.warnings {
        tracing::warn!("{warning}");
    }
    tracing::info!("Starting server...");

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.enable_all();
    if let Some(threads) = context.config.runtime.worker_threads {
        builder.worker_threads(threads.max(1));
    }
    let runtime = match builder.build() {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!("Cannot build the async runtime: {error}");
            return StartStatus::RuntimeUnavailable;
        }
    };

    match runtime.block_on(serve(context, token.shutdown_source())) {
        Ok(()) => StartStatus::Ok,
        Err(StartupError::Bind { address, source }) => {
            tracing::error!("Cannot listen on {address}: {source}");
            StartStatus::BindFailed
        }
        Err(error) => {
            tracing::error!("Cannot start server: {error}");
            StartStatus::StartupFailed
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::sync::Notify;

    use super::*;

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
}
