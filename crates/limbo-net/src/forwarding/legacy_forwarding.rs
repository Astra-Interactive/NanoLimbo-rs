use crate::forwarding::forwarded_identity::ForwardedIdentity;
use crate::forwarding::handshake_fields::split_forwarded_fields;
use crate::forwarding::legacy_forwarding_error::LegacyForwardingError;
use crate::identity::parse_uuid;

/// Reads the identity BungeeCord's legacy forwarding packs into the handshake host.
///
/// The host arrives as `host \0 address \0 uuid`, optionally followed by the player's
/// profile properties, which a limbo has no use for and the Java implementation also
/// ignored. Anything else is a client connecting directly to a server that expects a
/// proxy, and is turned away.
pub fn parse_legacy_handshake(host: &str) -> Result<ForwardedIdentity, LegacyForwardingError> {
    let fields = split_forwarded_fields(host);

    match fields.as_slice() {
        [_host, address, uuid_text] | [_host, address, uuid_text, _] => Ok(ForwardedIdentity {
            address: (*address).to_owned(),
            uuid: parse_uuid(uuid_text)?,
        }),
        _ => Err(LegacyForwardingError::UnexpectedFieldCount {
            field_count: fields.len(),
        }),
    }
}
