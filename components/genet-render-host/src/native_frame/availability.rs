// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One-shot availability hints share native admission's phase mutex.

use super::{Arc, Mutex, NativeFrames, RenderCore, State, Wake, Weak};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default)]
pub(super) struct Gate {
    pub(super) phase: State,
    next_id: u64,
    waiters: BTreeMap<u64, Weak<Signal>>,
}

pub(super) struct Signal {
    ready: AtomicBool,
    wake: Wake,
}
impl Signal {
    pub(super) fn notify(&self) {
        // A failed notification cannot starve the other waiting hosts. Every
        // readiness bit was set before any callback, outside all gate locks.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (self.wake)()));
    }
}

impl Gate {
    pub(super) fn finish(&mut self, reusable: bool) -> Vec<Arc<Signal>> {
        if !matches!(self.phase, State::Closed) {
            self.phase = if reusable {
                State::Idle
            } else {
                State::Unavailable
            };
        }
        self.take_waiters()
    }
    pub(super) fn close(&mut self) -> Vec<Arc<Signal>> {
        self.phase = State::Closed;
        self.take_waiters()
    }
    fn take_waiters(&mut self) -> Vec<Arc<Signal>> {
        let signals: Vec<_> = std::mem::take(&mut self.waiters)
            .into_values()
            .filter_map(|waiter| waiter.upgrade())
            .collect();
        for signal in &signals {
            signal.ready.store(true, Ordering::Release);
        }
        signals
    }
}

/// One registered availability hint. This retains neither Core nor GPU state:
/// its owner link is weak and points only to the CPU admission gate.
///
/// Drop unregisters it. A notification already taken by `finish` may still run
/// after Drop, so the supplied wake must retain notification state rather than
/// a Host/Core. Readiness is sticky, not a reservation; another host may obtain
/// admission before the notified host retries.
#[must_use = "retain the waiter until retry or cancellation"]
pub struct NativeFrameWaiter {
    owner: Weak<Mutex<Gate>>,
    id: Option<u64>,
    signal: Arc<Signal>,
}
impl NativeFrameWaiter {
    pub fn ready(&self) -> bool {
        self.signal.ready.load(Ordering::Acquire) || self.owner.strong_count() == 0
    }
}
impl Drop for NativeFrameWaiter {
    fn drop(&mut self) {
        if let Some(id) = self.id.take()
            && let Some(owner) = self.owner.upgrade()
        {
            owner
                .lock()
                .expect("native frames poisoned")
                .waiters
                .remove(&id);
        }
    }
}

impl NativeFrames {
    fn notify_available(&self, wake: Wake) -> NativeFrameWaiter {
        let signal = Arc::new(Signal {
            ready: AtomicBool::new(false),
            wake,
        });
        let id = {
            let mut gate = self.state.lock().expect("native frames poisoned");
            if matches!(gate.phase, State::Idle | State::Unavailable | State::Closed) {
                signal.ready.store(true, Ordering::Release);
                None
            } else {
                gate.next_id = gate
                    .next_id
                    .checked_add(1)
                    .expect("native frame waiter ID exhausted");
                let id = gate.next_id;
                gate.waiters.insert(id, Arc::downgrade(&signal));
                Some(id)
            }
        };
        if id.is_none() {
            signal.notify();
        }
        NativeFrameWaiter {
            owner: Arc::downgrade(&self.state),
            id,
            signal,
        }
    }
}

impl RenderCore {
    /// Register atomically with native retirement, avoiding the Busy-to-idle
    /// lost-wake race. Idle or terminal unavailability notifies immediately;
    /// every callback runs outside the gate lock and GPU qualification scopes.
    pub fn notify_native_frame_available(self: &Arc<Self>, wake: Wake) -> NativeFrameWaiter {
        self.native_frames.notify_available(wake)
    }
}

#[cfg(test)]
mod tests;
