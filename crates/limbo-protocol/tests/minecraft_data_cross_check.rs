//! Cross-checks the packet id tables against a second, independent oracle.
//!
//! Every other level of the suite proves the port matches the Java implementation, which by
//! construction cannot find a bug the Java implementation already has: the port would
//! reproduce it byte for byte and the comparison would stay green.
//! `PrismarineJS/minecraft-data` is extracted from the vanilla client rather than from
//! NanoLimbo, so a disagreement between the two tables is evidence that one of them is wrong.
//!
//! It has already earned its place twice: the serverbound configuration `custom_payload` id
//! (0x01 from 1.20.2, not 0x02) and the clientbound play
//! `Disconnect` id on 1.20.3.
//!
//! The fixture is distilled from upstream by `fixtures/minecraft-data/distil_packet_ids.py`;
//! the README beside it records the commit it came from and how to refresh it.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use limbo_protocol::packet::{ConnectionState, PacketDirection, PacketKind, PacketRoute};
use limbo_protocol::version::ProtocolVersion;

const FIXTURE: &str = include_str!("../../../fixtures/minecraft-data/packet_ids.json");

/// Packet ids are varints, but no release has ever published one above 0xFE. Scanning to
/// 0x1FF discovers every mapping either table holds, with room to spare.
const SCANNED_ID_LIMIT: i32 = 0x1FF;

/// A truncated or mis-parsed fixture must not turn this check vacuously green. The tables
/// currently produce 1269 comparisons over 48 published versions; the floors leave room for
/// upstream churn without letting coverage quietly collapse.
const MINIMUM_COMPARISONS: usize = 1_200;
const MINIMUM_COVERED_VERSIONS: usize = 45;

/// A route where minecraft-data publishes no id for a packet our table claims.
struct UnpublishedRoute {
    kind: PacketKind,
    state: ConnectionState,
    direction: PacketDirection,
    /// Oldest version that publishes the packet upstream. Below it, absence is expected.
    published_from: ProtocolVersion,
}

/// The only routes where an absent upstream id is accepted, and why.
///
/// Login plugin messages arrived in 1.13 (protocol 393). minecraft-data lists neither below
/// it and Velocity registers both at `MINECRAFT_1_13`, so upstream is not incomplete here —
/// our range is too wide. It inherits the Java `map(0x02, Version.getMin(), getMax())` and
/// `map(0x04, ...)` spans, which claim ids the protocol did not have yet. The over-claim is
/// inert in practice: NanoLimbo exchanges login plugin messages only for Velocity modern
/// forwarding, which itself requires 1.13. The range should still be narrowed.
///
/// Absence is all this tolerates. A different name at the same id still fails, so an entry
/// here cannot mask a wrong id.
static UNPUBLISHED_ROUTES: &[UnpublishedRoute] = &[
    UnpublishedRoute {
        kind: PacketKind::LoginPluginRequest,
        state: ConnectionState::Login,
        direction: PacketDirection::ClientBound,
        published_from: ProtocolVersion::V1_13,
    },
    UnpublishedRoute {
        kind: PacketKind::LoginPluginResponse,
        state: ConnectionState::Login,
        direction: PacketDirection::ServerBound,
        published_from: ProtocolVersion::V1_13,
    },
];

