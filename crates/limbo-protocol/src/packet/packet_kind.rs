/// Identifies a packet by meaning rather than by numeric id.
///
/// The id a packet travels under depends on the connection state and the protocol
/// version, so it is resolved through [`PacketRoute`](crate::packet::PacketRoute)
/// rather than stored on the packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketKind {
    BossBar,
    ChatMessage,
    ChunkWithLight,
    DeclareCommands,
    Disconnect,
    FinishConfiguration,
    GameEvent,
    Handshake,
    JoinGame,
    KeepAlive,
    KnownPacks,
    LoginAcknowledged,
    LoginDisconnect,
    LoginPluginRequest,
    LoginPluginResponse,
    LoginStart,
    LoginSuccess,
    PlayerAbilities,
    PlayerInfo,
    PlayerListHeader,
    PlayerPositionAndLook,
    PluginMessage,
    RegistryData,
    SpawnPosition,
    StatusPing,
    StatusRequest,
    StatusResponse,
    TitleLegacy,
    TitleSetSubTitle,
    TitleSetTitle,
    TitleTimes,
    UpdateTags,
}
