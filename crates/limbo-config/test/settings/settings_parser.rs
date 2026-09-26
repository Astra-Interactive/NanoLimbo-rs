use std::time::Duration;

use crate::settings::settings_parser::{
    optional_millis, optional_rate, optional_seconds, optional_threads,
};

#[test]
fn given_a_disabled_limit_when_read_then_zero_and_negatives_both_switch_it_off() {
    assert_eq!(optional_millis(-1), None);
    assert_eq!(optional_millis(0), None);
    assert_eq!(optional_millis(30_000), Some(Duration::from_millis(30_000)));
}

#[test]
fn given_an_interval_in_seconds_when_read_then_it_keeps_its_fraction() {
    assert_eq!(optional_seconds(7.0), Some(Duration::from_secs(7)));
    assert_eq!(optional_seconds(0.5), Some(Duration::from_millis(500)));
    assert_eq!(optional_seconds(-1.0), None);
}

/// A `Duration` cannot hold either, and building one from them panics, so both have
/// to be filtered out before the conversion rather than after it.
#[test]
fn given_an_interval_that_is_not_a_number_when_read_then_it_is_simply_off() {
    assert_eq!(optional_seconds(f64::NAN), None);
    assert_eq!(optional_seconds(f64::INFINITY), None);
    assert_eq!(optional_rate(f64::NAN), None);
}

#[test]
fn given_a_thread_count_of_zero_when_read_then_the_runtime_decides() {
    assert_eq!(optional_threads(0), None);
    assert_eq!(optional_threads(-4), None);
    assert_eq!(optional_threads(4), Some(4));
}
