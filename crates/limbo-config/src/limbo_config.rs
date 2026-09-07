use std::net::SocketAddr;
use std::time::Duration;

use limbo_text::chat::Component;
use limbo_world::DimensionType;

use crate::boss_bar_config::BossBarConfig;
use crate::header_and_footer_config::HeaderAndFooterConfig;
use crate::info_forwarding::InfoForwarding;
use crate::ping_config::PingConfig;
use crate::player_list_config::PlayerListConfig;
use crate::runtime_config::RuntimeConfig;
use crate::title_config::TitleConfig;
use crate::traffic_config::TrafficConfig;

/// Everything `settings.yml` says, in the types the rest of the server thinks in.
///
/// A section the configuration switches off is `None` rather than a flag beside unused
/// values: the reference implementation only parsed the content of an enabled section,
/// and the enabled-but-empty state it left behind is not worth reproducing.
#[derive(Debug, Clone, PartialEq)]
pub struct LimboConfig {
    /// Where the listener binds. An empty `bind.ip` means every interface.
    pub bind_address: SocketAddr,
    /// Reported in the ping and in the join packet. Negative means no limit, and is sent
    /// to the client as it stands.
    pub max_players: i32,
    pub ping: PingConfig,
    pub dimension: DimensionType,
    /// 0 survival, 1 creative, 2 adventure, 3 spectator. Not narrowed to an enum because
    /// the reference implementation passes any number straight through to the client.
    pub game_mode: i32,
    /// Suppresses the insecure-chat toast on 1.20.5 and later.
    pub secure_profile: bool,
    pub player_list: PlayerListConfig,
    pub header_and_footer: Option<HeaderAndFooterConfig>,
    /// The server name shown under F3, for 1.13 and later.
    pub brand_name: Option<Component>,
    pub join_message: Option<Component>,
    pub boss_bar: Option<BossBarConfig>,
    pub title: Option<TitleConfig>,
    pub info_forwarding: InfoForwarding,
    /// How long a connection may stay silent, or `None` for no limit.
    pub read_timeout: Option<Duration>,
    /// When false, addresses are redacted in the log.
    pub log_players_ip: bool,
    /// 0 errors, 1 warnings, 2 info, 3 debug.
    pub debug_level: i32,
    pub runtime: RuntimeConfig,
    pub traffic: Option<TrafficConfig>,
}
