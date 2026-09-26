use crate::version::ProtocolVersion;
use bytes::BufMut;
use valence_nbt::Compound;
use valence_nbt::binary::{to_binary, written_size};

use crate::buffer::nbt_encode_error::NbtEncodeError;

/// Bytes an encoder writes ahead of the compound body when the root name is empty: the
/// root tag id (`0x0A`) followed by the two-byte length of that empty name.
pub(crate) const NAMED_ROOT_HEADER_LEN: usize = 3;

fn encode_named_root(compound: &Compound) -> Result<Vec<u8>, NbtEncodeError> {
    let mut encoded = Vec::with_capacity(written_size(compound, ""));
    to_binary(compound, &mut encoded, "").map_err(|error| NbtEncodeError::Rejected {
        reason: error.to_string(),
    })?;

    Ok(encoded)
}

/// Appends `compound` to `buffer` in the shape `version` expects.
///
/// Ports Java's `ByteMessage.writeCompoundTag`. Up to 1.20, a compound on the wire is
/// the named form with an empty name; from 1.20.2 the client reads the nameless form,
/// which is the same bytes with the two name-length bytes removed.
pub fn write_compound<B>(
    buffer: &mut B,
    compound: &Compound,
    version: ProtocolVersion,
) -> Result<(), NbtEncodeError>
where
    B: BufMut + ?Sized,
{
    let named = encode_named_root(compound)?;

    if version < ProtocolVersion::V1_20_2 {
        buffer.put_slice(&named);
        return Ok(());
    }

    match (named.first(), named.get(NAMED_ROOT_HEADER_LEN..)) {
        (Some(&root_tag), Some(body)) => {
            buffer.put_u8(root_tag);
            buffer.put_slice(body);
            Ok(())
        }
        _ => Err(NbtEncodeError::MalformedRootHeader),
    }
}
