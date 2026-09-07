use std::net::SocketAddr;

use crate::bench_error::BenchError;

/// One server to measure.
///
/// The process id is optional because a server in a container, or on another machine, can
/// still be load-tested — only its memory goes unreported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchTarget {
    pub name: String,
    pub address: SocketAddr,
    pub pid: Option<u32>,
}

impl BenchTarget {
    /// Reads the `name=address[:pid]` form the command line uses.
    ///
    /// The process id is separated by `@` rather than `:`, which an address already uses.
    pub fn parse(argument: &str) -> Result<Self, BenchError> {
        let (name, rest) = argument
            .split_once('=')
            .ok_or_else(|| BenchError::MalformedTarget {
                argument: argument.to_owned(),
            })?;

        let (address_text, pid) = match rest.split_once('@') {
            Some((address, pid_text)) => {
                let pid = pid_text
                    .parse()
                    .map_err(|_invalid| BenchError::MalformedTarget {
                        argument: argument.to_owned(),
                    })?;
                (address, Some(pid))
            }
            None => (rest, None),
        };

        Ok(Self {
            name: name.to_owned(),
            address: address_text
                .parse()
                .map_err(|_invalid| BenchError::MalformedTarget {
                    argument: argument.to_owned(),
                })?,
            pid,
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
        assert_eq!(target.pid, None);
    }

    #[test]
    fn given_a_target_with_a_pid_when_parsed_then_memory_can_be_reported_for_it() {
        let target = BenchTarget::parse("java=127.0.0.1:25577@4321").expect("a valid target");

        assert_eq!(target.pid, Some(4321));
    }

    #[test]
    fn given_an_ipv6_address_when_parsed_then_its_colons_are_not_mistaken_for_a_pid() {
        let target = BenchTarget::parse("rust=[::1]:25565@99").expect("a valid target");

        assert_eq!(target.address.port(), 25565);
        assert_eq!(target.pid, Some(99));
    }

    #[test]
    fn given_something_that_is_not_a_target_when_parsed_then_it_is_rejected() {
        assert!(BenchTarget::parse("no-equals-sign").is_err());
        assert!(BenchTarget::parse("rust=not-an-address").is_err());
        assert!(BenchTarget::parse("rust=127.0.0.1:1@not-a-pid").is_err());
    }
}
