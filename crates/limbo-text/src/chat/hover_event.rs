use crate::chat::component::Component;

/// What the client shows when the pointer rests on the text.
///
/// Only `show_text` is modelled: a limbo server has no items or entities to describe.
/// The payload is a component, so hover text carries its own styling.
#[derive(Debug, Clone, PartialEq)]
pub enum HoverEvent {
    ShowText { text: Box<Component> },
}

impl HoverEvent {
    /// The `action` value written in JSON.
    pub const fn action(&self) -> &'static str {
        match self {
            Self::ShowText { .. } => "show_text",
        }
    }
}
