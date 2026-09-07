use std::collections::HashMap;

use limbo_config::LimboConfig;
use limbo_packet::configuration::{
    FinishConfiguration, KnownPack, KnownPacks, RegistryData, RegistryEntry, UpdateTags,
};
use limbo_packet::login::LoginSuccess;
use limbo_packet::play::{
    BossBar, BossBarColor, BossBarDivision, ChatMessage, ChatPosition, ChunkWithLight,
    DeclareCommands, GameEvent, JoinGame, PlayerAbilities, PlayerInfo, PlayerListHeader,
    PlayerPositionAndLook, PluginMessage, SpawnPosition, TitleLegacy, TitleSetSubTitle,
    TitleSetTitle, TitleTimes,
};
use limbo_packet::{PacketEncodeError, PreEncodedPacket};
use limbo_protocol::version::ProtocolVersion;
use limbo_server::id_source::IdSource;
use limbo_world::{DimensionRegistry, UpdateTagsRegistry, VersionedDimension};
use uuid::Uuid;

use crate::title_snapshots::TitleSnapshots;

/// Channel the server announces its name on, shown under F3.
const BRAND_CHANNEL: &str = "minecraft:brand";

/// Usernames are capped at sixteen characters, and the player list entry is a username.
const MAX_USERNAME_LENGTH: usize = 16;

/// Height the spawn point is recorded at, as whole blocks.
const SPAWN_BLOCK_HEIGHT: i64 = 400;

/// Height the player is parked at. High enough that 1.19 clients do not get stuck in
/// terrain loading, which is why upstream moved it off the ground.
const SPAWN_HEIGHT: f64 = 400.0;

/// Where clients older than 1.9 are parked instead, since they predate that problem.
const LEGACY_SPAWN_HEIGHT: f64 = 64.0;

/// Game event that tells the client to stop waiting and render.
const WAITING_FOR_CHUNKS_EVENT: u8 = 13;

/// Radius, in chunks, of the empty area sent to clients that expect terrain.
const CHUNK_RADIUS: i32 = 1;

/// First version that reads registries one at a time rather than as a single codec.
const FIRST_SPLIT_REGISTRY_VERSION: ProtocolVersion = ProtocolVersion::V1_20_5;

/// The versions that go through a configuration phase with separate registries.
fn configuring_versions() -> impl Iterator<Item = ProtocolVersion> {
    ProtocolVersion::all().filter(|version| *version >= FIRST_SPLIT_REGISTRY_VERSION)
}

fn boss_bar_color(color: limbo_config::BossBarColor) -> BossBarColor {
    match color {
        limbo_config::BossBarColor::Pink => BossBarColor::Pink,
        limbo_config::BossBarColor::Blue => BossBarColor::Blue,
        limbo_config::BossBarColor::Red => BossBarColor::Red,
        limbo_config::BossBarColor::Green => BossBarColor::Green,
        limbo_config::BossBarColor::Yellow => BossBarColor::Yellow,
        limbo_config::BossBarColor::Purple => BossBarColor::Purple,
        limbo_config::BossBarColor::White => BossBarColor::White,
    }
}

fn boss_bar_division(division: limbo_config::BossBarDivision) -> BossBarDivision {
    match division {
        limbo_config::BossBarDivision::Solid => BossBarDivision::Solid,
        limbo_config::BossBarDivision::Dashes6 => BossBarDivision::Dashes6,
        limbo_config::BossBarDivision::Dashes10 => BossBarDivision::Dashes10,
        limbo_config::BossBarDivision::Dashes12 => BossBarDivision::Dashes12,
        limbo_config::BossBarDivision::Dashes20 => BossBarDivision::Dashes20,
    }
}

/// The player list entry's name, trimmed to what the protocol allows.
fn player_list_username(config: &LimboConfig) -> String {
    config
        .player_list
        .username
        .chars()
        .take(MAX_USERNAME_LENGTH)
        .collect()
}

