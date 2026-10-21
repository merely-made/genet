// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The window and its event loop.
//!
//! One `winit` window, one `SurfaceHost`, one `DocumentSession<Scene>`. The
//! shell owns exactly three things a session cannot: the window, the fetch
//! seam, and navigation. Following a link is spawning a new session for the
//! resolved address — there is no history, no registry, and no controller
//! between the winit event and the session's own input vocabulary.

use std::sync::Arc;
use std::time::Instant;

use document_session_api::session_engine::{
    DocumentSession, SessionButtonState, SessionClick, SessionCursor, SessionEffect, SessionEngine,
    SessionIme, SessionInput, SessionKey, SessionModifiers, SessionPointerButton, SessionScrollKey,
    SessionSpawnRequest,
};
use genet_documents::LiverySessionEngine;
#[cfg(feature = "scripted")]
use genet_documents::{ScriptWake, ScriptWakeEvent, ScriptedSessionEngine};
#[cfg(not(feature = "scripted"))]
#[derive(Clone, Default)]
struct ScriptWake;
#[cfg(not(feature = "scripted"))]
impl ScriptWake {
    fn new() -> Self {
        Self
    }
}
use genet_host_api::navigation::resolve_href;
use genet_render_host::RenderCore;
use genet_winit_host::{AccessKitBridge, BridgeStatus, SurfaceHost, wheel_delta_from_winit};
use netrender::{ColorLoad, ExternalTexturePlacement, NetrenderOptions, Scene};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use crate::a11y::{Accessibility, RoutedAction};
use crate::args::{Action, Config};
use crate::fetch::OrtetFetcher;
use crate::mutate::MutationPlan;
use crate::receipt;
use crate::timing::{FrameTiming, Phase, TimingLog, TimingSummary, json_string};
#[cfg(feature = "scripted")]
use crate::webgl::WebGlHost;

#[cfg(feature = "scripted")]
fn worker_helper(backend: script_runtime_api::WorkerBackend) -> Option<script_runtime_api::WorkerProcessLauncher> {
    let executable = std::env::var_os("GENET_WORKER_HELPER")?;
    let executable = std::path::PathBuf::from(executable);
    executable.is_absolute().then(|| script_runtime_api::WorkerProcessLauncher::new(executable, backend))
}

/// The actual session-transition owner, borrowed by window and headless hosts.
struct SessionDriver<'a> {
    engine: &'a dyn SessionEngine<Scene>,
    session: &'a mut Box<dyn DocumentSession<Scene>>,
    address: &'a mut String,
    a11y: &'a mut Accessibility,
    captured: &'a mut bool,
    viewport: (u32, u32),
}
#[derive(Default)]
struct DriverOutcome {
    installed: bool,
    declined: bool,
    address_changed: bool,
    input: Option<document_session_api::SessionInputResult>,
    error: Option<String>,
}
impl SessionDriver<'_> {
    fn sync_address(&mut self) -> DriverOutcome {
        let address_changed = self.session.current_address().is_some_and(|address| {
            if address == *self.address { false } else { *self.address = address; true }
        });
        DriverOutcome { address_changed, ..Default::default() }
    }
    fn install(&mut self, session: Box<dyn DocumentSession<Scene>>, address: String) -> DriverOutcome {
        let address = session.current_address().unwrap_or(address);
        let address_changed = address != *self.address;
        *self.session = session;
        *self.address = address;
        *self.captured = false;
        self.a11y.replace_session();
        DriverOutcome { installed: true, address_changed, ..Default::default() }
    }
    fn navigate(&mut self, target: &str) -> DriverOutcome {
        let resolved = resolve_href(self.address, target);
        if resolved == *self.address { return DriverOutcome::default(); }
        let request = SessionSpawnRequest::new(&resolved).with_viewport(self.viewport.0, self.viewport.1);
        match self.engine.spawn(&request) {
            Ok(session) => self.install(session, resolved),
            Err(error) => DriverOutcome { declined: true, error: Some(error.to_string()), ..Default::default() },
        }
    }
    fn input(&mut self, input: SessionInput) -> DriverOutcome {
        let result = self.session.input(input);
        // Consuming here pairs the exact request with this Submit, before another input.
        let mut outcome = match &result.effect {
            SessionEffect::Submit(_) => {
                match self.session.take_encoded_form_submission() {
                    Some(encoded) if encoded.target.is_empty() || ["_self", "_parent", "_top"].iter()
                        .any(|name| encoded.target.eq_ignore_ascii_case(name)) => {
                        let spawn = SessionSpawnRequest::new(&encoded.request.url)
                            .with_viewport(self.viewport.0, self.viewport.1);
                        match self.engine.spawn_request(&spawn, &encoded.request) {
                            Ok(loaded) => self.install(loaded.session, loaded.navigation_address),
                            Err(error) => DriverOutcome { declined: true, error: Some(error.to_string()), ..Default::default() },
                        }
                    },
                    _ => DriverOutcome { declined: true,
                        error: Some("encoded submission or supported top target unavailable".into()), ..Default::default() },
                }
            },
            SessionEffect::Navigate(target) => self.navigate(target),
            _ => DriverOutcome::default(),
        };
        if !outcome.installed && !outcome.declined {
            if let Some(capture) = result.capture { *self.captured = capture; }
            outcome.address_changed |= self.sync_address().address_changed;
        }
        outcome.input = Some(result);
        outcome
    }
    fn click(&mut self, x: f32, y: f32) -> DriverOutcome {
        let press = self.input(SessionInput::PointerButton { x, y, button: SessionPointerButton::Primary,
            state: SessionButtonState::Pressed, modifiers: SessionModifiers::default() });
        if press.installed || press.declined { return press; }
        let mut release = self.input(SessionInput::PointerButton { x, y, button: SessionPointerButton::Primary,
            state: SessionButtonState::Released, modifiers: SessionModifiers::default() });
        release.address_changed |= press.address_changed;
        release
    }
    fn pump(&mut self, now: f64) -> DriverOutcome {
        self.session.pump(now); self.sync_address()
    }
    fn frame(&mut self, width: u32, height: u32) -> (Scene, DriverOutcome) {
        let frame = self.session.frame(width, height); (frame, self.sync_address())
    }
}

/// What a completed run has to say for itself.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    /// The id claimed by the concrete engine that hosted this run.
    pub engine_id: String,
    /// The command-line backend choice. Boa and Nova intentionally have
    /// different backends even though Boa uses the generic scripted engine id.
    pub backend: String,
    pub metadata: BuildMetadata,
    pub address: String,
    pub frames: u32,
    pub size: (u32, u32),
    pub artifact: Option<std::path::PathBuf>,
    pub digest: Option<u64>,
    /// The semantic completion condition proven at the same final frame as a
    /// scripted receipt artifact, when the caller requested one.
    pub matched_heading: Option<String>,
    /// Cumulative `(reflectors_unpinned, nodes_collected)` the session's own
    /// frame-cadence GC ticks reported since it spawned (`0, 0` for lanes
    /// without a scripted engine). This is the same production
    /// `Runtime::collect_garbage` accounting the S1/S2/S5 arena receipts use
    /// as their collection proof, surfaced here so a G5 host receipt can
    /// correlate it with the captured frame instead of approximating
    /// collection from script-visible behavior alone.
    pub collection_stats: (usize, usize),
    /// Live nodes in the **top** document's arena at the first laid-out frame
    /// and at the last frame presented. `None` for a lane with no scripted
    /// arena to count. A sequence that adopts a subtree out and back, and then
    /// navigates the top level, must not end above where it began: a leaked
    /// outgoing document would leave the last count higher.
    pub live_nodes: (Option<usize>, Option<usize>),
    /// The T3 mutation instrument's per-frame attribution over the measured
    /// frames: actual work apart from presentation wait.
    pub timing: TimingSummary,
    /// One line per `--mutate` stream, saying how many elements it resolved to.
    pub mutation_streams: Vec<String>,
    /// Host mutations the engine reported as applied, and as missed.
    pub host_mutations: (u64, u64),
    /// Why host mutation did not run, when it was asked for and refused.
    pub mutation_unsupported: Option<String>,
}

/// Build facts that a host receipt can report without guessing a source
/// revision. `GENET_SOURCE_REVISION` is supplied by a release build when it
/// has one; local builds honestly report it as unknown.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildMetadata {
    pub target: String,
    pub features: String,
    pub source_revision: Option<String>,
}

pub fn build_metadata() -> BuildMetadata {
    BuildMetadata {
        target: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        features: enabled_features().to_owned(),
        source_revision: option_env!("GENET_SOURCE_REVISION").map(str::to_owned),
    }
}

#[cfg(feature = "scripted-nova")]
const fn enabled_features() -> &'static str {
    "scripted,scripted-nova"
}

#[cfg(all(feature = "scripted", not(feature = "scripted-nova")))]
const fn enabled_features() -> &'static str {
    "scripted"
}

#[cfg(not(feature = "scripted"))]
const fn enabled_features() -> &'static str {
    "none"
}

