use crate::version::ProtocolVersion;

/// An inclusive span of protocol versions over which one packet id stays put.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VersionRange {
    start: ProtocolVersion,
    end: ProtocolVersion,
}

impl VersionRange {
    pub const fn new(start: ProtocolVersion, end: ProtocolVersion) -> Self {
        Self { start, end }
    }

    pub const fn start(&self) -> ProtocolVersion {
        self.start
    }

    pub const fn end(&self) -> ProtocolVersion {
        self.end
    }

    pub const fn contains(&self, version: ProtocolVersion) -> bool {
        self.start.number() <= version.number() && version.number() <= self.end.number()
    }
}
