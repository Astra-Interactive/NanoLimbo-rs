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
