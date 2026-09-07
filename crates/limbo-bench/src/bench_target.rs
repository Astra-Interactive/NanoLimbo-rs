use std::net::SocketAddr;

use crate::bench_error::BenchError;
use crate::memory_source::MemorySource;

/// One server to measure.
///
/// The memory source is optional because a server on another machine can still be
/// load-tested — only its memory goes unreported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchTarget {
    pub name: String,
    pub address: SocketAddr,
    pub memory: Option<MemorySource>,
}

impl BenchTarget {
    /// Reads the `name=address[@pid|@container]` form the command line uses.
    ///
    /// Separated by `@` rather than `:`, which an address already uses.
    pub fn parse(argument: &str) -> Result<Self, BenchError> {
        let (name, rest) = argument
            .split_once('=')
            .ok_or_else(|| BenchError::MalformedTarget {
                argument: argument.to_owned(),
            })?;

        let (address_text, memory) = match rest.split_once('@') {
            Some((_address, "")) => {
                return Err(BenchError::MalformedTarget {
                    argument: argument.to_owned(),
                });
            }
            Some((address, suffix)) => (address, Some(MemorySource::parse(suffix))),
            None => (rest, None),
        };

        Ok(Self {
            name: name.to_owned(),
            address: address_text
                .parse()
                .map_err(|_invalid| BenchError::MalformedTarget {
                    argument: argument.to_owned(),
                })?,
            memory,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_target_without_a_pid_when_parsed_then_only_its_address_is_kept() {
        let target = BenchTarget::parse("rust=127.0.0.1:25565").expect("a valid target");

        assert_eq!(target.name, "rust");
        assert_eq!(target.address.port(), 25565);
        assert_eq!(target.memory, None);
    }

    #[test]
    fn given_a_container_name_when_parsed_then_memory_comes_from_docker() {
        let target = BenchTarget::parse("rust=127.0.0.1:25565@nanolimbo-1").expect("valid");

        assert_eq!(
            target.memory,
            Some(MemorySource::Container {
                name: "nanolimbo-1".to_owned()
            })
        );
    }

    #[test]
    fn given_a_target_with_a_pid_when_parsed_then_memory_can_be_reported_for_it() {
        let target = BenchTarget::parse("java=127.0.0.1:25577@4321").expect("a valid target");

        assert_eq!(target.memory, Some(MemorySource::Process { pid: 4321 }));
    }

    #[test]
    fn given_an_ipv6_address_when_parsed_then_its_colons_are_not_mistaken_for_a_pid() {
        let target = BenchTarget::parse("rust=[::1]:25565@99").expect("a valid target");

        assert_eq!(target.address.port(), 25565);
        assert_eq!(target.memory, Some(MemorySource::Process { pid: 99 }));
    }

    #[test]
    fn given_something_that_is_not_a_target_when_parsed_then_it_is_rejected() {
        assert!(BenchTarget::parse("no-equals-sign").is_err());
        assert!(BenchTarget::parse("rust=not-an-address").is_err());
        assert!(BenchTarget::parse("rust=127.0.0.1:1@").is_err());
    }
}
