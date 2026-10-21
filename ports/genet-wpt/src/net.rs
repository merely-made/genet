// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;

use script_runtime_api::{
    FetchHandler, FetchOutcome, FetchRequest, WebSocketHandler, WebSocketRequest,
};

use crate::harness::ScriptSrcLoader;

/// One persistent worker thread owns the ONLY Tokio runtime that touches
/// netfetcher, so netfetcher's process-wide hyper client pool binds to a runtime
/// that is always being driven. Both blocking resource GETs (`Job::Get`) and
/// deferred `fetch()` calls (`Job::Fetch`) route through it. A current-thread
/// runtime + `spawn_blocking` job intake keeps the runtime thread free to drive
/// in-flight fetches; only plain owned data crosses the channel, so the engine
/// stays `!Send`.
fn worker_jobs() -> std::sync::mpsc::Sender<Job> {
    static WORKER: OnceLock<std::sync::Mutex<std::sync::mpsc::Sender<Job>>> = OnceLock::new();
    WORKER
        .get_or_init(|| {
            let (tx, rx) = std::sync::mpsc::channel::<Job>();
            std::thread::spawn(move || worker_loop(rx));
            std::sync::Mutex::new(tx)
        })
        .lock()
        .expect("worker job sender")
        .clone()
}

fn worker_loop(rx: std::sync::mpsc::Receiver<Job>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("worker tokio runtime");
    rt.block_on(async move {
        let mut handles: std::collections::HashMap<u64, tokio::task::AbortHandle> =
            std::collections::HashMap::new();
        // Per-fetch pull credit: a chunk is streamed only when the JS body
        // ReadableStream demands one (Job::Pull). Keyed by JS id (the routing key
        // the reply events already use).
        let mut pulls: std::collections::HashMap<u64, tokio::sync::mpsc::UnboundedSender<()>> =
            std::collections::HashMap::new();
        // Live WebSocket connections, keyed by the JS socket id: the channel a
        // send / close command travels down to the task that owns the socket.
        let mut sockets: std::collections::HashMap<u64, tokio::sync::mpsc::UnboundedSender<WsCmd>> =
            std::collections::HashMap::new();
        let mut rx = Some(rx);
        loop {
            // Await the next job on the blocking pool, so the runtime thread stays
            // free to drive in-flight fetch tasks meanwhile.
            let owned = rx.take().unwrap();
            let (owned, job) = tokio::task::spawn_blocking(move || {
                let j = owned.recv();
                (owned, j)
            })
            .await
            .expect("worker recv join");
            rx = Some(owned);
            match job {
                Ok(Job::Get(url, reply)) => {
                    tokio::spawn(async move {
                        let _ = reply.send(do_get(&url).await);
                    });
                },
                Ok(Job::GetDocument(url, options, reply)) => {
                    tokio::spawn(async move {
                        let _ = reply.send(do_get_document(&url, options).await);
                    });
                },
                Ok(Job::Fetch(key, id, req, reply)) => {
                    let (pull_tx, pull_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
                    pulls.insert(id, pull_tx);
                    let h =
                        tokio::spawn(run_fetch_streaming(id, req, reply, pull_rx)).abort_handle();
                    handles.insert(key, h);
                },
                Ok(Job::Pull(id)) => {
                    // Grant one chunk of credit; a dead receiver (task finished)
                    // means the entry is stale, so drop it.
                    if let Some(tx) = pulls.get(&id) {
                        if tx.send(()).is_err() {
                            pulls.remove(&id);
                        }
                    }
                },
                Ok(Job::WsConnect(id, req, reply)) => {
                    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<WsCmd>();
                    sockets.insert(id, tx);
                    tokio::spawn(run_websocket(id, req, reply, rx));
                },
                Ok(Job::WsCommand(id, cmd)) => {
                    // A dead receiver means the socket task has finished, so the
                    // entry is stale; the runtime already has its close event.
                    if let Some(tx) = sockets.get(&id) {
                        if tx.send(cmd).is_err() {
                            sockets.remove(&id);
                        }
                    }
                },
                Ok(Job::Cancel(key)) => {
                    if let Some(h) = handles.remove(&key) {
                        h.abort(); // drop the in-flight future
                    }
                },
                Err(_) => break, // all senders dropped: shut down
            }
            handles.retain(|_, h| !h.is_finished());
        }
    });
}

/// One process-wide HTTP cache, shared across every deferred fetch so the
/// request cache modes (default / force-cache / only-if-cached / ...) have a
/// persistent store to act against. WPT cache tests key on a per-subtest uuid,
/// so a global cache does not cross subtests.
fn shared_cache() -> std::sync::Arc<netfetcher::InMemoryHttpCache> {
    static CACHE: std::sync::OnceLock<std::sync::Arc<netfetcher::InMemoryHttpCache>> =
        std::sync::OnceLock::new();
    CACHE
        .get_or_init(|| std::sync::Arc::new(netfetcher::InMemoryHttpCache::new()))
        .clone()
}

/// One process-wide cookie jar, shared across every deferred fetch so a
/// `Set-Cookie` from one request is attached to the next (credentials tests set
/// a cookie, then verify the following request carries it).
fn shared_cookies() -> std::sync::Arc<netfetcher::InMemoryCookieJar> {
    static JAR: std::sync::OnceLock<std::sync::Arc<netfetcher::InMemoryCookieJar>> =
        std::sync::OnceLock::new();
    JAR.get_or_init(|| std::sync::Arc::new(netfetcher::InMemoryCookieJar::default()))
        .clone()
}

/// A `CookieStore` view over the shared jar (`FetchContext.cookies` is a `Box`,
/// so each context wraps a cheap clone of the shared `Arc`).
struct SharedJar(std::sync::Arc<netfetcher::InMemoryCookieJar>);
impl netfetcher::CookieStore for SharedJar {
    fn cookies_for(&self, url: &url::Url, ctx: netfetcher::SameSiteContext) -> Vec<String> {
        self.0.cookies_for(url, ctx)
    }
    fn set_cookie(&self, url: &url::Url, header: &str) {
        self.0.set_cookie(url, header)
    }
}

/// A fetch context with the shared HTTP cache + cookie jar wired in (the
/// `fetch()` path).
fn fetch_context() -> netfetcher::FetchContext {
    let mut cx = netfetcher::FetchContext::permissive();
    cx.cache = shared_cache();
    cx.cookies = Box::new(SharedJar(shared_cookies()));
    cx
}

/// The document (page) origin every fetch is initiated from — the WPT server
/// origin. Drives cross-origin detection (CORS / response tainting): a request
/// whose target origin differs is cross-origin. Set once when server mode is
/// established; `None` in disk mode (every fetch treated as same-origin).
static PAGE_ORIGIN: std::sync::RwLock<Option<url::Origin>> = std::sync::RwLock::new(None);

/// Record the page origin (the last call wins). A test runs on the origin that
/// serves it, which is the https origin for a `.https.` file, so each test sets
/// it before its document loads.
pub fn set_page_origin(origin_str: &str) {
    if let Ok(u) = url::Url::parse(origin_str) {
        *PAGE_ORIGIN.write().expect("page origin") = Some(u.origin());
    }
}

fn page_origin() -> Option<url::Origin> {
    PAGE_ORIGIN.read().expect("page origin").clone()
}

/// A globally-unique abort key (the JS `id` is per-test, so it cannot key the
/// shared worker's abort map).
/// Ceiling on one blocking (synchronous XHR) fetch. It holds the engine thread,
/// so it must not outlive the per-test drive deadline by much.
const SYNC_FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

fn next_key() -> u64 {
    static KEY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    KEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Blocking HTTP GET on the worker runtime, body as a (UTF-8-lossy) string.
/// `None` on parse / network error or non-2xx. Used for `<script src>` and
/// readiness probes; the caller blocks on the reply.
pub fn http_get(url: &str) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    worker_jobs().send(Job::Get(url.to_owned(), tx)).ok()?;
    rx.recv().ok().flatten()
}

#[derive(Clone, Debug)]
pub struct LoadedHtmlDocument {
    pub document: genet_scripted_dom::html_encoding::DecodedHtmlDocument,
    pub final_url: String,
    /// The final response's headers, as received.
    pub headers: Vec<(String, String)>,
}
pub fn http_get_document(
    url: &str,
    options: script_runtime_api::HtmlDecodeOptions,
) -> Option<LoadedHtmlDocument> {
    let (tx, rx) = std::sync::mpsc::channel();
    worker_jobs()
        .send(Job::GetDocument(url.to_owned(), options, tx))
        .ok()?;
    rx.recv().ok().flatten()
}
async fn do_get_document(
    url: &str,
    options: script_runtime_api::HtmlDecodeOptions,
) -> Option<LoadedHtmlDocument> {
    let req = netfetcher::Request::get(url::Url::parse(url).ok()?);
    let cx = netfetcher::FetchContext::permissive();
    let response = netfetcher::fetch(req, &cx).await;
    if response.is_network_error()
        || response.status < 200
        || response.status >= 300
        || matches!(response.status, 204 | 205)
    {
        return None;
    }
    let final_url = response
        .url_list
        .last()
        .map(ToString::to_string)
        .unwrap_or_else(|| url.to_owned());
    let content_type = response
        .headers
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.clone());
    let headers = response.headers.clone();
    let bytes = response.bytes().await.ok()?;
    Some(LoadedHtmlDocument {
        headers,
        document: genet_scripted_dom::html_encoding::decode_html_document(
            &bytes,
            content_type.as_deref(),
            options,
        ),
        final_url,
    })
}
async fn do_get(url: &str) -> Option<String> {
    let u = url::Url::parse(url).ok()?;
    let req = netfetcher::Request::get(u);
    let cx = netfetcher::FetchContext::permissive();
    let resp = netfetcher::fetch(req, &cx).await;
    if resp.is_network_error() || resp.status < 200 || resp.status >= 300 {
        return None;
    }
    resp.bytes()
        .await
        .ok()
        .map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// The canonical HTTP reason phrase for a status code (netfetcher discards the
/// wire reason). WPT checks `response.statusText`, so synthesize it.
fn reason_phrase(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a Teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Entity",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        _ => "",
    }
}

