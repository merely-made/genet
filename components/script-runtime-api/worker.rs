/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Dedicated workers: a second agent of the same engine in a supervised
//! process on native hosts, or a thread under the explicit trusted policy.
//!
//! Both engines are `!Send`, so nothing engine-shaped crosses the boundary. The
//! worker thread constructs its own [`Runtime`] and owns it for its whole life;
//! the only things on the channel are strings (the serialized clone record from
//! [`crate::structured_clone`]) and plain request/response data.
//!
//! Placement: `Worker` and `__workerPump` live on the page runtime; the worker
//! runtime additionally converts its global into a `DedicatedWorkerGlobalScope`
//! (no `document`, no `window`) and gets `importScripts`, `close`, and a
//! `postMessage` that emits onto the link.
//!
//! Resource loads (the worker script, `importScripts`, `fetch`) are **synchronous
//! requests back to the page**: the worker blocks on the channel and the page
//! answers from its own resource route ([`ScriptResourceLoader`], else the page's
//! [`FetchHandler`]). That is what "the same resource route as the page" means
//! here, and it keeps the network stack on the thread that owns it.
//!
//! Residuals, named rather than faked: module workers throw `NotSupportedError`;
//! `SharedWorker` does not exist; `BroadcastChannel` stays per-agent; a
//! transferred `ArrayBuffer` copies its bytes onto the wire and leaves the
//! sender's handle in the cheap-globals detachment emulation.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{
    Receiver, RecvTimeoutError, Sender, SyncSender, TryRecvError, TrySendError, channel,
    sync_channel,
};
use std::time::{Duration, Instant};

use script_engine_api::{CallCx, NativeFn, ScriptEngine};
use serde::{Deserialize, Serialize};

#[cfg(not(target_arch = "wasm32"))]
use crate::worker_process::{ProcessExit, WorkerProcessHandle};

use crate::{FetchHandler, FetchOutcome, FetchRequest, HostState, Runtime, SharedHost, js_str};

/// A host must explicitly accept cooperative shutdown for trusted scripts.
/// The default requires an explicitly configured native helper process so
/// running VM work and its nested worker tree can be killed and reaped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerPolicy {
    RequireCancellation,
    TrustedCooperative {
        resource_timeout: Duration,
        shutdown_grace: Duration,
    },
}

impl Default for WorkerPolicy {
    fn default() -> Self {
        Self::RequireCancellation
    }
}

/// Results for directly owned worker threads and root helper processes.
/// Descendants inside a helper are reclaimed with its process; trusted thread
/// descendants are untracked and may still execute after the root is joined.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkerShutdownReport {
    pub direct_joined: usize,
    pub direct_detached: usize,
    /// Root worker processes that exited and were reaped without a kill.
    pub processes_joined: usize,
    /// Root worker processes killed and reaped, immediately or after grace.
    pub processes_killed: usize,
    /// Root worker processes that exited nonzero and were reaped.
    pub processes_crashed: usize,
    /// A process could not be reaped; ownership must not be claimed.
    pub processes_failed: usize,
    /// True when trusted worker threads might have spawned untracked descendants.
    pub descendants_untracked: bool,
}

/// Where a worker's classic script, `importScripts` target or `fetch()` is read
/// from when the page has no network seam. Disk-mode WPT installs one; a hosted
/// page leaves it unset and everything goes through the [`FetchHandler`].
pub trait ScriptResourceLoader {
    /// The text of `url` on the page's resource route, or `None` to fall through
    /// to the network seam.
    fn load(&self, url: &str) -> Option<String>;

    /// Optional metadata for the existing plain-GET document source route.
    /// None leaves the source-only `load` route authoritative. Some claims the
    /// load even on failure, so network errors and 204/205 cannot fall through
    /// to a different fetcher. Policies are those of this legacy loader, not
    /// the fields of a rich navigation request; rich requests bypass this seam.
    fn load_document(&self, _url: &str) -> Option<FetchOutcome> {
        None
    }

    /// Load a classic script on the same resource route. Text-only loaders
    /// already return decoded source, but cannot verify integrity against the
    /// original response bytes, so they decline nonempty integrity metadata.
    /// Byte-backed hosts override this with their parser's decoding/SRI path.
    fn load_classic_script(
        &self,
        url: &str,
        _charset: Option<&str>,
        integrity: Option<&str>,
    ) -> Option<String> {
        if integrity.is_some_and(|metadata| !metadata.trim().is_empty()) {
            return None;
        }
        self.load(url)
    }

    /// Start a worker resource request. Returning `Some(request)` declines
    /// deferred service and hands the request back to the page fetch route;
    /// returning `None` means the loader accepted it and will invoke
    /// `complete` later.
    fn start(
        &self,
        _id: u64,
        request: FetchRequest,
        _complete: Box<dyn FnOnce(FetchOutcome) + Send>,
    ) -> Option<FetchRequest> {
        Some(request)
    }
}

/// Page -> worker.
#[derive(Serialize, Deserialize)]
pub(crate) enum ToWorker {
    Message(String),
    PortMessage(String, String),
    Resource(u64, Box<FetchOutcome>),
    Terminate,
}

/// Worker -> page.
#[derive(Serialize, Deserialize)]
pub(crate) enum FromWorker {
    Message(String),
    PortMessage(String, String),
    Error(String),
    Resource(u64, Box<FetchRequest>),
    /// "I have consumed n messages and have nothing left to do." The count is
    /// what makes it safe: the page only believes it if `n` matches everything
    /// it has sent, so a report that crossed a later message in flight is
    /// ignored rather than quiescing the page over live work.
    Idle(u64),
    Closed,
}

/// Native process IPC uses bounded nonblocking queues. A full queue marks the
/// bridge failed, so the owner kills the process instead of losing a message.
pub(crate) enum WorkerSender<T> {
    Thread(Sender<T>),
    Process {
        tx: SyncSender<T>,
        failed: Arc<AtomicBool>,
        child_side: bool,
    },
}

impl<T> Clone for WorkerSender<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Thread(tx) => Self::Thread(tx.clone()),
            Self::Process {
                tx,
                failed,
                child_side,
            } => Self::Process {
                tx: tx.clone(),
                failed: failed.clone(),
                child_side: *child_side,
            },
        }
    }
}

impl<T> WorkerSender<T> {
    pub(crate) fn send(&self, message: T) -> Result<(), ()> {
        match self {
            Self::Thread(tx) => tx.send(message).map_err(|_| ()),
            Self::Process {
                tx,
                failed,
                child_side,
            } => match tx.try_send(message) {
                Ok(()) => Ok(()),
                Err(TrySendError::Full(_)) => {
                    failed.store(true, Ordering::Release);
                    if *child_side {
                        std::process::exit(2);
                    }
                    Err(())
                },
                Err(TrySendError::Disconnected(_)) => {
                    failed.store(true, Ordering::Release);
                    if *child_side {
                        std::process::exit(2);
                    }
                    Err(())
                },
            },
        }
    }
}

