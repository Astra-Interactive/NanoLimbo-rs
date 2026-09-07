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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::identity::UuidParseError;

    use super::*;

    const PLAYER_UUID: Uuid = Uuid::from_u128(0x29c6_6bf5_7218_3158_9983_b554_b116_9e82);

    #[test]
    fn given_the_three_fields_bungeecord_sends_when_parsed_then_the_identity_is_read() {
        let identity = parse_legacy_handshake(
            "limbo.example\u{0}198.51.100.7\u{0}29c66bf5721831589983b554b1169e82",
        )
        .unwrap();

        assert_eq!(
            identity,
            ForwardedIdentity {
                address: "198.51.100.7".to_owned(),
                uuid: PLAYER_UUID,
            }
        );
    }

    #[test]
    fn given_profile_properties_after_the_uuid_when_parsed_then_they_are_ignored() {
        let identity = parse_legacy_handshake(
            "limbo.example\u{0}198.51.100.7\u{0}29c66bf5-7218-3158-9983-b554b1169e82\u{0}[{\"name\":\"textures\"}]",
        )
        .unwrap();

        assert_eq!(identity.uuid, PLAYER_UUID);
    }

    #[test]
    fn given_a_direct_connection_without_forwarded_fields_when_parsed_then_it_is_turned_away() {
        let error = parse_legacy_handshake("limbo.example").unwrap_err();

        assert_eq!(
            error,
            LegacyForwardingError::UnexpectedFieldCount { field_count: 1 }
        );
    }

    #[test]
    fn given_more_fields_than_bungeecord_sends_when_parsed_then_it_is_turned_away() {
        let error =
            parse_legacy_handshake("a\u{0}b\u{0}29c66bf5721831589983b554b1169e82\u{0}c\u{0}d")
                .unwrap_err();

        assert_eq!(
            error,
            LegacyForwardingError::UnexpectedFieldCount { field_count: 5 }
        );
    }

    #[test]
    fn given_a_malformed_uuid_when_parsed_then_the_reason_is_reported() {
        let error =
            parse_legacy_handshake("limbo.example\u{0}198.51.100.7\u{0}not-a-uuid").unwrap_err();

        assert_eq!(
            error,
            LegacyForwardingError::MalformedUuid(UuidParseError::UnexpectedLength { length: 10 })
        );
    }
}
