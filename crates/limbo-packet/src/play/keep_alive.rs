use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// Proves the connection is alive; the client echoes the id back unchanged.
///
/// The id narrowed twice on its way through the versions, so a 1.7 client sees only the
/// low 32 bits of what a modern one sees.
pub struct KeepAlive {
    /// Java draws this from `ThreadLocalRandom` every five seconds. It arrives as a
    /// field so the connection's timer, not this crate, owns the randomness.
    pub id: i64,
}

impl ClientboundPacket for KeepAlive {
    fn kind(&self) -> PacketKind {
        PacketKind::KeepAlive
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        if version >= ProtocolVersion::V1_12_2 {
            buffer.put_i64(self.id);
        } else if version >= ProtocolVersion::V1_8 {
            buffer.write_var_int(self.id as i32);
        } else {
            buffer.put_i32(self.id as i32);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

    fn encoded(id: i64, version: ProtocolVersion) -> Vec<u8> {
        let mut buffer = BytesMut::new();
        KeepAlive { id }
            .encode(&mut buffer, version)
            .expect("encoding cannot fail");

        buffer.to_vec()
    }

    #[test]
    fn given_the_release_that_widened_the_id_when_encoded_then_the_width_changes_with_it() {
        assert_eq!(encoded(1, ProtocolVersion::V1_12_1), vec![0x01]);
        assert_eq!(
            encoded(1, ProtocolVersion::V1_12_2),
            vec![0, 0, 0, 0, 0, 0, 0, 1]
        );
    }

    #[test]
    fn given_an_id_wider_than_the_client_when_encoded_then_only_the_low_bits_travel() {
        assert_eq!(
            encoded(0x0000_0001_0000_0002, ProtocolVersion::V1_7_2),
            vec![0, 0, 0, 2]
        );
    }
}
