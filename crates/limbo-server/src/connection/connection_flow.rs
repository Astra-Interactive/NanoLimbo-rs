use limbo_protocol::packet::ConnectionState;
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;

use crate::connection::ConnectionAction;
use crate::connection::ForwardingMode;
use crate::connection::ServerPolicy;
use crate::player::GameProfile;
use crate::serverbound::ServerBoundPacket;

/// Clients up to this version need the play burst delayed.
const LAST_VERSION_NEEDING_SPAWN_DELAY: ProtocolVersion = ProtocolVersion::V1_7_6;

/// First version with a configuration phase between login and play.
pub(crate) const FIRST_CONFIGURATION_VERSION: ProtocolVersion = ProtocolVersion::V1_20_2;

/// First version that negotiates resource packs before receiving registries.
pub(crate) const FIRST_KNOWN_PACKS_VERSION: ProtocolVersion = ProtocolVersion::V1_20_5;

fn refused(message: &str) -> ConnectionAction {
    let mut reason = Component::text(message);
    reason.style.color = Some(limbo_text::chat::TextColor::Named(
        limbo_text::chat::NamedColor::Red,
    ));
    ConnectionAction::Disconnect { reason }
}

/// Drives one connection from its first byte to the play phase.
///
/// Holds no socket and performs no I/O: every step is a decoded packet in and a list of
/// actions out. That is what lets the whole login sequence be replayed for all 51
/// versions in a unit test.
#[derive(Debug, Clone)]
pub struct ConnectionFlow {
    policy: ServerPolicy,
    state: ConnectionState,
    version: ProtocolVersion,
    profile: GameProfile,
    forwarding_request_id: Option<i32>,
}

impl ConnectionFlow {
    pub const fn new(policy: ServerPolicy) -> Self {
        Self {
            policy,
            state: ConnectionState::Handshaking,
            version: ProtocolVersion::MIN,
            profile: GameProfile::unidentified(),
            forwarding_request_id: None,
        }
    }

    pub const fn state(&self) -> ConnectionState {
        self.state
    }

    pub const fn version(&self) -> ProtocolVersion {
        self.version
    }

    pub const fn profile(&self) -> &GameProfile {
        &self.profile
    }

    /// Whether the player has reached a phase where they count towards the online total.
    pub const fn is_registered(&self) -> bool {
        matches!(
            self.state,
            ConnectionState::Configuration | ConnectionState::Play
        )
    }

    /// The steps that follow a successful login, once the identity is settled.
    ///
    /// From 1.20.2 the server switches its outgoing codec to configuration immediately
    /// and waits for the client to acknowledge; older clients go straight to play.
    pub(crate) fn complete_login(&mut self) -> Vec<ConnectionAction> {
        if self.policy.forwarding == ForwardingMode::Modern && self.forwarding_request_id.is_none()
        {
            return vec![refused("You need to connect with Velocity")];
        }

        let mut actions = vec![
            ConnectionAction::SendLoginSuccess,
            ConnectionAction::RegisterPlayer,
        ];

        if self.version >= FIRST_CONFIGURATION_VERSION {
            self.state = ConnectionState::Configuration;
            actions.push(ConnectionAction::EnterOutgoingState {
                state: ConnectionState::Configuration,
            });
            return actions;
        }

        actions.extend(self.spawn_player());
        actions
    }

    fn spawn_player(&mut self) -> Vec<ConnectionAction> {
        self.state = ConnectionState::Play;
        vec![
            ConnectionAction::EnterState {
                state: ConnectionState::Play,
            },
            if self.version <= LAST_VERSION_NEEDING_SPAWN_DELAY {
                ConnectionAction::SpawnPlayerAfterDelay
            } else {
                ConnectionAction::SpawnPlayer
            },
        ]
    }

