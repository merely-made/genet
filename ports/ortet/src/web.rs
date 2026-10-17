// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The raw `wasm32` Ortet host.
//!
//! [`start`] receives a real browser canvas, boots the target-neutral
//! `RenderCore`, and connects DOM input to the same retained Livery session
//! path that the native host uses. It intentionally has no winit, AccessKit,
//! Cambium, or Mere dependency: browser ownership stops at the canvas and DOM
//! events. Fetches in an older navigation are ignored when a newer generation
//! wins; aborting their browser requests is a separate follow-up.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use document_session_api::session_engine::{
    DocumentSession, SessionButtonState, SessionDocumentRequest, SessionEffect,
    SessionEncodedFormSubmission, SessionEngine, SessionInput, SessionInputResult, SessionKey,
    SessionModifiers, SessionPointerButton, SessionScrollKey, SessionSpawnRequest,
};
use genet_document_resources::ResourceLimits;
use genet_documents::{LiveryResourcePreparation, LiverySessionEngine, LocalFetcher};
use genet_host_api::navigation::resolve_href;
use genet_host_api::{ResourceFetchPolicy, ResourceResponse};
use genet_render_host::{
    BrowserFrameStartError, PendingBrowserFrame, RasterizeFailure, RenderCore, SurfaceAcquire,
    WindowSurface,
};
#[cfg(feature = "browser-frame-fixture")]
use genet_render_host::BrowserFrameSubmission;
use netrender::{ColorLoad, ExternalTexturePlacement, NetrenderOptions, Scene};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    Event, EventTarget, HtmlCanvasElement, KeyboardEvent, PointerEvent, Response, WheelEvent,
};

/// Browser-host bounds for one navigation.  The defaults retain the shared
/// host response cap and make the aggregate/resource caps explicit here.
#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct BrowserResourceLimits {
    max_response_bytes: u32,
    max_total_bytes: u32,
    max_resources: u32,
}

impl Default for BrowserResourceLimits {
    fn default() -> Self {
        let policy = ResourceFetchPolicy::default();
        Self {
            max_response_bytes: policy.max_response_bytes.min(u32::MAX as usize) as u32,
            max_total_bytes: policy
                .max_response_bytes
                .saturating_mul(4)
                .min(u32::MAX as usize) as u32,
            max_resources: 64,
        }
    }
}

#[wasm_bindgen]
impl BrowserResourceLimits {
    #[wasm_bindgen(constructor)]
    pub fn new(max_response_bytes: u32, max_total_bytes: u32, max_resources: u32) -> Self {
        Self {
            max_response_bytes,
            max_total_bytes,
            max_resources,
        }
    }
}

/// Mount Ortet on `canvas` and open `address`. Browser fetch and asynchronous
/// scheduling remain in this host; the document engine receives only immutable
/// response snapshots through its existing synchronous resource seam.
#[wasm_bindgen]
pub async fn start(canvas: HtmlCanvasElement, address: String) -> Result<(), JsValue> {
    start_with_limits(canvas, address, BrowserResourceLimits::default()).await
}

/// Mount Ortet with host-selected acquisition bounds.
#[wasm_bindgen]
pub async fn start_with_limits(
    canvas: HtmlCanvasElement,
    address: String,
    limits: BrowserResourceLimits,
) -> Result<(), JsValue> {
    let (layout_width, layout_height, scale, physical_width, physical_height) =
        canvas_size(&canvas)?;
    canvas.set_width(physical_width);
    canvas.set_height(physical_height);
    canvas.set_tab_index(0);
    let loaded = load_session(&address, layout_width, layout_height, limits).await?;
    let options = NetrenderOptions {
        tile_cache_size: Some(64),
        enable_vello: true,
        ..NetrenderOptions::for_untrusted_content()
    };
    let core = Arc::new(RenderCore::boot_async(options)
        .await
        .map_err(|error| JsValue::from_str(&format!("could not boot renderer: {error}")))?);
    let surface = core
        .create_surface(
            wgpu::SurfaceTarget::Canvas(canvas.clone()),
            physical_width,
            physical_height,
        )
        .map_err(|error| JsValue::from_str(&format!("could not create canvas surface: {error}")))?;

    let host = Rc::new(RefCell::new(WebOrtet {
        canvas: canvas.clone(),
        core,
        surface,
        address: loaded.navigation_address,
        session: loaded.session,
        layout_width,
        layout_height,
        scale,
        physical_width,
        physical_height,
        cursor: (0.0, 0.0),
        modifiers: SessionModifiers::default(),
        pointer_capture_id: None,
        browser_limits: limits,
        navigation_generation: 0,
        pending_navigation: None,
        self_weak: Weak::new(),
        render_running: false,
        render_queued: false,
        surface_resize_pending: false,
        surface_size_generation: 0,
        render_revision: 0,
        outdated_retry_remaining: 0,
    }));
    host.borrow_mut().self_weak = Rc::downgrade(&host);
    host.borrow_mut().render();
    install_dom_events(&host)?;
    Ok(())
}

struct WebOrtet {
    canvas: HtmlCanvasElement,
    core: Arc<RenderCore>,
    surface: WindowSurface,
    address: String,
    session: Box<dyn DocumentSession<Scene>>,
    layout_width: u32,
    layout_height: u32,
    scale: f32,
    physical_width: u32,
    physical_height: u32,
    cursor: (f32, f32),
    modifiers: SessionModifiers,
    pointer_capture_id: Option<i32>,
    browser_limits: BrowserResourceLimits,
    navigation_generation: u64,
    pending_navigation: Option<BrowserNavigation>,
    self_weak: Weak<RefCell<WebOrtet>>,
    render_running: bool,
    render_queued: bool,
    surface_resize_pending: bool,
    surface_size_generation: u64,
    render_revision: u64,
    outdated_retry_remaining: u8,
}

