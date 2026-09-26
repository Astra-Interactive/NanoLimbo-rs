/// How the server learns who a connecting player is.
///
/// Mirrors the `infoForwarding.type` setting. The variants differ in *when* the identity
/// arrives: in the handshake for the two proxy formats, in a login plugin response for
/// Velocity, and not at all when the server is reached directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForwardingMode {
    /// Reached directly. The identity is derived from the username.
    None,
    /// BungeeCord packs it into the handshake host field.
    Legacy,
    /// Velocity sends it, signed, in reply to a login plugin request.
    Modern,
    /// BungeeGuard extends the BungeeCord format with a shared token.
    BungeeGuard,
}
