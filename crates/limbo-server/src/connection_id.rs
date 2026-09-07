/// A handle for one open connection, unique for the lifetime of the process.
///
/// Deliberately not the player's UUID. Upstream keys its connection map by UUID, and in
/// offline mode a UUID is derived from the username, so two players with the same name
/// collide: the second evicts the first, and when the first leaves it removes the
/// second's entry, leaving the online count permanently wrong. A counter cannot collide.
/// See MIGRATION_PLAN.md section 3.1.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConnectionId(u64);

impl ConnectionId {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}
