use crate::packet::connection_state::ConnectionState;
use crate::packet::packet_direction::PacketDirection;
use crate::packet::packet_kind::PacketKind;
use crate::packet::packet_mapping::PacketMapping;
use crate::packet::version_range::VersionRange;
use crate::version::ProtocolVersion;

pub(crate) static HANDSHAKING_SERVERBOUND: &[PacketMapping] = &[PacketMapping::new(
    PacketKind::Handshake,
    0x00,
    VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V26_2),
)];

pub(crate) static HANDSHAKING_CLIENTBOUND: &[PacketMapping] = &[];

pub(crate) static STATUS_SERVERBOUND: &[PacketMapping] = &[
    PacketMapping::new(
        PacketKind::StatusRequest,
        0x00,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::StatusPing,
        0x01,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V26_2),
    ),
];

pub(crate) static STATUS_CLIENTBOUND: &[PacketMapping] = &[
    PacketMapping::new(
        PacketKind::StatusResponse,
        0x00,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::StatusPing,
        0x01,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V26_2),
    ),
];

pub(crate) static LOGIN_SERVERBOUND: &[PacketMapping] = &[
    PacketMapping::new(
        PacketKind::LoginStart,
        0x00,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::LoginPluginResponse,
        0x02,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::LoginAcknowledged,
        0x03,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V26_2),
    ),
];

pub(crate) static LOGIN_CLIENTBOUND: &[PacketMapping] = &[
    PacketMapping::new(
        PacketKind::LoginDisconnect,
        0x00,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::LoginSuccess,
        0x02,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::LoginPluginRequest,
        0x04,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V26_2),
    ),
];

pub(crate) static CONFIGURATION_SERVERBOUND: &[PacketMapping] = &[
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x01,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    // Diverges from the Java table, which starts this range at 1.20.2 and
    // collides with FinishConfiguration.
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x02,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::FinishConfiguration,
        0x02,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::FinishConfiguration,
        0x03,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x03,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x04,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::KnownPacks,
        0x07,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V26_2),
    ),
];

pub(crate) static CONFIGURATION_CLIENTBOUND: &[PacketMapping] = &[
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x00,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x01,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x01,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x02,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::FinishConfiguration,
        0x02,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::FinishConfiguration,
        0x03,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x03,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x04,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::KnownPacks,
        0x0E,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::UpdateTags,
        0x0D,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::RegistryData,
        0x05,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::RegistryData,
        0x07,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V26_2),
    ),
];

