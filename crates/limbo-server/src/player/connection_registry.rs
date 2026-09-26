use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::player::ConnectedPlayer;
use crate::player::ConnectionId;

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
