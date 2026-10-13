// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One recovery owner per shared core, with an explicit host-detachment barrier.
use crate::RenderCore;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::task::{Context, Poll, Waker};

pub type CoreGeneration = u64;
pub type ReplacementFactory = Arc<dyn Fn() -> Result<Arc<RenderCore>, String> + Send + Sync>;
/// The factory itself is shareable; the returned browser future need not be Send.
pub type ReplacementFuture =
    Pin<Box<dyn Future<Output = Result<Arc<RenderCore>, String>> + 'static>>;
pub type AsyncReplacementFactory = Arc<dyn Fn() -> ReplacementFuture + Send + Sync>;
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
    Recovering {
        loss: DeviceLossNotice,
        attempt: u64,
        waker: Option<Waker>,
    },
    Failed {
        loss: DeviceLossNotice,
        error: String,
    },
}

enum Factory {
    Sync(ReplacementFactory),
    Async(AsyncReplacementFactory),
}

/// Refusal to complete a replacement that no longer owns this transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplacementCompletionError {
    OwnerDropped,
    ForeignOwner,
    StaleTransition,
}
impl std::fmt::Display for ReplacementCompletionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "replacement completion refused: {self:?}")
    }
}
impl std::error::Error for ReplacementCompletionError {}

struct ReplacementClaim {
    owner: Weak<SharedRenderCore>,
    loss: DeviceLossNotice,
    attempt: u64,
    active: bool,
}
impl Drop for ReplacementClaim {
    fn drop(&mut self) {
        if self.active {
            if let Some(owner) = self.owner.upgrade() {
                let _ = owner.fail_attempt(
                    self.loss.generation,
                    self.attempt,
                    "replacement task was abandoned".into(),
                );
            }
        }
    }
}

/// An independent deadline handle. It retains no core or future. Expiry is
/// sticky; late task completion cannot overwrite it or start another attempt.
#[derive(Clone)]
pub struct ReplacementExpiry {
    owner: Weak<SharedRenderCore>,
    generation: CoreGeneration,
    attempt: u64,
}
impl ReplacementExpiry {
    pub fn expire(&self) -> Result<RecoveryStatus, ReplacementCompletionError> {
        self.owner
            .upgrade()
            .ok_or(ReplacementCompletionError::OwnerDropped)?
            .fail_attempt(
                self.generation,
                self.attempt,
                "asynchronous replacement deadline expired".into(),
            )
    }
}

