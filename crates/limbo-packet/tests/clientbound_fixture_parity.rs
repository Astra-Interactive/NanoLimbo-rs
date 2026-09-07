//! Level 1 of the testing pyramid: every clientbound packet, for every version it is
//! sent to, against the bytes the Java implementation produced.
//!
//! The reference lives in `fixtures/packets/clientbound.json`, grouped by the versions
//! that share an encoding. See `fixtures/README.md` for how it was dumped.
//!
//! A payload is compared byte for byte except where it embeds NBT, and NBT compounds are
//! unordered, so two encoders may spell the same tree differently. There are two such
//! regions, kept apart because their reasons are different:
//!
//! * an **embedded dimension codec**, where Java's `ImmutableCollections.SALT` randomises
//!   `Map.of` iteration order per JVM run, so the dump reorders those keys on every run;
//! * a **chat component** from 1.20.3, where Adventure builds the compound through a
//!   `HashMap`-backed builder, so its keys land in Java hash order.
//!
//! Both are parsed and compared as trees; the bytes around them stay exact. The component
//! comparison also undoes one Java defect this port deliberately does not reproduce — see
//! [`undo_empty_key_wrapping`]. Everything else in the dump reproduces byte for byte
//! across runs, so neither exception may be widened to cover a genuine mismatch.

// `clippy.toml` sanctions `expect` and `panic` in tests, but clippy only recognises code
// inside a `#[test]` function, which the module-level helpers of an integration-test
// crate are not. Marking the file `#![cfg(test)]` would satisfy clippy and compile the
// whole suite away if that flag were ever absent, which is the one failure this suite
// must never have.
#![allow(clippy::expect_used, clippy::panic)]

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;

use bytes::{Buf, BytesMut};
use limbo_packet::ClientboundPacket;
use limbo_packet::configuration::{FinishConfiguration, KnownPack, KnownPacks, RegistryData};
use limbo_packet::login::{LoginDisconnect, LoginSuccess};
use limbo_packet::play::{
    BossBar, BossBarColor, BossBarDivision, ChatMessage, ChatPosition, ChunkWithLight,
    DeclareCommands, Disconnect, GameEvent, JoinGame, PlayerAbilities, PlayerInfo,
    PlayerListHeader, PlayerPositionAndLook, PluginMessage, SpawnPosition, TitleSetSubTitle,
    TitleSetTitle, TitleTimes,
};
use limbo_protocol::buffer::{ProtocolRead, ProtocolWrite};
use limbo_protocol::packet::{ConnectionState, PacketDirection, PacketKind, PacketRoute};
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;
use limbo_text::{parse, to_legacy_string};
use limbo_world::{DimensionRegistry, DimensionType, VersionedDimension};
use serde_json::Value as JsonValue;
use uuid::{Uuid, uuid};
use valence_nbt::binary::from_binary;
use valence_nbt::{Compound, List, Value, compound};

/// Values `FixtureDumper` fixed in place of the ones a running server randomises.
const ENTITY_ID: i32 = 1337;
const TELEPORT_ID: i32 = 7654321;
const PLAYER_UUID: Uuid = uuid!("29c66bf5-7218-3158-9983-b554b1169e82");
const SESSION_UUID: Uuid = uuid!("00000000-0000-4000-8000-000000000001");
const BOSS_BAR_UUID: Uuid = uuid!("00000000-0000-4000-8000-000000000002");
const CHAT_SENDER_UUID: Uuid = uuid!("00000000-0000-4000-8000-000000000003");

const USERNAME: &str = "NanoLimbo";
const BRAND_CHANNEL: &str = "minecraft:brand";
const WELCOME_TEXT: &str = "<white>Welcome to the <gradient:blue:white>NanoLimbo<white>!";
const HEADER_TEXT: &str = "<white>Welcome!";
const FOOTER_TEXT: &str = "<gradient:blue:white>NanoLimbo";
const TITLE_TEXT: &str = "<white><b>Welcome!";

