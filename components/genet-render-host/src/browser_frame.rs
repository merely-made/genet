// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A single actual Core owns scopes and shader-counter attribution until retirement.
//! Host state, textures and DOM borrows are deliberately absent from this module.

use std::{cell::RefCell, future::Future as _, sync::{Arc, Mutex, Weak}, sync::atomic::{AtomicU8, Ordering}, task::{Context, Poll}, time::Duration};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::{RenderCore, RasterizeFailure, DeviceLossNotice};
use crate::recovery::{LossCancellation, LossOperation};

const IDLE: u8 = 0;
const OWNED: u8 = 1;
const UNAVAILABLE: u8 = 2;
const LEGACY: u8 = 3;

#[derive(Default)]
pub(crate) struct FrameGate {
    state: AtomicU8,
}
impl FrameGate {
    pub(crate) fn acquire_legacy(&self) -> Result<(), RasterizeFailure> {
        self.state.compare_exchange(IDLE, LEGACY, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ()).map_err(|_| legacy_refusal())
    }
}

/// A retained Renderer borrow keeps legacy access exclusive for its lifetime.
/// It cannot be used to obtain a long-lived raw reference after releasing the gate.
pub struct RendererAccess<'a> { core: &'a RenderCore }
impl std::ops::Deref for RendererAccess<'_> {
    type Target = netrender::Renderer;
    fn deref(&self) -> &Self::Target { &self.core.renderer }
}
impl Drop for RendererAccess<'_> {
    fn drop(&mut self) {
        let _ = self.core.browser_frame_gate.state.compare_exchange(LEGACY, IDLE, Ordering::AcqRel, Ordering::Acquire);
    }
}
impl RenderCore {
    pub fn try_renderer(&self) -> Result<RendererAccess<'_>, RasterizeFailure> {
        #[cfg(feature = "without-classic-vello")]
        self.unavailable();
        if let Some(loss) = self.device_failure() { return Err(RasterizeFailure::DeviceLost(loss)); }
        self.browser_frame_gate.acquire_legacy()?;
        Ok(RendererAccess { core:self })
    }

    /// Owned-resource cleanup submits no work and observes no quality FIFO.
    /// It is also valid after loss, unlike acquiring a live renderer accessor.
    pub fn retire_owned_external_image(&self, key:netrender::ImageKey) {
        #[cfg(feature = "without-classic-vello")]
        self.unavailable();
        self.renderer.unregister_external_image(key);
    }
    pub fn retire_owned_image(&self, key:netrender::ImageKey) {
        #[cfg(feature = "without-classic-vello")]
        self.unavailable();
        self.renderer.unregister_image_vello(key);
    }
    pub fn retire_owned_fragment(&self, id:u64) {
        #[cfg(feature = "without-classic-vello")]
        self.unavailable();
        self.renderer.remove_fragment(id);
    }
}

struct Admission {
    operation: Option<LossOperation>,
    loss: Option<DeviceLossNotice>,
}
struct Lease {
    core: Weak<RenderCore>,
    gate: Arc<FrameGate>,
    admission: Arc<Mutex<Admission>>,
    _cancellation: LossCancellation,
    deadline: web_time::Instant,
    generation: u64,
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    capture_identity:crate::SurfaceCaptureFrameIdentity,
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    quality_phase:u32,
}
impl Lease {
    fn release(&mut self, reusable: bool) {
        let operation = self.admission.lock().expect("frame admission poisoned").operation.take();
        self.gate.state.store(if reusable { IDLE } else { UNAVAILABLE }, Ordering::Release);
        drop(operation); // Loss wake must run outside the admission lock.
    }
    fn core(&self) -> Result<Arc<RenderCore>, RasterizeFailure> {
        if let Some(loss) = self.admission.lock().expect("frame admission poisoned").loss.clone() {
            return Err(RasterizeFailure::DeviceLost(loss));
        }
        let core = self.core.upgrade().ok_or_else(|| unavailable("frame Core was retired"))?;
        if let Some(loss) = core.device_failure() {
            return Err(RasterizeFailure::DeviceLost(loss));
        }
        Ok(core)
    }
    fn check(&self) -> Result<Arc<RenderCore>, RasterizeFailure> {
        let core = self.core()?; // Actual loss has priority over clock/provider errors.
        if web_time::Instant::now() >= self.deadline {
            return Err(unavailable("browser current-frame acceptance exceeded five seconds"));
        }
        Ok(core)
    }
}

