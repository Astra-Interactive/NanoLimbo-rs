use limbo_text::chat::Component;

use crate::boss_bar_color::BossBarColor;
use crate::boss_bar_division::BossBarDivision;
use crate::boss_bar_health::BossBarHealth;

/// The bar drawn across the top of the screen once the player is in.
#[derive(Debug, Clone, PartialEq)]
pub struct BossBarConfig {
    pub text: Component,
    pub health: BossBarHealth,
    pub color: BossBarColor,
    pub division: BossBarDivision,
}
