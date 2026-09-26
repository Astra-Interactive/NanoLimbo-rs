use std::process::Command;

use crate::memory::MemorySource;

/// Resident memory of another process, in bytes.
///
/// Reported for whichever servers the comparison was told about, so the two figures come
/// from the same source and the same moment rather than from two different tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResidentMemory {
    pub bytes: u64,
}

impl ResidentMemory {
    pub const fn megabytes(self) -> f64 {
        self.bytes as f64 / (1024.0 * 1024.0)
    }

    /// Parses the resident page count from the second field of `/proc/<pid>/statm`.
    pub(crate) fn parse_statm(contents: &str, page_size: u64) -> Option<Self> {
        let pages: u64 = contents.split_whitespace().nth(1)?.parse().ok()?;
        Some(Self {
            bytes: pages.saturating_mul(page_size),
        })
    }

    /// Parses the kilobyte figure `ps -o rss=` prints.
    pub(crate) fn parse_ps_rss(output: &str) -> Option<Self> {
        let kilobytes: u64 = output.trim().parse().ok()?;
        Some(Self {
            bytes: kilobytes.saturating_mul(1024),
        })
    }

    /// Reads `/proc/<pid>/statm`, or nothing where there is no procfs.
    fn read_statm(pid: u32) -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            std::fs::read_to_string(format!("/proc/{pid}/statm")).ok()
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = pid;
            None
        }
    }

    /// Parses the figure before the slash in `docker stats`' "12.3MiB / 7.7GiB".
    pub(crate) fn parse_docker_usage(output: &str) -> Option<Self> {
        let used = output.split('/').next()?.trim();
        let split = used.find(|character: char| character.is_alphabetic())?;
        let (amount, unit) = used.split_at(split);
        let amount: f64 = amount.trim().parse().ok()?;

        let scale = match unit.trim() {
            "B" => 1.0,
            "KiB" | "kB" => 1024.0,
            "MiB" | "MB" => 1024.0 * 1024.0,
            "GiB" | "GB" => 1024.0 * 1024.0 * 1024.0,
            _ => return None,
        };
        Some(Self {
            bytes: (amount * scale) as u64,
        })
    }

    fn of_container(name: &str) -> Option<Self> {
        let output = Command::new("docker")
            .args(["stats", "--no-stream", "--format", "{{.MemUsage}}", name])
            .output()
            .ok()?;
        Self::parse_docker_usage(&String::from_utf8_lossy(&output.stdout))
    }

    /// Reads one process's resident memory, or `None` when it cannot be determined.
    ///
    /// Linux comes straight from procfs. Elsewhere this shells out to `ps`, which is slower
    /// but keeps the comparison runnable on a developer's machine rather than only in CI.
    fn of_process(pid: u32) -> Option<Self> {
        const PAGE_SIZE: u64 = 4096;

        if let Some(statm) = Self::read_statm(pid) {
            return Self::parse_statm(&statm, PAGE_SIZE);
        }

        let output = Command::new("ps")
            .args(["-o", "rss=", "-p", &pid.to_string()])
            .output()
            .ok()?;
        Self::parse_ps_rss(&String::from_utf8_lossy(&output.stdout))
    }

    /// Reads a target's memory, or `None` when it cannot be determined.
    ///
    /// A container is measured whole, which is the number an operator sees; a process is
    /// measured on its own. The two are not directly comparable, so do not mix them in one
    /// run and then read the columns side by side.
    pub fn of_source(source: &MemorySource) -> Option<Self> {
        match source {
            MemorySource::Process { pid } => Self::of_process(*pid),
            MemorySource::Container { name } => Self::of_container(name),
        }
    }
}