/// Open the window and run the document until the frame budget or the user
/// closes it.
pub fn run(config: Config, fetcher: OrtetFetcher) -> Result<Outcome, String> {
    let render_options = NetrenderOptions {
        tile_cache_size: Some(64),
        enable_vello: true,
        ..NetrenderOptions::for_untrusted_content()
    };
    let core = Arc::new(RenderCore::boot(render_options)?);
    let wake = ScriptWake::new();
    #[cfg(feature = "scripted")]
    let webgl = WebGlHost::new(core.device().clone(), core.queue().clone());
    let engine = make_engine_with_wake(
        config.engine,
        fetcher,
        wake.clone(),
        #[cfg(feature = "scripted")]
        Some(webgl.clone()),
        #[cfg(not(feature = "scripted"))]
        (),
    )?;
    // Spawn before the window exists so a bad address fails without flashing a
    // window at anyone.
    let session = spawn(engine.as_ref(), &config.address, config.size)?;
    let event_loop =
        EventLoop::new().map_err(|error| format!("could not create the event loop: {error}"))?;
    let mut app = Ortet::new(
        config,
        engine,
        session,
        wake,
        core,
        #[cfg(feature = "scripted")]
        webgl,
    );
    event_loop
        .run_app(&mut app)
        .map_err(|error| format!("the ortet event loop failed: {error}"))?;
    // The timing receipt is written even for a failed run: a run that died
    // part-way still measured the frames it presented, and hiding them would
    // make a partial sweep cell unreportable.
    app.write_timing_receipt()?;
    match app.failure {
        Some(failure) => Err(failure),
        None => Ok(app.outcome()),
    }
}

fn spawn(
    engine: &dyn SessionEngine<Scene>,
    address: &str,
    (width, height): (u32, u32),
) -> Result<Box<dyn DocumentSession<Scene>>, String> {
    let request = SessionSpawnRequest::new(address).with_viewport(width, height);
    engine
        .spawn(&request)
        .map_err(|error| format!("could not open {address}: {error}"))
}

/// Build the selected engine behind the common host contract. This is the O5a
/// seam: engine choice is explicit host policy, while session construction,
/// input, pumping, and frames remain the one `SessionEngine<Scene>` path.
#[cfg(test)]
fn make_engine(
    choice: crate::args::EngineChoice,
    fetcher: OrtetFetcher,
) -> Result<Box<dyn SessionEngine<Scene>>, String> {
    make_engine_with_wake(
        choice,
        fetcher,
        ScriptWake::new(),
        #[cfg(feature = "scripted")]
        None,
        #[cfg(not(feature = "scripted"))]
        (),
    )
}

#[cfg(feature = "scripted")]
fn make_engine_with_wake(
    choice: crate::args::EngineChoice,
    fetcher: OrtetFetcher,
    wake: ScriptWake,
    webgl: Option<WebGlHost>,
) -> Result<Box<dyn SessionEngine<Scene>>, String> {
    match choice {
        crate::args::EngineChoice::Livery => Ok(Box::new(LiverySessionEngine::new(fetcher))),
        crate::args::EngineChoice::Boa => {
            #[cfg(feature = "scripted")]
            {
                let engine =
                    ScriptedSessionEngine::<script_engine_boa::BoaEngine, _>::new_with_wake(
                        document_session_api::engine_ids::ENGINE_GENET_SCRIPTED,
                        fetcher,
                        wake,
                    );
                let launcher = worker_helper(script_runtime_api::WorkerBackend::Boa);
                Ok(Box::new(engine.with_options_factory(move |_| {
                    let mut options = webgl.as_ref().map_or_else(
                        genet_scripted::ScriptedDocumentOptions::default,
                        WebGlHost::scripted_document_options,
                    );
                    options.worker_process_launcher = launcher.clone();
                    Ok(options)
                })))
            }
            #[cfg(not(feature = "scripted"))]
            {
                let _ = fetcher;
                Err(
                    "the boa engine is unavailable: rebuild ortet with --features scripted"
                        .to_owned(),
                )
            }
        },
        crate::args::EngineChoice::Nova => {
            #[cfg(all(feature = "scripted-nova", target_pointer_width = "64"))]
            {
                let engine =
                    ScriptedSessionEngine::<script_engine_nova::NovaEngine, _>::new_with_wake(
                        document_session_api::engine_ids::ENGINE_GENET_SCRIPTED_NOVA,
                        fetcher,
                        wake,
                    );
                let launcher = worker_helper(script_runtime_api::WorkerBackend::Nova);
                Ok(Box::new(engine.with_options_factory(move |_| {
                    let mut options = webgl.as_ref().map_or_else(
                        genet_scripted::ScriptedDocumentOptions::default,
                        WebGlHost::scripted_document_options,
                    );
                    options.worker_process_launcher = launcher.clone();
                    Ok(options)
                })))
            }
            #[cfg(all(feature = "scripted-nova", not(target_pointer_width = "64")))]
            {
                let _ = fetcher;
                Err("the nova engine is unavailable on 32-bit targets".to_owned())
            }
            #[cfg(not(feature = "scripted-nova"))]
            {
                let _ = fetcher;
                Err(
                    "the nova engine is unavailable: rebuild ortet with --features scripted-nova"
                        .to_owned(),
                )
            }
        },
    }
}

#[cfg(not(feature = "scripted"))]
fn make_engine_with_wake(
    choice: crate::args::EngineChoice,
    fetcher: OrtetFetcher,
    _wake: ScriptWake,
    _webgl: (),
) -> Result<Box<dyn SessionEngine<Scene>>, String> {
    match choice {
        crate::args::EngineChoice::Livery => Ok(Box::new(LiverySessionEngine::new(fetcher))),
        crate::args::EngineChoice::Boa => {
            Err("the boa engine is unavailable: rebuild ortet with --features scripted".to_owned())
        },
        crate::args::EngineChoice::Nova => Err(
            "the nova engine is unavailable: rebuild ortet with --features scripted-nova"
                .to_owned(),
        ),
    }
}

struct Ortet {
    config: Config,
    engine: Box<dyn SessionEngine<Scene>>,
    address: String,
    session: Box<dyn DocumentSession<Scene>>,
    window: Option<Arc<Window>>,
    host: Option<SurfaceHost>,
    core: Arc<RenderCore>,
    #[cfg(feature = "scripted")]
    webgl: WebGlHost,
    a11y_bridge: Option<AccessKitBridge>,
    a11y: Accessibility,
    width: u32,
    height: u32,
    /// Physical device pixels per logical layout pixel.
    scale_factor: f32,
    frames: u32,
    modifiers: SessionModifiers,
    /// Last cursor position in logical pixels; winit's button events carry none.
    cursor: (f32, f32),
    pointer_captured: bool,
    /// Driving steps still to apply. They run once, after the first frame has
    /// established geometry, so a `click` has a laid-out box to hit.
    pending_actions: Vec<Action>,
    /// The resolved `--mutate` streams. Resolved once at construction: the
    /// document's element ids exist from the parse, and a plan that re-resolved
    /// every frame would charge its own walk to the measurement.
    mutations: MutationPlan,
    mutations_applied: u64,
    mutations_missed: u64,
    mutation_unsupported: Option<String>,
    timing: TimingLog,
    start: Instant,
    capture: Option<(std::path::PathBuf, u64)>,
    /// Top-document arena census at the first laid-out frame and at the most
    /// recent one. Sampled through the session's own observation downcast.
    live_nodes_first: Option<usize>,
    live_nodes_last: Option<usize>,
    /// The last projection revision appended to `--a11y-dump`, so a steady
    /// document does not write the same block on every frame.
    a11y_dumped_revision: Option<u64>,
    wake_deadline: Option<Instant>,
    receipt_deadline: Option<Instant>,
    failure: Option<String>,
    matched_heading: Option<String>,
    #[cfg(feature = "scripted")]
    wake: ScriptWake,
    #[cfg(feature = "scripted")]
    last_external_generation: Option<u64>,
}

impl Ortet {
    fn new(
        config: Config,
        engine: Box<dyn SessionEngine<Scene>>,
        session: Box<dyn DocumentSession<Scene>>,
        wake: ScriptWake,
        core: Arc<RenderCore>,
        #[cfg(feature = "scripted")] webgl: WebGlHost,
    ) -> Self {
        let start = Instant::now();
        let receipt_deadline = (config.artifact.is_some() || config.expect_heading.is_some())
            .then_some(start + config.receipt_timeout);
        #[cfg(not(feature = "scripted"))]
        let _ = wake;
        let mutations = MutationPlan::resolve(&config.mutations, |prefix| {
            session.element_ids_with_prefix(prefix)
        });
        let timing = TimingLog::new(config.warmup_frames);
        let mut host = Self {
            address: config.address.clone(),
            pending_actions: config.actions.clone(),
            mutations,
            mutations_applied: 0,
            mutations_missed: 0,
            mutation_unsupported: None,
            timing,
            width: config.size.0,
            height: config.size.1,
            config,
            engine,
            session,
            window: None,
            host: None,
            core,
            #[cfg(feature = "scripted")]
            webgl,
            a11y_bridge: None,
            a11y: Accessibility::default(),
            scale_factor: 1.0,
            frames: 0,
            modifiers: SessionModifiers::default(),
            cursor: (0.0, 0.0),
            pointer_captured: false,
            start,
            capture: None,
            live_nodes_first: None,
            live_nodes_last: None,
            a11y_dumped_revision: None,
            wake_deadline: None,
            receipt_deadline,
            matched_heading: None,
            failure: None,
            #[cfg(feature = "scripted")]
            wake,
            #[cfg(feature = "scripted")]
            last_external_generation: None,
        };
        let outcome = host.session_driver().sync_address();
        host.present_driver_outcome(&outcome);
        host
    }

