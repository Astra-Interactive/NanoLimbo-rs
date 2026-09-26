use bytes::BytesMut;

use crate::buffer::ProtocolWrite;

fn written<F>(write: F) -> Vec<u8>
where
    F: FnOnce(&mut BytesMut),
{
    let mut buffer = BytesMut::new();
    write(&mut buffer);
    buffer.to_vec()
}

#[test]
fn given_varint_boundary_values_when_written_then_group_encoding_matches_the_protocol() {
    let cases = [
        (0_i32, vec![0x00]),
        (1, vec![0x01]),
        (127, vec![0x7F]),
        (128, vec![0x80, 0x01]),
        (255, vec![0xFF, 0x01]),
        (16_383, vec![0xFF, 0x7F]),
        (16_384, vec![0x80, 0x80, 0x01]),
        (2_097_151, vec![0xFF, 0xFF, 0x7F]),
        (2_097_152, vec![0x80, 0x80, 0x80, 0x01]),
        (268_435_455, vec![0xFF, 0xFF, 0xFF, 0x7F]),
        (268_435_456, vec![0x80, 0x80, 0x80, 0x80, 0x01]),
        (i32::MAX, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07]),
    ];

    for (value, expected) in cases {
        assert_eq!(
            written(|buffer| buffer.write_var_int(value)),
            expected,
            "{value}"
        );
    }
}

#[test]
fn given_a_negative_varint_when_written_then_it_occupies_the_full_five_bytes() {
    assert_eq!(
        written(|buffer| buffer.write_var_int(-1)),
        vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]
    );
    assert_eq!(
        written(|buffer| buffer.write_var_int(i32::MIN)),
        vec![0x80, 0x80, 0x80, 0x80, 0x08]
    );
}

#[test]
fn given_a_multibyte_string_when_written_then_the_prefix_counts_bytes_not_characters() {
    // Four characters, ten UTF-8 bytes: 1 + 2 + 3 + 4.
    assert_eq!(
        written(|buffer| buffer.write_string("aä€𝄞")),
        vec![
            0x0A, b'a', 0xC3, 0xA4, 0xE2, 0x82, 0xAC, 0xF0, 0x9D, 0x84, 0x9E
        ]
    );
}

#[test]
fn given_an_empty_long_array_when_written_then_only_a_zero_length_is_emitted() {
    assert_eq!(written(|buffer| buffer.write_long_array(&[])), vec![0x00]);
}
