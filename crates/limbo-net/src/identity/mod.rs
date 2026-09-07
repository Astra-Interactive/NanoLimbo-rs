//! Player identity: the offline-mode UUID and the UUID text a proxy sends.

mod offline_uuid;
mod uuid_parse_error;
mod uuid_text;

pub use offline_uuid::offline_mode_uuid;
pub use uuid_parse_error::UuidParseError;
pub use uuid_text::parse_uuid;