/// Every packet the dumper writes. Checked against the fixture so a file that lost an
/// entry fails the run instead of quietly testing less.
const EXPECTED_ENTRIES: [&str; 23] = [
    "boss_bar",
    "chat_message",
    "chunk_with_light",
    "declare_commands",
    "disconnect_play",
    "finish_configuration",
    "game_event",
    "join_game",
    "known_packs",
    "login_disconnect",
    "login_success",
    "player_abilities",
    "player_info",
    "player_list_header",
    "plugin_message_configuration",
    "plugin_message_play",
    "position_and_look",
    "position_and_look_legacy",
    "registry_data_legacy",
    "spawn_position",
    "title_set_subtitle",
    "title_set_title",
    "title_times",
];

/// Number of (packet, version) pairs the fixture describes, so a shrunken file cannot
/// make this suite vacuously green.
const EXPECTED_PAIRS: usize = 788;

/// How many pairs embed a dimension codec, and so are compared as trees rather than as
/// bytes. Pinned so the exception cannot quietly grow.
const CODEC_STRUCTURAL_PAIRS: usize = 27;

/// How many pairs carry a chat component in its NBT form, and so are compared as trees.
const COMPONENT_STRUCTURAL_PAIRS: usize = 72;

/// Java's own table records `0x15` here, which on 1.20.3 is `set_slot`; the Rust table
/// deliberately says `0x1B`. See `fixtures/README.md` and MIGRATION_PLAN.md 3.1.6. A
/// payload does not depend on the id it travels under, so only the id check is skipped.
const DISCONNECT_ID_DEFECT_PROTOCOL: i32 = 765;
const DISCONNECT_ID_IN_JAVA: i32 = 0x15;
const DISCONNECT_ID_CORRECTED: i32 = 0x1B;

/// The first version whose clients read chat components as NBT rather than as JSON text.
const FIRST_NBT_COMPONENT_VERSION: ProtocolVersion = ProtocolVersion::V1_20_3;

fn load_fixture() -> JsonValue {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/packets/clientbound.json");
    let raw = fs::read_to_string(path).expect("the fixture must exist");

    serde_json::from_str(&raw).expect("the fixture must be valid json")
}

fn decode_hex(hex: &str) -> Vec<u8> {
    assert!(hex.len().is_multiple_of(2), "hex must be whole bytes");

    hex.as_bytes()
        .chunks(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).expect("hex is ascii");
            u8::from_str_radix(text, 16).expect("hex digits")
        })
        .collect()
}

fn json_array<'a>(value: &'a JsonValue, key: &str) -> &'a Vec<JsonValue> {
    value
        .get(key)
        .and_then(JsonValue::as_array)
        .unwrap_or_else(|| panic!("the fixture entry must carry an array under {key}"))
}

fn json_string<'a>(value: &'a JsonValue, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(JsonValue::as_str)
        .unwrap_or_else(|| panic!("the fixture entry must carry a string under {key}"))
}

fn protocol_version(number: i64) -> ProtocolVersion {
    let number = i32::try_from(number).expect("a protocol number fits in an i32");

    ProtocolVersion::from_number(number).expect("the fixture only names supported versions")
}

fn protocols_of(group: &JsonValue) -> Vec<ProtocolVersion> {
    json_array(group, "protocols")
        .iter()
        .map(|number| protocol_version(number.as_i64().expect("a protocol is a number")))
        .collect()
}

fn connection_state(name: &str) -> ConnectionState {
    match name {
        "LOGIN" => ConnectionState::Login,
        "CONFIGURATION" => ConnectionState::Configuration,
        "PLAY" => ConnectionState::Play,
        other => panic!("the fixture names an unknown state: {other}"),
    }
}

fn canonical_list(list: &List) -> List {
    match list {
        List::Compound(items) => List::Compound(items.iter().map(canonical_compound).collect()),
        List::List(items) => List::List(items.iter().map(canonical_list).collect()),
        other => other.clone(),
    }
}

fn canonical_value(value: &Value) -> Value {
    match value {
        Value::Compound(compound) => Value::Compound(canonical_compound(compound)),
        Value::List(list) => Value::List(canonical_list(list)),
        other => other.clone(),
    }
}

/// First normalisation: sorts every compound key, recursively, so two encodings of the
/// same tree compare equal however their keys were ordered. List order is left alone,
/// because a component's `extra` is a sequence and its order is what the player reads.
fn canonical_compound(compound: &Compound) -> Compound {
    let mut keys: Vec<&String> = compound.keys().collect();
    keys.sort();

    let mut sorted = Compound::new();
    for key in keys {
        if let Some(value) = compound.get(key) {
            sorted.insert(key.clone(), canonical_value(value));
        }
    }

    sorted
}

