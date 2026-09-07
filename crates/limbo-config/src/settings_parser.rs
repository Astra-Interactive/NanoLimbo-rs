use std::path::Path;
use std::time::Duration;

use limbo_text::chat::Component;
use limbo_world::DimensionType;

use crate::bind_address::resolve_bind_address;
use crate::boss_bar_config::BossBarConfig;
use crate::boss_bar_health::BossBarHealth;
use crate::config_file_system::ConfigFileSystem;
use crate::config_warning::ConfigWarning;
use crate::dimension_parser::parse_dimension_type;
use crate::dto::boss_bar_dto::BossBarDto;
use crate::dto::brand_name_dto::BrandNameDto;
use crate::dto::header_and_footer_dto::HeaderAndFooterDto;
use crate::dto::join_message_dto::JoinMessageDto;
use crate::dto::netty_dto::NettyDto;
use crate::dto::ping_dto::PingDto;
use crate::dto::player_list_dto::PlayerListDto;
use crate::dto::scalar_text::ScalarText;
use crate::dto::settings_dto::SettingsDto;
use crate::dto::title_dto::TitleDto;
use crate::dto::traffic_dto::TrafficDto;
use crate::header_and_footer_config::HeaderAndFooterConfig;
use crate::info_forwarding_resolver::resolve_info_forwarding;
use crate::limbo_config::LimboConfig;
use crate::loaded_config::LoadedConfig;
use crate::ping_config::PingConfig;
use crate::player_list_config::PlayerListConfig;
use crate::runtime_config::RuntimeConfig;
use crate::settings_error::SettingsError;
use crate::ticks::Ticks;
use crate::title_config::TitleConfig;
use crate::traffic_config::TrafficConfig;
use crate::transport_type::TransportType;

/// Every text setting goes through the same parser as the reference implementation's, so
/// MiniMessage, legacy colour codes and pasted JSON all keep working.
fn text(configured: &ScalarText) -> Component {
    limbo_text::parse(configured.as_str())
}

/// A timeout that a non-positive value switches off, which is what the idle handler the
/// reference implementation used did with anything at or below zero.
fn optional_millis(millis: i64) -> Option<Duration> {
    u64::try_from(millis)
        .ok()
        .filter(|&value| value > 0)
        .map(Duration::from_millis)
}

fn optional_seconds(seconds: f64) -> Option<Duration> {
    if seconds <= 0.0 {
        return None;
    }
    Duration::try_from_secs_f64(seconds).ok()
}

fn optional_rate(rate: f64) -> Option<f64> {
    (rate > 0.0).then_some(rate)
}

fn optional_size(size: i32) -> Option<u32> {
    u32::try_from(size).ok().filter(|&value| value > 0)
}

/// Netty read a thread count of zero as "size the pool yourself", and so does tokio.
fn optional_threads(threads: i32) -> Option<usize> {
    usize::try_from(threads).ok().filter(|&value| value > 0)
}

fn ping_config(dto: &PingDto) -> PingConfig {
    PingConfig {
        description: text(&dto.description),
        version: text(&dto.version),
        protocol: (dto.protocol > 0).then_some(dto.protocol),
    }
}

fn player_list_config(dto: &PlayerListDto) -> PlayerListConfig {
    PlayerListConfig {
        enabled: dto.enable,
        username: dto.username.as_str().to_owned(),
    }
}

fn header_and_footer_config(dto: &HeaderAndFooterDto) -> Option<HeaderAndFooterConfig> {
    dto.enable.then(|| HeaderAndFooterConfig {
        header: text(&dto.header),
        footer: text(&dto.footer),
    })
}

fn brand_name(dto: &BrandNameDto) -> Option<Component> {
    dto.enable.then(|| text(&dto.content))
}

fn join_message(dto: &JoinMessageDto) -> Option<Component> {
    dto.enable.then(|| text(&dto.text))
}

/// The colour and the division are only looked at for a boss bar that is switched on,
/// which is what lets a disabled block keep whatever it happens to say.
fn boss_bar_config(dto: &BossBarDto) -> Result<Option<BossBarConfig>, SettingsError> {
    if !dto.enable {
        return Ok(None);
    }

    Ok(Some(BossBarConfig {
        text: text(&dto.text),
        health: BossBarHealth::new(dto.health)?,
        color: dto.color.as_str().parse()?,
        division: dto.division.as_str().parse()?,
    }))
}

fn title_config(dto: &TitleDto) -> Option<TitleConfig> {
    dto.enable.then(|| TitleConfig {
        title: text(&dto.title),
        subtitle: text(&dto.subtitle),
        fade_in: Ticks::new(dto.fade_in),
        stay: Ticks::new(dto.stay),
        fade_out: Ticks::new(dto.fade_out),
    })
}

