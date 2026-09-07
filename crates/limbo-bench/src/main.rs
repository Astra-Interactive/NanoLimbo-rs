//! Logs a crowd of players into one or more limbo servers and reports what each one cost.
//!
//! Written so the comparison is reproducible rather than something to take on trust:
//! point it at two servers and it prints the same table.
//!
//! It speaks the protocol through the same version tables the server uses, so a new
//! Minecraft release cannot leave it silently measuring a failed login.

mod bench_error;
mod bench_options;
mod bench_target;
mod login_client;
mod memory_source;
mod resident_memory;
mod target_report;

use std::process::ExitCode;
use std::time::{Duration, Instant};

/// How long one player waits for the server to stop sending before calling the join done.
///
/// Well below the five-second keep alive, so a keep alive never extends it.
const JOIN_QUIET_PERIOD: Duration = Duration::from_millis(400);

use crate::bench_options::BenchOptions;
use crate::bench_target::BenchTarget;
use crate::login_client::LoginClient;
use crate::target_report::{TargetReport, print_table};

const USAGE: &str = "\
usage: limbo-bench [options] <name=address[@pid]>...

  --players N           how many players to log in (default 300)
  --protocol N          protocol number to connect as (default: the newest supported)
  --settle-seconds N    how long to hold them before reading memory (default 2)

After @ give a process id, or a container name to read the whole container with
docker stats. Without one the server is still load-tested; only its memory goes
unmeasured. Do not mix the two in one run: a container figure includes more than a
process figure does.

  limbo-bench rust=127.0.0.1:25565@1234 java=127.0.0.1:25577@5678
  limbo-bench --players 300 rust=127.0.0.1:25565@nanolimbo-1
";

/// Waits until the server can actually serve a player, or the deadline passes.
///
/// Readiness is a completed login, not an accepted socket. Docker's port forwarder
/// accepts a connection before the container behind it is listening and then resets it,
/// so a connect-only probe reports ready too early; a shell probing `/dev/tcp` through
/// the same forwarder reports the opposite. Only joining proves the server is serving.
///
/// The probe player is dropped again before anything is measured, so it does not show up
/// in the idle reading.
async fn wait_until_serving(target: &BenchTarget, options: &BenchOptions) -> bool {
    let deadline = Instant::now() + options.wait;

    loop {
        match LoginClient::join(target.address, options.version, "Probe").await {
            Ok(_probe) => return true,
            Err(_not_yet) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            Err(_gave_up) => return false,
        }
    }
}

/// Logs players in one at a time and reports how far it got.
///
/// Sequential on purpose. Connecting concurrently measures how fast the client can open
/// sockets as much as anything the server does, and the figure of interest here is what
/// the server holds once they are all in, not how quickly they arrive.
async fn measure(target: &BenchTarget, options: &BenchOptions) -> TargetReport {
    if !wait_until_serving(target, options).await {
        return TargetReport::unreachable(target, options.players);
    }
    let idle_memory = target.memory.as_ref().and_then(resident_memory::of_source);

    let mut held = Vec::with_capacity(options.players);
    let mut first_failure = None;
    let mut join_bytes = None;
    let started = Instant::now();

    for index in 0..options.players {
        let username = format!("Bench{index}");
        match LoginClient::join(target.address, options.version, &username).await {
            Ok(mut client) => {
                // One player waits out the burst so the join can be sized; the rest would
                // only measure the same thing again, slowly.
                if index == 0 {
                    join_bytes = Some(client.drain_join_burst(JOIN_QUIET_PERIOD).await);
                }
                held.push(client);
            }
            Err(error) => {
                first_failure = Some(error.to_string());
                break;
            }
        }
    }
    let elapsed = started.elapsed();

    // Let the server finish whatever the arrival burst started before reading memory.
    tokio::time::sleep(options.settle).await;
    let loaded_memory = target.memory.as_ref().and_then(resident_memory::of_source);

    TargetReport {
        name: target.name.clone(),
        requested: options.players,
        logged_in: held.len(),
        elapsed,
        join_bytes,
        idle_memory,
        loaded_memory,
        first_failure,
    }
}

/// Returns whether every target served at least one player.
///
/// A target that served nobody is a failure worth an exit code: this doubles as the
/// readiness-and-smoke check for a freshly built container, and a script needs to be able
/// to tell. Serving fewer than asked is not a failure — a player cap is a setting.
async fn run(options: BenchOptions) -> bool {
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
    reports.iter().all(|report| report.logged_in > 0)
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

    match runtime.block_on(run(options)) {
        true => ExitCode::SUCCESS,
        false => ExitCode::FAILURE,
    }
}
