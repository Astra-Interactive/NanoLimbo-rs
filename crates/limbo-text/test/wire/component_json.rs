use crate::chat::{
    ClickEvent, Component, ComponentContent, HoverEvent, NamedColor, RgbColor, Style, TextColor,
};
use crate::wire::{JsonProfile, to_json_string};

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
