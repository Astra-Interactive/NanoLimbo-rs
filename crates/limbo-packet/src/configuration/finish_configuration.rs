use bytes::BufMut;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// Ends the configuration state and moves the connection into play.
///
/// Carries nothing: the id is the whole message.
pub struct FinishConfiguration;

impl ClientboundPacket for FinishConfiguration {
    fn kind(&self) -> PacketKind {
        PacketKind::FinishConfiguration
    }

    fn encode<B>(&self, _buffer: &mut B, _version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        Ok(())
    }
}
