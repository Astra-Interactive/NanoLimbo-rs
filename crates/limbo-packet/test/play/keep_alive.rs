use bytes::BytesMut;
use limbo_protocol::version::ProtocolVersion;

use crate::encoding::ClientboundPacket;
use crate::play::KeepAlive;

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
