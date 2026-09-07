use crate::forwarding_mode::ForwardingMode;

/// The configured facts the connection state machine consults.
///
/// A narrow view of the configuration rather than the whole of it, so the state machine
/// stays testable without building a full settings file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerPolicy {
    pub forwarding: ForwardingMode,
    /// Zero or below means no limit, as it does in the configuration file.
    pub max_players: i32,
}

impl ServerPolicy {
    pub const fn new(forwarding: ForwardingMode, max_players: i32) -> Self {
        Self {
            forwarding,
            max_players,
        }
    }

    pub const fn is_full(&self, online: i32) -> bool {
        self.max_players > 0 && online >= self.max_players
    }
}