/// What the spawning thread hands the worker thread. Every field is `Send`; no
/// engine type appears, which is why the spawn compiles without an engine bound.
pub(crate) struct WorkerBoot {
    pub(crate) url: String,
    pub(crate) name: String,
    pub(crate) rx: Receiver<ToWorker>,
    pub(crate) tx: WorkerSender<FromWorker>,
    pub(crate) wake: Option<Arc<dyn Fn() + Send + Sync>>,
    pub(crate) policy: WorkerPolicy,
    /// The page's shared clock: the worker's time origin is a reading of it.
    pub(crate) clock: crate::SharedClock,
}

/// The page's handle on one live worker.
pub(crate) struct WorkerHandle {
    tx: WorkerSender<ToWorker>,
    rx: Receiver<FromWorker>,
    join: Option<std::thread::JoinHandle<()>>,
    #[cfg(not(target_arch = "wasm32"))]
    process: Option<WorkerProcessHandle>,
    #[cfg(not(target_arch = "wasm32"))]
    bridge_failed: Option<Arc<AtomicBool>>,
    /// The worker reported it had nothing left to do. The page's drive loop may
    /// quiesce only when every worker is idle: the worker cannot wake itself.
    idle: bool,
    alive: bool,
    /// Messages sent to this worker; compared against the count in `Idle`.
    sent: Arc<AtomicU64>,
    shutdown_grace: Duration,
    /// Maximum envelopes admitted to the page event queue per pump turn.
    pump_limit: usize,
}

/// Process-unique ids for cross-agent message ports.
static NEXT_PORT_ID: AtomicU64 = AtomicU64::new(1);

/// The worker thread's end of the link, shared by its fetch handler and its loop.
pub(crate) struct WorkerLink {
    tx: WorkerSender<FromWorker>,
    rx: Receiver<ToWorker>,
    wake: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Envelopes waiting for `__ws_take`.
    inbox: VecDeque<String>,
    next_req: u64,
    /// Messages taken off the channel, echoed back in `Idle`.
    consumed: u64,
    /// `close()` was called in the worker.
    closed: bool,
    /// `terminate()` was called on the page, or the link died.
    terminated: bool,
    resource_timeout: Duration,
}

impl WorkerLink {
    fn notify(&self) {
        if let Some(wake) = &self.wake {
            wake();
        }
    }

    fn envelope(msg: ToWorker) -> Option<String> {
        match msg {
            ToWorker::Message(p) => Some(format!("{{\"k\":\"m\",\"d\":{}}}", js_str(&p))),
            ToWorker::PortMessage(id, p) => Some(format!(
                "{{\"k\":\"p\",\"i\":{},\"d\":{}}}",
                js_str(&id),
                js_str(&p)
            )),
            _ => None,
        }
    }

    /// Drain the channel into the inbox. `block` waits that long for the first
    /// message. Returns whether anything arrived.
    fn pump(&mut self, block: Option<Duration>) -> bool {
        let mut got = false;
        if let Some(d) = block {
            match self.rx.recv_timeout(d) {
                Ok(ToWorker::Terminate) => {
                    self.consumed += 1;
                    self.terminated = true;
                    return true;
                },
                Ok(m) => {
                    self.consumed += 1;
                    if let Some(e) = Self::envelope(m) {
                        self.inbox.push_back(e);
                    }
                    got = true;
                },
                Err(RecvTimeoutError::Disconnected) => {
                    self.terminated = true;
                    return true;
                },
                Err(RecvTimeoutError::Timeout) => {},
            }
        }
        loop {
            match self.rx.try_recv() {
                Ok(ToWorker::Terminate) => {
                    self.consumed += 1;
                    self.terminated = true;
                    return true;
                },
                Ok(m) => {
                    self.consumed += 1;
                    if let Some(e) = Self::envelope(m) {
                        self.inbox.push_back(e);
                    }
                    got = true;
                },
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.terminated = true;
                    return true;
                },
            }
        }
        got
    }

    /// Ask the page for a resource and block until it answers. Messages that
    /// arrive meanwhile are buffered, not dropped.
    fn request(&mut self, req: FetchRequest) -> FetchOutcome {
        let id = self.next_req;
        self.next_req += 1;
        if self
            .tx
            .send(FromWorker::Resource(id, Box::new(req)))
            .is_err()
        {
            self.terminated = true;
            return FetchOutcome::network_error();
        }
        self.notify();
        let started = Instant::now();
        loop {
            let left = self.resource_timeout.saturating_sub(started.elapsed());
            if left.is_zero() {
                return FetchOutcome::network_error();
            }
            match self.rx.recv_timeout(left) {
                Ok(ToWorker::Resource(rid, out)) if rid == id => {
                    self.consumed += 1;
                    return *out;
                },
                Ok(ToWorker::Terminate) => {
                    self.consumed += 1;
                    self.terminated = true;
                    return FetchOutcome::network_error();
                },
                Ok(other) => {
                    self.consumed += 1;
                    if let Some(e) = Self::envelope(other) {
                        self.inbox.push_back(e);
                    }
                },
                Err(_) => {
                    self.terminated = true;
                    return FetchOutcome::network_error();
                },
            }
        }
    }
}

/// The worker's `fetch()` seam: every request is answered by the page.
struct WorkerFetch(Rc<RefCell<WorkerLink>>);

impl FetchHandler for WorkerFetch {
    fn fetch(&self, request: FetchRequest) -> FetchOutcome {
        self.0.borrow_mut().request(request)
    }
}

fn get_request(url: &str) -> FetchRequest {
    FetchRequest {
        method: "GET".to_owned(),
        url: url.to_owned(),
        headers: Vec::new(),
        body: None,
        cache: "default".to_owned(),
        redirect: "follow".to_owned(),
        mode: "same-origin".to_owned(),
        referrer: String::new(),
        referrer_policy: String::new(),
        credentials: "same-origin".to_owned(),
        integrity: String::new(),
    }
}

fn ok_status(out: &FetchOutcome) -> bool {
    !out.network_error && (out.status == 0 || (200..300).contains(&out.status))
}

// ── the worker thread ────────────────────────────────────────────────────────

