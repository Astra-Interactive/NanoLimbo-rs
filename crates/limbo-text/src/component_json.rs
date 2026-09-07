use serde_json::{Map, Value};

use crate::chat::{
    ClickEvent, Component, ComponentContent, HoverEvent, NamedColor, Style, TextColor,
};
use crate::json_profile::JsonProfile;

/// Serializes a colour the way the client for this profile expects it.
///
/// A colour that exactly matches one of the sixteen named colours is always written by
/// name, even where hex is allowed — the reference implementation does the same, and the
/// difference is visible in the bytes.
fn color_value(color: TextColor, profile: JsonProfile) -> Value {
    if !profile.emits_rgb() {
        return Value::String(color.to_named().key().to_owned());
    }

    let rgb = color.rgb();
    match NamedColor::ALL.into_iter().find(|named| named.rgb() == rgb) {
        Some(named) => Value::String(named.key().to_owned()),
        None => Value::String(format!("#{:02X}{:02X}{:02X}", rgb.red, rgb.green, rgb.blue)),
    }
}

fn click_event_value(event: &ClickEvent, profile: JsonProfile) -> Value {
    let mut object = Map::new();
    object.insert(
        "action".to_owned(),
        Value::String(event.action().to_owned()),
    );
    object.insert(
        profile.click_payload_key(event).to_owned(),
        Value::String(event.value().to_owned()),
    );
    Value::Object(object)
}

/// Writes the parts of a style that are set, leaving the rest absent so the component
/// inherits them.
fn insert_style(
    target: &mut Map<String, Value>,
    style: &Style,
    profile: JsonProfile,
    hover_payload: impl Fn(&Component) -> Value,
) {
    let decorations = [
        ("bold", style.bold),
        ("italic", style.italic),
        ("underlined", style.underlined),
        ("strikethrough", style.strikethrough),
        ("obfuscated", style.obfuscated),
    ];
    for (key, enabled) in decorations {
        if let Some(value) = enabled {
            target.insert(key.to_owned(), Value::Bool(value));
        }
    }

    if let Some(color) = style.color {
        target.insert("color".to_owned(), color_value(color, profile));
    }
    if let Some(font) = &style.font {
        target.insert("font".to_owned(), Value::String(font.clone()));
    }
    if let Some(insertion) = &style.insertion {
        target.insert("insertion".to_owned(), Value::String(insertion.clone()));
    }
    if let Some(click) = &style.click_event {
        target.insert(
            profile.click_event_key().to_owned(),
            click_event_value(click, profile),
        );
    }
    if let Some(HoverEvent::ShowText { text }) = &style.hover_event {
        let mut hover = Map::new();
        hover.insert("action".to_owned(), Value::String("show_text".to_owned()));
        hover.insert(profile.hover_text_key().to_owned(), hover_payload(text));
        target.insert(profile.hover_event_key().to_owned(), Value::Object(hover));
    }
}

/// Whether this component can be written as a bare string instead of an object.
fn is_bare_text(component: &Component) -> bool {
    matches!(component.content, ComponentContent::Text { .. })
        && component.style.is_empty()
        && component.children.is_empty()
}

/// Builds the JSON tree for a component.
///
/// `compaction` is separate from the profile because the two consumers disagree. The JSON
/// wire format collapses unstyled text to a bare string from 1.20.3; the NBT wire format
/// must not, because a bare string has nowhere to live inside an NBT list of compounds.
pub(crate) fn build_value(component: &Component, profile: JsonProfile, compaction: bool) -> Value {
    if compaction
        && profile.compacts_plain_text()
        && let ComponentContent::Text { text } = &component.content
        && is_bare_text(component)
    {
        return Value::String(text.clone());
    }

    let mut object = Map::new();

    match &component.content {
        ComponentContent::Text { text } => {
            object.insert("text".to_owned(), Value::String(text.clone()));
        }
        ComponentContent::Translatable { key, arguments } => {
            object.insert("translate".to_owned(), Value::String(key.clone()));
            if !arguments.is_empty() {
                object.insert(
                    "with".to_owned(),
                    Value::Array(
                        arguments
                            .iter()
                            .map(|argument| build_value(argument, profile, compaction))
                            .collect(),
                    ),
                );
            }
        }
        ComponentContent::Keybind { keybind } => {
            object.insert("keybind".to_owned(), Value::String(keybind.clone()));
        }
    }

    insert_style(&mut object, &component.style, profile, |hover| {
        build_value(hover, profile, compaction)
    });

    if !component.children.is_empty() {
        object.insert(
            "extra".to_owned(),
            Value::Array(
                component
                    .children
                    .iter()
                    .map(|child| build_value(child, profile, compaction))
                    .collect(),
            ),
        );
    }

    Value::Object(object)
}

/// Serializes a component to the JSON the given client version understands.
///
/// Keys come out in alphabetical order because `serde_json` backs its maps with a
/// `BTreeMap`, which is what the reference implementation emits too. Enabling the
/// `preserve_order` feature would silently change the bytes.
pub fn to_json_string(component: &Component, profile: JsonProfile) -> String {
    build_value(component, profile, true).to_string()
}

fn color_from_json(value: &Value) -> Option<TextColor> {
    let text = value.as_str()?;
    if let Some(digits) = text.strip_prefix('#') {
        return u32::from_str_radix(digits, 16)
            .ok()
            .map(|packed| TextColor::Rgb(crate::chat::RgbColor::from_packed(packed)));
    }
    NamedColor::from_key(text).map(TextColor::Named)
}