fn undo_empty_key_wrapping_list(list: &List) -> List {
    match list {
        List::Compound(items) => {
            List::Compound(items.iter().map(undo_empty_key_wrapping).collect())
        }
        List::List(items) => List::List(items.iter().map(undo_empty_key_wrapping_list).collect()),
        other => other.clone(),
    }
}

fn undo_empty_key_wrapping_value(value: &Value) -> Value {
    match value {
        Value::Compound(compound) => Value::Compound(undo_empty_key_wrapping(compound)),
        Value::List(list) => Value::List(undo_empty_key_wrapping_list(list)),
        other => other.clone(),
    }
}

/// Second normalisation, applied to the **reference side only**: a compound whose sole
/// key is the empty string is rewritten to carry that value under `text`.
///
/// This undoes MIGRATION_PLAN.md 3.1.10 rather than papering over a difference. Java
/// reaches NBT by way of JSON, so from 1.20.3 a child the JSON serializer compacted to a
/// bare string arrives at `NbtUtils.fromJson0` as a list element that is not a compound,
/// and is wrapped under the empty key. The client finds no `text` there and renders
/// nothing, which is why the shipped `settings.yml` join message loses its trailing `!` on
/// every client from 1.20.3. This port builds NBT straight from the component tree, so the
/// text survives; matching Java here would mean reintroducing the defect.
///
/// See `given_a_compacted_child_when_sent_as_nbt_then_the_text_java_drops_survives`, which
/// asserts the surviving text directly.
fn undo_empty_key_wrapping(compound: &Compound) -> Compound {
    let sole_empty_key = compound.len() == 1 && compound.contains_key("");

    let mut rewritten = Compound::new();
    for (key, value) in compound {
        let rewritten_key = if sole_empty_key { "text" } else { key.as_str() };
        rewritten.insert(
            rewritten_key.to_owned(),
            undo_empty_key_wrapping_value(value),
        );
    }

    rewritten
}

/// Reads a compound and leaves `cursor` just past it.
///
/// From 1.20.2 the root carries no name, which is the named form with the two
/// name-length bytes removed; putting them back is enough to reuse the ordinary reader.
fn take_compound(cursor: &mut &[u8], version: ProtocolVersion) -> Compound {
    if version < ProtocolVersion::V1_20_2 {
        let (compound, _root_name): (Compound, String) =
            from_binary(cursor).expect("a named compound");

        return compound;
    }

    let (&root_tag, body) = cursor.split_first().expect("a compound is never empty");
    let mut named = Vec::with_capacity(cursor.len() + 2);
    named.push(root_tag);
    named.extend_from_slice(&[0, 0]);
    named.extend_from_slice(body);

    let mut named_cursor = named.as_slice();
    let (compound, _root_name): (Compound, String) =
        from_binary(&mut named_cursor).expect("a nameless compound");

    cursor.advance(named.len() - named_cursor.len() - 2);

    compound
}

/// A payload cut into the byte runs compared exactly and the two kinds of embedded NBT
/// compared as trees. The two are kept apart because they are excused for different
/// reasons and normalised differently.
struct PayloadSegments {
    exact: Vec<Vec<u8>>,
    codecs: Vec<Compound>,
    components: Vec<Compound>,
}

impl PayloadSegments {
    fn whole(payload: &[u8]) -> Self {
        Self {
            exact: vec![payload.to_vec()],
            codecs: Vec::new(),
            components: Vec::new(),
        }
    }
}

/// Versions whose join game payload embeds the dimension codec, and so cannot be
/// compared byte for byte.
fn join_game_embeds_codec(version: ProtocolVersion) -> bool {
    (ProtocolVersion::V1_16..ProtocolVersion::V1_20_2).contains(&version)
}

