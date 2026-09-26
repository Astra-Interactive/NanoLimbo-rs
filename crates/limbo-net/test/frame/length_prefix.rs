use crate::frame::FrameError;
use crate::frame::length_prefix::{LengthPrefix, MAX_FRAME_LENGTH};

#[test]
fn given_single_byte_prefix_when_read_then_reports_one_consumed_byte() {
    let prefix = LengthPrefix::read(&[0x7F, 0xAA]).unwrap().unwrap();

    assert_eq!(
        prefix,
        LengthPrefix {
            payload_length: 127,
            prefix_length: 1,
        }
    );
}

#[test]
fn given_widest_legal_prefix_when_read_then_yields_the_21_bit_maximum() {
    let prefix = LengthPrefix::read(&[0xFF, 0xFF, 0x7F]).unwrap().unwrap();

    assert_eq!(
        prefix,
        LengthPrefix {
            payload_length: MAX_FRAME_LENGTH,
            prefix_length: 3,
        }
    );
}

#[test]
fn given_prefix_cut_short_by_the_stream_when_read_then_reports_incomplete() {
    assert_eq!(LengthPrefix::read(&[]).unwrap(), None);
    assert_eq!(LengthPrefix::read(&[0x80]).unwrap(), None);
    assert_eq!(LengthPrefix::read(&[0x80, 0x80]).unwrap(), None);
}

#[test]
fn given_fourth_continuation_byte_when_read_then_rejects_the_prefix() {
    let error = LengthPrefix::read(&[0x80, 0x80, 0x80, 0x01]).unwrap_err();

    assert!(matches!(error, FrameError::LengthPrefixTooWide));
}

#[test]
fn given_three_continuation_bytes_and_nothing_else_when_read_then_still_rejects() {
    // The Java slow path silently truncated this to a 21 bit value; a prefix that
    // long cannot describe a legal frame however many bytes follow it.
    let error = LengthPrefix::read(&[0x80, 0x80, 0x80]).unwrap_err();

    assert!(matches!(error, FrameError::LengthPrefixTooWide));
}
