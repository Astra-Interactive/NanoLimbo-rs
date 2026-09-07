//! Clientbound packets: what the server says, in the shape each protocol version reads.
//!
//! Every packet here is a plain description of its payload with public fields, and every
//! encoder is a transcription of the Java implementation's `encode` method including its
//! version conditionals. Payloads carry no id prefix — the id depends on the connection
//! state as well as the version, so it is resolved through
//! [`PacketRoute`](limbo_protocol::packet::PacketRoute) by the layer that frames them.
//!
//! Nothing in this crate reads a clock or a random source. Values the Java implementation
//! drew from `Random` — entity ids, teleport ids, session and boss bar uuids, keep alive
//! ids — arrive as fields, which is what makes byte-level golden tests possible. See
//! `MIGRATION_PLAN.md` section 7.5.

pub mod configuration;
pub mod login;
pub mod play;
pub mod status;

mod clientbound_packet;
mod packet_encode_error;
mod pre_encoded_packet;
mod write_bool;

pub use clientbound_packet::ClientboundPacket;
pub use packet_encode_error::PacketEncodeError;
pub use pre_encoded_packet::PreEncodedPacket;
