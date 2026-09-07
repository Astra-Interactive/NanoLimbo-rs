use bytes::Buf;
use uuid::Uuid;

use crate::buffer::packet_decode_error::PacketDecodeError;

/// Widest varint the protocol permits, in bytes.
const MAX_VAR_INT_BYTES: usize = 5;

/// Minecraft caps string payloads at three bytes per permitted character, which is the
/// longest UTF-8 encoding of a character in the Basic Multilingual Plane.
const MAX_BYTES_PER_CHAR: usize = 3;

fn ensure_remaining<B>(buffer: &B, needed: usize) -> Result<(), PacketDecodeError>
where
    B: Buf + ?Sized,
{
    let available = buffer.remaining();
    if available < needed {
        return Err(PacketDecodeError::UnexpectedEndOfInput { needed, available });
    }
    Ok(())
}

/// Reads the Minecraft protocol's primitive types out of a buffer.
///
/// Every method validates before it consumes, so a truncated or hostile frame produces
/// an error instead of a panic. Prefer these over the raw [`Buf`] getters, which panic
/// when the buffer runs out.
pub trait ProtocolRead: Buf {
    fn read_u8(&mut self) -> Result<u8, PacketDecodeError>;

    /// Reads a protocol boolean: any non-zero byte is true, matching the Java client.
    fn read_bool(&mut self) -> Result<bool, PacketDecodeError>;

    fn read_u16(&mut self) -> Result<u16, PacketDecodeError>;

    fn read_i32(&mut self) -> Result<i32, PacketDecodeError>;

    fn read_i64(&mut self) -> Result<i64, PacketDecodeError>;

    /// Reads a variable-length signed 32-bit integer, little-endian in 7-bit groups.
    fn read_var_int(&mut self) -> Result<i32, PacketDecodeError>;

    /// Reads a length-prefixed UTF-8 string.
    ///
    /// `max_chars` bounds the character count; the byte length is bounded at three
    /// times that and is checked before anything is allocated, so a client cannot ask
    /// the server to reserve an arbitrary buffer.
    fn read_string(&mut self, max_chars: usize) -> Result<String, PacketDecodeError>;

    fn read_uuid(&mut self) -> Result<Uuid, PacketDecodeError>;

    /// Reads a length-prefixed byte array, rejecting anything longer than `limit`.
    fn read_byte_array(&mut self, limit: usize) -> Result<Vec<u8>, PacketDecodeError>;
}

