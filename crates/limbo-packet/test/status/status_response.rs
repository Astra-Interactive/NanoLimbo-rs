use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;
use uuid::Uuid;

use crate::status::{StatusPlayer, StatusResponse};

#[test]
fn given_a_status_response_when_serialized_then_keys_follow_the_declaration_order() {
    let description = Component::text("A Limbo");
    let response = StatusResponse {
        version_name: "NanoLimbo",
        protocol: 767,
        max_players: 100,
        online_players: 1,
        sample: &[],
        description: &description,
    };

    assert_eq!(
        response.to_json(ProtocolVersion::V1_16),
        r#"{"version":{"name":"NanoLimbo","protocol":767},"players":{"max":100,"online":1,"sample":[]},"description":{"text":"A Limbo"}}"#
    );
}

#[test]
fn given_a_client_that_compacts_plain_text_when_serialized_then_the_motd_follows_its_profile() {
    let description = Component::text("A Limbo");
    let response = StatusResponse {
        version_name: "NanoLimbo",
        protocol: 767,
        max_players: 100,
        online_players: 1,
        sample: &[],
        description: &description,
    };

    assert!(
        response
            .to_json(ProtocolVersion::V1_21)
            .ends_with(r#""description":"A Limbo"}"#)
    );
}

#[test]
fn given_a_player_sample_when_serialized_then_each_entry_carries_a_hyphenated_uuid() {
    let description = Component::text("");
    let sample = [StatusPlayer {
        name: "Nano\"Limbo".to_owned(),
        unique_id: Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_0000_0001),
    }];
    let response = StatusResponse {
        version_name: "1.21",
        protocol: 767,
        max_players: 1,
        online_players: 1,
        sample: &sample,
        description: &description,
    };

    assert!(response.to_json(ProtocolVersion::V1_21).contains(
        r#""sample":[{"name":"Nano\"Limbo","uniqueId":"00000000-0000-4000-8000-000000000001"}]"#
    ));
}