fn style_from_json(fields: &Map<String, Value>) -> Style {
    let mut style = Style::empty();
    let decoration = |key: &str| fields.get(key).and_then(Value::as_bool);

    style.bold = decoration("bold");
    style.italic = decoration("italic");
    style.underlined = decoration("underlined");
    style.strikethrough = decoration("strikethrough");
    style.obfuscated = decoration("obfuscated");
    style.color = fields.get("color").and_then(color_from_json);
    style.font = fields
        .get("font")
        .and_then(Value::as_str)
        .map(str::to_owned);
    style.insertion = fields
        .get("insertion")
        .and_then(Value::as_str)
        .map(str::to_owned);
    style
}

fn component_from_value(value: &Value) -> Option<Component> {
    match value {
        Value::String(text) => Some(Component::text(text.clone())),
        Value::Object(fields) => {
            let content = if let Some(text) = fields.get("text").and_then(Value::as_str) {
                ComponentContent::Text {
                    text: text.to_owned(),
                }
            } else if let Some(key) = fields.get("translate").and_then(Value::as_str) {
                ComponentContent::Translatable {
                    key: key.to_owned(),
                    arguments: Vec::new(),
                }
            } else if let Some(keybind) = fields.get("keybind").and_then(Value::as_str) {
                ComponentContent::Keybind {
                    keybind: keybind.to_owned(),
                }
            } else {
                ComponentContent::Text {
                    text: String::new(),
                }
            };

            let children = fields
                .get("extra")
                .and_then(Value::as_array)
                .map(|items| items.iter().filter_map(component_from_value).collect())
                .unwrap_or_default();

            Some(Component::new(content, style_from_json(fields), children))
        }
        _ => None,
    }
}

/// Reads a component written as raw JSON, as a configuration file may carry it.
///
/// Returns `None` for anything that is not valid JSON describing a component, so the
/// caller can fall back to the other notations. Click and hover events are not read
/// back: nothing in the configuration round-trips them, and guessing at the several
/// shapes they have had over the years would invite silent misreadings.
pub fn from_json_str(input: &str) -> Option<Component> {
    let value: Value = serde_json::from_str(input).ok()?;
    match value {
        Value::Object(_) => component_from_value(&value),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::chat::RgbColor;

    use super::*;

    fn styled(style: Style) -> Component {
        Component::new(
            ComponentContent::Text {
                text: "x".to_owned(),
            },
            style,
            Vec::new(),
        )
    }

    fn with_color(color: TextColor) -> Component {
        let mut style = Style::empty();
        style.color = Some(color);
        styled(style)
    }

    #[test]
    fn given_a_hex_colour_when_serialized_for_a_pre_1_16_client_then_it_is_reduced_to_a_name() {
        let orange = with_color(TextColor::Rgb(RgbColor::from_packed(0xFF8800)));

        assert_eq!(
            to_json_string(&orange, JsonProfile::Legacy),
            r#"{"color":"gold","text":"x"}"#
        );
        assert_eq!(
            to_json_string(&orange, JsonProfile::HexColours),
            r##"{"color":"#FF8800","text":"x"}"##
        );
    }

    #[test]
    fn given_a_colour_matching_a_name_exactly_when_serialized_then_the_name_is_used() {
        let blue = with_color(TextColor::Rgb(NamedColor::Blue.rgb()));

        assert_eq!(
            to_json_string(&blue, JsonProfile::HexColours),
            r#"{"color":"blue","text":"x"}"#
        );
    }

    #[test]
    fn given_unstyled_text_when_serialized_then_only_the_later_profiles_collapse_it() {
        let plain = Component::text("plain");

        assert_eq!(
            to_json_string(&plain, JsonProfile::HexColours),
            r#"{"text":"plain"}"#
        );
        assert_eq!(
            to_json_string(&plain, JsonProfile::CompactText),
            r#""plain""#
        );
    }

    #[test]
    fn given_a_hover_event_when_serialized_then_the_key_and_payload_follow_the_profile() {
        let mut style = Style::empty();
        style.hover_event = Some(HoverEvent::ShowText {
            text: Box::new(Component::text("tip")),
        });
        let component = styled(style);

        assert_eq!(
            to_json_string(&component, JsonProfile::Legacy),
            r#"{"hoverEvent":{"action":"show_text","value":{"text":"tip"}},"text":"x"}"#
        );
        assert_eq!(
            to_json_string(&component, JsonProfile::HexColours),
            r#"{"hoverEvent":{"action":"show_text","contents":{"text":"tip"}},"text":"x"}"#
        );
        assert_eq!(
            to_json_string(&component, JsonProfile::CompactText),
            r#"{"hoverEvent":{"action":"show_text","contents":"tip"},"text":"x"}"#
        );
        assert_eq!(
            to_json_string(&component, JsonProfile::RenamedEvents),
            r#"{"hover_event":{"action":"show_text","value":"tip"},"text":"x"}"#
        );
    }

    #[test]
    fn given_a_click_event_when_serialized_then_the_payload_key_follows_the_action() {
        let mut style = Style::empty();
        style.click_event = Some(ClickEvent::OpenUrl {
            url: "https://example.com".to_owned(),
        });
        let open_url = styled(style);

        assert_eq!(
            to_json_string(&open_url, JsonProfile::CompactText),
            r#"{"clickEvent":{"action":"open_url","value":"https://example.com"},"text":"x"}"#
        );
        assert_eq!(
            to_json_string(&open_url, JsonProfile::RenamedEvents),
            r#"{"click_event":{"action":"open_url","url":"https://example.com"},"text":"x"}"#
        );
    }

    #[test]
    fn given_style_keys_when_serialized_then_they_come_out_alphabetically() {
        let mut style = Style::empty();
        style.bold = Some(true);
        style.color = Some(TextColor::Named(NamedColor::White));

        assert_eq!(
            to_json_string(&styled(style), JsonProfile::Legacy),
            r#"{"bold":true,"color":"white","text":"x"}"#
        );
    }
}