/// The worker thread's whole life. Reached as a `fn(WorkerBoot)` pointer, so the
/// spawning side needs no engine bound and no engine value crosses the boundary.
pub(crate) fn worker_main<E: ScriptEngine>(boot: WorkerBoot) {
    let WorkerBoot {
        url,
        name,
        rx,
        tx,
        wake,
        policy,
        clock,
    } = boot;
    let link = Rc::new(RefCell::new(WorkerLink {
        tx: tx.clone(),
        rx,
        wake: wake.clone(),
        inbox: VecDeque::new(),
        next_req: 1,
        consumed: 0,
        closed: false,
        terminated: false,
        resource_timeout: match policy {
            WorkerPolicy::TrustedCooperative {
                resource_timeout, ..
            } => resource_timeout,
            WorkerPolicy::RequireCancellation => Duration::from_secs(30),
        },
    }));
    let mut rt = match Runtime::<E>::new_worker(clock) {
        Ok(rt) => rt,
        Err(_) => {
            let _ = tx.send(FromWorker::Error(
                "worker runtime failed to start".to_owned(),
            ));
            let _ = tx.send(FromWorker::Closed);
            return;
        },
    };
    rt.set_worker_policy(policy);
    rt.host().borrow_mut().worker_link = Some(link.clone());
    rt.set_fetch_handler(Box::new(WorkerFetch(link.clone())));
    let _ = rt.set_base_url(&url);
    if install_worker_scope(rt.engine_mut()).is_err() {
        let _ = tx.send(FromWorker::Error(
            "worker scope failed to install".to_owned(),
        ));
        let _ = tx.send(FromWorker::Closed);
        return;
    }
    let _ = rt.eval(&format!("__wsInit({});", js_str(&name)));

    // The classic script, over the page's resource route.
    let out = link.borrow_mut().request(get_request(&url));
    if !ok_status(&out) {
        let _ = tx.send(FromWorker::Error(format!(
            "failed to load worker script '{url}'"
        )));
        let _ = tx.send(FromWorker::Closed);
        return;
    }
    let src = String::from_utf8_lossy(&out.body).into_owned();
    // Timers installed by the startup script are measured from its entry,
    // including time that synchronous script itself spends running.
    let clock_start = Instant::now();
    if let Err(error) = rt.run_timers(0, 0.0) {
        let _ = tx.send(FromWorker::Error(rt.describe_error(&error)));
        let _ = tx.send(FromWorker::Closed);
        return;
    }
    if let Err(e) = rt.eval(&src) {
        let msg = rt.describe_error(&e);
        let _ = tx.send(FromWorker::Error(msg));
    }
    run_worker_loop(&mut rt, &link, &tx, wake.as_ref(), clock_start);
    let _ = tx.send(FromWorker::Closed);
}

/// The worker's event loop: deliver messages, run microtasks and due timers on
/// elapsed host time, then wait for the next message or timer deadline.
fn run_worker_loop<E: ScriptEngine>(
    rt: &mut Runtime<E>,
    link: &Rc<RefCell<WorkerLink>>,
    tx: &WorkerSender<FromWorker>,
    wake: Option<&Arc<dyn Fn() + Send + Sync>>,
    clock_start: Instant,
) {
    let mut idle_sent = false;
    loop {
        {
            let l = link.borrow();
            if l.terminated || l.closed {
                break;
            }
        }
        // A message callback may schedule a timer. Bring the shared timer
        // queue up to the current host instant before running that callback,
        // rather than giving its new timer the previous turn's stale base.
        let turn_start_ms = clock_start.elapsed().as_secs_f64() * 1_000.0;
        if let Err(error) = rt.run_timers(0, turn_start_ms) {
            let _ = tx.send(FromWorker::Error(rt.describe_error(&error)));
            break;
        }
        let mut work = 0usize;
        if !link.borrow().inbox.is_empty() {
            idle_sent = false;
            let _ = rt.eval("__wsPump()");
            work += 1;
        }
        rt.run_microtasks();
        let elapsed_ms = clock_start.elapsed().as_secs_f64() * 1_000.0;
        match rt.run_timers(64, elapsed_ms) {
            Ok(fired) => work += fired,
            Err(error) => {
                let _ = tx.send(FromWorker::Error(rt.describe_error(&error)));
                break;
            },
        }
        work += pump(rt);
        let next_delay = match rt.next_timer_delay() {
            Ok(delay) => delay,
            Err(error) => {
                let _ = tx.send(FromWorker::Error(rt.describe_error(&error)));
                break;
            },
        };
        if work > 0 {
            idle_sent = false;
        }
        if work == 0 {
            // A future timer still keeps the worker active. The page must not
            // observe idle before that timer's callback has run.
            if next_delay.is_none() && !idle_sent {
                let _ = tx.send(FromWorker::Idle(link.borrow().consumed));
                if let Some(wake) = wake {
                    wake();
                }
                idle_sent = true;
            }
            // `next_timer_delay` is relative to the clock sample passed to
            // `run_timers`; callback work since then has already used time.
            let spent_ms = clock_start.elapsed().as_secs_f64() * 1_000.0 - elapsed_ms;
            let wait_ms = next_delay
                .filter(|delay| delay.is_finite())
                .map(|delay| delay - spent_ms)
                .unwrap_or(50.0)
                .clamp(0.0, 50.0);
            let mut l = link.borrow_mut();
            l.pump(Some(Duration::from_secs_f64(wait_ms / 1_000.0)));
        }
    }
}

/// Install the `DedicatedWorkerGlobalScope` conversion and its sinks.
fn install_worker_scope<E: ScriptEngine>(engine: &mut E) -> Result<(), E::Error> {
    engine.set_function::<WsEmit>("__ws_emit", 2)?;
    engine.set_function::<WsPortForward>("__ws_port_forward", 3)?;
    engine.set_function::<WsTake>("__ws_take", 0)?;
    engine.set_function::<WsClose>("__ws_close", 0)?;
    engine.set_function::<WsImport>("__ws_import", 1)?;
    engine.eval(WORKER_SCOPE_BOOTSTRAP)?;
    Ok(())
}

// ── the page side ────────────────────────────────────────────────────────────

/// Install `Worker` and its sinks on a page runtime.
pub(crate) fn install_worker_surface<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    engine.set_function::<WorkerCreate>("__worker_create", 2)?;
    engine.set_function::<WorkerPost>("__worker_post", 2)?;
    engine.set_function::<WorkerTerminate>("__worker_terminate", 1)?;
    engine.set_function::<WorkerTake>("__worker_take", 0)?;
    engine.set_function::<WorkerBindPort>("__worker_bind_port", 2)?;
    engine.set_function::<PortAllocId>("__portAllocId", 0)?;
    engine.set_function::<PortForward>("__port_forward_page", 3)?;
    engine.eval(WORKER_HOST_BOOTSTRAP)?;
    Ok(())
}

fn host_state<E: ScriptEngine>(cx: &E::CallCx<'_>) -> Option<Rc<dyn std::any::Any>> {
    cx.host_data()
}

/// Run `f` with the host state, or return undefined.
fn with_host<E: ScriptEngine, R>(
    cx: &mut E::CallCx<'_>,
    f: impl FnOnce(&mut HostState) -> R,
) -> Option<R> {
    let data = host_state::<E>(cx)?;
    let cell = data.downcast_ref::<RefCell<HostState>>()?;
    let mut host = cell.borrow_mut();
    Some(f(&mut host))
}

fn arg_string<E: ScriptEngine>(cx: &mut E::CallCx<'_>, i: usize) -> Result<String, E::Error> {
    let v = cx.arg(i);
    cx.value_to_string(&v)
}