enum BrowserNavigation { Link(String), Form(SessionEncodedFormSubmission) }

struct LoadedSession {
    session: Box<dyn DocumentSession<Scene>>,
    navigation_address: String,
}

impl WebOrtet {
    fn render(&mut self) {
        self.render_revision = self.render_revision.wrapping_add(1);
        self.outdated_retry_remaining = 1;
        self.render_queued = true;
        if !self.render_running {
            self.render_running = true;
            spawn_local(run_render_queue(self.self_weak.clone()));
        }
    }

    fn resize(&mut self) {
        let Ok((layout_width, layout_height, scale, physical_width, physical_height)) =
            canvas_size(&self.canvas)
        else {
            return;
        };
        if (physical_width, physical_height) != (self.physical_width, self.physical_height) {
            self.canvas.set_width(physical_width);
            self.canvas.set_height(physical_height);
            self.surface_resize_pending = true;
            self.surface_size_generation = self.surface_size_generation.wrapping_add(1);
        }
        self.layout_width = layout_width;
        self.layout_height = layout_height;
        self.scale = scale;
        self.physical_width = physical_width;
        self.physical_height = physical_height;
        self.render();
    }

    fn sync_address(&mut self) {
        if let Some(address) = self.session.current_address() { self.address = address; }
    }
    fn dispatch_input(&mut self, input: SessionInput) -> SessionInputResult {
        let result = self.session.input(input);
        match &result.effect {
            SessionEffect::Navigate(target) => {
                self.pending_navigation = Some(BrowserNavigation::Link(resolve_href(&self.address, target)));
            },
            SessionEffect::Submit(_) => {
                match self.session.take_encoded_form_submission() {
                    Some(encoded) => self.pending_navigation = Some(BrowserNavigation::Form(encoded)),
                    None => {
                        self.navigation_generation = self.navigation_generation.wrapping_add(1);
                        web_sys::console::warn_1(&JsValue::from_str("[ortet] encoded submission unavailable"));
                    },
                }
                // Focus changes can retire the old hit-test layout. Keep the
                // source interactive while its navigation is still pending.
                self.render();
            },
            SessionEffect::Handled | SessionEffect::Cancelled => self.render(),
            SessionEffect::Ignored => {},
        }
        self.sync_address();
        result
    }

    fn capture_pointer(&mut self, pointer_id: i32) {
        if self.canvas.set_pointer_capture(pointer_id).is_ok() {
            self.pointer_capture_id = Some(pointer_id);
        }
    }

    fn release_pointer(&mut self, pointer_id: i32) {
        if self.pointer_capture_id == Some(pointer_id) {
            let _ = self.canvas.release_pointer_capture(pointer_id);
            self.pointer_capture_id = None;
        }
    }

    fn release_captured_pointer(&mut self) {
        if let Some(pointer_id) = self.pointer_capture_id {
            self.release_pointer(pointer_id);
        }
    }

    fn pointer_position(&self, event: &PointerEvent) -> (f32, f32) {
        let rect = self.canvas.get_bounding_client_rect();
        let width = rect.width().max(1.0);
        let height = rect.height().max(1.0);
        (
            ((f64::from(event.client_x()) - rect.left()) * f64::from(self.layout_width) / width)
                .clamp(0.0, f64::from(self.layout_width)) as f32,
            ((f64::from(event.client_y()) - rect.top()) * f64::from(self.layout_height) / height)
                .clamp(0.0, f64::from(self.layout_height)) as f32,
        )
    }
}

enum BrowserRenderAttempt {
    Retry,
    Pending {
        pending: PendingBrowserFrame,
        surface_frame: Option<(wgpu::Texture, wgpu::SurfaceTexture)>,
        core: std::sync::Weak<RenderCore>,
        size_generation: u64,
        render_revision: u64,
        outdated: bool,
    },
    Refused(RasterizeFailure),
    Idle,
}

#[cfg(feature = "browser-frame-fixture")]
fn emit_frame_fixture_event(canvas: &HtmlCanvasElement, name: &str) {
    if let Ok(event) = Event::new(name) {
        let _ = canvas.dispatch_event(&event);
    }
}

