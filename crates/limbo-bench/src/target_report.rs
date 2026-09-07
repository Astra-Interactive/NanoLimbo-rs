use std::time::Duration;

use crate::resident_memory::ResidentMemory;

/// What one server did when the players arrived.
pub struct TargetReport {
    pub name: String,
    pub requested: usize,
    pub logged_in: usize,
    pub elapsed: Duration,
    /// Bytes one player is sent to get into the world, measured to quiescence.
    pub join_bytes: Option<usize>,
    pub idle_memory: Option<ResidentMemory>,
    pub loaded_memory: Option<ResidentMemory>,
    /// Why players stopped arriving, when not all of them did.
    pub first_failure: Option<String>,
}

impl TargetReport {
    /// A server that never accepted a connection, so nothing about it was measured.
    pub fn unreachable(target: &crate::bench_target::BenchTarget, requested: usize) -> Self {
        Self {
            name: target.name.clone(),
            requested,
            logged_in: 0,
            elapsed: Duration::ZERO,
            join_bytes: None,
            idle_memory: None,
            loaded_memory: None,
            first_failure: Some(format!("{} never served a player", target.address)),
        }
    }

    /// Memory attributable to the players, spread over them.
    ///
    /// `None` when either reading is missing, or when memory went down — which happens on
    /// a runtime that reclaims during the run and makes the per-player figure meaningless
    /// rather than merely small.
    pub fn per_player_bytes(&self) -> Option<f64> {
        let idle = self.idle_memory?.bytes;
        let loaded = self.loaded_memory?.bytes;
        let players = u64::try_from(self.logged_in)
            .ok()
            .filter(|count| *count > 0)?;

        loaded
            .checked_sub(idle)
            .map(|growth| growth as f64 / players as f64)
    }
}

fn megabytes(memory: Option<ResidentMemory>) -> String {
    match memory {
        Some(reading) => format!("{:.1} MB", reading.megabytes()),
        None => "n/a".to_owned(),
    }
}

fn kilobytes(bytes: Option<f64>) -> String {
    match bytes {
        Some(per_player) => format!("{:.1} KB", per_player / 1024.0),
        None => "n/a".to_owned(),
    }
}

/// Below this many players the per-player figure is mostly startup noise: a runtime that
/// allocates in chunks, or warms up as it goes, spreads a fixed cost over too few players.
const RELIABLE_PLAYER_COUNT: usize = 200;

/// Prints the comparison as a table, widest column first so the numbers line up.
pub fn print_table(reports: &[TargetReport]) {
    println!(
        "\n{:<10} {:>7} {:>7} {:>9} {:>10} {:>11} {:>12} {:>12}",
        "target", "asked", "joined", "time", "idle", "loaded", "mem/player", "join bytes"
    );
    println!("{}", "-".repeat(84));

    for report in reports {
        println!(
            "{:<10} {:>7} {:>7} {:>8.2}s {:>10} {:>11} {:>12} {:>12}",
            report.name,
            report.requested,
            report.logged_in,
            report.elapsed.as_secs_f64(),
            megabytes(report.idle_memory),
            megabytes(report.loaded_memory),
            kilobytes(report.per_player_bytes()),
            kilobytes(report.join_bytes.map(|bytes| bytes as f64)),
        );
    }

    if reports
        .iter()
        .any(|report| report.logged_in < RELIABLE_PLAYER_COUNT)
    {
        println!(
            "\nFewer than {RELIABLE_PLAYER_COUNT} players joined, so mem/player is mostly \
             startup cost spread thin. Raise --players for a figure worth quoting."
        );
    }

    for report in reports {
        if let Some(reason) = &report.first_failure {
            println!(
                "\n{}: stopped after {} of {} — {reason}",
                report.name, report.logged_in, report.requested
            );
        }
    }
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(idle: Option<u64>, loaded: Option<u64>, players: usize) -> TargetReport {
        TargetReport {
            name: "test".to_owned(),
            requested: players,
            logged_in: players,
            elapsed: Duration::from_millis(1),
            join_bytes: None,
            idle_memory: idle.map(|bytes| ResidentMemory { bytes }),
            loaded_memory: loaded.map(|bytes| ResidentMemory { bytes }),
            first_failure: None,
        }
    }

    #[test]
    fn given_growth_across_a_hundred_players_when_divided_then_each_carries_its_share() {
        let measured = report(Some(1_000_000), Some(1_100_000), 100);

        assert_eq!(measured.per_player_bytes(), Some(1000.0));
    }

    #[test]
    fn given_memory_that_fell_during_the_run_then_no_per_player_figure_is_invented() {
        assert_eq!(
            report(Some(2_000_000), Some(1_000_000), 10).per_player_bytes(),
            None
        );
    }

    #[test]
    fn given_a_server_whose_memory_could_not_be_read_then_the_figure_is_absent() {
        assert_eq!(report(None, Some(1_000_000), 10).per_player_bytes(), None);
        assert_eq!(report(Some(1_000_000), None, 10).per_player_bytes(), None);
    }

    #[test]
    fn given_nobody_logged_in_then_dividing_by_them_is_not_attempted() {
        assert_eq!(report(Some(1), Some(2), 0).per_player_bytes(), None);
    }
}
