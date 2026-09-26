use limbo_protocol::packet::ConnectionState;
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;

/// Something the runtime must do on behalf of one connection.
///
/// The state machine decides *what* happens and in *what order*; the runtime that owns
/// the socket decides how. Keeping the two apart is what makes the whole login sequence
/// testable without a client, across all 51 versions, in milliseconds.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionAction {
    /// The client named its protocol version. Both codecs must follow.
    AdoptVersion {
        version: ProtocolVersion,
    },

    /// Move both codecs to a new phase.
    EnterState {
        state: ConnectionState,
    },

    /// Move only the outgoing codec. Used at login success on 1.20.2 and later, where the
    /// server starts speaking configuration before the client has acknowledged.
    EnterOutgoingState {
        state: ConnectionState,
    },

    /// Replace the recorded remote address with one a proxy vouched for.
    AdoptForwardedAddress {
        address: String,
    },

    SendStatusResponse,

    /// Echo the client's token and hang up, which is how a ping ends.
    SendPongAndClose {
        payload: i64,
    },

    /// Ask the client for Velocity's forwarded player data.
    ///
    /// Carries no message id: that is drawn from the runtime's id source and handed back
    /// through [`ConnectionFlow::expect_forwarding_reply`], which keeps randomness out of
    /// the decision logic and leaves it reproducible in tests.
    RequestForwardedPlayerInfo,

    SendLoginSuccess,

    /// Count the player as online. Deliberately after login success, as upstream does.
    RegisterPlayer,

    SendBrand,
    SendKnownPacks,
    SendRegistryData,
    SendUpdateTags,
    SendFinishConfiguration,

    /// Send the play-phase burst: join game, abilities, position, and everything the
    /// configuration enables.
    SpawnPlayer,

    /// Send the burst only after a pause.
    ///
    /// Clients up to 1.7.6 drop packets that arrive in the same breath as the join game
    /// packet; upstream waits 100 ms and so must this.
    SpawnPlayerAfterDelay,

    Disconnect {
        reason: Component,
    },
}