/// A queued redraw does not borrow the host across an await. The checked
/// frame admission covers retained scene production, rasterization, surface
/// composition, and the actual GPU completion ledger. Presentation follows
/// only after acceptance on the original Core and deadline.
async fn run_render_queue(host: Weak<RefCell<WebOrtet>>) {
    loop {
        let Some(owner) = host.upgrade() else { return; };
        let attempt = {
            let mut state = owner.borrow_mut();
            if !state.render_queued {
                state.render_running = false;
                BrowserRenderAttempt::Idle
            } else {
                let core = state.core.clone();
                match core.try_begin_browser_frame() {
                    Err(BrowserFrameStartError::Busy) => BrowserRenderAttempt::Retry,
                    Err(BrowserFrameStartError::Refused(error)) => {
                        state.render_queued = false;
                        state.render_running = false;
                        BrowserRenderAttempt::Refused(error)
                    },
                    Ok(mut submission) => {
                        state.render_queued = false;
                        let (layout_width, layout_height) = (state.layout_width, state.layout_height);
                        let scene = state.session.frame(layout_width, layout_height);
                        state.sync_address();
                        let size_generation = state.surface_size_generation;
                        let render_revision = state.render_revision;
                        let outdated = Cell::new(false);
                        let result = submission.submit(|commands| {
                            if state.surface_resize_pending {
                                let (width, height) = (state.physical_width, state.physical_height);
                                state.surface.resize(&core, width, height);
                                state.surface_resize_pending = false;
                            }
                            let (texture, view) = commands.rasterize_scaled_for(
                                0,
                                &scene,
                                state.physical_width,
                                state.physical_height,
                                ColorLoad::Clear(wgpu::Color::WHITE),
                                state.scale,
                            )?;
                            let frame = match state.surface.acquire_status(&core) {
                                SurfaceAcquire::Frame(frame) => frame,
                                SurfaceAcquire::DeviceLost(loss) => {
                                    return Err(RasterizeFailure::DeviceLost(loss));
                                },
                                SurfaceAcquire::Reconfigured => {
                                    outdated.set(true);
                                    return Ok(None);
                                },
                                SurfaceAcquire::Unavailable { .. } => {
                                    return Ok(None);
                                },
                                SurfaceAcquire::SurfaceLost => {
                                    return Ok(None);
                                },
                            };
                            let target = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                            commands.compose_external_texture(
                                &view,
                                &target,
                                state.surface.format(),
                                state.physical_width,
                                state.physical_height,
                                ExternalTexturePlacement::new([
                                    0.0,
                                    0.0,
                                    state.physical_width as f32,
                                    state.physical_height as f32,
                                ]),
                            )?;
                            Ok(Some((texture, frame)))
                        });
                        match submission.seal() {
                            Ok(pending) => BrowserRenderAttempt::Pending {
                                pending,
                                surface_frame: result.ok().flatten(),
                                core: Arc::downgrade(&core),
                                size_generation,
                                render_revision,
                                outdated: outdated.get(),
                            },
                            Err(error) => BrowserRenderAttempt::Refused(error),
                        }
                    },
                }
            }
        };
        drop(owner);
        match attempt {
            BrowserRenderAttempt::Idle => return,
            BrowserRenderAttempt::Retry => {
                gloo_timers::future::TimeoutFuture::new(16).await;
            },
            BrowserRenderAttempt::Refused(error) => {
                if let Some(owner) = host.upgrade() {
                    let mut state = owner.borrow_mut();
                    state.render_running = false;
                }
                web_sys::console::warn_1(&JsValue::from_str(&format!("[ortet] browser frame refused: {error}")));
                return;
            },
            BrowserRenderAttempt::Pending { pending, surface_frame, core, size_generation, render_revision, outdated } => {
                // The fixture queues a real resize event here, after submission
                // and outside the host borrow, before the completion wait.
                #[cfg(feature = "browser-frame-fixture")]
                if surface_frame.is_some() {
                    if let Some(owner) = host.upgrade() {
                        let canvas = owner.borrow().canvas.clone();
                        emit_frame_fixture_event(&canvas, "ortet-fixture-submitted");
                    }
                }
                match pending.wait().await {
                    Ok(accepted) => {
                        if let Some((_texture, frame)) = surface_frame {
                            let (size_is_current, render_is_current) = host.upgrade().map(|owner| {
                                let state = owner.borrow();
                                (state.surface_size_generation == size_generation,
                                 state.render_revision == render_revision)
                            }).unwrap_or((false, false));
                            match (accepted.check_current(), core.upgrade(), size_is_current && render_is_current) {
                                (Ok(()), Some(core), true) => {
                                    core.queue().present(frame);
                                    #[cfg(feature = "browser-frame-fixture")]
                                    if let Some(owner) = host.upgrade() {
                                        let canvas = owner.borrow().canvas.clone();
                                        emit_frame_fixture_event(&canvas, "ortet-fixture-presented");
                                    }
                                },
                                (Err(error), _, _) => web_sys::console::warn_1(&JsValue::from_str(
                                    &format!("[ortet] browser frame expired before presentation: {error}"),
                                )),
                                _ => {
                                    #[cfg(feature = "browser-frame-fixture")]
                                    if size_is_current && !render_is_current {
                                        if let Some(owner) = host.upgrade() {
                                            let canvas = owner.borrow().canvas.clone();
                                            emit_frame_fixture_event(&canvas, "ortet-fixture-stale-skipped");
                                        }
                                    }
                                },
                            }
                        }
                        if outdated {
                            let retry = host.upgrade().is_some_and(|owner| {
                                let mut state = owner.borrow_mut();
                                if state.render_revision == render_revision
                                    && state.outdated_retry_remaining > 0 {
                                    state.outdated_retry_remaining -= 1;
                                    state.render_queued = true;
                                    true
                                } else {
                                    false
                                }
                            });
                            if retry { gloo_timers::future::TimeoutFuture::new(16).await; }
                        }
                    },
                    Err(error) => web_sys::console::warn_1(&JsValue::from_str(
                        &format!("[ortet] browser frame failed: {error}"),
                    )),
                }
            },
        }
    }
}

fn launch_pending_navigation(host: &Rc<RefCell<WebOrtet>>) {
    let Some((generation, address, request, width, height, limits)) = (|| {
        let mut host = host.borrow_mut();
        let navigation = host.pending_navigation.take()?;
        if matches!(&navigation, BrowserNavigation::Link(address) if address == &host.address) { return None; }
        host.navigation_generation = host.navigation_generation.wrapping_add(1);
        let (address, request) = match navigation {
            BrowserNavigation::Link(address) => {
                if address == host.address { return None; } (address, None)
            },
            BrowserNavigation::Form(encoded) => {
                if !encoded.target.is_empty() && !["_self", "_parent", "_top"].iter()
                    .any(|target| encoded.target.eq_ignore_ascii_case(target)) {
                    web_sys::console::warn_1(&JsValue::from_str("[ortet] unsupported native form target")); return None;
                }
                if let Err(error) = browser_request(&encoded.request) { web_sys::console::warn_1(&error); return None; }
                (encoded.request.url.clone(), Some(encoded.request))
            },
        };
        Some((host.navigation_generation, address, request, host.layout_width, host.layout_height, host.browser_limits))
    })() else { return; };
    let host = host.clone();
    spawn_local(async move {
        match load_session_request(&address, width, height, limits, request.as_ref(), || {
            host.borrow().navigation_generation == generation
        }).await {
            Ok(loaded) => {
                let mut host = host.borrow_mut();
                if host.navigation_generation != generation { return; }
                host.release_captured_pointer(); host.session = loaded.session;
                host.address = loaded.navigation_address; host.render();
            },
            Err(error) => {
                if host.borrow().navigation_generation == generation {
                    web_sys::console::warn_1(&format!("[ortet] could not open {address}: {error:?}").into());
                }
            },
        }
    });
}

