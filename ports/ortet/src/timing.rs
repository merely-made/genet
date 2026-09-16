// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Per-frame phase attribution, with **actual work kept apart from
//! presentation wait**.
//!
//! The T3 mutation instrument's whole point is that "12 ms per frame" says
//! nothing until you know whether the thread spent it laying the document out
//! or blocked on the compositor. So every frame records disjoint spans and the
//! two totals are derived, never measured separately:
//!
//! - **work** — host mutation, `session.frame` (style, layout, paint list),
//!   rasterization, and the compose encode. The thread is doing something.
//! - **wait** — swapchain acquire and present. The thread is blocked on the
//!   display, and a present-mode change moves this number without any engine
//!   change at all.
//!
//! Everything else the frame did (pump, accessibility, receipt checks) is the
//! residual `other`, reported rather than hidden inside either total.

use std::time::{Duration, Instant};

/// Microseconds, saturating. Phase spans are small; a `u64` of microseconds is
/// the same unit Cambium's `FrameProfile` reports, so receipts compare.
pub fn elapsed_us(span: Duration) -> u64 {
    u64::try_from(span.as_micros()).unwrap_or(u64::MAX)
}

/// A restartable stopwatch for one phase.
pub struct Phase(Instant);

impl Phase {
    pub fn start() -> Self {
        Self(Instant::now())
    }

    /// Microseconds since [`Self::start`], and rearm for the next phase.
    pub fn lap(&mut self) -> u64 {
        let now = Instant::now();
        let span = elapsed_us(now.duration_since(self.0));
        self.0 = now;
        span
    }

    /// Microseconds since the last lap, without rearming.
    pub fn read(&self) -> u64 {
        elapsed_us(self.0.elapsed())
    }
}

/// One presented frame's attribution.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameTiming {
    /// Zero-based index among presented frames.
    pub index: u32,
    // ── work ───────────────────────────────────────────────────────────────
    /// Applying the host mutation batch: DOM change, drain, restyle.
    pub mutate_us: u64,
    /// `DocumentSession::frame` — style, layout and paint list.
    pub frame_us: u64,
    /// Scene rasterization into the owned target.
    pub raster_us: u64,
    /// Encoding the composite of that target onto the backbuffer.
    pub compose_us: u64,
    // ── presentation wait ──────────────────────────────────────────────────
    /// Blocking on a swapchain image.
    pub acquire_us: u64,
    /// Handing the frame to the presentation engine.
    pub present_us: u64,
    // ── envelope ───────────────────────────────────────────────────────────
    /// The whole render call, wall clock.
    pub total_us: u64,
    /// Host mutations applied this frame.
    pub mutations: u32,
    /// Elements the engine restyled for them.
    pub restyled: u32,
    // -- engine phase spans -------------------------------------------------
    /// HTML parse and DOM construction. It happens once, before the first
    /// frame, and is drained onto whichever frame first reads the recorder.
    pub parse_us: u64,
    /// Cascade and computed-value resolution inside `frame_us`.
    pub style_us: u64,
    /// Box generation, formatting and the fragment tree, inside `frame_us`.
    pub layout_us: u64,
    /// Paint-list production, inside `frame_us`.
    pub paint_us: u64,
}

impl FrameTiming {
    /// Actual work: everything the thread computed.
    pub const fn work_us(&self) -> u64 {
        self.mutate_us + self.frame_us + self.raster_us + self.compose_us
    }

    /// Presentation wait: everything the thread blocked on.
    pub const fn wait_us(&self) -> u64 {
        self.acquire_us + self.present_us
    }

    /// The part of the frame neither total claims (pump, accessibility,
    /// receipt conditions). Reported, not absorbed.
    pub const fn other_us(&self) -> u64 {
        self.total_us
            .saturating_sub(self.work_us() + self.wait_us())
    }

    /// The engine phases that live inside `frame_us`. Parse is excluded: it
    /// runs before the frame loop, so it is not part of any frame's span.
    pub const fn engine_phases_us(&self) -> u64 {
        self.style_us + self.layout_us + self.paint_us
    }