/// Encodes the server brand as the plugin message payload the client reads.
fn brand_payload(brand: &limbo_text::chat::Component) -> Vec<u8> {
    use limbo_protocol::buffer::ProtocolWrite;

    let mut payload = Vec::new();
    payload.write_string(&limbo_text::to_legacy_string(brand));
    payload
}

/// Every packet the server can send, encoded once per version before the listener opens.
///
/// This is the port of Java's `PacketSnapshots`, which held them in public static fields.
/// Here it is a value the composition root builds and hands down, so nothing reaches it
/// by ambient access and a test can build a second one with different settings.
pub struct PacketSnapshots {
    pub login_success: PreEncodedPacket,
    pub join_game: PreEncodedPacket,
    pub player_abilities: PreEncodedPacket,
    pub position_and_look: PreEncodedPacket,
    pub position_and_look_legacy: PreEncodedPacket,
    pub spawn_position: PreEncodedPacket,
    pub player_info: PreEncodedPacket,
    pub declare_commands: PreEncodedPacket,
    pub finish_configuration: PreEncodedPacket,
    pub known_packs: PreEncodedPacket,
    pub update_tags: PreEncodedPacket,
    /// The whole dimension codec, for the two versions that read it that way.
    pub registry_data: PreEncodedPacket,
    /// One packet per registry, for clients that read them separately.
    pub split_registry_data: HashMap<ProtocolVersion, Vec<PreEncodedPacket>>,
    pub start_waiting_chunks: PreEncodedPacket,
    pub chunks: Vec<PreEncodedPacket>,
    pub brand: Option<PreEncodedPacket>,
    pub join_message: Option<PreEncodedPacket>,
    pub boss_bar: Option<PreEncodedPacket>,
    pub header_and_footer: Option<PreEncodedPacket>,
    pub title: Option<TitleSnapshots>,
    /// The identity every player is shown as in the tab list.
    pub player_list_uuid: Uuid,
    pub player_list_username: String,
}

impl PacketSnapshots {
    fn build_registry_data(
        dimensions: &DimensionRegistry,
    ) -> Result<PreEncodedPacket, PacketEncodeError> {
        PreEncodedPacket::for_versions(
            ProtocolVersion::all().filter(|version| *version < FIRST_SPLIT_REGISTRY_VERSION),
            |version| match dimensions.codec(version) {
                Some(codec) => Ok(RegistryData::WholeCodec { codec }),
                None => Err(PacketEncodeError::DimensionUnresolved {
                    key: "codec".to_owned(),
                    version,
                }),
            },
        )
    }

    fn build_split_registry_data(
        dimensions: &DimensionRegistry,
    ) -> Result<HashMap<ProtocolVersion, Vec<PreEncodedPacket>>, PacketEncodeError> {
        let mut per_version = HashMap::new();

        for version in ProtocolVersion::all().filter(|v| *v >= FIRST_SPLIT_REGISTRY_VERSION) {
            let Some(codec) = dimensions.codec(version) else {
                return Err(PacketEncodeError::DimensionUnresolved {
                    key: "codec".to_owned(),
                    version,
                });
            };

            let mut packets = Vec::new();
            for registry in RegistryData::split_codec(codec) {
                packets.push(PreEncodedPacket::for_versions([version], |_same| {
                    Ok(match &registry {
                        RegistryData::WholeCodec { codec } => RegistryData::WholeCodec { codec },
                        RegistryData::Registry { key, entries } => RegistryData::Registry {
                            key,
                            entries: entries
                                .iter()
                                .map(|entry| RegistryEntry {
                                    name: entry.name,
                                    element: entry.element,
                                })
                                .collect(),
                        },
                    })
                })?);
            }
            per_version.insert(version, packets);
        }

        Ok(per_version)
    }

    fn build_chunks(
        dimension: &VersionedDimension,
    ) -> Result<Vec<PreEncodedPacket>, PacketEncodeError> {
        let mut chunks = Vec::new();
        for x in -CHUNK_RADIUS..=CHUNK_RADIUS {
            for z in -CHUNK_RADIUS..=CHUNK_RADIUS {
                chunks.push(PreEncodedPacket::for_all_versions(|_version| {
                    Ok(ChunkWithLight { x, z, dimension })
                })?);
            }
        }
        Ok(chunks)
    }