async fn load_session(
    address: &str,
    width: u32,
    height: u32,
    limits: BrowserResourceLimits,
) -> Result<LoadedSession, JsValue> {
    load_session_request(address, width, height, limits, None, || true).await
}

async fn load_session_request(address: &str, width: u32, height: u32,
    limits: BrowserResourceLimits, outbound: Option<&SessionDocumentRequest>,
    is_current: impl Fn() -> bool) -> Result<LoadedSession, JsValue> {
    let check_current = || if is_current() { Ok(()) } else {
        Err(JsValue::from_str("superseded browser navigation"))
    };
    check_current()?;
    let request = SessionSpawnRequest::new(address).with_viewport(width, height);
    let engine = LiverySessionEngine::new(LocalFetcher);
    if !is_remote_http(address) {
        if outbound.is_some() { return Err(JsValue::from_str("unsupported browser form transport")); }
        let session = engine
            .spawn(&request)
            .map_err(|error| JsValue::from_str(&format!("could not open {address}: {error}")))?;
        return Ok(LoadedSession {
            session,
            navigation_address: address.to_owned(),
        });
    }
    let mut total_bytes = 0usize;
    if limits.max_resources == 0 {
        return Err(JsValue::from_str("browser resource count limit exceeded"));
    }
    let source_response = fetch_browser_request(address, outbound, &limits, &mut total_bytes).await?;
    // A held source may finish after a newer navigation has already won.
    // Refuse it before preparing or fetching any of its dependencies.
    check_current()?;
    let navigation_address = redirected_navigation_address(address, &source_response.final_url);
    let mut staged =
        LiveryResourcePreparation::new(&request, source_response, ResourceLimits::default());
    let mut resources = 1usize;
    loop {
        let pending = staged.pending_requests();
        if pending.is_empty() {
            break;
        }
        for request in pending {
            check_current()?;
            resources = resources.saturating_add(1);
            if resources > limits.max_resources as usize {
                return Err(JsValue::from_str("browser resource count limit exceeded"));
            }
            match fetch_browser_response(&request.url, &limits, &mut total_bytes).await {
                Ok(response) => staged
                    .provide_response(request, response)
                    .map_err(|error| JsValue::from_str(&format!("invalid provision: {error:?}")))?,
                Err(_) => {
                    staged.provide_unavailable(request).map_err(|error| {
                        JsValue::from_str(&format!("invalid provision: {error:?}"))
                    })?;
                },
            }
        }
    }
    let session = engine
        .spawn_prepared(&request, staged)
        .map_err(|error| JsValue::from_str(&format!("could not open {address}: {error}")))?;
    Ok(LoadedSession {
        session,
        navigation_address,
    })
}

async fn fetch_browser_response(
    url: &str,
    limits: &BrowserResourceLimits,
    total_bytes: &mut usize,
) -> Result<ResourceResponse, JsValue> {
    fetch_browser_request(url, None, limits, total_bytes).await
}

/// Translate a native navigation into the browser's supported source-fetch lane.
/// The neutral record retains native intent and the raw source URL for eligibility.
fn browser_request(request: &SessionDocumentRequest) -> Result<web_sys::Request, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("browser window unavailable"))?;
    let outer_origin = window.location().origin()?;
    let initiator = web_sys::Url::new(&request.referrer)?;
    let destination = web_sys::Url::new(&request.url)?;
    if !matches!(initiator.protocol().as_str(), "http:" | "https:") || initiator.origin() != outer_origin {
        return Err(JsValue::from_str("embedded source origin cannot initiate browser form fetch"));
    }
    if !matches!(destination.protocol().as_str(), "http:" | "https:")
        || !matches!(request.method.as_str(), "GET" | "POST")
        || request.method == "GET" && request.body.is_some()
        || !matches!(request.cache.as_str(), "default" | "no-store" | "reload" | "no-cache" | "force-cache" | "only-if-cached")
        || !matches!(request.redirect.as_str(), "follow" | "error" | "manual")
        || !matches!(request.mode.as_str(), "navigate" | "cors" | "same-origin")
        || !matches!(request.credentials.as_str(), "omit" | "same-origin" | "include")
        || !matches!(request.referrer_policy.as_str(), "" | "no-referrer" | "no-referrer-when-downgrade" | "same-origin"
            | "origin" | "strict-origin" | "origin-when-cross-origin" | "strict-origin-when-cross-origin" | "unsafe-url") {
        return Err(JsValue::from_str("unsupported browser form request field"));
    }
    let mode = if request.mode == "navigate" {
        if destination.origin() == outer_origin { "same-origin" } else { "cors" }
    } else { request.mode.as_str() };
    if request.cache == "only-if-cached" && mode != "same-origin" {
        return Err(JsValue::from_str("unsupported browser cache and mode combination"));
    }
    let init = web_sys::RequestInit::new();
    for (key, value) in [("method", request.method.as_str()), ("cache", request.cache.as_str()),
        ("redirect", request.redirect.as_str()), ("mode", mode), ("referrer", request.referrer.as_str()),
        ("referrerPolicy", request.referrer_policy.as_str()), ("credentials", request.credentials.as_str()),
        ("integrity", request.integrity.as_str())] {
        js_sys::Reflect::set(init.as_ref(), &JsValue::from_str(key), &JsValue::from_str(value))?;
    }
    let headers = web_sys::Headers::new()?;
    for (name, value) in &request.headers {
        // Fetch silently strips forbidden headers: decline fields this lane cannot forward.
        if !name.eq_ignore_ascii_case("content-type") {
            return Err(JsValue::from_str("unsupported browser form header"));
        }
        headers.append(name, value)?;
    }
    js_sys::Reflect::set(init.as_ref(), &JsValue::from_str("headers"), headers.as_ref())?;
    if let Some(body) = &request.body {
        let bytes = js_sys::Uint8Array::from(body.as_slice());
        js_sys::Reflect::set(init.as_ref(), &JsValue::from_str("body"), bytes.as_ref())?;
    }
    web_sys::Request::new_with_str_and_init(&request.url, &init)
}

