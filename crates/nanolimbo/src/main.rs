//! Composition root: reads the configuration, wires every module, and starts the server.
//!
//! Nothing below this file reaches for a global. Configuration, pre-encoded packets, the
//! connection registry and the source of random ids are all built here and handed down,
//! which is what lets every layer under it be tested without a socket.

use std::path::Path;
use std::process::ExitCode;

use nanolimbo::logging;
use nanolimbo::startup::{prepare, serve};
use nanolimbo::startup_error::StartupError;

fn run() -> Result<(), StartupError> {
    let prepared = prepare(Path::new("."))?;
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

    builder
        .build()
        .map_err(StartupError::Runtime)?
        .block_on(serve(context))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // The subscriber may not be installed yet, so this goes to stderr directly.
            eprintln!("Cannot start server: {error}");
            ExitCode::FAILURE
        }
    }
}
