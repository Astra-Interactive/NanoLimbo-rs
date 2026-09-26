use std::sync::Arc;

use limbo_protocol::version::ProtocolVersion;
use valence_nbt::{Compound, List, Value};

use crate::codec::Dimension;
use crate::resource::ResourceLoadError;
use crate::resource::ResourceSelection;
use crate::resource::VersionMatch;

/// Which dimension codec each protocol version receives, transcribed rung by rung from
/// Java's `DimensionRegistry.getRegistryByVersion`.
///
/// The chain is deliberately irregular: most 1.21.x releases claim exactly one version,
/// while 1.17, 1.19.1, 1.20 and 1.16.2 are lower bounds that absorb the releases above
/// them. Reordering the rungs, or widening an equality into a bound, silently hands a
/// client the registry of a different release.
pub(crate) static CODEC_LADDER: &[ResourceSelection] = &[
    ResourceSelection::new(
        "codec_26_2.nbt",
        include_bytes!("../../resources/dimension/codec_26_2.nbt"),
        VersionMatch::AtLeast(ProtocolVersion::V26_2),
    ),
    ResourceSelection::new(
        "codec_26_1.nbt",
        include_bytes!("../../resources/dimension/codec_26_1.nbt"),
        VersionMatch::AtLeast(ProtocolVersion::V26_1),
    ),
    ResourceSelection::new(
        "codec_1_21_11.nbt",
        include_bytes!("../../resources/dimension/codec_1_21_11.nbt"),
        VersionMatch::AtLeast(ProtocolVersion::V1_21_11),
    ),
    ResourceSelection::new(
        "codec_1_21_9.nbt",
        include_bytes!("../../resources/dimension/codec_1_21_9.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_9),
    ),
    ResourceSelection::new(
        "codec_1_21_7.nbt",
        include_bytes!("../../resources/dimension/codec_1_21_7.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_7),
    ),
    ResourceSelection::new(
        "codec_1_21_6.nbt",
        include_bytes!("../../resources/dimension/codec_1_21_6.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_6),
    ),
    ResourceSelection::new(
        "codec_1_21_5.nbt",
        include_bytes!("../../resources/dimension/codec_1_21_5.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_5),
    ),
    ResourceSelection::new(
        "codec_1_21_4.nbt",
        include_bytes!("../../resources/dimension/codec_1_21_4.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_4),
    ),
    ResourceSelection::new(
        "codec_1_21_2.nbt",
        include_bytes!("../../resources/dimension/codec_1_21_2.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_2),
    ),
    ResourceSelection::new(
        "codec_1_21.nbt",
        include_bytes!("../../resources/dimension/codec_1_21.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21),
    ),
    ResourceSelection::new(
        "codec_1_20_5.nbt",
        include_bytes!("../../resources/dimension/codec_1_20_5.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_20_5),
    ),
    ResourceSelection::new(
        "codec_1_20.nbt",
        include_bytes!("../../resources/dimension/codec_1_20.nbt"),
        VersionMatch::AtLeast(ProtocolVersion::V1_20),
    ),
    ResourceSelection::new(
        "codec_1_19_4.nbt",
        include_bytes!("../../resources/dimension/codec_1_19_4.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_19_4),
    ),
    ResourceSelection::new(
        "codec_1_19_1.nbt",
        include_bytes!("../../resources/dimension/codec_1_19_1.nbt"),
        VersionMatch::AtLeast(ProtocolVersion::V1_19_1),
    ),
    ResourceSelection::new(
        "codec_1_19.nbt",
        include_bytes!("../../resources/dimension/codec_1_19.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_19),
    ),
    ResourceSelection::new(
        "codec_1_18_2.nbt",
        include_bytes!("../../resources/dimension/codec_1_18_2.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_18_2),
    ),
    ResourceSelection::new(
        "codec_1_17.nbt",
        include_bytes!("../../resources/dimension/codec_1_17.nbt"),
        VersionMatch::AtLeast(ProtocolVersion::V1_17),
    ),
    ResourceSelection::new(
        "codec_1_16_2.nbt",
        include_bytes!("../../resources/dimension/codec_1_16_2.nbt"),
        VersionMatch::AtLeast(ProtocolVersion::V1_16_2),
    ),
    ResourceSelection::new(
        "codec_1_16.nbt",
        include_bytes!("../../resources/dimension/codec_1_16.nbt"),
        VersionMatch::Any,
    ),
];

fn string_field<'a>(compound: &'a Compound, key: &str) -> Option<&'a str> {
    match compound.get(key) {
        Some(Value::String(text)) => Some(text),
        _ => None,
    }
}

/// Reads a numeric field the way adventure's `CompoundBinaryTag.getInt` does: any number
/// narrows to `i32`, and an absent or non-numeric field reads as zero.
fn int_field_or_zero(compound: &Compound, key: &str) -> i32 {
    compound
        .get(key)
        .and_then(|value| value.as_i32())
        .unwrap_or(0)
}

fn compound_field<'a>(compound: &'a Compound, key: &str) -> Option<&'a Compound> {
    match compound.get(key) {
        Some(Value::Compound(nested)) => Some(nested),
        _ => None,
    }
}

/// Reads a list of compounds, treating an absent or differently typed field as empty —
/// adventure's `getList` behaves the same way.
fn compound_list<'a>(compound: &'a Compound, key: &str) -> &'a [Compound] {
    match compound.get(key) {
        Some(Value::List(List::Compound(entries))) => entries,
        _ => &[],
    }
}

/// The list a codec keeps its dimension types in.
///
/// From 1.16.2 they live under `minecraft:dimension_type` → `value`; the 1.16 codec has
/// a bare top-level `dimension` list instead. A codec that has the wrapper but no
/// `value` list yields nothing rather than falling back to the legacy key, matching the
/// Java branch that only consults `dimension` when the wrapper is missing entirely.
pub(crate) fn dimension_entries(codec: &Compound) -> &[Compound] {
    match compound_field(codec, "minecraft:dimension_type") {
        Some(dimension_type) => compound_list(dimension_type, "value"),
        None => compound_list(codec, "dimension"),
    }
}

/// Modern codec shape: each entry carries its own `id` and an `element` compound holding
/// the dimension's properties.
fn find_modern_dimension(
    codec: &Arc<Compound>,
    entries: &[Compound],
    dimension_key: &str,
) -> Option<Dimension> {
    entries.iter().find_map(|entry| {
        if string_field(entry, "name") != Some(dimension_key) {
            return None;
        }
        let element = compound_field(entry, "element")?;

        Some(Dimension::new(
            dimension_key.to_owned(),
            int_field_or_zero(entry, "id"),
            int_field_or_zero(element, "height"),
            Arc::clone(codec),
            element.clone(),
        ))
    })
}

/// Legacy 1.16 codec shape: the entry is the element, and its position in the list is
/// the id the client will use.
fn find_legacy_dimension(
    codec: &Arc<Compound>,
    entries: &[Compound],
    dimension_key: &str,
) -> Option<Dimension> {
    entries.iter().enumerate().find_map(|(index, entry)| {
        if string_field(entry, "name") != Some(dimension_key) {
            return None;
        }

        Some(Dimension::new(
            dimension_key.to_owned(),
            i32::try_from(index).ok()?,
            int_field_or_zero(entry, "height"),
            Arc::clone(codec),
            entry.clone(),
        ))
    })
}

/// Every dimension codec the server can send, decoded once at startup.
///
/// Ports Java's `DimensionRegistry`. Decoding all nineteen codecs eagerly costs a few
/// milliseconds before the listener binds and turns a corrupt resource into a startup
/// failure rather than a disconnect for the first client of that version.
pub struct DimensionRegistry {
    codecs: Vec<Arc<Compound>>,
}

impl DimensionRegistry {
    pub fn load() -> Result<Self, ResourceLoadError> {
        let codecs = ResourceSelection::decode_all(CODEC_LADDER)?
            .into_iter()
            .map(Arc::new)
            .collect();

        Ok(Self { codecs })
    }

    fn shared_codec(&self, version: ProtocolVersion) -> Option<&Arc<Compound>> {
        let rung = ResourceSelection::select(CODEC_LADDER, version)?;

        self.codecs.get(rung)
    }

    /// The registry codec this protocol version receives.
    pub fn codec(&self, version: ProtocolVersion) -> Option<&Compound> {
        self.shared_codec(version).map(Arc::as_ref)
    }

    /// Resolves a dimension by its namespaced key, for example `minecraft:overworld`.
    ///
    /// Tries the modern shape first, then the legacy one, exactly as Java does — a
    /// modern codec whose matching entry has no `element` therefore falls through to the
    /// legacy reading of the same list.
    ///
    /// Java also re-applies its search to the list's first entry when the loop finds
    /// nothing, but that fallback re-runs the same name comparison and so can only
    /// succeed when entry zero already matched. It is unreachable, and leaving it out
    /// changes no behaviour: an unknown key resolves to `None` in both implementations.
    pub fn find_dimension(
        &self,
        version: ProtocolVersion,
        dimension_key: &str,
    ) -> Option<Dimension> {
        let codec = self.shared_codec(version)?;
        let entries = dimension_entries(codec);

        find_modern_dimension(codec, entries, dimension_key)
            .or_else(|| find_legacy_dimension(codec, entries, dimension_key))
    }
}