async fn fetch_browser_request(url: &str, outbound: Option<&SessionDocumentRequest>,
    limits: &BrowserResourceLimits, total_bytes: &mut usize) -> Result<ResourceResponse, JsValue> {
    let window =
        web_sys::window().ok_or_else(|| JsValue::from_str("browser window is unavailable"))?;
    let fetch = match outbound { Some(request) => window.fetch_with_request(&browser_request(request)?),
        None => window.fetch_with_str(url) };
    let response = JsFuture::from(fetch)
        .await?
        .dyn_into::<Response>()?;
    let kind = js_sys::Reflect::get(response.as_ref(), &JsValue::from_str("type"))?.as_string().unwrap_or_default();
    if matches!(response.status(), 0 | 204 | 205) || !matches!(kind.as_str(), "basic" | "cors" | "default") {
        return Err(JsValue::from_str("browser response cannot install a document"));
    }
    if response
        .headers()
        .get("content-length")?
        .and_then(|length| length.parse::<usize>().ok())
        .is_some_and(|length| length > limits.max_response_bytes as usize)
    {
        return Err(JsValue::from_str("browser response byte limit exceeded"));
    }
    let final_url = if response.url().is_empty() {
        url.to_owned()
    } else {
        response.url()
    };
    let content_type = response.headers().get("content-type")?;
    let Some(body) = response.body() else {
        return Ok(ResourceResponse { final_url, content_type, bytes: Vec::new() });
    };
    let reader = body
        .get_reader()
        .dyn_into::<web_sys::ReadableStreamDefaultReader>()?;
    let mut bytes = Vec::new();
    loop {
        let read = JsFuture::from(reader.read()).await?;
        if js_sys::Reflect::get(&read, &JsValue::from_str("done"))?
            .as_bool()
            .unwrap_or(false)
        {
            break;
        }
        let chunk =
            js_sys::Uint8Array::new(&js_sys::Reflect::get(&read, &JsValue::from_str("value"))?);
        let length = chunk.length() as usize;
        if bytes.len().saturating_add(length) > limits.max_response_bytes as usize
            || total_bytes.saturating_add(length) > limits.max_total_bytes as usize
        {
            let _ = reader.cancel();
            return Err(JsValue::from_str("browser response byte limit exceeded"));
        }
        bytes.extend_from_slice(&chunk.to_vec());
        *total_bytes += length;
    }
    Ok(ResourceResponse {
        final_url,
        content_type,
        bytes,
    })
}

fn is_remote_http(address: &str) -> bool {
    address.split_once(':').is_some_and(|(scheme, _)| {
        scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https")
    })
}

/// Keep an explicitly requested fragment while replacing the transport base
/// with the redirect-final document identity.
fn redirected_navigation_address(requested: &str, final_url: &str) -> String {
    if final_url.contains('#') { return final_url.to_owned(); }
    requested.split_once('#').map_or_else(|| final_url.to_owned(),
        |(_, fragment)| format!("{final_url}#{fragment}"))
}

/// Real-WebGPU lifecycle probe used by `tests/web/checked_frame.html`.
/// Its own shared Core keeps the destructive loss case away from the visible
/// Ortet canvas while exercising the same lease and completion implementation.
#[cfg(feature = "browser-frame-fixture")]
fn submit_probe_frame(
    submission: &mut BrowserFrameSubmission,
    core: &RenderCore,
    surface: &WindowSurface,
    surface_id: u64,
    color: wgpu::Color,
    called: &Cell<bool>,
) -> Result<Option<(wgpu::Texture, wgpu::SurfaceTexture)>, RasterizeFailure> {
    let scene = Scene::new(64, 64);
    submission.submit(|commands| {
        called.set(true);
        let (texture, view) = commands.rasterize_scaled_for(
            surface_id, &scene, 64, 64, ColorLoad::Clear(color), 1.0,
        )?;
        let frame = match surface.acquire_status(core) {
            SurfaceAcquire::Frame(frame) => frame,
            SurfaceAcquire::DeviceLost(loss) => return Err(RasterizeFailure::DeviceLost(loss)),
            SurfaceAcquire::SurfaceLost
            | SurfaceAcquire::Reconfigured
            | SurfaceAcquire::Unavailable { .. } => return Ok(None),
        };
        let target = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        commands.compose_external_texture(
            &view, &target, surface.format(), 64, 64,
            ExternalTexturePlacement::new([0.0, 0.0, 64.0, 64.0]),
        )?;
        Ok(Some((texture, frame)))
    })
}

