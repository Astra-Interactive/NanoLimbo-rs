/// One tag: its namespaced key and the registry entry ids it groups.
pub struct TagEntry {
    key: String,
    entry_ids: Vec<i32>,
}

impl TagEntry {
    pub fn new(key: String, entry_ids: Vec<i32>) -> Self {
        Self { key, entry_ids }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    /// Ids of the registry entries in this tag. Legitimately empty: vanilla ships tags
    /// such as `minecraft:incorrect_for_diamond_tool` with no members at all.
    pub fn entry_ids(&self) -> &[i32] {
        &self.entry_ids
    }
}
