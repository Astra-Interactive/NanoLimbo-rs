//! What a `settings.yml` in the wild turns into.
//!
//! These go through the public API only, so they also stand in for the composition root:
//! if the crate is awkward to load a configuration with, it shows up here first.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use limbo_config::{
    BossBarColor, BossBarDivision, ConfigFileSystem, ConfigLoader, ConfigWarning, DEFAULT_SETTINGS,
    InfoForwarding, LoadedConfig, SETTINGS_FILE_NAME, SettingsError, Ticks, TransportType,
    parse_settings,
};
use limbo_text::chat::Component;
use limbo_world::DimensionType;

/// A filesystem held in memory, shared by every clone of the handle.
///
/// Configuration loading reads three kinds of file — the settings, a secret, a list of
/// tokens — and what matters about each is which path it was read from and what came
/// back. A real temporary directory would be testing the operating system instead.
#[derive(Clone)]
struct FakeFileSystem {
    files: Rc<RefCell<HashMap<PathBuf, Vec<u8>>>>,
}

impl FakeFileSystem {
    fn empty() -> Self {
        Self {
            files: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn put(&self, path: impl Into<PathBuf>, content: &str) {
        self.files
            .borrow_mut()
            .insert(path.into(), content.as_bytes().to_vec());
    }

    fn content_of(&self, path: impl AsRef<Path>) -> Option<Vec<u8>> {
        self.files.borrow().get(path.as_ref()).cloned()
    }
}

impl ConfigFileSystem for FakeFileSystem {
    fn exists(&self, path: &Path) -> bool {
        self.files.borrow().contains_key(path)
    }

    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        self.files.borrow().get(path).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is absent", path.display()),
            )
        })
    }

    fn write(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        self.files
            .borrow_mut()
            .insert(path.to_path_buf(), contents.to_vec());
        Ok(())
    }
}

const ROOT: &str = "/srv/limbo";

/// Said in every test that expects its configuration to be usable.
const MUST_LOAD: &str = "this configuration must load";

fn root() -> PathBuf {
    PathBuf::from(ROOT)
}

fn settings_path() -> PathBuf {
    root().join(SETTINGS_FILE_NAME)
}

/// Reads a file holding only the settings under test, so that the fallbacks are
/// exercised instead of whatever the shipped file happens to say.
fn read(yaml: &str) -> Result<LoadedConfig, SettingsError> {
    parse_settings(yaml, &root(), &FakeFileSystem::empty())
}

#[test]
fn given_no_configuration_file_when_loaded_then_the_shipped_default_is_written_and_used() {
    let file_system = FakeFileSystem::empty();

    let loaded = ConfigLoader::new(file_system.clone())
        .load(&root())
        .expect(MUST_LOAD);

    assert_eq!(
        file_system.content_of(settings_path()),
        Some(DEFAULT_SETTINGS.as_bytes().to_vec())
    );
    assert_eq!(loaded.config.max_players, 100);
}

#[test]
fn given_an_existing_configuration_file_when_loaded_then_it_is_not_overwritten() {
    let file_system = FakeFileSystem::empty();
    file_system.put(settings_path(), "maxPlayers: 7");

    let loaded = ConfigLoader::new(file_system.clone())
        .load(&root())
        .expect(MUST_LOAD);

    assert_eq!(loaded.config.max_players, 7);
    assert_eq!(
        file_system.content_of(settings_path()),
        Some(b"maxPlayers: 7".to_vec())
    );
}

#[test]
fn given_the_shipped_default_when_loaded_then_the_server_identity_is_as_published() {
    let config = read(DEFAULT_SETTINGS).expect(MUST_LOAD).config;

    assert!(config.bind_address.ip().is_loopback());
    assert_eq!(config.bind_address.port(), 65535);
    assert_eq!(config.max_players, 100);
    assert_eq!(config.ping.description.to_plain_text(), "NanoLimbo");
    assert_eq!(config.ping.version.to_plain_text(), "NanoLimbo");
    assert_eq!(config.ping.protocol, None);
    assert_eq!(config.dimension, DimensionType::TheEnd);
    assert_eq!(config.game_mode, 3);
    assert!(!config.secure_profile);
}

