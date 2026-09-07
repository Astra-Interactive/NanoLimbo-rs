use serde_json::Value as JsonValue;
use valence_nbt::{Compound, List, Value as NbtValue};

use crate::chat::Component;
use crate::component_json::build_value;
use crate::json_profile::JsonProfile;

fn json_to_nbt(value: &JsonValue) -> Option<NbtValue> {
    match value {
        JsonValue::Bool(flag) => Some(NbtValue::Byte(i8::from(*flag))),
        JsonValue::String(text) => Some(NbtValue::String(text.clone())),
        JsonValue::Number(number) => number
            .as_i64()
            .and_then(|integer| i32::try_from(integer).ok())
            .map(NbtValue::Int)
            .or_else(|| number.as_f64().map(NbtValue::Double)),
        JsonValue::Object(fields) => Some(NbtValue::Compound(json_object_to_compound(fields))),
        JsonValue::Array(items) => {
            let compounds: Option<Vec<Compound>> = items
                .iter()
                .map(|item| match json_to_nbt(item) {
                    Some(NbtValue::Compound(compound)) => Some(compound),
                    _ => None,
                })
                .collect();
            compounds.map(|entries| {
                if entries.is_empty() {
                    NbtValue::List(List::End)
                } else {
                    NbtValue::List(List::Compound(entries))
                }
            })
        }
        JsonValue::Null => None,
    }
}

fn json_object_to_compound(fields: &serde_json::Map<String, JsonValue>) -> Compound {
    let mut compound = Compound::new();
    for (key, value) in fields {
        if let Some(converted) = json_to_nbt(value) {
            compound.insert(key.clone(), converted);
        }
    }
    compound
}

/// Builds the NBT a client from 1.20.3 onwards reads a component as.
///
/// Compaction is deliberately switched off here, which is a **fix** rather than a port.
/// The reference implementation reaches NBT by serializing to JSON first and converting,
/// so an unstyled child inside `extra` arrives as a bare string; its converter then wraps
/// any non-object list element as a compound under an empty key. The client finds no
/// `text` field there and renders nothing, which silently drops the text.
///
/// That is not hypothetical: the shipped `settings.yml` join message ends in `!`, which
/// becomes `{"": "!"}` and disappears for every client from 1.20.3 on. Building the tree
/// straight from the component avoids the round trip and the loss. See MIGRATION_PLAN.md
/// section 3.1.10.
pub fn to_nbt_compound(component: &Component, profile: JsonProfile) -> Compound {
    match build_value(component, profile, false) {
        JsonValue::Object(fields) => json_object_to_compound(&fields),
        other => {
            let mut compound = Compound::new();
            if let Some(value) = json_to_nbt(&other) {
                compound.insert("text", value);
            }
            compound
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chat::{ComponentContent, Style};

    use super::*;

    #[test]
    fn given_unstyled_text_when_converted_to_nbt_then_it_stays_a_compound() {
        let compound = to_nbt_compound(&Component::text("plain text"), JsonProfile::CompactText);

        assert_eq!(
            compound.get("text"),
            Some(&NbtValue::String("plain text".to_owned()))
        );
    }

    #[test]
    fn given_an_unstyled_child_when_converted_to_nbt_then_its_text_is_not_lost() {
        let mut bold = Style::empty();
        bold.bold = Some(true);
        let root = Component::new(
            ComponentContent::Text {
                text: String::new(),
            },
            Style::empty(),
            vec![
                Component::new(
                    ComponentContent::Text {
                        text: "bold".to_owned(),
                    },
                    bold,
                    Vec::new(),
                ),
                Component::text(" normal"),
            ],
        );

        let compound = to_nbt_compound(&root, JsonProfile::CompactText);
        let Some(NbtValue::List(List::Compound(children))) = compound.get("extra") else {
            panic!(
                "extra must be a list of compounds, got {:?}",
                compound.get("extra")
            );
        };

        assert_eq!(children.len(), 2);
        assert_eq!(
            children[1].get("text"),
            Some(&NbtValue::String(" normal".to_owned())),
            "the trailing text must survive; the reference implementation loses it here"
        );
        assert_eq!(children[1].get(""), None, "no empty-key wrapper may appear");
    }
}
