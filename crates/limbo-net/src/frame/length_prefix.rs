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