#[test]
fn given_the_shipped_default_when_loaded_then_every_message_is_switched_on_and_formatted() {
    let config = read(DEFAULT_SETTINGS).expect(MUST_LOAD).config;

    assert!(config.player_list.enabled);
    assert_eq!(config.player_list.username, "NanoLimbo");

    let header_and_footer = config
        .header_and_footer
        .expect("the shipped default fills the tab list header");
    assert_eq!(header_and_footer.header.to_plain_text(), "Welcome!");
    assert_eq!(header_and_footer.footer.to_plain_text(), "NanoLimbo");

    let brand_name = config.brand_name.expect("the shipped default sets a brand");
    assert_eq!(brand_name.to_plain_text(), "NanoLimbo");

    let join_message = config
        .join_message
        .expect("the shipped default sends a join message");
    assert_eq!(join_message.to_plain_text(), "Welcome to the NanoLimbo!");

    let boss_bar = config
        .boss_bar
        .expect("the shipped default shows a boss bar");
    assert_eq!(boss_bar.text.to_plain_text(), "Welcome to the NanoLimbo!");
    assert_eq!(boss_bar.health.value(), 1.0);
    assert_eq!(boss_bar.color, BossBarColor::Blue);
    assert_eq!(boss_bar.division, BossBarDivision::Solid);

    let title = config.title.expect("the shipped default shows a title");
    assert_eq!(title.title.to_plain_text(), "Welcome!");
    assert_eq!(title.subtitle.to_plain_text(), "NanoLimbo");
    assert_eq!(title.fade_in, Ticks::new(10));
    assert_eq!(title.stay, Ticks::new(100));
    assert_eq!(title.fade_out, Ticks::new(10));
}

/// The shipped file is written in MiniMessage. Comparing plain text alone would pass even
/// if the markup were stored verbatim, so one field is checked for having been styled.
#[test]
fn given_a_text_setting_with_markup_when_loaded_then_it_becomes_a_formatted_component() {
    let config = read(DEFAULT_SETTINGS).expect(MUST_LOAD).config;
    let title = config.title.expect("the shipped default shows a title");

    assert_ne!(title.title, Component::text("Welcome!"));
    assert_eq!(title.title.to_plain_text(), "Welcome!");
}

#[test]
fn given_the_shipped_default_when_loaded_then_the_connection_limits_are_as_published() {
    let config = read(DEFAULT_SETTINGS).expect(MUST_LOAD).config;

    assert_eq!(config.info_forwarding, InfoForwarding::None);
    assert_eq!(config.read_timeout, Some(Duration::from_secs(30)));
    assert!(config.log_players_ip);
    assert_eq!(config.debug_level, 2);
    assert_eq!(config.runtime.transport_type, TransportType::Epoll);
    assert_eq!(config.runtime.worker_threads, Some(4));

    let traffic = config.traffic.expect("the shipped default limits traffic");
    assert_eq!(traffic.max_packet_size, Some(8192));
    assert_eq!(traffic.interval, Some(Duration::from_secs(7)));
    assert_eq!(traffic.max_packet_rate, Some(500.0));
    assert_eq!(traffic.max_packet_bytes_rate, Some(2048.0));
    assert!(traffic.measures_rates());
}

/// A default deployment must start silently. The shipped file sets `bossGroup: 1`, and one
/// accept loop is exactly what the runtime provides, so nothing is being ignored. Warning
/// here would teach operators that startup warnings are noise.
#[test]
fn given_the_shipped_default_when_loaded_then_nothing_is_warned_about() {
    let warnings = read(DEFAULT_SETTINGS).expect(MUST_LOAD).warnings;

    assert_eq!(warnings, []);
}

#[test]
fn given_an_empty_configuration_when_loaded_then_every_setting_falls_back_as_published() {
    let config = read("{}").expect(MUST_LOAD).config;

    assert!(config.bind_address.ip().is_unspecified());
    assert_eq!(config.max_players, 100);
    assert_eq!(config.ping.protocol, None);
    assert_eq!(config.ping.description, Component::empty());
    assert_eq!(config.dimension, DimensionType::TheEnd);
    assert_eq!(config.game_mode, 3);
    assert!(!config.secure_profile);
    assert_eq!(config.read_timeout, Some(Duration::from_secs(30)));
    assert!(config.log_players_ip);
    assert_eq!(config.debug_level, 2);
    assert_eq!(config.runtime.transport_type, TransportType::Epoll);
    assert_eq!(config.runtime.worker_threads, Some(4));
    assert_eq!(config.info_forwarding, InfoForwarding::None);
}

/// Every section with an `enable` flag is off unless the file switches it on, and the
/// traffic block falls back to no limits rather than to the numbers the shipped file
/// happens to suggest.
#[test]
fn given_an_empty_configuration_when_loaded_then_no_optional_section_is_switched_on() {
    let config = read("{}").expect(MUST_LOAD).config;

    assert!(!config.player_list.enabled);
    assert_eq!(config.player_list.username, "");
    assert_eq!(config.header_and_footer, None);
    assert_eq!(config.brand_name, None);
    assert_eq!(config.join_message, None);
    assert_eq!(config.boss_bar, None);
    assert_eq!(config.title, None);
    assert_eq!(config.traffic, None);
}

