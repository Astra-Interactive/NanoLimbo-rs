//! Everything between reading the configuration and the first byte on the wire.

use std::path::Path;
use std::sync::Arc;

use limbo_config::{ConfigLoader, DiskFileSystem};
use limbo_server::player::ConnectionRegistry;
use limbo_world::{DimensionRegistry, UpdateTagsRegistry};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

use crate::connection::accept_until_shutdown;
use crate::di::ForwardingVerifiers;
use crate::di::ServerContext;
use crate::id::RandomIdSource;
use crate::lifecycle::PreparedServer;
use crate::lifecycle::ShutdownSource;
use crate::lifecycle::StartupError;
use crate::lifecycle::{read_console_lines, run_console};
use crate::snapshot::PacketSnapshots;

/// Reported by the `version` command and useful in a bug report.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Does everything that can fail before a port is opened.
///
/// Loading resources and encoding every packet up front means a misconfiguration or a
/// corrupt resource is reported at startup, to an operator watching, rather than to the
/// first player who happens to connect on the affected version.
pub fn prepare(root: &Path) -> Result<PreparedServer, StartupError> {
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

/// Serves connections until the server is asked to stop.
///
/// `shutdown_source` decides what "asked to stop" means, which is what lets the same
/// runtime be driven both by the binary and by a host process that embeds it.
pub async fn serve(
    context: Arc<ServerContext>,
    shutdown_source: ShutdownSource,
) -> Result<(), StartupError> {
    let address = context.config.bind_address;
    let listener = TcpListener::bind(address)
        .await
        .map_err(|source| StartupError::Bind { address, source })?;

    let (shutdown, _) = broadcast::channel(1);
    tracing::info!("Server started on {address}");

    if shutdown_source.owns_console() {
        tokio::spawn(run_console(
            read_console_lines(),
            Arc::clone(&context.connections),
            VERSION.to_owned(),
            shutdown.clone(),
        ));
    }

    let accepting = tokio::spawn(accept_until_shutdown(
        listener,
        Arc::clone(&context),
        shutdown.clone(),
        shutdown.subscribe(),
    ));

    let mut stopping = shutdown.subscribe();
    tokio::select! {
        _ = shutdown_source.wait() => {}
        _ = stopping.recv() => {}
    }

    tracing::info!("Stopping server...");
    let _ = shutdown.send(());
    let _ = accepting.await;
    tracing::info!("Server stopped, goodbye!");
    Ok(())
}
