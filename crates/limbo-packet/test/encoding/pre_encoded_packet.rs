use bytes::{BufMut, Bytes};
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::encoding::{ClientboundPacket, PacketEncodeError, PreEncodedPacket};

/// Writes one byte that changes at 1.16, so payloads coalesce into exactly two groups.
struct VersionSplitPacket {
    marker: u8,
}

impl ClientboundPacket for VersionSplitPacket {
    fn kind(&self) -> PacketKind {
        PacketKind::KeepAlive
    }

    fn encode<B>(&self, buffer: &mut B, _version: ProtocolVersion) -> Result<(), PacketEncodeError>
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
    let packet =
        PreEncodedPacket::for_versions([ProtocolVersion::V1_20_5], split_at_1_16).expect("encode");

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
