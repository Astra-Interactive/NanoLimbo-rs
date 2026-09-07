use std::fmt;

/// How the player's real identity reaches this server from the proxy in front of it.
///
/// The credential each scheme needs lives in the variant that uses it, so there is no
/// state in which a secret is configured but unused, or needed but absent.
#[derive(Clone, PartialEq, Eq)]
pub enum InfoForwarding {
    /// Players connect directly; whatever the handshake says is taken at face value.
    None,
    /// BungeeCord's extra handshake fields, unauthenticated.
    Legacy,
    /// Velocity's signed forwarding data.
    Modern { secret: Vec<u8> },
    /// BungeeCord with the BungeeGuard plugin: the handshake carries a shared token.
    BungeeGuard { tokens: Vec<String> },
}

impl InfoForwarding {
    /// The key the Velocity forwarding signature is verified with.
    pub fn secret(&self) -> Option<&[u8]> {
        match self {
            Self::Modern { secret } => Some(secret),
            _ => None,
        }
    }

    /// Whether a token presented by a client is one of the configured ones.
    pub fn has_token(&self, token: &str) -> bool {
        match self {
            Self::BungeeGuard { tokens } => tokens.iter().any(|configured| configured == token),
            _ => false,
        }
    }
}

/// Written by hand so that a debug dump of the configuration — which a composition root
/// is likely to log at startup — cannot leak the Velocity secret or the BungeeGuard
/// tokens. Both are credentials that grant a bypass of the proxy's authentication.
impl fmt::Debug for InfoForwarding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("None"),
            Self::Legacy => formatter.write_str("Legacy"),
            Self::Modern { secret } => formatter
                .debug_struct("Modern")
                .field("secret", &format_args!("<{} bytes redacted>", secret.len()))
                .finish(),
            Self::BungeeGuard { tokens } => formatter
                .debug_struct("BungeeGuard")
                .field("tokens", &format_args!("<{} redacted>", tokens.len()))
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_bungee_guard_tokens_when_one_is_presented_then_only_a_configured_one_matches() {
        let forwarding = InfoForwarding::BungeeGuard {
            tokens: vec!["first".to_owned(), "second".to_owned()],
        };

        assert!(forwarding.has_token("second"));
        assert!(!forwarding.has_token("third"));
        assert!(!forwarding.has_token(""));
    }

    #[test]
    fn given_a_scheme_without_tokens_when_a_token_is_presented_then_it_never_matches() {
        assert!(!InfoForwarding::Legacy.has_token("anything"));
        assert!(
            !InfoForwarding::Modern {
                secret: b"anything".to_vec(),
            }
            .has_token("anything")
        );
    }

    #[test]
    fn given_credentials_when_the_configuration_is_debug_printed_then_they_do_not_appear() {
        let modern = InfoForwarding::Modern {
            secret: b"velocity-secret".to_vec(),
        };
        let bungee_guard = InfoForwarding::BungeeGuard {
            tokens: vec!["bungee-token".to_owned()],
        };

        assert!(!format!("{modern:?}").contains("velocity-secret"));
        assert!(!format!("{bungee_guard:?}").contains("bungee-token"));
    }

    #[test]
    fn given_modern_forwarding_when_the_secret_is_read_then_it_is_the_configured_key() {
        let forwarding = InfoForwarding::Modern {
            secret: b"key".to_vec(),
        };

        assert_eq!(forwarding.secret(), Some(b"key".as_slice()));
        assert_eq!(InfoForwarding::None.secret(), None);
    }
}
