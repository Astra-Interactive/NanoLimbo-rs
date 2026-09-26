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
