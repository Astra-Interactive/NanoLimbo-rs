use bytes::BufMut;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::packet_encode_error::PacketEncodeError;

/// A packet the server sends, able to write itself for any protocol version.
///
/// The method is generic rather than taking `&mut dyn BufMut`, so encoding into a
/// `BytesMut` costs no virtual calls; the price is that the trait is not object safe,
/// which is no loss here because packets are pre-encoded per version at startup.
pub trait ClientboundPacket {
    /// Which packet this is, so the caller can resolve the id it travels under for the
    /// connection's state and version.
    fn kind(&self) -> PacketKind;

    /// Appends the payload — the packet without its id prefix — in the shape `version`
    /// reads.
    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized;
}
