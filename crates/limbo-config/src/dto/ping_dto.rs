use serde::Deserialize;

use crate::dto::scalar_text::ScalarText;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct PingDto {
    pub description: ScalarText,
    pub version: ScalarText,
    /// Negative means "answer with the protocol number the client sent".
    pub protocol: i32,
}

impl Default for PingDto {
    fn default() -> Self {
        Self {
            description: ScalarText::default(),
            version: ScalarText::default(),
            protocol: -1,
        }
    }
}
