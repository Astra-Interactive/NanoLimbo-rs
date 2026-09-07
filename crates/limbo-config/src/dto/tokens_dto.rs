use serde::Deserialize;

use crate::dto::scalar_text::ScalarText;

/// What `infoForwarding.tokens` may be written as.
///
/// A list of tokens, a list mixing tokens with `@file` references, or a bare `@file`
/// scalar naming a file of them. A bare scalar that is not an `@file` reference loads
/// nothing at all, which is what the reference implementation did.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TokensDto {
    Listed(Vec<ScalarText>),
    Single(ScalarText),
}

impl Default for TokensDto {
    fn default() -> Self {
        Self::Listed(Vec::new())
    }
}
