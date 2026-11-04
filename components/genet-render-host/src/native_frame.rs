// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One demand-started completion worker per shared Core. UI collection never
//! polls or waits on the device; only accepted staging can reach publication.

use crate::recovery::{LossCancellation, LossOperation};
use crate::{DeviceLossNotice, RasterizeFailure, RenderCore};
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Condvar, Mutex, Weak, mpsc},
    task::{Context, Poll},
    time::{Duration, Instant},
};

type Scope = Pin<Box<dyn Future<Output = Option<wgpu::Error>> + Send>>;
type Wake = Arc<dyn Fn() + Send + Sync>;
#[derive(Default)]
pub(crate) struct NativeFrames {
    state: Arc<Mutex<State>>,
    worker: Arc<Worker>,
    changed: Arc<Condvar>,
}
#[derive(Default)]
enum State {
    #[default]
    Idle,
    Legacy,
    Submitting,
    Pending,
    Publishing(std::thread::ThreadId, bool),
    Unavailable,
    Closed,
}
pub(crate) struct LegacyRaster<'a>(&'a NativeFrames);
impl Drop for LegacyRaster<'_> {
    fn drop(&mut self) {
        self.0.finish(true);
    }
}
impl NativeFrames {
    pub(crate) fn checked(&self) -> Result<LegacyRaster<'_>, RasterizeFailure> {
        let mut state = self.state.lock().expect("native frames poisoned");
        if matches!(*state, State::Idle) {
            *state = State::Legacy;
            Ok(LegacyRaster(self))
        } else {
            Err(unavailable(
                "shared Core owns current native frame attribution",
            ))
        }
    }
    fn finish(&self, reusable: bool) {
        let mut state = self.state.lock().expect("native frames poisoned");
        if !matches!(*state, State::Closed) {
            *state = if reusable {
                State::Idle
            } else {
                State::Unavailable
            };
        }
    }
    fn schedule(&self, job: Job) {
        let job = Arc::new(Mutex::new(Some(job)));
        let worker_job = job.clone();
        if let Err(reason) = self.worker.schedule(Box::new(move || {
            worker_job.lock().unwrap().take().unwrap().run();
        })) {
            job.lock().unwrap().take().unwrap().refuse(&reason);
        }
    }
    fn close(&self) -> Result<(), String> {
        self.shutdown_handle().close()
    }
    fn shutdown_handle(&self) -> NativeFrameShutdown {
        NativeFrameShutdown {
            state: self.state.clone(),
            worker: self.worker.clone(),
            changed: self.changed.clone(),
        }
    }
}