/// Splits a join game payload around the codec, and around the joined world's own entry
/// where the client reads that too.
fn split_join_game(payload: &[u8], version: ProtocolVersion) -> PayloadSegments {
    let mut cursor: &[u8] = payload;

    cursor.advance(4);
    if version >= ProtocolVersion::V1_16_2 {
        cursor.advance(1);
    }
    cursor.advance(2);
    let worlds = cursor.read_var_int().expect("a world count");
    for _ in 0..worlds {
        cursor.read_string(256).expect("a world key");
    }

    let prefix_length = payload.len() - cursor.len();
    let mut codecs = vec![take_compound(&mut cursor, version)];
    if (ProtocolVersion::V1_16_2..ProtocolVersion::V1_19).contains(&version) {
        codecs.push(take_compound(&mut cursor, version));
    }

    PayloadSegments {
        exact: vec![
            payload
                .get(..prefix_length)
                .expect("the prefix was just measured")
                .to_vec(),
            cursor.to_vec(),
        ],
        codecs,
        components: Vec::new(),
    }
}

fn split_registry_data(payload: &[u8], version: ProtocolVersion) -> PayloadSegments {
    let mut cursor: &[u8] = payload;
    let codec = take_compound(&mut cursor, version);
    assert!(
        cursor.is_empty(),
        "the payload is one compound and nothing else"
    );

    PayloadSegments {
        exact: Vec::new(),
        codecs: vec![codec],
        components: Vec::new(),
    }
}

/// Splits a payload that opens with `count` chat components, such as the two lines of the
/// player list header.
fn split_leading_components(
    payload: &[u8],
    version: ProtocolVersion,
    count: usize,
) -> PayloadSegments {
    let mut cursor: &[u8] = payload;
    let mut components = Vec::with_capacity(count);
    for _ in 0..count {
        components.push(take_compound(&mut cursor, version));
    }

    PayloadSegments {
        exact: vec![cursor.to_vec()],
        codecs: Vec::new(),
        components,
    }
}

/// The boss bar names and opens the bar before it says what it says.
fn split_boss_bar(payload: &[u8], version: ProtocolVersion) -> PayloadSegments {
    let mut cursor: &[u8] = payload;
    cursor.advance(16);
    cursor.read_var_int().expect("the bar action");

    let prefix_length = payload.len() - cursor.len();
    let component = take_compound(&mut cursor, version);

    PayloadSegments {
        exact: vec![
            payload
                .get(..prefix_length)
                .expect("the prefix was just measured")
                .to_vec(),
            cursor.to_vec(),
        ],
        codecs: Vec::new(),
        components: vec![component],
    }
}

/// Splits the packets that carry chat components, or `None` for the ones that carry none.
///
/// Only called from 1.20.3. Below it a component is a length-prefixed JSON string, which
/// this port reproduces byte for byte, so there is nothing to excuse.
fn split_component_payload(
    name: &str,
    version: ProtocolVersion,
    payload: &[u8],
) -> Option<PayloadSegments> {
    match name {
        "boss_bar" => Some(split_boss_bar(payload, version)),
        "player_list_header" => Some(split_leading_components(payload, version, 2)),
        "chat_message" | "disconnect_play" | "title_set_title" | "title_set_subtitle" => {
            Some(split_leading_components(payload, version, 1))
        }
        _ => None,
    }
}

fn split_payload(name: &str, version: ProtocolVersion, payload: &[u8]) -> PayloadSegments {
    if name == "registry_data_legacy" {
        return split_registry_data(payload, version);
    }
    if name == "join_game" && join_game_embeds_codec(version) {
        return split_join_game(payload, version);
    }
    if version >= FIRST_NBT_COMPONENT_VERSION
        && let Some(segments) = split_component_payload(name, version, payload)
    {
        return segments;
    }

    PayloadSegments::whole(payload)
}

/// A few bytes either side of `offset`, clamped to what the slice actually holds.
fn window_around(bytes: &[u8], offset: usize) -> &[u8] {
    let start = offset.saturating_sub(4).min(bytes.len());
    let end = (offset + 8).min(bytes.len());

    bytes.get(start..end).unwrap_or_default()
}

/// Describes where two byte runs first diverge, in a form short enough to read.
fn describe_difference(expected: &[u8], actual: &[u8]) -> String {
    let offset = expected
        .iter()
        .zip(actual)
        .position(|(reference, ours)| reference != ours)
        .unwrap_or(expected.len().min(actual.len()));
    format!(
        "first differs at byte {offset} of {}/{}; reference {:02x?} ours {:02x?}",
        expected.len(),
        actual.len(),
        window_around(expected, offset),
        window_around(actual, offset)
    )
}

