use std::time::{Duration, Instant};

use crate::traffic::{PacketBucket, TrafficSample};

const WINDOW: Duration = Duration::from_millis(150);
const RESOLUTION: Duration = Duration::from_millis(1);

fn packet_of(bytes: usize) -> TrafficSample {
    TrafficSample::single_packet(bytes)
}

#[test]
fn given_packets_inside_one_bucket_when_recorded_then_the_window_holds_them_all() {
    let start = Instant::now();
    let mut bucket = PacketBucket::new(WINDOW);

    bucket.record(packet_of(10), start);
    bucket.record(packet_of(20), start + RESOLUTION / 2);

    assert_eq!(
        bucket.totals(),
        TrafficSample {
            packets: 2,
            bytes: 30
        }
    );
}

#[test]
fn given_a_bucket_boundary_is_crossed_when_recorded_then_both_buckets_still_count() {
    let start = Instant::now();
    let mut bucket = PacketBucket::new(WINDOW);

    bucket.record(packet_of(10), start);
    bucket.record(packet_of(20), start + RESOLUTION);

    assert_eq!(
        bucket.totals(),
        TrafficSample {
            packets: 2,
            bytes: 30
        }
    );
}

#[test]
fn given_a_sample_reaches_the_far_end_of_the_window_when_it_slides_out_then_it_stops_counting() {
    let start = Instant::now();
    let mut bucket = PacketBucket::new(WINDOW);

    bucket.record(packet_of(100), start);
    bucket.record(packet_of(10), start + WINDOW - RESOLUTION);
    let before_sliding_out = bucket.totals();
    bucket.record(packet_of(1), start + WINDOW);

    assert_eq!(
        before_sliding_out,
        TrafficSample {
            packets: 2,
            bytes: 110
        }
    );
    assert_eq!(
        bucket.totals(),
        TrafficSample {
            packets: 2,
            bytes: 11
        }
    );
}

#[test]
fn given_a_gap_longer_than_the_window_when_recorded_then_only_the_new_packet_counts() {
    let start = Instant::now();
    let mut bucket = PacketBucket::new(WINDOW);

    bucket.record(packet_of(100), start);
    bucket.record(packet_of(7), start + WINDOW);

    assert_eq!(
        bucket.totals(),
        TrafficSample {
            packets: 1,
            bytes: 7
        }
    );
}

#[test]
fn given_buckets_are_skipped_when_the_window_moves_then_the_skipped_ones_are_empty() {
    let start = Instant::now();
    let mut bucket = PacketBucket::new(WINDOW);

    bucket.record(packet_of(100), start);
    bucket.record(packet_of(1), start + RESOLUTION * 10);
    bucket.record(packet_of(1), start + WINDOW + RESOLUTION * 9);

    // The first packet has aged out; the second is still one bucket short of the
    // far end of the window.
    assert_eq!(
        bucket.totals(),
        TrafficSample {
            packets: 2,
            bytes: 2
        }
    );
}

#[test]
fn given_a_clock_that_runs_backwards_when_recorded_then_the_packet_joins_the_newest_bucket() {
    let start = Instant::now() + WINDOW;
    let mut bucket = PacketBucket::new(WINDOW);

    bucket.record(packet_of(10), start);
    bucket.record(packet_of(20), start - RESOLUTION * 5);

    assert_eq!(
        bucket.totals(),
        TrafficSample {
            packets: 2,
            bytes: 30
        }
    );
}

#[test]
fn given_a_window_of_zero_when_time_passes_then_earlier_packets_are_forgotten() {
    let start = Instant::now();
    let mut bucket = PacketBucket::new(Duration::ZERO);

    bucket.record(packet_of(10), start);
    bucket.record(packet_of(20), start + Duration::from_nanos(150));

    assert_eq!(
        bucket.totals(),
        TrafficSample {
            packets: 1,
            bytes: 20
        }
    );
}
