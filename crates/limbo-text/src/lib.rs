//! Chat components and the text formats the server converts between.
//!
//! The Java implementation delegated all of this to Kyori Adventure. No Rust crate covers
//! the same ground — Adventure's per-version JSON profiles and its NBT component encoding
//! in particular have no equivalent — so this crate implements the subset `settings.yml`
//! can express, and is checked against fixtures dumped from Adventure.

pub mod chat {
    //! The component model: what the server can say and how it is styled.
    //!
    //! Shapes follow the protocol's serialized form rather than Rust convenience, because
    //! this crate's contract is byte-level parity with Kyori Adventure.

    mod click_event;
    mod component;
    mod component_content;
    mod hover_event;
    mod hsv_color;
    mod named_color;
    mod rgb_color;
    mod style;
    mod text_color;

    pub use click_event::ClickEvent;
    pub use component::Component;
    pub use component_content::ComponentContent;
    pub use hover_event::HoverEvent;
    pub(crate) use hsv_color::HsvColor;
    pub use named_color::NamedColor;
    pub use rgb_color::RgbColor;
    pub use style::Style;
    pub use text_color::TextColor;
}

mod markup {
    //! The text formats an administrator writes in `settings.yml`: MiniMessage, legacy
    //! `&`/`§` codes and raw component JSON, all read into a [`Component`](crate::chat::Component).

    mod legacy_codes;
    mod mini_message;
    mod mini_message_node;
    mod text_parser;

    pub(crate) use legacy_codes::legacy_codes_to_tags;
    pub use legacy_codes::to_legacy_string;
    pub(crate) use mini_message::parse_mini_message;
    pub(crate) use mini_message_node::MiniMessageNode;
    pub use text_parser::parse;
}

mod wire {
    //! What a client reads: a component encoded as JSON or NBT in the shape its protocol
    //! version expects.

    mod component_json;
    mod component_nbt;
    mod component_writer;
    mod json_profile;

    pub(crate) use component_json::{build_value, from_json_str, to_json_string};
    pub(crate) use component_nbt::to_nbt_compound;
    pub use component_writer::{to_json_for, write_component};
    pub use json_profile::JsonProfile;
}

pub use markup::{parse, to_legacy_string};
pub use wire::{JsonProfile, to_json_for, write_component};

#[cfg(test)]
#[path = "../test/lib.rs"]
mod test;