/// The outcome of comparing one payload against the reference: what differed, if
/// anything, and how much of the payload had to be compared as a tree rather than as
/// bytes.
struct PayloadComparison {
    codecs: usize,
    components: usize,
    difference: Option<String>,
}

/// Compares one payload against the reference, and says what differs when it does.
fn compare_payload(
    name: &str,
    version: ProtocolVersion,
    expected: &[u8],
    actual: &[u8],
) -> PayloadComparison {
    let expected = split_payload(name, version, expected);
    let actual = split_payload(name, version, actual);
    let mut difference = None;

    for (index, reference_run) in expected.exact.iter().enumerate() {
        let our_run = actual
            .exact
            .get(index)
            .expect("the same layout on both sides");
        if reference_run != our_run && difference.is_none() {
            difference = Some(format!(
                "byte run {index}: {}",
                describe_difference(reference_run, our_run)
            ));
        }
    }

    for (index, reference_codec) in expected.codecs.iter().enumerate() {
        let our_codec = actual
            .codecs
            .get(index)
            .expect("the same layout on both sides");
        if canonical_compound(reference_codec) != canonical_compound(our_codec)
            && difference.is_none()
        {
            difference = Some(format!("embedded codec {index} differs in content"));
        }
    }

    for (index, reference_component) in expected.components.iter().enumerate() {
        let our_component = actual
            .components
            .get(index)
            .expect("the same layout on both sides");
        let reference_tree = canonical_compound(&undo_empty_key_wrapping(reference_component));
        if reference_tree != canonical_compound(our_component) && difference.is_none() {
            difference = Some(format!("component {index} differs in content"));
        }
    }

    PayloadComparison {
        codecs: expected.codecs.len(),
        components: expected.components.len(),
        difference,
    }
}

/// The bytes and the identity of one encoded packet.
struct EncodedPacket {
    kind: PacketKind,
    payload: Vec<u8>,
}

fn encoded<P>(packet: &P, version: ProtocolVersion) -> EncodedPacket
where
    P: ClientboundPacket,
{
    let mut buffer = BytesMut::new();
    packet
        .encode(&mut buffer, version)
        .expect("encoding a well-formed packet must succeed");

    EncodedPacket {
        kind: packet.kind(),
        payload: buffer.to_vec(),
    }
}

/// The same inputs `FixtureDumper` fed the Java encoders, held together so the borrowed
/// packets have something to point at.
struct FixtureInputs {
    registry: DimensionRegistry,
    dimension: VersionedDimension,
    login_disconnect_reason: Component,
    welcome: Component,
    play_disconnect_reason: Component,
    header: Component,
    footer: Component,
    title: Component,
    brand_payload: Vec<u8>,
}

impl FixtureInputs {
    fn load() -> Self {
        let registry = DimensionRegistry::load().expect("every embedded codec must load");
        let dimension = DimensionType::TheEnd
            .resolve(&registry)
            .expect("the end must resolve for every version");

        let mut brand_payload = Vec::new();
        brand_payload.write_string(&to_legacy_string(&parse(FOOTER_TEXT)));

        Self {
            registry,
            dimension,
            login_disconnect_reason: parse("<red>Unsupported client version"),
            welcome: parse(WELCOME_TEXT),
            play_disconnect_reason: parse("<red>Too many players connected"),
            header: parse(HEADER_TEXT),
            footer: parse(FOOTER_TEXT),
            title: parse(TITLE_TEXT),
            brand_payload,
        }
    }