fn traffic_config(dto: &TrafficDto) -> Option<TrafficConfig> {
    dto.enable.then(|| TrafficConfig {
        max_packet_size: optional_size(dto.max_packet_size),
        interval: optional_seconds(dto.interval),
        max_packet_rate: optional_rate(dto.max_packet_rate),
        max_packet_bytes_rate: optional_rate(dto.max_packet_bytes_rate),
    })
}

fn dimension(configured: Option<&ScalarText>) -> Result<DimensionType, SettingsError> {
    configured.map_or(Ok(DimensionType::TheEnd), |value| {
        parse_dimension_type(value.as_str())
    })
}

fn runtime_config(dto: &NettyDto) -> Result<RuntimeConfig, SettingsError> {
    let transport_type = dto
        .transport_type
        .as_ref()
        .map_or(Ok(TransportType::Epoll), |value| value.as_str().parse())?;

    Ok(RuntimeConfig {
        transport_type,
        worker_threads: optional_threads(dto.threads.worker_group),
    })
}

fn runtime_warnings(transport_type: TransportType, boss_group: Option<i32>) -> Vec<ConfigWarning> {
    let mut warnings = Vec::new();

    if !transport_type.is_available() {
        warnings.push(ConfigWarning::UnavailableTransport {
            configured: transport_type,
        });
    }

    // A single accept loop is exactly what the runtime does, so the shipped default of 1
    // is not a setting being ignored — warning about it would train operators to ignore
    // startup warnings. Anything else genuinely will not be honoured.
    if let Some(configured) = boss_group.filter(|threads| *threads != 1) {
        warnings.push(ConfigWarning::AcceptThreadsIgnored { configured });
    }

    warnings
}

/// Reads the content of a `settings.yml`.
///
/// `root` is the directory an `@file` credential reference resolves against; it is the
/// working directory in a normal run. `file_system` is only touched for those references,
/// so a configuration without them is parsed without any I/O at all.
pub fn parse_settings<F: ConfigFileSystem>(
    yaml: &str,
    root: &Path,
    file_system: &F,
) -> Result<LoadedConfig, SettingsError> {
    // A file that is empty, or that has had everything commented out, is a YAML document
    // with no content rather than an empty mapping; it means every setting falls back.
    let settings = serde_yaml_ng::from_str::<Option<SettingsDto>>(yaml)?.unwrap_or_default();

    let runtime = runtime_config(&settings.netty)?;
    let mut warnings = runtime_warnings(runtime.transport_type, settings.netty.threads.boss_group);

    let forwarding = resolve_info_forwarding(&settings.info_forwarding, root, file_system)?;
    warnings.extend(forwarding.warning);

    let config = LimboConfig {
        bind_address: resolve_bind_address(
            settings.bind.ip.as_ref().map(ScalarText::as_str),
            settings.bind.port,
        )?,
        max_players: settings.max_players,
        ping: ping_config(&settings.ping),
        dimension: dimension(settings.dimension.as_ref())?,
        game_mode: settings.game_mode,
        secure_profile: settings.secure_profile,
        player_list: player_list_config(&settings.player_list),
        header_and_footer: header_and_footer_config(&settings.header_and_footer),
        brand_name: brand_name(&settings.brand_name),
        join_message: join_message(&settings.join_message),
        boss_bar: boss_bar_config(&settings.boss_bar)?,
        title: title_config(&settings.title),
        info_forwarding: forwarding.forwarding,
        read_timeout: optional_millis(settings.read_timeout),
        log_players_ip: settings.log_players_ip,
        debug_level: settings.debug_level,
        runtime,
        traffic: traffic_config(&settings.traffic),
    };

    Ok(LoadedConfig { config, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_disabled_limit_when_read_then_zero_and_negatives_both_switch_it_off() {
        assert_eq!(optional_millis(-1), None);
        assert_eq!(optional_millis(0), None);
        assert_eq!(optional_millis(30_000), Some(Duration::from_millis(30_000)));
    }

    #[test]
    fn given_an_interval_in_seconds_when_read_then_it_keeps_its_fraction() {
        assert_eq!(optional_seconds(7.0), Some(Duration::from_secs(7)));
        assert_eq!(optional_seconds(0.5), Some(Duration::from_millis(500)));
        assert_eq!(optional_seconds(-1.0), None);
    }

    /// A `Duration` cannot hold either, and building one from them panics, so both have
    /// to be filtered out before the conversion rather than after it.
    #[test]
    fn given_an_interval_that_is_not_a_number_when_read_then_it_is_simply_off() {
        assert_eq!(optional_seconds(f64::NAN), None);
        assert_eq!(optional_seconds(f64::INFINITY), None);
        assert_eq!(optional_rate(f64::NAN), None);
    }

    #[test]
    fn given_a_thread_count_of_zero_when_read_then_the_runtime_decides() {
        assert_eq!(optional_threads(0), None);
        assert_eq!(optional_threads(-4), None);
        assert_eq!(optional_threads(4), Some(4));
    }
}
