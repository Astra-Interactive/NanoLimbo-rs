use crate::settings::forwarding::InfoForwarding;

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
