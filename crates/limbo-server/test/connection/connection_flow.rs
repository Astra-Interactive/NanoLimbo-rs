use bytes::{BufMut, BytesMut};
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::{ConnectionState, PacketDirection, PacketRoute};
use limbo_protocol::version::ProtocolVersion;

use crate::connection::connection_flow::{FIRST_CONFIGURATION_VERSION, FIRST_KNOWN_PACKS_VERSION};
use crate::connection::{ConnectionAction, ConnectionFlow, ForwardingMode, ServerPolicy};
use crate::serverbound::ClientIntent;
use crate::serverbound::Handshake;
use crate::serverbound::LoginStart;
use crate::serverbound::ServerBoundPacket;

fn open() -> ConnectionFlow {
    ConnectionFlow::new(ServerPolicy::new(ForwardingMode::None, 100))
}

fn handshake(version: ProtocolVersion, intent: ClientIntent) -> ServerBoundPacket {
    ServerBoundPacket::Handshake(Handshake {
        protocol: version.number(),
        host: "limbo".to_owned(),
        port: 25565,
        intent: Some(intent),
    })
}

fn login(name: &str) -> ServerBoundPacket {
    ServerBoundPacket::LoginStart(LoginStart {
        username: name.to_owned(),
        uuid: None,
    })
}

/// Replays a full join for one version and returns everything the server would send.
fn join(version: ProtocolVersion) -> Vec<ConnectionAction> {
    let mut flow = open();
    let mut actions = flow.handle(handshake(version, ClientIntent::Login), 0);
    actions.extend(flow.handle(login("Notch"), 0));

    if version >= FIRST_CONFIGURATION_VERSION {
        actions.extend(flow.handle(ServerBoundPacket::LoginAcknowledged, 0));
        if version >= FIRST_KNOWN_PACKS_VERSION {
            actions.extend(flow.handle(ServerBoundPacket::KnownPacks, 0));
        }
        actions.extend(flow.handle(ServerBoundPacket::FinishConfiguration, 0));
    }

    assert_eq!(flow.state(), ConnectionState::Play, "{version}");
    actions
}

#[test]
fn given_any_supported_version_when_it_joins_then_it_reaches_play_and_is_registered() {
    for version in ProtocolVersion::all() {
        let actions = join(version);

        assert!(
            actions.contains(&ConnectionAction::SendLoginSuccess),
            "{version} never received login success"
        );
        assert!(
            actions.contains(&ConnectionAction::RegisterPlayer),
            "{version} was never counted as online"
        );
        assert!(
            actions.contains(&ConnectionAction::SpawnPlayer)
                || actions.contains(&ConnectionAction::SpawnPlayerAfterDelay),
            "{version} never spawned"
        );
    }
}

#[test]
fn given_a_client_older_than_the_configuration_phase_then_login_leads_straight_to_play() {
    let actions = join(ProtocolVersion::V1_8);

    assert!(!actions.iter().any(|action| matches!(
        action,
        ConnectionAction::EnterState {
            state: ConnectionState::Configuration
        }
    )));
    assert!(!actions.contains(&ConnectionAction::SendRegistryData));
}

#[test]
fn given_a_1_20_2_client_when_it_acknowledges_login_then_registries_precede_the_finish() {
    let mut flow = open();
    flow.handle(handshake(ProtocolVersion::V1_20_2, ClientIntent::Login), 0);
    flow.handle(login("Notch"), 0);

    let actions = flow.handle(ServerBoundPacket::LoginAcknowledged, 0);

    assert_eq!(
        actions,
        vec![
            ConnectionAction::EnterState {
                state: ConnectionState::Configuration
            },
            ConnectionAction::SendBrand,
            ConnectionAction::SendRegistryData,
            ConnectionAction::SendFinishConfiguration,
        ]
    );
}

#[test]
fn given_a_1_20_5_client_when_it_acknowledges_login_then_known_packs_come_first() {
    let mut flow = open();
    flow.handle(handshake(ProtocolVersion::V1_20_5, ClientIntent::Login), 0);
    flow.handle(login("Notch"), 0);

    let acknowledged = flow.handle(ServerBoundPacket::LoginAcknowledged, 0);
    assert_eq!(acknowledged.last(), Some(&ConnectionAction::SendKnownPacks));

    let negotiated = flow.handle(ServerBoundPacket::KnownPacks, 0);
    assert_eq!(
        negotiated,
        vec![
            ConnectionAction::SendRegistryData,
            ConnectionAction::SendUpdateTags,
            ConnectionAction::SendFinishConfiguration,
        ]
    );
}

#[test]
fn given_a_client_that_drops_packets_sent_too_early_then_its_spawn_is_delayed() {
    for version in [ProtocolVersion::V1_7_2, ProtocolVersion::V1_7_6] {
        assert!(
            join(version).contains(&ConnectionAction::SpawnPlayerAfterDelay),
            "{version}"
        );
    }
    assert!(join(ProtocolVersion::V1_8).contains(&ConnectionAction::SpawnPlayer));
}

#[test]
fn given_an_unsupported_protocol_when_handshaking_then_the_client_is_refused() {
    let mut flow = open();

    let actions = flow.handle(
        ServerBoundPacket::Handshake(Handshake {
            protocol: 1,
            host: "limbo".to_owned(),
            port: 25565,
            intent: Some(ClientIntent::Login),
        }),
        0,
    );

    assert!(matches!(
        actions.last(),
        Some(ConnectionAction::Disconnect { .. })
    ));
    assert_eq!(
        flow.version(),
        ProtocolVersion::MIN,
        "the kick must go out in the only layout an unknown client can read"
    );
}

