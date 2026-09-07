/// The name the configuration file has always had.
pub const SETTINGS_FILE_NAME: &str = "settings.yml";

/// The configuration written on first run.
///
/// Compiled in rather than shipped beside the binary: the server is one static file, and
/// the reference implementation read this same content out of its jar.
pub const DEFAULT_SETTINGS: &str = include_str!("../resources/settings.yml");
