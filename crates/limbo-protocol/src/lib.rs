//! Minecraft protocol primitives shared by every layer of the server.
//!
//! This crate is pure domain logic: it performs no I/O and depends on no runtime,
//! so all of it is host-testable.

pub mod buffer {
    //! Reading and writing the protocol's primitive types over [`bytes`] buffers.
    //!
    //! Replaces the Java `ByteMessage`, which wrapped Netty's `ByteBuf` and re-declared its
    //! entire surface. Extension traits over `Buf`/`BufMut` add only what the Minecraft
    //! protocol defines beyond fixed-width big-endian primitives.

    mod nbt_encode_error;
    mod packet_decode_error;
    mod protocol_read;
    mod protocol_write;
    mod write_compound;

    pub use nbt_encode_error::NbtEncodeError;
    pub use packet_decode_error::PacketDecodeError;
    pub use protocol_read::ProtocolRead;
    pub use protocol_write::ProtocolWrite;
    pub use write_compound::write_compound;
}

pub mod packet {
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
}

pub mod version {
    //! Protocol version identification and the table of versions the server supports.

    mod protocol_version;
    mod version_descriptor;

    pub use protocol_version::{ProtocolVersion, SUPPORTED};
    pub use version_descriptor::VersionDescriptor;
}
