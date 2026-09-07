use bytes::BufMut;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

const FLAG_INVINCIBLE: u8 = 0x01;
const FLAG_FLYING: u8 = 0x02;
const FLAG_CAN_FLY: u8 = 0x04;
const FLAG_CREATIVE: u8 = 0x08;

/// What the player may do: fly, take damage, and how fast the world moves past them.
///
/// Identical on every version, which is why the limbo can hold a player in the air on a
/// 1.7 client and a 26.2 one with the same bytes.
pub struct PlayerAbilities {
    pub invincible: bool,
    pub can_fly: bool,
    pub flying: bool,
    pub creative: bool,
    pub flying_speed: f32,
    /// Field of view modifier, applied on top of the client's own setting.
    pub field_of_view: f32,
}

impl PlayerAbilities {
    fn flags(&self) -> u8 {
        let mut flags = 0;

        if self.invincible {
            flags |= FLAG_INVINCIBLE;
        }
        if self.can_fly {
            flags |= FLAG_CAN_FLY;
        }
        if self.flying {
            flags |= FLAG_FLYING;
        }
        if self.creative {
            flags |= FLAG_CREATIVE;
        }

        flags
    }
}

impl ClientboundPacket for PlayerAbilities {
    fn kind(&self) -> PacketKind {
        PacketKind::PlayerAbilities
    }

    fn encode<B>(&self, buffer: &mut B, _version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.put_u8(self.flags());
        buffer.put_f32(self.flying_speed);
        buffer.put_f32(self.field_of_view);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_every_ability_when_flags_are_built_then_each_claims_its_own_bit() {
        let abilities = PlayerAbilities {
            invincible: true,
            can_fly: true,
            flying: true,
            creative: true,
            flying_speed: 0.0,
            field_of_view: 0.1,
        };

        assert_eq!(abilities.flags(), 0x0F);
    }

    #[test]
    fn given_only_flying_when_flags_are_built_then_the_can_fly_bit_stays_clear() {
        let abilities = PlayerAbilities {
            invincible: false,
            can_fly: false,
            flying: true,
            creative: false,
            flying_speed: 0.0,
            field_of_view: 0.1,
        };

        assert_eq!(abilities.flags(), FLAG_FLYING);
    }
}
