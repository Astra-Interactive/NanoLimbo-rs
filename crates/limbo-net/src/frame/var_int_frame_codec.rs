use bytes::{Buf, BufMut, Bytes, BytesMut};
use limbo_protocol::buffer::ProtocolWrite;
use tokio_util::codec::{Decoder, Encoder};

use crate::frame::frame_error::FrameError;
use crate::frame::length_prefix::{LengthPrefix, MAX_FRAME_LENGTH, MAX_PREFIX_BYTES};

/// Byte a proxy or a legacy client may pad a stream with between frames.
const PADDING_BYTE: u8 = 0x00;

/// Splits a byte stream into `varint(length) || payload` frames and writes them back.
///
/// The port of the Java implementation's `VarIntFrameDecoder` and `VarIntLengthEncoder`,
/// including two behaviours that look accidental but are not:
///
/// - runs of `0x00` before a frame are skipped, because some proxies pad with them;
/// - a length of zero yields no frame at all, so an empty payload cannot round trip.
///
/// A negative length is unrepresentable here: 21 bits are read into an unsigned value,
/// so the Java `length < 0` guard has no counterpart.
#[derive(Debug, Clone, Copy)]
pub struct VarIntFrameCodec;

impl Decoder for VarIntFrameCodec {
    type Item = Bytes;
    type Error = FrameError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            let padding = src.iter().take_while(|byte| **byte == PADDING_BYTE).count();
            src.advance(padding);

            let Some(prefix) = LengthPrefix::read(src.as_ref())? else {
                return Ok(None);
            };

            if prefix.payload_length == 0 {
                // Only an over-long encoding such as `0x80 0x00` reaches this, a bare
                // `0x00` having been skipped as padding. Netty dropped the prefix and
                // waited for the next read; consuming it and looping instead keeps a
                // frame that arrived in the same segment from stalling until the client
                // happens to send more.
                src.advance(prefix.prefix_length);
                continue;
            }

            let frame_length = prefix.prefix_length + prefix.payload_length;
            if src.len() < frame_length {
                src.reserve(frame_length - src.len());
                return Ok(None);
            }

            src.advance(prefix.prefix_length);
            return Ok(Some(src.split_to(prefix.payload_length).freeze()));
        }
    }
}

impl Encoder<Bytes> for VarIntFrameCodec {
    type Error = FrameError;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        if item.len() > MAX_FRAME_LENGTH {
            return Err(FrameError::FrameTooLarge {
                length: item.len(),
                limit: MAX_FRAME_LENGTH,
            });
        }

        // The guard above bounds the length far inside `i32`.
        let length = item.len() as i32;

        dst.reserve(item.len() + MAX_PREFIX_BYTES);
        dst.write_var_int(length);
        dst.put_slice(&item);
        Ok(())
    }
}