struct WorkerCreate;
impl<E: ScriptEngine> NativeFn<E> for WorkerCreate {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let url = arg_string::<E>(cx, 0)?;
        let url = with_host::<E, _>(cx, |h| {
            h.document_base_url()
                .and_then(|base| url::Url::parse(&base).ok())
                .and_then(|base| base.join(&url).ok())
                .map(|url| url.to_string())
        })
        .flatten()
        .unwrap_or(url);
        let name = arg_string::<E>(cx, 1)?;
        let policy = with_host::<E, _>(cx, |h| h.worker_policy).unwrap_or_default();
        // Required cancellation admits only a configured killable process.
        // VM fuel or completed-job counts do not interrupt arbitrary callbacks.
        #[cfg(not(target_arch = "wasm32"))]
        let launcher = with_host::<E, _>(cx, |h| h.worker_process_launcher.clone()).flatten();
        #[cfg(target_arch = "wasm32")]
        if matches!(policy, WorkerPolicy::RequireCancellation) {
            return cx.make_string("-2");
        }
        #[cfg(not(target_arch = "wasm32"))]
        if matches!(policy, WorkerPolicy::RequireCancellation) && launcher.is_none() {
            return cx.make_string("-2");
        }
        #[cfg(not(target_arch = "wasm32"))]
        if matches!(policy, WorkerPolicy::RequireCancellation)
            && launcher.as_ref().is_some_and(|launcher| !launcher.valid())
        {
            return cx.make_string("-1");
        }
        #[cfg(not(target_arch = "wasm32"))]
        if matches!(policy, WorkerPolicy::RequireCancellation)
            && launcher.as_ref().is_some_and(|launcher| {
                E::native_worker_backend() != Some(launcher.backend.as_str())
            })
        {
            return cx.make_string("-2");
        }
        let spawn = with_host::<E, _>(cx, |h| h.worker_spawn).flatten();
        let wake = with_host::<E, _>(cx, |h| h.worker_wake.clone()).flatten();
        let clock = with_host::<E, _>(cx, |h| h.agent.upgrade().map(|a| a.borrow().clock.clone()))
            .flatten()
            .unwrap_or_default();
        let Some(spawn) = spawn else {
            return cx.make_string("-1");
        };
        #[cfg(not(target_arch = "wasm32"))]
        let process_mode = matches!(policy, WorkerPolicy::RequireCancellation);
        #[cfg(not(target_arch = "wasm32"))]
        let failed = Arc::new(AtomicBool::new(false));
        #[cfg(not(target_arch = "wasm32"))]
        let (tx_to, rx_to) = if process_mode {
            let limit = launcher
                .as_ref()
                .expect("required launcher")
                .max_queued_messages;
            let (tx, rx) = sync_channel(limit);
            (
                WorkerSender::Process {
                    tx,
                    failed: failed.clone(),
                    child_side: false,
                },
                rx,
            )
        } else {
            let (tx, rx) = channel();
            (WorkerSender::Thread(tx), rx)
        };
        #[cfg(target_arch = "wasm32")]
        let (tx_to, rx_to) = {
            let (tx, rx) = channel();
            (WorkerSender::Thread(tx), rx)
        };
        #[cfg(not(target_arch = "wasm32"))]
        let (tx_from, rx_from) = if process_mode {
            let limit = launcher
                .as_ref()
                .expect("required launcher")
                .max_queued_messages;
            let (tx, rx) = sync_channel(limit);
            (
                WorkerSender::Process {
                    tx,
                    failed: failed.clone(),
                    child_side: false,
                },
                rx,
            )
        } else {
            let (tx, rx) = channel();
            (WorkerSender::Thread(tx), rx)
        };
        #[cfg(target_arch = "wasm32")]
        let (tx_from, rx_from) = {
            let (tx, rx) = channel();
            (WorkerSender::Thread(tx), rx)
        };
        let sent = Arc::new(AtomicU64::new(0));
        let boot = WorkerBoot {
            url,
            name,
            rx: rx_to,
            tx: tx_from,
            wake,
            policy,
            clock,
        };
        #[cfg(not(target_arch = "wasm32"))]
        let pump_limit = launcher
            .as_ref()
            .map_or(usize::MAX, |launcher| launcher.max_queued_messages);
        #[cfg(target_arch = "wasm32")]
        let pump_limit = usize::MAX;
        #[cfg(not(target_arch = "wasm32"))]
        let (join, process) = if let Some(launcher) =
            launcher.filter(|_| matches!(policy, WorkerPolicy::RequireCancellation))
        {
            match WorkerProcessHandle::spawn(boot, &launcher, failed.clone()) {
                Ok(process) => (None, Some(process)),
                Err(_) => return cx.make_string("-1"),
            }
        } else {
            let join = std::thread::Builder::new()
                .name("genet-worker".to_owned())
                .spawn(move || spawn(boot))
                .ok();
            (join, None)
        };
        #[cfg(target_arch = "wasm32")]
        let join = std::thread::Builder::new()
            .name("genet-worker".to_owned())
            .spawn(move || spawn(boot))
            .ok();
        if join.is_none() {
            #[cfg(not(target_arch = "wasm32"))]
            if process.is_none() {
                return cx.make_string("-1");
            }
            #[cfg(target_arch = "wasm32")]
            return cx.make_string("-1");
        }
        let id = with_host::<E, _>(cx, |h| {
            h.workers.push(WorkerHandle {
                tx: tx_to,
                rx: rx_from,
                join,
                #[cfg(not(target_arch = "wasm32"))]
                process,
                #[cfg(not(target_arch = "wasm32"))]
                bridge_failed: process_mode.then_some(failed),
                idle: false,
                alive: true,
                sent,
                shutdown_grace: match policy {
                    WorkerPolicy::TrustedCooperative { shutdown_grace, .. } => shutdown_grace,
                    WorkerPolicy::RequireCancellation => Duration::ZERO,
                },
                pump_limit,
            });
            h.workers.len() - 1
        });
        match id {
            Some(id) => cx.make_string(&id.to_string()),
            None => cx.make_string("-1"),
        }
    }
}

struct WorkerPost;
impl<E: ScriptEngine> NativeFn<E> for WorkerPost {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id: usize = arg_string::<E>(cx, 0)?.parse().unwrap_or(usize::MAX);
        let payload = arg_string::<E>(cx, 1)?;
        let accepted = with_host::<E, _>(cx, |h| {
            if let Some(w) = h.workers.get_mut(id) {
                w.sent.fetch_add(1, Ordering::Release);
                if w.tx.send(ToWorker::Message(payload)).is_ok() {
                    w.idle = false;
                    return true;
                }
                #[cfg(not(target_arch = "wasm32"))]
                if w.process.is_some() {
                    // Serialization has already committed transfers. A full or
                    // broken process link is a terminal worker error, reported
                    // asynchronously, rather than a synchronous refused send.
                    w.idle = false;
                    return true;
                }
                w.sent.fetch_sub(1, Ordering::Release);
            }
            false
        })
        .unwrap_or(false);
        cx.make_string(if accepted { "1" } else { "0" })
    }
}

struct WorkerTerminate;
impl<E: ScriptEngine> NativeFn<E> for WorkerTerminate {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id: usize = arg_string::<E>(cx, 0)?.parse().unwrap_or(usize::MAX);
        with_host::<E, _>(cx, |h| {
            if let Some(w) = h.workers.get_mut(id) {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(process) = w.process.as_mut() {
                    // Process termination bypasses a full data queue.
                    w.alive = false;
                    w.idle = true;
                    let _ = process.shutdown(true);
                    return;
                }
                w.sent.fetch_add(1, Ordering::Release);
                if w.tx.send(ToWorker::Terminate).is_err() {
                    w.sent.fetch_sub(1, Ordering::Release);
                }
                w.alive = false;
                w.idle = true;
            }
        });
        Ok(cx.undefined())
    }
}

