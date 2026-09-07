//! Length-prefixed framing.
//!
//! Every Minecraft packet travels as `varint(length) || payload`. [`VarIntFrameCodec`]
//! is the port of the Java implementation's `VarIntFrameDecoder` and
//! `VarIntLengthEncoder`, and it keeps their quirks: leading `0x00` bytes are skipped,
//! and the length prefix is capped at 21 bits.

mod frame_error;
mod length_prefix;
mod var_int_frame_codec;

pub use frame_error::FrameError;
pub use var_int_frame_codec::VarIntFrameCodec;
