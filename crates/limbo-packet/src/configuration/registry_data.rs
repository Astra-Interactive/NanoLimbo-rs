use bytes::BufMut;
use limbo_protocol::buffer::ProtocolWrite;
use limbo_protocol::packet::PacketKind;
use limbo_protocol::version::ProtocolVersion;
use limbo_world::write_compound;
use valence_nbt::{Compound, List, Value};

use crate::configuration::RegistryEntry;
use crate::encoding::ClientboundPacket;
use crate::encoding::PacketEncodeError;

fn registry_values(registry: &Value) -> &[Compound] {
    match registry {
        Value::Compound(compound) => match compound.get("value") {
            Some(Value::List(List::Compound(values))) => values,
            _ => &[],
        },
        _ => &[],
    }
}

fn entry_name(entry: &Compound) -> Option<&str> {
    match entry.get("name") {
        Some(Value::String(name)) => Some(name),
        _ => None,
    }
}

fn entry_element(entry: &Compound) -> Option<&Compound> {
    match entry.get("element") {
        Some(Value::Compound(element)) => Some(element),
        _ => None,
    }
}

/// The registries the client is missing, in whichever of the two shapes it reads.
///
/// One packet, two payloads: 1.20.2 hands over the entire dimension codec at once, and
/// 1.20.5 replaced that with one packet per registry so a client can skip the ones its
/// own data pack already has. The Java implementation expresses the same split with an
/// injected writer; an enum says it in the type.
pub enum RegistryData<'a> {
    /// 1.20.2 and 1.20.3: the whole codec as a single compound.
    WholeCodec { codec: &'a Compound },
    /// From 1.20.5: one registry, named, with its entries.
    Registry {
        key: &'a str,
        entries: Vec<RegistryEntry<'a>>,
    },
}

impl<'a> RegistryData<'a> {
    /// Splits a dimension codec into the per-registry packets a 1.20.5 client expects,
    /// in the order the codec declares them.
    ///
    /// An entry with no `element` compound is announced by name only, which tells the
    /// client to keep whatever its own data pack defines.
    pub fn split_codec(codec: &'a Compound) -> Vec<Self> {
        codec
            .iter()
            .map(|(key, registry)| Self::Registry {
                key,
                entries: registry_values(registry)
                    .iter()
                    .filter_map(|entry| {
                        Some(RegistryEntry {
                            name: entry_name(entry)?,
                            element: entry_element(entry),
                        })
                    })
                    .collect(),
            })
            .collect()
    }

    fn write_registry<B>(
        buffer: &mut B,
        version: ProtocolVersion,
        key: &str,
        entries: &[RegistryEntry<'_>],
    ) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        buffer.write_string(key);
        buffer.write_var_int(entries.len() as i32);

        for entry in entries {
            buffer.write_string(entry.name);
            match entry.element {
                Some(element) => {
                    buffer.put_u8(1);
                    write_compound(buffer, element, version)?;
                }
                None => buffer.put_u8(0),
            }
        }

        Ok(())
    }
}

impl ClientboundPacket for RegistryData<'_> {
    fn kind(&self) -> PacketKind {
        PacketKind::RegistryData
    }

    fn encode<B>(&self, buffer: &mut B, version: ProtocolVersion) -> Result<(), PacketEncodeError>
    where
        B: BufMut + ?Sized,
    {
        match self {
            Self::WholeCodec { codec } => write_compound(buffer, codec, version)?,
            Self::Registry { key, entries } => {
                Self::write_registry(buffer, version, key, entries)?;
            }
        }

        Ok(())
    }
}