#[path = "native_frame_queue.rs"]
mod queue;
pub(crate) use queue::NativeFrameShutdown;
use queue::Worker;
#[derive(Debug)]
pub enum NativeFrameStartError {
    Busy,
    Refused(RasterizeFailure),
}
fn unavailable(message: &str) -> RasterizeFailure {
    RenderCore::native_raster_unavailable(message)
}
struct Admission {
    operation: Option<LossOperation>,
    loss: Option<DeviceLossNotice>,
}
struct Qualification {
    core: Weak<RenderCore>,
    admission: Arc<Mutex<Admission>>,
    _cancellation: LossCancellation,
    scopes: Vec<Scope>,
    first_error: Option<RasterizeFailure>,
    deadline: Instant,
    admitted: Instant,
    sealed: Option<Instant>,
    worker_started: Option<Instant>,
    last_observation: Option<(usize, Option<(usize, u64)>)>,
}
impl Qualification {
    fn check(&self) -> Result<Arc<RenderCore>, RasterizeFailure> {
        if let Some(loss) = &self
            .admission
            .lock()
            .expect("native admission poisoned")
            .loss
        {
            return Err(RasterizeFailure::DeviceLost(loss.clone()));
        }
        let core = self
            .core
            .upgrade()
            .ok_or_else(|| unavailable("native frame Core retired"))?;
        if let Some(loss) = core.device_failure() {
            return Err(RasterizeFailure::DeviceLost(loss));
        }
        if Instant::now() >= self.deadline {
            let now = Instant::now();
            return Err(RenderCore::native_raster_unavailable(format!(
                "native frame qualification exceeded five seconds (admission={:?}, seal={:?}, worker={:?}, last scopes/backlog={})",
                now.duration_since(self.admitted),
                self.sealed.map(|at| now.duration_since(at)),
                self.worker_started.map(|at| now.duration_since(at)),
                self.last_observation
                    .map_or_else(|| "unknown".to_string(), |value| format!("{value:?}")),
            )));
        }
        Ok(core)
    }
    fn poll(&mut self) -> Option<Result<(), RasterizeFailure>> {
        let core = match self.check() {
            Ok(core) => core,
            Err(error) => return Some(Err(error)),
        };
        if let Err(error) = core.device().poll(wgpu::PollType::Poll) {
            return Some(Err(self.check().err().unwrap_or_else(|| {
                RenderCore::native_raster_unavailable(format!(
                    "native completion poll failed: {error}"
                ))
            })));
        }
        if let Err(error) = self.check() {
            return Some(Err(error));
        }
        self.observe_scopes();
        let mut empty = false;
        let mut last_backlog = None;
        for _ in 0..=256 {
            match core.renderer.poll_vello_curve_quality() {
                Ok(Some((0, 0))) => {
                    last_backlog = Some((0, 0));
                    empty = true;
                    break;
                },
                Ok(Some(counts)) => {
                    last_backlog = Some(counts);
                    break;
                },
                Ok(None) => {
                    return Some(Err(self.check().err().unwrap_or_else(|| {
                        unavailable("native Classic quality observer unavailable")
                    })));
                },
                Err(error) => {
                    self.first_error
                        .get_or_insert(RasterizeFailure::ProviderError(Arc::new(error)));
                },
            }
        }
        self.last_observation = Some((self.scopes.len(), last_backlog));
        if let Err(error) = self.check() {
            return Some(Err(error));
        }
        if empty && self.scopes.is_empty() {
            Some(self.first_error.take().map_or(Ok(()), Err))
        } else {
            None
        }
    }
    fn observe_scopes(&mut self) {
        let mut context = Context::from_waker(std::task::Waker::noop());
        let mut remaining = Vec::new();
        for mut scope in self.scopes.drain(..) {
            match scope.as_mut().poll(&mut context) {
                Poll::Ready(Some(error)) => {
                    self.first_error
                        .get_or_insert(RasterizeFailure::ProviderError(Arc::new(
                            netrender::VelloError::WgpuErrorFromScope(error),
                        )));
                },
                Poll::Ready(None) => {},
                Poll::Pending => remaining.push(scope),
            }
        }
        self.scopes = remaining;
    }
    fn wait(&mut self) -> Result<(), RasterizeFailure> {
        loop {
            if let Some(result) = self.poll() {
                return result;
            }
            // Only the shared completion worker enters this bounded wait. Do
            // not retain the Core while waiting or introduce a UI-side poll.
            let device = self.check()?.device().clone();
            let _ = device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_millis(5)),
            });
            std::thread::yield_now();
        }
    }
    fn drained(&self) -> bool {
        self.scopes.is_empty()
            && self
                .core
                .upgrade()
                .is_some_and(|core| core.renderer.vello_curve_quality_status() == Some((0, 0)))
    }
}
struct Completion {
    qualification: Option<Qualification>,
    result: Result<(), RasterizeFailure>,
    reusable: bool,
}
impl Drop for Completion {
    fn drop(&mut self) {
        if let Some(qualification) = self.qualification.take() {
            if let Some(core) = qualification.core.upgrade() {
                core.native_frames.finish(self.reusable);
            }
            // Admission destruction may invoke host wakes; no gate lock held.
            drop(qualification);
        }
    }
}
struct Job {
    qualification: Qualification,
    completed: Option<mpsc::Sender<Completion>>,
    wake: Wake,
}
#[path = "native_frame_worker.rs"]
mod worker;

