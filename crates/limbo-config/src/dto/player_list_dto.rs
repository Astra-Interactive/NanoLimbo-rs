use serde::Deserialize;

use crate::dto::scalar_text::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct PlayerListDto {
    pub enable: bool,
    pub username: ScalarText,
}
