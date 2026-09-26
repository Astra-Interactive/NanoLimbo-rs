/// Where a target's memory reading comes from.
///
/// A server in a container has no process id the host can see — on Docker Desktop it does
/// not even run on the host kernel — so the two cases genuinely differ rather than being
/// one lookup with a flag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemorySource {
    /// A process on this machine, read from procfs or `ps`.
    Process { pid: u32 },
    /// A container, read from `docker stats`.
    Container { name: String },
}

impl MemorySource {
    /// Reads the `@` suffix of a target.
    ///
    /// Digits mean a process id; anything else names a container. Container names cannot
    /// be all digits, so the two never collide.
    pub fn parse(suffix: &str) -> Self {
        match suffix.parse() {
            Ok(pid) => Self::Process { pid },
            Err(_not_a_number) => Self::Container {
                name: suffix.to_owned(),
            },
        }
    }
}
