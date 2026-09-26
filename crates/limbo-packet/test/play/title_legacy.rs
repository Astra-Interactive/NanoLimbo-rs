use bytes::BytesMut;
use limbo_protocol::version::ProtocolVersion;

use crate::encoding::ClientboundPacket;
use crate::play::{TitleLegacy, TitleTimes};

fn timing_action_id(version: ProtocolVersion) -> u8 {
    let mut buffer = BytesMut::new();
    TitleLegacy::SetTimesAndDisplay {
        times: TitleTimes {
            fade_in: 10,
            stay: 100,
            fade_out: 10,
        },
    }
    .encode(&mut buffer, version)
    .expect("encoding cannot fail");

    buffer
        .first()
        .copied()
        .expect("an action id is always written")
}

#[test]
fn given_the_release_that_added_the_action_bar_when_timings_are_sent_then_the_id_shifts() {
    assert_eq!(timing_action_id(ProtocolVersion::V1_10), 2);
    assert_eq!(timing_action_id(ProtocolVersion::V1_11), 3);
}
