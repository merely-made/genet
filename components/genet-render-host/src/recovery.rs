// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One recovery owner per shared core, with an explicit host-detachment barrier.
use crate::RenderCore;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};

pub type CoreGeneration = u64;
pub type ReplacementFactory = Arc<dyn Fn() -> Result<Arc<RenderCore>, String> + Send + Sync>;
type Wake = Arc<dyn Fn() + Send + Sync>;

pub(crate) fn next_generation() -> CoreGeneration {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
        .expect("render core generation exhausted")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceLossNotice {
    pub generation: CoreGeneration,
    pub reason: wgpu::DeviceLostReason,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryStatus {
    Ready {
        generation: CoreGeneration,
    },
    SurfaceUnavailable {
        generation: CoreGeneration,
        error: String,
    },
    Waiting {
        loss: DeviceLossNotice,
        remaining_hosts: usize,
    },
    Recovering {
        loss: DeviceLossNotice,
    },
    Failed {
        loss: DeviceLossNotice,
        error: String,
    },
}
impl RecoveryStatus {
    pub fn loss(&self) -> Option<&DeviceLossNotice> {
        match self {
            Self::Ready { .. } | Self::SurfaceUnavailable { .. } => None,
            Self::Waiting { loss, .. } | Self::Recovering { loss } | Self::Failed { loss, .. } => {
                Some(loss)
            },
        }
    }
}

#[derive(Default)]
struct WakeHub {
    next: AtomicU64,
    callbacks: Mutex<BTreeMap<u64, Wake>>,
}
impl WakeHub {
    fn wake_all(&self) {
        let callbacks: Vec<_> = self
            .callbacks
            .lock()
            .expect("wake hub poisoned")
            .values()
            .cloned()
            .collect();
        for wake in callbacks {
            wake();
        }
    }
    fn ids(&self) -> BTreeSet<u64> {
        self.callbacks
            .lock()
            .expect("wake hub poisoned")
            .keys()
            .copied()
            .collect()
    }
    fn subscribe(self: &Arc<Self>, wake: Wake) -> RecoverySubscription {
        let id = self
            .next
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .expect("recovery subscription exhausted");
        self.callbacks
            .lock()
            .expect("wake hub poisoned")
            .insert(id, wake);
        RecoverySubscription {
            id,
            hub: self.clone(),
        }
    }
}

/// A live host must release resources before acknowledging. Dropping the
/// subscription removes a closed host from the barrier and wakes the survivors.
pub struct RecoverySubscription {
    id: u64,
    hub: Arc<WakeHub>,
}
impl Drop for RecoverySubscription {
    fn drop(&mut self) {
        self.hub
            .callbacks
            .lock()
            .expect("wake hub poisoned")
            .remove(&self.id);
        self.hub.wake_all();
    }
}

/// The actual device callback and deterministic injection enter this same
/// generation-checked, deduplicating monitor. It owns no renderer or host.
pub struct DeviceLossMonitor {
    pub(crate) generation: CoreGeneration,
    loss: Mutex<Option<DeviceLossNotice>>,
    hub: Mutex<Option<Arc<WakeHub>>>,
    invalidations: AtomicUsize,
    next_cancellation: AtomicU64,
    cancellations: Mutex<BTreeMap<u64, Arc<dyn Fn(&DeviceLossNotice) + Send + Sync>>>,
}
pub struct LossCancellation {
    id: u64,
    monitor: Weak<DeviceLossMonitor>,
}
impl Drop for LossCancellation {
    fn drop(&mut self) {
        if let Some(monitor) = self.monitor.upgrade() {
            monitor
                .cancellations
                .lock()
                .expect("loss cancellations poisoned")
                .remove(&self.id);
        }
    }
}
impl DeviceLossMonitor {
    pub fn new(generation: CoreGeneration) -> Self {
        Self {
            generation,
            loss: Mutex::new(None),
            hub: Mutex::new(None),
            invalidations: AtomicUsize::new(0),
            next_cancellation: AtomicU64::new(1),
            cancellations: Mutex::new(BTreeMap::new()),
        }
    }
    pub fn notice(&self) -> Option<DeviceLossNotice> {
        let notice = self.pending_notice();
        if self.invalidations.load(Ordering::Acquire) != 0 {
            return None;
        }
        notice
    }
    pub fn pending_notice(&self) -> Option<DeviceLossNotice> {
        self.loss
            .lock()
            .expect("device loss monitor poisoned")
            .clone()
    }
    pub fn notify_loss(&self, notice: DeviceLossNotice) -> bool {
        if notice.generation != self.generation {
            return false;
        }
        {
            let mut loss = self.loss.lock().expect("device loss monitor poisoned");
            if loss.is_some() {
                return false;
            }
            self.invalidations.fetch_add(1, Ordering::AcqRel);
            *loss = Some(notice.clone());
        }
        let cancellations: Vec<_> = self
            .cancellations
            .lock()
            .expect("loss cancellations poisoned")
            .values()
            .cloned()
            .collect();
        for cancel in cancellations {
            cancel(&notice);
        }
        self.finish_invalidation();
        true
    }
    pub fn register_cancellation(
        self: &Arc<Self>,
        cancel: Arc<dyn Fn(&DeviceLossNotice) + Send + Sync>,
    ) -> LossCancellation {
        let delivered = AtomicBool::new(false);
        let cancel: Arc<dyn Fn(&DeviceLossNotice) + Send + Sync> = Arc::new(move |loss| {
            if !delivered.swap(true, Ordering::AcqRel) {
                cancel(loss);
            }
        });
        let id = self
            .next_cancellation
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .expect("loss cancellation identity exhausted");
        // A registration racing the snapshot is itself part of invalidation.
        // The once wrapper can return while a different thread still executes
        // its callback; a count remains held until that thread finishes.
        let loss = {
            let loss = self.loss.lock().expect("device loss monitor poisoned");
            if loss.is_some() {
                self.invalidations.fetch_add(1, Ordering::AcqRel);
            }
            // Publish the callback under the same loss lock as admission:
            // either the notifier must see it, or we hold a post-loss count.
            self.cancellations
                .lock()
                .expect("loss cancellations poisoned")
                .insert(id, cancel.clone());
            loss.clone()
        };
        if let Some(loss) = loss {
            cancel(&loss);
            self.finish_invalidation();
        }
        LossCancellation {
            id,
            monitor: Arc::downgrade(self),
        }
    }
    fn finish_invalidation(&self) {
        if self.invalidations.fetch_sub(1, Ordering::AcqRel) == 1 && self.pending_notice().is_some()
        {
            let hub = self.hub.lock().expect("device loss hub poisoned").clone();
            if let Some(hub) = hub {
                hub.wake_all();
            }
        }
    }
    // Admission covers readback creation before it has registered cancellable
    // resources. A published loss cannot become ready while creation is live.
    pub(crate) fn begin_operation(self: &Arc<Self>) -> Result<LossOperation, DeviceLossNotice> {
        let loss = self.loss.lock().expect("device loss monitor poisoned");
        if let Some(loss) = &*loss {
            return Err(loss.clone());
        }
        self.invalidations.fetch_add(1, Ordering::AcqRel);
        Ok(LossOperation(self.clone()))
    }
    fn bind_hub(&self, hub: Arc<WakeHub>) {
        *self.hub.lock().expect("device loss hub poisoned") = Some(hub.clone());
        if self.notice().is_some() {
            hub.wake_all();
        }
    }
}

pub(crate) struct LossOperation(Arc<DeviceLossMonitor>);
impl Drop for LossOperation {
    fn drop(&mut self) {
        self.0.finish_invalidation();
    }
}

enum State {
    Ready(Arc<RenderCore>),
    Waiting {
        loss: DeviceLossNotice,
        pending: BTreeSet<u64>,
    },
    Recovering(DeviceLossNotice),
    Failed {
        loss: DeviceLossNotice,
        error: String,
    },
}

/// The replacement recipe belongs to this ownership group, not to whichever
/// window happens to observe loss last. No callback/factory runs under a lock.
pub struct SharedRenderCore {
    state: Mutex<State>,
    hub: Arc<WakeHub>,
    factory: ReplacementFactory,
}
impl SharedRenderCore {
    /// First attachment wins. Reattaching an already-owned core returns its
    /// owner; it cannot establish a second replacement factory for that device.
    pub fn with_factory(core: Arc<RenderCore>, factory: ReplacementFactory) -> Arc<Self> {
        let mut association = core
            .recovery_owner
            .lock()
            .expect("core association poisoned");
        if let Some(owner) = association.upgrade() {
            return owner;
        }
        let owner = Arc::new(Self {
            state: Mutex::new(State::Ready(core.clone())),
            hub: Arc::new(WakeHub::default()),
            factory,
        });
        *association = Arc::downgrade(&owner);
        drop(association);
        core.loss.bind_hub(owner.hub.clone());
        owner
    }
    /// Attach an explicitly detached loss state. This is also the CPU-only
    /// entrance for testing the real failed-factory/host handling path, without
    /// pretending a GPU core was created or destroyed.
    pub fn from_detached_loss(loss: DeviceLossNotice, factory: ReplacementFactory) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State::Waiting {
                loss,
                pending: BTreeSet::new(),
            }),
            hub: Arc::new(WakeHub::default()),
            factory,
        })
    }
    pub fn subscribe(&self, wake: Wake) -> RecoverySubscription {
        let subscription = self.hub.subscribe(wake);
        let waiting = {
            let mut state = self.state.lock().expect("recovery owner poisoned");
            if let State::Waiting { pending, .. } = &mut *state {
                pending.insert(subscription.id);
                true
            } else {
                false
            }
        };
        if waiting {
            self.hub.wake_all();
        }
        subscription
    }
    pub fn status(&self) -> RecoveryStatus {
        let mut state = self.state.lock().expect("recovery owner poisoned");
        if let State::Ready(core) = &*state {
            if let Some(loss) = core.loss_notice() {
                *state = State::Waiting {
                    loss,
                    pending: self.hub.ids(),
                };
            } else if let Some(loss) = core.device_failure() {
                // Resources are being cancelled on the callback thread. The
                // device is already unavailable, but do not open the detach
                // barrier/factory until cancellation has completed.
                return RecoveryStatus::Recovering { loss };
            }
        }
        match &mut *state {
            State::Ready(core) => RecoveryStatus::Ready {
                generation: core.generation(),
            },
            State::Waiting { loss, pending } => {
                let live = self.hub.ids();
                pending.retain(|id| live.contains(id));
                RecoveryStatus::Waiting {
                    loss: loss.clone(),
                    remaining_hosts: pending.len(),
                }
            },
            State::Recovering(loss) => RecoveryStatus::Recovering { loss: loss.clone() },
            State::Failed { loss, error } => RecoveryStatus::Failed {
                loss: loss.clone(),
                error: error.clone(),
            },
        }
    }
    pub fn current(&self) -> Result<Arc<RenderCore>, RecoveryStatus> {
        let status = self.status();
        if !matches!(status, RecoveryStatus::Ready { .. }) {
            return Err(status);
        }
        let state = self.state.lock().expect("recovery owner poisoned");
        if let State::Ready(core) = &*state {
            Ok(core.clone())
        } else {
            drop(state);
            Err(self.status())
        }
    }
    pub fn notify_loss(&self, notice: DeviceLossNotice) -> bool {
        let core = {
            let state = self.state.lock().expect("recovery owner poisoned");
            match &*state {
                State::Ready(core) => Some(core.clone()),
                _ => None,
            }
        };
        core.is_some_and(|core| core.loss.notify_loss(notice))
    }
    pub fn acknowledge_detached(
        &self,
        subscription: &RecoverySubscription,
        generation: CoreGeneration,
    ) -> bool {
        if !Arc::ptr_eq(&subscription.hub, &self.hub) {
            return false;
        }
        let changed = {
            let mut state = self.state.lock().expect("recovery owner poisoned");
            match &mut *state {
                State::Waiting { loss, pending } if loss.generation == generation => {
                    pending.remove(&subscription.id)
                },
                _ => false,
            }
        };
        if changed {
            self.hub.wake_all();
        }
        changed
    }
    pub fn recover(self: &Arc<Self>) -> RecoveryStatus {
        self.status();
        let loss = {
            let mut state = self.state.lock().expect("recovery owner poisoned");
            match &*state {
                State::Waiting { loss, pending } if pending.is_empty() => {
                    let loss = loss.clone();
                    *state = State::Recovering(loss.clone());
                    loss
                },
                _ => {
                    drop(state);
                    return self.status();
                },
            }
        };
        let replacement = (self.factory)().and_then(|core| {
            if core.generation() == loss.generation || core.device_failure().is_some() {
                return Err("replacement factory returned a stale or lost core".into());
            }
            let mut association = core
                .recovery_owner
                .lock()
                .expect("core association poisoned");
            if association
                .upgrade()
                .is_some_and(|owner| !Arc::ptr_eq(&owner, self))
            {
                return Err("replacement core belongs to another recovery owner".into());
            }
            *association = Arc::downgrade(self);
            drop(association);
            core.loss.bind_hub(self.hub.clone());
            Ok(core)
        });
        {
            let mut state = self.state.lock().expect("recovery owner poisoned");
            *state = match replacement {
                Ok(core) => State::Ready(core),
                Err(error) => State::Failed { loss, error },
            };
        }
        self.hub.wake_all();
        self.status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    fn notice(generation: u64) -> DeviceLossNotice {
        DeviceLossNotice {
            generation,
            reason: wgpu::DeviceLostReason::Unknown,
            message: "injected actual notice seam".into(),
        }
    }
    #[test]
    fn detached_owner_waits_for_all_hosts_and_failed_factory_is_once() {
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let owner = SharedRenderCore::from_detached_loss(
            notice(7),
            Arc::new(move || {
                counted.fetch_add(1, Ordering::SeqCst);
                Err("deliberate boot refusal".into())
            }),
        );
        let first = owner.subscribe(Arc::new(|| {}));
        let second = owner.subscribe(Arc::new(|| {}));
        assert!(!owner.acknowledge_detached(&first, 6));
        assert!(owner.acknowledge_detached(&first, 7));
        assert!(matches!(
            owner.recover(),
            RecoveryStatus::Waiting {
                remaining_hosts: 1,
                ..
            }
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        drop(second);
        assert!(matches!(owner.recover(), RecoveryStatus::Failed { .. }));
        assert!(matches!(owner.recover(), RecoveryStatus::Failed { .. }));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn monitor_rejects_stale_and_duplicate_notices_and_wakes_without_locks() {
        let monitor = Arc::new(DeviceLossMonitor::new(9));
        let hub = Arc::new(WakeHub::default());
        monitor.bind_hub(hub.clone());
        let reads = monitor.clone();
        let seen = Arc::new(AtomicUsize::new(0));
        let count = seen.clone();
        let _subscription = hub.subscribe(Arc::new(move || {
            assert_eq!(reads.notice().unwrap().generation, 9);
            count.fetch_add(1, Ordering::SeqCst);
        }));
        assert!(!monitor.notify_loss(notice(8)));
        assert!(monitor.notify_loss(notice(9)));
        assert!(!monitor.notify_loss(notice(9)));
        assert_eq!(seen.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn foreign_subscription_cannot_acknowledge_and_cancellation_is_once() {
        let first = SharedRenderCore::from_detached_loss(
            notice(11),
            Arc::new(|| Err("first refused".into())),
        );
        let second = SharedRenderCore::from_detached_loss(
            notice(11),
            Arc::new(|| Err("second refused".into())),
        );
        let token = first.subscribe(Arc::new(|| {}));
        let foreign = second.subscribe(Arc::new(|| {}));
        assert!(!first.acknowledge_detached(&foreign, 11));
        assert!(matches!(
            first.recover(),
            RecoveryStatus::Waiting {
                remaining_hosts: 1,
                ..
            }
        ));
        assert!(first.acknowledge_detached(&token, 11));
        let monitor = Arc::new(DeviceLossMonitor::new(11));
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let _cancel = monitor.register_cancellation(Arc::new(move |_| {
            counted.fetch_add(1, Ordering::SeqCst);
        }));
        assert!(monitor.notify_loss(notice(11)));
        assert!(!monitor.notify_loss(notice(11)));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn concurrently_registered_cancellation_finishes_before_loss_is_ready() {
        use std::sync::mpsc;
        use std::time::Duration;
        let monitor = Arc::new(DeviceLossMonitor::new(31));
        let (first_started_send, first_started) = mpsc::channel();
        let (first_release, first_release_recv) = mpsc::channel();
        let first_release_recv = Mutex::new(first_release_recv);
        let _first = monitor.register_cancellation(Arc::new(move |_| {
            first_started_send.send(()).unwrap();
            first_release_recv
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5))
                .unwrap();
        }));
        let notifier = monitor.clone();
        let (notified_send, notified) = mpsc::channel();
        let notify_thread = std::thread::spawn(move || {
            assert!(notifier.notify_loss(notice(31)));
            notified_send.send(()).unwrap();
        });
        first_started.recv_timeout(Duration::from_secs(5)).unwrap();
        let registrar = monitor.clone();
        let (second_started_send, second_started) = mpsc::channel();
        let (second_release, second_release_recv) = mpsc::channel();
        let registration_thread = std::thread::spawn(move || {
            let second_release_recv = Mutex::new(second_release_recv);
            registrar.register_cancellation(Arc::new(move |_| {
                second_started_send.send(()).unwrap();
                second_release_recv
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap();
            }))
        });
        second_started.recv_timeout(Duration::from_secs(5)).unwrap();
        first_release.send(()).unwrap();
        notified.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(monitor.pending_notice(), Some(notice(31)));
        assert!(
            monitor.notice().is_none(),
            "a late registration still owns resources"
        );
        second_release.send(()).unwrap();
        let _second = registration_thread.join().unwrap();
        notify_thread.join().unwrap();
        assert_eq!(monitor.notice(), Some(notice(31)));
    }
    #[test]
    fn admitted_readback_creation_delays_loss_readiness_until_resources_are_registered() {
        let monitor = Arc::new(DeviceLossMonitor::new(32));
        let creation = monitor.begin_operation().unwrap();
        assert!(monitor.notify_loss(notice(32)));
        assert!(monitor.notice().is_none());
        assert!(matches!(monitor.begin_operation(), Err(loss) if loss == notice(32)));
        let cancelled = Arc::new(AtomicBool::new(false));
        let flag = cancelled.clone();
        let _resources = monitor.register_cancellation(Arc::new(move |_| {
            flag.store(true, Ordering::Release);
        }));
        assert!(cancelled.load(Ordering::Acquire));
        assert!(monitor.notice().is_none());
        drop(creation);
        assert_eq!(monitor.notice(), Some(notice(32)));
    }
}