fn map_response_type(t: netfetcher::ResponseType) -> String {
    match t {
        netfetcher::ResponseType::Basic => "basic",
        netfetcher::ResponseType::Cors => "cors",
        netfetcher::ResponseType::Opaque => "opaque",
        netfetcher::ResponseType::OpaqueRedirect => "opaqueredirect",
        netfetcher::ResponseType::Error => "error",
    }
    .to_owned()
}

/// Run a deferred fetch and report it to the test's channel: a network error is
/// `Fail`; otherwise `StartStream` once the headers are in (so `await fetch()`
/// resolves before the body finishes, which is what lets a mid-flight abort run),
/// then a `Chunk` per body chunk as it decodes, then `Close` (or `Error` if a
/// chunk fails to decode, which errors the already-resolved response's body).
/// Dropping this task (Job::Cancel) drops the in-flight body future, cancelling
/// the request.
async fn run_fetch_streaming(
    id: u64,
    req: FetchRequest,
    reply: std::sync::mpsc::Sender<FetchEvent>,
    mut pull_rx: tokio::sync::mpsc::UnboundedReceiver<()>,
) {
    let Ok(url) = url::Url::parse(&req.url) else {
        let _ = reply.send(FetchEvent::Fail(id, "Failed to fetch".to_string()));
        return;
    };
    let mut request = netfetcher::Request::get(url);
    request.method = match req.method.as_str() {
        "GET" => netfetcher::Method::Get,
        "HEAD" => netfetcher::Method::Head,
        "POST" => netfetcher::Method::Post,
        "PUT" => netfetcher::Method::Put,
        "DELETE" => netfetcher::Method::Delete,
        "PATCH" => netfetcher::Method::Patch,
        "OPTIONS" => netfetcher::Method::Options,
        // A custom method token (e.g. "patcH", "REPORT") — kept verbatim so it
        // is treated as non-simple (preflighted) and sent as-is.
        other => netfetcher::Method::Other(other.to_string()),
    };
    request.headers = req.headers;
    request.body = req.body.map(bytes::Bytes::from);
    request.cache = match req.cache.as_str() {
        "no-store" => netfetcher::CacheMode::NoStore,
        "reload" => netfetcher::CacheMode::Reload,
        "no-cache" => netfetcher::CacheMode::NoCache,
        "force-cache" => netfetcher::CacheMode::ForceCache,
        "only-if-cached" => netfetcher::CacheMode::OnlyIfCached,
        _ => netfetcher::CacheMode::Default,
    };
    request.redirect = match req.redirect.as_str() {
        "error" => netfetcher::RedirectMode::Error,
        "manual" => netfetcher::RedirectMode::Manual,
        _ => netfetcher::RedirectMode::Follow,
    };
    request.mode = match req.mode.as_str() {
        "no-cors" => netfetcher::RequestMode::NoCors,
        "same-origin" => netfetcher::RequestMode::SameOrigin,
        "navigate" => netfetcher::RequestMode::Navigate,
        _ => netfetcher::RequestMode::Cors,
    };
    // The initiator origin (the WPT page) drives cross-origin detection. In disk
    // mode it stays None (every fetch is same-origin).
    request.origin = page_origin();
    // Referrer + policy drive the `Referer` header (empty referrer = none).
    request.referrer = (!req.referrer.is_empty())
        .then(|| url::Url::parse(&req.referrer).ok())
        .flatten();
    request.referrer_policy = match req.referrer_policy.as_str() {
        "no-referrer" => netfetcher::ReferrerPolicy::NoReferrer,
        "no-referrer-when-downgrade" => netfetcher::ReferrerPolicy::NoReferrerWhenDowngrade,
        "same-origin" => netfetcher::ReferrerPolicy::SameOrigin,
        "origin" => netfetcher::ReferrerPolicy::Origin,
        "strict-origin" => netfetcher::ReferrerPolicy::StrictOrigin,
        "origin-when-cross-origin" => netfetcher::ReferrerPolicy::OriginWhenCrossOrigin,
        "strict-origin-when-cross-origin" => {
            netfetcher::ReferrerPolicy::StrictOriginWhenCrossOrigin
        },
        "unsafe-url" => netfetcher::ReferrerPolicy::UnsafeUrl,
        _ => netfetcher::ReferrerPolicy::Empty,
    };
    request.credentials = match req.credentials.as_str() {
        "omit" => netfetcher::Credentials::Omit,
        "include" => netfetcher::Credentials::Include,
        _ => netfetcher::Credentials::SameOrigin,
    };
    request.integrity = req.integrity.clone();
    request.upgrade_insecure_requests = req.upgrade_insecure_requests;
    request.block_all_mixed_content = req.block_all_mixed_content;

    let cx = fetch_context();
    let mut resp = netfetcher::fetch(request, &cx).await;
    if resp.is_network_error() {
        let _ = reply.send(FetchEvent::Fail(id, "Failed to fetch".to_string()));
        return;
    }
    let meta = FetchOutcome {
        network_error: false,
        status: resp.status,
        status_text: reason_phrase(resp.status).to_string(),
        response_type: map_response_type(resp.response_type),
        url: resp
            .url_list
            .last()
            .map(|u| u.to_string())
            .unwrap_or_default(),
        redirected: resp.url_list.len() > 1,
        headers: resp.headers.clone(),
        body: vec![],
    };
    if reply.send(FetchEvent::StartStream(id, meta)).is_err() {
        return;
    }
    // Pull-driven body: stream one chunk per credit from the JS ReadableStream.
    // A body the script never reads sends no credit, so it is never fetched (no
    // streaming a 300 MB response nobody consumes); the task idles here until the
    // test ends and Job::Cancel aborts it.
    while pull_rx.recv().await.is_some() {
        match resp.body.next_chunk().await {
            Some(Ok(bytes)) => {
                if reply.send(FetchEvent::Chunk(id, bytes.to_vec())).is_err() {
                    return; // the test's channel is gone (run ended)
                }
            },
            Some(Err(_)) => {
                // Body decode error (e.g. a bad Content-Encoding): error the
                // body stream so reads reject, rather than closing it cleanly.
                let _ = reply.send(FetchEvent::Error(id));
                return;
            },
            None => {
                let _ = reply.send(FetchEvent::Close(id));
                return;
            },
        }
    }
}

