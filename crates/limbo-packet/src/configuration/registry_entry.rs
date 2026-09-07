use valence_nbt::Compound;

/// One entry of a registry: its key, and the compound describing it when the server
/// overrides what the client's own data pack already has.
pub struct RegistryEntry<'a> {
    pub name: &'a str,
    pub element: Option<&'a Compound>,
}