    fn outcome(&self) -> Outcome {
        Outcome {
            engine_id: self.engine.engine_id().to_owned(),
            backend: self.config.engine.name().to_owned(),
            metadata: build_metadata(),
            address: self.address.clone(),
            frames: self.frames,
            size: (self.width, self.height),
            artifact: self.capture.as_ref().map(|(path, _)| path.clone()),
            digest: self.capture.as_ref().map(|(_, digest)| *digest),
            matched_heading: self.matched_heading.clone(),
            collection_stats: self.session.collection_stats(),
            live_nodes: (self.live_nodes_first, self.live_nodes_last),
            timing: self.timing.summary(),
            mutation_streams: self.mutations.describe(),
            host_mutations: (self.mutations_applied, self.mutations_missed),
            mutation_unsupported: self.mutation_unsupported.clone(),
        }
    }

    /// Apply this frame's host mutation batch. Frame zero is left alone: it is
    /// the document's cold parse/style/layout, and a mutation folded into it
    /// would be measuring lane T2 instead.
    fn drive_mutations(&mut self, timing: &mut FrameTiming) {
        if self.frames == 0 || self.mutations.is_empty() {
            return;
        }
        let batch = self.mutations.batch(self.frames);
        if batch.is_empty() {
            return;
        }
        match self.session.apply_host_mutations(&batch) {
            Ok(report) => {
                self.mutations_applied += report.applied as u64;
                self.mutations_missed += report.missed as u64;
                timing.mutations = report.applied as u32;
                timing.restyled = report.restyled_elements as u32;
            },
            Err(error) => {
                // Recorded once, not per frame: an engine without the seam is
                // a fact about the run, and the receipt must not claim a
                // mutation measurement it never made.
                self.mutation_unsupported
                    .get_or_insert_with(|| error.to_string());
            },
        }
    }