/// Run one WebSocket connection on the worker runtime, reporting every stage to
/// the test's channel: `WsOpen` once the handshake succeeds (carrying the
/// selected subprotocol and negotiated extensions), `WsText` / `WsBinary` per
/// incoming frame, `WsFlushed` once a frame this socket was asked to send has
/// left the queue, and `WsClose` / `WsError` at the end.
///
/// The browser policy this path needs — scheme rules, blocked ports, HSTS,
/// mixed-content blocking, redirect rejection, cookies and `Origin` — is
/// netfetcher's, not this file's: `connect` enforces it against the same shared
/// jar and `FetchContext` the fetch path uses, and every refusal arrives here as
/// a typed `WsError` that becomes the one failure detail the spec lets script
/// see (`error`, then `close` 1006).
async fn run_websocket(
    id: u64,
    req: WebSocketRequest,
    reply: std::sync::mpsc::Sender<FetchEvent>,
    mut cmds: tokio::sync::mpsc::UnboundedReceiver<WsCmd>,
) {
    let Ok(url) = url::Url::parse(&req.url) else {
        let _ = reply.send(FetchEvent::WsError(id));
        return;
    };
    let mut request = netfetcher::WsRequest::new(url).with_protocols(req.protocols);
    // The initiating document's origin travels as `Origin`, and its scheme is
    // what mixed-content blocking keys on.
    if let Some(origin) = page_origin().or_else(|| {
        url::Url::parse(&req.origin)
            .ok()
            .map(|u| u.origin())
            .filter(|o| o.is_tuple())
    }) {
        request.secure_context = origin.ascii_serialization().starts_with("https://");
        request = request.with_origin(origin);
    }

    let cx = fetch_context();
    let mut socket = match netfetcher::connect_websocket(request, &cx).await {
        Ok(s) => s,
        Err(e) => {
            // The specification lets script see only "an error occurred", so the
            // typed reason stops here.
            let _ = e;
            let _ = reply.send(FetchEvent::WsError(id));
            return;
        },
    };
    if reply
        .send(FetchEvent::WsOpen(
            id,
            socket.protocol().to_owned(),
            socket.extensions().to_owned(),
        ))
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            cmd = cmds.recv() => match cmd {
                Some(WsCmd::Text(text)) => {
                    let n = text.len() as u64;
                    if socket.send(netfetcher::WsMessage::Text(text)).await.is_err() {
                        let _ = reply.send(FetchEvent::WsError(id));
                        return;
                    }
                    if reply.send(FetchEvent::WsFlushed(id, n)).is_err() { return; }
                },
                Some(WsCmd::Binary(data)) => {
                    let n = data.len() as u64;
                    if socket.send(netfetcher::WsMessage::Binary(data)).await.is_err() {
                        let _ = reply.send(FetchEvent::WsError(id));
                        return;
                    }
                    if reply.send(FetchEvent::WsFlushed(id, n)).is_err() { return; }
                },
                Some(WsCmd::Close(code, reason)) => {
                    // Start the closing handshake and keep reading: the peer's
                    // close frame carries the code and reason the `close` event
                    // reports, so the socket is not torn down here.
                    if socket.close(code.map(|c| (c, reason))).await.is_err() {
                        let _ = reply.send(FetchEvent::WsClose(
                            id,
                            netfetcher::WsClose::ABNORMAL,
                            String::new(),
                            false,
                        ));
                        return;
                    }
                },
                // The handler dropped (the test ended): stop owning the socket.
                None => return,
            },
            incoming = socket.recv() => match incoming {
                Ok(Some(netfetcher::WsIncoming::Message(m))) => {
                    let event = match m {
                        netfetcher::WsMessage::Text(t) => FetchEvent::WsText(id, t),
                        netfetcher::WsMessage::Binary(b) => FetchEvent::WsBinary(id, b),
                        // Ping / Pong are transport bookkeeping; tungstenite
                        // answers a ping itself and neither is script-visible.
                        _ => continue,
                    };
                    if reply.send(event).is_err() { return; }
                },
                Ok(Some(netfetcher::WsIncoming::Close(c))) => {
                    let _ = reply.send(FetchEvent::WsClose(id, c.code, c.reason, c.clean));
                    return;
                },
                // End of stream with no closing handshake, or a transport error:
                // both are an abnormal closure, which script sees as error+close.
                Ok(None) | Err(_) => {
                    let _ = reply.send(FetchEvent::WsError(id));
                    return;
                },
            },
        }
    }
}