pub(crate) fn legacy_refusal() -> RasterizeFailure { unavailable("shared Core owns or refuses current-frame quality attribution") }

fn unavailable(message: &str) -> RasterizeFailure {
    gpu_error("internal", message)
}
fn gpu_error(kind: &str, message: &str) -> RasterizeFailure {
    let description: String = message.chars().take(1024).collect();
    let source = || Box::new(std::io::Error::other(description.clone())) as Box<dyn std::error::Error + Send + Sync>;
    let error = match kind {
        "validation" => wgpu::Error::Validation { source: source(), description },
        "out-of-memory" => wgpu::Error::OutOfMemory { source: source() },
        _ => wgpu::Error::Internal { source: source(), description },
    };
    RasterizeFailure::ProviderError(Arc::new(netrender::VelloError::WgpuErrorFromScope(error)))
}

// Only raw scopes are used: mixing these pops with Rust ErrorScopeGuards
// would corrupt wgpu's scope-depth bookkeeping. allSettled attaches rejection
// handlers immediately, including when the Rust owner is disposed afterward.
#[wasm_bindgen(inline_js = r#"
export function genet_begin_frame_scopes(device) {
  const token = {device, pushed: [], closed:false};
  try {
    for (const filter of ['out-of-memory','internal','validation']) {
      device.pushErrorScope(filter); token.pushed.push(filter);
    }
    return token;
  } catch (error) {
    const cleanup = [];
    while (token.pushed.length) {
      token.pushed.pop();
      try { cleanup.push(device.popErrorScope()); } catch (_) {}
    }
    Promise.allSettled(cleanup); throw error;
  }
}
export function genet_seal_frame_scopes(token) {
  if (token.closed) throw new Error('frame scopes already sealed');
  token.closed = true;
  const entries = [];
  while (token.pushed.length) {
    const filter = token.pushed.pop();
    try { entries.push({filter, promise:token.device.popErrorScope()}); }
    catch (error) { entries.push({filter, promise:Promise.reject(error)}); }
  }
  try { entries.push({filter:'queue', promise:token.device.queue.onSubmittedWorkDone()}); }
  catch (error) { entries.push({filter:'queue', promise:Promise.reject(error)}); }
  return Promise.allSettled(entries.map(e => e.promise)).then(results => results.map((r,i) => ({
    filter:entries[i].filter, status:r.status,
    error:r.status === 'fulfilled' ? r.value != null : true,
    message:String(r.status === 'rejected' ? r.reason : r.value?.message ?? '').slice(0,1024)
  })));
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)] fn genet_begin_frame_scopes(device: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)] fn genet_seal_frame_scopes(token: &JsValue) -> Result<js_sys::Promise, JsValue>;
}

/// Acquired before any changed-source staging, shadow construction or raster.
/// One synchronous submission closure covers the actual whole host frame.
#[must_use]
pub struct BrowserFrameSubmission {
    lease: Option<Lease>,
    scopes: Option<std::pin::Pin<Box<JsFuture>>>,
    submission_error: Option<RasterizeFailure>,
}

#[derive(Clone, Debug)]
pub enum BrowserFrameStartError {
    Busy,
    Refused(RasterizeFailure),
}

