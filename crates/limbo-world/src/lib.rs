//! Dimension codecs and registry tags, selected by the protocol version of the client
//! they are being sent to.
//!
//! Pure domain logic: the NBT resources are compiled into the binary, so nothing here
//! touches the filesystem or the network and all of it is host-testable.

mod dimension;
mod dimension_lookup_error;
mod dimension_registry;
mod dimension_type;
mod nbt_encode_error;
mod resource_load_error;
mod resource_selection;
mod tag_entry;
mod tag_registry;
mod update_tags_error;
mod update_tags_registry;
mod version_match;
mod versioned_dimension;
mod write_compound;

pub use dimension::Dimension;
pub use dimension_lookup_error::DimensionLookupError;
pub use dimension_registry::DimensionRegistry;
pub use dimension_type::DimensionType;
pub use nbt_encode_error::NbtEncodeError;
pub use resource_load_error::ResourceLoadError;
pub use resource_selection::ResourceSelection;
pub use tag_entry::TagEntry;
pub use tag_registry::TagRegistry;
pub use update_tags_error::UpdateTagsError;
pub use update_tags_registry::UpdateTagsRegistry;
pub use version_match::VersionMatch;
pub use versioned_dimension::VersionedDimension;
pub use write_compound::write_compound;