#[cfg(feature = "browser-frame-fixture")]
#[wasm_bindgen]
pub async fn probe_browser_frame_contract(
    canvas_a: HtmlCanvasElement,
    canvas_b: HtmlCanvasElement,
    witness: js_sys::Function,
) -> Result<String, JsValue> {
    let core = Arc::new(RenderCore::boot_async(NetrenderOptions {
        tile_cache_size: Some(8),
        enable_vello: true,
        ..NetrenderOptions::for_untrusted_content()
    }).await.map_err(|error| JsValue::from_str(&format!("probe boot: {error}")))?);
    let surface_a = core.create_surface(wgpu::SurfaceTarget::Canvas(canvas_a), 64, 64)
        .map_err(|error| JsValue::from_str(&format!("surface A: {error}")))?;
    let surface_b = core.create_surface(wgpu::SurfaceTarget::Canvas(canvas_b), 64, 64)
        .map_err(|error| JsValue::from_str(&format!("surface B: {error}")))?;
    let red = wgpu::Color { r: 0.8, g: 0.1, b: 0.1, a: 1.0 };
    let blue = wgpu::Color { r: 0.1, g: 0.2, b: 0.8, a: 1.0 };
    let mut presented_a = 0u32;
    let mut presented_b = 0u32;

    let unsubmitted = core.try_begin_browser_frame()
        .map_err(|error| JsValue::from_str(&format!("unsubmitted admission: {error:?}")))?;
    if !matches!(core.try_begin_browser_frame(), Err(BrowserFrameStartError::Busy)) {
        return Err(JsValue::from_str("unsubmitted lease did not own the shared Core"));
    }
    drop(unsubmitted);
    let mut busy = core.try_begin_browser_frame()
        .map_err(|error| JsValue::from_str(&format!("busy admission: {error:?}")))?;
    if !matches!(core.try_begin_browser_frame(), Err(BrowserFrameStartError::Busy)) {
        return Err(JsValue::from_str("shared-Core busy admission was not refused"));
    }
    let a_called = Cell::new(false);
    let a_frame = submit_probe_frame(&mut busy, &core, &surface_a, 0xa, red, &a_called)
        .map_err(|error| JsValue::from_str(&format!("surface A submit: {error}")))?
        .ok_or_else(|| JsValue::from_str("surface A did not acquire"))?;
    if !a_called.get() { return Err(JsValue::from_str("surface A submission did not run")); }
    let accepted = busy.seal().map_err(|error| JsValue::from_str(&format!("surface A seal: {error}")))?
        .wait().await.map_err(|error| JsValue::from_str(&format!("surface A completion: {error}")))?;
    accepted.check_current().map_err(|error| JsValue::from_str(&format!("surface A acceptance: {error}")))?;
    let (_texture, frame) = a_frame;
    core.queue().present(frame);
    presented_a += 1;
    drop(accepted);

    let mut dropped = core.try_begin_browser_frame()
        .map_err(|error| JsValue::from_str(&format!("drop admission: {error:?}")))?;
    let drop_called = Cell::new(false);
    let dropped_frame = submit_probe_frame(&mut dropped, &core, &surface_a, 0xa, red, &drop_called)
        .map_err(|error| JsValue::from_str(&format!("drop submit: {error}")))?
        .ok_or_else(|| JsValue::from_str("drop frame did not acquire"))?;
    if !drop_called.get() { return Err(JsValue::from_str("dropped frame submission did not run")); }
    let pending = dropped.seal().map_err(|error| JsValue::from_str(&format!("drop seal: {error}")))?;
    drop(dropped_frame); // Submitted but deliberately never presented.
    drop(pending);
    if !matches!(core.try_begin_browser_frame(), Err(BrowserFrameStartError::Busy)) {
        return Err(JsValue::from_str("dropped pending frame released its Core before completion"));
    }
    let drop_deadline = js_sys::Date::now() + 6000.0;
    let mut after_drop = loop {
        match core.try_begin_browser_frame() {
            Ok(frame) => break frame,
            Err(BrowserFrameStartError::Busy) if js_sys::Date::now() < drop_deadline => {
                gloo_timers::future::TimeoutFuture::new(16).await;
            },
            Err(other) => return Err(JsValue::from_str(&format!("dropped frame did not retire: {other:?}"))),
        }
    };
    let b_called = Cell::new(false);
    let b_frame = submit_probe_frame(&mut after_drop, &core, &surface_b, 0xb, blue, &b_called)
        .map_err(|error| JsValue::from_str(&format!("surface B submit: {error}")))?
        .ok_or_else(|| JsValue::from_str("surface B did not acquire"))?;
    if !b_called.get() { return Err(JsValue::from_str("surface B submission did not run")); }
    let accepted = after_drop.seal().map_err(|error| JsValue::from_str(&format!("surface B seal: {error}")))?
        .wait().await.map_err(|error| JsValue::from_str(&format!("surface B completion: {error}")))?;
    accepted.check_current().map_err(|error| JsValue::from_str(&format!("surface B acceptance: {error}")))?;
    let (_texture, frame) = b_frame;
    core.queue().present(frame);
    presented_b += 1;
    drop(accepted);

    // The page reads actual A/B canvas pixels before the destructive loss
    // case can clear those browser surfaces. Its Promise must settle first.
    let proof = witness.call0(&JsValue::NULL)?;
    JsFuture::from(proof.dyn_into::<js_sys::Promise>()?).await?;

    let before_refusals = (presented_a, presented_b);

    let mut expired = core.try_begin_browser_frame()
        .map_err(|error| JsValue::from_str(&format!("deadline admission: {error:?}")))?;
    gloo_timers::future::TimeoutFuture::new(5100).await;
    let expired_called = Cell::new(false);
    if !matches!(submit_probe_frame(&mut expired, &core, &surface_a, 0xa, red, &expired_called), Err(RasterizeFailure::ProviderError(_)))
        || expired_called.get() {
        return Err(JsValue::from_str("expired frame was not refused by its original deadline"));
    }
    drop(expired);
    if (presented_a, presented_b) != before_refusals {
        return Err(JsValue::from_str("deadline refusal presented a frame"));
    }

    let mut lost = core.try_begin_browser_frame()
        .map_err(|error| JsValue::from_str(&format!("loss admission: {error:?}")))?;
    core.device().destroy();
    let loss_deadline = js_sys::Date::now() + 5000.0;
    while core.device_failure().is_none() && js_sys::Date::now() < loss_deadline {
        gloo_timers::future::TimeoutFuture::new(16).await;
    }
    if core.device_failure().is_none() {
        return Err(JsValue::from_str("actual device-loss callback did not arrive"));
    }
    // Both facts now hold. Loss must win even though the original clock has
    // expired and cancellation has already retired the admission.
    gloo_timers::future::TimeoutFuture::new(5100).await;
    let lost_called = Cell::new(false);
    if !matches!(submit_probe_frame(&mut lost, &core, &surface_b, 0xb, blue, &lost_called), Err(RasterizeFailure::DeviceLost(_)))
        || lost_called.get() {
        return Err(JsValue::from_str("device loss did not outrank frame deadline"));
    }
    if !matches!(core.try_begin_browser_frame(), Err(BrowserFrameStartError::Refused(RasterizeFailure::DeviceLost(_)))) {
        return Err(JsValue::from_str("lost Core admitted a new frame"));
    }
    if (presented_a, presented_b) != before_refusals {
        return Err(JsValue::from_str("device-loss refusal presented a frame"));
    }
    Ok(format!("unsubmitted=reusable busy=refused drop=retired_after_completion presented_a={presented_a} presented_b={presented_b} timeout=refused loss=priority refusal_presents=0"))
}