/// One claimed asynchronous replacement. The host must poll/await this local
/// future and impose a finite deadline with `with_deadline()` and `expiry()`.
/// The predicate rejects late readiness even when a JS timer is delayed;
/// the independent expiry wake interrupts a stalled promise. Dropping it cancels the
/// attempt after dropping its in-flight future, and wakes subscribers outside
/// locks. A factory or completion failure is terminal for this loss transition.
#[must_use = "await replacement or drop it to refuse the claimed transition"]
pub struct PendingReplacement {
    future: Option<ReplacementFuture>,
    claim: Option<ReplacementClaim>,
    deadline_expired: Option<Box<dyn Fn() -> bool + 'static>>,
}
impl PendingReplacement {
    /// Supply the host's local monotonic clock/deadline predicate. It runs
    /// outside owner locks before acquisition polling and again before a ready
    /// result can install a core. The closure need not be Send; retain only
    /// clock/deadline facts, never old GPU resources. Also arm `expiry()` to
    /// wake the task if its acquisition promise does not settle.
    pub fn with_deadline(mut self, is_expired: impl Fn() -> bool + 'static) -> Self {
        self.deadline_expired = Some(Box::new(is_expired));
        self
    }
    fn deadline_is_expired(&self) -> bool {
        self.deadline_expired.as_ref().is_some_and(|check| check())
    }
    fn refuse_deadline(&mut self) -> Result<RecoveryStatus, ReplacementCompletionError> {
        // Disposal precedes the failed-state wake, even when the acquisition
        // was ready during a long final poll before an overdue timer could run.
        drop(self.future.take());
        let mut claim = self.claim.take().expect("replacement already collected");
        claim.active = false;
        claim
            .owner
            .upgrade()
            .ok_or(ReplacementCompletionError::OwnerDropped)
            .and_then(|owner| {
                owner.fail_attempt(
                    claim.loss.generation,
                    claim.attempt,
                    "asynchronous replacement deadline expired".into(),
                )
            })
    }
    pub fn loss(&self) -> &DeviceLossNotice {
        &self
            .claim
            .as_ref()
            .expect("replacement already collected")
            .loss
    }
    pub fn expiry(&self) -> ReplacementExpiry {
        let claim = self.claim.as_ref().expect("replacement already collected");
        ReplacementExpiry {
            owner: claim.owner.clone(),
            generation: claim.loss.generation,
            attempt: claim.attempt,
        }
    }
}
impl Future for PendingReplacement {
    type Output = Result<RecoveryStatus, ReplacementCompletionError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let claim = this.claim.as_ref().expect("replacement already collected");
        let admitted = claim
            .owner
            .upgrade()
            .ok_or(ReplacementCompletionError::OwnerDropped)
            .and_then(|owner| owner.poll_attempt(claim, cx.waker()));
        if let Err(error) = admitted {
            drop(this.future.take());
            if let Some(mut claim) = this.claim.take() {
                claim.active = false;
            }
            return Poll::Ready(Err(error));
        }
        if this.deadline_is_expired() {
            return Poll::Ready(this.refuse_deadline());
        }
        let result = match this
            .future
            .as_mut()
            .expect("replacement already collected")
            .as_mut()
            .poll(cx)
        {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(result) => result,
        };
        // A completed/aborted factory future must release its handles before
        // completion can publish a core or wake any host.
        drop(this.future.take());
        if this.deadline_is_expired() {
            drop(result);
            return Poll::Ready(this.refuse_deadline());
        }
        let mut claim = this.claim.take().expect("replacement already collected");
        let result = claim
            .owner
            .upgrade()
            .ok_or(ReplacementCompletionError::OwnerDropped)
            .and_then(|owner| owner.complete_attempt(&mut claim, result));
        Poll::Ready(result)
    }
}
impl Drop for PendingReplacement {
    fn drop(&mut self) {
        drop(self.future.take());
        drop(self.claim.take());
    }
}

