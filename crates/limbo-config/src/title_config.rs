use limbo_text::chat::Component;

use crate::ticks::Ticks;

/// The title and subtitle shown when the player joins.
#[derive(Debug, Clone, PartialEq)]
pub struct TitleConfig {
    /// Either half may be the empty component, which is how the configuration asks for
    /// only the other one.
    pub title: Component,
    pub subtitle: Component,
    pub fade_in: Ticks,
    pub stay: Ticks,
    pub fade_out: Ticks,
}
