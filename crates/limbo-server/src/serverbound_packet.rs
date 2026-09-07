use bytes::Buf;
use limbo_protocol::buffer::{PacketDecodeError, ProtocolRead};
use limbo_protocol::packet::{PacketKind, PacketRoute};
use limbo_protocol::version::ProtocolVersion;

use crate::handshake::Handshake;
use crate::login_plugin_response::LoginPluginResponse;
use crate::login_start::LoginStart;
use crate::plugin_message::PluginMessage;

/// A client may advertise at most this many resource packs it already has.
const MAX_KNOWN_PACKS: i32 = 16;

/// Namespace, id and version strings inside a known-pack entry.
const MAX_KNOWN_PACK_FIELD_LENGTH: usize = 256;

/// Everything the server understands from a client.
///
/// A limbo server reads ten packets out of the hundred-odd the protocol defines. The rest
/// are recognised by id and dropped, which is why decoding returns an `Option` rather than
/// failing on an id it has no use for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerBoundPacket {
    Handshake(Handshake),
    StatusRequest,
    StatusPing { payload: i64 },
    LoginStart(LoginStart),
    LoginPluginResponse(LoginPluginResponse),
    LoginAcknowledged,
    PluginMessage(PluginMessage),
    FinishConfiguration,
    KnownPacks,
    KeepAlive { id: i64 },
}

/// Consumes a known-packs list without keeping it.
///
/// The server answers the same way regardless of what the client already has, but the
/// list still has to be validated: an unbounded count here would be a client telling the
/// server how much work to do.
fn skip_known_packs<B>(buffer: &mut B) -> Result<(), PacketDecodeError>
where
    B: Buf + ?Sized,
{
    let count = buffer.read_var_int()?;
    if !(0..=MAX_KNOWN_PACKS).contains(&count) {
        return Err(PacketDecodeError::ByteArrayTooLong {
            actual: count.unsigned_abs() as usize,
            limit: MAX_KNOWN_PACKS as usize,
        });
    }

    for _ in 0..count {
        for _ in 0..3 {
            buffer.read_string(MAX_KNOWN_PACK_FIELD_LENGTH)?;
        }
    }
    Ok(())
}

/// Reads a keep-alive token, which changed width twice.
fn read_keep_alive<B>(buffer: &mut B, version: ProtocolVersion) -> Result<i64, PacketDecodeError>
where
    B: Buf + ?Sized,
{
    if version >= ProtocolVersion::V1_12_2 {
        buffer.read_i64()
    } else if version >= ProtocolVersion::V1_8 {
        buffer.read_var_int().map(i64::from)
    } else {
        buffer.read_i32().map(i64::from)
    }
}

impl ServerBoundPacket {
    /// Decodes the packet arriving under `id` on `route`.
    ///
    /// `Ok(None)` means the id is not one the server acts on; the caller drops the frame
    /// and carries on. `Err` means the bytes were malformed, which ends the connection.
    pub fn decode<B>(
        route: PacketRoute,
        id: i32,
        buffer: &mut B,
    ) -> Result<Option<Self>, PacketDecodeError>
    where
        B: Buf + ?Sized,
    {
        let version = route.version();

        let packet = match route.kind_of(id) {
            Some(PacketKind::Handshake) => Self::Handshake(Handshake::decode(buffer)?),
            Some(PacketKind::StatusRequest) => Self::StatusRequest,
            Some(PacketKind::StatusPing) => Self::StatusPing {
                payload: buffer.read_i64()?,
            },
            Some(PacketKind::LoginStart) => Self::LoginStart(LoginStart::decode(buffer, version)?),
            Some(PacketKind::LoginPluginResponse) => {
                Self::LoginPluginResponse(LoginPluginResponse::decode(buffer)?)
            }
            Some(PacketKind::LoginAcknowledged) => Self::LoginAcknowledged,
            Some(PacketKind::PluginMessage) => Self::PluginMessage(PluginMessage::decode(buffer)?),
            Some(PacketKind::FinishConfiguration) => Self::FinishConfiguration,
            Some(PacketKind::KnownPacks) => {
                skip_known_packs(buffer)?;
                Self::KnownPacks
            }
            Some(PacketKind::KeepAlive) => Self::KeepAlive {
                id: read_keep_alive(buffer, version)?,
            },
            _ => return Ok(None),
        };

        Ok(Some(packet))
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};
    use limbo_protocol::buffer::ProtocolWrite;
    use limbo_protocol::packet::{ConnectionState, PacketDirection};

    use crate::client_intent::ClientIntent;

    use super::*;

    fn route(state: ConnectionState, version: ProtocolVersion) -> PacketRoute {
        PacketRoute::new(state, PacketDirection::ServerBound, version)
    }

    fn handshake_bytes(protocol: i32, host: &str, intent: i32) -> BytesMut {
        let mut buffer = BytesMut::new();
        buffer.write_var_int(protocol);
        buffer.write_string(host);
        buffer.put_u16(25565);
        buffer.write_var_int(intent);
        buffer
    }