impl<B> ProtocolRead for B
where
    B: Buf + ?Sized,
{
    fn read_u8(&mut self) -> Result<u8, PacketDecodeError> {
        ensure_remaining(self, size_of::<u8>())?;
        Ok(self.get_u8())
    }

    fn read_bool(&mut self) -> Result<bool, PacketDecodeError> {
        self.read_u8().map(|byte| byte != 0)
    }

    fn read_u16(&mut self) -> Result<u16, PacketDecodeError> {
        ensure_remaining(self, size_of::<u16>())?;
        Ok(self.get_u16())
    }

    fn read_i32(&mut self) -> Result<i32, PacketDecodeError> {
        ensure_remaining(self, size_of::<i32>())?;
        Ok(self.get_i32())
    }

    fn read_i64(&mut self) -> Result<i64, PacketDecodeError> {
        ensure_remaining(self, size_of::<i64>())?;
        Ok(self.get_i64())
    }

    fn read_var_int(&mut self) -> Result<i32, PacketDecodeError> {
        let mut value: i32 = 0;

        for group in 0..MAX_VAR_INT_BYTES {
            let byte = self.read_u8()?;
            value |= i32::from(byte & 0x7F) << (group * 7);

            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }

        Err(PacketDecodeError::VarIntTooWide)
    }

    fn read_string(&mut self, max_chars: usize) -> Result<String, PacketDecodeError> {
        let declared = self.read_var_int()?;
        let byte_count = usize::try_from(declared)
            .map_err(|_| PacketDecodeError::NegativeLength { length: declared })?;

        let max_bytes = max_chars.saturating_mul(MAX_BYTES_PER_CHAR);
        if byte_count > max_bytes {
            return Err(PacketDecodeError::StringTooManyBytes {
                actual: byte_count,
                limit: max_bytes,
            });
        }
        ensure_remaining(self, byte_count)?;

        let mut bytes = vec![0_u8; byte_count];
        self.copy_to_slice(&mut bytes);
        let text = String::from_utf8(bytes).map_err(|_| PacketDecodeError::MalformedUtf8)?;

        // The client measures this limit in UTF-16 code units, so a character outside
        // the Basic Multilingual Plane counts twice. Counting `chars` instead would
        // accept strings the client rejects.
        let char_count = text.encode_utf16().count();
        if char_count > max_chars {
            return Err(PacketDecodeError::StringTooManyChars {
                actual: char_count,
                limit: max_chars,
            });
        }

        Ok(text)
    }

    fn read_uuid(&mut self) -> Result<Uuid, PacketDecodeError> {
        let most_significant = self.read_i64()?;
        let least_significant = self.read_i64()?;
        Ok(Uuid::from_u64_pair(
            most_significant as u64,
            least_significant as u64,
        ))
    }

    fn read_byte_array(&mut self, limit: usize) -> Result<Vec<u8>, PacketDecodeError> {
        let declared = self.read_var_int()?;
        let byte_count = usize::try_from(declared)
            .map_err(|_| PacketDecodeError::NegativeLength { length: declared })?;

        if byte_count > limit {
            return Err(PacketDecodeError::ByteArrayTooLong {
                actual: byte_count,
                limit,
            });
        }
        ensure_remaining(self, byte_count)?;

        let mut bytes = vec![0_u8; byte_count];
        self.copy_to_slice(&mut bytes);
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string_frame(declared_length: i32, payload: &[u8]) -> Vec<u8> {
        let mut frame = Vec::new();
        let mut remaining = declared_length as u32;
        loop {
            if remaining & !0x7F == 0 {
                frame.push(remaining as u8);
                break;
            }
            frame.push((remaining as u8 & 0x7F) | 0x80);
            remaining >>= 7;
        }
        frame.extend_from_slice(payload);
        frame
    }

    #[test]
    fn given_varint_boundary_encodings_when_read_then_the_original_values_come_back() {
        let cases = [
            (vec![0x00], 0_i32),
            (vec![0x7F], 127),
            (vec![0x80, 0x01], 128),
            (vec![0xFF, 0x7F], 16_383),
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07], i32::MAX),
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F], -1),
            (vec![0x80, 0x80, 0x80, 0x80, 0x08], i32::MIN),
        ];

        for (encoded, expected) in cases {
            let mut input = encoded.as_slice();

            assert_eq!(input.read_var_int(), Ok(expected), "{encoded:02X?}");
        }
    }

    #[test]
    fn given_a_varint_needing_a_sixth_byte_when_read_then_it_is_rejected_as_too_wide() {
        let mut input: &[u8] = &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];

        assert_eq!(input.read_var_int(), Err(PacketDecodeError::VarIntTooWide));
    }

    #[test]
    fn given_a_varint_cut_off_mid_sequence_when_read_then_the_shortfall_is_reported() {
        let mut input: &[u8] = &[0x80, 0x80];

        assert_eq!(
            input.read_var_int(),
            Err(PacketDecodeError::UnexpectedEndOfInput {
                needed: 1,
                available: 0
            })
        );
    }

    #[test]
    fn given_a_string_length_beyond_the_byte_limit_when_read_then_nothing_is_allocated() {
        // A one-byte buffer claiming a megabyte. The limit must be enforced from the
        // declared length alone, before the server reserves anything.
        let mut input: &[u8] = &[0x80, 0x89, 0x7A, b'x'];

        assert_eq!(
            input.read_string(16),
            Err(PacketDecodeError::StringTooManyBytes {
                actual: 2_000_000,
                limit: 48
            })
        );
    }

    #[test]
    fn given_characters_outside_the_basic_plane_when_read_then_each_counts_as_two() {
        let payload = "𝄞𝄞𝄞".as_bytes();
        assert_eq!(payload.len(), 12);

        let frame = string_frame(12, payload);

        assert_eq!(
            frame.as_slice().read_string(5),
            Err(PacketDecodeError::StringTooManyChars {
                actual: 6,
                limit: 5
            })
        );
        assert_eq!(frame.as_slice().read_string(6), Ok("𝄞𝄞𝄞".to_owned()));
    }

    #[test]
    fn given_a_negative_string_length_when_read_then_it_is_rejected() {
        let mut input: &[u8] = &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F];

        assert_eq!(
            input.read_string(16),
            Err(PacketDecodeError::NegativeLength { length: -1 })
        );
    }

    #[test]
    fn given_a_string_payload_that_is_not_utf8_when_read_then_it_is_rejected() {
        let frame = string_frame(2, &[0xC3, 0x28]);

        assert_eq!(
            frame.as_slice().read_string(16),
            Err(PacketDecodeError::MalformedUtf8)
        );
    }

    #[test]
    fn given_a_string_shorter_than_declared_when_read_then_the_shortfall_is_reported() {
        let frame = string_frame(10, b"abc");

        assert_eq!(
            frame.as_slice().read_string(16),
            Err(PacketDecodeError::UnexpectedEndOfInput {
                needed: 10,
                available: 3
            })
        );
    }

    #[test]
    fn given_a_byte_array_longer_than_the_limit_when_read_then_it_is_rejected() {
        let frame = string_frame(600, &[0_u8; 600]);

        assert_eq!(
            frame.as_slice().read_byte_array(512),
            Err(PacketDecodeError::ByteArrayTooLong {
                actual: 600,
                limit: 512
            })
        );
        assert_eq!(
            frame
                .as_slice()
                .read_byte_array(1024)
                .map(|bytes| bytes.len()),
            Ok(600)
        );
    }

    #[test]
    fn given_an_empty_buffer_when_fixed_width_values_are_read_then_errors_replace_panics() {
        let empty: &[u8] = &[];
        let one_byte: &[u8] = &[0x00];
        let three_bytes: &[u8] = &[0x00, 0x01, 0x02];
        let seven_bytes: &[u8] = &[0x00_u8; 7];
        let fifteen_bytes: &[u8] = &[0x00_u8; 15];

        assert!({ empty }.read_u8().is_err());
        assert!({ one_byte }.read_u16().is_err());
        assert!({ three_bytes }.read_i32().is_err());
        assert!({ seven_bytes }.read_i64().is_err());
        assert!({ fifteen_bytes }.read_uuid().is_err());
    }
}
