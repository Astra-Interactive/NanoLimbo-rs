use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// A message on a named side channel, used here to tell the client the server's brand.
///
/// Sent in both the play and configuration states; which id it travels under is the
/// caller's concern.
pub struct PluginMessage<'a> {
    pub channel: &'a str,
    /// Written raw: the packet frame delimits the payload, so it carries no length.
    pub data: &'a [u8],
}

impl ClientboundPacket for PluginMessage<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::PluginMessage
    }

    fn encode<B>(&self, buffer: &mut B, _version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.write_string(self.channel);
        buffer.put_slice(self.data);

        Ok(())
    }
}
