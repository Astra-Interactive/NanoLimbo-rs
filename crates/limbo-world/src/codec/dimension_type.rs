use std::collections::HashMap;

use limbo_protocol::version::ProtocolVersion;

use crate::codec::DimensionLookupError;
use crate::codec::DimensionRegistry;
use crate::codec::VersionedDimension;

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
