use serde::Deserialize;

use crate::settings::schema::ScalarText;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct JoinMessageDto {
    pub enable: bool,
    pub text: ScalarText,
}
