use std::sync::Arc;

use valence_nbt::Compound;

/// Sections a chunk column is split into, each 16 blocks tall.
const BLOCKS_PER_CHUNK_SECTION: i32 = 16;

/// A dimension as one protocol version sees it.
///
/// The same dimension has a different id, height and element layout depending on which
/// codec the client is sent, so a value of this type is only meaningful together with
/// the version it was resolved for.
pub struct Dimension {
    key: String,
    id: i32,
    height: i32,
    codec: Arc<Compound>,
    element_codec: Compound,
}

impl Dimension {
    /// `codec` is shared: one registry codec backs every dimension resolved from it, and
    /// cloning it per dimension per version would copy hundreds of kilobytes. The
    /// element is a small slice of that codec and is owned outright.
    pub fn new(
        key: String,
        id: i32,
        height: i32,
        codec: Arc<Compound>,
        element_codec: Compound,
    ) -> Self {
        Self {
            key,
            id,
            height,
            codec,
            element_codec,
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    /// Id the client uses to refer to this dimension: the `id` field of a modern codec
    /// entry, or the entry's position in the 1.16 codec's flat list.
    pub fn id(&self) -> i32 {
        self.id
    }

    /// Build height in blocks.
    ///
    /// Zero for the 1.16 and 1.16.2 codecs, which predate the field. The Java
    /// implementation reads it with `getInt`, whose default is also zero, so both
    /// implementations send the same chunk layout to those clients.
    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn chunk_sections(&self) -> i32 {
        self.height / BLOCKS_PER_CHUNK_SECTION
    }

    /// The whole registry codec this dimension was resolved from, as sent in Join Game
    /// before 1.20.5.
    pub fn codec(&self) -> &Compound {
        &self.codec
    }

    /// Just this dimension's own entry, as sent in Join Game between 1.16.2 and 1.18.2.
    pub fn element_codec(&self) -> &Compound {
        &self.element_codec
    }
}
