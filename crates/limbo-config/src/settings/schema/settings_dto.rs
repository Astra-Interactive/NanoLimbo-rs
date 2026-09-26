use serde::Deserialize;

use crate::settings::schema::BindDto;
use crate::settings::schema::BossBarDto;
use crate::settings::schema::BrandNameDto;
use crate::settings::schema::HeaderAndFooterDto;
use crate::settings::schema::InfoForwardingDto;
use crate::settings::schema::JoinMessageDto;
use crate::settings::schema::NettyDto;
use crate::settings::schema::PingDto;
use crate::settings::schema::PlayerListDto;
use crate::settings::schema::ScalarText;
use crate::settings::schema::TitleDto;
use crate::settings::schema::TrafficDto;

/// `settings.yml`, key for key.
///
/// Unknown keys are ignored rather than rejected, so a file written for a later version,
/// or one carrying a comment-shaped leftover, still loads.
#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SettingsDto {
    pub bind: BindDto,
    pub max_players: i32,
    pub ping: PingDto,
    /// Absent means the end, which is what the reference implementation defaulted to.
    pub dimension: Option<ScalarText>,
    pub game_mode: i32,
    pub secure_profile: bool,
    pub player_list: PlayerListDto,
    pub header_and_footer: HeaderAndFooterDto,
    pub brand_name: BrandNameDto,
    pub join_message: JoinMessageDto,
    pub boss_bar: BossBarDto,
    pub title: TitleDto,
    pub info_forwarding: InfoForwardingDto,
    /// Milliseconds.
    pub read_timeout: i64,
    pub log_players_ip: bool,
    pub debug_level: i32,
    pub netty: NettyDto,
    pub traffic: TrafficDto,
}

impl Default for SettingsDto {
    fn default() -> Self {
        Self {
            bind: BindDto::default(),
            max_players: 100,
            ping: PingDto::default(),
            dimension: None,
            game_mode: 3,
            secure_profile: false,
            player_list: PlayerListDto::default(),
            header_and_footer: HeaderAndFooterDto::default(),
            brand_name: BrandNameDto::default(),
            join_message: JoinMessageDto::default(),
            boss_bar: BossBarDto::default(),
            title: TitleDto::default(),
            info_forwarding: InfoForwardingDto::default(),
            read_timeout: 30_000,
            log_players_ip: true,
            debug_level: 2,
            netty: NettyDto::default(),
            traffic: TrafficDto::default(),
        }
    }
}
