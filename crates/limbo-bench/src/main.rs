//! Logs a crowd of players into one or more limbo servers and reports what each one cost.
//!
//! Written to make the comparison in `MIGRATION_PLAN.md` section 9 reproducible rather
//! than something to take on trust: point it at two servers and it prints the same table.
//!
//! It speaks the protocol through the same version tables the server uses, so a new
//! Minecraft release cannot leave it silently measuring a failed login.

mod bench_error;
mod bench_options;
mod bench_target;
mod login_client;
mod resident_memory;
mod target_report;

use std::process::ExitCode;
use std::time::Instant;

use crate::bench_options::BenchOptions;
use crate::bench_target::BenchTarget;
use crate::login_client::LoginClient;
use crate::target_report::{TargetReport, print_table};

const USAGE: &str = "\
usage: limbo-bench [options] <name=address[@pid]>...

  --players N           how many players to log in (default 300)
  --protocol N          protocol number to connect as (default: the newest supported)
  --settle-seconds N    how long to hold them before reading memory (default 2)

Give a process id after @ to have memory reported for that server. Without one the
server is still load-tested; only its memory goes unmeasured.

  limbo-bench rust=127.0.0.1:25565@1234 java=127.0.0.1:25577@5678
";

/// Logs players in one at a time and reports how far it got.
///
/// Sequential on purpose. Connecting concurrently measures how fast the client can open
/// sockets as much as anything the server does, and the figure of interest here is what
/// the server holds once they are all in, not how quickly they arrive.
async fn measure(target: &BenchTarget, options: &BenchOptions) -> TargetReport {
    let idle_memory = target.pid.and_then(resident_memory::of_process);

    let mut held = Vec::with_capacity(options.players);
    let mut first_failure = None;
    let started = Instant::now();

    for index in 0..options.players {
        let username = format!("Bench{index}");
        match LoginClient::join(target.address, options.version, &username).await {
            Ok(client) => held.push(client),
            Err(error) => {
                first_failure = Some(error.to_string());
                break;
            }
        }
    }
    let elapsed = started.elapsed();

    // Let the server finish whatever the arrival burst started before reading memory.
    tokio::time::sleep(options.settle).await;
    let loaded_memory = target.pid.and_then(resident_memory::of_process);

    TargetReport {
        name: target.name.clone(),
        requested: options.players,
        logged_in: held.len(),
        elapsed,
        bytes_received: held.iter().map(LoginClient::bytes_received).sum(),
        idle_memory,
        loaded_memory,
        first_failure,
    }
}

async fn run(options: BenchOptions) {
    println!(
        "logging in {} players as protocol {} ({})",
        options.players,
        options.version.number(),
        options.version
    );

    let mut reports = Vec::new();
    for target in &options.targets {
        println!("  {} at {}...", target.name, target.address);
        reports.push(measure(target, &options).await);
    }

    print_table(&reports);
}

fn main() -> ExitCode {
    let options = match BenchOptions::parse(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}\n\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("could not build the async runtime: {error}");
            return ExitCode::FAILURE;
        }
    };

    runtime.block_on(run(options));
    ExitCode::SUCCESS
}