    /// Write `--timing-json`, when one was asked for.
    fn write_timing_receipt(&self) -> Result<(), String> {
        let Some(path) = self.config.timing_json.as_deref() else {
            return Ok(());
        };
        let metadata = build_metadata();
        let streams = self
            .mutations
            .describe()
            .iter()
            .map(|line| json_string(line))
            .collect::<Vec<_>>()
            .join(", ");
        let header = vec![
            ("address", json_string(&self.address)),
            ("engine", json_string(self.engine.engine_id())),
            ("backend", json_string(self.config.engine.name())),
            ("target", json_string(&metadata.target)),
            ("features", json_string(&metadata.features)),
            (
                "source_revision",
                metadata
                    .source_revision
                    .as_deref()
                    .map_or_else(|| "null".to_owned(), json_string),
            ),
            ("size", format!("[{}, {}]", self.width, self.height)),
            ("scale_factor", format!("{:.4}", self.scale_factor)),
            ("presented_frames", self.frames.to_string()),
            ("mutation_streams", format!("[{streams}]")),
            ("mutations_applied", self.mutations_applied.to_string()),
            ("mutations_missed", self.mutations_missed.to_string()),
            (
                "mutation_unsupported",
                self.mutation_unsupported
                    .as_deref()
                    .map_or_else(|| "null".to_owned(), json_string),
            ),
            ("phase_timing", self.config.phase_timing.to_string()),
        ];
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
        }
        std::fs::write(path, self.timing.to_json(&header))
            .map_err(|error| format!("could not write {}: {error}", path.display()))
    }

    fn logical_size(&self) -> (u32, u32) {
        let logical = |extent: u32| {
            if self.scale_factor > 0.0 {
                ((extent as f32 / self.scale_factor).round() as u32).max(1)
            } else {
                extent.max(1)
            }
        };
        (logical(self.width), logical(self.height))
    }

    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn receipt_settled(&mut self) -> Result<bool, String> {
        if let Some(expected) = self.config.expect_heading.as_deref() {
            let matched = self
                .session
                .inspect()
                .is_some_and(|report| report.headings.iter().any(|h| h == expected));
            if matched {
                self.matched_heading = Some(expected.to_owned());
                return Ok(true);
            }
            if self
                .receipt_deadline
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                return Err(format!(
                    "receipt completion heading {expected:?} was absent before the {}ms deadline",
                    self.config.receipt_timeout.as_millis()
                ));
            }
            return Ok(false);
        }
        if self.config.artifact.is_none() || self.session.settled() {
            return Ok(true);
        }
        if self
            .receipt_deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return Err(format!(
                "semantic completion timed out after {}ms",
                self.config.receipt_timeout.as_millis()
            ));
        }
        Ok(false)
    }

    fn schedule_wake(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if self
            .wake_deadline
            .take()
            .is_some_and(|deadline| deadline <= now)
        {
            self.request_redraw();
        }
        let pending = self.session.pending_work();
        #[cfg(feature = "scripted")]
        if pending.external {
            let generation = self.wake.active_generation();
            if self.last_external_generation != Some(generation) {
                eprintln!(
                    "ortet: receipt idle generation={generation} timer={} microtasks={} external=true",
                    pending
                        .timer
                        .map(|delay| format!("{delay:?}"))
                        .unwrap_or_else(|| "none".to_owned()),
                    pending.microtasks
                );
                self.last_external_generation = Some(generation);
            }
        } else {
            #[cfg(feature = "scripted")]
            {
                self.last_external_generation = None;
            }
        }
        let timer_deadline = pending.next_wake(now);
        let receipt_deadline = self.receipt_deadline;
        let deadline = match (timer_deadline, receipt_deadline) {
            (Some(timer), Some(receipt)) => Some(timer.min(receipt)),
            (Some(timer), None) => Some(timer),
            (None, Some(receipt)) => Some(receipt),
            (None, None) => None,
        };
        if let Some(deadline) = deadline {
            self.wake_deadline = Some(deadline);
            event_loop.set_control_flow(ControlFlow::WaitUntil(deadline));
        } else {
            self.wake_deadline = None;
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }

    fn retitle(&self) {
        if let Some(window) = self.window.as_ref() {
            window.set_title(&format!("ortet — {}", self.address));
        }
    }

    /// Follow a link: resolve it against the current address and replace the
    /// session. Ortet keeps no history, so this is the whole of navigation.
    fn session_driver(&mut self) -> SessionDriver<'_> {
        let viewport = self.logical_size();
        SessionDriver { engine: self.engine.as_ref(), session: &mut self.session,
            address: &mut self.address, a11y: &mut self.a11y,
            captured: &mut self.pointer_captured, viewport }
    }
    fn present_driver_outcome(&mut self, outcome: &DriverOutcome) {
        if let Some(error) = &outcome.error { eprintln!("[ortet] {error}"); }
        if let Some(result) = &outcome.input {
            if let Some(window) = self.window.as_ref() {
                if let Some(cursor) = result.cursor {
                    window.set_cursor(match cursor {
                        SessionCursor::Default => winit::window::CursorIcon::Default,
                        SessionCursor::Pointer => winit::window::CursorIcon::Pointer,
                        SessionCursor::Text => winit::window::CursorIcon::Text,
                    });
                }
                window.set_ime_allowed(if outcome.installed { self.session.editable_focus() } else { result.editable });
            }
        }
        if outcome.address_changed || outcome.installed { self.retitle(); }
        if outcome.installed { self.publish_accessibility(); }
        if outcome.installed || outcome.address_changed || outcome.input.as_ref().is_some_and(|r| r.effect.is_handled()) {
            self.request_redraw();
        }
    }
    fn navigate(&mut self, target: &str) {
        let outcome = self.session_driver().navigate(target);
        self.present_driver_outcome(&outcome);
    }
    fn driver_click(&mut self, x: f32, y: f32) {
        let outcome = self.session_driver().click(x, y);
        self.present_driver_outcome(&outcome);
    }
    fn driver_frame(&mut self, width: u32, height: u32) -> Scene {
        let (scene, outcome) = self.session_driver().frame(width, height);
        self.present_driver_outcome(&outcome); scene
    }

    fn publish_accessibility(&mut self) {
        let projection = self.session.accessibility_projection();
        if let Some(path) = self.config.a11y_dump.clone() {
            self.append_accessibility_dump(&path, projection.as_ref());
        }
        let update = self.a11y.publish(projection);
        if let Some(update) = update
            && let Some(bridge) = self.a11y_bridge.as_mut()
        {
            bridge.update(update);
        }
    }

    /// Append one projection block to the `--a11y-dump` file, once per
    /// revision. The format is deliberately flat text: a receipt greps for the
    /// node it cares about and reads its role, name, actions and bounds, so the
    /// dump proves what the host advertised rather than what the pixels imply.
    fn append_accessibility_dump(
        &mut self,
        path: &std::path::Path,
        projection: Option<&document_session_api::DocumentA11yProjection>,
    ) {
        use std::io::Write as _;

        let mut block = String::new();
        match projection {
            Some(projection) => {
                if self.a11y_dumped_revision == Some(projection.revision()) {
                    return;
                }
                self.a11y_dumped_revision = Some(projection.revision());
                block.push_str(&format!(
                    "projection revision={} frames={} address={} nodes={} root={}\n",
                    projection.revision(),
                    self.frames,
                    self.address,
                    projection.nodes().len(),
                    projection.root().get(),
                ));
                for node in projection.nodes() {
                    let bounds = node.bounds.as_ref().map_or_else(
                        || "none".to_owned(),
                        |bounds| {
                            format!(
                                "{:.1},{:.1},{:.1},{:.1}",
                                bounds.x, bounds.y, bounds.width, bounds.height
                            )
                        },
                    );
                    block.push_str(&format!(
                        "  node id={} role={:?} name={:?} value={:?} actions={:?} bounds={bounds}\n",
                        node.id.get(),
                        node.role,
                        node.name.as_deref().unwrap_or(""),
                        node.value.as_deref().unwrap_or(""),
                        node.actions,
                    ));
                }
            },
            None => {
                if self.a11y_dumped_revision.is_none() {
                    return;
                }
                self.a11y_dumped_revision = None;
                block.push_str(&format!(
                    "projection none frames={} address={}\n",
                    self.frames, self.address
                ));
            },
        }
        let opened = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path);
        match opened {
            Ok(mut file) => {
                if let Err(error) = file.write_all(block.as_bytes()) {
                    eprintln!("[ortet] could not write {}: {error}", path.display());
                }
            },
            Err(error) => eprintln!("[ortet] could not open {}: {error}", path.display()),
        }
    }

    /// Live nodes in the top document's arena, through the session's own
    /// observation downcast. `None` for the script-free lane, which has no
    /// arena to count — reported honestly rather than as zero.
    #[cfg(feature = "scripted")]
    fn sample_live_nodes(&mut self) -> Option<usize> {
        if let Some(session) = self
            .session
            .as_any()
            .downcast_mut::<genet_documents::ScriptedDocumentSession<script_engine_boa::BoaEngine>>(
            )
        {
            return Some(session.document_mut().live_node_count());
        }
        #[cfg(all(feature = "scripted-nova", target_pointer_width = "64"))]
        if let Some(session) = self
            .session
            .as_any()
            .downcast_mut::<genet_documents::ScriptedDocumentSession<script_engine_nova::NovaEngine>>()
        {
            return Some(session.document_mut().live_node_count());
        }
        None
    }

    #[cfg(not(feature = "scripted"))]
    fn sample_live_nodes(&mut self) -> Option<usize> {
        None
    }

    fn drain_accessibility_actions(&mut self) {
        let requests = self
            .a11y_bridge
            .as_mut()
            .map(AccessKitBridge::drain_actions)
            .unwrap_or_default();
        for request in requests {
            match self.a11y.route(&mut *self.session, &request) {
                RoutedAction::Rejected => {
                    // The fresh-ID publication policy (a11y.rs) refuses any
                    // request whose host id, generation, revision, or
                    // advertised action no longer matches the currently
                    // published tree. Logging it lets a native receipt
                    // observe the refusal the platform bridge just delivered,
                    // not merely a unit test's simulated request.
                    eprintln!(
                        "ortet: receipt bridge-action rejected target={:?} action={:?}",
                        request.target_node, request.action
                    );
                },
                RoutedAction::Dispatched => self.request_redraw(),
                RoutedAction::Click { x, y } => {
                    self.driver_click(x, y);
                },
            }
        }
    }

    /// Run the `--actions` list once, against a document that has already been
    /// laid out at the current viewport.
    fn drive_pending_actions(&mut self) {
        let actions = std::mem::take(&mut self.pending_actions);
        let (width, height) = self.logical_size();
        for action in actions {
            match action {
                Action::Scroll { dx, dy } => {
                    let centre = (width as f32 * 0.5, height as f32 * 0.5);
                    self.session.scroll_at(centre.0, centre.1, dx, dy);
                },
                Action::Click { x, y } => {
                    // Press and release, the same pair a mouse produces: the
                    // Livery lane activates a link on the matching release.
                    self.driver_click(x, y);
                },
            }
        }
    }

    /// Dispatch one neutral input and apply everything the session asked the
    /// host for. Returns whether the keyboard scroll default still applies.
    fn apply_input(&mut self, input: SessionInput) -> bool {
        let outcome = self.session_driver().input(input);
        let editable = if outcome.installed { self.session.editable_focus() }
            else { outcome.input.as_ref().is_some_and(|r| r.editable) };
        let allows_default_scroll = outcome.input.as_ref().map_or_else(
            || SessionEffect::Ignored.allows_default_scroll(editable),
            |result| result.effect.allows_default_scroll(editable),
        );
        self.present_driver_outcome(&outcome);
        allows_default_scroll
    }

    /// The per-frame shape the host crates document: apply the host mutation
    /// batch, rasterize the scene into a texture, acquire the backbuffer,
    /// composite, present.
    ///
    /// Each phase is timed into one [`FrameTiming`], with the two blocking
    /// phases — acquire and present — kept out of the work total. See
    /// `crate::timing`.
    fn render(&mut self, event_loop: &ActiveEventLoop) {
        let frame_started = Instant::now();
        let mut timing = FrameTiming {
            index: self.frames,
            ..FrameTiming::default()
        };
        let now_ms = self.start.elapsed().as_secs_f64() * 1000.0;
        let outcome = self.session_driver().pump(now_ms);
        self.present_driver_outcome(&outcome);
        if self.host.is_none() {
            return;
        }
        self.drain_accessibility_actions();
        let mut phase = Phase::start();
        self.drive_mutations(&mut timing);
        timing.mutate_us = phase.lap();
        // The scene is produced before the host is borrowed: driving the
        // pending actions can replace the session, which needs `&mut self`.
        let (width, height) = self.logical_size();
        let mut scene = self.driver_frame(width, height);
        if !self.pending_actions.is_empty() {
            self.drive_pending_actions();
            // The actions changed retained state (and may have replaced the
            // session). Present that, not the geometry probe above.
            scene = self.driver_frame(width, height);
        }
        timing.frame_us = phase.lap();
        // Drain the engine's phase spans for this frame. On the first frame
        // the parse span, recorded during document spawn, comes with them.
        if self.config.phase_timing {
            let [parse, style, layout, paint] = genet_documents::phase::take();
            timing.parse_us = parse;
            timing.style_us = style;
            timing.layout_us = layout;
            timing.paint_us = paint;
        }
        self.publish_accessibility();
        // Sampled after the frame that laid the document out, so the first
        // reading is a real arena rather than a pre-layout one, and before the
        // surface is borrowed for presentation.
        let live = self.sample_live_nodes();
        if self.live_nodes_first.is_none() {
            self.live_nodes_first = live;
        }
        if live.is_some() {
            self.live_nodes_last = live;
        }
        let receipt_ready = match self.receipt_settled() {
            Ok(ready) => ready,
            Err(error) => {
                self.failure = Some(error);
                event_loop.exit();
                return;
            },
        };
        let Some(host) = self.host.as_ref() else {
            return;
        };
        // Stage/retire sources before ordinary scene rasterization. The canvas
        // is an in-order SceneImage now, so it inherits Livery's transform,
        // clip, and opacity stack instead of a flat post-scene overlay.
        #[cfg(feature = "scripted")]
        self.webgl
            .sync_external_images(host.renderer(), self.session.external_texture_draws());
        // The residual so far — accessibility, arena census, receipt
        // conditions — belongs to neither work nor wait.
        let _ = phase.lap();
        let (_scene_texture, view) = host.rasterize_scaled(
            &scene,
            self.width.max(1),
            self.height.max(1),
            ColorLoad::Clear(wgpu::Color::WHITE),
            self.scale_factor,
        );
        timing.raster_us = phase.lap();
        let capture_now = receipt_ready
            && self.config.artifact.is_some()
            && self.capture.is_none()
            && self
                .config
                .frames
                .is_none_or(|limit| self.frames.saturating_add(1) >= limit);
        let captured = if capture_now {
            let path = self
                .config
                .artifact
                .as_deref()
                .expect("capture is gated on an artifact path");
            match receipt::capture(host, &view, self.width, self.height, path) {
                Ok(captured) => Some(captured),
                Err(error) => {
                    self.failure = Some(error);
                    event_loop.exit();
                    return;
                },
            }
        } else {
            None
        };

        // The capture read-back is a receipt cost, not a per-frame one.
        let _ = phase.lap();
        // Presentation wait: the thread blocks on the swapchain here and again
        // at present. The compose encode between them is work, and is timed as
        // work — the two totals are about what the thread was doing, not about
        // where in the frame it happened.
        let Some(frame) = host.acquire() else { return };
        timing.acquire_us = phase.lap();
        let target = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let presented = captured.as_ref().map_or(&view, |captured| &captured.view);
        host.renderer().compose_external_texture(
            presented,
            &target,
            host.format(),
            self.width,
            self.height,
            ExternalTexturePlacement::new([0.0, 0.0, self.width as f32, self.height as f32]),
        );
        timing.compose_us = phase.lap();
        host.queue().present(frame);
        timing.present_us = phase.lap();
        timing.total_us = crate::timing::elapsed_us(frame_started.elapsed());
        self.timing.push(timing);
        self.frames += 1;
        if let Some(captured) = captured {
            self.capture = Some((captured.path, captured.digest));
        }

        if let Some(limit) = self.config.frames {
            let completion_ready = self.config.expect_heading.is_none() || receipt_ready;
            if self.frames >= limit
                && completion_ready
                && (self.config.artifact.is_none() || self.capture.is_some())
            {
                event_loop.exit();
                return;
            }
            // A bounded run owns its redraws until its frame budget and, when
            // requested, semantic receipt completion are both satisfied. The
            // pending-work scheduler supplies timer wakeups between frames.
            if self.frames < limit {
                self.request_redraw();
            }
        } else if receipt_ready
            && (self.config.expect_heading.is_some() || self.config.artifact.is_some())
        {
            event_loop.exit();
        }
    }
}