#[cfg(test)]
mod tests {
    use super::{is_remote_http, redirected_navigation_address};

    #[test]
    fn redirected_navigation_uses_final_base_and_requested_fragment() {
        assert_eq!(
            redirected_navigation_address(
                "HTTPS://example.test/start#proof",
                "https://cdn.example.test/final/index.html",
            ),
            "https://cdn.example.test/final/index.html#proof"
        );
        assert!(is_remote_http("HTTPS://example.test/start"));
    }
}

fn install_dom_events(host: &Rc<RefCell<WebOrtet>>) -> Result<(), JsValue> {
    let canvas = host.borrow().canvas.clone();
    let target: &EventTarget = canvas.unchecked_ref();

    listen(target, "pointermove", {
        let host = host.clone();
        move |event| {
            let Some(event) = event.dyn_ref::<PointerEvent>() else {
                return;
            };
            {
                let mut state = host.borrow_mut();
                state.cursor = state.pointer_position(event);
                let (x, y) = state.cursor;
                state.modifiers = modifiers_from_pointer(event);
                let modifiers = state.modifiers;
                state.dispatch_input(SessionInput::PointerMoved { x, y, modifiers });
            }
            launch_pending_navigation(&host);
        }
    })?;
    listen(target, "pointerdown", {
        let host = host.clone();
        move |event| {
            let Some(event) = event.dyn_ref::<PointerEvent>() else {
                return;
            };
            // Focusing synchronously invokes our focus listener, which needs
            // a mutable host borrow of its own.
            let canvas = { host.borrow().canvas.clone() };
            let _ = canvas.focus();
            {
                let mut state = host.borrow_mut();
                state.cursor = state.pointer_position(event);
                let (x, y) = state.cursor;
                state.modifiers = modifiers_from_pointer(event);
                let modifiers = state.modifiers;
                if state.dispatch_input(SessionInput::PointerButton {
                    x,
                    y,
                    button: pointer_button(event.button()),
                    state: SessionButtonState::Pressed,
                    modifiers,
                }).capture == Some(true)
                {
                    state.capture_pointer(event.pointer_id());
                }
            }
            launch_pending_navigation(&host);
        }
    })?;
    listen(target, "pointerup", {
        let host = host.clone();
        move |event| {
            let Some(event) = event.dyn_ref::<PointerEvent>() else {
                return;
            };
            {
                let mut state = host.borrow_mut();
                state.cursor = state.pointer_position(event);
                let (x, y) = state.cursor;
                state.modifiers = modifiers_from_pointer(event);
                let modifiers = state.modifiers;
                state.dispatch_input(SessionInput::PointerButton {
                    x,
                    y,
                    button: pointer_button(event.button()),
                    state: SessionButtonState::Released,
                    modifiers,
                });
                state.release_pointer(event.pointer_id());
            }
            launch_pending_navigation(&host);
        }
    })?;
    listen(target, "pointercancel", {
        let host = host.clone();
        move |event| {
            let Some(event) = event.dyn_ref::<PointerEvent>() else {
                return;
            };
            let mut host = host.borrow_mut();
            host.dispatch_input(SessionInput::Cancel);
            host.release_pointer(event.pointer_id());
        }
    })?;
    listen(target, "wheel", {
        let host = host.clone();
        move |event| {
            let Some(event) = event.dyn_ref::<WheelEvent>() else {
                return;
            };
            event.prevent_default();
            let mut host = host.borrow_mut();
            let scale = if host.scale > 0.0 { host.scale } else { 1.0 };
            let (x, y) = host.cursor;
            if host.session.scroll_at(
                x,
                y,
                event.delta_x() as f32 / scale,
                event.delta_y() as f32 / scale,
            ) {
                host.render();
            }
        }
    })?;
    listen(target, "keydown", {
        let host = host.clone();
        move |event| {
            let Some(event) = event.dyn_ref::<KeyboardEvent>() else {
                return;
            };
            {
                let mut state = host.borrow_mut();
                state.modifiers = modifiers_from_key(event);
                let modifiers = state.modifiers;
                let key = session_key(event.key());
                let scroll = scroll_key(&key, modifiers.shift);
                let result = state.dispatch_input(SessionInput::Key {
                    key,
                    state: SessionButtonState::Pressed,
                    modifiers,
                    repeat: event.repeat(),
                });
                let scrolled = result.effect.allows_default_scroll(result.editable)
                    && scroll.is_some_and(|scroll| state.session.scroll_for_key(scroll));
                if result.effect.is_handled() || (result.editable && scroll.is_some()) || scrolled {
                    event.prevent_default();
                }
                if scrolled {
                    state.render();
                }
            }
            launch_pending_navigation(&host);
        }
    })?;
    listen(target, "keyup", {
        let host = host.clone();
        move |event| {
            let Some(event) = event.dyn_ref::<KeyboardEvent>() else {
                return;
            };
            {
                let mut state = host.borrow_mut();
                state.modifiers = modifiers_from_key(event);
                let modifiers = state.modifiers;
                state.dispatch_input(SessionInput::Key {
                    key: session_key(event.key()),
                    state: SessionButtonState::Released,
                    modifiers,
                    repeat: false,
                });
            }
            launch_pending_navigation(&host);
        }
    })?;
    listen(target, "focus", {
        let host = host.clone();
        move |_| {
            host.borrow_mut().dispatch_input(SessionInput::Focus(true));
        }
    })?;
    listen(target, "blur", {
        let host = host.clone();
        move |_| {
            let mut host = host.borrow_mut();
            if host.pointer_capture_id.is_some() {
                host.dispatch_input(SessionInput::Cancel);
            }
            host.release_captured_pointer();
            host.dispatch_input(SessionInput::Focus(false));
        }
    })?;
    let window =
        web_sys::window().ok_or_else(|| JsValue::from_str("browser window is unavailable"))?;
    let window_target: &EventTarget = window.unchecked_ref();
    listen(window_target, "resize", {
        let host = host.clone();
        move |_| host.borrow_mut().resize()
    })?;
    Ok(())
}

