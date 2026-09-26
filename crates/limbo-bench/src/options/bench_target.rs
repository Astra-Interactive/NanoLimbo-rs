use std::net::SocketAddr;

use crate::memory::MemorySource;
use crate::options::BenchError;

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
