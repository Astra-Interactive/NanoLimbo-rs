use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_world::TagRegistry;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// Tells the client which registry entries belong to which tag, from 1.20.5.
///
/// The largest packet the limbo sends by far: several thousand ids across a dozen
/// registries, which is why it is pre-encoded once per version rather than built per
/// connection.
pub struct UpdateTags<'a> {
    pub registries: &'a [TagRegistry],
}

impl ClientboundPacket for UpdateTags<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::UpdateTags
    }

    fn encode<B>(&self, buffer: &mut B, _version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.write_var_int(self.registries.len() as i32);

        for registry in self.registries {
            buffer.write_string(registry.key());
            buffer.write_var_int(registry.tags().len() as i32);

            for tag in registry.tags() {
                buffer.write_string(tag.key());
                buffer.write_var_int(tag.entry_ids().len() as i32);

                for entry_id in tag.entry_ids() {
                    buffer.write_var_int(*entry_id);
                }
            }
        }

        Ok(())
    }
}
