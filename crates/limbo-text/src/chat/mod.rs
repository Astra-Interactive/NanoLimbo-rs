//! The component model: what the server can say and how it is styled.
//!
//! Shapes follow the protocol's serialized form rather than Rust convenience, because
//! this crate's contract is byte-level parity with Kyori Adventure.

mod click_event;
mod component;
mod component_content;
mod hover_event;
mod named_color;
mod rgb_color;
mod style;
mod text_color;

pub use click_event::ClickEvent;
pub use component::Component;
pub use component_content::ComponentContent;
pub use hover_event::HoverEvent;
pub use named_color::NamedColor;
pub use rgb_color::RgbColor;
pub use style::Style;
pub use text_color::TextColor;
