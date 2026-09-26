use crate::chat::{Component, ComponentContent, NamedColor, Style, TextColor};

/// The character the client itself uses for formatting codes.
pub const SECTION_SIGN: char = '\u{00A7}';

/// The character configuration files conventionally use, so a `§` never has to be typed.
pub const AMPERSAND: char = '&';

fn format_code(code: char) -> Option<&'static str> {
    match code.to_ascii_lowercase() {
        'k' => Some("obfuscated"),
        'l' => Some("bold"),
        'm' => Some("strikethrough"),
        'n' => Some("underlined"),
        'o' => Some("italic"),
        'r' => Some("reset"),
        _ => None,
    }
}

/// Reads the `&x&r&r&g&g&b&b` form Bukkit invented for hex colours.
fn read_bukkit_hex(characters: &[char], start: usize) -> Option<String> {
    let mut digits = String::with_capacity(6);
    for step in 0..6 {
        let marker = characters.get(start + step * 2)?;
        let digit = characters.get(start + step * 2 + 1)?;
        if *marker != AMPERSAND && *marker != SECTION_SIGN || !digit.is_ascii_hexdigit() {
            return None;
        }
        digits.push(*digit);
    }
    Some(digits)
}

/// Rewrites legacy formatting codes as the MiniMessage tags that mean the same thing.
///
/// Doing the translation up front means one parser handles both notations, including the
/// mixtures that appear in configuration files people have carried between servers. The
/// reference implementation instead round-trips legacy text through a MiniMessage
/// serializer, which produces the same rendering by a longer route.
pub fn legacy_codes_to_tags(input: &str) -> String {
    let characters: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(input.len());
    let mut index = 0;

    while index < characters.len() {
        let Some(&character) = characters.get(index) else {
            break;
        };
        let is_marker = character == AMPERSAND || character == SECTION_SIGN;
        let next = characters.get(index + 1).copied();

        let Some(code) = next.filter(|_| is_marker) else {
            output.push(character);
            index += 1;
            continue;
        };

        if code.eq_ignore_ascii_case(&'x')
            && let Some(digits) = read_bukkit_hex(&characters, index + 2)
        {
            output.push_str(&format!("<#{digits}>"));
            index += 14;
            continue;
        }

        if let Some(color) = NamedColor::from_legacy_code(code) {
            output.push_str(&format!("<{}>", color.key()));
            index += 2;
            continue;
        }

        if let Some(tag) = format_code(code) {
            output.push_str(&format!("<{tag}>"));
            index += 2;
            continue;
        }

        output.push(character);
        index += 1;
    }

    output
}

/// Legacy decoration codes, in the order the reference implementation emits them.
const DECORATION_CODES: [char; 5] = ['k', 'l', 'm', 'n', 'o'];

/// What the receiving client currently has switched on.
struct LegacyState {
    color: Option<TextColor>,
    decorations: [bool; 5],
}

impl LegacyState {
    const fn cleared() -> Self {
        Self {
            color: None,
            decorations: [false; 5],
        }
    }
}

fn decorations_of(style: &Style) -> [bool; 5] {
    [
        style.obfuscated == Some(true),
        style.bold == Some(true),
        style.strikethrough == Some(true),
        style.underlined == Some(true),
        style.italic == Some(true),
    ]
}

fn inherit(parent: &Style, child: &Style) -> Style {
    let mut merged = parent.clone();
    if child.color.is_some() {
        merged.color = child.color;
    }
    for (target, source) in [
        (&mut merged.bold, child.bold),
        (&mut merged.italic, child.italic),
        (&mut merged.underlined, child.underlined),
        (&mut merged.strikethrough, child.strikethrough),
        (&mut merged.obfuscated, child.obfuscated),
    ] {
        if source.is_some() {
            *target = source;
        }
    }
    merged
}

fn push_code(output: &mut String, code: char) {
    output.push(SECTION_SIGN);
    output.push(code);
}

/// Emits the codes that move the client from `state` to `style`.
///
/// A colour code implicitly clears decorations, so switching colour re-emits whatever
/// decorations are still wanted. Turning a decoration *off* has no code of its own, which
/// is why the only way back is a full reset. Colours are compared by value, not by the
/// code they reduce to, so a gradient re-emits its code on every character even where
/// consecutive characters land on the same legacy colour.
fn apply_style(output: &mut String, state: &mut LegacyState, style: &Style) {
    let wanted = decorations_of(style);
    let must_remove = state
        .decorations
        .iter()
        .zip(wanted.iter())
        .any(|(active, keep)| *active && !*keep);

    if state.color != style.color {
        match style.color {
            Some(color) => {
                push_code(output, color.to_named().legacy_code());
                *state = LegacyState {
                    color: Some(color),
                    decorations: [false; 5],
                };
            }
            None => {
                push_code(output, 'r');
                *state = LegacyState::cleared();
            }
        }
    } else if must_remove {
        push_code(output, 'r');
        *state = LegacyState::cleared();
        if let Some(color) = style.color {
            push_code(output, color.to_named().legacy_code());
            state.color = Some(color);
        }
    }

    for (index, code) in DECORATION_CODES.into_iter().enumerate() {
        let enabled = wanted.get(index).copied().unwrap_or(false);
        let active = state.decorations.get(index).copied().unwrap_or(false);
        if enabled && !active {
            push_code(output, code);
            if let Some(slot) = state.decorations.get_mut(index) {
                *slot = true;
            }
        }
    }
}

fn append_legacy(
    component: &Component,
    inherited: &Style,
    state: &mut LegacyState,
    output: &mut String,
) {
    let resolved = inherit(inherited, &component.style);

    let body = match &component.content {
        ComponentContent::Text { text } => text.clone(),
        ComponentContent::Translatable { key, .. } => key.clone(),
        ComponentContent::Keybind { keybind } => keybind.clone(),
    };
    if !body.is_empty() {
        apply_style(output, state, &resolved);
        output.push_str(&body);
    }

    for child in &component.children {
        append_legacy(child, &resolved, state, output);
    }
}

/// Renders a component using the client's own section-sign formatting codes.
///
/// Used for the server brand and the version line of the status response, both of which
/// predate components and accept nothing else.
pub fn to_legacy_string(component: &Component) -> String {
    let mut output = String::new();
    let mut state = LegacyState::cleared();
    append_legacy(component, &Style::empty(), &mut state, &mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_legacy_colour_and_format_codes_when_translated_then_they_become_tags() {
        assert_eq!(
            legacy_codes_to_tags("&aGreen &lBold&r plain"),
            "<green>Green <bold>Bold<reset> plain"
        );
    }

    #[test]
    fn given_the_bukkit_hex_form_when_translated_then_it_becomes_one_hex_tag() {
        assert_eq!(
            legacy_codes_to_tags("&x&f&f&0&0&0&0hex legacy"),
            "<#ff0000>hex legacy"
        );
    }

    #[test]
    fn given_a_section_sign_when_translated_then_it_is_treated_like_an_ampersand() {
        assert_eq!(legacy_codes_to_tags("\u{00A7}aGreen"), "<green>Green");
    }

    #[test]
    fn given_a_marker_that_starts_nothing_when_translated_then_it_stays_literal() {
        assert_eq!(legacy_codes_to_tags("100% & rising"), "100% & rising");
        assert_eq!(legacy_codes_to_tags("trailing &"), "trailing &");
    }
}
