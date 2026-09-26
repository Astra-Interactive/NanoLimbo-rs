use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;

use crate::encoding::ClientboundPacket;
use crate::encoding::PacketEncodeError;
use crate::play::TitleSetSubTitle;
use crate::play::TitleSetTitle;
use crate::play::TitleTimes;

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
