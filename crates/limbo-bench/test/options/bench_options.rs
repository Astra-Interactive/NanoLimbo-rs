use std::time::Duration;

use limbo_protocol::version::ProtocolVersion;

use crate::options::bench_options::{DEFAULT_PLAYERS, DEFAULT_SETTLE, DEFAULT_WAIT};
use crate::options::{BenchError, BenchOptions};

fn parse(arguments: &[&str]) -> Result<BenchOptions, BenchError> {
    BenchOptions::parse(arguments.iter().map(|argument| (*argument).to_owned()))
}

#[test]
fn given_only_targets_when_parsed_then_the_documented_defaults_apply() {
    let options = parse(&["rust=127.0.0.1:1"]).expect("a valid command line");

    assert_eq!(options.players, DEFAULT_PLAYERS);
    assert_eq!(options.version, ProtocolVersion::MAX);
    assert_eq!(options.settle, DEFAULT_SETTLE);
    assert_eq!(options.wait, DEFAULT_WAIT);
}

#[test]
fn given_a_wait_when_parsed_then_the_tool_will_retry_for_that_long() {
    let options =
        parse(&["--wait-seconds", "30", "rust=127.0.0.1:1"]).expect("a valid command line");

    assert_eq!(options.wait, Duration::from_secs(30));
}

#[test]
fn given_several_targets_when_parsed_then_all_of_them_are_measured() {
    let options = parse(&["rust=127.0.0.1:1", "java=127.0.0.1:2@9"]).expect("a valid command line");

    assert_eq!(options.targets.len(), 2);
    assert!(options.targets[1].memory.is_some());
}

#[test]
fn given_a_protocol_the_build_does_not_speak_when_parsed_then_it_is_refused() {
    let error = parse(&["--protocol", "9999", "rust=127.0.0.1:1"])
        .expect_err("an unsupported protocol must not be accepted");

    assert!(matches!(error, BenchError::UnsupportedProtocol { .. }));
}

#[test]
fn given_no_targets_when_parsed_then_it_refuses_rather_than_measuring_nothing() {
    assert!(matches!(
        parse(&["--players", "10"]),
        Err(BenchError::NoTargets)
    ));
}

#[test]
fn given_a_flag_without_its_value_when_parsed_then_it_is_reported() {
    assert!(matches!(
        parse(&["rust=127.0.0.1:1", "--players"]),
        Err(BenchError::MissingValue { .. })
    ));
}

#[test]
fn given_a_misspelt_option_when_parsed_then_it_is_not_taken_for_a_target() {
    assert!(matches!(
        parse(&["--playerz", "10", "rust=127.0.0.1:1"]),
        Err(BenchError::UnknownOption { .. })
    ));
}
