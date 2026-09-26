use uuid::Uuid;

use crate::id::IdSource;

/// An id source that returns the same values every time.
///
/// Not a test double: the packets built once at startup — the boss bar, the join game
/// entity id, the teleport id — are shared by every player, so the server draws each of
/// them exactly once anyway. Handing the encoders a fixed source makes that explicit, and
/// makes a startup snapshot reproducible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedIdSource {
    entity_id: i32,
    teleport_id: i32,
    keep_alive_id: i64,
    message_id: i32,
    uuid: Uuid,
}

impl FixedIdSource {
    pub const fn new(
        entity_id: i32,
        teleport_id: i32,
        keep_alive_id: i64,
        message_id: i32,
        uuid: Uuid,
    ) -> Self {
        Self {
            entity_id,
            teleport_id,
            keep_alive_id,
            message_id,
            uuid,
        }
    }
}

impl IdSource for FixedIdSource {
    fn next_entity_id(&self) -> i32 {
        self.entity_id
    }

    fn next_teleport_id(&self) -> i32 {
        self.teleport_id
    }

    fn next_keep_alive_id(&self) -> i64 {
        self.keep_alive_id
    }

    fn next_message_id(&self) -> i32 {
        self.message_id
    }

    fn next_uuid(&self) -> Uuid {
        self.uuid
    }
}
