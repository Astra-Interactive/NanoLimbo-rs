//! Packet identity: which packet a numeric id means, and which id a packet travels under.
//!
//! Ids depend on the connection state, the direction of travel and the protocol version.
//! The bindings live in declarative tables validated by the crate's integration tests,
//! so an ambiguous table fails the build rather than being resolved by declaration order
//! the way the Java implementation resolved it.

mod connection_state;
mod packet_direction;
mod packet_kind;
mod packet_mapping;
mod packet_mappings;
mod packet_route;
mod version_range;

pub use connection_state::ConnectionState;
pub use packet_direction::PacketDirection;
pub use packet_kind::PacketKind;
pub use packet_mapping::PacketMapping;
pub use packet_route::PacketRoute;
pub use version_range::VersionRange;
