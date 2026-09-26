/// What a client says it is connecting for, in the handshake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientIntent {
    /// Asking for the server list entry, then disconnecting.
    Status,
    /// Joining.
    Login,
    /// Joining after being sent here by another server. Treated exactly as `Login`.
    Transfer,
}

impl ClientIntent {
    /// Resolves the intent field. Anything else is a client the server will not serve.
    pub const fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(Self::Status),
            2 => Some(Self::Login),
            3 => Some(Self::Transfer),
            _ => None,
        }
    }

    pub const fn is_join(self) -> bool {
        matches!(self, Self::Login | Self::Transfer)
    }
}
