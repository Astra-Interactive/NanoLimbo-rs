use crate::chat::Component;
use crate::legacy_codes::legacy_codes_to_tags;
use crate::mini_message::parse_mini_message;

/// Parses a line of configured text into a component.
///
/// Three notations are accepted, in the order a configuration file is likely to use them:
/// a raw JSON component pasted from elsewhere, legacy `&` or `§` colour codes, and
/// MiniMessage. Legacy codes are rewritten as MiniMessage tags first, so a file mixing
/// the two — which is common in configurations carried between servers — works.
pub fn parse(input: &str) -> Component {
    if input.is_empty() {
        return Component::empty();
    }

    if let Some(component) = crate::component_json::from_json_str(input) {
        return component;
    }

    parse_mini_message(&legacy_codes_to_tags(input))
}

#[cfg(test)]
mod tests {
    use crate::chat::{ComponentContent, NamedColor, TextColor};

    use super::*;

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
}
