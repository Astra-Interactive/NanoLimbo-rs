use tracing::Level;
use tracing_subscriber::EnvFilter;

/// Translates the configured `debugLevel` into a tracing level.
///
/// The numbering is the one `settings.yml` documents, kept so an existing configuration
/// keeps meaning what it meant. Anything outside the range is clamped rather than
/// rejected: a typo in a log setting is no reason to refuse to start.
pub fn level_from_debug_setting(debug_level: i32) -> Level {
    match debug_level {
        ..=0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        _ => Level::DEBUG,
    }
}

/// Installs the log subscriber.
///
/// `RUST_LOG` wins when set, so an operator can turn on detail for one module without
/// editing the configuration file.
pub fn install(debug_level: i32) {
    let level = level_from_debug_setting(debug_level);
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_unset| EnvFilter::new(level.to_string().to_lowercase()));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_documented_debug_level_when_translated_then_it_matches_the_published_meaning() {
        assert_eq!(level_from_debug_setting(0), Level::ERROR);
        assert_eq!(level_from_debug_setting(1), Level::WARN);
        assert_eq!(level_from_debug_setting(2), Level::INFO);
        assert_eq!(level_from_debug_setting(3), Level::DEBUG);
    }

    #[test]
    fn given_a_level_outside_the_documented_range_when_translated_then_it_clamps() {
        assert_eq!(level_from_debug_setting(-7), Level::ERROR);
        assert_eq!(level_from_debug_setting(99), Level::DEBUG);
    }
}
