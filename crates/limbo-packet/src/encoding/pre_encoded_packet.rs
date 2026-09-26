use std::collections::{HashMap, HashSet};

use bytes::{Bytes, BytesMut};
use limbo_protocol::version::ProtocolVersion;

use crate::encoding::ClientboundPacket;
use crate::encoding::PacketEncodeError;

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
    distinct_payloads: usize,
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

        Ok(Self {
            payloads,
            distinct_payloads: distinct.len(),
        })
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
        self.distinct_payloads
    }
}
