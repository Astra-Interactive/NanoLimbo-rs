/// The entry the player sees for themselves in the tab list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListConfig {
    /// Whether the tab list entry is sent at all. A 1.16.5 client crashes without one,
    /// so the connection handler sends it to that version regardless.
    pub enabled: bool,
    /// Also the name the login success packet carries, and the seed of the offline-mode
    /// UUID, so it is read even when the tab list is switched off.
    pub username: String,
}
