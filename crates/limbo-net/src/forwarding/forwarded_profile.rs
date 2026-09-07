use crate::forwarding::forwarded_identity::ForwardedIdentity;

/// Everything Velocity's modern forwarding says about the player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForwardedProfile {
    pub identity: ForwardedIdentity,
    pub username: String,
}
