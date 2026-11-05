// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn intervals_telescope_without_microsecond_rounding_or_overlapping_worker_time() {
    let sealed_at = Instant::now();
    let worker_started_at = sealed_at + Duration::from_nanos(1_399);
    let ready_at = worker_started_at + Duration::from_nanos(2_799);
    let collected_at = ready_at + Duration::from_nanos(3_899);
    let timing = ReadyTiming {
        sealed_at,
        worker_started_at,
        ready_at,
    }
    .collected(collected_at);

    assert_eq!(timing.sealed_at(), sealed_at);
    assert_eq!(timing.collected_at(), collected_at);
    assert_eq!(timing.queue(), Duration::from_nanos(1_399));
    assert_eq!(timing.worker(), Duration::from_nanos(2_799));
    assert_eq!(timing.ready_to_collect(), Duration::from_nanos(3_899));
    assert_eq!(
        timing.queue() + timing.worker() + timing.ready_to_collect(),
        timing.seal_to_collect()
    );
    assert_eq!(timing.seal_to_collect(), Duration::from_nanos(8_097));
    assert_ne!(
        timing.queue().as_micros()
            + timing.worker().as_micros()
            + timing.ready_to_collect().as_micros(),
        timing.seal_to_collect().as_micros()
    );
}

#[test]
fn delaying_channel_receipt_changes_only_ready_to_collect() {
    let start = Instant::now();
    let ready = ReadyTiming {
        sealed_at: start,
        worker_started_at: start + Duration::from_micros(3),
        ready_at: start + Duration::from_micros(11),
    };
    let first = ready.collected(start + Duration::from_micros(16));
    let later = ready.collected(start + Duration::from_micros(27));
    assert_eq!(first.queue(), later.queue());
    assert_eq!(first.worker(), later.worker());
    assert_eq!(first.ready_to_collect(), Duration::from_micros(5));
    assert_eq!(later.ready_to_collect(), Duration::from_micros(16));
    assert_eq!(
        later.seal_to_collect() - first.seal_to_collect(),
        Duration::from_micros(11)
    );
}

#[test]
fn zero_length_boundaries_remain_available_observations() {
    let now = Instant::now();
    let timing = ReadyTiming {
        sealed_at: now,
        worker_started_at: now,
        ready_at: now,
    }
    .collected(now);
    assert_eq!(timing.queue(), Duration::ZERO);
    assert_eq!(timing.worker(), Duration::ZERO);
    assert_eq!(timing.ready_to_collect(), Duration::ZERO);
    assert_eq!(timing.seal_to_collect(), Duration::ZERO);
}
