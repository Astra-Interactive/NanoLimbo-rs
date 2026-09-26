use crate::play::spawn_position::encode_position;

#[test]
fn given_the_origin_when_packed_then_only_the_height_survives() {
    assert_eq!(encode_position(0, 400, 0), 400);
}

#[test]
fn given_a_coordinate_that_reaches_the_sign_bit_when_packed_then_it_wraps_like_java() {
    // 0x2000000 is bit 25 of X, which the 38-bit shift moves onto bit 63.
    assert_eq!(encode_position(0x200_0000, 0, 0), i64::MIN);
}

#[test]
fn given_negative_coordinates_when_packed_then_each_field_keeps_its_low_bits() {
    let packed = encode_position(-1, -1, -1);

    assert_eq!(packed >> 38, -1);
    assert_eq!((packed << 26) >> 38, -1);
    assert_eq!((packed << 52) >> 52, -1);
}

#[test]
fn given_a_height_wider_than_twelve_bits_when_packed_then_it_is_truncated() {
    assert_eq!(encode_position(0, 0x1000, 0), 0);
}