    fn join_game(&self) -> JoinGame<'_> {
        JoinGame {
            entity_id: ENTITY_ID,
            hardcore: false,
            game_mode: 3,
            previous_game_mode: -1,
            dimension: &self.dimension,
            level_type: "flat",
            seed: 0,
            difficulty: 0,
            max_players: 100,
            view_distance: 0,
            simulation_distance: 2,
            reduced_debug_info: true,
            normal_respawn: true,
            debug: false,
            flat: false,
            limited_crafting: false,
            portal_cooldown: 0,
            sea_level: 0,
            online_mode: false,
            secure_profile: false,
        }
    }

    fn encode(&self, name: &str, version: ProtocolVersion) -> EncodedPacket {
        match name {
            "login_success" => encoded(
                &LoginSuccess {
                    uuid: PLAYER_UUID,
                    username: USERNAME,
                    session_id: SESSION_UUID,
                },
                version,
            ),
            "login_disconnect" => encoded(
                &LoginDisconnect {
                    reason: &self.login_disconnect_reason,
                },
                version,
            ),
            "join_game" => encoded(&self.join_game(), version),
            "player_abilities" => encoded(
                &PlayerAbilities {
                    invincible: false,
                    can_fly: false,
                    flying: true,
                    creative: false,
                    flying_speed: 0.0,
                    field_of_view: 0.1,
                },
                version,
            ),
            "position_and_look_legacy" => encoded(
                &PlayerPositionAndLook {
                    x: 0.0,
                    y: 64.0,
                    z: 0.0,
                    yaw: 0.0,
                    pitch: 0.0,
                    teleport_id: TELEPORT_ID,
                },
                version,
            ),
            "position_and_look" => encoded(
                &PlayerPositionAndLook {
                    x: 0.0,
                    y: 400.0,
                    z: 0.0,
                    yaw: 0.0,
                    pitch: 0.0,
                    teleport_id: TELEPORT_ID,
                },
                version,
            ),
            "spawn_position" => encoded(
                &SpawnPosition {
                    dimension_key: self.dimension.key(),
                    x: 0,
                    y: 400,
                    z: 0,
                    yaw: 0.0,
                    pitch: 0.0,
                },
                version,
            ),
            "player_info" => encoded(
                &PlayerInfo {
                    game_mode: 3,
                    username: USERNAME,
                    uuid: PLAYER_UUID,
                },
                version,
            ),
            "declare_commands" => encoded(&DeclareCommands { commands: &[] }, version),
            "plugin_message_play" | "plugin_message_configuration" => encoded(
                &PluginMessage {
                    channel: BRAND_CHANNEL,
                    data: &self.brand_payload,
                },
                version,
            ),
            "chat_message" => encoded(
                &ChatMessage {
                    message: &self.welcome,
                    position: ChatPosition::SystemMessage,
                    sender: CHAT_SENDER_UUID,
                },
                version,
            ),
            "boss_bar" => encoded(
                &BossBar {
                    uuid: BOSS_BAR_UUID,
                    text: &self.welcome,
                    health: 1.0,
                    color: BossBarColor::Blue,
                    division: BossBarDivision::Solid,
                    flags: 0,
                },
                version,
            ),
            "player_list_header" => encoded(
                &PlayerListHeader {
                    header: &self.header,
                    footer: &self.footer,
                },
                version,
            ),
            "title_set_title" => encoded(&TitleSetTitle { title: &self.title }, version),
            "title_set_subtitle" => encoded(
                &TitleSetSubTitle {
                    subtitle: &self.footer,
                },
                version,
            ),
            "title_times" => encoded(
                &TitleTimes {
                    fade_in: 10,
                    stay: 100,
                    fade_out: 10,
                },
                version,
            ),
            "disconnect_play" => encoded(
                &Disconnect {
                    reason: &self.play_disconnect_reason,
                },
                version,
            ),
            "game_event" => encoded(
                &GameEvent {
                    event: 13,
                    value: 0.0,
                },
                version,
            ),
            "chunk_with_light" => encoded(
                &ChunkWithLight {
                    x: 0,
                    z: 0,
                    dimension: &self.dimension,
                },
                version,
            ),
            "finish_configuration" => encoded(&FinishConfiguration, version),
            "known_packs" => {
                let packs = [KnownPack {
                    namespace: "minecraft",
                    id: "core",
                    version: version
                        .display_name()
                        .expect("a supported release is named"),
                }];

                encoded(&KnownPacks { packs: &packs }, version)
            }
            "registry_data_legacy" => encoded(
                &RegistryData::WholeCodec {
                    codec: self
                        .registry
                        .codec(ProtocolVersion::V1_20)
                        .expect("the 1.20 codec must load"),
                },
                version,
            ),
            other => panic!("the fixture names a packet this test cannot build: {other}"),
        }
    }
}

