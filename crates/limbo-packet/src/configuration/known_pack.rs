/// A data pack both sides claim to know, so its contents need not be sent.
pub struct KnownPack<'a> {
    pub namespace: &'a str,
    pub id: &'a str,
    /// The pack's version, which for `minecraft:core` is the client's own release name.
    pub version: &'a str,
}
