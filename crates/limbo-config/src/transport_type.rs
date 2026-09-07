use std::str::FromStr;

use crate::settings_error::SettingsError;

/// The socket transport the configuration asks for.
///
/// Tokio picks the polling mechanism for the platform on its own — epoll on Linux, kqueue
/// on the BSDs — so this value cannot steer anything the way Netty's channel factory did.
/// It is kept because an existing `settings.yml` still carries it, and because a request
/// for `IO_URING` is worth a warning rather than silence: that one is a genuinely
/// different I/O model the runtime does not offer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    Nio,
    Epoll,
    IoUring,
    KQueue,
}

impl TransportType {
    /// The name as it is spelled in `settings.yml`.
    pub const fn config_name(self) -> &'static str {
        match self {
            Self::Nio => "NIO",
            Self::Epoll => "EPOLL",
            Self::IoUring => "IO_URING",
            Self::KQueue => "KQUEUE",
        }
    }

    /// Whether the runtime can plausibly be doing what this asks for.
    ///
    /// Readiness-based transports all describe what tokio already does; `io_uring` is
    /// completion-based and has no equivalent here.
    pub const fn is_available(self) -> bool {
        !matches!(self, Self::IoUring)
    }
}

impl FromStr for TransportType {
    type Err = SettingsError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_uppercase().as_str() {
            "NIO" => Ok(Self::Nio),
            "EPOLL" => Ok(Self::Epoll),
            "IO_URING" => Ok(Self::IoUring),
            "KQUEUE" => Ok(Self::KQueue),
            _ => Err(SettingsError::UnknownTransportType {
                value: value.to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_every_spelling_in_the_shipped_comment_when_parsed_then_all_are_recognised() {
        for name in ["EPOLL", "IO_URING", "KQUEUE", "NIO"] {
            assert_eq!(
                name.parse::<TransportType>()
                    .map(TransportType::config_name)
                    .ok(),
                Some(name)
            );
        }
    }

    #[test]
    fn given_a_transport_that_is_not_offered_when_parsed_then_it_is_rejected() {
        assert!("SPINNING_RUST".parse::<TransportType>().is_err());
    }

    #[test]
    fn given_io_uring_when_asked_whether_the_runtime_offers_it_then_it_does_not() {
        assert!(!TransportType::IoUring.is_available());
        assert!(TransportType::Epoll.is_available());
        assert!(TransportType::KQueue.is_available());
        assert!(TransportType::Nio.is_available());
    }
}
