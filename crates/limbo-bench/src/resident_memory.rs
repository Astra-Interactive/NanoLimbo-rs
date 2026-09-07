use std::process::Command;

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
}

/// Parses the resident page count from the second field of `/proc/<pid>/statm`.
fn parse_statm(contents: &str, page_size: u64) -> Option<ResidentMemory> {
    let pages: u64 = contents.split_whitespace().nth(1)?.parse().ok()?;
    Some(ResidentMemory {
        bytes: pages.saturating_mul(page_size),
    })
}

/// Parses the kilobyte figure `ps -o rss=` prints.
fn parse_ps_rss(output: &str) -> Option<ResidentMemory> {
    let kilobytes: u64 = output.trim().parse().ok()?;
    Some(ResidentMemory {
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

/// Reads one process's resident memory, or `None` when it cannot be determined.
///
/// Linux comes straight from procfs. Elsewhere this shells out to `ps`, which is slower
/// but keeps the comparison runnable on a developer's machine rather than only in CI.
pub fn of_process(pid: u32) -> Option<ResidentMemory> {
    const PAGE_SIZE: u64 = 4096;

    if let Some(statm) = read_statm(pid) {
        return parse_statm(&statm, PAGE_SIZE);
    }

    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    parse_ps_rss(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_statm_line_when_parsed_then_the_resident_pages_become_bytes() {
        assert_eq!(
            parse_statm("2048 512 100 1 0 200 0", 4096),
            Some(ResidentMemory { bytes: 512 * 4096 })
        );
    }

    #[test]
    fn given_ps_output_when_parsed_then_its_kilobytes_become_bytes() {
        assert_eq!(
            parse_ps_rss("  119412\n"),
            Some(ResidentMemory {
                bytes: 119_412 * 1024
            })
        );
    }

    #[test]
    fn given_output_that_makes_no_sense_when_parsed_then_nothing_is_reported() {
        assert_eq!(parse_ps_rss(""), None);
        assert_eq!(parse_ps_rss("not a number"), None);
        assert_eq!(parse_statm("2048", 4096), None);
    }
}
