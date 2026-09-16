// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Engine phase spans: parse, style, layout, paint, for one frame.
//!
//! T2's wing attribution had to be done by generating a fixture per factor
//! because the engine reported nothing. This is the instrument that replaces
//! that: it is off unless a host turns it on, it costs one relaxed atomic read
//! per span when off, and it reports the four phases a first frame is made of.
//!
//! Spans are **disjoint by construction**, not nested: each instrumentation
//! point wraps one phase of `LiveryDocument::frame` (or, for parse, the host's
//! DOM construction before the document exists). Whatever a frame did outside
//! all four is the host's residual to report, the same discipline Ortet's
//! per-frame `other_us` already follows.
//!
//! Accumulation is per thread. Every instrumented phase runs on the thread
//! that called `frame`, so a host reads back what it measured; a future
//! parallel layout would need per-thread totals joined here rather than a
//! shared counter.

use std::{
    cell::Cell,
    sync::atomic::{AtomicU8, Ordering},
    time::Instant,
};

/// One engine phase of a frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    /// HTML parse and DOM construction, before the document is built.
    Parse,
    /// Cascade, computed-value resolution, container queries, animations.
    Style,
    /// Box generation, formatting, fragment tree, positioned passes.
    Layout,
    /// Paint-list production from the resolved layout.
    Paint,
}

impl Phase {
    const COUNT: usize = 4;

    const fn slot(self) -> usize {
        match self {
            Self::Parse => 0,
            Self::Style => 1,
            Self::Layout => 2,
            Self::Paint => 3,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Style => "style",
            Self::Layout => "layout",
            Self::Paint => "paint",
        }
    }

    pub const ALL: [Self; Self::COUNT] = [Self::Parse, Self::Style, Self::Layout, Self::Paint];
}

/// 0 unresolved, 1 off, 2 on.
static STATE: AtomicU8 = AtomicU8::new(0);

/// The environment variable a host can set instead of calling [`enable`].
pub const ENV_FLAG: &str = "GENET_PHASE_TIMING";

thread_local! {
    static TOTALS: [Cell<u64>; Phase::COUNT] = const {
        [Cell::new(0), Cell::new(0), Cell::new(0), Cell::new(0)]
    };
}

/// Turn phase spans on for this process. Idempotent.
pub fn enable() {
    STATE.store(2, Ordering::Relaxed);
}

/// Whether spans are recording. Resolved once from [`ENV_FLAG`]; any value
/// other than empty or `0` turns them on.
pub fn enabled() -> bool {
    match STATE.load(Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => {
            let on = std::env::var(ENV_FLAG)
                .map(|value| !value.is_empty() && value != "0")
                .unwrap_or(false);
            STATE.store(if on { 2 } else { 1 }, Ordering::Relaxed);
            on
        },
    }
}

/// Start a span, or nothing at all when spans are off.
pub fn span(phase: Phase) -> Option<Span> {
    enabled().then(|| Span {
        phase,
        start: Instant::now(),
    })
}

/// A running phase span. Its elapsed microseconds are added on drop.
#[derive(Debug)]
pub struct Span {
    phase: Phase,
    start: Instant,
}

impl Drop for Span {
    fn drop(&mut self) {
        let micros = u64::try_from(self.start.elapsed().as_micros()).unwrap_or(u64::MAX);
        TOTALS.with(|totals| {
            let slot = &totals[self.phase.slot()];
            slot.set(slot.get().saturating_add(micros));
        });
    }
}

/// Read the accumulated microseconds per phase and reset them, so the next
/// frame's totals stand alone.
pub fn take() -> [u64; Phase::COUNT] {
    TOTALS.with(|totals| {
        let mut out = [0; Phase::COUNT];
        for (slot, total) in out.iter_mut().zip(totals) {
            *slot = total.replace(0);
        }
        out
    })
}

/// Read the accumulated microseconds per phase without resetting them.
pub fn peek() -> [u64; Phase::COUNT] {
    TOTALS.with(|totals| {
        let mut out = [0; Phase::COUNT];
        for (slot, total) in out.iter_mut().zip(totals) {
            *slot = total.get();
        }
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `STATE` is process-wide, so the two tests that set it take turns.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// The recorder is off by default, so an uninstrumented host pays nothing
    /// and reads nothing.
    #[test]
    fn a_disabled_recorder_records_nothing() {
        let _guard = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
        STATE.store(1, Ordering::Relaxed);
        let _ = take();
        assert!(span(Phase::Layout).is_none());
        assert_eq!(take(), [0; Phase::COUNT]);
    }

    /// Each phase accumulates into its own slot, and `take` drains.
    #[test]
    fn enabled_spans_accumulate_per_phase_and_drain() {
        let _guard = SERIAL.lock().unwrap_or_else(|error| error.into_inner());
        enable();
        let _ = take();
        {
            let _style = span(Phase::Style);
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        {
            let _layout = span(Phase::Layout);
            std::thread::sleep(std::time::Duration::from_millis(4));
        }
        {
            let _layout = span(Phase::Layout);
            std::thread::sleep(std::time::Duration::from_millis(4));
        }
        let totals = peek();
        assert_eq!(totals[Phase::Parse.slot()], 0);
        assert!(totals[Phase::Style.slot()] >= 1_000, "{totals:?}");
        assert!(
            totals[Phase::Layout.slot()] > totals[Phase::Style.slot()],
            "two layout spans sum: {totals:?}"
        );
        assert_eq!(take(), totals);
        assert_eq!(take(), [0; Phase::COUNT]);
        STATE.store(1, Ordering::Relaxed);
    }

    #[test]
    fn phase_names_are_stable_receipt_keys() {
        assert_eq!(
            Phase::ALL.map(Phase::name),
            ["parse", "style", "layout", "paint"]
        );
    }
}
