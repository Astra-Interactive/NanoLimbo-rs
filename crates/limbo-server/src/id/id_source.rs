use uuid::Uuid;

/// Supplies the values the server would otherwise pull from a random number generator.
///
/// Every one of these reaches the wire, so making them an injected dependency is what
/// allows a packet to be compared byte-for-byte against a reference dump. Calling a
/// global generator inside the encoders would make that impossible, which is why this
/// port exists at all.
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
