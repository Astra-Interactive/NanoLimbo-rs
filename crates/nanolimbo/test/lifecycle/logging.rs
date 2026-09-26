use tracing::Level;

use crate::lifecycle::level_from_debug_setting;

#[test]
fn given_a_documented_debug_level_when_translated_then_it_matches_the_published_meaning() {
    assert_eq!(level_from_debug_setting(0), Level::ERROR);
    assert_eq!(level_from_debug_setting(1), Level::WARN);
    assert_eq!(level_from_debug_setting(2), Level::INFO);
    assert_eq!(level_from_debug_setting(3), Level::DEBUG);
}

#[test]
fn given_a_level_outside_the_documented_range_when_translated_then_it_clamps() {
    assert_eq!(level_from_debug_setting(-7), Level::ERROR);
    assert_eq!(level_from_debug_setting(99), Level::DEBUG);
}