    /// Encodes everything, once, for every version.
    ///
    /// All the cost of the server's output lives here: after this returns, sending a
    /// packet is a reference count bump and a length prefix. Failing here means the
    /// server cannot serve some version at all, so it is reported rather than deferred.
    pub fn build(
        config: &LimboConfig,
        dimensions: &DimensionRegistry,
        dimension: &VersionedDimension,
        tags: &UpdateTagsRegistry,
        ids: &impl IdSource,
    ) -> Result<Self, PacketEncodeError> {
        // Both of these vary per version and are borrowed by the packets, so they are
        // materialised before encoding rather than leaked from inside the closure.
        let known_packs: HashMap<ProtocolVersion, Vec<KnownPack<'_>>> = configuring_versions()
            .map(|version| {
                let pack = KnownPack {
                    namespace: "minecraft",
                    id: "core",
                    version: version.display_name().unwrap_or_default(),
                };
                (version, vec![pack])
            })
            .collect();

        let mut update_tags = HashMap::new();
        for version in configuring_versions() {
            let registries = tags.tags_for(version).map_err(|_unsupported| {
                PacketEncodeError::DimensionUnresolved {
                    key: "tags".to_owned(),
                    version,
                }
            })?;
            update_tags.insert(version, registries);
        }

        let username = player_list_username(config);
        let player_uuid = limbo_net::identity::offline_mode_uuid(&username);
        let session_id = ids.next_uuid();
        let entity_id = ids.next_entity_id();
        let teleport_id = ids.next_teleport_id();

        let login_success = PreEncodedPacket::for_all_versions(|_version| {
            Ok(LoginSuccess {
                uuid: player_uuid,
                username: &username,
                session_id,
            })
        })?;

        let join_game = PreEncodedPacket::for_all_versions(|_version| {
            Ok(JoinGame {
                entity_id,
                hardcore: false,
                game_mode: config.game_mode,
                previous_game_mode: -1,
                dimension,
                level_type: "flat",
                seed: 0,
                difficulty: 0,
                max_players: config.max_players,
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
                secure_profile: config.secure_profile,
            })
        })?;

        let position_of = |height: f64| {
            move |_version: ProtocolVersion| {
                Ok(PlayerPositionAndLook {
                    x: 0.0,
                    y: height,
                    z: 0.0,
                    yaw: 0.0,
                    pitch: 0.0,
                    teleport_id,
                })
            }
        };

        Ok(Self {
            login_success,
            join_game,
            player_abilities: PreEncodedPacket::for_all_versions(|_version| {
                Ok(PlayerAbilities {
                    invincible: false,
                    can_fly: false,
                    flying: true,
                    creative: false,
                    flying_speed: 0.0,
                    field_of_view: 0.1,
                })
            })?,
            position_and_look: PreEncodedPacket::for_all_versions(position_of(SPAWN_HEIGHT))?,
            position_and_look_legacy: PreEncodedPacket::for_all_versions(position_of(
                LEGACY_SPAWN_HEIGHT,
            ))?,
            spawn_position: PreEncodedPacket::for_all_versions(|_version| {
                Ok(SpawnPosition {
                    dimension_key: dimension.key(),
                    x: 0,
                    y: SPAWN_BLOCK_HEIGHT,
                    z: 0,
                    yaw: 0.0,
                    pitch: 0.0,
                })
            })?,
            player_info: PreEncodedPacket::for_all_versions(|_version| {
                Ok(PlayerInfo {
                    game_mode: config.game_mode,
                    username: &username,
                    uuid: player_uuid,
                })
            })?,
            declare_commands: PreEncodedPacket::for_all_versions(|_version| {
                Ok(DeclareCommands { commands: &[] })
            })?,
            finish_configuration: PreEncodedPacket::for_all_versions(|_version| {
                Ok(FinishConfiguration)
            })?,
            known_packs: PreEncodedPacket::for_versions(configuring_versions(), |version| {
                match known_packs.get(&version) {
                    Some(packs) => Ok(KnownPacks { packs }),
                    None => Err(PacketEncodeError::DimensionUnresolved {
                        key: "minecraft:core".to_owned(),
                        version,
                    }),
                }
            })?,
            update_tags: PreEncodedPacket::for_versions(configuring_versions(), |version| {
                match update_tags.get(&version) {
                    Some(registries) => Ok(UpdateTags { registries }),
                    None => Err(PacketEncodeError::DimensionUnresolved {
                        key: "tags".to_owned(),
                        version,
                    }),
                }
            })?,
            registry_data: Self::build_registry_data(dimensions)?,
            split_registry_data: Self::build_split_registry_data(dimensions)?,
            start_waiting_chunks: PreEncodedPacket::for_all_versions(|_version| {
                Ok(GameEvent {
                    event: WAITING_FOR_CHUNKS_EVENT,
                    value: 0.0,
                })
            })?,
            chunks: Self::build_chunks(dimension)?,
            brand: config
                .brand_name
                .as_ref()
                .map(|brand| {
                    let payload = brand_payload(brand);
                    PreEncodedPacket::for_all_versions(|_version| {
                        Ok(PluginMessage {
                            channel: BRAND_CHANNEL,
                            data: &payload,
                        })
                    })
                })
                .transpose()?,
            join_message: config
                .join_message
                .as_ref()
                .map(|message| {
                    let sender = ids.next_uuid();
                    PreEncodedPacket::for_all_versions(|_version| {
                        Ok(ChatMessage {
                            message,
                            position: ChatPosition::SystemMessage,
                            sender,
                        })
                    })
                })
                .transpose()?,
            boss_bar: config
                .boss_bar
                .as_ref()
                .map(|bar| {
                    let uuid = ids.next_uuid();
                    PreEncodedPacket::for_all_versions(|_version| {
                        Ok(BossBar {
                            uuid,
                            text: &bar.text,
                            health: bar.health.value(),
                            color: boss_bar_color(bar.color),
                            division: boss_bar_division(bar.division),
                            flags: 0,
                        })
                    })
                })
                .transpose()?,
            header_and_footer: config
                .header_and_footer
                .as_ref()
                .map(|banner| {
                    PreEncodedPacket::for_all_versions(|_version| {
                        Ok(PlayerListHeader {
                            header: &banner.header,
                            footer: &banner.footer,
                        })
                    })
                })
                .transpose()?,
            title: config
                .title
                .as_ref()
                .map(|title| {
                    // Rebuilt per call rather than captured: the encoder takes it by
                    // value and the closure is an `Fn`, so it cannot give the same one away
                    // twice.
                    let times = || TitleTimes {
                        fade_in: title.fade_in.count(),
                        stay: title.stay.count(),
                        fade_out: title.fade_out.count(),
                    };
                    Ok::<TitleSnapshots, PacketEncodeError>(TitleSnapshots {
                        title: PreEncodedPacket::for_all_versions(|_version| {
                            Ok(TitleSetTitle {
                                title: &title.title,
                            })
                        })?,
                        subtitle: PreEncodedPacket::for_all_versions(|_version| {
                            Ok(TitleSetSubTitle {
                                subtitle: &title.subtitle,
                            })
                        })?,
                        times: PreEncodedPacket::for_all_versions(|_version| Ok(times()))?,
                        legacy_title: PreEncodedPacket::for_all_versions(|_version| {
                            Ok(TitleLegacy::SetTitle {
                                title: TitleSetTitle {
                                    title: &title.title,
                                },
                            })
                        })?,
                        legacy_subtitle: PreEncodedPacket::for_all_versions(|_version| {
                            Ok(TitleLegacy::SetSubtitle {
                                subtitle: TitleSetSubTitle {
                                    subtitle: &title.subtitle,
                                },
                            })
                        })?,
                        legacy_times: PreEncodedPacket::for_all_versions(|_version| {
                            Ok(TitleLegacy::SetTimesAndDisplay { times: times() })
                        })?,
                    })
                })
                .transpose()?,
            player_list_uuid: player_uuid,
            player_list_username: username,
        })
    }
}
