use crate::chat::component::Component;

/// What a component says, as opposed to how it looks.
///
/// The protocol defines further kinds — score, selector, NBT — that a limbo server has
/// no way to populate and therefore never sends.
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentContent {
    /// Literal text.
    Text { text: String },
    /// A translation key the client resolves against its own language file.
    Translatable {
        key: String,
        arguments: Vec<Component>,
    },
    /// The key currently bound to an action, rendered by the client.
    Keybind { keybind: String },
}
