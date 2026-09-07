use std::collections::{HashMap, HashSet};

use bytes::{Bytes, BytesMut};
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;

/// A packet encoded once per protocol version, ready to be written to any client.
///
/// Ports Java's `PacketSnapshot`. Most packets produce the same bytes for long runs of
/// versions, so identical payloads are stored once and shared: a [`Bytes`] handed to a
/// connection is a reference count bump, not a copy.
///
/// Java deduplicated by a 32-bit hash of the encoded buffer and treated equal hashes as
/// equal payloads, so a collision would silently send one version another version's
/// bytes. This compares content, which cannot be wrong.
pub struct PreEncodedPacket {
    payloads: HashMap<ProtocolVersion, Bytes>,
    distinct: HashSet<Bytes>,
}

impl PreEncodedPacket {
    /// Encodes `build_packet(version)` for each version in `versions`.
    ///
    /// The packet is rebuilt per version rather than encoded once, because some packets
    /// carry version-dependent content — the known packs handshake names the client's
    /// own release — and because a borrowed packet is cheap to rebuild.
    pub fn for_versions<P, F, V>(versions: V, build_packet: F) -> Result<Self, PacketEncodeError>
    where
        P: ClientboundPacket,
        F: Fn(ProtocolVersion) -> Result<P, PacketEncodeError>,
        V: IntoIterator<Item = ProtocolVersion>,
    {
        let mut distinct: HashSet<Bytes> = HashSet::new();
        let mut payloads = HashMap::new();

        for version in versions {
            let mut buffer = BytesMut::new();
            build_packet(version)?.encode(&mut buffer, version)?;
            let encoded = buffer.freeze();

            let shared = match distinct.get(&encoded) {
                Some(existing) => existing.clone(),
                None => {
                    distinct.insert(encoded.clone());
                    encoded
                }
            };
            payloads.insert(version, shared);
        }

        Ok(Self { payloads, distinct })
    }

    /// Encodes for every version the server speaks.
    pub fn for_all_versions<P, F>(build_packet: F) -> Result<Self, PacketEncodeError>
    where
        P: ClientboundPacket,
        F: Fn(ProtocolVersion) -> Result<P, PacketEncodeError>,
    {
        Self::for_versions(ProtocolVersion::all(), build_packet)
    }

    /// The payload for `version`, or `None` when this packet is not sent to it.
    pub fn payload(&self, version: ProtocolVersion) -> Option<&Bytes> {
        self.payloads.get(&version)
    }

    /// How many distinct payloads back this packet, as opposed to how many versions it
    /// covers. Reported at startup, where it says how much the deduplication saved.
    pub fn distinct_payload_count(&self) -> usize {
        self.distinct.len()
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;
    use limbo_protocol::packet::PacketKind;

    use super::*;

    /// Writes one byte that changes at 1.16, so payloads coalesce into exactly two groups.
    struct VersionSplitPacket {
        marker: u8,
    }

    impl ClientboundPacket for VersionSplitPacket {
        fn kind(&self) -> PacketKind {
            PacketKind::KeepAlive
        }

        fn encode<B>(
            &self,
            buffer: &mut B,
            _version: ProtocolVersion,
        ) -> Result<(), PacketEncodeError>
        where
            B: BufMut + ?Sized,
        {
            buffer.put_u8(self.marker);
            Ok(())
        }
    }

    fn split_at_1_16(version: ProtocolVersion) -> Result<VersionSplitPacket, PacketEncodeError> {
        Ok(VersionSplitPacket {
            marker: u8::from(version >= ProtocolVersion::V1_16),
        })
    }

    #[test]
    fn given_two_distinct_payloads_when_pre_encoded_then_equal_bytes_are_stored_once() {
        let packet = PreEncodedPacket::for_all_versions(split_at_1_16).expect("encode");

        assert_eq!(packet.distinct_payload_count(), 2);
        assert_eq!(
            packet.payload(ProtocolVersion::V1_7_2).map(Bytes::as_ref),
            Some([0x00].as_slice())
        );
        assert_eq!(
            packet.payload(ProtocolVersion::V26_2).map(Bytes::as_ref),
            Some([0x01].as_slice())
        );
    }

    #[test]
    fn given_a_restricted_version_set_when_pre_encoded_then_other_versions_have_no_payload() {
        let packet = PreEncodedPacket::for_versions([ProtocolVersion::V1_20_5], split_at_1_16)
            .expect("encode");

        assert!(packet.payload(ProtocolVersion::V1_20_5).is_some());
        assert!(packet.payload(ProtocolVersion::V1_20_3).is_none());
    }

    #[test]
    fn given_a_failing_build_when_pre_encoded_then_the_error_reaches_the_caller() {
        let failure = PreEncodedPacket::for_all_versions(|version| {
            Err::<VersionSplitPacket, _>(PacketEncodeError::DimensionUnresolved {
                key: "minecraft:the_end".to_owned(),
                version,
            })
        });

        assert!(failure.is_err());
    }
}
