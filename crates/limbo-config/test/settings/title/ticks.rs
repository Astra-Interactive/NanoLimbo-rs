use std::time::Duration;

use crate::settings::title::Ticks;

#[test]
fn given_one_second_of_ticks_when_converted_then_it_is_one_second() {
    assert_eq!(Ticks::new(20).as_duration(), Some(Duration::from_secs(1)));
}

#[test]
fn given_a_single_tick_when_converted_then_it_is_fifty_milliseconds() {
    assert_eq!(Ticks::new(1).as_duration(), Some(Duration::from_millis(50)));
}

#[test]
fn given_a_negative_count_when_converted_then_there_is_no_duration() {
    assert_eq!(Ticks::new(-1).as_duration(), None);
    assert_eq!(Ticks::new(-1).count(), -1);
}
