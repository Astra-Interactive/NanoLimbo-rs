use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// Asks a modded client, or a proxy speaking for one, to answer on a private channel.
///
/// The limbo uses it for Velocity's modern forwarding handshake.
pub struct LoginPluginRequest<'a> {
    /// Correlates the response; the client echoes it back.
    pub message_id: i32,
    pub channel: &'a str,
    /// Written raw, with no length prefix: the packet's own frame delimits it.
    pub data: &'a [u8],
}

impl ClientboundPacket for LoginPluginRequest<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::LoginPluginRequest
    }

    fn encode<B>(&self, buffer: &mut B, _version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.write_var_int(self.message_id);
        buffer.write_string(self.channel);
        buffer.put_slice(self.data);

        Ok(())
    }
}
