use limbo_protocol::version::ProtocolVersion;

use crate::play::chunk_with_light::{empty_section, filled_bit_set};

#[test]
fn given_a_mask_shorter_than_a_word_when_built_then_one_partly_filled_word_is_emitted() {
    assert_eq!(filled_bit_set(18), vec![0x3_FFFF]);
}

#[test]
fn given_a_mask_of_exactly_one_word_when_built_then_no_empty_word_is_appended() {
    assert_eq!(filled_bit_set(64), vec![-1]);
}

#[test]
fn given_a_mask_spanning_two_words_when_built_then_the_high_word_holds_the_remainder() {
    assert_eq!(filled_bit_set(66), vec![-1, 0b11]);
}

#[test]
fn given_no_bits_when_built_then_the_mask_is_empty() {
    assert_eq!(filled_bit_set(0), Vec::<i64>::new());
}

#[test]
fn given_the_release_that_dropped_the_storage_length_when_a_section_is_built_then_it_shrinks() {
    assert_eq!(empty_section(ProtocolVersion::V1_21_4).len(), 8);
    assert_eq!(empty_section(ProtocolVersion::V1_21_5).len(), 6);
    assert_eq!(empty_section(ProtocolVersion::V26_1).len(), 8);
}
