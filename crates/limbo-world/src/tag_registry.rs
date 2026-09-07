use crate::tag_entry::TagEntry;

/// Every tag defined for one registry, in the order the resource declares them.
pub struct TagRegistry {
    key: String,
    tags: Vec<TagEntry>,
}

impl TagRegistry {
    pub fn new(key: String, tags: Vec<TagEntry>) -> Self {
        Self { key, tags }
    }

    /// Registry this group of tags belongs to, for example `minecraft:block`.
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn tags(&self) -> &[TagEntry] {
        &self.tags
    }
}
