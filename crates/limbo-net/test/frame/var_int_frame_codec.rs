use bytes::{BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::frame::length_prefix::MAX_FRAME_LENGTH;
use crate::frame::{FrameError, VarIntFrameCodec};

fn encoded(payloads: &[&[u8]]) -> BytesMut {
    let mut codec = VarIntFrameCodec;
    let mut buffer = BytesMut::new();

    for payload in payloads {
        codec
            .encode(Bytes::copy_from_slice(payload), &mut buffer)
            .expect("payloads are within the protocol maximum");
    }

    buffer
}

fn drain(buffer: &mut BytesMut) -> Vec<Bytes> {
    let mut codec = VarIntFrameCodec;
    let mut frames = Vec::new();

    while let Some(frame) = codec.decode(buffer).expect("stream is well formed") {
        frames.push(frame);
    }

    frames
}

/// Payload lengths that need a one, two and three byte prefix respectively.
fn sample_payloads() -> Vec<Vec<u8>> {
    vec![
        vec![0x2A],
        (0..200_u32).map(|index| index as u8).collect(),
        (0..20_000_u32).map(|index| index as u8).collect(),
    ]
}

#[test]
fn given_encoded_frames_when_decoded_then_every_payload_round_trips() {
    let payloads = sample_payloads();
    let borrowed: Vec<&[u8]> = payloads.iter().map(Vec::as_slice).collect();
    let mut buffer = encoded(&borrowed);

    let frames = drain(&mut buffer);

    assert_eq!(frames, payloads);
    assert!(buffer.is_empty());
}

#[test]
fn given_stream_split_into_single_bytes_when_decoded_then_frames_match_unsplit_decode() {
    let payloads = sample_payloads();
    let borrowed: Vec<&[u8]> = payloads.iter().map(Vec::as_slice).collect();
    let stream = encoded(&borrowed);
    let unsplit = drain(&mut stream.clone());

    let mut codec = VarIntFrameCodec;
    let mut buffer = BytesMut::new();
    let mut fragmented = Vec::new();
    for byte in stream.iter() {
        buffer.put_u8(*byte);
        while let Some(frame) = codec.decode(&mut buffer).expect("stream is well formed") {
            fragmented.push(frame);
        }
    }

    assert_eq!(fragmented, unsplit);
    assert!(buffer.is_empty());
}

#[test]
fn given_leading_nul_padding_when_decoded_then_the_frame_still_arrives() {
    let mut buffer = BytesMut::from(&[0x00, 0x00, 0x00][..]);
    buffer.extend_from_slice(&encoded(&[b"hi"]));

    let frames = drain(&mut buffer);

    assert_eq!(frames, vec![Bytes::from_static(b"hi")]);
}

#[test]
fn given_only_nul_bytes_when_decoded_then_the_buffer_is_drained() {
    let mut buffer = BytesMut::from(&[0x00; 8][..]);

    let frames = drain(&mut buffer);

    assert!(frames.is_empty());
    assert!(buffer.is_empty());
}

#[test]
fn given_over_long_zero_length_prefix_when_decoded_then_the_next_frame_still_arrives() {
    let mut buffer = BytesMut::from(&[0x80, 0x00][..]);
    buffer.extend_from_slice(&encoded(&[b"hi"]));

    let frames = drain(&mut buffer);

    assert_eq!(frames, vec![Bytes::from_static(b"hi")]);
    assert!(buffer.is_empty());
}

#[test]
fn given_incomplete_prefix_when_decoded_then_the_buffer_is_left_untouched() {
    let mut codec = VarIntFrameCodec;
    let mut buffer = BytesMut::from(&[0x80, 0x80][..]);

    let frame = codec
        .decode(&mut buffer)
        .expect("prefix may still complete");

    assert!(frame.is_none());
    assert_eq!(buffer.as_ref(), &[0x80, 0x80]);
}

#[test]
fn given_incomplete_payload_when_decoded_then_the_buffer_is_left_untouched() {
    let mut codec = VarIntFrameCodec;
    let mut buffer = BytesMut::from(&[0x04, 0x01, 0x02][..]);

    let frame = codec.decode(&mut buffer).expect("payload may still arrive");

    assert!(frame.is_none());
    assert_eq!(buffer.as_ref(), &[0x04, 0x01, 0x02]);
}

#[test]
fn given_prefix_wider_than_21_bits_when_decoded_then_the_stream_is_rejected() {
    let mut codec = VarIntFrameCodec;
    let mut buffer = BytesMut::from(&[0xFF, 0xFF, 0xFF, 0x7F][..]);

    let error = codec.decode(&mut buffer).unwrap_err();

    assert!(matches!(error, FrameError::LengthPrefixTooWide));
}

#[test]
fn given_empty_frame_when_encoded_then_the_decoder_reads_it_as_padding() {
    let mut buffer = encoded(&[b"", b"after"]);

    let frames = drain(&mut buffer);

    assert_eq!(frames, vec![Bytes::from_static(b"after")]);
}

#[test]
fn given_frame_beyond_the_protocol_maximum_when_encoded_then_it_is_rejected() {
    let mut codec = VarIntFrameCodec;
    let oversized = Bytes::from(vec![0_u8; MAX_FRAME_LENGTH + 1]);

    let error = codec
        .encode(oversized, &mut BytesMut::new())
        .expect_err("no client can read a frame this large");

    assert!(matches!(
        error,
        FrameError::FrameTooLarge {
            length,
            limit: MAX_FRAME_LENGTH
        } if length == MAX_FRAME_LENGTH + 1
    ));
}

#[test]
fn given_the_largest_legal_frame_when_encoded_then_it_round_trips() {
    let payload = vec![0x5A_u8; MAX_FRAME_LENGTH];
    let mut buffer = encoded(&[&payload]);

    let frames = drain(&mut buffer);

    assert_eq!(frames, vec![Bytes::from(payload)]);
}
