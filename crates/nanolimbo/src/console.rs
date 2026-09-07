use std::sync::Arc;

use limbo_server::connection_registry::ConnectionRegistry;
use limbo_server::console_command::ConsoleCommand;
use limbo_server::memory_usage;
use tokio::sync::{broadcast, mpsc};

/// Reads console input on a blocking thread and reports each line.
///
/// Standard input has no async form that can be cancelled, so it lives on its own thread
/// and speaks to the runtime through a channel. When the thread ends — a closed pipe, or
/// the terminal going away — the channel closes and the command loop simply stops
/// listening, which leaves the server running.
pub fn read_lines() -> mpsc::Receiver<String> {
    let (sender, receiver) = mpsc::channel(8);

    std::thread::spawn(move || {
        for line in std::io::stdin().lines() {
            let Ok(line) = line else { return };
            if sender.blocking_send(line).is_err() {
                return;
            }
        }
    });

    receiver
}

fn describe_memory() {
    match memory_usage::current() {
        Some(usage) => tracing::info!("Memory used: {} MB", usage.resident_megabytes()),
        None => tracing::info!("Memory usage is not available on this platform"),
    }
}

/// Runs a command and reports the result. Returns whether the server should stop.
fn execute(command: ConsoleCommand, connections: &ConnectionRegistry, version: &str) -> bool {
    match command {
        ConsoleCommand::Help => {
            tracing::info!("Available commands:");
            for available in ConsoleCommand::ALL {
                let names = available.names().join(", ");
                tracing::info!("{names} - {}", available.description());
            }
        }
        ConsoleCommand::Connections => {
            tracing::info!("Connections: {}", connections.count());
        }
        ConsoleCommand::Memory => describe_memory(),
        ConsoleCommand::Version => tracing::info!("Version: {version}"),
        ConsoleCommand::Stop => return true,
    }
    false
}

/// Handles console input until the server stops or input runs out.
pub async fn run(
    mut lines: mpsc::Receiver<String>,
    connections: Arc<ConnectionRegistry>,
    version: String,
    shutdown: broadcast::Sender<()>,
) {
    let mut stopping = shutdown.subscribe();

    loop {
        let line = tokio::select! {
            line = lines.recv() => match line {
                Some(line) => line,
                None => return,
            },
            _ = stopping.recv() => return,
        };

        match ConsoleCommand::parse(&line) {
            Some(command) => {
                if execute(command, &connections, &version) {
                    let _ = shutdown.send(());
                    return;
                }
            }
            None if line.trim().is_empty() => {}
            None => tracing::info!("Unknown command. Type \"help\" to get commands list"),
        }
    }
}