/// Available only in the owner's synchronous closure. Checked legacy APIs
/// refuse while occupied, including unrelated reentrant host calls.
pub struct BrowserFrameCommands<'a> {
    core: &'a RenderCore,
    lease: &'a Lease,
    // Borrowed only by the synchronous closure. Never retained across await.
    first_error: &'a RefCell<Option<RasterizeFailure>>,
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    capture_context:crate::browser_surface_readback::SurfaceCaptureContext,
}
impl BrowserFrameCommands<'_> {
    fn observe<T>(&self, result:Result<T,RasterizeFailure>) -> Result<T,RasterizeFailure> {
        if let Err(error) = &result {
            self.first_error.borrow_mut().get_or_insert(error.clone());
        }
        result
    }
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    pub(crate) fn matches_core(&self, core:&RenderCore) -> bool { std::ptr::eq(self.core,core) }
    /// A typed capture result is distinct from the renderer/frame outcome.
    /// The exact enclosing phase2 still qualifies copy scopes/queue/FIFO.
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    pub fn start_acquired_surface_readback(
        &self, frame:&wgpu::SurfaceTexture,configuration:&crate::SurfaceCaptureConfiguration,
        budget:Arc<crate::SurfaceCaptureBudget>,
    ) -> Result<crate::PendingSurfaceReadback,crate::SurfaceReadbackFailure> {
        crate::PendingSurfaceReadback::start(self.capture_context.clone(),frame,configuration,budget)
    }
    /// Original frame loss/deadline admission, including immediately before present.
    pub fn check_current(&self) -> Result<(),RasterizeFailure> { self.observe(self.lease.check().map(|_| ())) }
    pub fn generation(&self) -> u64 { self.core.generation() }
    pub fn device(&self) -> &wgpu::Device { self.core.device() }
    pub fn queue(&self) -> &wgpu::Queue { self.core.queue() }
    pub fn renderer(&self) -> &netrender::Renderer { &self.core.renderer }
    /// Compose inside this actual owner's synchronous scoped submission.
    /// GPU scope and queue outcomes are collected by the enclosing pending
    /// frame; a synchronous Ok does not prove completion or presentation.
    pub fn compose_external_texture(
        &self, source: &wgpu::TextureView, target: &wgpu::TextureView,
        format: wgpu::TextureFormat, width: u32, height: u32,
        placement: netrender::ExternalTexturePlacement,
    ) -> Result<(), RasterizeFailure> {
        self.observe((|| {
            if let Some(loss) = self.core.device_failure() {
                return Err(RasterizeFailure::DeviceLost(loss));
            }
            self.core.renderer.compose_external_texture(source, target, format, width, height, placement);
            if let Some(loss) = self.core.device_failure() {
                return Err(RasterizeFailure::DeviceLost(loss));
            }
            Ok(())
        })())
    }
    pub fn rasterize_scaled_for(
        &self, surface:u64, scene:&netrender::Scene, width:u32, height:u32,
        clear:netrender::ColorLoad, scale:f32,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), RasterizeFailure> {
        self.observe((|| {
            if let Some(loss) = self.core.device_failure() { return Err(RasterizeFailure::DeviceLost(loss)); }
            let (texture, view) = self.core.scene_target(width, height);
            if let Some(loss) = self.core.device_failure() { return Err(RasterizeFailure::DeviceLost(loss)); }
            let rendered = self.core.renderer.render_vello_scaled_for_checked(surface, scene, &view, clear, scale);
            if let Some(loss) = self.core.device_failure() { return Err(RasterizeFailure::DeviceLost(loss)); }
            rendered.map_err(|error| RasterizeFailure::ProviderError(Arc::new(error)))?;
            Ok((texture, view))
        })())
    }
}
impl RenderCore {
    pub fn try_begin_browser_frame(self: &Arc<Self>) -> Result<BrowserFrameSubmission, BrowserFrameStartError> {
        #[cfg(feature = "without-classic-vello")]
        self.unavailable();
        if let Some(loss) = self.device_failure() { return Err(BrowserFrameStartError::Refused(RasterizeFailure::DeviceLost(loss))); }
        self.browser_frame_gate.state.compare_exchange(IDLE, OWNED, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|state| if state == OWNED || state == LEGACY { BrowserFrameStartError::Busy } else { BrowserFrameStartError::Refused(unavailable("shared Core quality outcome is unavailable")) })?;
        self.begin_browser_frame_owned().map_err(BrowserFrameStartError::Refused)
    }
    fn begin_browser_frame_owned(self: &Arc<Self>) -> Result<BrowserFrameSubmission, RasterizeFailure> {
        #[cfg(feature = "without-classic-vello")]
        self.unavailable();
        let operation = match self.loss.begin_operation() {
            Ok(operation) => operation,
            Err(loss) => { self.browser_frame_gate.state.store(IDLE, Ordering::Release); return Err(RasterizeFailure::DeviceLost(loss)); }
        };
        let admission = Arc::new(Mutex::new(Admission { operation: Some(operation), loss: None }));
        let weak_admission = Arc::downgrade(&admission);
        let gate = self.browser_frame_gate.clone();
        let callback_gate = gate.clone();
        let cancellation = self.loss.register_cancellation(Arc::new(move |loss| {
            let operation = weak_admission.upgrade().and_then(|admission| {
                let mut admission = admission.lock().expect("frame admission poisoned");
                admission.loss = Some(loss.clone()); admission.operation.take()
            });
            callback_gate.state.store(IDLE, Ordering::Release);
            drop(operation);
        }));
        let mut lease = Lease { core: Arc::downgrade(self), gate, admission, _cancellation:cancellation, deadline:web_time::Instant::now() + Duration::from_secs(5), generation:self.generation(),
            #[cfg(feature = "browser-surface-readback-diagnostics")]
            capture_identity:crate::SurfaceCaptureFrameIdentity::new(),
            #[cfg(feature = "browser-surface-readback-diagnostics")]
            quality_phase:1,
        };
        let setup = (|| {
            lease.check()?;
            match self.renderer.poll_vello_curve_quality() {
                Ok(Some((0,0))) => {},
                Err(error) => return Err(RasterizeFailure::ProviderError(Arc::new(error))),
                _ => return Err(unavailable("unattributed or unsupported Classic quality backlog")),
            }
            self.device().as_webgpu().ok_or_else(|| unavailable("current-frame acceptance requires actual WebGPU"))?;
            Ok(())
        })();
        match setup {
            Ok(()) => Ok(BrowserFrameSubmission { lease:Some(lease), scopes:None, submission_error:None }),
            Err(error) => { let error = lease.core().err().unwrap_or(error); lease.release(false); Err(error) }
        }
    }
}
impl BrowserFrameSubmission {
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    pub fn frame_identity(&self) -> Option<crate::SurfaceCaptureFrameIdentity> {
        self.lease.as_ref().map(|lease|lease.capture_identity.clone())
    }
    /// No callback, borrow or error scope escapes across an await.
    pub fn submit<T>(&mut self, submit: impl FnOnce(&BrowserFrameCommands<'_>) -> Result<T, RasterizeFailure>) -> Result<T, RasterizeFailure> {
        let lease = self.lease.as_ref().expect("frame already sealed");
        if self.scopes.is_some() { return Err(unavailable("frame submission is single-use")); }
        let core = lease.check()?;
        let device = core.device().as_webgpu().ok_or_else(|| unavailable("current-frame acceptance requires actual WebGPU"))?;
        let token = genet_begin_frame_scopes(device.as_ref()).map_err(|_| unavailable("GPU scope admission threw"))?;
        let first_error = RefCell::new(self.submission_error.take());
        let result = submit(&BrowserFrameCommands { core:&core, lease, first_error:&first_error,
            #[cfg(feature = "browser-surface-readback-diagnostics")]
            capture_context:crate::browser_surface_readback::SurfaceCaptureContext {
                core:lease.core.clone(),loss:core.loss.clone(),generation:lease.generation,quality_phase:lease.quality_phase,
                frame:lease.capture_identity.clone(),deadline:lease.deadline,
            },
        });
        self.submission_error = first_error.into_inner();
        if let Err(error) = &result { self.submission_error.get_or_insert(error.clone()); }
        let promise = match genet_seal_frame_scopes(&token) {
            Ok(promise) => promise,
            Err(_) => { self.lease.take().expect("frame lease").release(false); return Err(unavailable("GPU scope seal threw")); }
        };
        self.scopes = Some(Box::pin(JsFuture::from(promise)));
        let lease = self.lease.as_ref().expect("frame lease");
        if let Err(error) = lease.check() {
            self.submission_error.get_or_insert(error.clone()); return Err(error);
        }
        result
    }
    pub fn seal(mut self) -> Result<PendingBrowserFrame, RasterizeFailure> {
        let future = self.scopes.take().ok_or_else(|| unavailable("frame has no sealed submission"))?;
        Ok(PendingBrowserFrame { lease:self.lease.take(), scopes:Some(future), scopes_ready:false, first_error:self.submission_error.take(),
            #[cfg(feature = "browser-surface-readback-diagnostics")]
            diagnostics:None,
        })
    }
}
impl Drop for BrowserFrameSubmission {
    fn drop(&mut self) {
        let Some(mut lease) = self.lease.take() else { return };
        match self.scopes.take() {
            Some(future) => retire(PendingBrowserFrame { lease:Some(lease), scopes:Some(future), scopes_ready:false, first_error:self.submission_error.take(),
            #[cfg(feature = "browser-surface-readback-diagnostics")]
            diagnostics:None,
        }),
            None => lease.release(true),
        }
    }
}

/// Poll on browser turns, including while hidden. Accepted construction is private.
#[must_use]
pub struct PendingBrowserFrame {
    lease: Option<Lease>,
    scopes: Option<std::pin::Pin<Box<JsFuture>>>,
    scopes_ready: bool,
    first_error: Option<RasterizeFailure>,
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    diagnostics:Option<crate::BrowserFrameQualityObservation>,
}
impl PendingBrowserFrame {
    /// Yield to browser turns while the submitted GPU work and Classic quality
    /// ledger settle. `poll` enforces the original five-second deadline and
    /// gives device loss priority over timeout.
    pub async fn wait(mut self) -> Result<AcceptedBrowserFrame, RasterizeFailure> {
        loop {
            if let Some(result) = self.poll() { return result; }
            gloo_timers::future::TimeoutFuture::new(16).await;
        }
    }
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    pub fn quality_observation(&self) -> Option<crate::BrowserFrameQualityObservation> { self.diagnostics.clone() }
    pub fn poll(&mut self) -> Option<Result<AcceptedBrowserFrame, RasterizeFailure>> {
        let lease = self.lease.as_ref()?;
        #[cfg(feature = "browser-surface-readback-diagnostics")]
        if self.diagnostics.is_none() {
            self.diagnostics=Some(crate::BrowserFrameQualityObservation {
                frame:lease.capture_identity.clone(),generation:lease.generation,phase:lease.quality_phase,
                rows:None,actual_row_count:0,wrapper_rejected:false,zero_classic_fifo_observed:false,
            });
        }
        let core = match lease.check() { Ok(core) => core, Err(error) => { self.finish(false); return Some(Err(error)); } };
        if !self.scopes_ready {
            let mut cx = Context::from_waker(std::task::Waker::noop());
            if let Poll::Ready(result) = self.scopes.as_mut().expect("scope future").as_mut().poll(&mut cx) {
                self.scopes.take(); self.scopes_ready = true;
                if let Ok(rows) = result {
                    let rows = js_sys::Array::from(&rows);
                    #[cfg(feature = "browser-surface-readback-diagnostics")]
                    if let Some(observation)=self.diagnostics.as_mut() {
                        observation.actual_row_count=rows.length();
                        observation.rows=Some(rows.iter().take(4).map(|row| {
                            let get=|name| js_sys::Reflect::get(&row,&JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED);
                            crate::BrowserFrameScopeObservation {
                                filter:get("filter").as_string().map(|value|value.chars().take(64).collect()),
                                status:get("status").as_string().map(|value|value.chars().take(32).collect()),
                                error:get("error").as_bool(),
                                message:get("message").as_string().unwrap_or_default().chars().take(1024).collect(),
                            }
                        }).collect());
                    }
                    if rows.length() != 4 { self.first_error.get_or_insert(unavailable("incomplete GPU scope/queue ledger")); }
                    for row in rows.iter() {
                        let get = |name| js_sys::Reflect::get(&row, &JsValue::from_str(name)).unwrap_or(JsValue::UNDEFINED);
                        if get("status").as_string().as_deref() != Some("fulfilled") {
                            self.first_error.get_or_insert(unavailable("actual GPU scope/queue promise rejected"));
                        } else if get("error").as_bool() != Some(false) {
                            self.first_error.get_or_insert(gpu_error(&get("filter").as_string().unwrap_or_default(), &get("message").as_string().unwrap_or_default()));
                        }
                    }
                    let filters: Vec<_> = rows.iter().map(|row| js_sys::Reflect::get(&row, &JsValue::from_str("filter")).ok().and_then(|v| v.as_string())).collect();
                    if filters != ["validation", "internal", "out-of-memory", "queue"].map(|filter| Some(filter.to_owned())) {
                        self.first_error.get_or_insert(unavailable("GPU scope ledger has foreign/missing filters"));
                    }
                } else {
                    #[cfg(feature = "browser-surface-readback-diagnostics")]
                    if let Some(observation)=self.diagnostics.as_mut() { observation.wrapper_rejected=true; }
                    self.first_error.get_or_insert(unavailable("GPU scope wrapper rejected"));
                }
            }
        }
        // Provider FIFO is bounded at 256. Each error consumes one failed
        // frame; do not return it while later counters remain unattributed.
        let mut zero = false;
        for _ in 0..=256 {
            match core.renderer.poll_vello_curve_quality() {
                Ok(Some((0,0))) => { zero = true; break },
                Ok(Some(_)) => break,
                Ok(None) => { self.first_error.get_or_insert(unavailable("Classic quality observer unavailable")); break },
                Err(error) => { self.first_error.get_or_insert(RasterizeFailure::ProviderError(Arc::new(error))); },
            }
        }
        if let Err(error) = self.lease.as_ref().expect("frame lease").check() { self.finish(false); return Some(Err(error)); }
        #[cfg(feature = "browser-surface-readback-diagnostics")]
        if let Some(observation)=self.diagnostics.as_mut() { observation.zero_classic_fifo_observed=zero; }
        if !(zero && self.scopes_ready) { return None; }
        if let Some(error) = self.first_error.take() { self.finish(true); Some(Err(error)) }
        else { Some(Ok(AcceptedBrowserFrame { lease:self.lease.take(),
            #[cfg(feature = "browser-surface-readback-diagnostics")]
            diagnostics:self.diagnostics.take(),
        })) }
    }
    fn finish(&mut self, reusable: bool) {
        if let Some(mut lease) = self.lease.take() { lease.release(reusable); }
    }
}
impl Drop for PendingBrowserFrame {
    fn drop(&mut self) {
        if self.lease.is_some() {
            retire(PendingBrowserFrame { lease:self.lease.take(), scopes:self.scopes.take(), scopes_ready:self.scopes_ready, first_error:self.first_error.take(),
                #[cfg(feature = "browser-surface-readback-diagnostics")]
                diagnostics:self.diagnostics.take(),
            });
        }
    }
}
fn retire(mut pending: PendingBrowserFrame) {
    // Bounded cleanup owns only a weak Core and monitor admission. Disposal
    // never keeps the host/old renderer alive; loss cancels admission eagerly.
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            if let Some(result) = pending.poll() { drop(result); break }
            gloo_timers::future::TimeoutFuture::new(16).await;
        }
    });
}

