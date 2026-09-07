use crate::chat::component_content::ComponentContent;
use crate::chat::style::Style;

/// A piece of formatted text, as the client understands it.
///
/// A component carries content, a style, and children that inherit that style. This
/// mirrors the shape the protocol serializes rather than any particular Rust
/// convenience, because byte-level parity with the reference implementation is the
/// contract this crate has to meet.
#[derive(Debug, Clone, PartialEq)]
pub struct Component {
    pub content: ComponentContent,
    pub style: Style,
    pub children: Vec<Component>,
}

impl Component {
    pub const fn new(content: ComponentContent, style: Style, children: Vec<Component>) -> Self {
        Self {
            content,
            style,
            children,
        }
    }

    /// Unstyled literal text with no children.
    pub fn text(text: impl Into<String>) -> Self {
        Self::new(
            ComponentContent::Text { text: text.into() },
            Style::empty(),
            Vec::new(),
        )
    }

    /// The empty component, which serializes as text with an empty string rather than
    /// as an absent value.
    pub fn empty() -> Self {
        Self::text(String::new())
    }

    pub fn keybind(keybind: impl Into<String>) -> Self {
        Self::new(
            ComponentContent::Keybind {
                keybind: keybind.into(),
            },
            Style::empty(),
            Vec::new(),
        )
    }

    pub fn translatable(key: impl Into<String>, arguments: Vec<Self>) -> Self {
        Self::new(
            ComponentContent::Translatable {
                key: key.into(),
                arguments,
            },
            Style::empty(),
            Vec::new(),
        )
    }

    /// The text this component and its descendants render to, with all styling dropped.
    ///
    /// Used for log lines and for the legacy ping version string. Translation keys and
    /// keybinds contribute nothing, because resolving them needs the client's language.
    pub fn to_plain_text(&self) -> String {
        let mut plain = String::new();
        self.append_plain_text(&mut plain);
        plain
    }

    fn append_plain_text(&self, target: &mut String) {
        if let ComponentContent::Text { text } = &self.content {
            target.push_str(text);
        }
        for child in &self.children {
            child.append_plain_text(target);
        }
    }
}