#[test]
fn given_a_full_server_when_a_player_logs_in_then_they_are_refused() {
    let mut flow = ConnectionFlow::new(ServerPolicy::new(ForwardingMode::None, 2));
    flow.handle(handshake(ProtocolVersion::V1_20_5, ClientIntent::Login), 2);

    let actions = flow.handle(login("Notch"), 2);

    assert!(matches!(
        actions.first(),
        Some(ConnectionAction::Disconnect { .. })
    ));
}

#[test]
fn given_no_player_cap_when_many_are_online_then_nobody_is_refused() {
    let mut flow = ConnectionFlow::new(ServerPolicy::new(ForwardingMode::None, -1));
    flow.handle(
        handshake(ProtocolVersion::V1_20_5, ClientIntent::Login),
        9999,
    );

    let actions = flow.handle(login("Notch"), 9999);

    assert!(actions.contains(&ConnectionAction::SendLoginSuccess));
}

#[test]
fn given_modern_forwarding_when_a_player_logs_in_then_the_proxy_is_asked_before_anything() {
    let mut flow = ConnectionFlow::new(ServerPolicy::new(ForwardingMode::Modern, 100));
    flow.handle(handshake(ProtocolVersion::V1_20_5, ClientIntent::Login), 0);

    let actions = flow.handle(login("Notch"), 0);

    assert_eq!(actions, vec![ConnectionAction::RequestForwardedPlayerInfo]);
    assert!(!actions.contains(&ConnectionAction::SendLoginSuccess));
}

#[test]
fn given_modern_forwarding_without_a_proxy_reply_then_login_cannot_complete() {
    let mut flow = ConnectionFlow::new(ServerPolicy::new(ForwardingMode::Modern, 100));
    flow.handle(handshake(ProtocolVersion::V1_20_5, ClientIntent::Login), 0);

    let actions = flow.complete_login();

    assert!(matches!(
        actions.as_slice(),
        [ConnectionAction::Disconnect { .. }]
    ));
}

#[test]
fn given_a_proxy_reply_when_the_identity_is_accepted_then_login_completes() {
    let mut flow = ConnectionFlow::new(ServerPolicy::new(ForwardingMode::Modern, 100));
    flow.handle(handshake(ProtocolVersion::V1_20_5, ClientIntent::Login), 0);
    flow.handle(login("Notch"), 0);
    flow.expect_forwarding_reply(7);

    let actions = flow.accept_forwarded_identity(
        "Notch".to_owned(),
        uuid::Uuid::from_u128(1),
        "203.0.113.7".to_owned(),
    );

    assert!(flow.forwarding_reply_matches(7));
    assert!(!flow.forwarding_reply_matches(8));
    assert_eq!(
        actions.first(),
        Some(&ConnectionAction::AdoptForwardedAddress {
            address: "203.0.113.7".to_owned()
        })
    );
    assert!(actions.contains(&ConnectionAction::SendLoginSuccess));
    assert_eq!(flow.profile().uuid, Some(uuid::Uuid::from_u128(1)));
}

#[test]
fn given_a_status_ping_when_handled_then_the_token_is_echoed_and_the_socket_closes() {
    let mut flow = open();
    flow.handle(handshake(ProtocolVersion::V1_20_5, ClientIntent::Status), 0);

    assert_eq!(flow.state(), ConnectionState::Status);
    assert_eq!(
        flow.handle(ServerBoundPacket::StatusRequest, 0),
        vec![ConnectionAction::SendStatusResponse]
    );
    assert_eq!(
        flow.handle(ServerBoundPacket::StatusPing { payload: -5 }, 0),
        vec![ConnectionAction::SendPongAndClose { payload: -5 }]
    );
}

#[test]
fn given_a_transfer_intent_when_handshaking_then_it_is_treated_as_a_join() {
    let mut flow = open();

    flow.handle(
        handshake(ProtocolVersion::V1_20_5, ClientIntent::Transfer),
        0,
    );

    assert_eq!(flow.state(), ConnectionState::Login);
}

#[test]
fn given_a_connection_that_never_logged_in_then_it_does_not_count_as_online() {
    let mut flow = open();
    assert!(!flow.is_registered());

    flow.handle(handshake(ProtocolVersion::V1_20_5, ClientIntent::Login), 0);
    assert!(!flow.is_registered());

    flow.handle(login("Notch"), 0);
    assert!(flow.is_registered());
}

#[test]
fn given_a_decoded_handshake_from_the_wire_when_run_through_the_flow_then_it_agrees() {
    let mut buffer = BytesMut::new();
    buffer.write_var_int(ProtocolVersion::V1_21_5.number());
    buffer.write_string("limbo.example.com");
    buffer.put_u16(25565);
    buffer.write_var_int(2);

    let route = PacketRoute::new(
        ConnectionState::Handshaking,
        PacketDirection::ServerBound,
        ProtocolVersion::MIN,
    );
    let Ok(Some(packet)) = ServerBoundPacket::decode(route, 0x00, &mut buffer) else {
        panic!("handshake must decode");
    };

    let mut flow = open();
    flow.handle(packet, 0);

    assert_eq!(flow.version(), ProtocolVersion::V1_21_5);
    assert_eq!(flow.state(), ConnectionState::Login);
}
