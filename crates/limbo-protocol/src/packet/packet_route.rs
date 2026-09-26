use crate::packet::connection_state::ConnectionState;
use crate::packet::packet_direction::PacketDirection;
use crate::packet::packet_kind::PacketKind;
use crate::packet::packet_mapping::PacketMapping;
use crate::packet::packet_mappings;
use crate::version::ProtocolVersion;

/// The coordinates that decide how a packet id is interpreted: connection state,
/// direction and protocol version.
///
/// A decoder holds one of these and replaces it whenever the connection changes state
/// or the client's version becomes known, which is cheaper than the per-version map of
/// suppliers the Java implementation rebuilt for every registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketRoute {
    state: ConnectionState,
    direction: PacketDirection,
    version: ProtocolVersion,
}

impl PacketRoute {
    pub const fn new(
        state: ConnectionState,
        direction: PacketDirection,
        version: ProtocolVersion,
    ) -> Self {
        Self {
            state,
            direction,
            version,
        }
    }

    pub(crate) fn mappings(&self) -> impl Iterator<Item = &'static PacketMapping> {
        let version = self.version;
        packet_mappings::table(self.state, self.direction)
            .iter()
            .filter(move |mapping| mapping.versions().contains(version))
    }

    pub const fn state(&self) -> ConnectionState {
        self.state
    }

    pub const fn direction(&self) -> PacketDirection {
        self.direction
    }

    pub const fn version(&self) -> ProtocolVersion {
        self.version
    }

    /// Which packet arrives under `id`, or `None` when this route defines no such id.
    ///
    /// An unknown id is normal traffic, not an error: the server ignores the packets it
    /// has no use for rather than dropping the connection.
    pub fn kind_of(&self, id: i32) -> Option<PacketKind> {
        self.mappings()
            .find(|mapping| mapping.id() == id)
            .map(PacketMapping::kind)
    }

    /// The id `kind` travels under, or `None` when the packet does not exist here.
    pub fn id_of(&self, kind: PacketKind) -> Option<i32> {
        self.mappings()
            .find(|mapping| mapping.kind() == kind)
            .map(PacketMapping::id)
    }
}
