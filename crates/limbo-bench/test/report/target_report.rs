use std::time::Duration;

use crate::memory::ResidentMemory;
use crate::report::TargetReport;

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