/// A job for the persistent worker. `Get` is a blocking resource GET (reply: the
/// body or `None`); `Fetch` is a deferred `fetch()` (reply: a `FetchEvent` to the
/// test's channel); `Cancel` aborts an in-flight fetch by its global key.
pub enum Job {
    Get(String, std::sync::mpsc::Sender<Option<String>>),
    GetDocument(
        String,
        script_runtime_api::HtmlDecodeOptions,
        std::sync::mpsc::Sender<Option<LoadedHtmlDocument>>,
    ),
    Fetch(u64, u64, FetchRequest, std::sync::mpsc::Sender<FetchEvent>),
    /// Demand the next body chunk for a streaming fetch, by its JS id.
    Pull(u64),
    Cancel(u64),
    /// Open a WebSocket for JS socket `id` (reply: a `FetchEvent::Ws*`).
    WsConnect(u64, WebSocketRequest, std::sync::mpsc::Sender<FetchEvent>),
    /// Send or close on an open WebSocket, by its JS socket id.
    WsCommand(u64, WsCmd),
}

/// What script asked an open socket to do. Routed to the task that owns it, so
/// the socket is never touched from two places.
pub enum WsCmd {
    Text(String),
    Binary(Vec<u8>),
    Close(Option<u16>, String),
}

/// A deferred fetch event, routed to the originating test's channel by the JS
/// `id` (not the global abort key). A response streams as `StartStream` (status +
/// headers) -> `Chunk`* (body, as it arrives) -> `Close`, or `Error` if the body
/// fails partway (e.g. a `Content-Encoding` decode error: the response already
/// resolved, so its body stream errors and body reads reject). A network error
/// before the headers is `Fail` (the `fetch()` Promise rejects as a `TypeError`).
pub enum FetchEvent {
    StartStream(u64, FetchOutcome),
    Chunk(u64, Vec<u8>),
    Close(u64),
    Error(u64),
    Fail(u64, String),
    /// WebSocket, routed by the JS socket id. The two id spaces are separate but
    /// never confused: the runtime routes by variant, not by id.
    WsOpen(u64, String, String),
    WsText(u64, String),
    WsBinary(u64, Vec<u8>),
    /// `n` bytes left the send queue (`bufferedAmount` drops by that much).
    WsFlushed(u64, u64),
    WsError(u64),
    WsClose(u64, u16, String, bool),
}

