use bytes::Buf;
use limbo_protocol::buffer::{PacketDecodeError, ProtocolRead};
use limbo_protocol::packet::{PacketKind, PacketRoute};
use limbo_protocol::version::ProtocolVersion;

use crate::serverbound::Handshake;
use crate::serverbound::LoginPluginResponse;
use crate::serverbound::LoginStart;
use crate::serverbound::PluginMessage;

/// A client may advertise at most this many resource packs it already has.
const MAX_KNOWN_PACKS: i32 = 16;

/// Namespace, id and version strings inside a known-pack entry.
const MAX_KNOWN_PACK_FIELD_LENGTH: usize = 256;

/// Everything the server understands from a client.
///
/// A limbo server reads ten packets out of the hundred-odd the protocol defines. The rest
/// are recognised by id and dropped, which is why decoding returns an `Option` rather than
/// failing on an id it has no use for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerBoundPacket {
    Handshake(Handshake),
    StatusRequest,
    StatusPing { payload: i64 },
    LoginStart(LoginStart),
    LoginPluginResponse(LoginPluginResponse),
    LoginAcknowledged,
    PluginMessage(PluginMessage),
    FinishConfiguration,
    KnownPacks,
    KeepAlive { id: i64 },
}

/// Consumes a known-packs list without keeping it.
///
/// The server answers the same way regardless of what the client already has, but the
/// list still has to be validated: an unbounded count here would be a client telling the
/// server how much work to do.
fn skip_known_packs<B>(buffer: &mut B) -> Result<(), PacketDecodeError>
where
    B: Buf + ?Sized,
{
    let count = buffer.read_var_int()?;
    if !(0..=MAX_KNOWN_PACKS).contains(&count) {
        return Err(PacketDecodeError::ByteArrayTooLong {
            actual: count.unsigned_abs() as usize,
            limit: MAX_KNOWN_PACKS as usize,
        });
    }

    for _ in 0..count {
        for _ in 0..3 {
            buffer.read_string(MAX_KNOWN_PACK_FIELD_LENGTH)?;
        }
    }
    Ok(())
}

/// Reads a keep-alive token, which changed width twice.
fn read_keep_alive<B>(buffer: &mut B, version: ProtocolVersion) -> Result<i64, PacketDecodeError>
where
    B: Buf + ?Sized,
{
    if version >= ProtocolVersion::V1_12_2 {
        buffer.read_i64()
    } else if version >= ProtocolVersion::V1_8 {
        buffer.read_var_int().map(i64::from)
    } else {
        buffer.read_i32().map(i64::from)
    }
}

impl ServerBoundPacket {
    /// Decodes the packet arriving under `id` on `route`.
    ///
    /// `Ok(None)` means the id is not one the server acts on; the caller drops the frame
    /// and carries on. `Err` means the bytes were malformed, which ends the connection.
    pub fn decode<B>(
        route: PacketRoute,
        id: i32,
        buffer: &mut B,
    ) -> Result<Option<Self>, PacketDecodeError>
    where
        B: Buf + ?Sized,
    {
        let version = route.version();

        let packet = match route.kind_of(id) {
            Some(PacketKind::Handshake) => Self::Handshake(Handshake::decode(buffer)?),
            Some(PacketKind::StatusRequest) => Self::StatusRequest,
            Some(PacketKind::StatusPing) => Self::StatusPing {
                payload: buffer.read_i64()?,
            },
            Some(PacketKind::LoginStart) => Self::LoginStart(LoginStart::decode(buffer, version)?),
            Some(PacketKind::LoginPluginResponse) => {
                Self::LoginPluginResponse(LoginPluginResponse::decode(buffer)?)
            }
            Some(PacketKind::LoginAcknowledged) => Self::LoginAcknowledged,
            Some(PacketKind::PluginMessage) => Self::PluginMessage(PluginMessage::decode(buffer)?),
            Some(PacketKind::FinishConfiguration) => Self::FinishConfiguration,
            Some(PacketKind::KnownPacks) => {
                skip_known_packs(buffer)?;
                Self::KnownPacks
            }
            Some(PacketKind::KeepAlive) => Self::KeepAlive {
                id: read_keep_alive(buffer, version)?,
            },
            _ => return Ok(None),
        };

        Ok(Some(packet))
    }
}
