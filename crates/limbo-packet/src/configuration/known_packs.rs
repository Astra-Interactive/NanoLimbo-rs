use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::configuration::known_pack::KnownPack;
use crate::packet_encode_error::PacketEncodeError;

/// Offers the client the data packs the server has, from 1.20.5.
///
/// The limbo claims only vanilla's own pack, named after the release the client is
/// running, which is what lets it skip sending most of the registries.
pub struct KnownPacks<'a> {
    pub packs: &'a [KnownPack<'a>],
}

impl ClientboundPacket for KnownPacks<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::KnownPacks
    }

    fn encode<B>(&self, buffer: &mut B, _version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.write_var_int(self.packs.len() as i32);
        for pack in self.packs {
            buffer.write_string(pack.namespace);
            buffer.write_string(pack.id);
            buffer.write_string(pack.version);
        }

        Ok(())
    }
}
