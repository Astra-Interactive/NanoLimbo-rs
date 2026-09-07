use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::clientbound_packet::ClientboundPacket;
use crate::packet_encode_error::PacketEncodeError;
use crate::play::title_set_subtitle::TitleSetSubTitle;
use crate::play::title_set_title::TitleSetTitle;
use crate::play::title_times::TitleTimes;

/// One title packet carrying an action id, as clients before 1.17 read it.
///
/// 1.17 split this into three packets whose payloads are unchanged, so each variant here
/// delegates to the modern packet rather than repeating it.
pub enum TitleLegacy<'a> {
    SetTitle {
        title: TitleSetTitle<'a>,
    },
    SetSubtitle {
        subtitle: TitleSetSubTitle<'a>,
    },
    /// Sets the timings and displays what was set, which is why it comes last.
    SetTimesAndDisplay {
        times: TitleTimes,
    },
}

impl TitleLegacy<'_> {
    /// The action number this client reads.
    ///
    /// 1.11 inserted "action bar" into the middle of the action list, pushing the timing
    /// action from 2 up to 3.
    fn action_id(&self, version: ProtocolVersion) -> i32 {
        match self {
            Self::SetTitle { .. } => 0,
            Self::SetSubtitle { .. } => 1,
            Self::SetTimesAndDisplay { .. } => {
                if version < ProtocolVersion::V1_11 {
                    2
                } else {
                    3
                }
            }
        }
    }
}

impl ClientboundPacket for TitleLegacy<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::TitleLegacy
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.write_var_int(self.action_id(version));

        match self {
            Self::SetTitle { title } => title.encode(buffer, version),
            Self::SetSubtitle { subtitle } => subtitle.encode(buffer, version),
            Self::SetTimesAndDisplay { times } => times.encode(buffer, version),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;

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
}
