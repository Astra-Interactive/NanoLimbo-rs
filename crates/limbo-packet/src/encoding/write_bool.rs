use bytes::BufMut;

/// Writes a protocol boolean: one byte, `0x01` for true and `0x00` for false.
///
/// [`BufMut`] has no boolean primitive, and spelling the conversion out at every call
/// site would bury the version conditionals these packets are really made of.
pub(crate) fn write_bool<B>(buffer: &mut B, value: bool)
where
    B: BufMut + ?Sized,
{
    buffer.put_u8(u8::from(value));
}