impl ApplicationHandler for Ortet {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.session.supports_freeze_resume() {
            self.session.resume();
        }
        if self.window.is_some() {
            self.request_redraw();
            return;
        }
        let attributes = Window::default_attributes()
            .with_title(format!("ortet — {}", self.address))
            .with_visible(false)
            .with_inner_size(winit::dpi::PhysicalSize::new(self.width, self.height));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.failure = Some(format!("could not create the window: {error}"));
                event_loop.exit();
                return;
            },
        };
        let size = window.inner_size();
        self.width = size.width.max(1);
        self.height = size.height.max(1);
        self.scale_factor = window.scale_factor() as f32;
        eprintln!(
            "ortet: display scale={} physical={}x{} logical={}x{}",
            self.scale_factor,
            self.width,
            self.height,
            self.logical_size().0,
            self.logical_size().1
        );
        #[cfg(feature = "scripted")]
        self.wake.set_callback({
            let wake_window = window.clone();
            move |event| {
                match event {
                    ScriptWakeEvent::Wake {
                        source,
                        generation,
                        resource,
                    } => {
                        if let Some(resource) = resource {
                            eprintln!(
                                "ortet: receipt wake source={source} generation={generation} resource={resource}"
                            );
                        } else {
                            eprintln!(
                                "ortet: receipt wake source={source} generation={generation}"
                            );
                        }
                        wake_window.request_redraw();
                    },
                    ScriptWakeEvent::Stale {
                        source,
                        generation,
                        active_generation,
                        resource,
                    } => {
                        if let Some(resource) = resource {
                            eprintln!(
                                "ortet: receipt stale-completion dropped source={source} generation={generation} active={active_generation} resource={resource}"
                            );
                        } else {
                            eprintln!(
                                "ortet: receipt stale-completion dropped source={source} generation={generation} active={active_generation}"
                            );
                        }
                    },
                }
            }
        });
        // The bridge must be installed while the native window is hidden on
        // Windows. Frame once to obtain the session's first real projection;
        // an honest empty document is used only if that engine has none.
        let (logical_width, logical_height) = self.logical_size();
        let _ = self.driver_frame(logical_width, logical_height);
        let initial = self
            .a11y
            .publish(self.session.accessibility_projection())
            .expect("accessibility publication always supplies a tree");
        let wake_window = window.clone();
        let mut bridge = AccessKitBridge::new(move || wake_window.request_redraw());
        if let Err(error) = bridge.install(&window, initial) {
            self.failure = Some(format!("could not install accessibility bridge: {error}"));
            event_loop.exit();
            return;
        }
        if bridge.status() == BridgeStatus::Installed {
            eprintln!("[ortet] accessibility bridge installed");
        }
        self.a11y_bridge = Some(bridge);
        match SurfaceHost::from_shared_core(
            window.clone(),
            self.width,
            self.height,
            Arc::clone(&self.core),
        ) {
            Ok(host) => self.host = Some(host),
            Err(error) => {
                self.failure = Some(error);
                event_loop.exit();
                return;
            },
        }
        self.window = Some(window);
        if let Some(window) = self.window.as_ref() {
            window.set_visible(true);
            window.request_redraw();
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if self.session.supports_freeze_resume() {
            self.session.freeze();
        }
        self.wake_deadline = None;
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.schedule_wake(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.window.as_ref().map(|window| window.id()) != Some(window_id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                self.width = size.width.max(1);
                self.height = size.height.max(1);
                if let Some(host) = self.host.as_mut() {
                    host.resize(self.width, self.height);
                }
                self.request_redraw();
            },
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor as f32;
                if let Some(window) = self.window.as_ref() {
                    let size = window.inner_size();
                    self.width = size.width.max(1);
                    self.height = size.height.max(1);
                }
                if let Some(host) = self.host.as_mut() {
                    host.resize(self.width, self.height);
                }
                self.request_redraw();
            },
            WindowEvent::MouseWheel { delta, .. } => {
                // The shared wheel default action: `genet-winit-host` owns the
                // translation, and the nested scroller under the pointer takes
                // it before the document viewport does.
                let (dx, dy) = wheel_delta_from_winit(delta);
                let scale = if self.scale_factor > 0.0 {
                    self.scale_factor
                } else {
                    1.0
                };
                let (dx, dy) = (dx / scale, dy / scale);
                if self.session.scroll_at(self.cursor.0, self.cursor.1, dx, dy) {
                    self.request_redraw();
                }
            },
            WindowEvent::ModifiersChanged(modifiers) => {
                let state = modifiers.state();
                self.modifiers = SessionModifiers {
                    shift: state.shift_key(),
                    control: state.control_key(),
                    alt: state.alt_key(),
                    meta: state.super_key(),
                };
            },
            WindowEvent::CursorMoved { position, .. } => {
                let scale = if self.scale_factor > 0.0 {
                    self.scale_factor
                } else {
                    1.0
                };
                self.cursor = (position.x as f32 / scale, position.y as f32 / scale);
                let (x, y) = self.cursor;
                let _ = self.apply_input(SessionInput::PointerMoved {
                    x,
                    y,
                    modifiers: self.modifiers,
                });
            },
            WindowEvent::MouseInput { state, button, .. } => {
                let (x, y) = self.cursor;
                let _ = self.apply_input(SessionInput::PointerButton {
                    x,
                    y,
                    button: pointer_button_from_winit(button),
                    state: button_state_from_winit(state),
                    modifiers: self.modifiers,
                });
            },
            WindowEvent::KeyboardInput { event, .. } => {
                let allows_default_scroll = self.apply_input(SessionInput::Key {
                    key: session_key_from_winit(&event.logical_key),
                    state: button_state_from_winit(event.state),
                    modifiers: self.modifiers,
                    repeat: event.repeat,
                });
                if event.state == ElementState::Pressed
                    && !self.modifiers.meta && !(self.modifiers.control && !self.modifiers.alt)
                    && let Some(text) = event.text.as_deref().or_else(||
                        matches!(&event.logical_key, Key::Named(NamedKey::Space)).then_some(" "))
                    && !text.is_empty() && text.chars().all(|c| !c.is_control())
                    && matches!(session_key_from_winit(&event.logical_key), SessionKey::Character(_) | SessionKey::Space)
                {
                    let _ = self.apply_input(SessionInput::KeyboardText(text.to_owned()));
                }
                if event.state == ElementState::Pressed
                    && allows_default_scroll
                    && let Some(key) =
                        scroll_key_from_winit(&event.logical_key, self.modifiers.shift)
                    && self.session.scroll_for_key(key)
                {
                    self.request_redraw();
                }
            },
            WindowEvent::Ime(ime) => {
                let _ = self.apply_input(SessionInput::Ime(ime_from_winit(ime)));
            },
            WindowEvent::Focused(focused) => {
                if let Some(bridge) = self.a11y_bridge.as_mut() {
                    bridge.update_window_focus(focused);
                }
                if !focused && self.pointer_captured {
                    let _ = self.apply_input(SessionInput::Cancel);
                    self.pointer_captured = false;
                }
                let _ = self.apply_input(SessionInput::Focus(focused));
            },
            WindowEvent::RedrawRequested => self.render(event_loop),
            _ => {},
        }
    }
}

