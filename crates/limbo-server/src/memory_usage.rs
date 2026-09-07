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
    pub const fn resident_megabytes(self) -> u64 {
        self.resident_bytes / (1024 * 1024)
    }
}

/// Size of a memory page, which `/proc/self/statm` counts in.
const PAGE_SIZE: u64 = 4096;

/// Parses the resident page count out of the second field of `/proc/self/statm`.
fn parse_statm(contents: &str, page_size: u64) -> Option<MemoryUsage> {
    let pages: u64 = contents.split_whitespace().nth(1)?.parse().ok()?;
    Some(MemoryUsage {
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
pub fn current() -> Option<MemoryUsage> {
    parse_statm(&read_statm()?, PAGE_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_statm_line_when_parsed_then_the_resident_pages_are_converted_to_bytes() {
        assert_eq!(
            parse_statm("2048 512 100 1 0 200 0", 4096),
            Some(MemoryUsage {
                resident_bytes: 512 * 4096
            })
        );
    }

    #[test]
    fn given_a_statm_line_that_makes_no_sense_when_parsed_then_nothing_is_reported() {
        assert_eq!(parse_statm("", 4096), None);
        assert_eq!(parse_statm("2048", 4096), None);
        assert_eq!(parse_statm("2048 notanumber", 4096), None);
    }

    #[test]
    fn given_a_page_count_that_would_overflow_when_converted_then_it_saturates() {
        let usage = parse_statm(&format!("1 {}", u64::MAX), 4096);

        assert_eq!(
            usage,
            Some(MemoryUsage {
                resident_bytes: u64::MAX
            })
        );
    }
}