fn listen(
    target: &EventTarget,
    event: &str,
    handler: impl FnMut(Event) + 'static,
) -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new(handler) as Box<dyn FnMut(Event)>);
    target.add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())?;
    // The browser owns the listener for the lifetime of its canvas. Holding the
    // closure in JS also retains the host state captured by each handler.
    closure.forget();
    Ok(())
}

fn canvas_size(canvas: &HtmlCanvasElement) -> Result<(u32, u32, f32, u32, u32), JsValue> {
    let window =
        web_sys::window().ok_or_else(|| JsValue::from_str("browser window is unavailable"))?;
    let rect = canvas.get_bounding_client_rect();
    let layout_width = rect.width().round().max(1.0) as u32;
    let layout_height = rect.height().round().max(1.0) as u32;
    let scale = window.device_pixel_ratio().max(1.0) as f32;
    Ok((
        layout_width,
        layout_height,
        scale,
        (layout_width as f32 * scale).round().max(1.0) as u32,
        (layout_height as f32 * scale).round().max(1.0) as u32,
    ))
}

fn modifiers_from_pointer(event: &PointerEvent) -> SessionModifiers {
    SessionModifiers {
        shift: event.shift_key(),
        control: event.ctrl_key(),
        alt: event.alt_key(),
        meta: event.meta_key(),
    }
}

fn modifiers_from_key(event: &KeyboardEvent) -> SessionModifiers {
    SessionModifiers {
        shift: event.shift_key(),
        control: event.ctrl_key(),
        alt: event.alt_key(),
        meta: event.meta_key(),
    }
}

fn pointer_button(button: i16) -> SessionPointerButton {
    match button {
        0 => SessionPointerButton::Primary,
        1 | 3 | 4 => SessionPointerButton::Auxiliary,
        2 => SessionPointerButton::Secondary,
        _ => SessionPointerButton::Auxiliary,
    }
}

fn session_key(key: String) -> SessionKey {
    match key.as_str() {
        "Enter" => SessionKey::Enter,
        "Tab" => SessionKey::Tab,
        "Backspace" => SessionKey::Backspace,
        "Delete" => SessionKey::Delete,
        "Escape" => SessionKey::Escape,
        " " => SessionKey::Space,
        "ArrowLeft" => SessionKey::ArrowLeft,
        "ArrowRight" => SessionKey::ArrowRight,
        "ArrowUp" => SessionKey::ArrowUp,
        "ArrowDown" => SessionKey::ArrowDown,
        "Home" => SessionKey::Home,
        "End" => SessionKey::End,
        "PageUp" => SessionKey::PageUp,
        "PageDown" => SessionKey::PageDown,
        _ if key.chars().count() == 1 => SessionKey::Character(key),
        _ => SessionKey::Unidentified,
    }
}

fn scroll_key(key: &SessionKey, shift: bool) -> Option<SessionScrollKey> {
    Some(match key {
        SessionKey::ArrowUp => SessionScrollKey::LineUp,
        SessionKey::ArrowDown => SessionScrollKey::LineDown,
        SessionKey::PageUp => SessionScrollKey::PageUp,
        SessionKey::PageDown => SessionScrollKey::PageDown,
        SessionKey::Home => SessionScrollKey::Home,
        SessionKey::End => SessionScrollKey::End,
        SessionKey::Space if shift => SessionScrollKey::PageUp,
        SessionKey::Space => SessionScrollKey::PageDown,
        _ => return None,
    })
}
