// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

/// Cumulative shaping operations and optional owner-thread timings.
/// `line_breaks` counts formatting break operations, not emitted lines.
/// `worker_task_ns` is summed task elapsed time, not OS CPU accounting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextShapingStats {
    /// Content-bearing formatting transactions; empty groups are excluded.
    /// Cache hits still count as requests even when no layout is built.
    pub format_requests: u64,
    pub layout_builds: u64,
    pub metric_layout_builds: u64,
    pub line_breaks: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub unique_jobs: u64,
    pub late_misses: u64,
    pub batches: u64,
    /// Successfully joined pool batches and unique jobs in those batches.
    pub parallel_batches: u64,
    pub parallel_jobs: u64,
    /// Current retained worker scratch slots; zero without an attached pool.
    pub worker_scratch_slots: usize,
    pub retained_entries: usize,
    pub retained_text_bytes: usize,
    pub collect_ns: u64,
    pub shape_wall_ns: u64,
    pub worker_task_ns: u64,
    pub place_ns: u64,
}

/// Limits and instrumentation settings for text shaping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextShapingOptions {
    pub retained_entries: usize,
    pub retained_text_bytes: usize,
    pub parallel_min_groups: usize,
    pub parallel_min_text_bytes: usize,
    pub record_timings: bool,
}

impl Default for TextShapingOptions {
    fn default() -> Self {
        Self {
            retained_entries: 512,
            retained_text_bytes: 4 * 1024 * 1024,
            parallel_min_groups: 8,
            parallel_min_text_bytes: 16 * 1024,
            record_timings: false,
        }
    }
}

pub(super) fn elapsed_ns(start: Option<std::time::Instant>) -> u64 {
    start.map_or(0, |s| {
        u64::try_from(s.elapsed().as_nanos()).unwrap_or(u64::MAX)
    })
}