pub(crate) static PLAY_SERVERBOUND: &[PacketMapping] = &[
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x00,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V1_8),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x0B,
        VersionRange::new(ProtocolVersion::V1_9, ProtocolVersion::V1_11_1),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x0C,
        VersionRange::new(ProtocolVersion::V1_12, ProtocolVersion::V1_12),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x0B,
        VersionRange::new(ProtocolVersion::V1_12_1, ProtocolVersion::V1_12_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x0E,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_13_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x0F,
        VersionRange::new(ProtocolVersion::V1_14, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x10,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x0F,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x11,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x12,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x11,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x12,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x14,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x15,
        VersionRange::new(ProtocolVersion::V1_20_3, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x18,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x1A,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_5),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x1B,
        VersionRange::new(ProtocolVersion::V1_21_6, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x1C,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
];

pub(crate) static PLAY_CLIENTBOUND: &[PacketMapping] = &[
    PacketMapping::new(
        PacketKind::Disconnect,
        0x40,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V1_8),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x1A,
        VersionRange::new(ProtocolVersion::V1_9, ProtocolVersion::V1_12_2),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x1B,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_13_2),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x1A,
        VersionRange::new(ProtocolVersion::V1_14, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x1B,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x1A,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_1),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x19,
        VersionRange::new(ProtocolVersion::V1_16_2, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x1A,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x17,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x19,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x17,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x1A,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    // Diverges from the Java table, which drops to 0x15 on 1.20.3 — a copy of the
    // serverbound KeepAlive line above it. 1.20.3 inserted no clientbound packet below
    // 0x42, so the id does not move; 0x15 is set_slot there. Confirmed by
    // PrismarineJS/minecraft-data and Velocity's StateRegistry.
    PacketMapping::new(
        PacketKind::Disconnect,
        0x1B,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x1D,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x1C,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::Disconnect,
        0x20,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::DeclareCommands,
        0x11,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::DeclareCommands,
        0x12,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::DeclareCommands,
        0x11,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_1),
    ),
    PacketMapping::new(
        PacketKind::DeclareCommands,
        0x10,
        VersionRange::new(ProtocolVersion::V1_16_2, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::DeclareCommands,
        0x12,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::DeclareCommands,
        0x0F,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::DeclareCommands,
        0x0E,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::DeclareCommands,
        0x10,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::DeclareCommands,
        0x11,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::DeclareCommands,
        0x10,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x01,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V1_8),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x23,
        VersionRange::new(ProtocolVersion::V1_9, ProtocolVersion::V1_12_2),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x25,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x26,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x25,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_1),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x24,
        VersionRange::new(ProtocolVersion::V1_16_2, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x26,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x23,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x25,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x24,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x28,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x29,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x2B,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x2C,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x2B,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x30,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::JoinGame,
        0x31,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x19,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_13_2),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x18,
        VersionRange::new(ProtocolVersion::V1_14, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x19,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x18,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_1),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x17,
        VersionRange::new(ProtocolVersion::V1_16_2, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x18,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x15,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x16,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x15,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x17,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x18,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x19,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::PluginMessage,
        0x18,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x39,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V1_8),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x2B,
        VersionRange::new(ProtocolVersion::V1_9, ProtocolVersion::V1_12),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x2C,
        VersionRange::new(ProtocolVersion::V1_12_1, ProtocolVersion::V1_12_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x2E,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_13_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x31,
        VersionRange::new(ProtocolVersion::V1_14, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x32,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x31,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_1),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x30,
        VersionRange::new(ProtocolVersion::V1_16_2, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x32,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x2F,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x31,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x30,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x34,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x36,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x38,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x3A,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x39,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x3E,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::PlayerAbilities,
        0x40,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x08,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V1_8),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x2E,
        VersionRange::new(ProtocolVersion::V1_9, ProtocolVersion::V1_12),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x2F,
        VersionRange::new(ProtocolVersion::V1_12_1, ProtocolVersion::V1_12_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x32,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_13_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x35,
        VersionRange::new(ProtocolVersion::V1_14, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x36,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x35,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_1),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x34,
        VersionRange::new(ProtocolVersion::V1_16_2, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x38,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x36,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x39,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x38,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x3C,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x3E,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x40,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x42,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x41,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x46,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::PlayerPositionAndLook,
        0x48,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x00,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V1_8),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x1F,
        VersionRange::new(ProtocolVersion::V1_9, ProtocolVersion::V1_12_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x21,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_13_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x20,
        VersionRange::new(ProtocolVersion::V1_14, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x21,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x20,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_1),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x1F,
        VersionRange::new(ProtocolVersion::V1_16_2, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x21,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x1E,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x20,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x1F,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x23,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x24,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x26,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x27,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x26,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x2B,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::KeepAlive,
        0x2C,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x02,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V1_8),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x0F,
        VersionRange::new(ProtocolVersion::V1_9, ProtocolVersion::V1_12_2),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x0E,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x0F,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x0E,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x0F,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x5F,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x62,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x60,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x64,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x67,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_2),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x69,
        VersionRange::new(ProtocolVersion::V1_20_3, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x6C,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x73,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x72,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x77,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::ChatMessage,
        0x79,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::BossBar,
        0x0C,
        VersionRange::new(ProtocolVersion::V1_9, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::BossBar,
        0x0D,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::BossBar,
        0x0C,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::BossBar,
        0x0D,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::BossBar,
        0x0A,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::BossBar,
        0x0B,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::BossBar,
        0x0A,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::BossBar,
        0x09,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x38,
        VersionRange::new(ProtocolVersion::V1_7_2, ProtocolVersion::V1_8),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x2D,
        VersionRange::new(ProtocolVersion::V1_9, ProtocolVersion::V1_12),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x2E,
        VersionRange::new(ProtocolVersion::V1_12_1, ProtocolVersion::V1_12_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x30,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_13_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x33,
        VersionRange::new(ProtocolVersion::V1_14, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x34,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x33,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_1),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x32,
        VersionRange::new(ProtocolVersion::V1_16_2, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x36,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x34,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x37,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x36,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x3A,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x3C,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x3E,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x40,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x3F,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x44,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::PlayerInfo,
        0x46,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::TitleLegacy,
        0x45,
        VersionRange::new(ProtocolVersion::V1_8, ProtocolVersion::V1_11_1),
    ),
    PacketMapping::new(
        PacketKind::TitleLegacy,
        0x47,
        VersionRange::new(ProtocolVersion::V1_12, ProtocolVersion::V1_12),
    ),
    PacketMapping::new(
        PacketKind::TitleLegacy,
        0x48,
        VersionRange::new(ProtocolVersion::V1_12_1, ProtocolVersion::V1_12_2),
    ),
    PacketMapping::new(
        PacketKind::TitleLegacy,
        0x4B,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_13_2),
    ),
    PacketMapping::new(
        PacketKind::TitleLegacy,
        0x4F,
        VersionRange::new(ProtocolVersion::V1_14, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::TitleLegacy,
        0x50,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::TitleLegacy,
        0x4F,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x59,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_17_1),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x5A,
        VersionRange::new(ProtocolVersion::V1_18, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x5D,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x5B,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x5F,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x61,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_2),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x63,
        VersionRange::new(ProtocolVersion::V1_20_3, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x65,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x6C,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x6B,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x70,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::TitleSetTitle,
        0x72,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x57,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_17_1),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x58,
        VersionRange::new(ProtocolVersion::V1_18, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x5B,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x59,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x5D,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x5F,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_2),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x61,
        VersionRange::new(ProtocolVersion::V1_20_3, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x63,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x6A,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x69,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x6E,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::TitleSetSubTitle,
        0x70,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x5A,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_17_1),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x5B,
        VersionRange::new(ProtocolVersion::V1_18, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x5E,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x5C,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x60,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x62,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_2),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x64,
        VersionRange::new(ProtocolVersion::V1_20_3, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x66,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x6D,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x6C,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x71,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::TitleTimes,
        0x73,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x47,
        VersionRange::new(ProtocolVersion::V1_8, ProtocolVersion::V1_8),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x48,
        VersionRange::new(ProtocolVersion::V1_9, ProtocolVersion::V1_9_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x47,
        VersionRange::new(ProtocolVersion::V1_9_4, ProtocolVersion::V1_11_1),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x49,
        VersionRange::new(ProtocolVersion::V1_12, ProtocolVersion::V1_12),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x4A,
        VersionRange::new(ProtocolVersion::V1_12_1, ProtocolVersion::V1_12_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x4E,
        VersionRange::new(ProtocolVersion::V1_13, ProtocolVersion::V1_13_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x53,
        VersionRange::new(ProtocolVersion::V1_14, ProtocolVersion::V1_14_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x54,
        VersionRange::new(ProtocolVersion::V1_15, ProtocolVersion::V1_15_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x53,
        VersionRange::new(ProtocolVersion::V1_16, ProtocolVersion::V1_16_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x5E,
        VersionRange::new(ProtocolVersion::V1_17, ProtocolVersion::V1_17_1),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x5F,
        VersionRange::new(ProtocolVersion::V1_18, ProtocolVersion::V1_18_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x60,
        VersionRange::new(ProtocolVersion::V1_19, ProtocolVersion::V1_19),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x63,
        VersionRange::new(ProtocolVersion::V1_19_1, ProtocolVersion::V1_19_1),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x61,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x65,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x68,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_2),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x6A,
        VersionRange::new(ProtocolVersion::V1_20_3, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x6D,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x74,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x73,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x78,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::PlayerListHeader,
        0x7A,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::SpawnPosition,
        0x4C,
        VersionRange::new(ProtocolVersion::V1_19_3, ProtocolVersion::V1_19_3),
    ),
    PacketMapping::new(
        PacketKind::SpawnPosition,
        0x50,
        VersionRange::new(ProtocolVersion::V1_19_4, ProtocolVersion::V1_20),
    ),
    PacketMapping::new(
        PacketKind::SpawnPosition,
        0x52,
        VersionRange::new(ProtocolVersion::V1_20_2, ProtocolVersion::V1_20_2),
    ),
    PacketMapping::new(
        PacketKind::SpawnPosition,
        0x54,
        VersionRange::new(ProtocolVersion::V1_20_3, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::SpawnPosition,
        0x56,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::SpawnPosition,
        0x5B,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::SpawnPosition,
        0x5A,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::SpawnPosition,
        0x5F,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::SpawnPosition,
        0x61,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::GameEvent,
        0x20,
        VersionRange::new(ProtocolVersion::V1_20_3, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::GameEvent,
        0x22,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::GameEvent,
        0x23,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::GameEvent,
        0x22,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::GameEvent,
        0x26,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V26_2),
    ),
    PacketMapping::new(
        PacketKind::ChunkWithLight,
        0x25,
        VersionRange::new(ProtocolVersion::V1_20_3, ProtocolVersion::V1_20_3),
    ),
    PacketMapping::new(
        PacketKind::ChunkWithLight,
        0x27,
        VersionRange::new(ProtocolVersion::V1_20_5, ProtocolVersion::V1_21),
    ),
    PacketMapping::new(
        PacketKind::ChunkWithLight,
        0x28,
        VersionRange::new(ProtocolVersion::V1_21_2, ProtocolVersion::V1_21_4),
    ),
    PacketMapping::new(
        PacketKind::ChunkWithLight,
        0x27,
        VersionRange::new(ProtocolVersion::V1_21_5, ProtocolVersion::V1_21_7),
    ),
    PacketMapping::new(
        PacketKind::ChunkWithLight,
        0x2C,
        VersionRange::new(ProtocolVersion::V1_21_9, ProtocolVersion::V1_21_11),
    ),
    PacketMapping::new(
        PacketKind::ChunkWithLight,
        0x2D,
        VersionRange::new(ProtocolVersion::V26_1, ProtocolVersion::V26_2),
    ),
];

/// Selects the mapping table that governs one direction of one connection state.
pub(crate) const fn table(
    state: ConnectionState,
    direction: PacketDirection,
) -> &'static [PacketMapping] {
    match (state, direction) {
        (ConnectionState::Handshaking, PacketDirection::ServerBound) => HANDSHAKING_SERVERBOUND,
        (ConnectionState::Handshaking, PacketDirection::ClientBound) => HANDSHAKING_CLIENTBOUND,
        (ConnectionState::Status, PacketDirection::ServerBound) => STATUS_SERVERBOUND,
        (ConnectionState::Status, PacketDirection::ClientBound) => STATUS_CLIENTBOUND,
        (ConnectionState::Login, PacketDirection::ServerBound) => LOGIN_SERVERBOUND,
        (ConnectionState::Login, PacketDirection::ClientBound) => LOGIN_CLIENTBOUND,
        (ConnectionState::Configuration, PacketDirection::ServerBound) => CONFIGURATION_SERVERBOUND,
        (ConnectionState::Configuration, PacketDirection::ClientBound) => CONFIGURATION_CLIENTBOUND,
        (ConnectionState::Play, PacketDirection::ServerBound) => PLAY_SERVERBOUND,
        (ConnectionState::Play, PacketDirection::ClientBound) => PLAY_CLIENTBOUND,
    }
}
