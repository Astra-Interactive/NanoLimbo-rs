use serde::Deserialize;

use crate::settings::schema::ScalarText;
use crate::settings::schema::TokensDto;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct InfoForwardingDto {
    #[serde(rename = "type")]
    pub forwarding_type: Option<ScalarText>,
    /// Used by `MODERN`. An `@`-prefixed value names a file to read it from.
    pub secret: ScalarText,
    /// Used by `BUNGEE_GUARD`.
    pub tokens: TokensDto,
}
