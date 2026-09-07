use serde::Deserialize;

use crate::dto::scalar_text::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct JoinMessageDto {
    pub enable: bool,
    pub text: ScalarText,
}
