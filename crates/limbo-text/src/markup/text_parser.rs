use crate::chat::Component;
use crate::markup::legacy_codes_to_tags;
use crate::markup::parse_mini_message;

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

    if let Some(component) = crate::wire::from_json_str(input) {
        return component;
    }

    parse_mini_message(&legacy_codes_to_tags(input))
}