    /// What `session.frame` spent outside the three instrumented phases:
    /// child-frame composition, scene translation, overlays. Reported so the
    /// attribution never quietly absorbs it.
    pub const fn frame_other_us(&self) -> u64 {
        self.frame_us.saturating_sub(self.engine_phases_us())
    }

    /// Whether the engine phase instrument contributed anything to this frame.
    pub const fn has_phases(&self) -> bool {
        self.parse_us + self.engine_phases_us() > 0
    }

    fn json(&self) -> String {
        format!(
            "{{\"frame\":{},\"mutate_us\":{},\"frame_us\":{},\"raster_us\":{},\"compose_us\":{},\
             \"acquire_us\":{},\"present_us\":{},\"work_us\":{},\"wait_us\":{},\"other_us\":{},\
             \"total_us\":{},\"mutations\":{},\"restyled\":{},\
             \"parse_us\":{},\"style_us\":{},\"layout_us\":{},\"paint_us\":{},\
             \"frame_other_us\":{}}}",
            self.index,
            self.mutate_us,
            self.frame_us,
            self.raster_us,
            self.compose_us,
            self.acquire_us,
            self.present_us,
            self.work_us(),
            self.wait_us(),
            self.other_us(),
            self.total_us,
            self.mutations,
            self.restyled,
            self.parse_us,
            self.style_us,
            self.layout_us,
            self.paint_us,
            self.frame_other_us(),
        )
    }
}

/// Median and 95th percentile of one measure across the recorded frames.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Percentiles {
    pub median_us: u64,
    pub p95_us: u64,
}

/// What a run has to say about its frames.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimingSummary {
    pub frames: u32,
    pub work: Percentiles,
    pub wait: Percentiles,
    pub total: Percentiles,
    pub frame_phase: Percentiles,
    pub raster: Percentiles,
    pub mutate: Percentiles,
    pub mutations: u64,
    pub restyled: u64,
    /// Engine phase spans, summed over the measured frames. Sums rather than
    /// percentiles: on a one-frame first-frame run there is nothing to take a
    /// percentile of, and parse lands on exactly one frame.
    pub parse_us: u64,
    pub style_us: u64,
    pub layout_us: u64,
    pub paint_us: u64,
}

/// Every presented frame's attribution, and the summary over them.
#[derive(Clone, Debug, Default)]
pub struct TimingLog {
    frames: Vec<FrameTiming>,
    /// Frames excluded from the summary. The first frame of a Genet document is
    /// a parse/style/layout cold start measured by lane T2, not a steady
    /// mutation cost, so a mutation summary that averaged it in would be
    /// reporting T2's number.
    warmup: u32,
}

impl TimingLog {
    pub fn new(warmup: u32) -> Self {
        Self {
            frames: Vec::new(),
            warmup,
        }
    }

    pub fn push(&mut self, timing: FrameTiming) {
        self.frames.push(timing);
    }

    pub fn frames(&self) -> &[FrameTiming] {
        &self.frames
    }

    /// The frames the summary is taken over: everything past the warm-up, or
    /// — for a run too short to have any — every frame, so a short run reports
    /// honestly instead of reporting nothing.
    pub fn measured(&self) -> &[FrameTiming] {
        let skip = self.warmup as usize;
        if self.frames.len() > skip {
            &self.frames[skip..]
        } else {
            &self.frames
        }
    }