/// Hold attribution through publication and the final surface handoff.
#[must_use]
pub struct AcceptedBrowserFrame {
    lease:Option<Lease>,
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    diagnostics:Option<crate::BrowserFrameQualityObservation>,
}
impl AcceptedBrowserFrame {
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    pub fn quality_observation(&self) -> Option<crate::BrowserFrameQualityObservation> { self.diagnostics.clone() }
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    pub fn frame_identity(&self) -> Option<crate::SurfaceCaptureFrameIdentity> {
        self.lease.as_ref().map(|lease|lease.capture_identity.clone())
    }
    /// Detach only a copy belonging to this exact accepted frame, retaining
    /// original deadline/loss checks without holding renderer attribution.
    #[cfg(feature = "browser-surface-readback-diagnostics")]
    pub fn accept_surface_readback(
        &self, pending:&mut Option<crate::PendingSurfaceReadback>,
    ) -> Result<crate::AcceptedSurfaceReadback,crate::SurfaceReadbackFailure> {
        let lease=self.lease.as_ref().ok_or(crate::SurfaceReadbackFailure::WrongFrame)?;
        let core=lease.check().map_err(crate::SurfaceReadbackFailure::FrameRejected)?;
        pending.as_ref().ok_or(crate::SurfaceReadbackFailure::AlreadyCollected)?
            .check_accept(&core,&lease.capture_identity,lease.generation,lease.quality_phase)?;
        Ok(crate::AcceptedSurfaceReadback::new(pending.take().expect("validated pending capture")))
    }
    /// Continue under the same actual Core owner after the first group was
    /// qualified. The original admission, cancellation and five-second
    /// deadline remain in force; no gate is reacquired or deadline restarted.
    /// The next submit/seal/poll uses the existing scope, queue and quality
    /// FIFO machinery. Host publication must await that next acceptance.
    /// This does not establish visible rollback or physical display completion.
    pub fn into_continuation(mut self) -> Result<BrowserFrameSubmission, RasterizeFailure> {
        self.check_current()?;
        #[cfg(feature = "browser-surface-readback-diagnostics")]
        {
            let lease=self.lease.as_mut().expect("accepted lease");
            lease.quality_phase=lease.quality_phase.checked_add(1).ok_or_else(||unavailable("quality phase exhausted"))?;
        }
        Ok(BrowserFrameSubmission {
            lease: self.lease.take(),
            scopes: None,
            submission_error: None,
        })
    }
    pub fn check_current(&self) -> Result<(), RasterizeFailure> {
        self.lease.as_ref().expect("accepted frame collected").check().map(|_| ())
    }
    pub fn generation(&self) -> u64 {
        self.lease.as_ref().expect("accepted frame collected").generation
    }
    pub fn promote_external_images(&self, moves:&[netrender::ExternalImagePromotion]) -> Result<(), RasterizeFailure> {
        let core = self.lease.as_ref().expect("accepted frame collected").check()?;
        core.renderer.promote_external_images(moves).map_err(|error| unavailable(&format!("image publication refused: {error:?}")))
    }
    /// Legacy synchronous, unscoped composition. Ok reports submission and
    /// loss checks, not GPU scope or queue completion. Qualified final-target
    /// work must use into_continuation and BrowserFrameCommands composition,
    /// then seal and poll that continuation before host publication.
    pub fn compose_external_texture(
        &self, source:&wgpu::TextureView, target:&wgpu::TextureView,
        format:wgpu::TextureFormat, width:u32, height:u32,
        placement:netrender::ExternalTexturePlacement,
    ) -> Result<(), RasterizeFailure> {
        let core = self.lease.as_ref().expect("accepted frame collected").check()?;
        core.renderer.compose_external_texture(source,target,format,width,height,placement);
        if let Some(loss) = core.device_failure() { return Err(RasterizeFailure::DeviceLost(loss)); }
        Ok(())
    }
}
impl Drop for AcceptedBrowserFrame {
    fn drop(&mut self) {
        if let Some(mut lease) = self.lease.take() { lease.release(true); }
    }
}
