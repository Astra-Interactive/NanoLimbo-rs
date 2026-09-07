use std::path::Path;

use crate::config_file_system::ConfigFileSystem;
use crate::config_warning::ConfigWarning;
use crate::dto::info_forwarding_dto::InfoForwardingDto;
use crate::dto::scalar_text::ScalarText;
use crate::dto::tokens_dto::TokensDto;
use crate::info_forwarding::InfoForwarding;
use crate::resolved_forwarding::ResolvedForwarding;
use crate::settings_error::SettingsError;

/// Prefix that turns a credential into the name of a file holding it, so that a secret
/// can live in a Docker secret or a SOPS-decrypted file instead of in `settings.yml`.
const EXTERNAL_FILE_PREFIX: char = '@';

fn read_external_file<F: ConfigFileSystem>(
    root: &Path,
    reference: &str,
    file_system: &F,
) -> Result<String, SettingsError> {
    let path = root.join(reference);

    let content =
        file_system
            .read(&path)
            .map_err(|error| SettingsError::ExternalFileUnreadable {
                path: path.clone(),
                source: error,
            })?;

    String::from_utf8(content).map_err(|error| SettingsError::ExternalFileNotUtf8 {
        path,
        source: error,
    })
}

/// One token per line. Blank lines and `#` comments are skipped so the file can say which
/// proxy each token belongs to.
fn token_lines(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// The secret is trimmed when it comes from a file, because a text editor puts a newline
/// at the end of one and that newline is not part of the key. An inline secret is taken
/// exactly as written.
fn resolve_secret<F: ConfigFileSystem>(
    configured: &ScalarText,
    root: &Path,
    file_system: &F,
) -> Result<Vec<u8>, SettingsError> {
    let Some(reference) = configured.as_str().strip_prefix(EXTERNAL_FILE_PREFIX) else {
        return Ok(configured.as_str().as_bytes().to_vec());
    };

    let content = read_external_file(root, reference, file_system)?;
    Ok(content.trim().as_bytes().to_vec())
}

fn resolve_listed_tokens<F: ConfigFileSystem>(
    entries: &[ScalarText],
    root: &Path,
    file_system: &F,
) -> Result<Vec<String>, SettingsError> {
    let mut tokens = Vec::with_capacity(entries.len());

    for entry in entries {
        match entry.as_str().strip_prefix(EXTERNAL_FILE_PREFIX) {
            Some(reference) => {
                tokens.extend(token_lines(&read_external_file(
                    root,
                    reference,
                    file_system,
                )?));
            }
            None => tokens.push(entry.as_str().to_owned()),
        }
    }

    Ok(tokens)
}

fn resolve_bungee_guard<F: ConfigFileSystem>(
    configured: &TokensDto,
    root: &Path,
    file_system: &F,
) -> Result<ResolvedForwarding, SettingsError> {
    let single = match configured {
        TokensDto::Listed(entries) => {
            return Ok(ResolvedForwarding {
                forwarding: InfoForwarding::BungeeGuard {
                    tokens: resolve_listed_tokens(entries, root, file_system)?,
                },
                warning: None,
            });
        }
        TokensDto::Single(value) => value.as_str(),
    };

    let Some(reference) = single.strip_prefix(EXTERNAL_FILE_PREFIX) else {
        // A lone token written without a list loads nothing, which is what the reference
        // implementation does. Left alone rather than fixed, because reading it as one
        // token would let a player in where the deployment used to reject everyone.
        return Ok(ResolvedForwarding {
            forwarding: InfoForwarding::BungeeGuard { tokens: Vec::new() },
            warning: (!single.is_empty()).then_some(ConfigWarning::PlainTokensIgnored),
        });
    };

    Ok(ResolvedForwarding {
        forwarding: InfoForwarding::BungeeGuard {
            tokens: token_lines(&read_external_file(root, reference, file_system)?),
        },
        warning: None,
    })
}

/// Reads the `infoForwarding` block, resolving whichever credential the chosen scheme
/// needs and ignoring the other, exactly as the reference implementation does — so a file
/// that still carries the shipped `<YOUR_SECRET_HERE>` placeholder under `type: NONE`
/// loads without complaint.
pub fn resolve_info_forwarding<F: ConfigFileSystem>(
    dto: &InfoForwardingDto,
    root: &Path,
    file_system: &F,
) -> Result<ResolvedForwarding, SettingsError> {
    let configured = dto
        .forwarding_type
        .as_ref()
        .map_or("NONE", ScalarText::as_str);

    let without_warning = |forwarding: InfoForwarding| ResolvedForwarding {
        forwarding,
        warning: None,
    };

    match configured.to_ascii_uppercase().as_str() {
        "NONE" => Ok(without_warning(InfoForwarding::None)),
        "LEGACY" => Ok(without_warning(InfoForwarding::Legacy)),
        "MODERN" => Ok(without_warning(InfoForwarding::Modern {
            secret: resolve_secret(&dto.secret, root, file_system)?,
        })),
        "BUNGEE_GUARD" => resolve_bungee_guard(&dto.tokens, root, file_system),
        _ => Err(SettingsError::UnknownForwardingType {
            value: configured.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_token_file_when_its_lines_are_read_then_comments_and_blanks_are_skipped() {
        let content = "# proxy one\nfirst\n\n   second   \n#trailing note\n";

        assert_eq!(token_lines(content), ["first", "second"]);
    }

    #[test]
    fn given_a_file_of_only_comments_when_its_lines_are_read_then_there_are_no_tokens() {
        assert!(token_lines("# nothing here\n\n").is_empty());
    }
}
