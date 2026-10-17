/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The hosted scripted resource bridge.
//!
//! A Livery scripted document has one host resource route. This adapter keeps
//! that route intact for parser/CSS resources, the JS `fetch()` surface and
//! dedicated workers. Page fetches and hosted worker resources are started
//! off-thread. Their completion wakes the document's host for its next pump.

use std::collections::{BTreeSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use script_engine_api::ScriptEngine;
use script_runtime_api::{FetchHandler, FetchOutcome, FetchRequest, Runtime, ScriptResourceLoader};

use crate::ResourceFetcher;

#[derive(Clone, Default)]
pub struct ScriptWake {
    state: Arc<Mutex<WakeState>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptWakeEvent {
    Wake {
        source: &'static str,
        generation: u64,
        resource: Option<String>,
    },
    Stale {
        source: &'static str,
        generation: u64,
        active_generation: u64,
        resource: Option<String>,
    },
}

struct WakeState {
    callback: Option<Arc<dyn Fn(ScriptWakeEvent) + Send + Sync>>,
    pending: Option<ScriptWakeEvent>,
    live_generations: BTreeSet<u64>,
}

impl Default for WakeState {
    fn default() -> Self {
        Self {
            callback: None,
            pending: None,
            live_generations: BTreeSet::new(),
        }
    }
}

impl ScriptWake {
    pub fn new() -> Self {
        Self::default()
    }

    /// Install the host's wake sink. If a completion arrived before the
    /// window existed, report it immediately after the sink is attached.
    pub fn set_callback(&self, callback: impl Fn(ScriptWakeEvent) + Send + Sync + 'static) {
        let callback: Arc<dyn Fn(ScriptWakeEvent) + Send + Sync> = Arc::new(callback);
        let should_wake = {
            let mut state = self.state.lock().expect("script wake mutex poisoned");
            state.callback = Some(Arc::clone(&callback));
            state.pending.clone()
        };
        if let Some(event) = should_wake {
            callback(event);
        }
    }

    fn wake(&self, source: &'static str, generation: u64) {
        self.wake_with_resource(source, generation, None);
    }

    fn wake_with_resource(&self, source: &'static str, generation: u64, resource: Option<String>) {
        let callback = {
            let mut state = self.state.lock().expect("script wake mutex poisoned");
            state.pending = Some(ScriptWakeEvent::Wake {
                source,
                generation,
                resource: resource.clone(),
            });
            state.callback.clone()
        };
        if let Some(callback) = callback {
            callback(ScriptWakeEvent::Wake {
                source,
                generation,
                resource,
            });
        }
    }

    pub fn notify_worker_for(&self, generation: u64) {
        let active = self.active_generation();
        if active != generation {
            self.stale("worker", generation, active, None);
            return;
        }
        self.wake("worker", generation);
    }

    pub fn notify_worker_resource(&self, generation: u64, resource: String) {
        self.notify_source("worker", generation, Some(resource));
    }

    fn notify_source(&self, source: &'static str, generation: u64, resource: Option<String>) {
        let active = self.active_generation();
        if active == generation {
            self.wake_with_resource(source, generation, resource);
        } else {
            self.stale(source, generation, active, resource);
        }
    }

    fn stale(
        &self,
        source: &'static str,
        generation: u64,
        active_generation: u64,
        resource: Option<String>,
    ) {
        let callback = self
            .state
            .lock()
            .expect("script wake mutex poisoned")
            .callback
            .clone();
        if let Some(callback) = callback {
            callback(ScriptWakeEvent::Stale {
                source,
                generation,
                active_generation,
                resource,
            });
        }
    }

    fn register_generation(&self, generation: u64) {
        let mut state = self.state.lock().expect("script wake mutex poisoned");
        state.live_generations.insert(generation);
    }

    fn unregister_generation(&self, generation: u64) {
        let dispatch = {
            let mut state = self.state.lock().expect("script wake mutex poisoned");
            let previous = state
                .live_generations
                .iter()
                .next_back()
                .copied()
                .unwrap_or(0);
            state.live_generations.remove(&generation);
            let active = state
                .live_generations
                .iter()
                .next_back()
                .copied()
                .unwrap_or(0);
            if previous == generation && active != 0 {
                let event = ScriptWakeEvent::Wake {
                    source: "generation",
                    generation: active,
                    resource: None,
                };
                state.pending = Some(event.clone());
                state.callback.clone().map(|callback| (callback, event))
            } else {
                None
            }
        };
        if let Some((callback, event)) = dispatch {
            callback(event);
        }
    }

    /// Consume the coalesced notification. This is useful to hosts that use a
    /// channel instead of a direct window callback.
    pub fn take_pending(&self) -> bool {
        let mut state = self.state.lock().expect("script wake mutex poisoned");
        state.pending.take().is_some()
    }

    pub fn active_generation(&self) -> u64 {
        self.state
            .lock()
            .expect("script wake mutex poisoned")
            .live_generations
            .iter()
            .next_back()
            .copied()
            .unwrap_or(0)
    }
}

struct Completion {
    id: u64,
    outcome: FetchOutcome,
    _reservation: FetchReservation,
}

/// Held until a page completion is consumed or discarded. Worker requests
/// release after delivery to the worker's mailbox.
struct FetchReservation(Arc<AtomicUsize>);

impl Drop for FetchReservation {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

struct CompletionState {
    closed: bool,
    queue: VecDeque<Completion>,
}

struct BridgeState {
    fetcher: Arc<dyn ResourceFetcher + Send + Sync>,
    completions: Mutex<CompletionState>,
    active: AtomicUsize,
    admitted: Arc<AtomicUsize>,
    admission_limit: usize,
    closed: AtomicBool,
    wake: ScriptWake,
    generation: u64,
}

impl BridgeState {
    fn try_reserve(&self) -> Option<FetchReservation> {
        if self.closed.load(Ordering::Acquire) {
            return None;
        }
        let mut current = self.admitted.load(Ordering::Acquire);
        loop {
            if current >= self.admission_limit {
                return None;
            }
            match self.admitted.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    let reservation = FetchReservation(Arc::clone(&self.admitted));
                    if self.closed.load(Ordering::Acquire) {
                        return None;
                    }
                    return Some(reservation);
                },
                Err(actual) => current = actual,
            }
        }
    }
}

/// A cloneable resource route installed into both the runtime and Livery.
///
/// The route owns no transport policy. It only calls the `ResourceFetcher`
/// supplied by the host, preserving that fetcher's final URL and content type
/// when turning a response into a JS `Response`.
#[derive(Clone)]
pub struct ScriptResourceBridge {
    state: Arc<BridgeState>,
}

impl ScriptResourceBridge {
    pub fn new<F>(fetcher: F, wake: ScriptWake) -> Self
    where
        F: ResourceFetcher + Send + Sync + 'static,
    {
        Self::new_with_generation(fetcher, wake, 0)
    }

    pub fn new_with_generation<F>(fetcher: F, wake: ScriptWake, generation: u64) -> Self
    where
        F: ResourceFetcher + Send + Sync + 'static,
    {
        let admission_limit = fetcher.max_concurrent_fetches();
        wake.register_generation(generation);
        Self {
            state: Arc::new(BridgeState {
                fetcher: Arc::new(fetcher),
                completions: Mutex::new(CompletionState {
                    closed: false,
                    queue: VecDeque::new(),
                }),
                active: AtomicUsize::new(0),
                admitted: Arc::new(AtomicUsize::new(0)),
                admission_limit,
                closed: AtomicBool::new(false),
                wake,
                generation,
            }),
        }
    }

    pub fn wake(&self) -> ScriptWake {
        self.state.wake.clone()
    }

    pub fn generation(&self) -> u64 {
        self.state.generation
    }

    pub fn has_pending(&self) -> bool {
        self.state.active.load(Ordering::Acquire) != 0
            || !self
                .state
                .completions
                .lock()
                .expect("script resource mutex poisoned")
                .queue
                .is_empty()
    }

    /// Drop queued results and reject any future delivery. The worker runtime
    /// is shut down by `Runtime`'s drop; this closes the resource half first so
    /// a late transport completion cannot enter a replacement session.
    pub fn close(&self) {
        if self.state.closed.swap(true, Ordering::AcqRel) {
            return;
        }
        // Retire the generation before waiting for the completion lock. A
        // transport thread that currently owns the lock will then report a
        // stale wake instead of waking the session being closed.
        self.state.wake.unregister_generation(self.state.generation);
        let mut completions = self
            .state
            .completions
            .lock()
            .expect("script resource mutex poisoned");
        completions.closed = true;
        completions.queue.clear();
    }

    pub fn pump<E: ScriptEngine>(&self, runtime: &mut Runtime<E>) -> Result<usize, String> {
        let completions = {
            let mut queue = self
                .state
                .completions
                .lock()
                .expect("script resource mutex poisoned");
            queue.queue.drain(..).collect::<Vec<_>>()
        };
        if self.state.closed.load(Ordering::Acquire) {
            return Ok(0);
        }
        let count = completions.len();
        let mut failures = Vec::new();
        for completion in completions {
            if let Err(error) = runtime.settle_fetch(completion.id, completion.outcome) {
                failures.push(format!("fetch {}: {error}", completion.id));
            }
        }
        if failures.is_empty() {
            Ok(count)
        } else {
            Err(failures.join("; "))
        }
    }

    fn outcome(&self, request: &FetchRequest) -> FetchOutcome {
        match self.state.fetcher.fetch_request(request) {
            Some(response) => {
                let mut headers = response.headers;
                if !headers
                    .iter()
                    .any(|(name, _)| name.eq_ignore_ascii_case("content-type"))
                {
                    if let Some(content_type) = response.content_type {
                        headers.push(("content-type".into(), content_type));
                    }
                }
                FetchOutcome {
                    network_error: false,
                    status: response.status,
                    status_text: response.status_text,
                    response_type: response.response_type,
                    redirected: response.redirected || response.final_url != request.url,
                    url: response.final_url,
                    headers,
                    body: response.bytes,
                }
            },
            None => FetchOutcome::network_error(),
        }
    }

    fn start_fetch_with(
        &self,
        id: u64,
        request: FetchRequest,
        spawn: impl FnOnce(Box<dyn FnOnce() + Send + 'static>) -> std::io::Result<()>,
    ) -> Option<FetchOutcome> {
        let Some(reservation) = self.state.try_reserve() else {
            return Some(FetchOutcome::network_error());
        };
        self.state.active.fetch_add(1, Ordering::AcqRel);
        let state = Arc::clone(&self.state);
        let work = Box::new(move || {
            let outcome = Self {
                state: Arc::clone(&state),
            }
            .outcome(&request);
            let resource = request.url.clone();
            let mut completions = state
                .completions
                .lock()
                .expect("script resource mutex poisoned");
            if completions.closed || state.closed.load(Ordering::Acquire) {
                drop(completions);
                state.active.fetch_sub(1, Ordering::AcqRel);
                state
                    .wake
                    .notify_source("fetch", state.generation, Some(resource));
                return;
            }
            completions.queue.push_back(Completion {
                id,
                outcome,
                _reservation: reservation,
            });
            state.active.fetch_sub(1, Ordering::AcqRel);
            drop(completions);
            state
                .wake
                .notify_source("fetch", state.generation, Some(resource));
        });
        if spawn(work).is_err() {
            self.state.active.fetch_sub(1, Ordering::AcqRel);
            return Some(FetchOutcome::network_error());
        }
        None
    }
}

impl Drop for ScriptResourceBridge {
    fn drop(&mut self) {
        if Arc::strong_count(&self.state) == 1 {
            self.close();
        }
    }
}

impl ResourceFetcher for ScriptResourceBridge {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.state.fetcher.fetch(url)
    }

    fn max_concurrent_fetches(&self) -> usize {
        self.state.admission_limit
    }

    fn fetch_request(
        &self,
        request: &genet_host_api::ResourceRequest,
    ) -> Option<genet_host_api::ResourceFetchResponse> {
        self.state.fetcher.fetch_request(request)
    }

    fn fetch_response(&self, url: &str) -> Option<genet_host_api::ResourceResponse> {
        self.state.fetcher.fetch_response(url)
    }
}

impl FetchHandler for ScriptResourceBridge {
    fn start(&self, id: u64, request: FetchRequest) -> Option<FetchOutcome> {
        self.start_fetch_with(id, request, |work| {
            std::thread::Builder::new()
                .name("genet-script-fetch".to_owned())
                .spawn(move || work())
                .map(|_| ())
        })
    }

    fn fetch(&self, request: FetchRequest) -> FetchOutcome {
        self.outcome(&request)
    }
}

impl ScriptResourceLoader for ScriptResourceBridge {
    fn load(&self, url: &str) -> Option<String> {
        let response = self.state.fetcher.fetch_response(url)?;
        String::from_utf8(response.bytes).ok()
    }

    fn load_document(&self, url: &str) -> Option<FetchOutcome> {
        // This is the existing URL-only document loader's policy boundary.
        // Rich/nondefault navigation requests never enter load_document.
        // Metadata retrieval uses a host plain GET (cors/same-origin), retaining
        // legacy loader compatibility rather than claiming navigate/include.
        let request = FetchRequest {
            url: url.into(),
            method: "GET".into(),
            headers: Vec::new(),
            body: None,
            cache: "default".into(),
            redirect: "follow".into(),
            mode: "cors".into(),
            referrer: String::new(),
            referrer_policy: String::new(),
            credentials: "same-origin".into(),
            integrity: String::new(),
        };
        Some(if self.state.closed.load(Ordering::Acquire) {
            FetchOutcome::network_error()
        } else {
            self.outcome(&request)
        })
    }

    fn load_classic_script(
        &self,
        url: &str,
        charset: Option<&str>,
        integrity: Option<&str>,
    ) -> Option<String> {
        if self.state.closed.load(Ordering::Acquire) {
            return None;
        }
        crate::document::fetch_external(Some((self, url)), url, charset, integrity)
    }

    fn start(
        &self,
        _id: u64,
        request: FetchRequest,
        complete: Box<dyn FnOnce(FetchOutcome) + Send>,
    ) -> Option<FetchRequest> {
        let Some(reservation) = self.state.try_reserve() else {
            complete(FetchOutcome::network_error());
            return None;
        };
        self.state.active.fetch_add(1, Ordering::AcqRel);
        let state = Arc::clone(&self.state);
        let completion = Arc::new(Mutex::new(Some(complete)));
        let worker_completion = Arc::clone(&completion);
        let spawned = std::thread::Builder::new()
            .name("genet-worker-resource".to_owned())
            .spawn(move || {
                let _reservation = reservation;
                let resource = request.url.clone();
                let outcome = Self {
                    state: Arc::clone(&state),
                }
                .outcome(&request);
                state.active.fetch_sub(1, Ordering::AcqRel);
                if state.closed.load(Ordering::Acquire) {
                    let active = state.wake.active_generation();
                    state
                        .wake
                        .stale("worker", state.generation, active, Some(resource));
                    return;
                }
                if let Some(complete) = worker_completion
                    .lock()
                    .expect("worker completion mutex poisoned")
                    .take()
                {
                    complete(outcome);
                }
            });
        if spawned.is_err() {
            self.state.active.fetch_sub(1, Ordering::AcqRel);
            if let Some(complete) = completion
                .lock()
                .expect("worker completion mutex poisoned")
                .take()
            {
                complete(FetchOutcome::network_error());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genet_host_api::{ResourceFetchResponse, ResourceResponse};
    use std::sync::atomic::AtomicUsize;
    use std::sync::{Condvar, mpsc};

    fn known_http_fixture(response: ResourceResponse) -> ResourceFetchResponse {
        let headers = response
            .content_type
            .as_ref()
            .map(|value| vec![("content-type".into(), value.clone())])
            .unwrap_or_default();
        ResourceFetchResponse {
            final_url: response.final_url,
            content_type: response.content_type,
            bytes: response.bytes,
            status: 200,
            status_text: "OK".into(),
            headers,
            response_type: "basic".into(),
            redirected: false,
        }
    }

    #[derive(Clone)]
    struct BlockingHost {
        calls: Arc<AtomicUsize>,
        entered: mpsc::Sender<()>,
        gate: Arc<(Mutex<bool>, Condvar)>,
    }

    impl ResourceFetcher for BlockingHost {
        fn fetch(&self, _: &str) -> Option<Vec<u8>> {
            None
        }

        fn max_concurrent_fetches(&self) -> usize {
            1
        }

        fn fetch_request(&self, request: &FetchRequest) -> Option<ResourceFetchResponse> {
            self.calls.fetch_add(1, Ordering::AcqRel);
            self.entered.send(()).unwrap();
            let (lock, changed) = &*self.gate;
            let released = lock.lock().unwrap();
            let (released, _) = changed
                .wait_timeout_while(released, std::time::Duration::from_secs(3), |open| !*open)
                .unwrap();
            if !*released {
                return None;
            }
            Some(known_http_fixture(ResourceResponse::new(
                request.url.clone(),
                b"ok".to_vec(),
            )))
        }
    }

    fn blocking_host() -> (BlockingHost, mpsc::Receiver<()>) {
        let (entered, receiver) = mpsc::channel();
        (
            BlockingHost {
                calls: Arc::new(AtomicUsize::new(0)),
                entered,
                gate: Arc::new((Mutex::new(false), Condvar::new())),
            },
            receiver,
        )
    }

    fn release(host: &BlockingHost) {
        let (lock, changed) = &*host.gate;
        *lock.lock().unwrap() = true;
        changed.notify_all();
    }

    #[test]
    fn page_admission_bounds_workers_and_queued_completions_until_pump() {
        let (host, entered) = blocking_host();
        let (woke_tx, woke_rx) = mpsc::channel();
        let wake = ScriptWake::new();
        wake.set_callback(move |_| {
            let _ = woke_tx.send(());
        });
        let bridge = ScriptResourceBridge::new(host.clone(), wake);
        assert!(FetchHandler::start(&bridge, 1, FetchRequest::plain_get("https://fixture.test/one")).is_none());
        entered.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert!(FetchHandler::start(&bridge, 2, FetchRequest::plain_get("https://fixture.test/two"))
            .unwrap().network_error);
        assert_eq!(host.calls.load(Ordering::Acquire), 1);
        release(&host);
        woke_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        assert_eq!(bridge.state.admitted.load(Ordering::Acquire), 1);
        assert!(FetchHandler::start(&bridge, 3, FetchRequest::plain_get("https://fixture.test/three"))
            .unwrap().network_error, "unconsumed completion still holds admission");
        let mut runtime = Runtime::<script_engine_boa::BoaEngine>::new().unwrap();
        // These direct starts use fixture IDs without registering JS promises;
        // the JS settlement hook deliberately ignores unknown IDs.
        assert_eq!(bridge.pump(&mut runtime).unwrap(), 1);
        assert_eq!(bridge.state.admitted.load(Ordering::Acquire), 0);
        assert!(FetchHandler::start(&bridge, 4, FetchRequest::plain_get("https://fixture.test/four")).is_none());
        entered.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        woke_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        bridge.close();
        assert_eq!(host.calls.load(Ordering::Acquire), 2);
        assert_eq!(bridge.state.admitted.load(Ordering::Acquire), 0);
    }

    #[test]
    fn worker_admission_refuses_inline_without_falling_back_to_sync_fetch() {
        let (host, entered) = blocking_host();
        let bridge = ScriptResourceBridge::new(host.clone(), ScriptWake::new());
        let (completed_tx, completed_rx) = mpsc::channel();
        assert!(ScriptResourceLoader::start(
            &bridge,
            1,
            FetchRequest::plain_get("https://fixture.test/worker-one"),
            Box::new(move |outcome| completed_tx.send(outcome).unwrap()),
        ).is_none());
        entered.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        let (refused_tx, refused_rx) = mpsc::channel();
        assert!(ScriptResourceLoader::start(
            &bridge,
            2,
            FetchRequest::plain_get("https://fixture.test/worker-two"),
            Box::new(move |outcome| refused_tx.send(outcome).unwrap()),
        ).is_none());
        assert!(refused_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap().network_error);
        assert_eq!(host.calls.load(Ordering::Acquire), 1);
        release(&host);
        assert!(!completed_rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap().network_error);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while bridge.state.admitted.load(Ordering::Acquire) != 0
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(bridge.state.admitted.load(Ordering::Acquire), 0);
    }

    #[test]
    fn page_spawn_failure_releases_reservation_without_panicking() {
        let bridge = ScriptResourceBridge::new(
            Fixture { calls: Arc::new(AtomicUsize::new(0)) },
            ScriptWake::new(),
        );
        let outcome = bridge.start_fetch_with(
            1,
            FetchRequest::plain_get("https://fixture.test/fail"),
            |_work| Err(std::io::Error::other("injected thread refusal")),
        );
        assert!(outcome.unwrap().network_error);
        assert_eq!(bridge.state.admitted.load(Ordering::Acquire), 0);
        assert!(!bridge.has_pending());
    }

    #[derive(Clone)]
    struct Fixture {
        calls: Arc<AtomicUsize>,
    }

    impl ResourceFetcher for Fixture {
        fn fetch(&self, url: &str) -> Option<Vec<u8>> {
            self.fetch_response(url).map(|r| r.bytes)
        }

        fn fetch_response(&self, _url: &str) -> Option<ResourceResponse> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Some(
                ResourceResponse::new("https://final.test/script.js", b"x".to_vec())
                    .with_content_type("text/javascript"),
            )
        }

        fn fetch_request(&self, request: &FetchRequest) -> Option<ResourceFetchResponse> {
            request
                .is_plain_get()
                .then(|| self.fetch_response(&request.url))
                .flatten()
                .map(known_http_fixture)
        }
    }

    fn product_bridge_redirect_identity<E: ScriptEngine>() {
        #[derive(Clone)]
        struct Redirecting(Arc<Mutex<Vec<String>>>);
        impl ResourceFetcher for Redirecting {
            fn fetch(&self, url: &str) -> Option<Vec<u8>> {
                self.fetch_response(url).map(|r| r.bytes)
            }
            fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
                self.0.lock().unwrap().push(url.into());
                let final_url = if url == "https://origin.test/requested.html" {
                    "https://final.test/final.html"
                } else {
                    url
                };
                Some(
                    ResourceResponse::new(final_url, b"<p id=mark>redirected</p>".to_vec())
                        .with_content_type("text/html"),
                )
            }
            fn fetch_request(&self, request: &FetchRequest) -> Option<ResourceFetchResponse> {
                request
                    .is_plain_get()
                    .then(|| self.fetch_response(&request.url))
                    .flatten()
                    .map(known_http_fixture)
            }
        }
        fn read<E: ScriptEngine>(runtime: &mut Runtime<E>, code: &str) -> String {
            let value = runtime.eval(code).unwrap();
            runtime.value_to_string(&value).unwrap()
        }
        let requests = Arc::new(Mutex::new(Vec::new()));
        let bridge = ScriptResourceBridge::new(Redirecting(requests.clone()), ScriptWake::new());
        let mut runtime = Runtime::<E>::new().unwrap();
        runtime
            .set_base_url("https://origin.test/start.html")
            .unwrap();
        // Product document installation uses this same bridge on both seams.
        runtime.set_fetch_handler(Box::new(bridge.clone()));
        runtime.set_script_resource_loader(Box::new(bridge));
        runtime.parse_document_interleaved("<p>start</p>", &script_runtime_api::NoScriptLoader);
        runtime.run_event_loop(100).unwrap();
        let window = runtime.eval("window").unwrap();
        runtime
            .engine_mut()
            .set_global_in_realm(script_engine_api::MAIN_REALM, "held", &window)
            .unwrap();
        runtime
            .eval("location.href='https://origin.test/requested.html'")
            .unwrap();
        runtime.run_event_loop(100).unwrap();
        assert_eq!(
            read(&mut runtime, "location.href"),
            "https://final.test/final.html"
        );
        assert_eq!(
            read(&mut runtime, "document.URL"),
            "https://final.test/final.html"
        );
        assert_eq!(
            read(&mut runtime, "document.documentURI"),
            "https://final.test/final.html"
        );
        assert_eq!(read(&mut runtime, "location.origin"), "https://final.test");
        assert_eq!(
            read(&mut runtime, "document.getElementById('mark').textContent"),
            "redirected"
        );
        let window = runtime.eval("window").unwrap();
        runtime
            .engine_mut()
            .set_global_in_realm(script_engine_api::MAIN_REALM, "current", &window)
            .unwrap();
        let value = runtime.engine_mut().eval("String(held===current)").unwrap();
        assert_eq!(runtime.value_to_string(&value).unwrap(), "true");
        assert_eq!(
            &*requests.lock().unwrap(),
            &["https://origin.test/requested.html"]
        );
        runtime.eval("history.back()").unwrap();
        runtime.run_event_loop(100).unwrap();
        assert_eq!(
            read(&mut runtime, "location.href"),
            "https://origin.test/start.html"
        );
        runtime.eval("history.forward()").unwrap();
        runtime.run_event_loop(100).unwrap();
        assert_eq!(
            read(&mut runtime, "document.URL"),
            "https://final.test/final.html"
        );
        assert_eq!(
            read(&mut runtime, "location.href"),
            "https://final.test/final.html"
        );
        assert_eq!(
            &*requests.lock().unwrap(),
            &[
                "https://origin.test/requested.html",
                "https://origin.test/start.html",
                "https://final.test/final.html"
            ]
        );
    }
    #[test]
    fn boa_product_bridge_redirect_identity() {
        product_bridge_redirect_identity::<script_engine_boa::BoaEngine>();
    }
    #[cfg(all(target_pointer_width = "64", feature = "scripted-nova"))]
    #[test]
    fn nova_product_bridge_redirect_identity() {
        product_bridge_redirect_identity::<script_engine_nova::NovaEngine>();
    }

    fn product_bridge_metadata_retains_document<E: ScriptEngine>() {
        #[derive(Clone)]
        struct Metadata {
            status: u16,
            calls: Arc<Mutex<Vec<genet_host_api::ResourceRequest>>>,
        }
        impl ResourceFetcher for Metadata {
            fn fetch(&self, _: &str) -> Option<Vec<u8>> {
                panic!("metadata load downgraded to source-only loader")
            }
            fn fetch_request(
                &self,
                request: &genet_host_api::ResourceRequest,
            ) -> Option<genet_host_api::ResourceFetchResponse> {
                self.calls.lock().unwrap().push(request.clone());
                if self.status == 0 {
                    return None;
                }
                let mut response = genet_host_api::ResourceFetchResponse {
                    final_url: "https://final.test/final.html".into(),
                    content_type: None,
                    bytes: b"<p>replacement</p>".to_vec(),
                    status: 200,
                    status_text: "OK".into(),
                    headers: Vec::new(),
                    response_type: "basic".into(),
                    redirected: false,
                };
                response.status = self.status;
                if self.status == 1 {
                    response.response_type = "error".into();
                }
                Some(response)
            }
        }
        struct Unrelated;
        impl FetchHandler for Unrelated {
            fn fetch(&self, _: FetchRequest) -> FetchOutcome {
                panic!("document loader failure fell through to unrelated fetch handler")
            }
        }
        for status in [204, 205, 0, 1] {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let bridge = ScriptResourceBridge::new(
                Metadata {
                    status,
                    calls: calls.clone(),
                },
                ScriptWake::new(),
            );
            let mut runtime = Runtime::<E>::new().unwrap();
            runtime
                .set_base_url("https://origin.test/start.html")
                .unwrap();
            runtime.set_fetch_handler(Box::new(Unrelated));
            runtime.set_script_resource_loader(Box::new(bridge));
            runtime.parse_document_interleaved("<p id=original>original</p><script>var lifecycle=[];addEventListener('pagehide',function(){lifecycle.push('pagehide')});addEventListener('unload',function(){lifecycle.push('unload')})</script>", &script_runtime_api::NoScriptLoader);
            runtime.run_event_loop(100).unwrap();
            let outgoing = runtime.top_realm();
            runtime
                .eval("location.href='https://origin.test/requested.html'")
                .unwrap();
            runtime.run_event_loop(100).unwrap();
            assert_eq!(
                runtime.top_realm(),
                outgoing,
                "status {status} retains document"
            );
            for (expression, expected) in [
                ("location.href", "https://origin.test/start.html"),
                (
                    "document.getElementById('original').textContent",
                    "original",
                ),
                ("lifecycle.join(',')", ""),
                ("history.length", "1"),
            ] {
                let value = runtime.eval(expression).unwrap();
                assert_eq!(runtime.value_to_string(&value).unwrap(), expected);
            }
            let calls = calls.lock().unwrap();
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].url, "https://origin.test/requested.html");
            assert_eq!(calls[0].method, "GET");
            assert_eq!(calls[0].mode, "cors");
            assert_eq!(calls[0].credentials, "same-origin");
            assert!(calls[0].referrer.is_empty());
            assert!(calls[0].referrer_policy.is_empty());
            assert!(calls[0].body.is_none());
            assert!(calls[0].headers.is_empty());
        }
    }
    #[test]
    fn boa_product_bridge_metadata_retains_document() {
        product_bridge_metadata_retains_document::<script_engine_boa::BoaEngine>();
    }
    #[cfg(all(target_pointer_width = "64", feature = "scripted-nova"))]
    #[test]
    fn nova_product_bridge_metadata_retains_document() {
        product_bridge_metadata_retains_document::<script_engine_nova::NovaEngine>();
    }

    fn fixture_request(method: &str) -> FetchRequest {
        FetchRequest {
            url: "https://origin.test/script.js".into(),
            method: method.into(),
            headers: vec![],
            body: None,
            cache: "default".into(),
            redirect: "follow".into(),
            mode: "cors".into(),
            referrer: String::new(),
            referrer_policy: String::new(),
            credentials: "same-origin".into(),
            integrity: String::new(),
        }
    }

    #[test]
    fn rich_request_preserves_bytes_policies_and_response_metadata() {
        #[derive(Clone)]
        struct Recording(Arc<Mutex<Vec<genet_host_api::ResourceRequest>>>);
        impl ResourceFetcher for Recording {
            fn fetch(&self, _: &str) -> Option<Vec<u8>> {
                panic!("rich request downgraded")
            }
            fn fetch_request(
                &self,
                request: &genet_host_api::ResourceRequest,
            ) -> Option<genet_host_api::ResourceFetchResponse> {
                self.0.lock().unwrap().push(request.clone());
                let mut response = genet_host_api::ResourceFetchResponse {
                    final_url: "https://final.test/posted".into(),
                    content_type: None,
                    bytes: b"accepted".to_vec(),
                    status: 200,
                    status_text: "OK".into(),
                    headers: Vec::new(),
                    response_type: "basic".into(),
                    redirected: false,
                };
                response.status = 201;
                response.status_text = "Created".into();
                response.response_type = "cors".into();
                response.redirected = true;
                response.headers = vec![("x-result".into(), "created".into())];
                Some(response)
            }
        }
        let requests = Arc::new(Mutex::new(Vec::new()));
        let bridge = ScriptResourceBridge::new(Recording(requests.clone()), ScriptWake::new());
        let mut request = fixture_request("POST");
        request.body = Some(vec![0, 128, 255]);
        request.headers = vec![("Content-Type".into(), "application/octet-stream".into())];
        request.cache = "no-store".into();
        request.redirect = "manual".into();
        request.mode = "navigate".into();
        request.referrer = "https://origin.test/form".into();
        request.referrer_policy = "origin".into();
        request.credentials = "include".into();
        request.integrity = "sha256-example".into();
        let outcome = FetchHandler::fetch(&bridge, request);
        let requests = requests.lock().unwrap();
        let request = &requests[0];
        assert_eq!(requests.len(), 1);
        assert_eq!(request.method, "POST");
        assert_eq!(request.url, "https://origin.test/script.js");
        assert_eq!(request.body.as_deref(), Some(&[0, 128, 255][..]));
        assert_eq!(
            request.headers,
            [("Content-Type".into(), "application/octet-stream".into())]
        );
        assert_eq!(request.cache, "no-store");
        assert_eq!(request.redirect, "manual");
        assert_eq!(request.mode, "navigate");
        assert_eq!(request.referrer, "https://origin.test/form");
        assert_eq!(request.referrer_policy, "origin");
        assert_eq!(request.credentials, "include");
        assert_eq!(request.integrity, "sha256-example");
        assert_eq!(outcome.status, 201);
        assert_eq!(outcome.status_text, "Created");
        assert_eq!(outcome.response_type, "cors");
        assert!(outcome.redirected);
        assert_eq!(outcome.url, "https://final.test/posted");
        assert_eq!(outcome.headers, [("x-result".into(), "created".into())]);
        assert_eq!(outcome.body, b"accepted");
    }

    #[test]
    fn url_only_host_refuses_rich_requests_without_claiming_http_success() {
        struct UrlOnly(Arc<AtomicUsize>);
        impl ResourceFetcher for UrlOnly {
            fn fetch(&self, _: &str) -> Option<Vec<u8>> {
                self.0.fetch_add(1, Ordering::Relaxed);
                Some(b"legacy".to_vec())
            }
        }
        let calls = Arc::new(AtomicUsize::new(0));
        let bridge = ScriptResourceBridge::new(UrlOnly(calls.clone()), ScriptWake::new());
        let outcome = FetchHandler::fetch(
            &bridge,
            FetchRequest {
                url: "https://origin.test/submit".into(),
                method: "POST".into(),
                headers: vec![("content-type".into(), "application/octet-stream".into())],
                body: Some(vec![0, 128, 255]),
                cache: "default".into(),
                redirect: "follow".into(),
                mode: "navigate".into(),
                referrer: "https://origin.test/form".into(),
                referrer_policy: "origin".into(),
                credentials: "include".into(),
                integrity: String::new(),
            },
        );
        assert!(
            outcome.network_error,
            "URL-only host cannot serve a POST as GET"
        );
        assert_eq!(calls.load(Ordering::Relaxed), 0);
        assert!(
            FetchHandler::fetch(&bridge, fixture_request("GET")).network_error,
            "URL-only bytes do not establish HTTP status or response policy"
        );
        assert_eq!(calls.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn async_mapping_preserves_final_url_and_content_type() {
        let bridge = ScriptResourceBridge::new(
            Fixture {
                calls: Arc::new(AtomicUsize::new(0)),
            },
            ScriptWake::new(),
        );
        let outcome = FetchHandler::fetch(&bridge, fixture_request("GET"));
        assert_eq!(outcome.url, "https://final.test/script.js");
        assert!(outcome.redirected);
        assert_eq!(
            outcome.headers,
            [("content-type".to_owned(), "text/javascript".to_owned())]
        );
        assert_eq!(outcome.body, b"x");
    }

    #[test]
    fn classic_script_loading_checks_integrity_and_stops_after_close() {
        let calls = Arc::new(AtomicUsize::new(0));
        let bridge = ScriptResourceBridge::new(
            Fixture {
                calls: Arc::clone(&calls),
            },
            ScriptWake::new(),
        );
        let url = "https://origin.test/script.js";
        assert_eq!(
            bridge.load_classic_script(
                url,
                None,
                Some("sha256-LXEWQrcmsEQBYnyp+6wy9chTD7GQPMTbAiWHF5IaSIE="),
            ),
            Some("x".to_owned())
        );
        assert_eq!(
            bridge.load_classic_script(
                url,
                None,
                Some("sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="),
            ),
            None
        );
        assert_eq!(calls.load(Ordering::Relaxed), 2);
        bridge.close();
        assert_eq!(bridge.load_classic_script(url, None, None), None);
        assert_eq!(calls.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn classic_script_loading_decodes_the_declared_charset() {
        struct Latin1Script;
        impl ResourceFetcher for Latin1Script {
            fn fetch(&self, _url: &str) -> Option<Vec<u8>> {
                Some(vec![0xe9])
            }
        }
        let bridge = ScriptResourceBridge::new(Latin1Script, ScriptWake::new());
        assert_eq!(
            bridge
                .load_classic_script("https://origin.test/script.js", Some("windows-1252"), None,),
            Some("é".to_owned())
        );
    }

    #[test]
    fn wake_callback_is_called_for_deferred_completion() {
        let wake = ScriptWake::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = Arc::clone(&calls);
        wake.set_callback(move |_| {
            observed.fetch_add(1, Ordering::Relaxed);
        });
        wake.wake("fetch", 0);
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        assert!(wake.take_pending());
        assert!(!wake.take_pending());
    }

    #[test]
    fn close_discards_queued_completions() {
        let bridge = ScriptResourceBridge::new(
            Fixture {
                calls: Arc::new(AtomicUsize::new(0)),
            },
            ScriptWake::new(),
        );
        bridge
            .state
            .completions
            .lock()
            .unwrap()
            .queue
            .push_back(Completion {
                id: 1,
                outcome: FetchOutcome::network_error(),
                _reservation: bridge.state.try_reserve().unwrap(),
            });
        assert!(bridge.has_pending());
        bridge.close();
        assert!(!bridge.has_pending());
        assert_eq!(bridge.state.admitted.load(Ordering::Acquire), 0);
    }

    #[test]
    fn failed_replacement_restores_the_previous_generation() {
        let wake = ScriptWake::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        let observed = Arc::clone(&events);
        wake.set_callback(move |event| observed.lock().unwrap().push(event));
        let first = ScriptResourceBridge::new_with_generation(
            Fixture {
                calls: Arc::new(AtomicUsize::new(0)),
            },
            wake.clone(),
            1,
        );
        let replacement = ScriptResourceBridge::new_with_generation(
            Fixture {
                calls: Arc::new(AtomicUsize::new(0)),
            },
            wake.clone(),
            2,
        );
        wake.notify_source("fetch", 1, Some("https://old.test/data".to_owned()));
        replacement.close();
        assert_eq!(wake.active_generation(), 1);
        assert!(matches!(
            events.lock().unwrap().last(),
            Some(ScriptWakeEvent::Wake {
                source: "generation",
                generation: 1,
                resource: None,
            })
        ));
        drop(first);
    }

    #[test]
    fn overlapping_failed_spawns_leave_the_latest_live_generation_active() {
        let wake = ScriptWake::new();
        let fixture = || Fixture {
            calls: Arc::new(AtomicUsize::new(0)),
        };
        let first = ScriptResourceBridge::new_with_generation(fixture(), wake.clone(), 1);
        let second = ScriptResourceBridge::new_with_generation(fixture(), wake.clone(), 2);
        let third = ScriptResourceBridge::new_with_generation(fixture(), wake.clone(), 3);
        second.close();
        assert_eq!(wake.active_generation(), 3);
        third.close();
        assert_eq!(wake.active_generation(), 1);
        drop(first);
    }
}
