use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::broadcast;

use crate::connection_task::serve;
use crate::server_context::ServerContext;

/// Accepts connections until told to stop.
///
/// An accept failure is logged and the loop continues: a single refused connection, or a
/// momentary shortage of file descriptors, must not take the server down.
pub async fn accept_until_shutdown(
    listener: TcpListener,
    context: Arc<ServerContext>,
    shutdown: broadcast::Sender<()>,
) {
    let mut stopping = shutdown.subscribe();

    loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, address)) => {
                    let context = Arc::clone(&context);
                    let receiver = shutdown.subscribe();
                    tokio::spawn(async move {
                        serve(stream, address, context, receiver).await;
                    });
                }
                Err(error) => tracing::warn!(%error, "could not accept a connection"),
            },
            _ = stopping.recv() => return,
        }
    }
}