struct WorkerTake;
impl<E: ScriptEngine> NativeFn<E> for WorkerTake {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let next = with_host::<E, _>(cx, |h| h.worker_events.pop_front()).flatten();
        cx.make_string(next.as_deref().unwrap_or(""))
    }
}

struct WorkerBindPort;
impl<E: ScriptEngine> NativeFn<E> for WorkerBindPort {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let worker: usize = arg_string::<E>(cx, 0)?.parse().unwrap_or(usize::MAX);
        let port = arg_string::<E>(cx, 1)?;
        with_host::<E, _>(cx, |h| {
            if !port.is_empty() {
                h.port_routes.push((port, worker));
            }
        });
        Ok(cx.undefined())
    }
}

struct PortAllocId;
impl<E: ScriptEngine> NativeFn<E> for PortAllocId {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id = NEXT_PORT_ID.fetch_add(1, Ordering::Relaxed);
        cx.make_string(&id.to_string())
    }
}

/// `__portForward(portId, payload, newPortIdsCsv)` on the page: route to the
/// worker the port belongs to, binding any ports the record itself carries to
/// the same link.
struct PortForward;
impl<E: ScriptEngine> NativeFn<E> for PortForward {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let port = arg_string::<E>(cx, 0)?;
        let payload = arg_string::<E>(cx, 1)?;
        let minted = arg_string::<E>(cx, 2)?;
        let accepted = with_host::<E, _>(cx, |h| {
            let Some(worker) = h.port_route(&port) else {
                return false;
            };
            if let Some(w) = h.workers.get_mut(worker) {
                w.sent.fetch_add(1, Ordering::Release);
                if w.tx.send(ToWorker::PortMessage(port, payload)).is_ok() {
                    w.idle = false;
                    for id in minted.split(',').filter(|s| !s.is_empty()) {
                        h.port_routes.push((id.to_owned(), worker));
                    }
                    return true;
                }
                #[cfg(not(target_arch = "wasm32"))]
                if w.process.is_some() {
                    w.idle = false;
                    for id in minted.split(',').filter(|s| !s.is_empty()) {
                        h.port_routes.push((id.to_owned(), worker));
                    }
                    return true;
                }
                w.sent.fetch_sub(1, Ordering::Release);
            }
            false
        })
        .unwrap_or(false);
        cx.make_string(if accepted { "1" } else { "0" })
    }
}

// ── worker-side sinks ────────────────────────────────────────────────────────

fn with_link<E: ScriptEngine, R>(
    cx: &mut E::CallCx<'_>,
    f: impl FnOnce(&mut WorkerLink) -> R,
) -> Option<R> {
    let data = host_state::<E>(cx)?;
    let cell = data.downcast_ref::<RefCell<HostState>>()?;
    let link = cell.borrow().worker_link.clone()?;
    let mut link = link.borrow_mut();
    Some(f(&mut link))
}

struct WsEmit;
impl<E: ScriptEngine> NativeFn<E> for WsEmit {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let payload = arg_string::<E>(cx, 0)?;
        let accepted = with_link::<E, _>(cx, |l| {
            let accepted = l.tx.send(FromWorker::Message(payload)).is_ok();
            if accepted {
                l.notify();
            }
            accepted
        })
        .unwrap_or(false);
        cx.make_string(if accepted { "1" } else { "0" })
    }
}

struct WsPortForward;
impl<E: ScriptEngine> NativeFn<E> for WsPortForward {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let port = arg_string::<E>(cx, 0)?;
        let payload = arg_string::<E>(cx, 1)?;
        let minted = arg_string::<E>(cx, 2)?;
        // A nested worker owns this port; otherwise it belongs to the page.
        let route = with_host::<E, _>(cx, |h| {
            let Some(worker) = h.port_route(&port) else {
                return 0;
            };
            if let Some(w) = h.workers.get_mut(worker) {
                w.sent.fetch_add(1, Ordering::Release);
                if w.tx
                    .send(ToWorker::PortMessage(port.clone(), payload.clone()))
                    .is_ok()
                {
                    w.idle = false;
                    for id in minted.split(',').filter(|s| !s.is_empty()) {
                        h.port_routes.push((id.to_owned(), worker));
                    }
                    return 1;
                }
                w.sent.fetch_sub(1, Ordering::Release);
            }
            2
        })
        .unwrap_or(2);
        let accepted = if route == 0 {
            with_link::<E, _>(cx, |l| {
                let accepted = l.tx.send(FromWorker::PortMessage(port, payload)).is_ok();
                if accepted {
                    l.notify();
                }
                accepted
            })
            .unwrap_or(false)
        } else {
            route == 1
        };
        cx.make_string(if accepted { "1" } else { "0" })
    }
}

struct WsTake;
impl<E: ScriptEngine> NativeFn<E> for WsTake {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let next = with_link::<E, _>(cx, |l| l.inbox.pop_front()).flatten();
        cx.make_string(next.as_deref().unwrap_or(""))
    }
}

struct WsClose;
impl<E: ScriptEngine> NativeFn<E> for WsClose {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        with_link::<E, _>(cx, |l| l.closed = true);
        Ok(cx.undefined())
    }
}

/// `__ws_import(url)` -> `{"ok":true,"src":"…"}` — a blocking load over the
/// page's resource route, which is what `importScripts` is.
struct WsImport;
impl<E: ScriptEngine> NativeFn<E> for WsImport {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let url = arg_string::<E>(cx, 0)?;
        let out = with_link::<E, _>(cx, |l| l.request(get_request(&url)));
        let json = match out {
            Some(out) if ok_status(&out) => {
                let src = String::from_utf8_lossy(&out.body).into_owned();
                format!("{{\"ok\":true,\"src\":{}}}", js_str(&src))
            },
            _ => "{\"ok\":false}".to_owned(),
        };
        cx.make_string(&json)
    }
}

// ── driving from the page ────────────────────────────────────────────────────

/// One turn of worker service: drain the channels, answer resource requests,
/// then dispatch the queued events into JS. Returns how much work happened.
pub(crate) fn pump<E: ScriptEngine>(rt: &mut Runtime<E>) -> usize {
    pump_in_realm(rt, rt.top_realm())
}