#[test]
fn given_settings_that_are_not_recognised_when_loaded_then_they_are_ignored() {
    let config = read("maxPlayers: 5\nsomethingFromALaterVersion: true\n")
        .expect(MUST_LOAD)
        .config;

    assert_eq!(config.max_players, 5);
}

#[test]
fn given_an_unlimited_player_count_when_loaded_then_the_negative_number_is_kept() {
    let config = read("maxPlayers: -1").expect(MUST_LOAD).config;

    assert_eq!(config.max_players, -1);
}

#[test]
fn given_an_empty_bind_address_when_loaded_then_the_listener_takes_every_interface() {
    let config = read("bind:\n  ip: \"\"\n  port: 25565\n")
        .expect(MUST_LOAD)
        .config;

    assert!(config.bind_address.ip().is_unspecified());
    assert_eq!(config.bind_address.port(), 25565);
}

#[test]
fn given_a_static_ping_protocol_when_loaded_then_it_is_advertised_instead_of_the_client_version() {
    let fixed = read("ping:\n  protocol: 47\n").expect(MUST_LOAD).config;
    let unset = read("ping:\n  protocol: -1\n").expect(MUST_LOAD).config;
    let zero = read("ping:\n  protocol: 0\n").expect(MUST_LOAD).config;

    assert_eq!(fixed.ping.protocol, Some(47));
    assert_eq!(unset.ping.protocol, None);
    assert_eq!(zero.ping.protocol, None);
}

#[test]
fn given_the_shorthand_dimension_names_when_loaded_then_they_mean_the_long_ones() {
    let nether = read("dimension: NETHER").expect(MUST_LOAD).config;
    let end = read("dimension: END").expect(MUST_LOAD).config;
    let overworld = read("dimension: overworld").expect(MUST_LOAD).config;

    assert_eq!(nether.dimension, DimensionType::TheNether);
    assert_eq!(end.dimension, DimensionType::TheEnd);
    assert_eq!(overworld.dimension, DimensionType::Overworld);
}

#[test]
fn given_a_dimension_that_does_not_exist_when_loaded_then_the_server_refuses_to_start() {
    assert!(read("dimension: THE_AETHER").is_err());
}

#[test]
fn given_a_boss_bar_health_outside_the_bar_when_loaded_then_the_server_refuses_to_start() {
    for health in ["-0.1", "1.1"] {
        let yaml = format!(
            "bossBar:\n  enable: true\n  health: {health}\n  color: BLUE\n  division: SOLID\n"
        );

        assert!(read(&yaml).is_err(), "health {health} should be rejected");
    }
}

#[test]
fn given_a_boss_bar_colour_or_division_that_does_not_exist_when_loaded_then_it_is_refused() {
    let unknown_colour =
        "bossBar:\n  enable: true\n  health: 1.0\n  color: BURGUNDY\n  division: SOLID\n";
    let unknown_division =
        "bossBar:\n  enable: true\n  health: 1.0\n  color: BLUE\n  division: DASHES_7\n";

    assert!(read(unknown_colour).is_err());
    assert!(read(unknown_division).is_err());
}

/// The content of a switched-off section is never looked at, so a boss bar disabled while
/// it was being experimented with does not stop the server.
#[test]
fn given_a_disabled_boss_bar_with_nonsense_in_it_when_loaded_then_it_is_simply_absent() {
    let config = read("bossBar:\n  enable: false\n  health: 9.0\n  color: BURGUNDY\n")
        .expect(MUST_LOAD)
        .config;

    assert_eq!(config.boss_bar, None);
}

#[test]
fn given_a_boss_bar_without_a_health_when_loaded_then_it_is_empty_rather_than_full() {
    let config = read("bossBar:\n  enable: true\n  color: RED\n  division: DASHES_20\n")
        .expect(MUST_LOAD)
        .config;
    let boss_bar = config.boss_bar.expect("the boss bar is switched on");

    assert_eq!(boss_bar.health.value(), 0.0);
    assert_eq!(boss_bar.color, BossBarColor::Red);
    assert_eq!(boss_bar.division, BossBarDivision::Dashes20);
}

