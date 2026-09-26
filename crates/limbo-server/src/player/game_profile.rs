use uuid::Uuid;

/// Who the server believes the connected player is.
///
/// Both fields fill in over the course of the login sequence, and which step fills them
/// depends on the forwarding mode, so neither can be required up front.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfile {
    pub username: Option<String>,
    pub uuid: Option<Uuid>,
}

impl GameProfile {
    pub const fn unidentified() -> Self {
        Self {
            username: None,
            uuid: None,
        }
    }
}
