use limbo_server::id_source::IdSource;
use rand::Rng;
use uuid::Uuid;

/// The production [`IdSource`], drawing from the thread-local generator.
///
/// Lives in the binary rather than in a library so no library can reach a random source
/// by accident: every value that ends up on the wire has to be handed down from here,
/// which is what keeps the encoders reproducible under test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomIdSource;

impl IdSource for RandomIdSource {
    fn next_entity_id(&self) -> i32 {
        rand::rng().random_range(1..1_000_000)
    }

    fn next_teleport_id(&self) -> i32 {
        rand::rng().random()
    }

    fn next_keep_alive_id(&self) -> i64 {
        rand::rng().random()
    }

    fn next_message_id(&self) -> i32 {
        rand::rng().random_range(0..i32::MAX)
    }

    fn next_uuid(&self) -> Uuid {
        Uuid::from_u128(rand::rng().random())
    }
}
