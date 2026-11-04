// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::time::{Duration, Instant};

#[derive(Default)]
pub(super) struct Progress {
    pub last_sample: Option<Instant>,
    pub last_quality_poll: Option<Duration>,
    pub last_device_poll: Option<Duration>,
    pub last_scope_poll: Option<Duration>,
    pub last_wait: Option<(Duration, String)>,
    pub wait_returned: Option<Instant>,
    pub worker_scope_pop: Option<Duration>,
    pub worker_scope_observe: Option<Duration>,
    pub worker_finished: Option<Instant>,
}
impl Progress {
    pub fn describe(&self, now: Instant) -> String {
        format!(
            "sample_age={:?}, device_poll={:?}, quality_poll={:?}, scope_poll={:?}, last_wait={:?}, wait_return_age={:?}, worker_scope_pop={:?}, worker_scope_observe={:?}, worker_finish_age={:?}",
            self.last_sample.map(|at| now.duration_since(at)),
            self.last_device_poll,
            self.last_quality_poll,
            self.last_scope_poll,
            self.last_wait,
            self.wait_returned.map(|at| now.duration_since(at)),
            self.worker_scope_pop,
            self.worker_scope_observe,
            self.worker_finished.map(|at| now.duration_since(at))
        )
    }
}