fn session_key_from_winit(key: &Key) -> SessionKey {
    match key {
        Key::Character(text) => SessionKey::Character(text.to_string()),
        Key::Named(NamedKey::Enter) => SessionKey::Enter,
        Key::Named(NamedKey::Tab) => SessionKey::Tab,
        Key::Named(NamedKey::Backspace) => SessionKey::Backspace,
        Key::Named(NamedKey::Delete) => SessionKey::Delete,
        Key::Named(NamedKey::Escape) => SessionKey::Escape,
        Key::Named(NamedKey::Space) => SessionKey::Space,
        Key::Named(NamedKey::ArrowLeft) => SessionKey::ArrowLeft,
        Key::Named(NamedKey::ArrowRight) => SessionKey::ArrowRight,
        Key::Named(NamedKey::ArrowUp) => SessionKey::ArrowUp,
        Key::Named(NamedKey::ArrowDown) => SessionKey::ArrowDown,
        Key::Named(NamedKey::Home) => SessionKey::Home,
        Key::Named(NamedKey::End) => SessionKey::End,
        Key::Named(NamedKey::PageUp) => SessionKey::PageUp,
        Key::Named(NamedKey::PageDown) => SessionKey::PageDown,
        _ => SessionKey::Unidentified,
    }
}

/// The keyboard scroll defaults, for keys the session did not consume.
fn scroll_key_from_winit(key: &Key, shift: bool) -> Option<SessionScrollKey> {
    Some(match key {
        Key::Named(NamedKey::ArrowUp) => SessionScrollKey::LineUp,
        Key::Named(NamedKey::ArrowDown) => SessionScrollKey::LineDown,
        Key::Named(NamedKey::PageUp) => SessionScrollKey::PageUp,
        Key::Named(NamedKey::PageDown) => SessionScrollKey::PageDown,
        Key::Named(NamedKey::Home) => SessionScrollKey::Home,
        Key::Named(NamedKey::End) => SessionScrollKey::End,
        Key::Named(NamedKey::Space) => {
            if shift {
                SessionScrollKey::PageUp
            } else {
                SessionScrollKey::PageDown
            }
        },
        _ => return None,
    })
}

fn pointer_button_from_winit(button: MouseButton) -> SessionPointerButton {
    match button {
        MouseButton::Left => SessionPointerButton::Primary,
        MouseButton::Right => SessionPointerButton::Secondary,
        MouseButton::Middle | MouseButton::Back | MouseButton::Forward | MouseButton::Other(_) => {
            SessionPointerButton::Auxiliary
        },
    }
}

fn button_state_from_winit(state: ElementState) -> SessionButtonState {
    match state {
        ElementState::Pressed => SessionButtonState::Pressed,
        ElementState::Released => SessionButtonState::Released,
    }
}

