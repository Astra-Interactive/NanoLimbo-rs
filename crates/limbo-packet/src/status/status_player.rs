use uuid::Uuid;

/// One entry of the player sample a client shows when hovering the player count.
pub struct StatusPlayer {
    pub name: String,
    pub unique_id: Uuid,
}