/// Error scopes are thread-local. Seal/drop on the submitting thread. Batch
/// targets are private staging until AcceptedNativeFrame::publish succeeds.
#[must_use]
pub struct NativeFrameSubmission {
    core: Arc<RenderCore>,
    qualification: Option<Qualification>,
    scopes: Vec<wgpu::ErrorScopeGuard>,
    error: Arc<Mutex<Option<RasterizeFailure>>>,
    _thread: PhantomData<Rc<()>>,
    _submitting: LossOperation,
}
impl RenderCore {
    pub(crate) fn native_frame_shutdown_handle(&self) -> NativeFrameShutdown {
        self.native_frames.shutdown_handle()
    }
    /// Permanently stop native frame admission and join its completion worker.
    /// Drop all active submissions first. Call for every shared Core before
    /// releasing host GPU resources or exiting the process. This may wait for
    /// an already admitted qualification or active publication to finish; it
    /// does not poll on the caller. Reentrant closure returns an error.
    pub fn close_native_frames(&self) -> Result<(), String> {
        self.native_frames.close()
    }
    /// Status-only scheduling hint for other hosts on this Core. Raster
    /// admission remains authoritative; this does not acquire a drawing lease.
    /// A terminal attribution refusal is reported by the checked draw itself.
    pub fn native_frame_busy(&self) -> bool {
        !matches!(
            *self
                .native_frames
                .state
                .lock()
                .expect("native frames poisoned"),
            State::Idle | State::Unavailable
        )
    }
    pub(super) fn rasterize_native_batch(
        &self,
        surface: u64,
        scene: &netrender::Scene,
        width: u32,
        height: u32,
        clear: netrender::ColorLoad,
        scale: f32,
        error: Arc<Mutex<Option<RasterizeFailure>>>,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), RasterizeFailure> {
        let result = (|| {
            if let Some(loss) = self.device_failure() {
                return Err(RasterizeFailure::DeviceLost(loss));
            }
            let (texture, view) = self.scene_target(width, height);
            if let Some(loss) = self.device_failure() {
                return Err(RasterizeFailure::DeviceLost(loss));
            }
            let result = self
                .renderer
                .render_vello_scaled_for_checked(surface, scene, &view, clear, scale);
            if let Some(loss) = self.device_failure() {
                return Err(RasterizeFailure::DeviceLost(loss));
            }
            result.map_err(|error| RasterizeFailure::ProviderError(Arc::new(error)))?;
            Ok((texture, view))
        })();
        if let Err(failure) = &result {
            error
                .lock()
                .expect("native batch error poisoned")
                .get_or_insert(failure.clone());
        }
        result
    }
    /// Begin one exclusive native batch, with error scopes and loss admission.
    /// Use the batch's explicit rasterize method; legacy checked drawing refuses
    /// while the batch is active/pending. Raw drawing must not interleave.
    pub fn try_begin_native_frame(
        self: &Arc<Self>,
    ) -> Result<NativeFrameSubmission, NativeFrameStartError> {
        if let Some(loss) = self.device_failure() {
            return Err(NativeFrameStartError::Refused(
                RasterizeFailure::DeviceLost(loss),
            ));
        }
        let mut state = self
            .native_frames
            .state
            .lock()
            .expect("native frames poisoned");
        match &*state {
            State::Idle => {},
            State::Unavailable | State::Closed => {
                return Err(NativeFrameStartError::Refused(unavailable(
                    "native frame attribution unavailable",
                )));
            },
            _ => return Err(NativeFrameStartError::Busy),
        }
        *state = State::Submitting;
        drop(state);
        if self.renderer.vello_curve_quality_status() != Some((0, 0)) {
            self.native_frames.finish(true);
            return Err(NativeFrameStartError::Refused(unavailable(
                "unattributed native Classic quality backlog",
            )));
        }
        let submitting = self.loss.begin_operation().map_err(|loss| {
            self.native_frames.finish(true);
            NativeFrameStartError::Refused(RasterizeFailure::DeviceLost(loss))
        })?;
        let operation = self.loss.begin_operation().map_err(|loss| {
            self.native_frames.finish(true);
            NativeFrameStartError::Refused(RasterizeFailure::DeviceLost(loss))
        })?;
        let admission = Arc::new(Mutex::new(Admission {
            operation: Some(operation),
            loss: None,
        }));
        let weak = Arc::downgrade(&admission);
        let cancellation = self.loss.register_cancellation(Arc::new(move |loss| {
            let operation = weak.upgrade().and_then(|admission| {
                let mut admission = admission.lock().expect("native admission poisoned");
                admission.loss = Some(loss.clone());
                admission.operation.take()
            });
            drop(operation);
        }));
        let scopes = [
            wgpu::ErrorFilter::OutOfMemory,
            wgpu::ErrorFilter::Internal,
            wgpu::ErrorFilter::Validation,
        ]
        .into_iter()
        .map(|filter| self.device().push_error_scope(filter))
        .collect();
        Ok(NativeFrameSubmission {
            core: self.clone(),
            qualification: Some(Qualification {
                core: Arc::downgrade(self),
                admission,
                _cancellation: cancellation,
                scopes: Vec::new(),
                first_error: None,
                deadline: Instant::now() + Duration::from_secs(5),
                admitted: Instant::now(),
                sealed: None,
                worker_started: None,
                last_observation: None,
            }),
            scopes,
            error: Arc::new(Mutex::new(None)),
            _thread: PhantomData,
            _submitting: submitting,
        })
    }
}
impl NativeFrameSubmission {
    pub fn rasterize(
        &self,
        surface: u64,
        scene: &netrender::Scene,
        width: u32,
        height: u32,
        clear: netrender::ColorLoad,
        scale: f32,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), RasterizeFailure> {
        self.core.rasterize_native_batch(
            surface,
            scene,
            width,
            height,
            clear,
            scale,
            self.error.clone(),
        )
    }
    fn finish_scopes(&mut self) -> Qualification {
        let mut qualification = self
            .qualification
            .take()
            .expect("native batch already sealed");
        qualification.sealed = Some(Instant::now());
        qualification.first_error = self
            .error
            .lock()
            .expect("native frame error poisoned")
            .take();
        while let Some(scope) = self.scopes.pop() {
            qualification.scopes.push(Box::pin(scope.pop()));
        }
        qualification
    }
    pub fn seal<T>(self, value: T) -> PendingNativeFrame<T> {
        self.seal_with_wake(value, Arc::new(|| {}))
    }
    pub fn seal_with_wake<T>(mut self, value: T, wake: Wake) -> PendingNativeFrame<T> {
        let qualification = self.finish_scopes();
        let admission = Arc::downgrade(&qualification.admission);
        let (sender, receiver) = mpsc::channel();
        *self
            .core
            .native_frames
            .state
            .lock()
            .expect("native frames poisoned") = State::Pending;
        self.core.native_frames.schedule(Job {
            qualification,
            completed: Some(sender),
            wake,
        });
        PendingNativeFrame {
            admission,
            core: Arc::downgrade(&self.core),
            completed: Some(receiver),
            value: Some(value),
        }
    }
}
impl Drop for NativeFrameSubmission {
    fn drop(&mut self) {
        if self.qualification.is_some() {
            let qualification = self.finish_scopes();
            *self
                .core
                .native_frames
                .state
                .lock()
                .expect("native frames poisoned") = State::Pending;
            self.core.native_frames.schedule(Job {
                qualification,
                completed: None,
                wake: Arc::new(|| {}),
            });
        }
    }
}
/// Collection only reads a channel; device polling and waits run off-thread.
#[must_use]
pub struct PendingNativeFrame<T> {
    admission: Weak<Mutex<Admission>>,
    core: Weak<RenderCore>,
    completed: Option<mpsc::Receiver<Completion>>,
    value: Option<T>,
}
impl<T> PendingNativeFrame<T> {
    pub fn poll(&mut self) -> Option<Result<AcceptedNativeFrame<T>, RasterizeFailure>> {
        let receiver = self.completed.as_ref()?;
        let loss = self.admission.upgrade().and_then(|admission| {
            admission
                .lock()
                .expect("native admission poisoned")
                .loss
                .clone()
        });
        if let Some(loss) = loss {
            drop(self.value.take());
            self.completed.take();
            return Some(Err(RasterizeFailure::DeviceLost(loss)));
        }
        let mut completion = match receiver.try_recv() {
            Ok(completion) => completion,
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => {
                if let Some(core) = self.core.upgrade() {
                    core.native_frames.finish(false);
                }
                self.completed.take();
                drop(self.value.take());
                return Some(Err(unavailable("native completion worker disconnected")));
            },
        };
        self.completed.take();
        let result = completion
            .qualification
            .as_ref()
            .expect("completion qualification")
            .check()
            .map(|_| ())
            .and(completion.result.clone());
        match result {
            Ok(()) => Some(Ok(AcceptedNativeFrame {
                qualification: completion.qualification.take(),
                value: self.value.take(),
            })),
            Err(error) => {
                drop(self.value.take());
                Some(Err(error))
            },
        }
    }
}
impl<T> Drop for PendingNativeFrame<T> {
    fn drop(&mut self) {
        drop(self.value.take());
    }
}
/// Original admission remains alive through final publication, rechecking loss.
#[must_use]
pub struct AcceptedNativeFrame<T> {
    qualification: Option<Qualification>,
    value: Option<T>,
}
impl<T> AcceptedNativeFrame<T> {
    pub fn publish<R>(mut self, publish: impl FnOnce(T) -> R) -> Result<R, RasterizeFailure> {
        let core = self
            .qualification
            .as_ref()
            .expect("native frame already published")
            .check()?;
        let _publication = core.native_frames.begin_publication()?;
        // Publication may access GPU resources. Cancellation may release an
        // idle lease, but recovery must await this active callback interval.
        let _publishing = core
            .loss
            .begin_operation()
            .map_err(RasterizeFailure::DeviceLost)?;
        let result = publish(self.value.take().expect("native frame value"));
        self.qualification.as_ref().unwrap().check()?;
        Ok(result)
    }
}
impl<T> Drop for AcceptedNativeFrame<T> {
    fn drop(&mut self) {
        drop(self.value.take());
        if let Some(qualification) = self.qualification.take() {
            if let Some(core) = qualification.core.upgrade() {
                core.native_frames.finish(true);
            }
            drop(qualification);
        }
    }
}

#[cfg(test)]
#[path = "native_frame_tests.rs"]
mod tests;
