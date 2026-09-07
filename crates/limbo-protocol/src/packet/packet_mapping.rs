use crate::packet::packet_kind::PacketKind;
use crate::packet::version_range::VersionRange;

/// Binds one packet to one numeric id over a span of protocol versions.
///
/// Mappings live in per-state, per-direction tables. Within a table, and for any single
/// version, the binding must be one-to-one in both directions: two packets sharing an id
/// make decoding ambiguous, and one packet holding two ids makes encoding ambiguous.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketMapping {
    kind: PacketKind,
    id: i32,
    versions: VersionRange,
}

impl PacketMapping {
    pub(crate) const fn new(kind: PacketKind, id: i32, versions: VersionRange) -> Self {
        Self { kind, id, versions }
    }

    pub const fn kind(&self) -> PacketKind {
        self.kind
    }

    pub const fn id(&self) -> i32 {
        self.id
    }

    pub const fn versions(&self) -> VersionRange {
        self.versions
    }
}
