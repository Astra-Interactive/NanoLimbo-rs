use serde::Deserialize;

use crate::settings::schema::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct HeaderAndFooterDto {
    pub enable: bool,
    pub header: ScalarText,
    pub footer: ScalarText,
}
