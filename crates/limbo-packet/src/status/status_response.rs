use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;
use limbo_text::to_json_for;
use serde_json::Value;

use crate::encoding::ClientboundPacket;
use crate::encoding::PacketEncodeError;
use crate::status::StatusPlayer;

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
    /// Assembled by hand because the key order is wire-visible and has to match Gson's,
    /// which writes fields in declaration order: `version`, `players`, `description`.
    /// Building a `serde_json::Map` instead would hold that order too, but the MOTD
    /// arrives from `limbo-text` already rendered as JSON text, so it would have to be
    /// parsed back into a `Value` purely to be written out again — a failure path bought
    /// for nothing.
    pub(crate) fn to_json(&self, version: ProtocolVersion) -> String {
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
