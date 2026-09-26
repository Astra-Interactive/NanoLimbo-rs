use limbo_text::chat::Component;

use crate::settings::boss_bar::BossBarColor;
use crate::settings::boss_bar::BossBarDivision;
use crate::settings::boss_bar::BossBarHealth;

/// The bar drawn across the top of the screen once the player is in.
#[derive(Debug, Clone, PartialEq)]
pub struct BossBarConfig {
    pub text: Component,
    pub health: BossBarHealth,
    pub color: BossBarColor,
    pub division: BossBarDivision,
}
