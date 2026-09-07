use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::connected_player::ConnectedPlayer;
use crate::connection_id::ConnectionId;

/// Everyone currently past the login phase.
///
/// Shared across every connection task, so the lock is held only long enough to touch the
/// map — never across an await.
#[derive(Debug)]
pub struct ConnectionRegistry {
    players: Mutex<HashMap<ConnectionId, ConnectedPlayer>>,
    next_id: AtomicU64,
}

impl ConnectionRegistry {
    pub fn empty() -> Self {
        Self {
            players: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(0),
        }
    }

    /// Reads the map, yielding a default if another task panicked while holding the lock.
    ///
    /// A poisoned lock must not cascade: one connection task falling over should not take
    /// the listener with it, and the count being briefly wrong is the lesser failure.
    fn with_players<T>(
        &self,
        read: impl FnOnce(&HashMap<ConnectionId, ConnectedPlayer>) -> T,
        fallback: T,
    ) -> T {
        match self.players.lock() {
            Ok(players) => read(&players),
            Err(_) => fallback,
        }
    }

    pub fn count(&self) -> usize {
        self.with_players(HashMap::len, 0)
    }

    pub fn usernames(&self) -> Vec<String> {
        self.with_players(
            |players| {
                players
                    .values()
                    .map(|player| player.username.clone())
                    .collect()
            },
            Vec::new(),
        )
    }

    /// Records a player as online and returns the handle used to remove them again.
    pub fn add(&self, player: ConnectedPlayer) -> ConnectionId {
        let id = ConnectionId::new(self.next_id.fetch_add(1, Ordering::Relaxed));
        if let Ok(mut players) = self.players.lock() {
            players.insert(id, player);
        }
        id
    }

    pub fn remove(&self, id: ConnectionId) -> Option<ConnectedPlayer> {
        self.players.lock().ok()?.remove(&id)
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use limbo_protocol::version::ProtocolVersion;
    use uuid::Uuid;

    use super::*;

    fn player(username: &str) -> ConnectedPlayer {
        ConnectedPlayer {
            username: username.to_owned(),
            // Offline-mode UUIDs are derived from the username, so two players with the
            // same name genuinely share one.
            uuid: Uuid::from_u128(7),
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 25565),
            version: ProtocolVersion::V1_20_5,
        }
    }

    #[test]
    fn given_two_players_sharing_a_uuid_when_both_join_then_both_are_counted() {
        let registry = ConnectionRegistry::empty();

        registry.add(player("Notch"));
        registry.add(player("Notch"));

        assert_eq!(registry.count(), 2);
    }

    #[test]
    fn given_two_players_sharing_a_uuid_when_one_leaves_then_the_other_stays_online() {
        let registry = ConnectionRegistry::empty();
        let first = registry.add(player("Notch"));
        registry.add(player("Notch"));

        registry.remove(first);

        assert_eq!(registry.count(), 1);
    }

    #[test]
    fn given_a_handle_already_removed_when_removed_again_then_the_count_is_unchanged() {
        let registry = ConnectionRegistry::empty();
        let id = registry.add(player("Notch"));

        assert!(registry.remove(id).is_some());
        assert!(registry.remove(id).is_none());
        assert_eq!(registry.count(), 0);
    }
}
