//! Protocol version identification and the table of versions the server supports.

mod protocol_version;
mod version_descriptor;

pub use protocol_version::{ProtocolVersion, SUPPORTED};
pub use version_descriptor::VersionDescriptor;
