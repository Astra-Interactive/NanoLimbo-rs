use serde::Deserialize;

use crate::settings::schema::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct BrandNameDto {
    pub enable: bool,
    pub content: ScalarText,
}