pub(crate) fn pump_in_realm<E: ScriptEngine>(
    rt: &mut Runtime<E>,
    realm: script_engine_api::RealmId,
) -> usize {
    let host = rt.host_in_realm(realm).expect("registered worker realm");
    let mut work = 0usize;
    let mut requests: Vec<(usize, u64, Box<FetchRequest>)> = Vec::new();
    {
        let mut h = host.borrow_mut();
        for i in 0..h.workers.len() {
            #[cfg(not(target_arch = "wasm32"))]
            if h.workers[i]
                .bridge_failed
                .as_ref()
                .is_some_and(|failed| failed.swap(false, Ordering::AcqRel))
            {
                let w = &mut h.workers[i];
                w.alive = false;
                w.idle = true;
                if let Some(process) = w.process.as_mut() {
                    let _ = process.shutdown(true);
                }
                h.worker_events.push_back(format!("{{\"k\":\"e\",\"w\":{i},\"d\":\"Worker process transport failed or exceeded its configured bound.\"}}"));
            }
            let mut drained = 0;
            while drained < h.workers[i].pump_limit {
                let msg = match h.workers[i].rx.try_recv() {
                    Ok(m) => m,
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        h.workers[i].alive = false;
                        h.workers[i].idle = true;
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(process) = h.workers[i].process.as_mut() {
                            let _ = process.shutdown(true);
                        }
                        break;
                    },
                };
                drained += 1;
                match msg {
                    // Only quiesce on a report that has seen everything sent.
                    FromWorker::Idle(seen) => {
                        h.workers[i].idle = seen == h.workers[i].sent.load(Ordering::Acquire)
                    },
                    FromWorker::Closed => {
                        h.workers[i].alive = false;
                        h.workers[i].idle = true;
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(process) = h.workers[i].process.as_mut() {
                            let _ = process.shutdown(false);
                        }
                    },
                    FromWorker::Message(p) => {
                        h.workers[i].idle = false;
                        let e = format!("{{\"k\":\"m\",\"w\":{i},\"d\":{}}}", js_str(&p));
                        h.worker_events.push_back(e);
                    },
                    FromWorker::PortMessage(id, p) => {
                        h.workers[i].idle = false;
                        let e =
                            format!("{{\"k\":\"p\",\"i\":{},\"d\":{}}}", js_str(&id), js_str(&p));
                        h.worker_events.push_back(e);
                    },
                    FromWorker::Error(m) => {
                        h.workers[i].idle = false;
                        let e = format!("{{\"k\":\"e\",\"w\":{i},\"d\":{}}}", js_str(&m));
                        h.worker_events.push_back(e);
                    },
                    FromWorker::Resource(id, req) => {
                        h.workers[i].idle = false;
                        requests.push((i, id, req));
                    },
                }
            }
        }
    }
    for (i, id, req) in requests {
        let (loader, fetch, tx, sent, wake, resource_wake) = {
            let h = host.borrow();
            let Some(w) = h.workers.get(i) else { continue };
            (
                h.script_loader.clone(),
                h.fetch.clone(),
                w.tx.clone(),
                Arc::clone(&w.sent),
                h.worker_wake.clone(),
                h.worker_resource_wake.clone(),
            )
        };
        let fallback = if let Some(loader) = loader.as_ref() {
            let resource_url = req.url.clone();
            let deferred_tx = tx.clone();
            let deferred_sent = Arc::clone(&sent);
            let complete = Box::new(move |out: FetchOutcome| {
                deferred_sent.fetch_add(1, Ordering::Release);
                if deferred_tx
                    .send(ToWorker::Resource(id, Box::new(out)))
                    .is_ok()
                {
                    // Delivery owns the reserved count.
                } else {
                    deferred_sent.fetch_sub(1, Ordering::Release);
                }
                if let Some(wake) = resource_wake {
                    wake(resource_url);
                } else if let Some(wake) = wake {
                    wake();
                }
            });
            loader.start(id, *req, complete)
        } else {
            Some(*req)
        };
        if let Some(req) = fallback {
            let out = load_resource(loader, fetch, req);
            sent.fetch_add(1, Ordering::Release);
            if tx.send(ToWorker::Resource(id, Box::new(out))).is_ok() {
                // Delivery owns the reserved count.
            } else {
                sent.fetch_sub(1, Ordering::Release);
            }
        }
        work += 1;
    }
    let queued = host.borrow().worker_events.len();
    if queued > 0 {
        if realm == rt.top_realm() {
            let _ = rt.eval("__workerPump()");
        } else {
            let _ = rt.eval_in_realm(realm, "__workerPump()");
        }
        work += queued;
    }
    work
}

/// Answer a worker resource request from the page's own route.
fn load_resource(
    loader: Option<Rc<dyn ScriptResourceLoader>>,
    fetch: Option<Rc<dyn FetchHandler>>,
    req: FetchRequest,
) -> FetchOutcome {
    if let Some(loader) = loader {
        if let Some(text) = loader.load(&req.url) {
            return FetchOutcome {
                network_error: false,
                status: 200,
                status_text: "OK".to_owned(),
                response_type: "basic".to_owned(),
                url: req.url,
                redirected: false,
                headers: vec![("content-type".to_owned(), "text/javascript".to_owned())],
                body: text.into_bytes(),
            };
        }
    }
    match fetch {
        Some(f) => f.fetch_blocking(req),
        None => FetchOutcome::network_error(),
    }
}

/// Whether the page must keep driving: a worker that is not idle can still
/// produce a message, and queued events have not been dispatched yet.
pub(crate) fn has_work(host: &SharedHost) -> bool {
    let h = host.borrow();
    !h.worker_events.is_empty() || h.workers.iter().any(|w| w.alive && !w.idle)
}

/// Request termination and join completed workers. A spinning trusted worker
/// remains detached and is counted as such; there is no VM interrupt seam.
pub(crate) fn shutdown(host: &SharedHost) -> WorkerShutdownReport {
    let handles: Vec<_> = {
        let mut h = host.borrow_mut();
        h.workers
            .iter_mut()
            .map(|w| {
                w.sent.fetch_add(1, Ordering::Release);
                if w.tx.send(ToWorker::Terminate).is_ok() {
                    // Delivery owns the reserved count.
                } else {
                    w.sent.fetch_sub(1, Ordering::Release);
                }
                w.alive = false;
                #[cfg(not(target_arch = "wasm32"))]
                {
                    (
                        w.join.take().map(|join| (join, w.shutdown_grace)),
                        w.process.take(),
                    )
                }
                #[cfg(target_arch = "wasm32")]
                {
                    w.join.take().map(|join| (join, w.shutdown_grace))
                }
            })
            .collect()
    };
    // A worker spinning inside script cannot observe the cooperative signal.
    // The grace only bounds the caller's wait; it does not reclaim that thread.
    let mut report = WorkerShutdownReport {
        ..WorkerShutdownReport::default()
    };
    for handle in handles {
        #[cfg(not(target_arch = "wasm32"))]
        let (thread, process) = handle;
        #[cfg(target_arch = "wasm32")]
        let thread = handle;
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(mut process) = process {
            match process.shutdown(false) {
                ProcessExit::Graceful => report.processes_joined += 1,
                ProcessExit::Killed => report.processes_killed += 1,
                ProcessExit::Crashed => report.processes_crashed += 1,
                ProcessExit::Failed => report.processes_failed += 1,
            }
        }
        let Some((join, grace)) = thread else {
            continue;
        };
        report.descendants_untracked = true;
        let started = Instant::now();
        while !join.is_finished() && started.elapsed() < grace {
            std::thread::sleep(Duration::from_millis(2));
        }
        if join.is_finished() {
            let _ = join.join();
            report.direct_joined += 1;
        } else {
            report.direct_detached += 1;
        }
    }
    report
}

