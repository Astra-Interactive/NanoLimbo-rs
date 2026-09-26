use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::broadcast;

use crate::connection::serve;
use crate::di::ServerContext;

/// Accepts connections until told to stop.
///
/// An accept failure is logged and the loop continues: a single refused connection, or a
/// momentary shortage of file descriptors, must not take the server down.
///
/// `stopping` is subscribed by the caller, before the task is spawned: a receiver
/// subscribed on the task's first poll misses a stop sent before that poll, and the loop
/// then never ends.
pub async fn accept_until_shutdown(
    listener: TcpListener,
    context: Arc<ServerContext>,
    shutdown: broadcast::Sender<()>,
    mut stopping: broadcast::Receiver<()>,
) {
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
