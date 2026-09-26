use bytes::{Buf, BufMut, BytesMut};
use limbo_protocol::buffer::{PacketDecodeError, ProtocolWrite};
use limbo_protocol::packet::{ConnectionState, PacketDirection, PacketRoute};
use limbo_protocol::version::ProtocolVersion;

use crate::serverbound::ClientIntent;
use crate::serverbound::{Handshake, LoginStart, ServerBoundPacket};

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
