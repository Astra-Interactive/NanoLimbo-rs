use uuid::Uuid;

/// Who a proxy says the player is.
///
/// The username is not part of this: the handshake based schemes take it from the login
/// packet that follows, so only modern forwarding carries one (see
/// [`ForwardedProfile`](crate::forwarding::ForwardedProfile)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardedIdentity {
    /// The player's address as the proxy saw it, which the limbo logs in place of the
    /// proxy's own.
    pub address: String,
    pub uuid: Uuid,
}
