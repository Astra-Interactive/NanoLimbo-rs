use std::io::Read;

use flate2::read::GzDecoder;
use limbo_protocol::version::ProtocolVersion;
use valence_nbt::Compound;
use valence_nbt::binary::from_binary;

use crate::resource_load_error::ResourceLoadError;
use crate::version_match::VersionMatch;

/// One rung of a version-selection ladder: a gzip-compressed NBT resource compiled into
/// the binary, plus the comparison that decides whether this rung serves a given client.
///
/// The resources ship inside the executable rather than next to it, so a deployment is a
/// single file and a missing data directory cannot produce a half-working server.
pub struct ResourceSelection {
    name: &'static str,
    gzip_bytes: &'static [u8],
    version_match: VersionMatch,
}

impl ResourceSelection {
    pub const fn new(
        name: &'static str,
        gzip_bytes: &'static [u8],
        version_match: VersionMatch,
    ) -> Self {
        Self {
            name,
            gzip_bytes,
            version_match,
        }
    }

    /// File name of the resource, used to say which one failed to load.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The resource's uncompressed NBT bytes, exactly as the Java implementation reads
    /// them through `BinaryTagIO.unlimitedReader()`.
    pub(crate) fn decompress(&self) -> Result<Vec<u8>, ResourceLoadError> {
        let mut decompressed = Vec::new();
        GzDecoder::new(self.gzip_bytes)
            .read_to_end(&mut decompressed)
            .map_err(|error| ResourceLoadError::Decompress {
                resource: self.name,
                reason: error.to_string(),
            })?;

        Ok(decompressed)
    }

    pub fn decode(&self) -> Result<Compound, ResourceLoadError> {
        let decompressed = self.decompress()?;

        let (compound, _root_name): (Compound, String) = from_binary(&mut decompressed.as_slice())
            .map_err(|error| ResourceLoadError::Parse {
                resource: self.name,
                reason: error.to_string(),
            })?;

        Ok(compound)
    }

    /// Decodes an entire ladder up front, so a corrupt resource is reported at startup
    /// instead of when the first client of that version connects.
    pub fn decode_all(ladder: &[Self]) -> Result<Vec<Compound>, ResourceLoadError> {
        ladder.iter().map(Self::decode).collect()
    }

    /// Index of the first rung that claims `version`.
    ///
    /// First, not best: the ladder is ordered, and the Java `if / else if` chain it was
    /// transcribed from resolves overlaps the same way.
    pub fn select(ladder: &[Self], version: ProtocolVersion) -> Option<usize> {
        ladder
            .iter()
            .position(|selection| selection.version_match.matches(version))
    }
}

#[cfg(test)]
mod tests {

    use valence_nbt::binary::to_binary;

    use crate::dimension_registry::CODEC_LADDER;
    use crate::update_tags_registry::TAG_LADDER;

    use super::*;

    fn every_embedded_resource() -> impl Iterator<Item = &'static ResourceSelection> {
        CODEC_LADDER.iter().chain(TAG_LADDER.iter())
    }

    #[test]
    fn given_every_embedded_resource_when_decoded_then_it_yields_a_compound_with_an_empty_root_name()
     {
        for selection in every_embedded_resource() {
            let decompressed = selection
                .decompress()
                .expect("every embedded resource must be valid gzip");

            let (compound, root_name): (Compound, String) =
                from_binary(&mut decompressed.as_slice())
                    .expect("every embedded resource must be valid nbt");

            assert_eq!(root_name, "", "{}", selection.name());
            assert!(!compound.is_empty(), "{}", selection.name());
        }
    }

    /// Re-encoding must reproduce the resource byte for byte. Anything less means the
    /// bytes reaching a client differ from the ones the Java implementation ships, which
    /// is precisely the class of difference a limbo server cannot afford.
    #[test]
    fn given_every_embedded_resource_when_re_encoded_then_the_bytes_are_unchanged() {
        for selection in every_embedded_resource() {
            let decompressed = selection
                .decompress()
                .expect("every embedded resource must be valid gzip");
            let (compound, root_name): (Compound, String) =
                from_binary(&mut decompressed.as_slice())
                    .expect("every embedded resource must be valid nbt");

            let mut re_encoded = Vec::with_capacity(decompressed.len());
            to_binary(&compound, &mut re_encoded, &root_name).expect("re-encoding must succeed");

            assert_eq!(re_encoded, decompressed, "{}", selection.name());
        }
    }

    #[test]
    fn given_the_two_ladders_when_counted_then_every_shipped_resource_is_reachable() {
        assert_eq!(CODEC_LADDER.len(), 19);
        assert_eq!(TAG_LADDER.len(), 11);
    }

    #[test]
    fn given_a_ladder_whose_last_rung_matches_anything_when_any_version_is_offered_then_a_rung_is_found()
     {
        for version in ProtocolVersion::all() {
            assert!(
                ResourceSelection::select(CODEC_LADDER, version).is_some(),
                "no codec for {version}"
            );
            assert!(
                ResourceSelection::select(TAG_LADDER, version).is_some(),
                "no tag set for {version}"
            );
        }
    }

    #[test]
    fn given_an_empty_ladder_when_a_version_is_offered_then_no_rung_is_found() {
        assert_eq!(ResourceSelection::select(&[], ProtocolVersion::V1_21), None);
    }
}
