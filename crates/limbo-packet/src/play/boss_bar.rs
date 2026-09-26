use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_text::chat::Component;
use limbo_text::write_component;
use uuid::Uuid;

use crate::encoding::ClientboundPacket;
use crate::encoding::PacketEncodeError;
use crate::play::BossBarColor;
use crate::play::BossBarDivision;

/// The only boss bar action the limbo sends: create the bar.
const ACTION_ADD: i32 = 0;

/// Puts a bar across the top of the screen, from 1.9.
pub struct BossBar<'a> {
    /// Identifies the bar for later updates. Java generates it with `UUID.randomUUID()`;
    /// here it is injected so the packet is reproducible.
    pub uuid: Uuid,
    pub text: &'a Component,
    /// Fraction of the bar that is filled, from 0.0 to 1.0.
    pub health: f32,
    pub color: BossBarColor,
    pub division: BossBarDivision,
    /// Darken sky, boss music and world fog, as a bit set the limbo leaves empty.
    pub flags: u8,
}

impl ClientboundPacket for BossBar<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::BossBar
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.write_uuid(self.uuid);
        buffer.write_var_int(ACTION_ADD);
        write_component(buffer, self.text, version)?;
        buffer.put_f32(self.health);
        buffer.write_var_int(self.color.index());
        buffer.write_var_int(self.division.index());
        buffer.put_u8(self.flags);

        Ok(())
    }
}