    #[test]
    fn given_a_handshake_when_decoded_then_its_fields_survive_intact() {
        let mut buffer = handshake_bytes(766, "limbo.example.com", 2);

        let packet = ServerBoundPacket::decode(
            route(ConnectionState::Handshaking, ProtocolVersion::MIN),
            0x00,
            &mut buffer,
        );

        assert_eq!(
            packet,
            Ok(Some(ServerBoundPacket::Handshake(Handshake {
                protocol: 766,
                host: "limbo.example.com".to_owned(),
                port: 25565,
                intent: Some(ClientIntent::Login),
            })))
        );
    }

    #[test]
    fn given_an_unknown_intent_when_decoded_then_it_is_reported_as_absent_not_as_an_error() {
        let mut buffer = handshake_bytes(766, "host", 99);

        let Ok(Some(ServerBoundPacket::Handshake(handshake))) = ServerBoundPacket::decode(
            route(ConnectionState::Handshaking, ProtocolVersion::MIN),
            0x00,
            &mut buffer,
        ) else {
            panic!("handshake must decode");
        };

        assert_eq!(handshake.intent, None);
    }

    #[test]
    fn given_an_id_the_server_ignores_when_decoded_then_the_frame_is_dropped_quietly() {
        let mut buffer = BytesMut::new();

        let packet = ServerBoundPacket::decode(
            route(ConnectionState::Play, ProtocolVersion::V1_20_5),
            0x7E,
            &mut buffer,
        );

        assert_eq!(packet, Ok(None));
    }

    #[test]
    fn given_a_keep_alive_when_decoded_then_its_width_follows_the_version() {
        let mut modern = BytesMut::new();
        modern.put_i64(1234567890);
        assert_eq!(
            ServerBoundPacket::decode(
                route(ConnectionState::Play, ProtocolVersion::V1_20_5),
                0x18,
                &mut modern
            ),
            Ok(Some(ServerBoundPacket::KeepAlive { id: 1234567890 }))
        );

        let mut legacy = BytesMut::new();
        legacy.write_var_int(4242);
        assert_eq!(
            ServerBoundPacket::decode(
                route(ConnectionState::Play, ProtocolVersion::V1_8),
                0x00,
                &mut legacy
            ),
            Ok(Some(ServerBoundPacket::KeepAlive { id: 4242 }))
        );
    }

    #[test]
    fn given_a_login_start_from_1_19_when_decoded_then_the_signing_key_is_stepped_over() {
        let mut buffer = BytesMut::new();
        buffer.write_string("Notch");
        buffer.put_u8(1); // a key follows
        buffer.put_i64(0);
        buffer.write_var_int(4);
        buffer.put_slice(&[1, 2, 3, 4]);
        buffer.write_var_int(2);
        buffer.put_slice(&[5, 6]);

        let packet = ServerBoundPacket::decode(
            route(ConnectionState::Login, ProtocolVersion::V1_19),
            0x00,
            &mut buffer,
        );

        assert_eq!(
            packet,
            Ok(Some(ServerBoundPacket::LoginStart(LoginStart {
                username: "Notch".to_owned(),
                uuid: None,
            })))
        );
        assert!(!buffer.has_remaining(), "the key must be fully consumed");
    }

    #[test]
    fn given_a_login_start_from_1_20_2_when_decoded_then_the_uuid_has_no_preceding_flag() {
        let mut buffer = BytesMut::new();
        buffer.write_string("Notch");
        buffer.write_uuid(uuid::Uuid::from_u128(42));

        let packet = ServerBoundPacket::decode(
            route(ConnectionState::Login, ProtocolVersion::V1_20_2),
            0x00,
            &mut buffer,
        );

        assert_eq!(
            packet,
            Ok(Some(ServerBoundPacket::LoginStart(LoginStart {
                username: "Notch".to_owned(),
                uuid: Some(uuid::Uuid::from_u128(42)),
            })))
        );
    }

    #[test]
    fn given_more_known_packs_than_allowed_when_decoded_then_it_is_rejected() {
        let mut buffer = BytesMut::new();
        buffer.write_var_int(1000);

        let packet = ServerBoundPacket::decode(
            route(ConnectionState::Configuration, ProtocolVersion::V1_20_5),
            0x07,
            &mut buffer,
        );

        assert!(matches!(
            packet,
            Err(PacketDecodeError::ByteArrayTooLong { .. })
        ));
    }

    #[test]
    fn given_a_truncated_handshake_when_decoded_then_it_errors_instead_of_panicking() {
        let mut buffer = BytesMut::new();
        buffer.write_var_int(766);
        buffer.write_string("host");

        let packet = ServerBoundPacket::decode(
            route(ConnectionState::Handshaking, ProtocolVersion::MIN),
            0x00,
            &mut buffer,
        );

        assert!(packet.is_err());
    }
}
