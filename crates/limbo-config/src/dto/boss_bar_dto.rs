use serde::Deserialize;

use crate::dto::scalar_text::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct BossBarDto {
    pub enable: bool,
    pub text: ScalarText,
    pub health: f32,
    pub color: ScalarText,
    pub division: ScalarText,
}
