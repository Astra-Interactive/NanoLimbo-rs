/// Resident memory the process is currently using, in bytes.
///
/// Upstream reports JVM heap figures here. A native binary has no heap to report, so this
/// is the operating system's view instead — the number an operator actually compares
/// against the old deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryUsage {
    pub resident_bytes: u64,
}

impl MemoryUsage {
    /// Size of a memory page, which `/proc/self/statm` counts in.
    const PAGE_SIZE: u64 = 4096;

    pub const fn resident_megabytes(self) -> u64 {
        self.resident_bytes / (1024 * 1024)
    }

    /// Parses the resident page count out of the second field of `/proc/self/statm`.
    pub(crate) fn parse_statm(contents: &str, page_size: u64) -> Option<Self> {
        let pages: u64 = contents.split_whitespace().nth(1)?.parse().ok()?;
        Some(Self {
            resident_bytes: pages.saturating_mul(page_size),
        })
    }

    /// Where the figures come from, which is the only part that varies by platform.
    ///
    /// Only Linux is covered: it is where this runs in production, and the alternatives
    /// elsewhere mean either a C dependency or spawning a process for a console nicety.
    fn read_statm() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string("/proc/self/statm").ok()
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }

    /// Reads resident memory, or `None` where the platform offers no cheap way to ask.
    pub fn current() -> Option<Self> {
        Self::parse_statm(&Self::read_statm()?, Self::PAGE_SIZE)
    }
}
