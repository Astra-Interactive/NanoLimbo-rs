use std::time::Duration;

use crate::resident_memory::ResidentMemory;

/// What one server did when the players arrived.
pub struct TargetReport {
    pub name: String,
    pub requested: usize,
    pub logged_in: usize,
    pub elapsed: Duration,
    pub bytes_received: usize,
    pub idle_memory: Option<ResidentMemory>,
    pub loaded_memory: Option<ResidentMemory>,
    /// Why players stopped arriving, when not all of them did.
    pub first_failure: Option<String>,
}

impl TargetReport {
    /// How much the server sent to get one player into the world.
    ///
    /// Worth watching beside memory: a version that suddenly costs far more bytes per
    /// join usually means a registry is being resent that used to be shared.
    pub fn bytes_per_player(&self) -> Option<f64> {
        let players = u32::try_from(self.logged_in)
            .ok()
            .filter(|count| *count > 0)?;
        Some(self.bytes_received as f64 / f64::from(players))
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

/// Prints the comparison as a table, widest column first so the numbers line up.
pub fn print_table(reports: &[TargetReport]) {
    println!(
        "\n{:<10} {:>7} {:>7} {:>9} {:>10} {:>11} {:>12} {:>12}",
        "target", "asked", "joined", "time", "idle", "loaded", "mem/player", "sent/player"
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
            kilobytes(report.bytes_per_player()),
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
            bytes_received: 0,
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
        assert_eq!(report(Some(1), Some(2), 0).bytes_per_player(), None);
    }

    #[test]
    fn given_bytes_across_several_players_when_divided_then_each_carries_its_share() {
        let mut measured = report(None, None, 4);
        measured.bytes_received = 8000;

        assert_eq!(measured.bytes_per_player(), Some(2000.0));
    }
}
