use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use limbo_protocol::version::ProtocolVersion;
use uuid::Uuid;

use crate::player::{ConnectedPlayer, ConnectionRegistry};

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