/// The deferred host `fetch()` seam: `start` hands the request to the shared
/// worker (tagged with a global key for cancellation + the JS id for routing) and
/// leaves the JS Promise pending; `cancel` relays an abort. The reply settles
/// later via the drive loop. This is the actor-mailbox shape: the handler owns a
/// send into the worker's inbox plus the test's reply channel. Per-test (a fresh
/// reply channel + key map), so a late reply from a prior test cannot cross over.
pub struct NetFetchHandler {
    reply: std::sync::mpsc::Sender<FetchEvent>,
    keys: std::cell::RefCell<std::collections::HashMap<u64, u64>>, // js id -> global key
}

impl NetFetchHandler {
    pub fn new(reply: std::sync::mpsc::Sender<FetchEvent>) -> Self {
        Self {
            reply,
            keys: std::cell::RefCell::new(std::collections::HashMap::new()),
        }
    }
}

impl FetchHandler for NetFetchHandler {
    fn start(&self, id: u64, request: FetchRequest) -> Option<FetchOutcome> {
        let key = next_key();
        self.keys.borrow_mut().insert(id, key);
        let _ = worker_jobs().send(Job::Fetch(key, id, request, self.reply.clone()));
        None // deferred: the drive loop settles it when the reply arrives
    }
    fn cancel(&self, id: u64) {
        if let Some(key) = self.keys.borrow_mut().remove(&id) {
            let _ = worker_jobs().send(Job::Cancel(key));
        }
    }
    fn fetch_blocking(&self, request: FetchRequest) -> FetchOutcome {
        // Synchronous XMLHttpRequest: the whole response must be in hand before
        // send() returns, so this drives the same worker job to Close on its own
        // private channel instead of leaving it to the drive loop. The routing id
        // is lifted out of the JS id space so a pull credit cannot land on a
        // deferred fetch that is still registered.
        let key = next_key();
        let id = key | (1u64 << 40);
        let (tx, rx) = std::sync::mpsc::channel::<FetchEvent>();
        if worker_jobs()
            .send(Job::Fetch(key, id, request, tx))
            .is_err()
        {
            return FetchOutcome::network_error();
        }
        let deadline = std::time::Instant::now() + SYNC_FETCH_TIMEOUT;
        let mut out: Option<FetchOutcome> = None;
        loop {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            if left.is_zero() {
                let _ = worker_jobs().send(Job::Cancel(key));
                return FetchOutcome::network_error();
            }
            match rx.recv_timeout(left) {
                Ok(FetchEvent::StartStream(_, meta)) => {
                    out = Some(meta);
                    let _ = worker_jobs().send(Job::Pull(id));
                },
                Ok(FetchEvent::Chunk(_, bytes)) => {
                    if let Some(o) = out.as_mut() {
                        o.body.extend_from_slice(&bytes);
                    }
                    let _ = worker_jobs().send(Job::Pull(id));
                },
                Ok(FetchEvent::Close(_)) => {
                    return out.unwrap_or_else(FetchOutcome::network_error);
                },
                // Anything else on this private channel (an error, a failure, or
                // a stray WebSocket event) ends the blocking read.
                Ok(_) | Err(_) => {
                    let _ = worker_jobs().send(Job::Cancel(key));
                    return FetchOutcome::network_error();
                },
            }
        }
    }
    fn request_chunk(&self, id: u64) {
        // The body's ReadableStream was read with an empty buffer: ask the worker
        // to stream one more chunk for this fetch (routed by JS id).
        let _ = worker_jobs().send(Job::Pull(id));
    }
}

impl Drop for NetFetchHandler {
    // When the per-test handler drops (the Runtime is torn down, e.g. after the
    // drive loop's deadline), cancel every fetch it ever started so the worker
    // drops any still-in-flight future instead of leaking a hung task and a
    // checked-out hyper connection. Cancelling an already-finished key is a no-op.
    fn drop(&mut self) {
        for key in self.keys.borrow().values() {
            let _ = worker_jobs().send(Job::Cancel(*key));
        }
    }
}

/// The host `WebSocket` seam: every call hands a job to the shared worker and
/// returns, and the reply reaches the runtime through the same per-test channel
/// the deferred fetches use. Per-test, so a late frame from a prior test lands on
/// a dropped channel and is discarded.
pub struct NetWebSocketHandler {
    reply: std::sync::mpsc::Sender<FetchEvent>,
    live: std::cell::RefCell<Vec<u64>>,
}

