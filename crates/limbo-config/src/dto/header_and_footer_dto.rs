use serde::Deserialize;

use crate::dto::scalar_text::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct HeaderAndFooterDto {
    pub enable: bool,
    pub header: ScalarText,
    pub footer: ScalarText,
}