#[test]
fn given_the_fixture_when_read_then_it_still_describes_every_packet_and_version_pair() {
    let fixture = load_fixture();
    let entries = json_array(&fixture, "entries");

    let names: BTreeSet<&str> = entries
        .iter()
        .map(|entry| json_string(entry, "name"))
        .collect();
    assert_eq!(
        names,
        BTreeSet::from(EXPECTED_ENTRIES),
        "the fixture no longer describes the same set of packets"
    );

    let mut pairs = 0;
    for entry in entries {
        let name = json_string(entry, "name");

        let encoded_protocols: BTreeSet<i32> = json_array(entry, "encodings")
            .iter()
            .flat_map(protocols_of)
            .map(ProtocolVersion::number)
            .collect();
        let identified_protocols: BTreeSet<i32> = json_array(entry, "ids")
            .iter()
            .flat_map(protocols_of)
            .map(ProtocolVersion::number)
            .collect();

        assert!(!encoded_protocols.is_empty(), "{name} covers no version");
        assert_eq!(
            encoded_protocols, identified_protocols,
            "{name}: the fixture disagrees with itself about which versions it covers"
        );

        pairs += encoded_protocols.len();
    }

    assert_eq!(
        pairs, EXPECTED_PAIRS,
        "the fixture covers a different number of (packet, version) pairs"
    );
}

#[test]
fn given_the_reference_dump_when_every_packet_is_encoded_then_the_payloads_match() {
    let inputs = FixtureInputs::load();
    let fixture = load_fixture();
    let mut compared = 0;
    let mut codec_structural = 0;
    let mut component_structural = 0;
    let mut mismatches = Vec::new();

    for entry in json_array(&fixture, "entries") {
        let name = json_string(entry, "name");

        for group in json_array(entry, "encodings") {
            let expected = decode_hex(json_string(group, "hex"));

            for version in protocols_of(group) {
                let actual = inputs.encode(name, version).payload;
                let comparison = compare_payload(name, version, &expected, &actual);

                if comparison.codecs > 0 {
                    codec_structural += 1;
                }
                if comparison.components > 0 {
                    component_structural += 1;
                }
                if let Some(difference) = comparison.difference {
                    mismatches.push(format!("{name} on {version}: {difference}"));
                }
                compared += 1;
            }
        }
    }

    assert_eq!(compared, EXPECTED_PAIRS);
    assert_eq!(
        codec_structural, CODEC_STRUCTURAL_PAIRS,
        "a different number of pairs embed a dimension codec"
    );
    assert_eq!(
        component_structural, COMPONENT_STRUCTURAL_PAIRS,
        "a different number of pairs carry a chat component as NBT"
    );
    assert!(
        mismatches.is_empty(),
        "{} of {compared} (packet, version) pairs differ from the reference dump:\n{}",
        mismatches.len(),
        mismatches.join("\n")
    );
}

#[test]
fn given_the_reference_dump_when_ids_are_resolved_then_the_tables_agree() {
    let inputs = FixtureInputs::load();
    let fixture = load_fixture();
    let mut compared = 0;

    for entry in json_array(&fixture, "entries") {
        let name = json_string(entry, "name");
        let state = connection_state(json_string(entry, "state"));

        for group in json_array(entry, "ids") {
            let expected = group
                .get("packetId")
                .and_then(JsonValue::as_i64)
                .expect("an id is a number") as i32;

            for version in protocols_of(group) {
                let kind = inputs.encode(name, version).kind;
                let resolved = PacketRoute::new(state, PacketDirection::ClientBound, version)
                    .id_of(kind)
                    .unwrap_or_else(|| panic!("{name} on {version}: the table has no id"));

                if name == "disconnect_play" && version.number() == DISCONNECT_ID_DEFECT_PROTOCOL {
                    assert_eq!(
                        expected, DISCONNECT_ID_IN_JAVA,
                        "the recorded Java defect changed shape"
                    );
                    assert_eq!(
                        resolved, DISCONNECT_ID_CORRECTED,
                        "the correction to the Java defect was lost"
                    );
                } else {
                    assert_eq!(
                        resolved, expected,
                        "{name} on {version}: id differs from the reference dump"
                    );
                }
                compared += 1;
            }
        }
    }

    assert_eq!(compared, EXPECTED_PAIRS);
}