#[test]
fn given_a_title_without_timings_when_loaded_then_it_uses_the_published_ones() {
    let config = read("title:\n  enable: true\n  title: \"hello\"\n")
        .expect(MUST_LOAD)
        .config;
    let title = config.title.expect("the title is switched on");

    assert_eq!(title.fade_in, Ticks::new(10));
    assert_eq!(title.stay, Ticks::new(100));
    assert_eq!(title.fade_out, Ticks::new(10));
    assert_eq!(title.subtitle, Component::empty());
}

#[test]
fn given_traffic_limits_switched_on_but_left_at_minus_one_when_loaded_then_nothing_is_limited() {
    let config = read("traffic:\n  enable: true\n").expect(MUST_LOAD).config;
    let traffic = config.traffic.expect("the traffic block is switched on");

    assert_eq!(traffic.max_packet_size, None);
    assert_eq!(traffic.interval, None);
    assert_eq!(traffic.max_packet_rate, None);
    assert_eq!(traffic.max_packet_bytes_rate, None);
    assert!(!traffic.measures_rates());
}

/// Both rates are averages over the interval, so without one there is nothing to average.
#[test]
fn given_rate_limits_without_an_interval_when_loaded_then_no_rate_can_be_measured() {
    let config = read("traffic:\n  enable: true\n  maxPacketRate: 500.0\n")
        .expect(MUST_LOAD)
        .config;
    let traffic = config.traffic.expect("the traffic block is switched on");

    assert_eq!(traffic.max_packet_rate, Some(500.0));
    assert!(!traffic.measures_rates());
}

#[test]
fn given_a_read_timeout_of_zero_when_loaded_then_a_connection_may_stay_silent() {
    let zero = read("readTimeout: 0").expect(MUST_LOAD).config;
    let negative = read("readTimeout: -1").expect(MUST_LOAD).config;

    assert_eq!(zero.read_timeout, None);
    assert_eq!(negative.read_timeout, None);
}

#[test]
fn given_a_transport_the_runtime_cannot_provide_when_loaded_then_it_warns_and_carries_on() {
    let loaded = read("netty:\n  transportType: IO_URING\n").expect(MUST_LOAD);

    assert_eq!(loaded.config.runtime.transport_type, TransportType::IoUring);
    assert_eq!(
        loaded.warnings,
        [ConfigWarning::UnavailableTransport {
            configured: TransportType::IoUring
        }]
    );
}

#[test]
fn given_an_accept_thread_count_when_loaded_then_it_warns_that_there_is_no_such_pool() {
    let loaded =
        read("netty:\n  threads:\n    bossGroup: 2\n    workerGroup: 8\n").expect(MUST_LOAD);

    assert_eq!(loaded.config.runtime.worker_threads, Some(8));
    assert_eq!(
        loaded.warnings,
        [ConfigWarning::AcceptThreadsIgnored { configured: 2 }]
    );
}

#[test]
fn given_a_netty_block_left_alone_when_loaded_then_nothing_is_warned_about() {
    let loaded = read("netty:\n  transportType: KQUEUE\n").expect(MUST_LOAD);

    assert!(loaded.warnings.is_empty());
}

#[test]
fn given_a_transport_that_does_not_exist_when_loaded_then_the_server_refuses_to_start() {
    assert!(read("netty:\n  transportType: SNEAKERNET\n").is_err());
}

#[test]
fn given_an_inline_velocity_secret_when_loaded_then_it_is_taken_as_written() {
    let config = read("infoForwarding:\n  type: MODERN\n  secret: \"  spaced  \"\n")
        .expect(MUST_LOAD)
        .config;

    assert_eq!(
        config.info_forwarding.secret(),
        Some(b"  spaced  ".as_slice())
    );
}

#[test]
fn given_a_velocity_secret_in_a_file_when_loaded_then_it_is_read_and_trimmed() {
    let file_system = FakeFileSystem::empty();
    file_system.put(root().join("secret.txt"), "from-a-file\n");

    let loaded = parse_settings(
        "infoForwarding:\n  type: MODERN\n  secret: \"@secret.txt\"\n",
        &root(),
        &file_system,
    )
    .expect(MUST_LOAD);

    assert_eq!(
        loaded.config.info_forwarding.secret(),
        Some(b"from-a-file".as_slice())
    );
}

#[test]
fn given_a_secret_file_that_is_not_there_when_loaded_then_it_is_an_error_and_not_a_panic() {
    let failure = read("infoForwarding:\n  type: MODERN\n  secret: \"@missing.txt\"\n");

    let message = failure
        .expect_err("a missing secret file is fatal")
        .to_string();

    assert!(message.contains("missing.txt"), "{message}");
}