/// `Worker`, the page's handle on another agent.
const WORKER_HOST_BOOTSTRAP: &str = r#"
(function() {
  var workers = [];
  var reportWorkerException = globalThis.__reportListenerException;


  // `ErrorEvent`: what a worker's unhandled exception is delivered as.
  function ErrorEvent(type, init) {
    Event.call(this, type, init);
    init = init || {};
    this.message = init.message === undefined ? '' : String(init.message);
    this.filename = init.filename === undefined ? '' : String(init.filename);
    this.lineno = init.lineno === undefined ? 0 : (init.lineno >>> 0);
    this.colno = init.colno === undefined ? 0 : (init.colno >>> 0);
    this.error = init.error;
  }
  ErrorEvent.prototype = Object.create(Event.prototype);
  ErrorEvent.prototype.constructor = ErrorEvent;
  if (typeof Symbol !== 'undefined' && Symbol.toStringTag) {
    ErrorEvent.prototype[Symbol.toStringTag] = 'ErrorEvent';
  }
  globalThis.ErrorEvent = ErrorEvent;

  function Worker(scriptURL, options) {
    if (arguments.length === 0) throw new TypeError("Worker requires a script URL");
    EventTarget.call(this);
    options = options || {};
    // Module workers need a module loader on the worker agent; named as a
    // residual rather than silently run as a classic script.
    if (options.type !== undefined && String(options.type) !== 'classic') {
      throw new DOMException("Module workers are not supported.", "NotSupportedError");
    }
    var url = String(scriptURL);
    // The URL must parse against the document base; a bad one is a SyntaxError
    // before anything is fetched.
    if (globalThis.URL && globalThis.URL.parse && globalThis.location) {
      if (/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(url) && !globalThis.URL.parse(url)) {
        throw new DOMException("Invalid worker script URL.", "SyntaxError");
      }
    }
    this._url = url;
    this._name = (options.name === undefined) ? '' : String(options.name);
    var id = Number(__worker_create(url, this._name));
    if (id === -2) throw new DOMException("Worker cancellation is unsupported by this script engine.", "NotSupportedError");
    if (!(id >= 0)) throw new DOMException("Worker could not be started.", "NotSupportedError");
    this._id = id;
    this._terminated = false;
    workers[id] = this;
  }
  Worker.prototype = Object.create(EventTarget.prototype);
  Worker.prototype.constructor = Worker;
  Worker.prototype.postMessage = function(message, transferOrOptions) {
    if (this._terminated) return;
    var transfer = globalThis.__normalizeTransfer(transferOrOptions);
    var payload = globalThis.__scSerialize(message, transfer);
    var minted = globalThis.__portTakePending();
    if (__worker_post(String(this._id), payload) === '0') {
      throw new DOMException('Worker message queue is full or closed.', 'QuotaExceededError');
    }
    for (var i = 0; i < minted.length; i++) __worker_bind_port(String(this._id), minted[i]);
  };
  Worker.prototype.terminate = function() {
    if (this._terminated) return;
    this._terminated = true;
    __worker_terminate(String(this._id));
  };
  globalThis.__defineEventHandler(Worker.prototype, 'message', null);
  globalThis.__defineEventHandler(Worker.prototype, 'messageerror', null);
  globalThis.__defineEventHandler(Worker.prototype, 'error', null);
  if (typeof Symbol !== 'undefined' && Symbol.toStringTag) {
    Worker.prototype[Symbol.toStringTag] = 'Worker';
  }
  globalThis.Worker = Worker;

  // Routed by the host, so `MessagePort` needs no knowledge of the link. The
  // worker scope reassigns this to its own sink.
  globalThis.__portForward = function(id, payload, minted) {
    return __port_forward_page(id, payload, minted);
  };

  // Dispatch everything the host drained off the worker links. One task per
  // event, in arrival order, which is the port message queue's ordering.
  globalThis.__workerPump = function() {
    var n = 0, s;
    while ((s = __worker_take()) !== '') {
      n++;
      var e = JSON.parse(s);
      if (e.k === 'p') { globalThis.__portDeliver(e.i, e.d); continue; }
      var w = workers[e.w];
      // "Terminate a worker" empties the port message queue: nothing already
      // in flight is delivered after `terminate()`.
      if (!w || w._terminated) continue;
      if (e.k === 'm') {
        var event;
        try {
          event = globalThis.__internalMessageEvent(globalThis.__scDeserialize(e.d), { origin: '' });
        } catch (ex) {
          event = new MessageEvent('messageerror', { origin: '' });
        }
        w.dispatchEvent(event);
      } else if (e.k === 'e') {
        var err = new ErrorEvent('error', {
          cancelable: true, message: e.d, filename: w._url, lineno: 0, colno: 0
        });
        // Not handled on the Worker object: the error reaches the window, which
        // is where an unhandled worker exception is observable.
        if (w.dispatchEvent(err)) reportWorkerException({ message: e.d });
      }
    }
    return n;
  };
})();
"#;

/// The worker agent's global: `DedicatedWorkerGlobalScope`, with no document.
const WORKER_SCOPE_BOOTSTRAP: &str = r#"
(function() {
  // A worker global has no document and no window; testharness.js selects its
  // environment on exactly `'document' in global_scope`. `window` is never
  // defined in this scope (see `SELF_WORKER_BOOTSTRAP`), and `document` is a
  // plain property here precisely so it can be deleted.
  function drop(name) {
    try { delete globalThis[name]; } catch (e) {}
    if (name in globalThis) { try { globalThis[name] = undefined; } catch (e) {} }
  }
  // The window-only names, as WPT's own
  // `constructors/Worker/unexpected-self-properties.worker.js` enumerates them.
  // Dropping a name this surface never defined is a no-op.
  // The window-only names, as WPT's own
  // `constructors/Worker/unexpected-self-properties.worker.js` enumerates them.
  // Dropping a name this surface never defined is a no-op.
  var WINDOW_ONLY = ['open', 'print', 'stop', 'getComputedStyle', 'getSelection',
    'releaseEvents', 'captureEvents', 'alert', 'confirm', 'prompt', 'back', 'forward',
    'navigate', 'DOMParser', 'XMLSerializer', 'XPathEvaluator', 'XSLTProcessor',
    'Image', 'Option', 'frames', 'Audio', 'closed', 'defaultStatus', 'document',
    'event', 'frameElement', 'history', 'innerHeight', 'innerWidth', 'opener',
    'outerHeight', 'outerWidth', 'pageXOffset', 'pageYOffset', 'parent', 'screen',
    'screenLeft', 'screenTop', 'screenX', 'screenY', 'status', 'top', 'window',
    'length', 'matchMedia', 'localStorage', 'scrollX', 'scrollY', 'scrollTo',
    'scrollBy', 'requestAnimationFrame', 'cancelAnimationFrame'];
  for (var i = 0; i < WINDOW_ONLY.length; i++) drop(WINDOW_ONLY[i]);


  function WorkerGlobalScope() {}
  function DedicatedWorkerGlobalScope() {}
  DedicatedWorkerGlobalScope.prototype = Object.create(WorkerGlobalScope.prototype);
  // The engine owns the global's real prototype, so `globalThis instanceof
  // DedicatedWorkerGlobalScope` is answered by the constructor instead.
  function answersForSelf(C) {
    try {
      Object.defineProperty(C, Symbol.hasInstance, {
        configurable: true,
        value: function(v) { return v === globalThis; }
      });
    } catch (e) {}
  }
  answersForSelf(WorkerGlobalScope);
  answersForSelf(DedicatedWorkerGlobalScope);
  globalThis.WorkerGlobalScope = WorkerGlobalScope;
  globalThis.DedicatedWorkerGlobalScope = DedicatedWorkerGlobalScope;

  function WorkerNavigator() {}
  globalThis.WorkerNavigator = WorkerNavigator;
  function WorkerLocation() {}
  globalThis.WorkerLocation = WorkerLocation;

  globalThis.__wsInit = function(name) {
    globalThis.name = String(name);
  };

  // Synchronous classic-script import over the page's resource route.
  globalThis.importScripts = function() {
    for (var i = 0; i < arguments.length; i++) {
      var url = String(arguments[i]);
      var r;
      try { r = JSON.parse(__ws_import(url)); } catch (e) { r = { ok: false }; }
      if (!r.ok) throw new DOMException("Failed to import '" + url + "'.", "NetworkError");
      // Indirect eval: the imported script's top level is this global's.
      (0, eval)(r.src);
    }
  };

  globalThis.postMessage = function(message, transferOrOptions) {
    var transfer = globalThis.__normalizeTransfer(transferOrOptions);
    var payload = globalThis.__scSerialize(message, transfer);
    var minted = globalThis.__portTakePending();
    if (__ws_emit(payload, minted.join(',')) === '0') {
      throw new DOMException('Worker message queue is full or closed.', 'QuotaExceededError');
    }
  };
  globalThis.__portForward = function(id, payload, minted) {
    return __ws_port_forward(id, payload, minted);
  };
  globalThis.close = function() { __ws_close(); };

  globalThis.__wsPump = function() {
    var n = 0, s;
    while ((s = __ws_take()) !== '') {
      n++;
      var e = JSON.parse(s);
      if (e.k === 'p') { globalThis.__portDeliver(e.i, e.d); continue; }
      if (e.k === 'm') {
        var event;
        try {
          event = globalThis.__internalMessageEvent(globalThis.__scDeserialize(e.d), { origin: '' });
        } catch (ex) {
          event = new MessageEvent('messageerror', { origin: '' });
        }
        globalThis.dispatchEvent(event);
      }
    }
    return n;
  };
})();
"#;

