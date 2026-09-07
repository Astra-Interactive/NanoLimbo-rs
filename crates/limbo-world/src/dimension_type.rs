use std::collections::HashMap;

use limbo_protocol::version::ProtocolVersion;

use crate::dimension_lookup_error::DimensionLookupError;
use crate::dimension_registry::DimensionRegistry;
use crate::versioned_dimension::VersionedDimension;

/// A dimension the limbo world can be placed in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DimensionType {
    Overworld,
    TheNether,
    TheEnd,
}

impl DimensionType {
    pub const ALL: [Self; 3] = [Self::Overworld, Self::TheNether, Self::TheEnd];

    /// Namespaced key as it appears in the `name` field of a codec entry.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Overworld => "minecraft:overworld",
            Self::TheNether => "minecraft:the_nether",
            Self::TheEnd => "minecraft:the_end",
        }
    }

    /// Number that identified this dimension before 1.16 replaced it with a key.
    pub const fn legacy_id(self) -> i32 {
        match self {
            Self::Overworld => 0,
            Self::TheNether => -1,
            Self::TheEnd => 1,
        }
    }

    /// Resolves this dimension against every supported protocol version.
    ///
    /// Java skips versions whose codec does not define the dimension and only notices
    /// when a packet for such a client is encoded, by which point the connection is
    /// already open. Failing here instead turns that into a refusal to start.
    pub fn resolve(
        self,
        registry: &DimensionRegistry,
    ) -> Result<VersionedDimension, DimensionLookupError> {
        let mut per_version = HashMap::new();

        for version in ProtocolVersion::all() {
            let dimension = registry.find_dimension(version, self.key()).ok_or(
                DimensionLookupError::Unresolved {
                    key: self.key(),
                    version,
                },
            )?;
            per_version.insert(version, dimension);
        }

        Ok(VersionedDimension::new(
            self.key().to_owned(),
            self.legacy_id(),
            per_version,
        ))
    }
}

#[cfg(test)]
mod tests {

    use crate::dimension::Dimension;

    use super::*;

    /// Build height the vanilla codecs declare for the overworld. It grew to 384 in the
    /// 1.18.2 codec; the 1.17 codec still says 256, and 1.18 clients are served that one.
    fn expected_overworld_height(version: ProtocolVersion) -> i32 {
        if version < ProtocolVersion::V1_17 {
            0
        } else if version < ProtocolVersion::V1_18_2 {
            256
        } else {
            384
        }
    }

    /// The nether and the end have been 256 blocks tall in every codec that states a
    /// height at all.
    fn expected_lower_dimension_height(version: ProtocolVersion) -> i32 {
        if version < ProtocolVersion::V1_17 {
            0
        } else {
            256
        }
    }

    fn resolved(dimension_type: DimensionType) -> VersionedDimension {
        let registry = DimensionRegistry::load().expect("every embedded codec must load");

        dimension_type
            .resolve(&registry)
            .expect("every supported version must define this dimension")
    }

    fn dimension_for(versioned: &VersionedDimension, version: ProtocolVersion) -> &Dimension {
        versioned
            .for_version(version)
            .expect("resolving covered every supported version")
    }

    #[test]
    fn given_every_supported_version_when_each_dimension_type_is_resolved_then_none_is_missing() {
        for dimension_type in DimensionType::ALL {
            let versioned = resolved(dimension_type);

            for version in ProtocolVersion::all() {
                assert!(
                    versioned.for_version(version).is_some(),
                    "{dimension_type:?} is missing for {version}"
                );
            }
        }
    }

    #[test]
    fn given_every_supported_version_when_the_overworld_is_resolved_then_its_height_follows_the_codec()
     {
        let overworld = resolved(DimensionType::Overworld);

        for version in ProtocolVersion::all() {
            assert_eq!(
                dimension_for(&overworld, version).height(),
                expected_overworld_height(version),
                "{version}"
            );
        }
    }

    #[test]
    fn given_every_supported_version_when_the_nether_and_the_end_are_resolved_then_they_stay_256_tall()
     {
        for dimension_type in [DimensionType::TheNether, DimensionType::TheEnd] {
            let versioned = resolved(dimension_type);

            for version in ProtocolVersion::all() {
                assert_eq!(
                    dimension_for(&versioned, version).height(),
                    expected_lower_dimension_height(version),
                    "{dimension_type:?} at {version}"
                );
            }
        }
    }

    #[test]
    fn given_a_resolved_dimension_when_its_chunk_sections_are_read_then_they_cover_its_height() {
        for dimension_type in DimensionType::ALL {
            let versioned = resolved(dimension_type);

            for version in ProtocolVersion::all() {
                let dimension = dimension_for(&versioned, version);

                assert_eq!(
                    dimension.chunk_sections() * 16,
                    dimension.height(),
                    "{version}"
                );
            }
        }
    }

    /// The pre-1.16 wire number and the codec id are unrelated: the nether is `-1` on the
    /// old wire but sits at index 2 or 3 in the codecs. Conflating them would place
    /// players in the wrong dimension.
    #[test]
    fn given_the_nether_when_both_of_its_ids_are_read_then_the_legacy_number_is_not_the_codec_id() {
        let nether = resolved(DimensionType::TheNether);

        assert_eq!(nether.legacy_id(), -1);
        assert_eq!(dimension_for(&nether, ProtocolVersion::V1_16).id(), 2);
        assert_eq!(dimension_for(&nether, ProtocolVersion::V1_21).id(), 3);
    }

    #[test]
    fn given_a_resolved_dimension_when_its_key_is_read_then_it_is_the_key_that_was_looked_up() {
        for dimension_type in DimensionType::ALL {
            let versioned = resolved(dimension_type);

            assert_eq!(versioned.key(), dimension_type.key());
            assert_eq!(
                dimension_for(&versioned, ProtocolVersion::V1_21).key(),
                dimension_type.key()
            );
        }
    }
}
