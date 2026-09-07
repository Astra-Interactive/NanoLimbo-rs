use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;
use limbo_text::to_json_for;
use serde_json::Value;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;
use crate::status::status_player::StatusPlayer;

/// Renders `text` as a JSON string literal, escaped the way the protocol requires.
fn json_string(text: &str) -> String {
    Value::String(text.to_owned()).to_string()
}

fn sample_json(sample: &[StatusPlayer]) -> String {
    let entries: Vec<String> = sample
        .iter()
        .map(|player| {
            format!(
                r#"{{"name":{},"uniqueId":{}}}"#,
                json_string(&player.name),
                json_string(&player.unique_id.hyphenated().to_string())
            )
        })
        .collect();

    format!("[{}]", entries.join(","))
}

/// The answer to a server list ping: the version line, the player counts and the MOTD.
pub struct StatusResponse<'a> {
    /// Shown when the client's protocol number does not match `protocol`, which is how
    /// the limbo puts its own name in the version slot.
    pub version_name: &'a str,
    pub protocol: i32,
    pub max_players: i32,
    pub online_players: i32,
    pub sample: &'a [StatusPlayer],
    pub description: &'a Component,
}

impl StatusResponse<'_> {
    /// Builds the JSON document the client parses.
    ///
    /// Assembled by hand rather than through a serializer because the key order is
    /// wire-visible and has to match Gson's, which writes fields in declaration order:
    /// `version`, `players`, `description`. A `serde_json::Map` would sort them, and
    /// enabling `preserve_order` would change how components serialize everywhere else.
    fn to_json(&self, version: ProtocolVersion) -> String {
        format!(
            r#"{{"version":{{"name":{},"protocol":{}}},"players":{{"max":{},"online":{},"sample":{}}},"description":{}}}"#,
            json_string(self.version_name),
            self.protocol,
            self.max_players,
            self.online_players,
            sample_json(self.sample),
            to_json_for(self.description, version)
        )
    }
}

impl ClientboundPacket for StatusResponse<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::StatusResponse
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.write_string(&self.to_json(version));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

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
}