impl NetWebSocketHandler {
    pub fn new(reply: std::sync::mpsc::Sender<FetchEvent>) -> Self {
        Self {
            reply,
            live: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl WebSocketHandler for NetWebSocketHandler {
    fn connect(&self, id: u64, request: WebSocketRequest) {
        self.live.borrow_mut().push(id);
        let _ = worker_jobs().send(Job::WsConnect(id, request, self.reply.clone()));
    }
    fn send_text(&self, id: u64, text: String) {
        let _ = worker_jobs().send(Job::WsCommand(id, WsCmd::Text(text)));
    }
    fn send_binary(&self, id: u64, data: Vec<u8>) {
        let _ = worker_jobs().send(Job::WsCommand(id, WsCmd::Binary(data)));
    }
    fn close(&self, id: u64, code: Option<u16>, reason: String) {
        let _ = worker_jobs().send(Job::WsCommand(id, WsCmd::Close(code, reason)));
    }
}

impl Drop for NetWebSocketHandler {
    // The test is over: close every socket it opened, so the worker drops the
    // connection instead of holding it (and its port) for the rest of the run.
    fn drop(&mut self) {
        for id in self.live.borrow().iter() {
            let _ = worker_jobs().send(Job::WsCommand(
                *id,
                WsCmd::Close(Some(1001), "going away".to_owned()),
            ));
        }
    }
}

/// Bridges a test's fetch-event channel to the harness drive loop. Owns the
/// receiver (per test, created alongside the handler's `Sender`).
pub struct ChannelCompletion {
    rx: std::sync::mpsc::Receiver<FetchEvent>,
}

impl ChannelCompletion {
    pub fn new(rx: std::sync::mpsc::Receiver<FetchEvent>) -> Self {
        Self { rx }
    }
}

impl crate::harness::CompletionSource for ChannelCompletion {
    fn drain(&self, apply: &mut dyn FnMut(crate::harness::FetchCompletion)) -> usize {
        let mut n = 0;
        while let Ok(ev) = self.rx.try_recv() {
            apply(to_completion(ev));
            n += 1;
        }
        n
    }
    fn wait(
        &self,
        timeout: std::time::Duration,
        apply: &mut dyn FnMut(crate::harness::FetchCompletion),
    ) -> usize {
        match self.rx.recv_timeout(timeout) {
            Ok(ev) => {
                apply(to_completion(ev));
                1
            },
            Err(_) => 0,
        }
    }
}

fn to_completion(ev: FetchEvent) -> crate::harness::FetchCompletion {
    match ev {
        FetchEvent::StartStream(id, o) => crate::harness::FetchCompletion::StartStream(id, o),
        FetchEvent::Chunk(id, b) => crate::harness::FetchCompletion::Chunk(id, b),
        FetchEvent::Close(id) => crate::harness::FetchCompletion::Close(id),
        FetchEvent::Error(id) => crate::harness::FetchCompletion::Error(id),
        FetchEvent::Fail(id, m) => crate::harness::FetchCompletion::Fail(id, m),
        FetchEvent::WsOpen(id, p, e) => crate::harness::FetchCompletion::WsOpen(id, p, e),
        FetchEvent::WsText(id, t) => crate::harness::FetchCompletion::WsText(id, t),
        FetchEvent::WsBinary(id, b) => crate::harness::FetchCompletion::WsBinary(id, b),
        FetchEvent::WsFlushed(id, n) => crate::harness::FetchCompletion::WsFlushed(id, n),
        FetchEvent::WsError(id) => crate::harness::FetchCompletion::WsError(id),
        FetchEvent::WsClose(id, c, r, clean) => {
            crate::harness::FetchCompletion::WsClose(id, c, r, clean)
        },
    }
}

/// Loads `<script src>` by HTTP GET, resolving each `src` against the test's
/// document URL (so `.sub.js` helpers like `get-host-info.sub.js` come back
/// substituted). One per test (cheap: it owns only the doc URL string).
pub struct ServerLoader {
    pub doc_url: String,
    /// The document's own response headers.
    pub headers: Vec<(String, String)>,
}

impl ScriptSrcLoader for ServerLoader {
    fn document_response_headers(&self) -> Vec<(String, String)> {
        self.headers.clone()
    }

    fn load_script(&self, src: &str) -> Option<String> {
        let base = url::Url::parse(&self.doc_url).ok()?;
        let abs = base.join(src).ok()?;
        http_get(abs.as_str())
    }
}

/// A connected (or spawned) `wpt serve`. `origin` is the plain-http origin the
/// runner drives. A spawned server is torn down on drop.
pub struct ServerCtx {
    pub origin: String,
    /// Origin serving `.https.` tests; `h2_origin` serves `.h2.` tests when set.
    /// Murm's request body is buffered, so a full-duplex h2 upload cannot be
    /// driven yet: `.h2.` files stay on the plain origin unless asked
    /// (`request-upload.h2.any.html` falls from 8 passes to 1 on port 9000).
    https_origin: String,
    h2_origin: Option<String>,
    _spawned: Option<ServerHandle>,
}

/// `wpt serve`'s default https port.
const WPT_HTTPS_PORT: u16 = 8443;

/// The same host as `origin` on `port`, over https.
fn https_origin_on(origin: &str, port: u16) -> String {
    let host = url::Url::parse(origin)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .unwrap_or_else(|| "web-platform.test".to_owned());
    format!("https://{host}:{port}")
}

/// Which `wpt serve` scheme a test is served on: WPT picks it from the dotted
/// flags in the file name (`x.https.any.html`, `y.h2.html`), not the directory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ServeFlag {
    Plain,
    Https,
    H2,
}

fn serve_flag(test_rel: &str) -> ServeFlag {
    let path = test_rel.split(['?', '#']).next().unwrap_or(test_rel);
    let file = path.rsplit('/').next().unwrap_or(path);
    let mut flag = ServeFlag::Plain;
    for part in file.split('.').skip(1) {
        match part {
            "h2" => return ServeFlag::H2,
            "https" => flag = ServeFlag::Https,
            _ => {},
        }
    }
    flag
}

impl ServerCtx {
    /// Connect to an already-running server at `origin` (the `--server-base`
    /// path). Probes once so a typo / down server fails loudly up front.
    pub fn connect(origin: String) -> Result<Self, String> {
        let origin = origin.trim_end_matches('/').to_owned();
        if http_get(&format!("{origin}/common/blank.html")).is_none() {
            return Err(format!(
                "no WPT server reachable at {origin} (is `wpt serve` up?)"
            ));
        }
        Ok(Self {
            https_origin: https_origin_on(&origin, WPT_HTTPS_PORT),
            h2_origin: None,
            origin,
            _spawned: None,
        })
    }

    /// Override the derived https and h2 origins (`--server-https-base`,
    /// `--server-h2-base`).
    pub fn with_secure_origins(mut self, https: Option<String>, h2: Option<String>) -> Self {
        if let Some(https) = https {
            self.https_origin = https.trim_end_matches('/').to_owned();
        }
        if let Some(h2) = h2 {
            self.h2_origin = Some(h2.trim_end_matches('/').to_owned());
        }
        self
    }

    /// Spawn `python wpt serve` under `tests_root`, discover its plain-http
    /// origin, and wait until it answers. Torn down when the returned ctx drops.
    pub fn spawn(tests_root: &Path) -> Result<Self, String> {
        let handle = ServerHandle::spawn(tests_root)?;
        let origin = handle.origin.clone();
        Ok(Self {
            https_origin: https_origin_on(&origin, WPT_HTTPS_PORT),
            h2_origin: None,
            origin,
            _spawned: Some(handle),
        })
    }

    /// The origin that serves a test, given its path relative to the tests root.
    pub fn origin_for(&self, test_rel: &str) -> &str {
        match serve_flag(test_rel) {
            ServeFlag::Plain => &self.origin,
            ServeFlag::Https => &self.https_origin,
            ServeFlag::H2 => self.h2_origin.as_deref().unwrap_or(&self.origin),
        }
    }

    /// The document URL for a test, given its path relative to the tests root.
    pub fn doc_url(&self, test_rel: &str) -> String {
        format!(
            "{}/{}",
            self.origin_for(test_rel),
            test_rel.trim_start_matches('/')
        )
    }

    pub fn loader(&self, doc_url: &str, headers: Vec<(String, String)>) -> ServerLoader {
        ServerLoader {
            doc_url: doc_url.to_owned(),
            headers,
        }
    }
}

/// A spawned `wpt serve` child; killed (whole tree) on drop.
pub struct ServerHandle {
    child: Child,
    pub origin: String,
}

impl ServerHandle {
    fn spawn(tests_root: &Path) -> Result<Self, String> {
        let mut child = Command::new("python")
            .arg("wpt")
            .arg("serve")
            .current_dir(tests_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawning `python wpt serve`: {e}"))?;

        // Read stdout until the canonical plain-http server announces its port,
        // then drain the rest off-thread so the pipe never backs up.
        let stdout = child.stdout.take().ok_or("no stdout from wpt serve")?;
        let mut reader = BufReader::new(stdout);
        let mut port = None;
        let mut line = String::new();
        loop {
            line.clear();
            match reader.read_line(&mut line) {
                Ok(0) => break, // EOF: server exited before binding
                Ok(_) => {
                    if let Some(p) = parse_http_port(&line) {
                        port = Some(p);
                        break;
                    }
                },
                Err(_) => break,
            }
        }
        std::thread::spawn(move || {
            let mut sink = String::new();
            while reader.read_line(&mut sink).map(|n| n > 0).unwrap_or(false) {
                sink.clear();
            }
        });

        let port = port.ok_or("could not read the wpt serve http port from its output")?;
        let origin = format!("http://web-platform.test:{port}");

        // Readiness: poll until the server answers (it logs the port before the
        // listener is fully up).
        for _ in 0..50 {
            if http_get(&format!("{origin}/common/blank.html")).is_some() {
                return Ok(Self { child, origin });
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        let _ = child.kill();
        Err(format!("wpt serve bound {origin} but never answered"))
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        // Kill the whole process tree: wpt serve forks per-protocol workers that
        // a bare child.kill() would orphan.
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/T", "/F", "/PID", &self.child.id().to_string()])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .output();
        }
        #[cfg(not(windows))]
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The primary plain-http port from a `wpt serve` log line. The canonical
/// server is tagged ` http on port N]` (with surrounding spaces, so it does not
/// match `http-local` / `http-public` / `http2`); the first such line is
/// `ports.http[0]`, the origin tests fetch from.
fn parse_http_port(line: &str) -> Option<u16> {
    let tag = " http on port ";
    let start = line.find(tag)? + tag.len();
    let rest = &line[start..];
    let end = rest.find(']')?;
    rest[..end].trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_primary_http_port_only() {
        // The canonical server line.
        assert_eq!(
            parse_http_port(
                "[2026-06-02 21:48:27,647 http on port 8000] INFO - Starting http server on http://web-platform.test:8000"
            ),
            Some(8000)
        );
        // The variant servers must not match (their tag is not ` http on port `).
        assert_eq!(
            parse_http_port("[ts http-local on port 62276] INFO - ..."),
            None
        );
        assert_eq!(
            parse_http_port("[ts http-public on port 62277] INFO - ..."),
            None
        );
        assert_eq!(parse_http_port("[ts h2 on port 9000] INFO - ..."), None);
        assert_eq!(parse_http_port("[ts ws on port 62280] INFO - ..."), None);
        // Noise lines.
        assert_eq!(parse_http_port("INFO:root:Status of subprocess ..."), None);
    }

    #[test]
    fn doc_url_joins_origin_and_test_path() {
        let ctx = ServerCtx {
            origin: "http://web-platform.test:8000".into(),
            https_origin: "https://web-platform.test:8443".into(),
            h2_origin: None,
            _spawned: None,
        };
        assert_eq!(
            ctx.doc_url("fetch/api/basic/x.any.js"),
            "http://web-platform.test:8000/fetch/api/basic/x.any.js"
        );
        // A leading slash on the rel path is not doubled.
        assert_eq!(
            ctx.doc_url("/fetch/api/basic/x.any.js"),
            "http://web-platform.test:8000/fetch/api/basic/x.any.js"
        );
    }

    fn ctx() -> ServerCtx {
        ServerCtx {
            origin: "http://web-platform.test:8000".into(),
            https_origin: https_origin_on("http://web-platform.test:8000", WPT_HTTPS_PORT),
            h2_origin: None,
            _spawned: None,
        }
    }

    #[test]
    fn the_serving_scheme_follows_the_dotted_flags_in_the_file_name() {
        assert_eq!(serve_flag("fetch/api/basic/x.any.html"), ServeFlag::Plain);
        assert_eq!(
            serve_flag("fetch/api/basic/x.https.any.html"),
            ServeFlag::Https
        );
        assert_eq!(
            serve_flag("mixed-content/gen/top.meta/opt-in/fetch.https.html"),
            ServeFlag::Https
        );
        assert_eq!(
            serve_flag("fetch/api/basic/status.h2.any.html?x"),
            ServeFlag::H2
        );
        // A directory named like a flag, or a query that carries one, is not a flag.
        assert_eq!(serve_flag("https/a/x.html"), ServeFlag::Plain);
        assert_eq!(serve_flag("a/x.html?y=.https."), ServeFlag::Plain);
        assert_eq!(serve_flag("a/https.html"), ServeFlag::Plain);
    }

    #[test]
    fn a_secure_test_is_served_from_the_https_origin_on_the_same_host() {
        let ctx = ctx();
        assert_eq!(ctx.origin_for("a/b.html"), "http://web-platform.test:8000");
        assert_eq!(
            ctx.doc_url("upgrade-insecure-requests/x.https.sub.html"),
            "https://web-platform.test:8443/upgrade-insecure-requests/x.https.sub.html"
        );
        // `.h2.` tests stay on the plain origin unless an h2 origin is given.
        assert_eq!(
            ctx.doc_url("fetch/api/basic/status.h2.any.html"),
            "http://web-platform.test:8000/fetch/api/basic/status.h2.any.html"
        );
        let ctx = ctx.with_secure_origins(
            Some("https://example.test:1/".into()),
            Some("https://example.test:2".into()),
        );
        assert_eq!(ctx.origin_for("a/b.https.html"), "https://example.test:1");
        assert_eq!(ctx.origin_for("a/b.h2.html"), "https://example.test:2");
    }
}

#[cfg(test)]
mod phase_c_document_encoding_fixtures {
    use super::*;
    // Test-local adapter exercises the actual rich initial netfetch worker loader.
    fn initial_document_adapter(url: &str) -> (String, Option<&'static str>, String) {
        let loaded = http_get_document(url, Default::default()).expect("actual initial HTTP GET");
        (
            loaded.document.source,
            Some(loaded.document.encoding.encoding.name()),
            loaded.final_url,
        )
    }
    #[test]
    fn phase_c_document_encoding_actual_net_initial_header_bytes() {
        let mut server = mockito::Server::new();
        let page = server
            .mock("GET", "/page")
            .with_status(200)
            .with_header("Content-Type", "text/html;charset=euc-kr")
            .with_body(b"<input value='\xB0\xA1'>")
            .expect(1)
            .create();
        let (source, name, _) = initial_document_adapter(&format!("{}/page", server.url()));
        assert_eq!(
            source, "<input value='가'>",
            "actual netfetch initial byte decoding"
        );
        assert_eq!(name, Some("EUC-KR"));
        page.assert();
    }
    #[test]
    fn phase_c_document_encoding_actual_net_initial_redirect_bom_metadata() {
        let mut server = mockito::Server::new();
        let final_url = format!("{}/final", server.url());
        let first = server
            .mock("GET", "/requested")
            .with_status(302)
            .with_header("Location", &final_url)
            .expect(1)
            .create();
        let last = server
            .mock("GET", "/final")
            .with_status(200)
            .with_header("Content-Type", "text/html;charset=windows-1252")
            .with_body(b"\xEF\xBB\xBF<p>\xC3\xA9</p>")
            .expect(1)
            .create();
        let (source, name, actual_url) =
            initial_document_adapter(&format!("{}/requested", server.url()));
        assert_eq!(source, "<p>é</p>", "leading BOM is consumed");
        assert_eq!(name, Some("UTF-8"));
        assert_eq!(
            actual_url, final_url,
            "redirect identity retained by initial loader"
        );
        first.assert();
        last.assert();
    }
    #[test]
    fn the_initial_document_keeps_the_final_responses_headers() {
        let mut server = mockito::Server::new();
        let first = server
            .mock("GET", "/requested")
            .with_status(302)
            .with_header("Location", "/final")
            .with_header("Referrer-Policy", "no-referrer")
            .expect(1)
            .create();
        let last = server
            .mock("GET", "/final")
            .with_status(200)
            .with_header("Content-Type", "text/html")
            .with_header("Referrer-Policy", "origin")
            .with_body("<p>x</p>")
            .expect(1)
            .create();
        let loaded = http_get_document(&format!("{}/requested", server.url()), Default::default())
            .expect("actual initial HTTP GET");
        let policies: Vec<&str> = loaded
            .headers
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case("referrer-policy"))
            .map(|(_, value)| value.as_str())
            .collect();
        assert_eq!(
            policies,
            ["origin"],
            "the final response's header, not the redirect's"
        );
        let loader = ServerLoader {
            doc_url: "http://x/".into(),
            headers: loaded.headers.clone(),
        };
        assert_eq!(loader.document_response_headers(), loaded.headers);
        first.assert();
        last.assert();
    }
}
