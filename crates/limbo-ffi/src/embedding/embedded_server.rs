use std::path::Path;

use nanolimbo::lifecycle::StartupError;
use nanolimbo::lifecycle::install_logging;
use nanolimbo::lifecycle::{prepare, serve};

use crate::embedding::CancellationToken;
use crate::embedding::StartStatus;

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

    install_logging(context.config.debug_level);
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
