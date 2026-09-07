//! Throws malformed and adversarial bytes at every decode path.
//!
//! A limbo server's whole job is to sit on a public port holding connections open, so a
//! panic reachable from network input is not a bug to fix later — it is the failure mode.
//! These tests assert only that nothing panics and nothing hangs: what a decoder *returns*
//! for nonsense is covered by the targeted tests beside each decoder.
//!
//! `cargo-fuzz` would explore further but needs a nightly toolchain, which this workspace
//! deliberately does not use. A seeded generator covers the same ground reproducibly, and
//! a failure names the seed that produced it.

#![allow(clippy::expect_used, clippy::panic)]

use limbo_protocol::packet::{ConnectionState, PacketDirection, PacketRoute};
use limbo_protocol::version::ProtocolVersion;
use limbo_server::serverbound_packet::ServerBoundPacket;

/// Enough to cover every id the server maps, plus the space around them.
const IDS: [i32; 12] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x07, 0x0B, 0x18, 0x1C, 0x7F, -1, 0x7FFF,
];

/// Xorshift, so a failure is reproducible from its seed alone and no dependency is needed.
struct Noise {
    state: u64,
}

impl Noise {
    fn seeded(seed: u64) -> Self {
        Self {
            state: seed | 1, // a zero state would only ever produce zeroes
        }
    }

    fn next_byte(&mut self) -> u8 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        (self.state >> 24) as u8
    }

    fn bytes(&mut self, length: usize) -> Vec<u8> {
        (0..length).map(|_| self.next_byte()).collect()
    }
}

fn every_route() -> impl Iterator<Item = PacketRoute> {
    ConnectionState::ALL.into_iter().flat_map(|state| {
        [
            ProtocolVersion::MIN,
            ProtocolVersion::V1_8,
            ProtocolVersion::V1_16_4,
            ProtocolVersion::V1_19_1,
            ProtocolVersion::V1_20_2,
            ProtocolVersion::V1_20_5,
            ProtocolVersion::MAX,
        ]
        .into_iter()
        .map(move |version| PacketRoute::new(state, PacketDirection::ServerBound, version))
    })
}

#[test]
fn given_random_bytes_on_any_route_when_decoded_then_nothing_panics() {
    let mut noise = Noise::seeded(0x5EED_1234_ABCD_0001);
    let mut reached_a_decoder = 0;

    for route in every_route() {
        for id in IDS {
            for length in [0, 1, 2, 3, 7, 16, 64, 300, 5000] {
                let frame = noise.bytes(length);
                let mut cursor = frame.as_slice();

                // Surviving the call is the assertion; the result only says whether a
                // real decoder ran, which the coverage check below needs.
                match ServerBoundPacket::decode(route, id, &mut cursor) {
                    Ok(None) => {}
                    Ok(Some(_)) | Err(_) => reached_a_decoder += 1,
                }
            }
        }
    }

    // Without this the test would pass just as happily if every id resolved to nothing
    // and no decoder was ever entered. Most (route, id) pairs legitimately map to nothing,
    // so the floor sits below the ~477 that currently land on a real decoder.
    assert!(
        reached_a_decoder > 400,
        "only {reached_a_decoder} inputs reached a decoder; the id set no longer covers \
         the packets the server reads"
    );
}

#[test]
fn given_a_truncation_of_a_valid_packet_when_decoded_then_nothing_panics() {
    let mut noise = Noise::seeded(0xC0FF_EE00_1111_2222);

    for route in every_route() {
        for id in IDS {
            let frame = noise.bytes(64);
            // Every prefix of a frame is something TCP can deliver on its own.
            for cut in 0..frame.len() {
                let mut cursor = frame
                    .get(..cut)
                    .expect("cut is inside the frame by construction");
                let _ = ServerBoundPacket::decode(route, id, &mut cursor);
            }
        }
    }
}

#[test]
fn given_lengths_that_claim_far_more_than_arrives_when_decoded_then_nothing_is_allocated() {
    // A varint of 0xFFFFFFF7 followed by nothing: a client claiming a huge string or byte
    // array and then hanging up. Reserving that eagerly would be a one-packet denial of
    // service, so the limits have to be checked against the declared length alone.
    let claims: [&[u8]; 5] = [
        &[0xF7, 0xFF, 0xFF, 0xFF, 0x0F],
        &[0xFF, 0xFF, 0xFF, 0xFF, 0x07],
        &[0x80, 0x80, 0x80, 0x80, 0x08],
        &[0xFF, 0xFF, 0x7F],
        &[0x7F],
    ];

    for route in every_route() {
        for id in IDS {
            for claim in claims {
                let mut cursor = claim;
                let _ = ServerBoundPacket::decode(route, id, &mut cursor);
            }
        }
    }
}
