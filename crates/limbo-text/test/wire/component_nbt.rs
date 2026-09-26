use valence_nbt::{List, Value as NbtValue};

use crate::chat::{Component, ComponentContent, Style};
use crate::wire::{JsonProfile, to_nbt_compound};

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
