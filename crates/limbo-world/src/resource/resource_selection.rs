use std::io::Read;

use flate2::read::GzDecoder;
use limbo_protocol::version::ProtocolVersion;
use valence_nbt::Compound;
use valence_nbt::binary::from_binary;

use crate::resource::ResourceLoadError;
use crate::resource::VersionMatch;

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
