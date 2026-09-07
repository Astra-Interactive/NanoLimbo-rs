use serde::Deserialize;

use crate::dto::scalar_text::ScalarText;

#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TitleDto {
    pub enable: bool,
    pub title: ScalarText,
    pub subtitle: ScalarText,
    pub fade_in: i32,
    pub stay: i32,
    pub fade_out: i32,
}

impl Default for TitleDto {
    fn default() -> Self {
        Self {
            enable: false,
            title: ScalarText::default(),
            subtitle: ScalarText::default(),
            fade_in: 10,
            stay: 100,
            fade_out: 10,
        }
    }
}
