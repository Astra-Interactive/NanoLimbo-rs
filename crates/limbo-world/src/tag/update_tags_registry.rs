use limbo_protocol::version::ProtocolVersion;
use valence_nbt::{Compound, List, Value};

use crate::resource::ResourceLoadError;
use crate::resource::ResourceSelection;
use crate::resource::VersionMatch;
use crate::tag::TagEntry;
use crate::tag::TagRegistry;
use crate::tag::UpdateTagsError;

/// Which tag set each protocol version receives, transcribed rung by rung from Java's
/// `DimensionRegistry.createUpdateTags`.
///
/// Like the codec ladder it mixes lower bounds with exact matches, and the trailing rung
/// claims everything older. Only clients from 1.20.5 are ever sent this packet, so the
/// rungs below that release exist to keep the transcription faithful, not because any
/// client reaches them.
pub(crate) static TAG_LADDER: &[ResourceSelection] = &[
    ResourceSelection::new(
        "tags_26_2.nbt",
        include_bytes!("../../resources/dimension/tags_26_2.nbt"),
        VersionMatch::AtLeast(ProtocolVersion::V26_2),
    ),
    ResourceSelection::new(
        "tags_26_1.nbt",
        include_bytes!("../../resources/dimension/tags_26_1.nbt"),
        VersionMatch::AtLeast(ProtocolVersion::V26_1),
    ),
    ResourceSelection::new(
        "tags_1_21_11.nbt",
        include_bytes!("../../resources/dimension/tags_1_21_11.nbt"),
        VersionMatch::AtLeast(ProtocolVersion::V1_21_11),
    ),
    ResourceSelection::new(
        "tags_1_21_9.nbt",
        include_bytes!("../../resources/dimension/tags_1_21_9.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_9),
    ),
    ResourceSelection::new(
        "tags_1_21_7.nbt",
        include_bytes!("../../resources/dimension/tags_1_21_7.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_7),
    ),
    ResourceSelection::new(
        "tags_1_21_6.nbt",
        include_bytes!("../../resources/dimension/tags_1_21_6.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_6),
    ),
    ResourceSelection::new(
        "tags_1_21_5.nbt",
        include_bytes!("../../resources/dimension/tags_1_21_5.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_5),
    ),
    ResourceSelection::new(
        "tags_1_21_4.nbt",
        include_bytes!("../../resources/dimension/tags_1_21_4.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_4),
    ),
    ResourceSelection::new(
        "tags_1_21_2.nbt",
        include_bytes!("../../resources/dimension/tags_1_21_2.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21_2),
    ),
    ResourceSelection::new(
        "tags_1_21.nbt",
        include_bytes!("../../resources/dimension/tags_1_21.nbt"),
        VersionMatch::Exactly(ProtocolVersion::V1_21),
    ),
    ResourceSelection::new(
        "tags_1_20_5.nbt",
        include_bytes!("../../resources/dimension/tags_1_20_5.nbt"),
        VersionMatch::Any,
    ),
];

fn parse_entry_ids(
    tag_value: &Value,
    registry_key: &str,
    tag_key: &str,
) -> Result<Vec<i32>, UpdateTagsError> {
    match tag_value {
        Value::List(List::Int(entry_ids)) => Ok(entry_ids.clone()),
        // An empty NBT list carries the `TAG_End` element type, so a tag with no members
        // is indistinguishable from a list of the wrong type by its tag id alone.
        Value::List(List::End) => Ok(Vec::new()),
        _ => Err(UpdateTagsError::TagNotIntList {
            registry: registry_key.to_owned(),
            tag: tag_key.to_owned(),
        }),
    }
}

fn parse_tags(registry_key: &str, tags: &Compound) -> Result<Vec<TagEntry>, UpdateTagsError> {
    tags.iter()
        .map(|(tag_key, tag_value)| {
            let entry_ids = parse_entry_ids(tag_value, registry_key, tag_key)?;

            Ok(TagEntry::new(tag_key.clone(), entry_ids))
        })
        .collect()
}

/// Flattens a tag resource into registries, their tags and the entry ids each tag holds.
///
/// Java collects this into nested `HashMap`s, so the order in which it writes registries
/// and tags to the wire is whatever `HashMap` iteration happens to produce for those
/// keys. This port keeps the order the NBT resource declares. Both are correct — the
/// packet is a set of tags, and the client indexes it by key — but it does mean the
/// Update Tags packet cannot be byte-compared against the Java oracle. Only its
/// structure can.
pub(crate) fn parse_update_tags(tag_set: &Compound) -> Result<Vec<TagRegistry>, UpdateTagsError> {
    tag_set
        .iter()
        .map(|(registry_key, registry_value)| {
            let Value::Compound(tags) = registry_value else {
                return Err(UpdateTagsError::RegistryNotCompound {
                    registry: registry_key.clone(),
                });
            };

            Ok(TagRegistry::new(
                registry_key.clone(),
                parse_tags(registry_key, tags)?,
            ))
        })
        .collect()
}

/// Every registry tag set the server can send, decoded once at startup.
pub struct UpdateTagsRegistry {
    tag_sets: Vec<Compound>,
}

impl UpdateTagsRegistry {
    pub fn load() -> Result<Self, ResourceLoadError> {
        Ok(Self {
            tag_sets: ResourceSelection::decode_all(TAG_LADDER)?,
        })
    }

    /// The Update Tags payload this protocol version receives.
    pub fn tags_for(&self, version: ProtocolVersion) -> Result<Vec<TagRegistry>, UpdateTagsError> {
        let tag_set = ResourceSelection::select(TAG_LADDER, version)
            .and_then(|rung| self.tag_sets.get(rung))
            .ok_or(UpdateTagsError::NoTagSet { version })?;

        parse_update_tags(tag_set)
    }
}
