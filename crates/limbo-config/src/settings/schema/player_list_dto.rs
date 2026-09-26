use serde::Deserialize;

use crate::settings::schema::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct PlayerListDto {
    pub enable: bool,
    pub username: ScalarText,
}
