//! Composition root: reads the configuration, wires every module, and starts the server.
//!
//! Nothing below this file reaches for a global. Configuration, pre-encoded packets, the
//! connection registry and the source of random ids are all built here and handed down,
//! which is what lets every layer under it be tested without a socket.

mod connection_task;
mod console;
mod forwarding_verifiers;
mod listener;
mod logging;
mod packet_snapshots;
mod prepared_server;
mod random_id_source;
mod server_context;
mod title_snapshots;

use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use limbo_config::{ConfigLoader, DiskFileSystem};
use limbo_server::connection_registry::ConnectionRegistry;
use limbo_world::{DimensionRegistry, UpdateTagsRegistry};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use crate::forwarding_verifiers::ForwardingVerifiers;
use crate::packet_snapshots::PacketSnapshots;
use crate::prepared_server::PreparedServer;
use crate::random_id_source::RandomIdSource;
use crate::server_context::ServerContext;

/// Reported by the `version` command and useful in a bug report.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Anything that stops the server before it has served anything.
#[derive(Debug, thiserror::Error)]
enum StartupError {
    #[error("configuration: {0}")]
    Config(#[from] limbo_config::ConfigError),

    #[error("embedded dimension data: {0}")]
    Resources(#[from] limbo_world::ResourceLoadError),

    #[error("dimension {0}")]
    Dimension(#[from] limbo_world::DimensionLookupError),

    #[error("packet encoding: {0}")]
    Encoding(#[from] limbo_packet::PacketEncodeError),

    #[error("could not build the async runtime: {0}")]
    Runtime(std::io::Error),

    #[error("could not listen on {address}: {source}")]
    Bind {
        address: std::net::SocketAddr,
        source: std::io::Error,
    },
}

/// Does everything that can fail before a port is opened.
///
/// Loading resources and encoding every packet up front means a misconfiguration or a
/// corrupt resource is reported at startup, to an operator watching, rather than to the
/// first player who happens to connect on the affected version.
fn prepare(root: &Path) -> Result<PreparedServer, StartupError> {
    let loaded = ConfigLoader::new(DiskFileSystem).load(root)?;
    let warnings = loaded
        .warnings
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let config = Arc::new(loaded.config);

    let dimensions = DimensionRegistry::load()?;
    let tags = UpdateTagsRegistry::load()?;
    let dimension = config.dimension.resolve(&dimensions)?;

    let ids = RandomIdSource;
    let snapshots = PacketSnapshots::build(&config, &dimensions, &dimension, &tags, &ids)?;
    let verifiers = ForwardingVerifiers::for_config(&config);

    Ok(PreparedServer {
        context: Arc::new(ServerContext {
            config,
            snapshots: Arc::new(snapshots),
            connections: Arc::new(ConnectionRegistry::empty()),
            ids,
            bungee_guard: verifiers.bungee_guard,
            modern_forwarding: verifiers.modern,
        }),
        warnings,
    })
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

async fn serve(context: Arc<ServerContext>) -> Result<(), StartupError> {
    let address = context.config.bind_address;
    let listener = TcpListener::bind(address)
        .await
        .map_err(|source| StartupError::Bind { address, source })?;

    let (shutdown, _) = broadcast::channel(1);
    tracing::info!("Server started on {address}");

    tokio::spawn(console::run(
        console::read_lines(),
        Arc::clone(&context.connections),
        VERSION.to_owned(),
        shutdown.clone(),
    ));

    let accepting = tokio::spawn(listener::accept_until_shutdown(
        listener,
        Arc::clone(&context),
        shutdown.clone(),
    ));

    let mut stopping = shutdown.subscribe();
    tokio::select! {
        _ = wait_for_signal() => {}
        _ = stopping.recv() => {}
    }

    tracing::info!("Stopping server...");
    let _ = shutdown.send(());
    let _ = accepting.await;
    tracing::info!("Server stopped, goodbye!");
    Ok(())
}

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
