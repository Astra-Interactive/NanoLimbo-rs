use bytes::BytesMut;
use limbo_protocol::version::ProtocolVersion;

use crate::encoding::ClientboundPacket;
use crate::play::DeclareCommands;

fn encoded(commands: &[String]) -> Vec<u8> {
    let mut buffer = BytesMut::new();
    DeclareCommands { commands }
        .encode(&mut buffer, ProtocolVersion::V1_20_5)
        .expect("encoding cannot fail");

    buffer.to_vec()
}

#[test]
fn given_no_commands_when_encoded_then_only_a_childless_root_is_declared() {
    assert_eq!(encoded(&[]), vec![0x01, 0x00, 0x00, 0x00]);
}

#[test]
fn given_two_commands_when_encoded_then_the_root_points_at_both_literal_nodes() {
    let commands = ["one".to_owned(), "two".to_owned()];

    let bytes = encoded(&commands);

    assert_eq!(
        bytes.get(..5),
        Some([0x05, 0x00, 0x02, 0x01, 0x03].as_slice())
    );
}
