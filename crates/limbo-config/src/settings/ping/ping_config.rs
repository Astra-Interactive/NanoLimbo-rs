use limbo_text::chat::Component;

/// What the server answers a server-list ping with.
#[derive(Debug, Clone, PartialEq)]
pub struct PingConfig {
    pub description: Component,
    /// Shown next to the player count. The client renders it only when the protocol
    /// number does not match its own, so it doubles as the "outdated" label.
    pub version: Component,
    /// A fixed protocol number to advertise, or `None` to answer with the number the
    /// client itself sent so that it never sees a version mismatch.
    pub protocol: Option<i32>,
}
