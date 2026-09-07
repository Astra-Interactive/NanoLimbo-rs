use std::time::Duration;

use limbo_protocol::version::ProtocolVersion;

use crate::bench_error::BenchError;
use crate::bench_target::BenchTarget;

/// How many players to log in when nothing is asked for.
const DEFAULT_PLAYERS: usize = 300;

/// How long to hold them there before reading memory, so the figure is not taken mid-burst.
const DEFAULT_SETTLE: Duration = Duration::from_secs(2);

/// How long to wait for a target to accept connections. Zero means it must already be up.
const DEFAULT_WAIT: Duration = Duration::ZERO;

/// What to measure, and against what.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchOptions {
    pub targets: Vec<BenchTarget>,
    pub players: usize,
    pub version: ProtocolVersion,
    pub settle: Duration,
    /// How long to keep retrying before giving up on a target that is not listening yet.
    ///
    /// Lets one tool both wait for a server and measure it, which is why the scripts
    /// around it no longer probe ports themselves.
    pub wait: Duration,
}

fn value_after(
    flag: &str,
    arguments: &mut impl Iterator<Item = String>,
) -> Result<String, BenchError> {
    arguments.next().ok_or_else(|| BenchError::MissingValue {
        flag: flag.to_owned(),
    })
}

fn parse_number<T: std::str::FromStr>(flag: &str, value: &str) -> Result<T, BenchError> {
    value
        .parse()
        .map_err(|_unreadable| BenchError::UnreadableValue {
            flag: flag.to_owned(),
            value: value.to_owned(),
        })
}

impl BenchOptions {
    /// Reads the command line.
    ///
    /// Hand-written rather than pulled from an argument crate: five flags do not justify a
    /// dependency that every build of the workspace would then have to compile, and the
    /// `name=address@pid` form needs its own parsing anyway.
    pub fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, BenchError> {
        let mut arguments = arguments.into_iter();
        let mut targets = Vec::new();
        let mut players = DEFAULT_PLAYERS;
        let mut version = ProtocolVersion::MAX;
        let mut settle = DEFAULT_SETTLE;
        let mut wait = DEFAULT_WAIT;

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--players" => {
                    let value = value_after("--players", &mut arguments)?;
                    players = parse_number("--players", &value)?;
                }
                "--protocol" => {
                    let value = value_after("--protocol", &mut arguments)?;
                    let number: i32 = parse_number("--protocol", &value)?;
                    version = ProtocolVersion::from_number(number)
                        .ok_or(BenchError::UnsupportedProtocol { number })?;
                }
                "--settle-seconds" => {
                    let value = value_after("--settle-seconds", &mut arguments)?;
                    settle = Duration::from_secs(parse_number("--settle-seconds", &value)?);
                }
                "--wait-seconds" => {
                    let value = value_after("--wait-seconds", &mut arguments)?;
                    wait = Duration::from_secs(parse_number("--wait-seconds", &value)?);
                }
                flag if flag.starts_with("--") => {
                    return Err(BenchError::UnknownOption {
                        flag: flag.to_owned(),
                    });
                }
                target => targets.push(BenchTarget::parse(target)?),
            }
        }

        if targets.is_empty() {
            return Err(BenchError::NoTargets);
        }

        Ok(Self {
            targets,
            players,
            version,
            settle,
            wait,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let options =
            parse(&["rust=127.0.0.1:1", "java=127.0.0.1:2@9"]).expect("a valid command line");

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
}
