use crate::buffer::{PacketDecodeError, ProtocolRead};

fn string_frame(declared_length: i32, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    let mut remaining = declared_length as u32;
    loop {
        if remaining & !0x7F == 0 {
            frame.push(remaining as u8);
            break;
        }
        frame.push((remaining as u8 & 0x7F) | 0x80);
        remaining >>= 7;
    }
    frame.extend_from_slice(payload);
    frame
}

#[test]
fn given_varint_boundary_encodings_when_read_then_the_original_values_come_back() {
    let cases = [
        (vec![0x00], 0_i32),
        (vec![0x7F], 127),
        (vec![0x80, 0x01], 128),
        (vec![0xFF, 0x7F], 16_383),
        (vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07], i32::MAX),
        (vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F], -1),
        (vec![0x80, 0x80, 0x80, 0x80, 0x08], i32::MIN),
    ];

    for (encoded, expected) in cases {
        let mut input = encoded.as_slice();

        assert_eq!(input.read_var_int(), Ok(expected), "{encoded:02X?}");
    }
}

#[test]
fn given_a_varint_needing_a_sixth_byte_when_read_then_it_is_rejected_as_too_wide() {
    let mut input: &[u8] = &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];

    assert_eq!(input.read_var_int(), Err(PacketDecodeError::VarIntTooWide));
}

#[test]
fn given_a_varint_cut_off_mid_sequence_when_read_then_the_shortfall_is_reported() {
    let mut input: &[u8] = &[0x80, 0x80];

    assert_eq!(
        input.read_var_int(),
        Err(PacketDecodeError::UnexpectedEndOfInput {
            needed: 1,
            available: 0
        })
    );
}

#[test]
fn given_a_string_length_beyond_the_byte_limit_when_read_then_nothing_is_allocated() {
    // A one-byte buffer claiming a megabyte. The limit must be enforced from the
    // declared length alone, before the server reserves anything.
    let mut input: &[u8] = &[0x80, 0x89, 0x7A, b'x'];

    assert_eq!(
        input.read_string(16),
        Err(PacketDecodeError::StringTooManyBytes {
            actual: 2_000_000,
            limit: 48
        })
    );
}

#[test]
fn given_characters_outside_the_basic_plane_when_read_then_each_counts_as_two() {
    let payload = "𝄞𝄞𝄞".as_bytes();
    assert_eq!(payload.len(), 12);

    let frame = string_frame(12, payload);

    assert_eq!(
        frame.as_slice().read_string(5),
        Err(PacketDecodeError::StringTooManyChars {
            actual: 6,
            limit: 5
        })
    );
    assert_eq!(frame.as_slice().read_string(6), Ok("𝄞𝄞𝄞".to_owned()));
}

#[test]
fn given_a_negative_string_length_when_read_then_it_is_rejected() {
    let mut input: &[u8] = &[0xFF, 0xFF, 0xFF, 0xFF, 0x0F];

    assert_eq!(
        input.read_string(16),
        Err(PacketDecodeError::NegativeLength { length: -1 })
    );
}

#[test]
fn given_a_string_payload_that_is_not_utf8_when_read_then_it_is_rejected() {
    let frame = string_frame(2, &[0xC3, 0x28]);

    assert_eq!(
        frame.as_slice().read_string(16),
        Err(PacketDecodeError::MalformedUtf8)
    );
}

#[test]
fn given_a_string_shorter_than_declared_when_read_then_the_shortfall_is_reported() {
    let frame = string_frame(10, b"abc");

    assert_eq!(
        frame.as_slice().read_string(16),
        Err(PacketDecodeError::UnexpectedEndOfInput {
            needed: 10,
            available: 3
        })
    );
}

#[test]
fn given_a_byte_array_longer_than_the_limit_when_read_then_it_is_rejected() {
    let frame = string_frame(600, &[0_u8; 600]);

    assert_eq!(
        frame.as_slice().read_byte_array(512),
        Err(PacketDecodeError::ByteArrayTooLong {
            actual: 600,
            limit: 512
        })
    );
    assert_eq!(
        frame
            .as_slice()
            .read_byte_array(1024)
            .map(|bytes| bytes.len()),
        Ok(600)
    );
}

#[test]
fn given_an_empty_buffer_when_fixed_width_values_are_read_then_errors_replace_panics() {
    let empty: &[u8] = &[];
    let one_byte: &[u8] = &[0x00];
    let three_bytes: &[u8] = &[0x00, 0x01, 0x02];
    let seven_bytes: &[u8] = &[0x00_u8; 7];
    let fifteen_bytes: &[u8] = &[0x00_u8; 15];

    assert!({ empty }.read_u8().is_err());
    assert!({ one_byte }.read_u16().is_err());
    assert!({ three_bytes }.read_i32().is_err());
    assert!({ seven_bytes }.read_i64().is_err());
    assert!({ fifteen_bytes }.read_uuid().is_err());
}