/// The name, or names, minecraft-data publishes for a packet we implement.
///
/// Exhaustive on purpose: a new [`PacketKind`] does not compile until someone states what
/// the independent oracle calls it. A few kinds carry more than one name because upstream
/// renamed the packet when Mojang reworked it, or because the same kind travels in two
/// states under different names. Listing several does not weaken the check: the comparison
/// requires exactly one of them to exist on any given route.
fn upstream_names(kind: PacketKind) -> &'static [&'static str] {
    match kind {
        PacketKind::BossBar => &["boss_bar"],
        // `chat` through 1.18.2; 1.19 split signed chat out and left `system_chat` behind.
        PacketKind::ChatMessage => &["chat", "system_chat"],
        PacketKind::ChunkWithLight => &["map_chunk"],
        PacketKind::DeclareCommands => &["declare_commands"],
        // `disconnect` in configuration, `kick_disconnect` in play.
        PacketKind::Disconnect => &["disconnect", "kick_disconnect"],
        PacketKind::FinishConfiguration => &["finish_configuration"],
        PacketKind::GameEvent => &["game_state_change"],
        PacketKind::Handshake => &["set_protocol"],
        PacketKind::JoinGame => &["login"],
        PacketKind::KeepAlive => &["keep_alive"],
        PacketKind::KnownPacks => &["select_known_packs"],
        PacketKind::LoginAcknowledged => &["login_acknowledged"],
        PacketKind::LoginDisconnect => &["disconnect"],
        PacketKind::LoginPluginRequest => &["login_plugin_request"],
        PacketKind::LoginPluginResponse => &["login_plugin_response"],
        PacketKind::LoginStart => &["login_start"],
        PacketKind::LoginSuccess => &["success"],
        PacketKind::PlayerAbilities => &["abilities"],
        PacketKind::PlayerInfo => &["player_info"],
        PacketKind::PlayerListHeader => &["playerlist_header"],
        PacketKind::PlayerPositionAndLook => &["position"],
        PacketKind::PluginMessage => &["custom_payload"],
        PacketKind::RegistryData => &["registry_data"],
        PacketKind::SpawnPosition => &["spawn_position"],
        PacketKind::StatusPing => &["ping"],
        PacketKind::StatusRequest => &["ping_start"],
        PacketKind::StatusResponse => &["server_info"],
        // One combined title packet until 1.16.4, three separate ones from 1.17.
        PacketKind::TitleLegacy => &["title"],
        PacketKind::TitleSetSubTitle => &["set_title_subtitle"],
        PacketKind::TitleSetTitle => &["set_title_text"],
        PacketKind::TitleTimes => &["set_title_time"],
        PacketKind::UpdateTags => &["tags"],
    }
}

fn state_key(state: ConnectionState) -> &'static str {
    match state {
        ConnectionState::Handshaking => "handshaking",
        ConnectionState::Status => "status",
        ConnectionState::Login => "login",
        ConnectionState::Configuration => "configuration",
        ConnectionState::Play => "play",
    }
}

fn direction_key(direction: PacketDirection) -> &'static str {
    match direction {
        PacketDirection::ServerBound => "toServer",
        PacketDirection::ClientBound => "toClient",
    }
}

fn parse_packet_id(raw: &str) -> Option<i32> {
    raw.strip_prefix("0x")
        .and_then(|digits| i32::from_str_radix(digits, 16).ok())
}

fn packet_id_table(
    protocol: &Value,
    state: ConnectionState,
    direction: PacketDirection,
) -> Option<&Map<String, Value>> {
    protocol
        .get("states")?
        .get(state_key(state))?
        .get(direction_key(direction))?
        .as_object()
}

/// Every id upstream assigns to one of `names` in this table.
///
/// Returning a set rather than a single id is what makes a multi-name entry safe: two names
/// living at two ids on the same version fails the comparison instead of silently picking one.
fn published_ids(table: &Map<String, Value>, names: &[&str]) -> BTreeSet<i32> {
    let mut ids = BTreeSet::new();

    for (raw_id, published) in table {
        let Some(name) = published.as_str() else {
            unreachable!("minecraft-data packet names are strings, got {published} at {raw_id}")
        };
        if !names.contains(&name) {
            continue;
        }
        let Some(id) = parse_packet_id(raw_id) else {
            unreachable!("minecraft-data packet ids are hexadecimal, got {raw_id}")
        };
        ids.insert(id);
    }

    ids
}

fn published_name(table: &Map<String, Value>, id: i32) -> Option<&str> {
    for (raw_id, published) in table {
        if parse_packet_id(raw_id) == Some(id) {
            return published.as_str();
        }
    }
    None
}

/// Renders ids the way the tables spell them, so a failure reads in protocol terms.
fn format_ids(ids: &BTreeSet<i32>) -> String {
    let rendered: Vec<String> = ids.iter().map(|id| format!("{id:#04X}")).collect();
    rendered.join(", ")
}