/// The replacement recipe belongs to this ownership group, not to whichever
/// window happens to observe loss last. No callback/factory runs under a lock.
pub struct SharedRenderCore {
    state: Mutex<State>,
    hub: Arc<WakeHub>,
    factory: Factory,
    next_attempt: AtomicU64,
}
impl SharedRenderCore {
    /// First attachment wins. Reattaching an already-owned core returns its
    /// owner; it cannot establish a second replacement factory for that device.
    pub fn with_factory(core: Arc<RenderCore>, factory: ReplacementFactory) -> Arc<Self> {
        Self::attach(core, Factory::Sync(factory))
    }
    /// Attach a deferred factory without storing a future or old GPU handles.
    /// As with native attachment, an existing owner always wins.
    pub fn with_async_factory(
        core: Arc<RenderCore>,
        factory: AsyncReplacementFactory,
    ) -> Arc<Self> {
        Self::attach(core, Factory::Async(factory))
    }
    fn attach(core: Arc<RenderCore>, factory: Factory) -> Arc<Self> {
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
            next_attempt: AtomicU64::new(1),
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
        Self::detached(loss, Factory::Sync(factory))
    }
    /// The same actual owner/barrier entrance for deferred replacement controls.
    pub fn from_detached_loss_async(
        loss: DeviceLossNotice,
        factory: AsyncReplacementFactory,
    ) -> Arc<Self> {
        Self::detached(loss, Factory::Async(factory))
    }
    fn detached(loss: DeviceLossNotice, factory: Factory) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(State::Waiting {
                loss,
                pending: BTreeSet::new(),
            }),
            hub: Arc::new(WakeHub::default()),
            factory,
            next_attempt: AtomicU64::new(1),
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
            State::Recovering { loss, .. } => RecoveryStatus::Recovering { loss: loss.clone() },
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
    fn begin_attempt(self: &Arc<Self>) -> Result<ReplacementClaim, RecoveryStatus> {
        self.status();
        let mut state = self.state.lock().expect("recovery owner poisoned");
        match &*state {
            State::Waiting { loss, pending } if pending.is_empty() => {
                let loss = loss.clone();
                let attempt = self
                    .next_attempt
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
                    .expect("replacement attempt identity exhausted");
                *state = State::Recovering {
                    loss: loss.clone(),
                    attempt,
                    waker: None,
                };
                Ok(ReplacementClaim {
                    owner: Arc::downgrade(self),
                    loss,
                    attempt,
                    active: true,
                })
            },
            _ => {
                drop(state);
                Err(self.status())
            },
        }
    }
    /// Claim the asynchronous factory once, after all hosts have detached.
    /// Ordinary `recover()` never invokes an asynchronous factory or fails it
    /// just because the browser task has not been launched yet.
    pub fn begin_async_replacement(self: &Arc<Self>) -> Result<PendingReplacement, RecoveryStatus> {
        let Factory::Async(factory) = &self.factory else {
            return Err(self.status());
        };
        let claim = self.begin_attempt()?;
        // Factory invocation is outside the owner lock; no Send bound is
        // imposed on the future returned for the browser event loop.
        let future = factory();
        Ok(PendingReplacement {
            future: Some(future),
            claim: Some(claim),
            deadline_expired: None,
        })
    }
    fn poll_attempt(
        &self,
        claim: &ReplacementClaim,
        waker: &Waker,
    ) -> Result<(), ReplacementCompletionError> {
        let replacement_waker = waker.clone();
        let retired;
        {
            let mut state = self.state.lock().expect("recovery owner poisoned");
            match &mut *state {
                State::Recovering {
                    loss,
                    attempt,
                    waker: stored,
                } if claim.active
                    && loss.generation == claim.loss.generation
                    && *attempt == claim.attempt =>
                {
                    retired = if stored
                        .as_ref()
                        .is_none_or(|stored| !stored.will_wake(waker))
                    {
                        stored.replace(replacement_waker)
                    } else {
                        Some(replacement_waker)
                    };
                },
                _ => {
                    drop(state);
                    return Err(ReplacementCompletionError::StaleTransition);
                },
            }
        }
        drop(retired);
        Ok(())
    }
    fn complete_attempt(
        self: &Arc<Self>,
        claim: &mut ReplacementClaim,
        replacement: Result<Arc<RenderCore>, String>,
    ) -> Result<RecoveryStatus, ReplacementCompletionError> {
        if !Weak::ptr_eq(&claim.owner, &Arc::downgrade(self)) {
            return Err(ReplacementCompletionError::ForeignOwner);
        }
        let mut replacement = replacement.and_then(|core| {
            if core.generation() == claim.loss.generation || core.device_failure().is_some() {
                return Err("replacement factory returned a stale or lost core".into());
            }
            Ok(core)
        });
        let mut retired = None;
        let mut foreign_owner = None;
        let retired_waker;
        let monitor;
        {
            let mut state = self.state.lock().expect("recovery owner poisoned");
            if !claim.active
                || !matches!(&*state, State::Recovering { loss, attempt, .. } if loss.generation == claim.loss.generation && *attempt == claim.attempt)
            {
                claim.active = false;
                drop(state);
                // A late successful future drops its new core outside locks.
                drop(replacement);
                return Err(ReplacementCompletionError::StaleTransition);
            }
            let foreign = if let Ok(core) = &replacement {
                let mut association = core
                    .recovery_owner
                    .lock()
                    .expect("core association poisoned");
                let associated = association.upgrade();
                if associated
                    .as_ref()
                    .is_some_and(|owner| !Arc::ptr_eq(owner, self))
                {
                    foreign_owner = associated;
                    true
                } else {
                    *association = Arc::downgrade(self);
                    false
                }
            } else {
                false
            };
            if foreign {
                retired = std::mem::replace(
                    &mut replacement,
                    Err("replacement core belongs to another recovery owner".into()),
                )
                .ok();
            }
            monitor = replacement.as_ref().ok().map(|core| core.loss.clone());
            retired_waker = match &mut *state {
                State::Recovering { waker, .. } => waker.take(),
                _ => None,
            };
            *state = match replacement {
                Ok(core) => State::Ready(core),
                Err(error) => State::Failed {
                    loss: claim.loss.clone(),
                    error,
                },
            };
            claim.active = false;
        }
        drop(retired);
        drop(foreign_owner);
        drop(retired_waker);
        // Binding can wake synchronously; never do it under the owner lock.
        // A monitor clone retains no renderer/device during reentrant wakes.
        if let Some(monitor) = monitor {
            monitor.bind_hub(self.hub.clone());
        }
        self.hub.wake_all();
        Ok(self.status())
    }
    fn fail_attempt(
        &self,
        generation: CoreGeneration,
        attempt: u64,
        error: String,
    ) -> Result<RecoveryStatus, ReplacementCompletionError> {
        let waker;
        {
            let mut state = self.state.lock().expect("recovery owner poisoned");
            let State::Recovering {
                loss,
                attempt: current,
                waker: pending,
            } = &mut *state
            else {
                return Err(ReplacementCompletionError::StaleTransition);
            };
            if loss.generation != generation || *current != attempt {
                return Err(ReplacementCompletionError::StaleTransition);
            }
            waker = pending.take();
            *state = State::Failed {
                loss: loss.clone(),
                error,
            };
        }
        // Expiry also wakes a pending local future, which drops its acquisition
        // future instead of waiting for a browser promise that may never settle.
        if let Some(waker) = waker {
            waker.wake();
        }
        self.hub.wake_all();
        Ok(self.status())
    }
    pub fn recover(self: &Arc<Self>) -> RecoveryStatus {
        let Factory::Sync(factory) = &self.factory else {
            return self.status();
        };
        let Ok(mut claim) = self.begin_attempt() else {
            return self.status();
        };
        let replacement = factory();
        self.complete_attempt(&mut claim, replacement)
            .unwrap_or_else(|_| self.status())
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
    struct CountWake(AtomicUsize);
    impl std::task::Wake for CountWake {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    struct LocalAcquisition {
        // A genuinely !Send acquisition future is accepted by the public API.
        _local: std::rc::Rc<()>,
        drops: Arc<AtomicUsize>,
    }
    impl Future for LocalAcquisition {
        type Output = Result<Arc<RenderCore>, String>;
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            Poll::Pending
        }
    }
    impl Drop for LocalAcquisition {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn deferred_owner(
        generation: u64,
        calls: Arc<AtomicUsize>,
        drops: Arc<AtomicUsize>,
    ) -> Arc<SharedRenderCore> {
        SharedRenderCore::from_detached_loss_async(
            notice(generation),
            Arc::new(move || {
                calls.fetch_add(1, Ordering::SeqCst);
                Box::pin(LocalAcquisition {
                    _local: std::rc::Rc::new(()),
                    drops: drops.clone(),
                })
            }),
        )
    }
    #[test]
    fn async_owner_requires_all_detachments_and_claims_once_without_sync_failure() {
        let calls = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let owner = deferred_owner(41, calls.clone(), drops.clone());
        let first = owner.subscribe(Arc::new(|| {}));
        let second = owner.subscribe(Arc::new(|| {}));
        assert!(owner.acknowledge_detached(&first, 41));
        assert!(matches!(
            owner.begin_async_replacement(),
            Err(RecoveryStatus::Waiting {
                remaining_hosts: 1,
                ..
            })
        ));
        assert!(matches!(
            owner.recover(),
            RecoveryStatus::Waiting {
                remaining_hosts: 1,
                ..
            }
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        // A closed host removes itself from the actual barrier.
        drop(second);
        assert!(matches!(
            owner.recover(),
            RecoveryStatus::Waiting {
                remaining_hosts: 0,
                ..
            }
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let pending = owner.begin_async_replacement().unwrap();
        assert_eq!(pending.loss().generation, 41);
        assert!(matches!(
            owner.begin_async_replacement(),
            Err(RecoveryStatus::Recovering { .. })
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        drop(pending);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert!(matches!(owner.recover(), RecoveryStatus::Failed { .. }));
        assert!(matches!(
            owner.begin_async_replacement(),
            Err(RecoveryStatus::Failed { .. })
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn dropping_async_task_releases_future_before_reentrant_failure_wake() {
        let drops = Arc::new(AtomicUsize::new(0));
        let owner = deferred_owner(42, Arc::new(AtomicUsize::new(0)), drops.clone());
        let weak = Arc::downgrade(&owner);
        let seen = Arc::new(AtomicUsize::new(0));
        let observed = seen.clone();
        let dropped = drops.clone();
        let host = owner.subscribe(Arc::new(move || {
            let owner = weak.upgrade().unwrap();
            if matches!(owner.status(), RecoveryStatus::Failed { .. }) {
                assert_eq!(dropped.load(Ordering::SeqCst), 1);
                assert!(matches!(owner.recover(), RecoveryStatus::Failed { .. }));
                observed.fetch_add(1, Ordering::SeqCst);
            }
        }));
        assert!(owner.acknowledge_detached(&host, 42));
        drop(owner.begin_async_replacement().unwrap());
        assert_eq!(seen.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn async_expiry_wakes_pending_task_and_rejects_late_completion() {
        let drops = Arc::new(AtomicUsize::new(0));
        let owner = deferred_owner(43, Arc::new(AtomicUsize::new(0)), drops.clone());
        let mut pending = owner.begin_async_replacement().unwrap();
        let expiry = pending.expiry();
        let wake = Arc::new(CountWake(AtomicUsize::new(0)));
        let waker = Waker::from(wake.clone());
        let mut cx = Context::from_waker(&waker);
        assert!(Pin::new(&mut pending).poll(&mut cx).is_pending());
        assert!(matches!(expiry.expire(), Ok(RecoveryStatus::Failed { .. })));
        assert_eq!(wake.0.load(Ordering::SeqCst), 1);
        assert!(matches!(
            Pin::new(&mut pending).poll(&mut cx),
            Poll::Ready(Err(ReplacementCompletionError::StaleTransition))
        ));
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert_eq!(
            expiry.expire(),
            Err(ReplacementCompletionError::StaleTransition)
        );
        assert!(matches!(owner.recover(), RecoveryStatus::Failed { .. }));
    }
    struct DeadlineAcquisition {
        expired: Arc<AtomicBool>,
        polls: Arc<AtomicUsize>,
        drops: Arc<AtomicUsize>,
    }
    impl Future for DeadlineAcquisition {
        type Output = Result<Arc<RenderCore>, String>;
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            self.polls.fetch_add(1, Ordering::SeqCst);
            // Model a final acquisition poll crossing the monotonic deadline
            // before the browser has delivered its independent timeout task.
            self.expired.store(true, Ordering::Release);
            Poll::Ready(Err("late factory response".into()))
        }
    }
    impl Drop for DeadlineAcquisition {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    fn deadline_owner(
        generation: u64,
        expired: Arc<AtomicBool>,
        polls: Arc<AtomicUsize>,
        drops: Arc<AtomicUsize>,
    ) -> Arc<SharedRenderCore> {
        SharedRenderCore::from_detached_loss_async(
            notice(generation),
            Arc::new(move || {
                Box::pin(DeadlineAcquisition {
                    expired: expired.clone(),
                    polls: polls.clone(),
                    drops: drops.clone(),
                })
            }),
        )
    }
    #[test]
    fn monotonic_deadline_refuses_before_acquisition_is_polled() {
        let expired = Arc::new(AtomicBool::new(true));
        let polls = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let owner = deadline_owner(47, expired.clone(), polls.clone(), drops.clone());
        let mut pending = owner
            .begin_async_replacement()
            .unwrap()
            .with_deadline(move || expired.load(Ordering::Acquire));
        let waker = Waker::from(Arc::new(CountWake(AtomicUsize::new(0))));
        let mut cx = Context::from_waker(&waker);
        assert!(
            matches!(Pin::new(&mut pending).poll(&mut cx), Poll::Ready(Ok(RecoveryStatus::Failed { error, .. })) if error == "asynchronous replacement deadline expired")
        );
        assert_eq!(polls.load(Ordering::SeqCst), 0);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert!(matches!(
            owner.begin_async_replacement(),
            Err(RecoveryStatus::Failed { .. })
        ));
    }
    #[test]
    fn late_ready_acquisition_cannot_beat_delayed_timer_or_install_a_core() {
        let expired = Arc::new(AtomicBool::new(false));
        let polls = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let owner = deadline_owner(48, expired.clone(), polls.clone(), drops.clone());
        let weak = Arc::downgrade(&owner);
        let released = drops.clone();
        let host = owner.subscribe(Arc::new(move || {
            if matches!(
                weak.upgrade().unwrap().status(),
                RecoveryStatus::Failed { .. }
            ) {
                assert_eq!(released.load(Ordering::SeqCst), 1);
            }
        }));
        owner.acknowledge_detached(&host, 48);
        let mut pending = owner
            .begin_async_replacement()
            .unwrap()
            .with_deadline(move || expired.load(Ordering::Acquire));
        let expiry = pending.expiry();
        let waker = Waker::from(Arc::new(CountWake(AtomicUsize::new(0))));
        let mut cx = Context::from_waker(&waker);
        // No independent timer callback is delivered before this ready result.
        assert!(
            matches!(Pin::new(&mut pending).poll(&mut cx), Poll::Ready(Ok(RecoveryStatus::Failed { error, .. })) if error == "asynchronous replacement deadline expired")
        );
        assert_eq!(polls.load(Ordering::SeqCst), 1);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert_eq!(
            expiry.expire(),
            Err(ReplacementCompletionError::StaleTransition)
        );
        assert!(matches!(
            owner.current(),
            Err(RecoveryStatus::Failed { .. })
        ));
    }
    #[test]
    fn dropped_owner_refuses_pending_completion_and_releases_local_future() {
        let drops = Arc::new(AtomicUsize::new(0));
        let owner = deferred_owner(46, Arc::new(AtomicUsize::new(0)), drops.clone());
        let mut pending = owner.begin_async_replacement().unwrap();
        let expiry = pending.expiry();
        drop(owner);
        assert_eq!(
            expiry.expire(),
            Err(ReplacementCompletionError::OwnerDropped)
        );
        let waker = Waker::from(Arc::new(CountWake(AtomicUsize::new(0))));
        let mut cx = Context::from_waker(&waker);
        assert!(matches!(
            Pin::new(&mut pending).poll(&mut cx),
            Poll::Ready(Err(ReplacementCompletionError::OwnerDropped))
        ));
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn foreign_completion_and_stale_deadline_do_not_mutate_live_transition() {
        let first = deferred_owner(
            44,
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicUsize::new(0)),
        );
        let second = deferred_owner(
            44,
            Arc::new(AtomicUsize::new(0)),
            Arc::new(AtomicUsize::new(0)),
        );
        let mut claim = first.begin_attempt().unwrap();
        assert_eq!(
            second.complete_attempt(&mut claim, Err("foreign".into())),
            Err(ReplacementCompletionError::ForeignOwner)
        );
        assert!(matches!(first.status(), RecoveryStatus::Recovering { .. }));
        assert!(matches!(second.status(), RecoveryStatus::Waiting { .. }));
        assert_eq!(
            first.fail_attempt(43, claim.attempt, "stale generation".into()),
            Err(ReplacementCompletionError::StaleTransition)
        );
        assert_eq!(
            first.fail_attempt(44, claim.attempt + 1, "stale attempt".into()),
            Err(ReplacementCompletionError::StaleTransition)
        );
        assert!(
            matches!(first.complete_attempt(&mut claim, Err("actual failure".into())), Ok(RecoveryStatus::Failed { error, .. }) if error == "actual failure")
        );
        assert_eq!(
            first.complete_attempt(&mut claim, Err("duplicate".into())),
            Err(ReplacementCompletionError::StaleTransition)
        );
    }
    #[test]
    fn async_factory_and_failed_completion_are_reentrant_and_once() {
        let holder = Arc::new(Mutex::new(Weak::<SharedRenderCore>::new()));
        let factory_owner = holder.clone();
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let owner = SharedRenderCore::from_detached_loss_async(
            notice(45),
            Arc::new(move || {
                let owner = factory_owner.lock().unwrap().upgrade().unwrap();
                assert!(matches!(owner.status(), RecoveryStatus::Recovering { .. }));
                counted.fetch_add(1, Ordering::SeqCst);
                Box::pin(std::future::ready(Err("async boot refused".into())))
            }),
        );
        *holder.lock().unwrap() = Arc::downgrade(&owner);
        let weak = Arc::downgrade(&owner);
        let host = owner.subscribe(Arc::new(move || {
            let _ = weak.upgrade().unwrap().status();
        }));
        owner.acknowledge_detached(&host, 45);
        let mut pending = owner.begin_async_replacement().unwrap();
        let waker = Waker::from(Arc::new(CountWake(AtomicUsize::new(0))));
        let mut cx = Context::from_waker(&waker);
        assert!(
            matches!(Pin::new(&mut pending).poll(&mut cx), Poll::Ready(Ok(RecoveryStatus::Failed { error, .. })) if error == "async boot refused")
        );
        assert!(matches!(owner.recover(), RecoveryStatus::Failed { .. }));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
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
