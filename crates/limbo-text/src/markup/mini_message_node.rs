/// One node of a parsed MiniMessage document.
///
/// The format is a tree of tags around literal text, so parsing happens in two passes:
/// first into this shape, then into components. Tags such as `<gradient>` need to see all
/// the text they enclose before they can colour any of it, which a single streaming pass
/// cannot do.
#[derive(Debug, Clone, PartialEq)]
pub enum MiniMessageNode {
    Text {
        text: String,
    },
    Tag {
        name: String,
        arguments: Vec<String>,
        /// The tag exactly as written, so an unrecognised one can be shown literally
        /// instead of vanishing.
        raw: String,
        children: Vec<MiniMessageNode>,
    },
}

impl MiniMessageNode {
    /// The literal text this node and its descendants contribute, ignoring styling.
    ///
    /// Colour-spreading tags need this up front to know how far to spread.
    pub fn text_length(&self) -> usize {
        match self {
            Self::Text { text } => text.chars().count(),
            Self::Tag { children, .. } => children.iter().map(Self::text_length).sum(),
        }
    }
}