    fn on_handshake(&mut self, packet: &crate::serverbound::Handshake) -> Vec<ConnectionAction> {
        let Some(intent) = packet.intent else {
            return vec![refused("Invalid handshake intent!")];
        };

        let Some(version) = ProtocolVersion::from_number(packet.protocol) else {
            // The codecs stay on the oldest layout, which is the only one an unknown
            // client is guaranteed to understand well enough to read a kick screen.
            self.state = ConnectionState::Login;
            return vec![
                ConnectionAction::EnterState {
                    state: ConnectionState::Login,
                },
                refused("Unsupported client version"),
            ];
        };
        self.version = version;

        if !intent.is_join() {
            self.state = ConnectionState::Status;
            return vec![
                ConnectionAction::AdoptVersion { version },
                ConnectionAction::EnterState {
                    state: ConnectionState::Status,
                },
            ];
        }

        self.state = ConnectionState::Login;
        vec![
            ConnectionAction::AdoptVersion { version },
            ConnectionAction::EnterState {
                state: ConnectionState::Login,
            },
        ]
    }

    fn on_login_start(&mut self, username: String, online: i32) -> Vec<ConnectionAction> {
        if self.policy.is_full(online) {
            return vec![refused("Too many players connected")];
        }

        if self.policy.forwarding == ForwardingMode::Modern {
            // The identity arrives later, in the plugin response.
            return vec![ConnectionAction::RequestForwardedPlayerInfo];
        }

        self.profile.username = Some(username);
        self.complete_login()
    }

    fn on_login_acknowledged(&mut self) -> Vec<ConnectionAction> {
        self.state = ConnectionState::Configuration;
        let mut actions = vec![
            ConnectionAction::EnterState {
                state: ConnectionState::Configuration,
            },
            ConnectionAction::SendBrand,
        ];

        if self.version >= FIRST_KNOWN_PACKS_VERSION {
            actions.push(ConnectionAction::SendKnownPacks);
            return actions;
        }

        actions.push(ConnectionAction::SendRegistryData);
        actions.push(ConnectionAction::SendFinishConfiguration);
        actions
    }

    /// Feeds one decoded packet through the flow.
    ///
    /// `online` is the current player count, needed only to answer status requests and to
    /// enforce the player cap; passing it in keeps the flow free of shared state.
    pub fn handle(&mut self, packet: ServerBoundPacket, online: i32) -> Vec<ConnectionAction> {
        match packet {
            ServerBoundPacket::Handshake(handshake) => self.on_handshake(&handshake),
            ServerBoundPacket::StatusRequest => vec![ConnectionAction::SendStatusResponse],
            ServerBoundPacket::StatusPing { payload } => {
                vec![ConnectionAction::SendPongAndClose { payload }]
            }
            ServerBoundPacket::LoginStart(login) => self.on_login_start(login.username, online),
            ServerBoundPacket::LoginPluginResponse(_) => Vec::new(),
            ServerBoundPacket::LoginAcknowledged => self.on_login_acknowledged(),
            ServerBoundPacket::KnownPacks => vec![
                ConnectionAction::SendRegistryData,
                ConnectionAction::SendUpdateTags,
                ConnectionAction::SendFinishConfiguration,
            ],
            ServerBoundPacket::FinishConfiguration => self.spawn_player(),
            ServerBoundPacket::PluginMessage(_) | ServerBoundPacket::KeepAlive { .. } => Vec::new(),
        }
    }

    /// Records the identity a proxy vouched for, and resumes the login.
    pub fn accept_forwarded_identity(
        &mut self,
        username: String,
        uuid: uuid::Uuid,
        address: String,
    ) -> Vec<ConnectionAction> {
        self.profile.username = Some(username);
        self.profile.uuid = Some(uuid);

        let mut actions = vec![ConnectionAction::AdoptForwardedAddress { address }];
        actions.extend(self.complete_login());
        actions
    }

    /// Notes which login plugin request the server sent, so a reply can be matched to it.
    pub const fn expect_forwarding_reply(&mut self, message_id: i32) {
        self.forwarding_request_id = Some(message_id);
    }

    pub const fn forwarding_reply_matches(&self, message_id: i32) -> bool {
        match self.forwarding_request_id {
            Some(expected) => expected == message_id,
            None => false,
        }
    }

    /// Sets the offline-mode identity, once it has been derived from the username.
    pub fn adopt_offline_identity(&mut self, uuid: uuid::Uuid) {
        self.profile.uuid = Some(uuid);
    }
}
