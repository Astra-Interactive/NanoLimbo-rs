use crate::console::MemoryUsage;

#[test]
fn given_a_statm_line_when_parsed_then_the_resident_pages_are_converted_to_bytes() {
    assert_eq!(
        MemoryUsage::parse_statm("2048 512 100 1 0 200 0", 4096),
        Some(MemoryUsage {
            resident_bytes: 512 * 4096
        })
    );
}

#[test]
fn given_a_statm_line_that_makes_no_sense_when_parsed_then_nothing_is_reported() {
    assert_eq!(MemoryUsage::parse_statm("", 4096), None);
    assert_eq!(MemoryUsage::parse_statm("2048", 4096), None);
    assert_eq!(MemoryUsage::parse_statm("2048 notanumber", 4096), None);
}

#[test]
fn given_a_page_count_that_would_overflow_when_converted_then_it_saturates() {
    let usage = MemoryUsage::parse_statm(&format!("1 {}", u64::MAX), 4096);

    assert_eq!(
        usage,
        Some(MemoryUsage {
            resident_bytes: u64::MAX
        })
    );
}
