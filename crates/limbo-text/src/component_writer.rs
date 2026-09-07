use bytes::BufMut;
use limbo_protocol::buffer::{NbtEncodeError, ProtocolWrite, write_compound};
use limbo_protocol::version::ProtocolVersion;

use crate::chat::Component;
use crate::component_json::to_json_string;
use crate::component_nbt::to_nbt_compound;
use crate::json_profile::JsonProfile;

/// The first version whose client reads chat components as NBT rather than as JSON text.
const FIRST_NBT_COMPONENT_VERSION: ProtocolVersion = ProtocolVersion::V1_20_3;

/// Appends a component to a packet in whichever form this client version reads.
///
/// Up to 1.20.2 that is a length-prefixed JSON string; from 1.20.3 it is an NBT compound.
/// The JSON profile still applies to the NBT form, since the two describe the same tree.
pub fn write_component<B>(
    buffer: &mut B,
    component: &Component,
    version: ProtocolVersion,
) -> Result<(), NbtEncodeError>
where
    B: BufMut + ?Sized,
{
    let profile = JsonProfile::for_version(version);

    if version < FIRST_NBT_COMPONENT_VERSION {
        buffer.write_string(&to_json_string(component, profile));
        return Ok(());
    }

    write_compound(buffer, &to_nbt_compound(component, profile), version)
}

/// Serializes a component as the JSON string a few packets carry on every version.
///
/// The login disconnect and status response packets never moved to NBT, so they need
/// this rather than [`write_component`].
pub fn to_json_for(component: &Component, version: ProtocolVersion) -> String {
    to_json_string(component, JsonProfile::for_version(version))
}
