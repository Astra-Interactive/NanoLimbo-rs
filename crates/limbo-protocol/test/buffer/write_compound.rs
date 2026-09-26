use bytes::BytesMut;
use valence_nbt::binary::from_binary;
use valence_nbt::{Compound, List, Value, compound};

use crate::buffer::write_compound;
use crate::buffer::write_compound::NAMED_ROOT_HEADER_LEN;
use crate::version::ProtocolVersion;

const ROOT_COMPOUND_TAG: u8 = 0x0A;

fn sample_compound() -> Compound {
    compound! {
        "height" => 384_i32,
        "name" => "minecraft:overworld",
        "nested" => compound! { "ambient_light" => 0.0_f32 },
        "ids" => List::Int(vec![1, 2, 3]),
        "empty" => List::End,
    }
}

fn written(version: ProtocolVersion) -> Vec<u8> {
    let mut buffer = BytesMut::new();
    write_compound(&mut buffer, &sample_compound(), version).expect("encoding must succeed");

    buffer.to_vec()
}

#[test]
fn given_a_client_below_1_20_2_when_a_compound_is_written_then_it_carries_an_empty_root_name() {
    let named = written(ProtocolVersion::V1_20);

    assert_eq!(
        named.get(..NAMED_ROOT_HEADER_LEN),
        Some([ROOT_COMPOUND_TAG, 0x00, 0x00].as_slice())
    );
}

#[test]
fn given_a_client_from_1_20_2_when_a_compound_is_written_then_only_the_name_length_is_dropped() {
    let named = written(ProtocolVersion::V1_20);
    let nameless = written(ProtocolVersion::V1_20_2);

    assert_eq!(nameless.first(), Some(&ROOT_COMPOUND_TAG));
    assert_eq!(nameless.get(1..), named.get(NAMED_ROOT_HEADER_LEN..));
    assert_eq!(nameless.len() + 2, named.len());
}

#[test]
fn given_the_named_root_shape_when_it_is_read_back_then_the_compound_is_unchanged() {
    let named = written(ProtocolVersion::V1_20);

    let (compound, root_name): (Compound, String) =
        from_binary(&mut named.as_slice()).expect("the named form must decode");

    assert_eq!(root_name, "");
    assert_eq!(compound, sample_compound());
}

#[test]
fn given_a_compound_with_a_nested_list_when_written_then_the_value_survives_the_round_trip() {
    let named = written(ProtocolVersion::V1_19);

    let (compound, _root_name): (Compound, String) =
        from_binary(&mut named.as_slice()).expect("the named form must decode");

    assert_eq!(
        compound.get("ids"),
        Some(&Value::List(List::Int(vec![1, 2, 3])))
    );
}

#[test]
fn given_a_compound_when_written_twice_into_one_buffer_then_the_second_copy_is_appended() {
    let mut buffer = BytesMut::new();
    let compound = sample_compound();

    write_compound(&mut buffer, &compound, ProtocolVersion::V1_21).expect("first write");
    let length_after_first = buffer.len();
    write_compound(&mut buffer, &compound, ProtocolVersion::V1_21).expect("second write");

    assert_eq!(buffer.len(), length_after_first * 2);
}

#[test]
fn given_an_empty_compound_when_written_nameless_then_only_the_tag_and_terminator_remain() {
    let mut buffer = BytesMut::new();

    write_compound(&mut buffer, &Compound::new(), ProtocolVersion::V1_21).expect("encoding");

    assert_eq!(buffer.to_vec(), vec![ROOT_COMPOUND_TAG, 0x00]);
}
