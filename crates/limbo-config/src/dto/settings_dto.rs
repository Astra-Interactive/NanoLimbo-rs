use serde::Deserialize;

use crate::dto::bind_dto::BindDto;
use crate::dto::boss_bar_dto::BossBarDto;
use crate::dto::brand_name_dto::BrandNameDto;
use crate::dto::header_and_footer_dto::HeaderAndFooterDto;
use crate::dto::info_forwarding_dto::InfoForwardingDto;
use crate::dto::join_message_dto::JoinMessageDto;
use crate::dto::netty_dto::NettyDto;
use crate::dto::ping_dto::PingDto;
use crate::dto::player_list_dto::PlayerListDto;
use crate::dto::scalar_text::ScalarText;
use crate::dto::title_dto::TitleDto;
use crate::dto::traffic_dto::TrafficDto;

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
