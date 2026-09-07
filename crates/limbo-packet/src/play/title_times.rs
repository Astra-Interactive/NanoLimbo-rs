use bytes::BufMut;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// How long a title fades in, stays and fades out, in ticks.
///
/// Ticks rather than a [`Duration`](std::time::Duration): this is the wire boundary, and
/// the client's own tick rate is what gives the numbers meaning.
pub struct TitleTimes {
    pub fade_in: i32,
    pub stay: i32,
    pub fade_out: i32,
}

impl ClientboundPacket for TitleTimes {
    fn kind(&self) -> PacketKind {
        PacketKind::TitleTimes
    }

    fn encode<B>(&self, buffer: &mut B, _version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.put_i32(self.fade_in);
        buffer.put_i32(self.stay);
        buffer.put_i32(self.fade_out);

        Ok(())
    }
}
