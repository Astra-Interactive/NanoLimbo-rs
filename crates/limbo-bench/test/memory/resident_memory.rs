use crate::memory::ResidentMemory;

#[test]
fn given_a_statm_line_when_parsed_then_the_resident_pages_become_bytes() {
    assert_eq!(
        ResidentMemory::parse_statm("2048 512 100 1 0 200 0", 4096),
        Some(ResidentMemory { bytes: 512 * 4096 })
    );
}

#[test]
fn given_ps_output_when_parsed_then_its_kilobytes_become_bytes() {
    assert_eq!(
        ResidentMemory::parse_ps_rss("  119412\n"),
        Some(ResidentMemory {
            bytes: 119_412 * 1024
        })
    );
}

#[test]
fn given_docker_stats_output_when_parsed_then_the_used_half_becomes_bytes() {
    assert_eq!(
        ResidentMemory::parse_docker_usage("4.094MiB / 7.652GiB\n"),
        Some(ResidentMemory {
            bytes: (4.094 * 1024.0 * 1024.0) as u64
        })
    );
    assert_eq!(
        ResidentMemory::parse_docker_usage("201.1MiB / 7.652GiB"),
        Some(ResidentMemory {
            bytes: (201.1 * 1024.0 * 1024.0) as u64
        })
    );
}

#[test]
fn given_output_that_makes_no_sense_when_parsed_then_nothing_is_reported() {
    assert_eq!(ResidentMemory::parse_docker_usage(""), None);
    assert_eq!(ResidentMemory::parse_docker_usage("lots / 7GiB"), None);
    assert_eq!(ResidentMemory::parse_docker_usage("12 / 7GiB"), None);
    assert_eq!(ResidentMemory::parse_ps_rss(""), None);
    assert_eq!(ResidentMemory::parse_ps_rss("not a number"), None);
    assert_eq!(ResidentMemory::parse_statm("2048", 4096), None);
}
