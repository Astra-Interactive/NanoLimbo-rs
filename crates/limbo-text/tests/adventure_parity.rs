//! Checks this crate against the reference implementation across every text input the
//! configuration can carry.
//!
//! The comparison is on **rendering**, not on bytes. Two component trees that nest
//! differently but resolve to the same styled text are indistinguishable to a client, and
//! the reference implementation's nesting is an artifact of how Adventure compacts trees
//! rather than anything the protocol requires. Flattening both sides to a list of styled
//! runs tests what a player actually sees, and it catches every difference that matters:
//! wrong colour, missing decoration, dropped text, mangled event.
//!
//! Colours are compared after serialization, per profile, so the pre-1.16 reduction of
//! hex to named colours and the later hex form are both covered.
//!
//! See `MIGRATION_PLAN.md` section 7.

use std::collections::BTreeMap;

use serde_json::Value;

use limbo_protocol::version::ProtocolVersion;
use limbo_text::{parse, to_json_for, to_legacy_string};

const FIXTURE: &str = include_str!("../../../fixtures/text/components.json");

/// A stretch of text with every style property already resolved against its ancestors.
#[derive(Debug, Clone, PartialEq, Eq)]
struct StyledRun {
    text: String,
    color: Option<String>,
    decorations: BTreeMap<String, bool>,
    click: Option<String>,
    hover: Option<String>,
}

impl StyledRun {
    fn root() -> Self {
        Self {
            text: String::new(),
            color: None,
            decorations: BTreeMap::new(),
            click: None,
            hover: None,
        }
    }
}

const DECORATIONS: [&str; 5] = [
    "bold",
    "italic",
    "underlined",
    "strikethrough",
    "obfuscated",
];

/// Reduces the two spellings of each event key to one, so profiles can be compared.
fn event_summary(fields: &serde_json::Map<String, Value>, keys: [&str; 2]) -> Option<String> {
    let event = keys
        .iter()
        .find_map(|key| fields.get(*key))
        .and_then(Value::as_object)?;

    let action = event.get("action").and_then(Value::as_str).unwrap_or("");
    let payload = event
        .iter()
        .filter(|(key, _)| key.as_str() != "action")
        .map(|(_, value)| match value {
            Value::String(text) => text.clone(),
            other => flatten(other, &StyledRun::root())
                .into_iter()
                .map(|run| run.text)
                .collect(),
        })
        .collect::<Vec<_>>()
        .join("");

    Some(format!("{action}={payload}"))
}

/// Walks a component tree, resolving inherited style into a flat list of runs.
fn flatten(value: &Value, inherited: &StyledRun) -> Vec<StyledRun> {
    let mut runs = Vec::new();

    let (body, fields) = match value {
        Value::String(text) => (text.clone(), None),
        Value::Object(fields) => {
            let body = fields
                .get("text")
                .or_else(|| fields.get("translate"))
                .or_else(|| fields.get("keybind"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            (body, Some(fields))
        }
        _ => return runs,
    };

    let mut resolved = inherited.clone();
    if let Some(fields) = fields {
        if let Some(color) = fields.get("color").and_then(Value::as_str) {
            resolved.color = Some(color.to_ascii_lowercase());
        }
        for decoration in DECORATIONS {
            if let Some(enabled) = fields.get(decoration).and_then(Value::as_bool) {
                resolved.decorations.insert(decoration.to_owned(), enabled);
            }
        }
        if let Some(click) = event_summary(fields, ["clickEvent", "click_event"]) {
            resolved.click = Some(click);
        }
        if let Some(hover) = event_summary(fields, ["hoverEvent", "hover_event"]) {
            resolved.hover = Some(hover);
        }
    }

    if !body.is_empty() {
        let mut run = resolved.clone();
        run.text = body;
        runs.push(run);
    }

    if let Some(children) = fields
        .and_then(|fields| fields.get("extra"))
        .and_then(Value::as_array)
    {
        for child in children {
            runs.extend(flatten(child, &resolved));
        }
    }

    runs
}

/// Merges neighbouring runs that share a style, so a difference in how text is split
/// across components does not register as a difference in what is rendered.
fn merge_adjacent(runs: Vec<StyledRun>) -> Vec<StyledRun> {
    let mut merged: Vec<StyledRun> = Vec::new();
    for run in runs {
        match merged.last_mut() {
            Some(previous)
                if previous.color == run.color
                    && previous.decorations == run.decorations
                    && previous.click == run.click
                    && previous.hover == run.hover =>
            {
                previous.text.push_str(&run.text);
            }
            _ => merged.push(run),
        }
    }
    merged
}

/// Parses and flattens a serialized component. `None` means the fixture itself is broken,
/// which the calling test turns into a failure rather than silently tolerating.
fn rendered(json: &str) -> Option<Vec<StyledRun>> {
    let value: Value = serde_json::from_str(json).ok()?;
    Some(merge_adjacent(flatten(&value, &StyledRun::root())))
}

fn version_of(protocol: &str) -> Option<ProtocolVersion> {
    ProtocolVersion::from_number(protocol.parse().ok()?)
}

#[test]
fn given_every_configured_text_when_parsed_and_serialized_then_it_renders_as_the_reference_does() {
    let document: Value = serde_json::from_str(FIXTURE).expect("fixture must parse");
    let entries = document["entries"].as_array().expect("entries array");
    let mut compared = 0;

    for entry in entries {
        let input = entry["input"].as_str().expect("input string");
        let component = parse(input);

        let profiles = entry["json"].as_object().expect("json profiles");
        for (protocol, expected) in profiles {
            let expected_json = expected.as_str().expect("profile json string");
            let version = version_of(protocol).expect("fixture protocols must be supported");
            let ours = to_json_for(&component, version);

            assert_eq!(
                rendered(&ours).expect("our own output must parse"),
                rendered(expected_json).expect("reference output must parse"),
                "\ninput:    {input:?}\nprotocol: {protocol}\nours:     {ours}\nreference:{expected_json}\n"
            );
            compared += 1;
        }
    }

    assert!(
        compared >= 100,
        "expected the corpus to cover at least 100 input/profile pairs, compared {compared}"
    );
    println!("compared {compared} input/profile pairs against the reference");
}

#[test]
fn given_every_configured_text_when_parsed_then_its_plain_rendering_matches_the_reference() {
    let document: Value = serde_json::from_str(FIXTURE).expect("fixture must parse");

    for entry in document["entries"].as_array().expect("entries array") {
        let input = entry["input"].as_str().expect("input string");
        let expected = entry["plain"].as_str().expect("plain string");

        assert_eq!(parse(input).to_plain_text(), expected, "input {input:?}");
    }
}

#[test]
fn given_every_configured_text_when_parsed_then_its_legacy_rendering_matches_the_reference() {
    let document: Value = serde_json::from_str(FIXTURE).expect("fixture must parse");

    for entry in document["entries"].as_array().expect("entries array") {
        let input = entry["input"].as_str().expect("input string");
        let expected = entry["legacy"].as_str().expect("legacy string");

        assert_eq!(to_legacy_string(&parse(input)), expected, "input {input:?}");
    }
}