fn tolerates_absence(route: PacketRoute, kind: PacketKind) -> bool {
    UNPUBLISHED_ROUTES.iter().any(|unpublished| {
        unpublished.kind == kind
            && unpublished.state == route.state()
            && unpublished.direction == route.direction()
            && route.version() < unpublished.published_from
    })
}

/// Compares every packet our table maps on `route`, and reports how many were compared.
fn compare_route(protocol: &Value, route: PacketRoute) -> usize {
    let missing = Map::new();
    let table = packet_id_table(protocol, route.state(), route.direction()).unwrap_or(&missing);
    let mut compared = 0;

    for id in 0..=SCANNED_ID_LIMIT {
        let Some(kind) = route.kind_of(id) else {
            continue;
        };
        let names = upstream_names(kind);
        let published = published_ids(table, names);

        if published.is_empty() {
            assert!(
                tolerates_absence(route, kind),
                "{} {:?}/{:?}: we map {kind:?} to {id:#04X}, \
                 minecraft-data publishes no {names:?} at all",
                route.version(),
                route.state(),
                route.direction(),
            );
            continue;
        }

        assert!(
            published == BTreeSet::from([id]),
            "{} {:?}/{:?}: we map {kind:?} to {id:#04X}, but minecraft-data puts {names:?} \
             at [{}] and calls {id:#04X} {:?}",
            route.version(),
            route.state(),
            route.direction(),
            format_ids(&published),
            published_name(table, id),
        );
        compared += 1;
    }

    compared
}

#[test]
fn given_a_version_minecraft_data_publishes_when_our_ids_are_resolved_then_the_names_agree() {
    let Ok(fixture) = serde_json::from_str::<Value>(FIXTURE) else {
        unreachable!("the vendored minecraft-data fixture must be valid JSON")
    };
    let Some(protocols) = fixture.get("protocols").and_then(Value::as_object) else {
        unreachable!("the vendored minecraft-data fixture must carry a `protocols` object")
    };

    let mut comparisons = 0;
    let mut covered = 0;
    let mut skipped = Vec::new();

    for version in ProtocolVersion::all() {
        let Some(protocol) = protocols.get(&version.number().to_string()) else {
            skipped.push(version);
            continue;
        };
        covered += 1;

        for state in ConnectionState::ALL {
            for direction in PacketDirection::ALL {
                comparisons += compare_route(protocol, PacketRoute::new(state, direction, version));
            }
        }
    }

    println!("compared {comparisons} packet ids across {covered} versions");
    for version in &skipped {
        println!(
            "skipped {version} (protocol {}): minecraft-data publishes no protocol.json for it",
            version.number(),
        );
    }

    assert!(
        covered >= MINIMUM_COVERED_VERSIONS,
        "only {covered} of the {} supported versions were cross-checked; \
         the fixture has lost coverage",
        ProtocolVersion::all().len(),
    );
    assert!(
        comparisons >= MINIMUM_COMPARISONS,
        "only {comparisons} ids were cross-checked, expected at least {MINIMUM_COMPARISONS}; \
         the fixture or the tables have lost content",
    );
}

#[test]
fn given_1_20_3_when_the_clientbound_play_disconnect_is_resolved_then_it_keeps_the_1_20_2_id() {
    let route_of =
        |version| PacketRoute::new(ConnectionState::Play, PacketDirection::ClientBound, version);

    // The Java table drops to 0x15 on 1.20.3, repeating the serverbound KeepAlive line above
    // it. 1.20.3 inserted no clientbound packet below 0x42, so the id does not move, and 0x15
    // is set_slot there. minecraft-data and Velocity's StateRegistry both keep it at 0x1B.
    assert_eq!(
        route_of(ProtocolVersion::V1_20_2).id_of(PacketKind::Disconnect),
        Some(0x1B),
    );
    assert_eq!(
        route_of(ProtocolVersion::V1_20_3).id_of(PacketKind::Disconnect),
        Some(0x1B),
    );
    assert_eq!(route_of(ProtocolVersion::V1_20_3).kind_of(0x15), None);
}
