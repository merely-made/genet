// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Monotonic attribution only. These spans add no polling, waiting or wakes.

use std::time::{Duration, Instant};

/// Non-overlapping wall intervals between scheduling and UI channel receipt.
/// Worker qualification includes device polling, error scopes and quality
/// checks; it is not GPU duration. Ready-to-collect includes notification,
/// event-loop dispatch and other scheduling until successful channel receipt.
#[derive(Clone, Copy, Debug)]
pub struct NativeFrameTiming {
    sealed_at: Instant,
    worker_started_at: Instant,
    ready_at: Instant,
    collected_at: Instant,
}

impl NativeFrameTiming {
    pub fn sealed_at(&self) -> Instant {
        self.sealed_at
    }

    pub fn collected_at(&self) -> Instant {
        self.collected_at
    }

    pub fn queue(&self) -> Duration {
        self.worker_started_at.duration_since(self.sealed_at)
    }

    pub fn worker(&self) -> Duration {
        self.ready_at.duration_since(self.worker_started_at)
    }

    pub fn ready_to_collect(&self) -> Duration {
        self.collected_at.duration_since(self.ready_at)
    }

    pub fn seal_to_collect(&self) -> Duration {
        self.collected_at.duration_since(self.sealed_at)
    }
}

pub(super) struct WorkTiming {
    sealed_at: Instant,
    worker_started_at: Instant,
}

impl WorkTiming {
    pub(super) fn new() -> Self {
        let sealed_at = Instant::now();
        Self {
            sealed_at,
            worker_started_at: sealed_at,
        }
    }

    pub(super) fn start_worker(&mut self) {
        self.worker_started_at = Instant::now();
    }

    pub(super) fn worker_started_at(&self) -> Instant {
        self.worker_started_at
    }

    pub(super) fn ready_at(self, ready_at: Instant) -> ReadyTiming {
        ReadyTiming {
            sealed_at: self.sealed_at,
            worker_started_at: self.worker_started_at,
            ready_at,
        }
    }
}

pub(super) struct ReadyTiming {
    sealed_at: Instant,
    worker_started_at: Instant,
    ready_at: Instant,
}

impl ReadyTiming {
    pub(super) fn collected(&self, collected_at: Instant) -> NativeFrameTiming {
        NativeFrameTiming {
            sealed_at: self.sealed_at,
            worker_started_at: self.worker_started_at,
            ready_at: self.ready_at,
            collected_at,
        }
    }
}

#[cfg(test)]
mod tests;
