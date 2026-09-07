use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_world::{Dimension, VersionedDimension, write_compound};

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;
use crate::write_bool::write_bool;

/// Spectator mode, the only game mode 1.7 does not have and so has to be mapped away.
const GAME_MODE_SPECTATOR: i32 = 3;

/// Drops the player into a world: their entity, the dimension they are in and the rules
/// the client applies while they are there.
///
/// Java calls this `PacketLogin`; the protocol calls it Join Game. It carries more
/// version conditionals than the rest of the packets put together, because every release
/// from 1.16 onwards moved a field into or out of it.
pub struct JoinGame<'a> {
    /// Java draws this from `Random`. It arrives as a field so the packet is reproducible.
    pub entity_id: i32,
    pub hardcore: bool,
    /// Vanilla game mode number; `3` is spectator, which is what holds a player still.
    pub game_mode: i32,
    /// Game mode the player left, or `-1` for none. Written as a signed byte.
    pub previous_game_mode: i32,
    pub dimension: &'a VersionedDimension,
    /// World generator name, read only before 1.16.
    pub level_type: &'a str,
    pub seed: i64,
    /// Read only before 1.14, when difficulty moved into its own packet.
    pub difficulty: i32,
    pub max_players: i32,
    pub view_distance: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    /// Whether death shows the respawn screen rather than respawning immediately.
    pub normal_respawn: bool,
    pub debug: bool,
    pub flat: bool,
    pub limited_crafting: bool,
    pub portal_cooldown: i32,
    pub sea_level: i32,
    pub online_mode: bool,
    pub secure_profile: bool,
}

impl JoinGame<'_> {
    fn dimension_for(&self, version: ProtocolVersion) -> Result<&Dimension, PacketEncodeError> {
        self.dimension
            .for_version(version)
            .ok_or_else(|| PacketEncodeError::DimensionUnresolved {
                key: self.dimension.key().to_owned(),
                version,
            })
    }

    fn write_identity<B>(&self, buffer: &mut B, version: ProtocolVersion)
    where
        B: BufMut + ?Sized,
    {
        buffer.put_i32(self.entity_id);

        if version >= ProtocolVersion::V1_16_2 {
            write_bool(buffer, self.hardcore);
        }
        if version < ProtocolVersion::V1_20_2 {
            // 1.7 has no spectator mode; the closest it can do is creative.
            let game_mode =
                if version <= ProtocolVersion::V1_7_6 && self.game_mode == GAME_MODE_SPECTATOR {
                    1
                } else {
                    self.game_mode
                };
            buffer.put_u8(game_mode as u8);
        }
    }

    /// Writes the world the player joins, in whichever of the three shapes this client
    /// reads: a bare number before 1.16, a key plus the whole registry codec from 1.16,
    /// and nothing at all from 1.20.2, where the codec moved to the configuration state.
    fn write_dimension<B>(
        &self,
        buffer: &mut B,
        version: ProtocolVersion,
        dimension: &Dimension,
    ) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        if version < ProtocolVersion::V1_16 {
            if version >= ProtocolVersion::V1_9 {
                buffer.put_i32(self.dimension.legacy_id());
            } else {
                buffer.put_u8(self.dimension.legacy_id() as u8);
            }
            return Ok(());
        }

        if version < ProtocolVersion::V1_20_2 {
            buffer.put_u8(self.previous_game_mode as u8);
        }

        // The list of worlds the client may be sent to, of which the limbo has one.
        buffer.write_var_int(1);
        buffer.write_string(self.dimension.key());

        if version < ProtocolVersion::V1_20_2 {
            write_compound(buffer, dimension.codec(), version)?;
        }

        if (ProtocolVersion::V1_16_2..ProtocolVersion::V1_19).contains(&version) {
            // These releases repeat the joined world's own entry inline.
            write_compound(buffer, dimension.element_codec(), version)?;
        } else if version < ProtocolVersion::V1_20_2 {
            buffer.write_string(self.dimension.key());
        }
        if version < ProtocolVersion::V1_20_2 {
            buffer.write_string(self.dimension.key());
        }

        Ok(())
    }

    fn write_world_settings<B>(&self, buffer: &mut B, version: ProtocolVersion)
    where
        B: BufMut + ?Sized,
    {
        if (ProtocolVersion::V1_15..ProtocolVersion::V1_20_2).contains(&version) {
            buffer.put_i64(self.seed);
        }
        if version < ProtocolVersion::V1_14 {
            buffer.put_u8(self.difficulty as u8);
        }
        if version >= ProtocolVersion::V1_16_2 {
            buffer.write_var_int(self.max_players);
        } else {
            buffer.put_u8(self.max_players as u8);
        }
        if version < ProtocolVersion::V1_16 {
            buffer.write_string(self.level_type);
        }
        if version >= ProtocolVersion::V1_14 {
            buffer.write_var_int(self.view_distance);
        }
        if version >= ProtocolVersion::V1_18 {
            buffer.write_var_int(self.simulation_distance);
        }
        if version >= ProtocolVersion::V1_8 {
            write_bool(buffer, self.reduced_debug_info);
        }
        if version >= ProtocolVersion::V1_15 {
            write_bool(buffer, self.normal_respawn);
        }
    }

    /// The spawn block 1.20.2 introduced, repeating what the pre-1.20.2 layout spread
    /// across the dimension fields.
    fn write_spawn_info<B>(&self, buffer: &mut B, version: ProtocolVersion, dimension: &Dimension)
    where
        B: BufMut + ?Sized,
    {
        if version < ProtocolVersion::V1_20_2 {
            return;
        }

        write_bool(buffer, self.limited_crafting);
        if version >= ProtocolVersion::V1_20_5 {
            buffer.write_var_int(dimension.id());
        } else {
            buffer.write_string(self.dimension.key());
        }
        buffer.write_string(self.dimension.key());
        buffer.put_i64(self.seed);
        buffer.put_u8(self.game_mode as u8);
        buffer.put_u8(self.previous_game_mode as u8);
    }

    fn write_trailing_flags<B>(&self, buffer: &mut B, version: ProtocolVersion)
    where
        B: BufMut + ?Sized,
    {
        if version >= ProtocolVersion::V1_16 {
            write_bool(buffer, self.debug);
            write_bool(buffer, self.flat);
        }
        if version >= ProtocolVersion::V1_19 {
            // No last death location.
            write_bool(buffer, false);
        }
        if version >= ProtocolVersion::V1_20 {
            buffer.write_var_int(self.portal_cooldown);
        }
        if version >= ProtocolVersion::V1_21_2 {
            buffer.write_var_int(self.sea_level);
        }
        if version >= ProtocolVersion::V26_2 {
            write_bool(buffer, self.online_mode);
        }
        if version >= ProtocolVersion::V1_20_5 {
            write_bool(buffer, self.secure_profile);
        }
    }
}

impl ClientboundPacket for JoinGame<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::JoinGame
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        let dimension = self.dimension_for(version)?;

        self.write_identity(buffer, version);
        self.write_dimension(buffer, version, dimension)?;
        self.write_world_settings(buffer, version);
        self.write_spawn_info(buffer, version, dimension);
        self.write_trailing_flags(buffer, version);

        Ok(())
    }
}