fn ime_from_winit(ime: winit::event::Ime) -> SessionIme {
    match ime {
        winit::event::Ime::Enabled => SessionIme::Enabled,
        winit::event::Ime::Preedit(text, selection) => SessionIme::Preedit { text, selection },
        winit::event::Ime::Commit(text) => SessionIme::Commit(text),
        winit::event::Ime::Disabled => SessionIme::Disabled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::EngineChoice;

    #[test]
    fn livery_selection_reports_the_concrete_engine_id() {
        let engine = make_engine(EngineChoice::Livery, OrtetFetcher::local_only())
            .expect("default Livery engine is available");
        assert_eq!(
            engine.engine_id(),
            document_session_api::engine_ids::ENGINE_GENET_LIVERY
        );
    }

    #[cfg(feature = "scripted")]
    #[test]
    fn boa_selection_reports_the_concrete_engine_id() {
        let engine = make_engine(EngineChoice::Boa, OrtetFetcher::local_only())
            .expect("Boa is available when the scripted feature is enabled");
        assert_eq!(
            engine.engine_id(),
            document_session_api::engine_ids::ENGINE_GENET_SCRIPTED
        );
    }

    #[cfg(feature = "scripted-nova")]
    #[cfg(target_pointer_width = "64")]
    #[test]
    fn nova_selection_reports_the_concrete_engine_id() {
        let engine = make_engine(EngineChoice::Nova, OrtetFetcher::local_only())
            .expect("Nova is available on 64-bit targets with the feature enabled");
        assert_eq!(
            engine.engine_id(),
            document_session_api::engine_ids::ENGINE_GENET_SCRIPTED_NOVA
        );
    }

    #[cfg(not(feature = "scripted"))]
    #[test]
    fn unavailable_boa_reports_the_build_feature() {
        let error = match make_engine(EngineChoice::Boa, OrtetFetcher::local_only()) {
            Ok(_) => panic!("default Ortet keeps Boa out of its dependency cone"),
            Err(error) => error,
        };
        assert!(error.contains("--features scripted"), "{error}");
    }

    #[cfg(not(feature = "scripted-nova"))]
    #[test]
    fn unavailable_nova_reports_the_build_feature() {
        let error = match make_engine(EngineChoice::Nova, OrtetFetcher::local_only()) {
            Ok(_) => panic!("default Ortet keeps Nova out of its dependency cone"),
            Err(error) => error,
        };
        assert!(error.contains("--features scripted-nova"), "{error}");
    }

    // TASK5 DRAFT SHELL FIXTURES BEGIN. Driver adapter is absence only.
    #[derive(Clone)]
    struct T5ShellReply { final_address: String, fail: bool }
    struct T5ShellSession {
        marker: String, projected: Option<String>, pending_projection:Option<(String,String)>, effects: std::collections::VecDeque<SessionEffect>,
        encoded: Option<genet_host_api::ResourceRequest>,
        log: Arc<std::sync::Mutex<Vec<String>>>,
    }
    impl T5ShellSession {
        fn callback(&mut self,event:String) {
            self.log.lock().unwrap().push(event.clone());
            if self.pending_projection.as_ref().is_some_and(|(on,_)|on==&event) {
                self.projected=Some(self.pending_projection.take().unwrap().1);
            }
        }
    }
    impl DocumentSession<Scene> for T5ShellSession {
        fn frame(&mut self,w:u32,h:u32)->Scene {
            self.callback("frame".into()); Scene::new(w,h)
        }
        fn pump(&mut self,_now:f64) { self.callback("pump".into()); }
        fn scroll_by(&mut self,_x:f32,_y:f32)->bool { false }
        fn scroll_for_key(&mut self,_key:SessionScrollKey)->bool { false }
        fn click_at(&mut self,_x:f32,_y:f32)->SessionClick {
            self.log.lock().unwrap().push("legacy:click".into()); SessionClick::Miss
        }
        fn pointer_down(&mut self,_x:f32,_y:f32)->SessionClick {
            self.log.lock().unwrap().push("legacy:down".into()); SessionClick::Handled
        }
        fn pointer_up(&mut self,_x:f32,_y:f32)->SessionClick {
            self.log.lock().unwrap().push("legacy:up".into()); SessionClick::Miss
        }
        fn input(&mut self,input:SessionInput)->document_session_api::SessionInputResult {
            let name=match input {
                SessionInput::PointerButton{state,..}=>format!("input:{state:?}"),
                SessionInput::PointerMoved{..}=>"input:move".into(),
                SessionInput::Key{state,..}=>format!("input:key:{state:?}"),
                _=>"input:other".into(),
            };
            self.callback(name);
            document_session_api::SessionInputResult { effect:self.effects.pop_front().unwrap_or(SessionEffect::Ignored),
                cursor:None,capture:None,editable:false }
        }
        fn current_address(&self)->Option<String> { self.projected.clone() }
        fn take_encoded_form_submission(&mut self)->Option<document_session_api::SessionEncodedFormSubmission> {
            self.log.lock().unwrap().push("take-encoded".into());
            self.encoded.take().map(|r| document_session_api::SessionEncodedFormSubmission {
                target: String::new(), request: document_session_api::SessionDocumentRequest {
                    url:r.url,method:r.method,headers:r.headers,body:r.body,cache:r.cache,redirect:r.redirect,
                    mode:r.mode,referrer:r.referrer,referrer_policy:r.referrer_policy,credentials:r.credentials,integrity:r.integrity,
                }
            })
        }
        fn links(&self)->Vec<document_session_api::SessionLink> {vec![]}
        fn as_any_ref(&self)->&dyn std::any::Any {self}
        fn as_any(&mut self)->&mut dyn std::any::Any {self}
    }
    struct T5ShellEngine { id:String, reply:T5ShellReply, log:Arc<std::sync::Mutex<Vec<String>>> }
    impl SessionEngine<Scene> for T5ShellEngine {
        fn engine_id(&self)->&str {&self.id}
        fn spawn(&self,_request:&SessionSpawnRequest)->Result<Box<dyn DocumentSession<Scene>>,document_session_api::SessionError> {
            self.log.lock().unwrap().push(format!("ordinary:{}",self.id));
            if self.reply.fail {return Err(document_session_api::SessionError::SpawnFailed("fixture network failure".into()));}
            Ok(Box::new(T5ShellSession { marker:"NEW".into(),projected:Some(self.reply.final_address.clone()),pending_projection:None,
                effects:Default::default(),encoded:None,log:self.log.clone() }))
        }
        fn spawn_request(&self,_spawn:&SessionSpawnRequest,_request:&document_session_api::SessionDocumentRequest)
            ->Result<document_session_api::SessionRequestSpawnResult<Scene>,document_session_api::SessionError> {
            self.log.lock().unwrap().push(format!("rich:{}",self.id));
            if self.reply.fail {return Err(document_session_api::SessionError::SpawnFailed("fixture network failure".into()));}
            Ok(document_session_api::SessionRequestSpawnResult { navigation_address:self.reply.final_address.clone(),
                session:Box::new(T5ShellSession { marker:"NEW".into(),projected:Some(self.reply.final_address.clone()),pending_projection:None,
                    effects:Default::default(),encoded:None,log:self.log.clone() }) })
        }
    }
    struct T5ShellState {
        engine:Box<dyn SessionEngine<Scene>>,session:Box<dyn DocumentSession<Scene>>,
        address:String,a11y:Accessibility,captured:bool,log:Arc<std::sync::Mutex<Vec<String>>>,
    }
    enum T5ShellCommand { Click(f32,f32),Input(SessionInput),Link(String),Initial,Pump(f64),Frame(u32,u32) }
    #[derive(Debug)]
    struct T5ShellResult { installed:bool,declined:bool,address_changed:bool }
    // GREEN must replace this body with actual production SessionDriver calls only.
    // Never put input/resolve/cache/status/installation algorithms in this adapter.
    fn t5_shell_driver_absent(_state:&mut T5ShellState,_command:T5ShellCommand)->Option<T5ShellResult> {
        let mut driver=SessionDriver {engine:_state.engine.as_ref(),session:&mut _state.session,address:&mut _state.address,
            a11y:&mut _state.a11y,captured:&mut _state.captured,viewport:(320,240)};
        let outcome=match _command {
            T5ShellCommand::Click(x,y)=>driver.click(x,y),T5ShellCommand::Input(input)=>driver.input(input),
            T5ShellCommand::Link(target)=>driver.navigate(&target),T5ShellCommand::Initial=>driver.sync_address(),
            T5ShellCommand::Pump(now)=>driver.pump(now),T5ShellCommand::Frame(w,h)=>driver.frame(w,h).1,
        };
        Some(T5ShellResult {installed:outcome.installed,declined:outcome.declined,address_changed:outcome.address_changed})
    }
    fn t5_shell_request(method:&str)->genet_host_api::ResourceRequest {
        genet_host_api::ResourceRequest {url:"https://forms.test/same".into(),method:method.into(),
            headers:vec![("Content-Type".into(),"text/plain".into())],body:Some(vec![0,128,255]),
            cache:"default".into(),redirect:"follow".into(),mode:"navigate".into(),
            referrer:"https://forms.test/source".into(),referrer_policy:"no-referrer".into(),
            credentials:"include".into(),integrity:String::new(),
            upgrade_insecure_requests:false,block_all_mixed_content:false}
    }
    fn t5_shell_state(effects:Vec<SessionEffect>,encoded:Option<genet_host_api::ResourceRequest>,fail:bool)->T5ShellState {
        let log=Arc::new(std::sync::Mutex::new(Vec::new()));
        T5ShellState {engine:Box::new(T5ShellEngine{id:"chosen-factory".into(),
                reply:T5ShellReply{final_address:"https://forms.test/final/index.html".into(),fail},log:log.clone()}),
            session:Box::new(T5ShellSession{marker:"OLD".into(),projected:None,pending_projection:None,effects:effects.into(),encoded,log:log.clone()}),
            address:"https://forms.test/same".into(),a11y:Accessibility::default(),captured:false,log}
    }
    fn t5_shell_submit()->SessionEffect {SessionEffect::Submit(document_session_api::SessionFormSubmission{
        action:"https://forms.test/same".into(),method:document_session_api::SessionFormMethod::Post,
        fields:vec![("q".into(),"legacy".into())]})}
    fn t5_shell_key()->SessionInput {SessionInput::Key{key:SessionKey::Enter,state:SessionButtonState::Pressed,
        modifiers:SessionModifiers::default(),repeat:false}}
    fn t5_shell_run(state:&mut T5ShellState,command:T5ShellCommand)->T5ShellResult {
        let result=t5_shell_driver_absent(state,command);
        assert!(result.is_some(),"missing capability: production headless SessionDriver seam; no test-only routing substitute");
        result.unwrap()
    }
    fn t5_shell_marker(state:&T5ShellState)->&str {
        &state.session.as_any_ref().downcast_ref::<T5ShellSession>().unwrap().marker
    }
    #[test]
    fn phase_c_t5_ortet_driver_click_uses_input() {
        let mut s=t5_shell_state(vec![SessionEffect::Handled,t5_shell_submit()],Some(t5_shell_request("POST")),false);
        t5_shell_run(&mut s,T5ShellCommand::Click(7.0,9.0));
        let log=s.log.lock().unwrap();
        assert_eq!(&log[..2],["input:Pressed","input:Released"]);
        assert!(!log.iter().any(|x|x.starts_with("legacy:")));
        assert!(log.iter().any(|x|x=="rich:chosen-factory"));
    }
    #[test]
    fn phase_c_t5_ortet_submit_consumed_immediately() {
        let mut s=t5_shell_state(vec![t5_shell_submit()],Some(t5_shell_request("POST")),false);
        t5_shell_run(&mut s,T5ShellCommand::Input(t5_shell_key()));
        assert_eq!(&*s.log.lock().unwrap(), &["input:key:Pressed","take-encoded","rich:chosen-factory"]);
    }
    #[test]
    fn phase_c_t5_ortet_same_url_form_link_distinction() {
        let mut s=t5_shell_state(vec![t5_shell_submit()],Some(t5_shell_request("POST")),false);
        assert!(!t5_shell_run(&mut s,T5ShellCommand::Link("https://forms.test/same".into())).installed);
        assert!(s.log.lock().unwrap().is_empty());
        assert!(t5_shell_run(&mut s,T5ShellCommand::Input(t5_shell_key())).installed);
        let mut get=t5_shell_state(vec![t5_shell_submit()],Some(genet_host_api::ResourceRequest{
            body:None,headers:vec![],..t5_shell_request("GET")}),false);
        assert!(t5_shell_run(&mut get,T5ShellCommand::Input(t5_shell_key())).installed);
        assert_eq!(get.log.lock().unwrap().iter().filter(|x|x.as_str()=="rich:chosen-factory").count(),1);
    }
    #[test]
    fn phase_c_t5_ortet_chosen_boxed_factory_retained() {
        let mut s=t5_shell_state(vec![t5_shell_submit()],Some(t5_shell_request("POST")),false);
        let id=s.engine.engine_id().to_owned();
        t5_shell_run(&mut s,T5ShellCommand::Input(t5_shell_key()));
        assert_eq!(s.engine.engine_id(),id);
        assert_eq!(t5_shell_marker(&s),"NEW");
        assert!(!s.log.lock().unwrap().iter().any(|x|x.starts_with("ordinary:")));
    }
    #[test]
    fn phase_c_t5_ortet_final_address_install() {
        let mut s=t5_shell_state(vec![t5_shell_submit()],Some(t5_shell_request("POST")),false);
        s.captured=true;
        assert!(t5_shell_run(&mut s,T5ShellCommand::Input(t5_shell_key())).installed);
        assert!(!s.captured,"actual driver clears old capture after installation");
        assert_eq!(s.address,"https://forms.test/final/index.html");
        assert_eq!(t5_shell_marker(&s),"NEW");
    }
    #[test]
    fn phase_c_t5_ortet_transport_failure_old_session() {
        let mut s=t5_shell_state(vec![t5_shell_submit()],Some(t5_shell_request("POST")),true);
        s.captured=true;
        assert!(t5_shell_run(&mut s,T5ShellCommand::Input(t5_shell_key())).declined);
        assert!(s.captured,"failed replacement keeps the old host state");
        assert_eq!((t5_shell_marker(&s),s.address.as_str()),("OLD","https://forms.test/same"));
        assert!(!s.log.lock().unwrap().iter().any(|x|x.starts_with("ordinary:")));
    }
    #[test]
    fn phase_c_t5_ortet_latest_input_no_replay() {
        let mut s=t5_shell_state(vec![t5_shell_submit(),SessionEffect::Ignored],Some(t5_shell_request("POST")),false);
        t5_shell_run(&mut s,T5ShellCommand::Input(t5_shell_key()));
        t5_shell_run(&mut s,T5ShellCommand::Input(SessionInput::PointerMoved{x:0.0,y:0.0,modifiers:SessionModifiers::default()}));
        assert_eq!(s.log.lock().unwrap().iter().filter(|x|x.as_str()=="rich:chosen-factory").count(),1);
    }
    #[test]
    fn phase_c_t5_ortet_encoded_absent_explicit_decline() {
        let mut s=t5_shell_state(vec![t5_shell_submit()],None,false);
        assert!(t5_shell_run(&mut s,T5ShellCommand::Input(t5_shell_key())).declined);
        assert_eq!(t5_shell_marker(&s),"OLD");
        assert!(!s.log.lock().unwrap().iter().any(|x|x.starts_with("rich:")||x.starts_with("ordinary:")));
    }
    #[test]
    fn phase_c_t5_ortet_status_before_install() {
        // Classification belongs to the real factory fixtures. Driver sees Ok/Err only.
        for (label,fail) in [("404-readable",false),("204",true),("205",true),("opaque",true),("network",true)] {
            let mut s=t5_shell_state(vec![t5_shell_submit()],Some(t5_shell_request("POST")),fail);
            let r=t5_shell_run(&mut s,T5ShellCommand::Input(t5_shell_key()));
            assert_eq!(r.installed,!fail,"driver factory outcome {label}");
            assert_eq!(t5_shell_marker(&s),if fail {"OLD"}else{"NEW"});
        }
    }
    #[test]
    fn phase_c_t5_ortet_initial_address_projection() {
        let mut s=t5_shell_state(vec![],None,false);
        s.session.as_any().downcast_mut::<T5ShellSession>().unwrap().projected=Some("https://forms.test/final/index.html".into());
        assert!(t5_shell_run(&mut s,T5ShellCommand::Initial).address_changed);
        assert_eq!(s.address,"https://forms.test/final/index.html");
        assert!(s.log.lock().unwrap().is_empty());
    }
    #[test]
    fn phase_c_t5_ortet_driver_sync_input_pump_frame() {
        let mut s=t5_shell_state(vec![SessionEffect::Ignored],None,false);
        for (url,command,event) in [
            ("https://forms.test/same#input",T5ShellCommand::Input(t5_shell_key()),"input:key:Pressed"),
            ("https://forms.test/same#pump",T5ShellCommand::Pump(1.0),"pump"),
            ("https://forms.test/same#frame",T5ShellCommand::Frame(320,240),"frame"),
        ] {
            s.session.as_any().downcast_mut::<T5ShellSession>().unwrap().pending_projection=Some((event.into(),url.into()));
            assert!(t5_shell_run(&mut s,command).address_changed);
            assert_eq!(s.address,url);
            assert_eq!(s.log.lock().unwrap().last().map(String::as_str),Some(event));
        }
    }
    #[test]
    fn phase_c_t5_ortet_address_none_failure_preserves() {
        let mut s=t5_shell_state(vec![t5_shell_submit()],Some(t5_shell_request("POST")),true);
        assert!(!t5_shell_run(&mut s,T5ShellCommand::Initial).address_changed);
        t5_shell_run(&mut s,T5ShellCommand::Input(t5_shell_key()));
        assert_eq!(s.address,"https://forms.test/same");
        assert_eq!(t5_shell_marker(&s),"OLD");
    }
    // TASK5 DRAFT SHELL FIXTURES END.
}

// TASK5 F2 INDEPENDENT REAL NATIVE DRIVER FIXTURES BEGIN.
#[cfg(test)]
mod f2_native_decline_tests {
    use super::*;

    #[derive(Clone, Default)]
    struct FetchCalls(Arc<std::sync::Mutex<Vec<String>>>);
    impl genet_host_api::ResourceFetcher for FetchCalls {
        fn fetch(&self, url: &str) -> Option<Vec<u8>> {
            self.0.lock().unwrap().push(format!("url:{url}"));
            None
        }
        fn fetch_request(&self, request: &genet_host_api::ResourceRequest)
            -> Option<genet_host_api::ResourceFetchResponse>
        {
            self.0.lock().unwrap().push(format!("request:{}:{}", request.method, request.url));
            None
        }
    }

    // This only selects a real painted node and its published geometry.
    // Production SessionDriver owns all input, decline, transport and replacement.
    fn point(session: &mut Box<dyn DocumentSession<Scene>>, name: &str) -> (f32, f32) {
        session.frame(320, 240);
        let projection = session.accessibility_projection().expect("real Livery projection");
        let node = projection.nodes().iter().find(|node| node.name.as_deref() == Some(name)
            && matches!(node.role, document_session_api::DocumentA11yRole::Button
                | document_session_api::DocumentA11yRole::TextField))
            .expect("real named native control");
        let rect = node.bounds.expect("visible native control bounds");
        assert!(rect.width > 0.0 && rect.height > 0.0);
        (rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
    }

    fn check_target(target: &str) {
        let fetch = FetchCalls::default();
        let engine = LiverySessionEngine::new(fetch.clone());
        let original_address = "https://forms.test/source#old";
        let source = format!("<style>body{{margin:0}}input,button{{display:block;width:180px;height:32px;margin:6px}}</style><p>OLD</p><form action='/send' method='post'><input id='q' aria-label='Value' name='q' required value=''><button formtarget='{target}'>Attempt</button></form>");
        let mut session = engine.spawn(&SessionSpawnRequest::new(original_address)
            .with_body(source).with_viewport(320, 240)).expect("actual native Livery session");
        assert!(session.as_any_ref().is::<genet_documents::LiveryDocumentSession>());
        assert!(fetch.0.lock().unwrap().is_empty(), "supplied source creates no transport");
        let mut address = original_address.to_string();
        let mut a11y = Accessibility::default();
        let mut captured = false;

        // Control 1 runs before the target-decline assertion: invalid form is
        // not a navigation intent, even when its submitter has this target.
        let (x, y) = point(&mut session, "Attempt");
        let invalid = SessionDriver { engine: &engine, session: &mut session, address: &mut address,
            a11y: &mut a11y, captured: &mut captured, viewport: (320, 240) }.click(x, y);
        assert!(!invalid.installed && !invalid.declined);
        assert!(!matches!(&invalid.input.as_ref().unwrap().effect, SessionEffect::Submit(_)));
        assert!(session.take_encoded_form_submission().is_none());
        assert!(fetch.0.lock().unwrap().is_empty());
        assert_eq!(address, original_address);

        // Control 2 uses actual ordinary host input to make the form valid.
        let (x, y) = point(&mut session, "Value");
        let focus = SessionDriver { engine: &engine, session: &mut session, address: &mut address,
            a11y: &mut a11y, captured: &mut captured, viewport: (320, 240) }.click(x, y);
        assert!(!focus.installed && !focus.declined);
        let edit = SessionDriver { engine: &engine, session: &mut session, address: &mut address,
            a11y: &mut a11y, captured: &mut captured, viewport: (320, 240) }
            .input(SessionInput::Text("ok".into()));
        assert_eq!(edit.input.as_ref().unwrap().effect, SessionEffect::Handled);
        assert!(!edit.installed && !edit.declined);
        assert!(session.take_encoded_form_submission().is_none());
        assert!(fetch.0.lock().unwrap().is_empty());
        assert_eq!(address, original_address);

        // A canceled composition is also a non-navigation outcome.
        let preedit = SessionDriver { engine: &engine, session: &mut session, address: &mut address,
            a11y: &mut a11y, captured: &mut captured, viewport: (320, 240) }
            .input(SessionInput::Ime(SessionIme::Preedit { text: "draft".into(), selection: None }));
        assert!(!preedit.installed && !preedit.declined);
        let cancel = SessionDriver { engine: &engine, session: &mut session, address: &mut address,
            a11y: &mut a11y, captured: &mut captured, viewport: (320, 240) }.input(SessionInput::Cancel);
        assert_eq!(cancel.input.as_ref().unwrap().effect, SessionEffect::Cancelled);
        assert!(!cancel.installed && !cancel.declined);
        assert!(session.take_encoded_form_submission().is_none());
        assert!(fetch.0.lock().unwrap().is_empty());
        assert_eq!(address, original_address);

        let (x, y) = point(&mut session, "Attempt");
        a11y.publish(session.accessibility_projection()).expect("actual custody publication");
        let original_custody = format!("{a11y:?}");
        let original_session = (&*session as *const dyn DocumentSession<Scene>).cast::<()>();
        captured = true;
        let outcome = SessionDriver { engine: &engine, session: &mut session, address: &mut address,
            a11y: &mut a11y, captured: &mut captured, viewport: (320, 240) }.click(x, y);
        // These retention/transport observations run before the expected RED.
        assert!(!outcome.installed);
        assert!(!outcome.address_changed);
        assert!(fetch.0.lock().unwrap().is_empty(), "unsupported target must not call either transport seam");
        assert_eq!((&*session as *const dyn DocumentSession<Scene>).cast::<()>(), original_session);
        assert_eq!(address, original_address);
        assert_eq!(session.current_address().as_deref(), Some(original_address));
        assert_eq!(format!("{a11y:?}"), original_custody, "no replacement of actual accessibility custody");
        assert!(session.clip().unwrap().text.contains("OLD"));
        assert!(outcome.declined, "valid unsupported target must be an explicit actual driver decline: {target}");
        // These carrier assertions only run after the decline assertion passes.
        assert!(matches!(&outcome.input.as_ref().unwrap().effect, SessionEffect::Submit(_)));
        assert_eq!(outcome.error.as_deref(), Some("encoded submission or supported top target unavailable"));
        assert!(session.take_encoded_form_submission().is_none());
        assert!(captured, "decline retains old host capture state");
    }

    #[test]
    fn phase_c_f2_ortet_real_native_child_target_declined() {
        check_target("child");
    }

    #[test]
    fn phase_c_f2_ortet_real_native_blank_target_declined() {
        check_target("_blank");
    }
}
// TASK5 F2 INDEPENDENT REAL NATIVE DRIVER FIXTURES END.