    pub fn summary(&self) -> TimingSummary {
        let measured = self.measured();
        TimingSummary {
            frames: measured.len() as u32,
            work: percentiles(measured, FrameTiming::work_us),
            wait: percentiles(measured, FrameTiming::wait_us),
            total: percentiles(measured, |frame| frame.total_us),
            frame_phase: percentiles(measured, |frame| frame.frame_us),
            raster: percentiles(measured, |frame| frame.raster_us),
            mutate: percentiles(measured, |frame| frame.mutate_us),
            mutations: measured
                .iter()
                .map(|frame| u64::from(frame.mutations))
                .sum(),
            restyled: measured.iter().map(|frame| u64::from(frame.restyled)).sum(),
            parse_us: measured.iter().map(|frame| frame.parse_us).sum(),
            style_us: measured.iter().map(|frame| frame.style_us).sum(),
            layout_us: measured.iter().map(|frame| frame.layout_us).sum(),
            paint_us: measured.iter().map(|frame| frame.paint_us).sum(),
        }
    }

    /// The receipt: every frame plus the summary, as JSON with no serializer
    /// dependency (Ortet writes RGBA8 and this; it reads nothing back).
    pub fn to_json(&self, header: &[(&str, String)]) -> String {
        let summary = self.summary();
        let mut out = String::from("{\n");
        for (key, value) in header {
            out.push_str(&format!("  {}: {},\n", json_string(key), value));
        }
        out.push_str(&format!("  \"warmup_frames\": {},\n", self.warmup));
        out.push_str(&format!(
            "  \"summary\": {{\"frames\": {}, \"mutations\": {}, \"restyled\": {}, \
             \"work_median_us\": {}, \"work_p95_us\": {}, \"wait_median_us\": {}, \
             \"wait_p95_us\": {}, \"total_median_us\": {}, \"total_p95_us\": {}, \
             \"frame_phase_median_us\": {}, \"raster_median_us\": {}, \"mutate_median_us\": {}, \
             \"parse_us\": {}, \"style_us\": {}, \"layout_us\": {}, \"paint_us\": {}}},\n",
            summary.frames,
            summary.mutations,
            summary.restyled,
            summary.work.median_us,
            summary.work.p95_us,
            summary.wait.median_us,
            summary.wait.p95_us,
            summary.total.median_us,
            summary.total.p95_us,
            summary.frame_phase.median_us,
            summary.raster.median_us,
            summary.mutate.median_us,
            summary.parse_us,
            summary.style_us,
            summary.layout_us,
            summary.paint_us,
        ));
        out.push_str("  \"frames\": [\n");
        for (index, frame) in self.frames.iter().enumerate() {
            out.push_str("    ");
            out.push_str(&frame.json());
            if index + 1 < self.frames.len() {
                out.push(',');
            }
            out.push('\n');
        }
        out.push_str("  ]\n}\n");
        out
    }
}

