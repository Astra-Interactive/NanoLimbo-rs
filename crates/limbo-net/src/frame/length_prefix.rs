use crate::frame::frame_error::FrameError;

/// Widest length prefix the protocol permits. Three groups of seven bits, which is what
/// both Netty's `Varint21FrameDecoder` and the vanilla client accept.
pub(crate) const MAX_PREFIX_BYTES: usize = 3;

/// The largest frame a 21 bit prefix can describe, just under two mebibytes.
pub(crate) const MAX_FRAME_LENGTH: usize = (1 << (MAX_PREFIX_BYTES * 7)) - 1;

/// A frame's declared payload length together with the room its prefix took on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LengthPrefix {
    pub(crate) payload_length: usize,
    pub(crate) prefix_length: usize,
}

impl LengthPrefix {
    /// Reads a prefix from the front of `bytes` without consuming anything.
    ///
    /// `Ok(None)` means the prefix is still incomplete and more bytes may finish it, so
    /// the caller must leave the buffer untouched and wait. A fourth continuation byte
    /// cannot describe a legal frame, so it is rejected rather than truncated.
    pub(crate) fn read(bytes: &[u8]) -> Result<Option<Self>, FrameError> {
        let mut payload_length: u32 = 0;

        for (index, byte) in bytes.iter().take(MAX_PREFIX_BYTES).enumerate() {
            payload_length |= u32::from(byte & 0x7F) << (index * 7);

            if byte & 0x80 == 0 {
                return Ok(Some(Self {
                    payload_length: payload_length as usize,
                    prefix_length: index + 1,
                }));
            }
        }

        if bytes.len() < MAX_PREFIX_BYTES {
            return Ok(None);
        }

        Err(FrameError::LengthPrefixTooWide)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_single_byte_prefix_when_read_then_reports_one_consumed_byte() {
        let prefix = LengthPrefix::read(&[0x7F, 0xAA]).unwrap().unwrap();

        assert_eq!(
            prefix,
            LengthPrefix {
                payload_length: 127,
                prefix_length: 1,
            }
        );
    }

    #[test]
    fn given_widest_legal_prefix_when_read_then_yields_the_21_bit_maximum() {
        let prefix = LengthPrefix::read(&[0xFF, 0xFF, 0x7F]).unwrap().unwrap();

        assert_eq!(
            prefix,
            LengthPrefix {
                payload_length: MAX_FRAME_LENGTH,
                prefix_length: 3,
            }
        );
    }

    #[test]
    fn given_prefix_cut_short_by_the_stream_when_read_then_reports_incomplete() {
        assert_eq!(LengthPrefix::read(&[]).unwrap(), None);
        assert_eq!(LengthPrefix::read(&[0x80]).unwrap(), None);
        assert_eq!(LengthPrefix::read(&[0x80, 0x80]).unwrap(), None);
    }

    #[test]
    fn given_fourth_continuation_byte_when_read_then_rejects_the_prefix() {
        let error = LengthPrefix::read(&[0x80, 0x80, 0x80, 0x01]).unwrap_err();

        assert!(matches!(error, FrameError::LengthPrefixTooWide));
    }

    #[test]
    fn given_three_continuation_bytes_and_nothing_else_when_read_then_still_rejects() {
        // The Java slow path silently truncated this to a 21 bit value; a prefix that
        // long cannot describe a legal frame however many bytes follow it.
        let error = LengthPrefix::read(&[0x80, 0x80, 0x80]).unwrap_err();

        assert!(matches!(error, FrameError::LengthPrefixTooWide));
    }
}
