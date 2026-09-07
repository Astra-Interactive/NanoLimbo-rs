use std::fmt;

use crate::transport_type::TransportType;

/// A setting that was accepted but cannot be honoured.
///
/// Returned rather than logged so that the composition root decides how a warning is
/// surfaced — this crate has no logger, and a configuration reader that writes to stderr
/// cannot be tested for what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigWarning {
    /// `netty.transportType` names a transport the runtime does not implement.
    UnavailableTransport { configured: TransportType },
    /// `netty.threads.bossGroup` sizes a pool that does not exist here: the listener
    /// accepts connections on the same runtime as everything else.
    AcceptThreadsIgnored { configured: i32 },
    /// `infoForwarding.tokens` is a plain string rather than a list or an `@file`
    /// reference, which loads no tokens at all and rejects every player.
    PlainTokensIgnored,
}

impl fmt::Display for ConfigWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnavailableTransport { configured } => write!(
                formatter,
                "netty.transportType {} is not available; the runtime polls with epoll or kqueue, whichever the platform provides",
                configured.config_name()
            ),
            Self::AcceptThreadsIgnored { configured } => write!(
                formatter,
                "netty.threads.bossGroup ({configured}) is ignored; connections are accepted on the worker runtime instead of a separate pool"
            ),
            Self::PlainTokensIgnored => formatter.write_str(
                "infoForwarding.tokens is a single string, so no tokens were loaded; write a list of tokens, or '@file' to read them from a file",
            ),
        }
    }
}
