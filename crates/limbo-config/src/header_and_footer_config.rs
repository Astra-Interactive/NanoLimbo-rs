use limbo_text::chat::Component;

/// The text drawn above and below the tab list.
#[derive(Debug, Clone, PartialEq)]
pub struct HeaderAndFooterConfig {
    pub header: Component,
    pub footer: Component,
}
