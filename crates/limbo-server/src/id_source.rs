use uuid::Uuid;

/// Supplies the values the server would otherwise pull from a random number generator.
///
/// Every one of these reaches the wire, so making them an injected dependency is what
/// allows a packet to be compared byte-for-byte against a reference dump. Calling a
/// global generator inside the encoders would make that impossible, which is why this
/// port exists at all — see MIGRATION_PLAN.md section 7.5.
pub trait IdSource: Send + Sync {
    /// Entity id for the join game packet.
    fn next_entity_id(&self) -> i32;

    /// Teleport id echoed back by the client after a position update.
    fn next_teleport_id(&self) -> i32;

    /// Token the client must return in its keep-alive reply.
    fn next_keep_alive_id(&self) -> i64;

    /// Correlation id for a login plugin request.
    fn next_message_id(&self) -> i32;

    /// A fresh identifier for something the protocol wants a UUID for, such as a boss bar.
    fn next_uuid(&self) -> Uuid;
}

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
