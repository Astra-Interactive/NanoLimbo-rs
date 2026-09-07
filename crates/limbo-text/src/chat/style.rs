use crate::chat::click_event::ClickEvent;
use crate::chat::hover_event::HoverEvent;
use crate::chat::text_color::TextColor;

/// Presentation applied to a component and inherited by its children.
///
/// Every field is optional and three-valued in effect: `None` inherits from the parent,
/// `Some(false)` explicitly switches a decoration off. Collapsing that to a plain `bool`
/// would change what reaches the client, because an absent key and an explicit `false`
/// are different on the wire.
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub color: Option<TextColor>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
    pub font: Option<String>,
    pub insertion: Option<String>,
    pub click_event: Option<ClickEvent>,
    pub hover_event: Option<HoverEvent>,
}

impl Style {
    /// A style that sets nothing, so the component inherits everything from its parent.
    pub const fn empty() -> Self {
        Self {
            color: None,
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
            font: None,
            insertion: None,
            click_event: None,
            hover_event: None,
        }
    }

    /// Whether serializing this style would emit no keys at all.
    pub const fn is_empty(&self) -> bool {
        self.color.is_none()
            && self.bold.is_none()
            && self.italic.is_none()
            && self.underlined.is_none()
            && self.strikethrough.is_none()
            && self.obfuscated.is_none()
            && self.font.is_none()
            && self.insertion.is_none()
            && self.click_event.is_none()
            && self.hover_event.is_none()
    }
}