#[cfg(test)]
mod shutdown_tests {
    use super::*;
    use std::sync::{Condvar, Mutex};

    struct FiniteThreadCleanup {
        gate: Arc<(Mutex<bool>, Condvar)>,
        finished: Option<Receiver<()>>,
    }

    impl FiniteThreadCleanup {
        fn release(&mut self) {
            let (lock, wake) = &*self.gate;
            *lock.lock().unwrap() = true;
            wake.notify_all();
            if let Some(finished) = self.finished.take() {
                finished.recv_timeout(Duration::from_secs(2)).unwrap();
            }
        }
    }

    impl Drop for FiniteThreadCleanup {
        fn drop(&mut self) {
            // Also releases the finite helper if shutdown or an assertion panics.
            if self.finished.is_some() {
                self.release();
            }
        }
    }

    #[test]
    fn finite_blocked_trusted_thread_is_reported_detached() {
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let worker_gate = Arc::clone(&gate);
        let (started_tx, started_rx) = channel();
        let (finished_tx, finished_rx) = channel();
        let mut cleanup = FiniteThreadCleanup {
            gate,
            finished: Some(finished_rx),
        };
        let join = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            let (lock, wake) = &*worker_gate;
            let mut released = lock.lock().unwrap();
            while !*released {
                released = wake.wait(released).unwrap();
            }
            finished_tx.send(()).unwrap();
        });
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let (tx, _rx) = channel();
        let (_reply_tx, reply_rx) = channel();
        let host = Rc::new(RefCell::new(HostState::default()));
        host.borrow_mut().workers.push(WorkerHandle {
            tx: WorkerSender::Thread(tx),
            rx: reply_rx,
            join: Some(join),
            #[cfg(not(target_arch = "wasm32"))]
            process: None,
            #[cfg(not(target_arch = "wasm32"))]
            bridge_failed: None,
            idle: false,
            alive: true,
            sent: Arc::new(AtomicU64::new(0)),
            shutdown_grace: Duration::ZERO,
            pump_limit: usize::MAX,
        });
        let report = shutdown(&host);
        cleanup.release();
        assert_eq!(
            report,
            WorkerShutdownReport {
                direct_joined: 0,
                direct_detached: 1,
                descendants_untracked: true,
                ..WorkerShutdownReport::default()
            }
        );
    }

    #[test]
    fn nested_shutdown_report_counts_only_direct_threads() {
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let inner_gate = Arc::clone(&gate);
        let (inner_finished_tx, inner_finished_rx) = channel();
        let mut cleanup = FiniteThreadCleanup {
            gate,
            finished: Some(inner_finished_rx),
        };
        let (outer_tx, outer_rx) = channel();
        let (ready_tx, ready_rx) = channel();
        let (inner_report_tx, inner_report_rx) = channel();
        let outer = std::thread::spawn(move || {
            let (nested_tx, nested_rx) = channel();
            let (_reply_tx, reply_rx) = channel();
            let inner = std::thread::spawn(move || {
                let _keep_channel_open = nested_rx;
                let (lock, wake) = &*inner_gate;
                let mut released = lock.lock().unwrap();
                while !*released {
                    released = wake.wait(released).unwrap();
                }
                inner_finished_tx.send(()).unwrap();
            });
            let nested = Rc::new(RefCell::new(HostState::default()));
            nested.borrow_mut().workers.push(WorkerHandle {
                tx: WorkerSender::Thread(nested_tx),
                rx: reply_rx,
                join: Some(inner),
                #[cfg(not(target_arch = "wasm32"))]
                process: None,
                #[cfg(not(target_arch = "wasm32"))]
                bridge_failed: None,
                idle: false,
                alive: true,
                sent: Arc::new(AtomicU64::new(0)),
                shutdown_grace: Duration::ZERO,
                pump_limit: usize::MAX,
            });
            ready_tx.send(()).unwrap();
            let _ = outer_rx.recv_timeout(Duration::from_secs(2));
            inner_report_tx.send(shutdown(&nested)).unwrap();
        });
        ready_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let (_reply_tx, reply_rx) = channel();
        let parent = Rc::new(RefCell::new(HostState::default()));
        parent.borrow_mut().workers.push(WorkerHandle {
            tx: WorkerSender::Thread(outer_tx),
            rx: reply_rx,
            join: Some(outer),
            #[cfg(not(target_arch = "wasm32"))]
            process: None,
            #[cfg(not(target_arch = "wasm32"))]
            bridge_failed: None,
            idle: false,
            alive: true,
            sent: Arc::new(AtomicU64::new(0)),
            shutdown_grace: Duration::from_secs(2),
            pump_limit: usize::MAX,
        });
        let direct = shutdown(&parent);
        let nested = inner_report_rx
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        cleanup.release();
        assert_eq!(direct.direct_joined, 1);
        assert_eq!(direct.direct_detached, 0);
        assert!(direct.descendants_untracked);
        assert_eq!(nested.direct_detached, 1);
    }
}
