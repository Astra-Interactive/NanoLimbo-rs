use serde_json::Value as JsonValue;
use valence_nbt::{Compound, List, Value as NbtValue};

use crate::chat::Component;
use crate::wire::JsonProfile;
use crate::wire::build_value;

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
/// straight from the component avoids the round trip and the loss.
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
