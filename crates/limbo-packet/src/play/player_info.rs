use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use uuid::Uuid;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;
use crate::write_bool::write_bool;

/// The actions this packet carries, as the bit set 1.19.3 replaced the action id with:
/// `add_player` is bit 0, `update_gamemode` bit 2 and `update_listed` bit 3.
///
/// The limbo only ever adds one player, so the set is constant.
const ADD_PLAYER_ACTIONS: u8 = 0b0000_1101;

/// Ping shown next to the player before 1.19.3, when latency was part of adding them.
const LEGACY_LATENCY_MILLIS: i32 = 60;

/// Puts the player in the tab list.
///
/// Only the "add player" action is ever sent, which is what lets three quite different
/// layouts collapse into one packet.
pub struct PlayerInfo<'a> {
    pub game_mode: i32,
    pub username: &'a str,
    pub uuid: Uuid,
}

impl PlayerInfo<'_> {
    /// 1.7 has no tab list entries as such, only a name and a ping.
    fn encode_legacy<B>(&self, buffer: &mut B)
    where
        B: BufMut + ?Sized,
    {
        buffer.write_string(self.username);
        write_bool(buffer, true);
        buffer.put_i16(0);
    }

    fn encode_action_set<B>(&self, buffer: &mut B)
    where
        B: BufMut + ?Sized,
    {
        buffer.put_u8(ADD_PLAYER_ACTIONS);

        buffer.write_var_int(1);
        buffer.write_uuid(self.uuid);
        buffer.write_string(self.username);
        // Signed profile properties, of which the limbo has none.
        buffer.write_var_int(0);

        write_bool(buffer, true);
        buffer.write_var_int(self.game_mode);
    }

    fn encode_add_player_action<B>(&self, buffer: &mut B, version: ProtocolVersion)
    where
        B: BufMut + ?Sized,
    {
        buffer.write_var_int(0);
        buffer.write_var_int(1);
        buffer.write_uuid(self.uuid);
        buffer.write_string(self.username);
        buffer.write_var_int(0);
        buffer.write_var_int(self.game_mode);
        buffer.write_var_int(LEGACY_LATENCY_MILLIS);
        // No display name, so the client renders the username.
        write_bool(buffer, false);

        if version >= ProtocolVersion::V1_19 {
            // No chat session key.
            write_bool(buffer, false);
        }
    }
}

impl ClientboundPacket for PlayerInfo<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::PlayerInfo
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        if version < ProtocolVersion::V1_8 {
            self.encode_legacy(buffer);
        } else if version >= ProtocolVersion::V1_19_3 {
            self.encode_action_set(buffer);
        } else {
            self.encode_add_player_action(buffer, version);
        }

        Ok(())
    }
}
