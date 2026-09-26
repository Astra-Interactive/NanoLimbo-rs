use crate::chat::{ComponentContent, NamedColor, TextColor};
use crate::markup::parse;

#[test]
fn given_plain_text_when_parsed_then_it_is_a_single_unstyled_component() {
    let component = parse("plain text");

    assert_eq!(
        component.content,
        ComponentContent::Text {
            text: "plain text".to_owned()
        }
    );
    assert!(component.style.is_empty());
    assert!(component.children.is_empty());
}

#[test]
fn given_a_colour_tag_when_parsed_then_the_colour_lands_on_the_text_itself() {
    let component = parse("<red>red text");

    assert_eq!(
        component.style.color,
        Some(TextColor::Named(NamedColor::Red))
    );
    assert_eq!(
        component.content,
        ComponentContent::Text {
            text: "red text".to_owned()
        }
    );
}

#[test]
fn given_an_escaped_angle_bracket_when_parsed_then_it_is_literal_text() {
    assert_eq!(parse("\\<not a tag>").to_plain_text(), "<not a tag>");
}

#[test]
fn given_raw_json_when_parsed_then_it_is_read_as_a_component() {
    let component = parse(r#"{"text":"raw json","color":"red"}"#);

    assert_eq!(component.to_plain_text(), "raw json");
    assert_eq!(
        component.style.color,
        Some(TextColor::Named(NamedColor::Red))
    );
}

#[test]
fn given_an_empty_string_when_parsed_then_it_is_the_empty_component() {
    assert_eq!(parse("").to_plain_text(), "");
}