#[test]
fn given_every_encoded_packet_when_grouped_by_content_then_the_version_boundaries_match() {
    let inputs = FixtureInputs::load();
    let fixture = load_fixture();

    for entry in json_array(&fixture, "entries") {
        let name = json_string(entry, "name");
        if name == "registry_data_legacy" || name == "join_game" {
            // Their reference grouping follows Java's per-run NBT key order, which this
            // port neither reproduces nor needs to.
            continue;
        }

        let mut versions_by_payload: HashMap<Vec<u8>, Vec<ProtocolVersion>> = HashMap::new();
        for group in json_array(entry, "encodings") {
            for version in protocols_of(group) {
                versions_by_payload
                    .entry(inputs.encode(name, version).payload)
                    .or_default()
                    .push(version);
            }
        }

        assert_eq!(
            versions_by_payload.len(),
            json_array(entry, "encodings").len(),
            "{name}: the port splits into a different number of distinct encodings, so a \
             version gate moved"
        );
    }
}

fn collect_text_from_list(list: &List, found: &mut Vec<String>) {
    if let List::Compound(items) = list {
        for item in items {
            collect_text(item, found);
        }
    }
}

/// Every `text` a component tree carries, at any depth.
fn collect_text(compound: &Compound, found: &mut Vec<String>) {
    for (key, value) in compound {
        match value {
            Value::String(text) if key == "text" => found.push(text.clone()),
            Value::Compound(nested) => collect_text(nested, found),
            Value::List(list) => collect_text_from_list(list, found),
            _ => {}
        }
    }
}

fn contains_empty_key_in_list(list: &List) -> bool {
    match list {
        List::Compound(items) => items.iter().any(contains_empty_key),
        _ => false,
    }
}

/// Whether any compound in the tree is keyed by the empty string — the shape a client
/// renders as nothing.
fn contains_empty_key(compound: &Compound) -> bool {
    compound.contains_key("")
        || compound.iter().any(|(_key, value)| match value {
            Value::Compound(nested) => contains_empty_key(nested),
            Value::List(list) => contains_empty_key_in_list(list),
            _ => false,
        })
}

/// The regression test behind MIGRATION_PLAN.md 3.1.10, and the reason
/// [`undo_empty_key_wrapping`] exists.
///
/// Java serializes a component to JSON before turning it into NBT, so a trailing child
/// the JSON serializer compacts to a bare string reaches `NbtUtils.fromJson0` as a list
/// element that is not a compound and is wrapped under the empty key. The client finds no
/// `text` there and draws nothing, losing the trailing run. This port builds NBT from the
/// component tree, so the run has to survive.
#[test]
fn given_a_compacted_child_when_sent_as_nbt_then_the_text_java_drops_survives() {
    let message = parse("<red>outer <blue>inner</blue> outer");
    let mut buffer = BytesMut::new();
    ChatMessage {
        message: &message,
        position: ChatPosition::SystemMessage,
        sender: CHAT_SENDER_UUID,
    }
    .encode(&mut buffer, FIRST_NBT_COMPONENT_VERSION)
    .expect("encoding cannot fail");

    let mut cursor: &[u8] = &buffer;
    let component = take_compound(&mut cursor, FIRST_NBT_COMPONENT_VERSION);

    let mut texts = Vec::new();
    collect_text(&component, &mut texts);

    assert!(
        texts.contains(&" outer".to_owned()),
        "the trailing run must reach the client, but the tree only carries {texts:?}"
    );
    assert!(
        !contains_empty_key(&component),
        "no child may be keyed by the empty string, which renders as nothing"
    );
}

#[test]
fn given_a_compound_that_only_looks_wrapped_when_normalised_then_its_empty_key_is_kept() {
    let wrapped = compound! { "" => " outer" };
    let ambiguous = compound! { "" => " outer", "color" => "red" };

    assert_eq!(
        undo_empty_key_wrapping(&wrapped),
        compound! { "text" => " outer" }
    );
    assert_eq!(undo_empty_key_wrapping(&ambiguous), ambiguous);
}