#[test]
fn given_bungee_guard_tokens_written_inline_when_loaded_then_each_one_is_accepted() {
    let config = read("infoForwarding:\n  type: BUNGEE_GUARD\n  tokens:\n  - first\n  - second\n")
        .expect(MUST_LOAD)
        .config;

    assert!(config.info_forwarding.has_token("first"));
    assert!(config.info_forwarding.has_token("second"));
    assert!(!config.info_forwarding.has_token("third"));
}

#[test]
fn given_a_token_file_named_on_its_own_when_loaded_then_its_lines_become_the_tokens() {
    let file_system = FakeFileSystem::empty();
    file_system.put(
        root().join("tokens.txt"),
        "# the lobby proxy\nfrom-file\n\n   indented   \n",
    );

    let loaded = parse_settings(
        "infoForwarding:\n  type: BUNGEE_GUARD\n  tokens: \"@tokens.txt\"\n",
        &root(),
        &file_system,
    )
    .expect(MUST_LOAD);

    assert!(loaded.config.info_forwarding.has_token("from-file"));
    assert!(loaded.config.info_forwarding.has_token("indented"));
    assert!(!loaded.config.info_forwarding.has_token("# the lobby proxy"));
    assert!(!loaded.config.info_forwarding.has_token(""));
}

#[test]
fn given_a_list_mixing_tokens_and_files_when_loaded_then_both_kinds_are_accepted() {
    let file_system = FakeFileSystem::empty();
    file_system.put(root().join("extra.txt"), "from-file\n");

    let loaded = parse_settings(
        "infoForwarding:\n  type: BUNGEE_GUARD\n  tokens:\n  - inline\n  - \"@extra.txt\"\n",
        &root(),
        &file_system,
    )
    .expect(MUST_LOAD);

    assert!(loaded.config.info_forwarding.has_token("inline"));
    assert!(loaded.config.info_forwarding.has_token("from-file"));
}

#[test]
fn given_a_token_file_that_is_not_there_when_loaded_then_it_is_an_error_and_not_a_panic() {
    let failure = read("infoForwarding:\n  type: BUNGEE_GUARD\n  tokens:\n  - \"@missing.txt\"\n");

    let message = failure
        .expect_err("a missing token file is fatal")
        .to_string();

    assert!(message.contains("missing.txt"), "{message}");
}

/// A single token written without a list loads nothing at all, which is what the
/// reference implementation does. Reading it as one token instead would let a player in
/// where the deployment had been rejecting everyone, so it is reported, not repaired.
#[test]
fn given_a_lone_token_written_without_a_list_when_loaded_then_it_warns_and_loads_none() {
    let loaded =
        read("infoForwarding:\n  type: BUNGEE_GUARD\n  tokens: only-token\n").expect(MUST_LOAD);

    assert!(!loaded.config.info_forwarding.has_token("only-token"));
    assert_eq!(loaded.warnings, [ConfigWarning::PlainTokensIgnored]);
}

#[test]
fn given_a_forwarding_type_that_does_not_exist_when_loaded_then_the_server_refuses_to_start() {
    assert!(read("infoForwarding:\n  type: TELEPATHY\n").is_err());
}

/// The shipped file leaves a placeholder secret and a placeholder token in place under
/// `type: NONE`; neither is looked at, and neither may stop the server.
#[test]
fn given_credentials_for_a_scheme_that_is_not_in_use_when_loaded_then_they_are_left_alone() {
    let config = read(
        "infoForwarding:\n  type: NONE\n  secret: \"@never-read.txt\"\n  tokens:\n  - \"@never-read.txt\"\n",
    )
    .expect(MUST_LOAD)
    .config;

    assert_eq!(config.info_forwarding, InfoForwarding::None);
}

#[test]
fn given_a_file_that_is_not_yaml_when_loaded_then_it_is_an_error_and_not_a_panic() {
    let broken = "bind:\n  ip: \"localhost\"\n    port: oops:\n  - [\n";

    assert!(read(broken).is_err());
}

#[test]
fn given_a_setting_of_the_wrong_shape_when_loaded_then_it_is_an_error_and_not_a_panic() {
    assert!(read("maxPlayers: many").is_err());
    assert!(read("bind:\n  port: 99999\n").is_err());
}

/// An operator who comments the whole file out is asking for the defaults, not for a
/// server that refuses to start.
#[test]
fn given_a_file_with_nothing_in_it_when_loaded_then_every_setting_falls_back() {
    for yaml in ["", "\n", "# everything is commented out\n"] {
        let config = read(yaml).expect(MUST_LOAD).config;

        assert_eq!(config.max_players, 100);
        assert_eq!(config.dimension, DimensionType::TheEnd);
    }
}
