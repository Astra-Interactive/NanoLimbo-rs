use serde::Deserialize;

use crate::dto::scalar_text::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct BrandNameDto {
    pub enable: bool,
    pub content: ScalarText,
}