/// A JSON string literal with the four escapes a path or a URL can contain.
pub fn json_string(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 2);
    out.push('"');
    for character in raw.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other if (other as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", other as u32)),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// Nearest-rank percentiles over a measure. Small frame counts, so sorting a
/// copy is cheaper than any streaming estimator and is exact.
fn percentiles(frames: &[FrameTiming], measure: impl Fn(&FrameTiming) -> u64) -> Percentiles {
    if frames.is_empty() {
        return Percentiles::default();
    }
    let mut values: Vec<u64> = frames.iter().map(&measure).collect();
    values.sort_unstable();
    let median_us = if values.len() % 2 == 1 {
        values[values.len() / 2]
    } else {
        let high = values.len() / 2;
        values[high - 1].midpoint(values[high])
    };
    // Nearest-rank: the smallest value at or above 95 % of the sample.
    let rank = (values.len() * 95).div_ceil(100).max(1);
    Percentiles {
        median_us,
        p95_us: values[rank - 1],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(index: u32, work: [u64; 4], wait: [u64; 2], other: u64) -> FrameTiming {
        let total = work.iter().chain(wait.iter()).sum::<u64>() + other;
        FrameTiming {
            index,
            mutate_us: work[0],
            frame_us: work[1],
            raster_us: work[2],
            compose_us: work[3],
            acquire_us: wait[0],
            present_us: wait[1],
            total_us: total,
            mutations: 1,
            restyled: 3,
            ..FrameTiming::default()
        }
    }

    /// The engine phase spans are a breakdown inside `frame_us`, never an
    /// addition to it: work stays what the thread computed, and whatever the
    /// frame did outside the three instrumented phases is named.
    #[test]
    fn engine_phases_partition_the_frame_span_without_changing_work() {
        let mut timing = frame(0, [10, 4_000, 900, 60], [15_800, 120], 45);
        assert!(!timing.has_phases());
        assert_eq!(timing.engine_phases_us(), 0);
        assert_eq!(timing.frame_other_us(), 4_000);

        timing.parse_us = 900_000;
        timing.style_us = 1_200;
        timing.layout_us = 2_300;
        timing.paint_us = 400;
        assert!(timing.has_phases());
        assert_eq!(timing.engine_phases_us(), 3_900);
        assert_eq!(timing.frame_other_us(), 100);
        // Parse happens before the frame loop, so it never enters work.
        assert_eq!(timing.work_us(), 4_970);
        assert_eq!(timing.other_us(), 45);
    }

    /// The accounting contract: work and wait name disjoint spans, neither
    /// claims a microsecond of the other, and the three parts reconstruct the
    /// measured frame exactly.
    #[test]
    fn work_and_wait_are_disjoint_and_sum_to_the_frame() {
        let timing = frame(0, [10, 4_000, 900, 60], [15_800, 120], 45);
        assert_eq!(timing.work_us(), 4_970);
        assert_eq!(timing.wait_us(), 15_920);
        assert_eq!(timing.other_us(), 45);
        assert_eq!(
            timing.work_us() + timing.wait_us() + timing.other_us(),
            timing.total_us,
            "the three parts partition the measured frame"
        );
        assert!(
            timing.wait_us() > timing.work_us(),
            "a vsync-bound frame must not report its block as work"
        );
    }

    /// A total smaller than its parts (clock granularity on a very fast frame)
    /// must not produce a negative residual.
    #[test]
    fn the_residual_never_goes_negative() {
        let mut timing = frame(0, [1, 1, 1, 1], [1, 1], 0);
        timing.total_us = 3;
        assert_eq!(timing.other_us(), 0);
    }

    #[test]
    fn percentiles_are_nearest_rank_over_the_measured_frames() {
        let mut log = TimingLog::new(1);
        // Frame 0 is the cold first frame and is excluded by the warm-up.
        log.push(frame(0, [0, 900_000, 1_000, 50], [200, 100], 0));
        for (index, work) in [1_000u64, 2_000, 3_000, 4_000, 100_000]
            .into_iter()
            .enumerate()
        {
            log.push(frame(index as u32 + 1, [0, work, 0, 0], [16_000, 0], 0));
        }
        let summary = log.summary();
        assert_eq!(summary.frames, 5, "the cold frame is not in the summary");
        assert_eq!(summary.work.median_us, 3_000);
        assert_eq!(summary.work.p95_us, 100_000);
        assert_eq!(summary.wait.median_us, 16_000);
        assert_eq!(summary.mutations, 5);
        assert_eq!(summary.restyled, 15);
    }

    /// A run shorter than its warm-up reports the frames it has rather than an
    /// empty summary.
    #[test]
    fn a_short_run_still_summarizes_what_it_measured() {
        let mut log = TimingLog::new(4);
        log.push(frame(0, [0, 5_000, 0, 0], [0, 0], 0));
        assert_eq!(log.summary().frames, 1);
        assert_eq!(log.summary().work.median_us, 5_000);
    }

    #[test]
    fn the_json_receipt_carries_every_frame_and_the_summary() {
        let mut log = TimingLog::new(0);
        log.push(frame(0, [10, 20, 30, 40], [50, 60], 5));
        let json = log.to_json(&[("address", json_string("file:///a\\b.html"))]);
        assert!(json.contains("\"address\": \"file:///a\\\\b.html\""));
        assert!(json.contains("\"work_us\":100"));
        assert!(json.contains("\"wait_us\":110"));
        assert!(json.contains("\"total_us\":215"));
        assert!(json.contains("\"work_median_us\": 100"));
    }
}
