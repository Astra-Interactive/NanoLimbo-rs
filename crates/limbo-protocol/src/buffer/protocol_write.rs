use bytes::BufMut;
use uuid::Uuid;

/// Writes the Minecraft protocol's primitive types into a growable buffer.
///
/// Writing is infallible: the target grows on demand, and every value the server sends
/// is built from data it already validated. Fixed-width big-endian primitives come from
/// [`BufMut`] directly; this trait only adds the types the protocol defines itself.
pub trait ProtocolWrite: BufMut {
    /// Writes a variable-length signed 32-bit integer, little-endian in 7-bit groups.
    ///
    /// A negative value always occupies the full five bytes, because it is encoded as
    /// its two's-complement bit pattern rather than sign-extended.
    fn write_var_int(&mut self, value: i32);

    /// Writes a UTF-8 string prefixed with its length in bytes.
    fn write_string(&mut self, text: &str);

    fn write_uuid(&mut self, value: Uuid);

    /// Writes a length-prefixed array of 64-bit integers.
    ///
    /// An empty slice yields a bare zero length, which is also how the protocol spells
    /// an absent bit set.
    fn write_long_array(&mut self, values: &[i64]);
}

impl<B> ProtocolWrite for B
where
    B: BufMut + ?Sized,
{
    fn write_var_int(&mut self, value: i32) {
        let mut remaining = value as u32;

        loop {
            if remaining & !0x7F == 0 {
                self.put_u8(remaining as u8);
                return;
            }

            self.put_u8((remaining as u8 & 0x7F) | 0x80);
            remaining >>= 7;
        }
    }

    fn write_string(&mut self, text: &str) {
        let bytes = text.as_bytes();
        self.write_var_int(bytes.len() as i32);
        self.put_slice(bytes);
    }

    fn write_uuid(&mut self, value: Uuid) {
        let (most_significant, least_significant) = value.as_u64_pair();
        self.put_u64(most_significant);
        self.put_u64(least_significant);
    }

    fn write_long_array(&mut self, values: &[i64]) {
        self.write_var_int(values.len() as i32);
        for value in values {
            self.put_i64(*value);
        }
    }
}
