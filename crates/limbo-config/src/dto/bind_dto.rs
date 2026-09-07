use serde::Deserialize;

use crate::dto::scalar_text::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct BindDto {
    /// Empty or absent means every interface.
    pub ip: Option<ScalarText>,
    pub port: u16,
}
