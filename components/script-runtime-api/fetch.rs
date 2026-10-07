// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The `fetch()` host seam.
//!
//! `fetch()` is the one host capability that needs the network, which Mere owns
//! (the layering: genet/the runtime never link a network stack). So the runtime
//! exposes a *sync* [`FetchHandler`] trait — a host (e.g. the WPT runner, or
//! Mere) implements it over an async engine like netfetcher, doing the async work
//! inside (`block_on`) — and the JS `fetch()` / `Request` / `Response` / `Headers`
//! surface is a bootstrap over a single native sink (`__fetch`). No network
//! dependency enters this crate; only the trait does.
//!
//! Request bodies are validated and collected through Streams readers before
//! the whole-body host call. A synchronous handler answers inline; a deferred
//! handler retains the existing abort and demand-driven incremental response
//! hooks. The runtime still owns no network transport.

use std::cell::RefCell;

use script_engine_api::{CallCx, NativeFn, ScriptEngine};

use crate::HostState;

/// A fetch the host should perform on script's behalf.
pub struct FetchRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    /// The HTTP cache mode name (`default` / `no-store` / `reload` / `no-cache` /
    /// `force-cache` / `only-if-cached`); the host maps it to its cache engine.
    pub cache: String,
    /// The redirect mode name (`follow` / `error` / `manual`); the host maps it to
    /// its redirect handling.
    pub redirect: String,
    /// The request mode name (`cors` / `no-cors` / `same-origin` / `navigate`); the
    /// host maps it to its CORS / response-tainting model.
    pub mode: String,
    /// The request's referrer URL (the initiator document), or empty for none; the
    /// host derives the `Referer` header from it per [`Self::referrer_policy`].
    pub referrer: String,
    /// The referrer policy name (`` / `no-referrer` / `origin` / `unsafe-url` / …);
    /// the host maps it to its referrer engine.
    pub referrer_policy: String,
    /// The credentials mode name (`omit` / `same-origin` / `include`); the host maps
    /// it to whether cookies/auth travel with the request.
    pub credentials: String,
    /// Subresource Integrity metadata (`alg-base64 ...`), or empty for none; the
    /// host verifies the response body against it.
    pub integrity: String,
}

/// The result handed back to script. A Fetch *network error* is
/// `network_error == true` (script sees a rejected promise / `TypeError`).
pub struct FetchOutcome {
    pub network_error: bool,
    pub status: u16,
    pub status_text: String,
    /// `basic` | `cors` | `opaque` | `opaqueredirect` | `error`.
    pub response_type: String,
    /// Final URL after redirects.
    pub url: String,
    /// At least one redirect was followed (drives `Response.redirected`).
    pub redirected: bool,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl FetchOutcome {
    pub fn network_error() -> Self {
        Self {
            network_error: true,
            status: 0,
            status_text: String::new(),
            response_type: "error".to_owned(),
            url: String::new(),
            redirected: false,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }
}

/// The host seam: a host (the WPT runner, or Mere's content actor) implements it
/// over a real network engine. Install with `Runtime::set_fetch_handler`.
///
/// Two shapes. A **synchronous** host implements only [`FetchHandler::fetch`] and
/// answers in place (the runner's offline mock, a `block_on` over netfetcher); the
/// JS `fetch()` Promise resolves in the same tick. An **async / deferred** host
/// overrides [`FetchHandler::start`], spawns the work off-thread, returns `None`,
/// and later drives [`Runtime::settle_fetch`] / [`Runtime::fail_fetch`] when the
/// reply arrives. The deferred handler owns its own delivery channel (it is the
/// actor-mailbox seam: a send into the script actor's inbox), so the runtime stays
/// network-free and the engine stays `!Send`. Deferred delivery is what makes
/// mid-flight abort and streaming response bodies possible.
pub trait FetchHandler {
    /// Begin a fetch. The default bridges to [`fetch`](FetchHandler::fetch) and
    /// answers **inline**: returning `Some(outcome)` resolves the JS Promise in the
    /// same tick (no Rust-to-JS re-entry, no pump). A deferred host spawns the work,
    /// returns `None` (Promise left pending), and settles later by `id`.
    fn start(&self, _id: u64, request: FetchRequest) -> Option<FetchOutcome> {
        Some(self.fetch(request))
    }
    /// Synchronous convenience: a host with the answer in hand implements only this.
    fn fetch(&self, _request: FetchRequest) -> FetchOutcome {
        FetchOutcome::network_error()
    }
    /// Cancel an in-flight deferred request (from `AbortController.abort()`). The
    /// default is a no-op (synchronous hosts have nothing in flight).
    fn cancel(&self, _id: u64) {}
    /// Answer a request **blocking**, for synchronous `XMLHttpRequest`. A sync XHR
    /// must have the whole response before `send()` returns, so a deferred host
    /// cannot use its mailbox here: it drives its own transport to completion and
    /// returns the outcome. The default bridges to [`fetch`](FetchHandler::fetch),
    /// which is right for a synchronous host and yields a network error for a
    /// deferred one that does not override this.
    fn fetch_blocking(&self, request: FetchRequest) -> FetchOutcome {
        self.fetch(request)
    }
    /// Demand the next body chunk for a streaming response `id` (the response's
    /// `ReadableStream` was read and its buffer is empty). A deferred host streams
    /// the body lazily: one chunk per request, so a body the script never reads is
    /// never fetched. The default is a no-op (inline hosts deliver the whole body).
    fn request_chunk(&self, _id: u64) {}
}

/// Clone the installed handler out from under the `HostState` borrow, so the
/// handler call holds no borrow (it must not be live if the handler re-enters a
/// native sink). `None` = no handler installed.
fn host_handler<E: ScriptEngine>(cx: &mut E::CallCx<'_>) -> Option<std::rc::Rc<dyn FetchHandler>> {
    let data = cx.host_data()?;
    let cell = data.downcast_ref::<RefCell<HostState>>()?;
    let h = cell.borrow().fetch.clone();
    h
}

/// `__fetch_start(id, method, url, headers, body)` — start a fetch for Promise
/// `id`. `headers` is a newline-delimited `k,v,k,v` list; `body` is the binary
/// string and a separate presence flag (an empty body remains present). Returns the JSON outcome string when the host
/// answered inline (sync), or `""` when the fetch is deferred (the JS bootstrap
/// leaves the Promise pending for a later `__fetchSettle` / `__fetchFail`). With no
/// handler installed, every fetch is an inline network error.
pub(crate) struct FetchStart;

/// Decode the twelve request arguments starting at `base` (method, url, flat
/// headers, body, cache, redirect, mode, referrer, referrer policy, credentials,
/// integrity, body presence). Shared by `__fetch_start` (which carries a leading id) and
/// `__fetch_sync` (which does not).
fn read_request<E: ScriptEngine>(
    cx: &mut E::CallCx<'_>,
    base: usize,
) -> Result<FetchRequest, E::Error> {
    let s = |i: usize, cx: &mut E::CallCx<'_>| -> Result<String, E::Error> {
        let a = cx.arg(base + i);
        cx.value_to_string(&a)
    };
    let method = s(0, cx)?;
    let url = s(1, cx)?;
    let headers_flat = s(2, cx)?;
    let body_str = s(3, cx)?;
    let cache = s(4, cx)?;
    let redirect = s(5, cx)?;
    let mode = s(6, cx)?;
    let referrer = s(7, cx)?;
    let referrer_policy = s(8, cx)?;
    let credentials = s(9, cx)?;
    let integrity = s(10, cx)?;
    let body_present = s(11, cx)? == "1";
    // The body crosses as a lossless "binary string": each JS char code (0-255)
    // is one byte. `char as u8` recovers the byte (every char is <= 0xFF).
    let body = body_present.then(|| body_str.chars().map(|c| c as u8).collect::<Vec<u8>>());
    Ok(FetchRequest {
        method,
        url,
        headers: parse_flat_headers(&headers_flat),
        body,
        cache,
        redirect,
        mode,
        referrer,
        referrer_policy,
        credentials,
        integrity,
    })
}

impl<E: ScriptEngine> NativeFn<E> for FetchStart {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let a0 = cx.arg(0);
        let id = cx.value_to_string(&a0)?.parse::<u64>().unwrap_or(0);
        let request = read_request::<E>(cx, 1)?;

        // Clone the handler before calling it (no borrow held across `start`).
        let outcome = match host_handler::<E>(cx) {
            Some(handler) => handler.start(id, request),
            None => Some(FetchOutcome::network_error()),
        };
        if outcome.is_none() {
            if let Some(data) = cx.host_data() {
                if let Some(host) = data.downcast_ref::<RefCell<HostState>>() {
                    if let Some(agent) = host.borrow().agent.upgrade() {
                        agent
                            .borrow_mut()
                            .fetch_realms
                            .insert(id, cx.current_realm());
                    }
                }
            }
        }
        match outcome {
            Some(o) => cx.make_string(&encode_outcome(&o)), // inline (sync) answer
            None => cx.make_string(""),                     // deferred: settle later
        }
    }
}

/// `__fetch_sync(method, url, headers, body, …)` — perform a fetch **blocking**
/// and return the JSON outcome. Backs synchronous `XMLHttpRequest`, which must
/// have the whole response before `send()` returns and so cannot use the deferred
/// path. With no handler installed the outcome is a network error.
pub(crate) struct FetchSync;

impl<E: ScriptEngine> NativeFn<E> for FetchSync {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let request = read_request::<E>(cx, 0)?;
        let outcome = match host_handler::<E>(cx) {
            Some(handler) => handler.fetch_blocking(request),
            None => FetchOutcome::network_error(),
        };
        cx.make_string(&encode_outcome(&outcome))
    }
}

/// `__fetch_abort(id)` — relay `AbortController.abort()` to the host so it can
/// cancel the in-flight deferred request. Tolerant of unknown / already-settled ids.
pub(crate) struct FetchAbort;

impl<E: ScriptEngine> NativeFn<E> for FetchAbort {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let a0 = cx.arg(0);
        let id = cx.value_to_string(&a0)?.parse::<u64>().unwrap_or(0);
        if let Some(handler) = host_handler::<E>(cx) {
            handler.cancel(id);
        }
        if let Some(data) = cx.host_data() {
            if let Some(host) = data.downcast_ref::<RefCell<HostState>>() {
                if let Some(agent) = host.borrow().agent.upgrade() {
                    agent.borrow_mut().fetch_realms.remove(&id);
                }
            }
        }
        Ok(cx.undefined())
    }
}

/// `__fetch_pull(id)` — demand the next body chunk for streaming response `id`
/// (the body's `ReadableStream` was read with an empty buffer). Relays to the host
/// so it streams one chunk; a body the script never reads is never fetched.
pub(crate) struct FetchPull;

impl<E: ScriptEngine> NativeFn<E> for FetchPull {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let a0 = cx.arg(0);
        let id = cx.value_to_string(&a0)?.parse::<u64>().unwrap_or(0);
        if let Some(handler) = host_handler::<E>(cx) {
            handler.request_chunk(id);
        }
        Ok(cx.undefined())
    }
}

/// `__resolve_url(url)` — resolve `url` against the document base URL (WHATWG URL
/// resolution via the `url` crate). An already-absolute `url` is returned
/// unchanged; a relative one with no base set is returned as-is (so a network
/// fetch of it fails, the disk-mode default). Backs relative `Request` / `fetch()`
/// URLs in server-mode WPT runs.
pub(crate) struct ResolveUrl;

impl<E: ScriptEngine> NativeFn<E> for ResolveUrl {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let a0 = cx.arg(0);
        let input = cx.value_to_string(&a0)?;
        let base = host_base_url::<E>(cx);
        cx.make_string(&resolve_against(base.as_deref(), &input))
    }
}

/// `__url_parse(input, base)` — parse `input` (optionally against a non-empty
/// `base`) and return its WHATWG components as JSON, or `""` on failure. Backs the
/// JS `URL` constructor.
pub(crate) struct UrlParse;

impl<E: ScriptEngine> NativeFn<E> for UrlParse {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let a0 = cx.arg(0);
        let input = cx.value_to_string(&a0)?;
        let a1 = cx.arg(1);
        let base = cx.value_to_string(&a1)?;
        let parsed = if base.is_empty() {
            url::Url::parse(&input)
        } else {
            url::Url::parse(&base).and_then(|b| b.join(&input))
        };
        cx.make_string(&parsed.map(|u| url_components_json(&u)).unwrap_or_default())
    }
}

/// `__url_with(href, part, value)` — parse `href`, apply the WHATWG setter for
/// `part`, and return the new components as JSON (or `""` on failure). Backs the JS
/// `URL` component setters via the `url` crate.
pub(crate) struct UrlWith;

impl<E: ScriptEngine> NativeFn<E> for UrlWith {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let a0 = cx.arg(0);
        let href = cx.value_to_string(&a0)?;
        let a1 = cx.arg(1);
        let part = cx.value_to_string(&a1)?;
        let a2 = cx.arg(2);
        let value = cx.value_to_string(&a2)?;

        let result = url::Url::parse(&href).ok().and_then(|mut u| {
            match part.as_str() {
                "href" => {
                    return url::Url::parse(&value)
                        .ok()
                        .map(|u| url_components_json(&u));
                },
                "protocol" => {
                    let _ = u.set_scheme(value.trim_end_matches(':'));
                },
                "username" => {
                    let _ = u.set_username(&value);
                },
                "password" => {
                    let _ = u.set_password((!value.is_empty()).then_some(value.as_str()));
                },
                "hostname" => {
                    let _ = u.set_host((!value.is_empty()).then_some(value.as_str()));
                },
                "host" => {
                    let (h, p) = value.split_once(':').unwrap_or((value.as_str(), ""));
                    let _ = u.set_host((!h.is_empty()).then_some(h));
                    let _ = u.set_port(p.parse().ok());
                },
                "port" => {
                    let _ = u.set_port(value.parse().ok());
                },
                "pathname" => u.set_path(&value),
                "search" => {
                    u.set_query((!value.is_empty()).then_some(value.trim_start_matches('?')))
                },
                "hash" => {
                    u.set_fragment((!value.is_empty()).then_some(value.trim_start_matches('#')))
                },
                _ => return None,
            }
            Some(url_components_json(&u))
        });
        cx.make_string(&result.unwrap_or_default())
    }
}

/// The WHATWG URL components of `u` as a JSON object (the shape the JS `URL`
/// reads): `href`, `protocol` (scheme + `:`), `username`, `password`, `host`
/// (host + `:port`), `hostname`, `port`, `origin`, `pathname`, `search` (with a
/// leading `?`), `hash` (with a leading `#`).
fn url_components_json(u: &url::Url) -> String {
    let host = match (u.host_str(), u.port()) {
        (Some(h), Some(p)) => format!("{h}:{p}"),
        (Some(h), None) => h.to_owned(),
        _ => String::new(),
    };
    let mut s = String::new();
    s.push('{');
    s.push_str("\"href\":");
    push_json_str(&mut s, u.as_str());
    s.push_str(",\"protocol\":");
    push_json_str(&mut s, &format!("{}:", u.scheme()));
    s.push_str(",\"username\":");
    push_json_str(&mut s, u.username());
    s.push_str(",\"password\":");
    push_json_str(&mut s, u.password().unwrap_or(""));
    s.push_str(",\"host\":");
    push_json_str(&mut s, &host);
    s.push_str(",\"hostname\":");
    push_json_str(&mut s, u.host_str().unwrap_or(""));
    s.push_str(",\"port\":");
    push_json_str(&mut s, &u.port().map(|p| p.to_string()).unwrap_or_default());
    s.push_str(",\"origin\":");
    push_json_str(&mut s, &u.origin().ascii_serialization());
    s.push_str(",\"pathname\":");
    push_json_str(&mut s, u.path());
    s.push_str(",\"search\":");
    push_json_str(
        &mut s,
        &u.query().map(|q| format!("?{q}")).unwrap_or_default(),
    );
    s.push_str(",\"hash\":");
    push_json_str(
        &mut s,
        &u.fragment().map(|f| format!("#{f}")).unwrap_or_default(),
    );
    s.push('}');
    s
}

/// Read the document base URL out of host state, if any.
fn host_base_url<E: ScriptEngine>(cx: &mut E::CallCx<'_>) -> Option<String> {
    let data = cx.host_data()?;
    let cell = data.downcast_ref::<RefCell<HostState>>()?;
    let base = cell.borrow().document_base_url();
    base
}

/// WHATWG-resolve `input` against `base`. Absolute `input` wins; with no base, a
/// relative `input` is returned unchanged.
pub(crate) fn resolve_against(base: Option<&str>, input: &str) -> String {
    match base.and_then(|b| url::Url::parse(b).ok()) {
        Some(b) => b
            .join(input)
            .map(|u| u.to_string())
            .unwrap_or_else(|_| input.to_owned()),
        None => input.to_owned(),
    }
}

/// Split a newline-delimited `k,v,k,v` header list into pairs (a trailing odd
/// element, if any, is dropped). A header name/value never contains a raw newline.
fn parse_flat_headers(flat: &str) -> Vec<(String, String)> {
    if flat.is_empty() {
        return Vec::new();
    }
    let parts: Vec<&str> = flat.split('\n').collect();
    parts
        .chunks_exact(2)
        .map(|kv| (kv[0].to_owned(), kv[1].to_owned()))
        .collect()
}

/// Encode the outcome as a JSON object the bootstrap parses. Hand-rolled (no JSON
/// dep). The body crosses losslessly as a "binary string": each byte becomes a
/// char (code point 0-255), which `push_json_str` escapes safely and the bootstrap
/// maps back to bytes — so binary bodies survive intact.
pub(crate) fn encode_outcome(o: &FetchOutcome) -> String {
    let mut s = String::new();
    s.push('{');
    s.push_str(&format!("\"networkError\":{},", o.network_error));
    s.push_str(&format!("\"status\":{},", o.status));
    s.push_str("\"statusText\":");
    push_json_str(&mut s, &o.status_text);
    s.push_str(",\"type\":");
    push_json_str(&mut s, &o.response_type);
    s.push_str(",\"url\":");
    push_json_str(&mut s, &o.url);
    s.push_str(&format!(",\"redirected\":{}", o.redirected));
    s.push_str(",\"headers\":[");
    for (i, (k, v)) in o.headers.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push('[');
        push_json_str(&mut s, k);
        s.push(',');
        push_json_str(&mut s, v);
        s.push(']');
    }
    s.push_str("],\"body\":");
    let body_bin: String = o.body.iter().map(|&b| b as char).collect();
    push_json_str(&mut s, &body_bin);
    s.push('}');
    s
}

/// Append `s` as a JSON string literal (quotes + minimal escaping).
fn push_json_str(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Install the deferred fetch sinks (`__fetch_start` / `__fetch_sync` /
/// `__fetch_abort`) and the `fetch()` / `Request` / `Response` / `Headers` /
/// `XMLHttpRequest` bootstrap.
pub(crate) fn install_fetch_surface<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    engine.set_function::<FetchStart>("__fetch_start", 13)?;
    engine.set_function::<FetchSync>("__fetch_sync", 12)?;
    engine.set_function::<FetchAbort>("__fetch_abort", 1)?;
    engine.set_function::<FetchPull>("__fetch_pull", 1)?;
    engine.set_function::<ResolveUrl>("__resolve_url", 1)?;
    engine.set_function::<UrlParse>("__url_parse", 2)?;
    engine.set_function::<UrlWith>("__url_with", 3)?;
    crate::text_encoding::install_text_encoding_surface(engine)?;
    engine.eval(FETCH_BOOTSTRAP)?;
    Ok(())
}

/// The Fetch API JS surface: `Headers` (with validation + sorted iteration +
/// getSetCookie), `TextEncoder` / `TextDecoder`, `URLSearchParams`, `Blob` /
/// `File`, `FormData`, a buffered `ReadableStream` (+ default reader), `Request`,
/// `Response` (+ `error`/`redirect`/`json` statics, `body` as a stream), a shared
/// body mixin (`text`/`json`/`arrayBuffer`/`blob`/`formData`) with WHATWG body
/// extraction (string / URLSearchParams / Blob / FormData / buffers / stream set
/// the right Content-Type), and `fetch()` over the `__fetch` sink. Bodies cross
/// that sink as a lossless binary string, so binary request / response bodies are
/// exact. `formData()` parses both urlencoded and multipart bodies. The separate
/// Streams bootstrap provides private reader/controller operations for bodies;
/// malformed multipart validation remains a separate Fetch limitation.
///
/// It also carries `ProgressEvent` and the `XMLHttpRequest` family
/// (`XMLHttpRequestEventTarget` / `XMLHttpRequestUpload`), which is a state
/// machine over the same seam rather than a second network path: an async send
/// runs through `fetch()` and a sync one through `__fetch_sync`. `responseXML`
/// and `responseType = "document"` are always null — the scripted tier has no
/// XML/HTML parser to build a document response from.
const FETCH_BOOTSTRAP: &str = r#"
(function() {
  // This bridge exists only during synchronous host installation. Shared weak
  // brands preserve body identity across realms; author code never sees slots.
  var streamsBridge = globalThis.__streamsFetch;
  delete globalThis.__streamsFetch;
  var Core = streamsBridge.core, ReadableOps = streamsBridge.readable;
  var PipingOps = streamsBridge.piping;
  var Uint8Array = Core.Uint8Array, ArrayBuffer = Core.ArrayBuffer, Promise = Core.Promise;
  var TypeError = Core.TypeError, RangeError = Core.RangeError;
  streamsBridge = undefined;
  var bodySlots = Core.sharedMap('fetch.bodies');
  var signalSlots = Core.sharedMap('fetch.signals');
  var controllerSlots = Core.sharedMap('fetch.abortControllers');
  var NativeEventTarget = EventTarget, NativeEvent = Event, NativeDOMException = DOMException;
  var nativeEventPrototype = Event.prototype;
  var nativeDispatch = EventTarget.prototype.dispatchEvent;
  var nativeArrayBufferIsView = ArrayBuffer.isView;
  var nativeU8Set = Uint8Array.prototype.set;
  var nativeJsonParse = JSON.parse;
  var nativeSetTimeout = setTimeout;
  var nativeFetchStart = globalThis.__fetch_start, nativeFetchSync = globalThis.__fetch_sync;
  var nativeFetchAbort = globalThis.__fetch_abort, nativeFetchPull = globalThis.__fetch_pull;
  var NativeWeakRef = typeof WeakRef === 'function' ? WeakRef : undefined;
  var nativeDeref = NativeWeakRef === undefined ? undefined : NativeWeakRef.prototype.deref;
  var hasSym = (typeof Symbol !== 'undefined' && Symbol.iterator);

  // RFC 7230 token for header names; values reject CR/LF/NUL and trim OWS.
  var TOKEN_RE = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/;
  function checkName(n) {
    n = String(n);
    if (!TOKEN_RE.test(n)) throw new TypeError("Invalid header name: '" + n + "'");
    return n.toLowerCase();
  }
  function checkValue(v) {
    // Normalize: strip leading/trailing HTTP whitespace (tab/LF/CR/space), then a
    // valid value has no interior NUL/CR/LF.
    v = String(v).replace(/^[\t\n\r ]+|[\t\n\r ]+$/g, "");
    if (/[\r\n\0]/.test(v)) throw new TypeError("Invalid header value");
    return v;
  }

  // ---- Header guards (WHATWG): request / request-no-cors / response / immutable. ----
  var FORBIDDEN_REQUEST = {
    'accept-charset': 1, 'accept-encoding': 1, 'access-control-request-headers': 1,
    'access-control-request-method': 1, 'connection': 1, 'content-length': 1,
    'cookie': 1, 'cookie2': 1, 'date': 1, 'dnt': 1, 'expect': 1, 'host': 1,
    'keep-alive': 1, 'origin': 1, 'permissions-policy': 1, 'referer': 1,
    'set-cookie': 1, 'te': 1, 'trailer': 1, 'transfer-encoding': 1, 'upgrade': 1, 'via': 1
  };
  var METHOD_OVERRIDE = { 'x-http-method': 1, 'x-http-method-override': 1, 'x-method-override': 1 };
  // name + value are already lower-cased name / normalized value.
  function isForbiddenRequestHeader(name, value) {
    if (FORBIDDEN_REQUEST[name] || /^proxy-/.test(name) || /^sec-/.test(name)) return true;
    if (METHOD_OVERRIDE[name]) {
      var toks = String(value).split(',');
      for (var i = 0; i < toks.length; i++) {
        var t = toks[i].replace(/^[\t\n\r ]+|[\t\n\r ]+$/g, "").toLowerCase();
        if (t === 'connect' || t === 'trace' || t === 'track') return true;
      }
    }
    return false;
  }
  function isForbiddenResponseHeader(name) {
    return name === 'set-cookie' || name === 'set-cookie2';
  }
  // A CORS-unsafe request-header byte: < 0x20 (except HT), or one of "():<>?@[\]{} or 0x7F.
  var CORS_UNSAFE_RE = /[\x00-\x08\x0a-\x1f"():<>?@\[\\\]{}\x7f]/;
  function isNoCorsSafelisted(name, value) {
    if (name === 'accept' || name === 'accept-language' || name === 'content-language') {
      return value.length <= 128 && !CORS_UNSAFE_RE.test(value);
    }
    if (name === 'content-type') {
      if (value.length > 128) return false;
      var mime = value.split(';')[0].replace(/^[\t\n\r ]+|[\t\n\r ]+$/g, "").toLowerCase();
      return mime === 'application/x-www-form-urlencoded' || mime === 'multipart/form-data' || mime === 'text/plain';
    }
    return false;
  }
  // Whether `guard` permits writing (name, value). `immutable` throws; the others
  // silently drop a disallowed header (the WHATWG "fill"/append behaviour).
  function guardAllows(guard, name, value) {
    switch (guard) {
      case 'immutable': throw new TypeError("Headers are immutable");
      case 'request': return !isForbiddenRequestHeader(name, value);
      case 'request-no-cors': return isNoCorsSafelisted(name, value);
      case 'response': return !isForbiddenResponseHeader(name);
      default: return true;
    }
  }

  // %IteratorPrototype% (two protos up from an array iterator), so our iterators
  // sit on the right chain for `checkIteratorProperties`-style WPT assertions.
  var IteratorProto = (hasSym && [][Symbol.iterator])
    ? Object.getPrototypeOf(Object.getPrototypeOf([][Symbol.iterator]()))
    : Object.prototype;
  // Build an iterator whose own prototype carries `next` (configurable/enumerable/
  // writable) and chains to %IteratorPrototype%.
  function makeIter(nextFn) {
    var proto = Object.create(IteratorProto);
    proto.next = nextFn;
    if (hasSym) proto[Symbol.iterator] = function() { return this; };
    return Object.create(proto);
  }
  // Snapshot iterator over a fixed array (URLSearchParams / FormData).
  function makeIterator(arr) {
    var i = 0;
    return makeIter(function() {
      return i < arr.length ? { value: arr[i++], done: false } : { value: undefined, done: true };
    });
  }
  // Live iterator over a Headers object: re-reads the sorted list each step, so
  // appends/deletes during iteration are observed (per the Headers iteration tests).
  // kind 0 = key, 1 = value, 2 = entry.
  function headersIter(headers, kind) {
    var i = 0;
    return makeIter(function() {
      var s = headers._sorted();
      if (i >= s.length) return { value: undefined, done: true };
      var p = s[i++];
      var v = kind === 0 ? p[0] : kind === 1 ? p[1] : [p[0], p[1]];
      return { value: v, done: false };
    });
  }

  function Headers(init) {
    this._h = [];
    if (init === undefined) return; // new Headers() / new Headers(undefined): empty
    if (init === null || typeof init !== 'object') throw new TypeError("Invalid HeadersInit");
    if (init instanceof Headers) {
      for (var i = 0; i < init._h.length; i++) this._h.push([init._h[i][0], init._h[i][1]]);
    } else if (Array.isArray(init) || (hasSym && typeof init[Symbol.iterator] === 'function')) {
      // sequence<sequence<ByteString>>: each entry is a [name, value] pair.
      var seq = Array.isArray(init) ? init : Array.from(init);
      for (var j = 0; j < seq.length; j++) {
        if (seq[j] == null || seq[j].length !== 2) throw new TypeError("Invalid header entry");
        this.append(seq[j][0], seq[j][1]);
      }
    } else {
      // record<ByteString, ByteString>: own enumerable keys, in order (the
      // Symbol.iterator probe above + Object.keys here match WebIDL trap ordering).
      var keys = Object.keys(init);
      for (var k = 0; k < keys.length; k++) this.append(keys[k], init[keys[k]]);
    }
  }
  Headers.prototype.append = function(n, v) {
    n = checkName(n); v = checkValue(v);
    // For request-no-cors the safelist check runs against the *combined* value.
    var check = v;
    if (this._guard === 'request-no-cors') { var cur = this.get(n); if (cur !== null) check = cur + ", " + v; }
    if (!guardAllows(this._guard || 'none', n, check)) return;
    this._h.push([n, v]);
  };
  Headers.prototype.set = function(n, v) {
    n = checkName(n); v = checkValue(v);
    if (!guardAllows(this._guard || 'none', n, v)) return;
    this._h = this._h.filter(function(p) { return p[0] !== n; });
    this._h.push([n, v]);
  };
  Headers.prototype.get = function(n) {
    n = checkName(n);
    var out = [];
    for (var i = 0; i < this._h.length; i++) if (this._h[i][0] === n) out.push(this._h[i][1]);
    return out.length ? out.join(", ") : null;
  };
  Headers.prototype.has = function(n) {
    n = checkName(n);
    for (var i = 0; i < this._h.length; i++) if (this._h[i][0] === n) return true;
    return false;
  };
  Headers.prototype['delete'] = function(n) {
    n = checkName(n);
    if (!guardAllows(this._guard || 'none', n, '')) return;
    this._h = this._h.filter(function(p) { return p[0] !== n; });
  };
  Headers.prototype.getSetCookie = function() {
    var out = [];
    for (var i = 0; i < this._h.length; i++) if (this._h[i][0] === 'set-cookie') out.push(this._h[i][1]);
    return out;
  };
  // Sorted, combined view for iteration (set-cookie kept un-combined).
  Headers.prototype._sorted = function() {
    var names = {}, order = [];
    for (var i = 0; i < this._h.length; i++) {
      var k = this._h[i][0];
      if (!(k in names)) { names[k] = []; order.push(k); }
      names[k].push(this._h[i][1]);
    }
    order.sort();
    var out = [];
    for (var j = 0; j < order.length; j++) {
      var n = order[j];
      if (n === 'set-cookie') { for (var c = 0; c < names[n].length; c++) out.push([n, names[n][c]]); }
      else out.push([n, names[n].join(", ")]);
    }
    return out;
  };
  Headers.prototype.forEach = function(cb, thisArg) {
    var s = this._sorted();
    for (var i = 0; i < s.length; i++) cb.call(thisArg, s[i][1], s[i][0], this);
  };
  Headers.prototype.entries = function() { return headersIter(this, 2); };
  Headers.prototype.keys = function() { return headersIter(this, 0); };
  Headers.prototype.values = function() { return headersIter(this, 1); };
  if (hasSym) Headers.prototype[Symbol.iterator] = Headers.prototype.entries;
  globalThis.Headers = Headers;

  // ---- Body mixin: privately branded streams, single-use readers. ----
  function bodyRecord(self) {
    var b = Core.get(bodySlots, self);
    if (b === undefined) throw new Core.TypeError('Illegal body receiver');
    return b;
  }
  function byteStrategy() {
    var strategy = Core.record(); strategy.highWaterMark = 0;
    strategy.sizeAlgorithm = undefined; return strategy;
  }
  function bufferedBodyStream(bytes) {
    var stream = ReadableOps.createByte(Core.record(), byteStrategy());
    if (Core.viewInfo(bytes).byteLength !== 0) ReadableOps.enqueue(stream, Core.copyBytes(bytes));
    ReadableOps.close(stream);
    return stream;
  }
  function setBody(self, kind, bytes, stream) {
    var b = Core.record(); b.kind = kind; b.bytes = bytes; b.nativeLive = false;
    b.stream = stream !== null ? stream : bytes !== null ? bufferedBodyStream(bytes) : null;
    Core.set(bodySlots, self, b); return b;
  }
  function bodyStream(self) { return bodyRecord(self).stream; }
  function bodyUsed(self) {
    var stream = bodyRecord(self).stream;
    return stream !== null && ReadableOps.isDisturbed(stream);
  }
  function bodyUnusable(b) {
    return b.stream !== null && (ReadableOps.isLocked(b.stream) || ReadableOps.isDisturbed(b.stream));
  }
  function cloneBodyInto(src, dst) {
    var original = bodyRecord(src);
    if (bodyUnusable(original)) throw new Core.TypeError('Body is disturbed or locked');
    if (original.nativeLive) throw new Core.TypeError('Cannot clone an in-flight native response');
    var copy = Core.record(); copy.kind = original.kind; copy.bytes = original.bytes;
    copy.nativeLive = false; copy.stream = null;
    if (original.stream !== null) {
      var branches = ReadableOps.tee(original.stream, true, function (chunk) { return Core.clone(chunk); });
      original.stream = branches[0]; copy.stream = branches[1];
    }
    Core.set(bodySlots, dst, copy);
  }
  // Private callbacks avoid adopting intermediate byte arrays/read-result
  // objects as promises. Object.prototype.then must not intercept read-all-bytes.
  function collectBody(self, success, failure, onReader) {
    var b, reader;
    try {
      b = bodyRecord(self);
      if (bodyUnusable(b)) throw new Core.TypeError('Body is disturbed or locked');
      if (b.stream === null) { success(new Core.Uint8Array(0)); return; }
      reader = ReadableOps.acquireReader(b.stream, 'default');
    } catch (e) { failure(e); return; }
    var parts = Core.queue(), total = 0, ended = false, running = false;
    function fail(reason) { if (!ended) { ended = true; failure(reason); } }
    function complete() {
      if (ended) return;
      ended = true;
      try {
        var all = new Core.Uint8Array(total), offset = 0;
        while (Core.length(parts)) {
          var part = Core.shift(parts);
          Core.call(nativeU8Set, all, [part, offset]); offset += Core.viewInfo(part).byteLength;
        }
        success(all);
      } catch (e) { failure(e); }
    }
    if (onReader !== undefined) onReader(reader);
    function pump() {
      if (running || ended) return;
      running = true;
      while (!ended) {
        var synchronous = true, received = false, steps = Core.record();
        steps.chunk = function (chunk) {
          if (ended) return;
          try {
            var info = Core.viewInfo(chunk);
            if (info.kind !== 'Uint8Array') throw new Core.TypeError('Body chunk is not a Uint8Array');
            var bytes = Core.copyBytes(chunk);
            Core.push(parts, bytes); total += info.byteLength; received = true;
          } catch (e) { fail(e); }
          if (!synchronous) pump();
        };
        steps.close = complete; steps.error = fail;
        try { ReadableOps.readRequest(reader, steps); } catch (e) { fail(e); }
        synchronous = false;
        if (!received) break;
      }
      running = false;
    }
    pump();
  }
  function consumeBody(self, convert) {
    var result = Core.deferred();
    collectBody(self, function (bytes) {
      try { result.resolve(convert(bytes)); } catch (e) { result.reject(e); }
    }, result.reject);
    return result.promise;
  }
  function utf8Encode(s) {
    var b = [];
    for (var i = 0; i < s.length; i++) {
      var c = s.charCodeAt(i);
      if (c < 0x80) b.push(c);
      else if (c < 0x800) b.push(0xC0 | (c >> 6), 0x80 | (c & 0x3F));
      else if (c >= 0xD800 && c <= 0xDBFF && i + 1 < s.length) {
        var cp = 0x10000 + ((c & 0x3FF) << 10) + (s.charCodeAt(++i) & 0x3FF);
        b.push(0xF0 | (cp >> 18), 0x80 | ((cp >> 12) & 0x3F), 0x80 | ((cp >> 6) & 0x3F), 0x80 | (cp & 0x3F));
      } else b.push(0xE0 | (c >> 12), 0x80 | ((c >> 6) & 0x3F), 0x80 | (c & 0x3F));
    }
    return new Uint8Array(b);
  }
  // WHATWG "UTF-8 decode": strip a leading UTF-8 BOM, then run the UTF-8 decoder
  // replacing every ill-formed sequence with U+FFFD (overlong, out-of-range, lone
  // continuation, and truncated sequences all collapse to the replacement char).
  function utf8Decode(bytes) {
    var out = '', i = 0, n = bytes.length;
    if (n >= 3 && bytes[0] === 0xEF && bytes[1] === 0xBB && bytes[2] === 0xBF) i = 3;
    while (i < n) {
      var b = bytes[i++];
      if (b < 0x80) { out += String.fromCharCode(b); continue; }
      var needed, cp, lower = 0x80, upper = 0xBF;
      if (b >= 0xC2 && b <= 0xDF) { needed = 1; cp = b & 0x1F; }
      else if (b >= 0xE0 && b <= 0xEF) { needed = 2; cp = b & 0x0F; if (b === 0xE0) lower = 0xA0; else if (b === 0xED) upper = 0x9F; }
      else if (b >= 0xF0 && b <= 0xF4) { needed = 3; cp = b & 0x07; if (b === 0xF0) lower = 0x90; else if (b === 0xF4) upper = 0x8F; }
      else { out += '�'; continue; }
      var ok = true;
      for (var k = 0; k < needed; k++) {
        if (i >= n) { ok = false; break; }
        var nb = bytes[i];
        var lo = (k === 0) ? lower : 0x80, hi = (k === 0) ? upper : 0xBF;
        if (nb < lo || nb > hi) { ok = false; break; }
        cp = (cp << 6) | (nb & 0x3F); i++;
      }
      if (!ok) { out += '�'; continue; }
      if (cp <= 0xFFFF) out += String.fromCharCode(cp);
      else { cp -= 0x10000; out += String.fromCharCode(0xD800 + (cp >> 10), 0xDC00 + (cp & 0x3FF)); }
    }
    return out;
  }

  // Body bytes cross the native __fetch sink as a lossless "binary string": each
  // char code (0-255) is one byte. These convert a Uint8Array to/from that form.
  function bytesToBinaryString(bytes) {
    var s = '', CH = 0x8000;
    for (var i = 0; i < bytes.length; i += CH) {
      s += String.fromCharCode.apply(null, bytes.subarray(i, i + CH));
    }
    return s;
  }
  function binaryStringToBytes(s) {
    var b = new Uint8Array(s.length);
    for (var i = 0; i < s.length; i++) b[i] = s.charCodeAt(i) & 0xFF;
    return b;
  }

  // ---- TextEncoder / TextDecoder ----
  // The WPT-facing wrappers keep only opaque native IDs. Decoder state and
  // encoding tables live in the host; shared weak brands let a method from one
  // same-agent realm accept an instance created in another realm.
  var encObject = Object, encDefine = encObject.defineProperty;
  var encGetDesc = encObject.getOwnPropertyDescriptor, encGetProto = encObject.getPrototypeOf;
  var encApply = Reflect.apply;
  var encWeakMap = WeakMap, encWeakMapGet = WeakMap.prototype.get, encWeakMapSet = WeakMap.prototype.set;
  var encWeakSet = WeakSet, encWeakSetHas = WeakSet.prototype.has, encWeakSetAdd = WeakSet.prototype.add;
  var encString = String, encStringFromCharCode = String.fromCharCode;
  var encStringSlice = String.prototype.slice, encStringIndexOf = String.prototype.indexOf;
  var encCreate = encObject.create, encTypeErrorCtor = TypeError, encRangeErrorCtor = RangeError;
  var encCharCodeAt = String.prototype.charCodeAt, encArrayBuffer = ArrayBuffer;
  var encIsView = ArrayBuffer.isView, encUint8Array = Uint8Array;
  var encTypedArrayProto = encGetProto(Uint8Array.prototype);
  var encTypedArrayTag = encGetDesc(encTypedArrayProto, Symbol.toStringTag).get;
  var encTypedBuffer = encGetDesc(encTypedArrayProto, 'buffer').get;
  var encTypedOffset = encGetDesc(encTypedArrayProto, 'byteOffset').get;
  var encTypedLength = encGetDesc(encTypedArrayProto, 'byteLength').get;
  var encDataViewProto = DataView.prototype;
  var encDataBuffer = encGetDesc(encDataViewProto, 'buffer').get;
  var encDataOffset = encGetDesc(encDataViewProto, 'byteOffset').get;
  var encDataLength = encGetDesc(encDataViewProto, 'byteLength').get;
  var encArrayBufferLength = encGetDesc(ArrayBuffer.prototype, 'byteLength').get;
  var encSharedBufferLength = typeof SharedArrayBuffer === 'function'
    ? encGetDesc(SharedArrayBuffer.prototype, 'byteLength').get : null;
  var encSharedState = globalThis.__agentTimers;
  if (encSharedState && (typeof encSharedState === 'object' || typeof encSharedState === 'function')) {
    var encSharedDesc = encGetDesc(encSharedState, '__encodingState');
    if (!encSharedDesc) {
      var encNewState = { decoders: new encWeakMap(), encoders: new encWeakSet() };
      encDefine(encSharedState, '__encodingState', { value: encNewState });
      encSharedDesc = { value: encNewState };
    }
    var encBrands = encSharedDesc.value;
  } else {
    var encBrands = { decoders: new encWeakMap(), encoders: new encWeakSet() };
  }
  var decoderBrands = encBrands.decoders, encoderBrands = encBrands.encoders;
  var nativeDecoderCreate = globalThis.__text_decoder_create;
  var nativeDecoderDecode = globalThis.__text_decoder_decode;
  var nativeDecoderRelease = globalThis.__text_decoder_release;
  try { delete globalThis.__text_decoder_create; delete globalThis.__text_decoder_decode; delete globalThis.__text_decoder_release; } catch (encDeleteError) {}
  if (typeof nativeDecoderCreate !== 'function' || typeof nativeDecoderDecode !== 'function' || typeof nativeDecoderRelease !== 'function')
    throw new TypeError('TextDecoder host is unavailable');

  // Capture the registry and registration operation before author scripts run.
  var EncFinalizationRegistry = globalThis.FinalizationRegistry;
  var encRegistry = typeof EncFinalizationRegistry === 'function'
    ? new EncFinalizationRegistry(function(id) { encApply(nativeDecoderRelease, undefined, [id]); }) : null;
  var encRegister = encRegistry ? EncFinalizationRegistry.prototype.register : null;
  function encGet(map, key) { return encApply(encWeakMapGet, map, [key]); }
  function encSet(map, key, value) { encApply(encWeakMapSet, map, [key, value]); }
  function encHas(set, key) { return encApply(encWeakSetHas, set, [key]); }
  function encAdd(set, key) { encApply(encWeakSetAdd, set, [key]); }
  function encTypeError(message) { return new encTypeErrorCtor(message); }
  function encToString(value) {
    if (typeof value === 'symbol') throw encTypeError('Cannot convert a Symbol value to a string');
    return encString(value);
  }
  function encOptions(options, key) {
    if (options === undefined || options === null) return false;
    if (typeof options !== 'object' && typeof options !== 'function')
      throw encTypeError('Dictionary options must be an object');
    return !!options[key];
  }
  function encNativeError(error) {
    if (error === 'L') throw new encRangeErrorCtor('The encoding label is invalid');
    if (error === 'F') throw encTypeError('The encoded data was not valid');
    throw encTypeError('TextDecoder host operation failed');
  }
  function encAsciiTrim(label) {
    var start = 0, end = label.length;
    function ws(c) { return c === 9 || c === 10 || c === 12 || c === 13 || c === 32; }
    while (start < end && ws(encApply(encCharCodeAt, label, [start]))) start++;
    while (end > start && ws(encApply(encCharCodeAt, label, [end - 1]))) end--;
    var out = '';
    for (var i = start; i < end; i++) out += label[i];
    return out;
  }
  function encByteSource(input, omitted) {
    if (omitted || input === undefined) return { kind: 'empty' };
    if (encIsView(input)) return { kind: 'view', value: input };
    // Vano has distinct shared-view variants which its isView currently omits.
    // These captured getters test intrinsic brands without author properties.
    if (encApply(encTypedArrayTag, input, []) !== undefined) return { kind: 'view', value: input };
    try { encApply(encDataBuffer, input, []); return { kind: 'view', value: input }; } catch (encNotDataView) {}
    try { encApply(encArrayBufferLength, input, []); return { kind: 'buffer', value: input }; } catch (encArrayBufferError) {}
    if (encSharedBufferLength) {
      try { encApply(encSharedBufferLength, input, []); return { kind: 'buffer', value: input }; } catch (encSharedBufferError) {}
    }
    throw encTypeError('TextDecoder input is not an ArrayBuffer or view');
  }
  function encByteString(source) {
    var buffer, offset, length;
    try {
      if (source.kind === 'empty') return '';
      if (source.kind === 'buffer') {
        buffer = source.value; offset = 0;
        try { length = encApply(encArrayBufferLength, buffer, []); }
        catch (encDetachedBuffer) { length = 0; }
        if (encSharedBufferLength) {
          try { length = encApply(encSharedBufferLength, buffer, []); } catch (encNotSharedBuffer) {}
        }
      } else {
        var view = source.value;
        var tag = encApply(encTypedArrayTag, view, []);
        if (tag === undefined) {
          buffer = encApply(encDataBuffer, view, []);
          offset = encApply(encDataOffset, view, []);
          length = encApply(encDataLength, view, []);
        } else {
          buffer = encApply(encTypedBuffer, view, []);
          offset = encApply(encTypedOffset, view, []);
          length = encApply(encTypedLength, view, []);
        }
      }
    } catch (encDetachedView) { return ''; }
    if (!length) return '';
    var bytes = source.kind === 'view' && tag === 'Uint8Array'
      ? view : new encUint8Array(buffer, offset, length);
    if (length === 1) return encApply(encStringFromCharCode, undefined, [bytes[0]]);
    var result = '', codes = encCreate(null), cap = 8192;
    for (var i = 0; i < length; i++) {
      codes[i % cap] = bytes[i];
      if (i % cap === cap - 1 || i === length - 1) {
        var count = i % cap + 1;
        codes.length = count;
        result += encApply(encStringFromCharCode, undefined, codes);
        codes = encCreate(null);
      }
    }
    return result;
  }
  function encUSVString(value, defaultEmpty) {
    var input = defaultEmpty && value === undefined ? '' : encToString(value);
    var output = '';
    for (var i = 0; i < input.length; i++) {
      var c = encApply(encCharCodeAt, input, [i]);
      if (c >= 0xD800 && c <= 0xDBFF) {
        if (i + 1 < input.length) {
          var next = encApply(encCharCodeAt, input, [i + 1]);
          if (next >= 0xDC00 && next <= 0xDFFF) { output += input[i] + input[++i]; continue; }
        }
        output += '\uFFFD';
      } else if (c >= 0xDC00 && c <= 0xDFFF) output += '\uFFFD';
      else output += input[i];
    }
    return output;
  }
  function encUtf8Bytes(input) {
    var codes = encCreate(null), length = 0;
    function put(a, b, c, d) {
      codes[length++] = a;
      if (b !== undefined) codes[length++] = b;
      if (c !== undefined) codes[length++] = c;
      if (d !== undefined) codes[length++] = d;
    }
    for (var i = 0; i < input.length; i++) {
      var c = encApply(encCharCodeAt, input, [i]);
      if (c >= 0xD800 && c <= 0xDBFF && i + 1 < input.length) {
        var low = encApply(encCharCodeAt, input, [i + 1]);
        if (low >= 0xDC00 && low <= 0xDFFF) {
          c = 0x10000 + ((c - 0xD800) << 10) + (low - 0xDC00);
          i++;
        }
      }
      if (c < 0x80) put(c);
      else if (c < 0x800) put(0xC0 | (c >> 6), 0x80 | (c & 0x3F));
      else if (c < 0x10000) put(0xE0 | (c >> 12), 0x80 | ((c >> 6) & 0x3F), 0x80 | (c & 0x3F));
      else put(0xF0 | (c >> 18), 0x80 | ((c >> 12) & 0x3F), 0x80 | ((c >> 6) & 0x3F), 0x80 | (c & 0x3F));
    }
    codes.length = length;
    var bytes = new encUint8Array(length);
    for (var j = 0; j < length; j++) bytes[j] = codes[j];
    return bytes;
  }
  function TextEncoder() {
    if (!new.target) throw encTypeError("Constructor TextEncoder requires 'new'");
    encAdd(encoderBrands, this);
  }
  var encoderGetter = encGetDesc({ get encoding() {
    if (!encHas(encoderBrands, this)) throw encTypeError('Illegal invocation');
    return 'utf-8';
  } }, 'encoding').get;
  encDefine(encoderGetter, 'name', { value: 'get encoding', configurable: true });
  var encoderOps = {
    encode(input) { if (!encHas(encoderBrands, this)) throw encTypeError('Illegal invocation'); return encUtf8Bytes(encUSVString(input, true)); },
    encodeInto(input, destination) {
      if (!encHas(encoderBrands, this)) throw encTypeError('Illegal invocation');
      if (arguments.length < 2) throw encTypeError("Failed to execute 'encodeInto': 2 arguments required");
      var source = encUSVString(input, false);
      if (encApply(encTypedArrayTag, destination, []) !== 'Uint8Array')
        throw encTypeError('encodeInto destination must be a Uint8Array');
      var destLength = 0;
      try { destLength = encApply(encTypedLength, destination, []); } catch (encDetachedDestination) {}
      var read = 0, written = 0;
      for (var i = 0; i < source.length;) {
        var first = encApply(encCharCodeAt, source, [i]);
        var cp = first, units = 1;
        if (first >= 0xD800 && first <= 0xDBFF && i + 1 < source.length) {
          var second = encApply(encCharCodeAt, source, [i + 1]);
          if (second >= 0xDC00 && second <= 0xDFFF) { cp = 0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00); units = 2; }
        }
        var a, b, c, d, count;
        if (cp < 0x80) { a = cp; count = 1; }
        else if (cp < 0x800) { a = 0xC0 | (cp >> 6); b = 0x80 | (cp & 0x3F); count = 2; }
        else if (cp < 0x10000) { a = 0xE0 | (cp >> 12); b = 0x80 | ((cp >> 6) & 0x3F); c = 0x80 | (cp & 0x3F); count = 3; }
        else { a = 0xF0 | (cp >> 18); b = 0x80 | ((cp >> 12) & 0x3F); c = 0x80 | ((cp >> 6) & 0x3F); d = 0x80 | (cp & 0x3F); count = 4; }
        if (written + count > destLength) break;
        destination[written++] = a;
        if (count > 1) destination[written++] = b;
        if (count > 2) destination[written++] = c;
        if (count > 3) destination[written++] = d;
        read += units; i += units;
      }
      return { read: read, written: written };
    }
  };
  encDefine(encoderOps.encode, 'length', { value: 0, configurable: true });
  encDefine(TextEncoder.prototype, 'encoding', { get: encoderGetter, enumerable: true, configurable: true });
  encDefine(TextEncoder.prototype, 'encode', { value: encoderOps.encode, writable: true, enumerable: true, configurable: true });
  encDefine(TextEncoder.prototype, 'encodeInto', { value: encoderOps.encodeInto, writable: true, enumerable: true, configurable: true });
  encDefine(TextEncoder.prototype, 'constructor', { value: TextEncoder, writable: true, enumerable: false, configurable: true });
  if (Symbol.toStringTag) encDefine(TextEncoder.prototype, Symbol.toStringTag, { value: 'TextEncoder', configurable: true });
  defineInterface('TextEncoder', TextEncoder, 0);

  function TextDecoder(label, options) {
    if (!new.target) throw encTypeError("Constructor TextDecoder requires 'new'");
    var convertedLabel = encAsciiTrim(arguments.length && label !== undefined ? encToString(label) : 'utf-8');
    var fatal = encOptions(options, 'fatal');
    var ignoreBOM = encOptions(options, 'ignoreBOM');
    var created = encApply(nativeDecoderCreate, undefined, [convertedLabel, fatal ? '1' : '0', ignoreBOM ? '1' : '0']);
    if (created[0] !== '+') encNativeError(created === '!L' ? 'L' : 'H');
    var separator = encApply(encStringIndexOf, created, [':', 1]);
    if (separator < 2 || separator === created.length - 1) encNativeError('H');
    var id = encApply(encStringSlice, created, [1, separator]);
    var metadata = {
      id: id,
      encoding: encApply(encStringSlice, created, [separator + 1]),
      fatal: fatal,
      ignoreBOM: ignoreBOM,
      decode: nativeDecoderDecode
    };
    encSet(decoderBrands, this, metadata);
    if (!encRegistry) {
      encApply(nativeDecoderRelease, undefined, [id]);
      throw encTypeError('FinalizationRegistry is unavailable');
    }
    try { encApply(encRegister, encRegistry, [this, id]); }
    catch (encRegistrationError) {
      encApply(nativeDecoderRelease, undefined, [id]);
      throw encRegistrationError;
    }
  }
  function decoderAttribute(name) {
    var getter = encGetDesc({ get value() {
      var metadata = encGet(decoderBrands, this);
      if (!metadata) throw encTypeError('Illegal invocation');
      return metadata[name];
    } }, 'value').get;
    encDefine(getter, 'name', { value: 'get ' + name, configurable: true });
    return getter;
  }
  var decoderOps = {
    decode(input, options) {
      var metadata = encGet(decoderBrands, this);
      if (!metadata) throw encTypeError('Illegal invocation');
      var source = encByteSource(input, arguments.length === 0);
      var stream = encOptions(options, 'stream');
      var result = encApply(metadata.decode, undefined, [metadata.id, encByteString(source), stream ? '1' : '0']);
      if (result[0] !== 'S') encNativeError(result === 'F' ? 'F' : 'H');
      return encApply(encStringSlice, result, [1]);
    }
  };
  encDefine(decoderOps.decode, 'length', { value: 0, configurable: true });
  encDefine(TextDecoder.prototype, 'encoding', { get: decoderAttribute('encoding'), enumerable: true, configurable: true });
  encDefine(TextDecoder.prototype, 'fatal', { get: decoderAttribute('fatal'), enumerable: true, configurable: true });
  encDefine(TextDecoder.prototype, 'ignoreBOM', { get: decoderAttribute('ignoreBOM'), enumerable: true, configurable: true });
  encDefine(TextDecoder.prototype, 'decode', { value: decoderOps.decode, writable: true, enumerable: true, configurable: true });
  encDefine(TextDecoder.prototype, 'constructor', { value: TextDecoder, writable: true, enumerable: false, configurable: true });
  if (Symbol.toStringTag) encDefine(TextDecoder.prototype, Symbol.toStringTag, { value: 'TextDecoder', configurable: true });
  defineInterface('TextDecoder', TextDecoder, 0);

  // ---- URLSearchParams ----
  function uspEnc(s) {
    return encodeURIComponent(String(s)).replace(/%20/g, '+')
      .replace(/[!'()~]/g, function(c) { return '%' + c.charCodeAt(0).toString(16).toUpperCase(); });
  }
  function uspDec(s) { return decodeURIComponent(String(s).replace(/\+/g, ' ')); }
  function URLSearchParams(init) {
    this._l = [];
    if (init == null || init === '') return;
    if (init instanceof URLSearchParams) {
      for (var i = 0; i < init._l.length; i++) this._l.push([init._l[i][0], init._l[i][1]]);
    } else if (typeof init === 'string') {
      var q = init.charAt(0) === '?' ? init.slice(1) : init;
      if (q) {
        var pairs = q.split('&');
        for (var k = 0; k < pairs.length; k++) {
          if (!pairs[k]) continue;
          var eq = pairs[k].indexOf('=');
          var nm = eq < 0 ? pairs[k] : pairs[k].slice(0, eq);
          var vl = eq < 0 ? '' : pairs[k].slice(eq + 1);
          this._l.push([uspDec(nm), uspDec(vl)]);
        }
      }
    } else if (Array.isArray(init)) {
      for (var a = 0; a < init.length; a++) {
        if (init[a].length !== 2) throw new TypeError("Invalid URLSearchParams pair");
        this.append(init[a][0], init[a][1]);
      }
    } else { for (var key in init) this.append(key, init[key]); }
  }
  URLSearchParams.prototype.append = function(n, v) { this._l.push([String(n), String(v)]); };
  URLSearchParams.prototype['delete'] = function(n) { n = String(n); this._l = this._l.filter(function(p) { return p[0] !== n; }); };
  URLSearchParams.prototype.get = function(n) { n = String(n); for (var i = 0; i < this._l.length; i++) if (this._l[i][0] === n) return this._l[i][1]; return null; };
  URLSearchParams.prototype.getAll = function(n) { n = String(n); var o = []; for (var i = 0; i < this._l.length; i++) if (this._l[i][0] === n) o.push(this._l[i][1]); return o; };
  URLSearchParams.prototype.has = function(n) { n = String(n); for (var i = 0; i < this._l.length; i++) if (this._l[i][0] === n) return true; return false; };
  URLSearchParams.prototype.set = function(n, v) {
    n = String(n); v = String(v); var done = false; var out = [];
    for (var i = 0; i < this._l.length; i++) {
      if (this._l[i][0] === n) { if (!done) { out.push([n, v]); done = true; } }
      else out.push(this._l[i]);
    }
    if (!done) out.push([n, v]);
    this._l = out;
  };
  URLSearchParams.prototype.sort = function() { this._l.sort(function(a, b) { return a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0; }); };
  URLSearchParams.prototype.forEach = function(cb, thisArg) { for (var i = 0; i < this._l.length; i++) cb.call(thisArg, this._l[i][1], this._l[i][0], this); };
  URLSearchParams.prototype.entries = function() { return makeIterator(this._l.map(function(p) { return [p[0], p[1]]; })); };
  URLSearchParams.prototype.keys = function() { return makeIterator(this._l.map(function(p) { return p[0]; })); };
  URLSearchParams.prototype.values = function() { return makeIterator(this._l.map(function(p) { return p[1]; })); };
  URLSearchParams.prototype.toString = function() { var o = []; for (var i = 0; i < this._l.length; i++) o.push(uspEnc(this._l[i][0]) + '=' + uspEnc(this._l[i][1])); return o.join('&'); };
  Object.defineProperty(URLSearchParams.prototype, 'size', { configurable: true, get: function() { return this._l.length; } });
  if (hasSym) URLSearchParams.prototype[Symbol.iterator] = URLSearchParams.prototype.entries;
  // Replace the pair list from a query string (URL.search setter feeding back in).
  URLSearchParams.prototype._reload = function(search) { this._l = new URLSearchParams(search)._l; };
  // Mutators notify an owning URL (if any) so url.href reflects the change.
  ['append', 'set', 'delete', 'sort'].forEach(function(m) {
    var orig = URLSearchParams.prototype[m];
    URLSearchParams.prototype[m] = function() {
      var r = orig.apply(this, arguments);
      if (this._onchange) this._onchange(this.toString());
      return r;
    };
  });
  globalThis.URLSearchParams = URLSearchParams;

  // ---- URL (WHATWG, backed by the Rust url crate via __url_parse / __url_with) ----
  function URL(url, base) {
    var j = __url_parse(String(url), (base === undefined || base === null) ? "" : String(base));
    if (!j) throw new TypeError("Failed to construct 'URL': Invalid URL");
    this._c = JSON.parse(j);
    this._sp = null;
  }
  function urlComponent(name) {
    Object.defineProperty(URL.prototype, name, {
      configurable: true,
      get: function() { return this._c[name]; },
      set: function(v) {
        var j = __url_with(this._c.href, name, String(v));
        if (j) { this._c = JSON.parse(j); if (this._sp) this._sp._reload(this._c.search); }
      }
    });
  }
  ['protocol', 'username', 'password', 'host', 'hostname', 'port', 'pathname', 'search', 'hash'].forEach(urlComponent);
  Object.defineProperty(URL.prototype, 'href', {
    configurable: true,
    get: function() { return this._c.href; },
    set: function(v) {
      var j = __url_parse(String(v), "");
      if (!j) throw new TypeError("Invalid URL");
      this._c = JSON.parse(j);
      if (this._sp) this._sp._reload(this._c.search);
    }
  });
  Object.defineProperty(URL.prototype, 'origin', { configurable: true, get: function() { return this._c.origin; } });
  Object.defineProperty(URL.prototype, 'searchParams', {
    configurable: true,
    get: function() {
      if (!this._sp) {
        var self = this;
        this._sp = new URLSearchParams(this._c.search);
        this._sp._onchange = function(q) { var j = __url_with(self._c.href, 'search', q ? '?' + q : ''); if (j) self._c = JSON.parse(j); };
      }
      return this._sp;
    }
  });
  URL.prototype.toString = function() { return this._c.href; };
  URL.prototype.toJSON = function() { return this._c.href; };
  URL.parse = function(url, base) { try { return new URL(url, base); } catch (e) { return null; } };
  URL.canParse = function(url, base) { return !!__url_parse(String(url), (base === undefined || base === null) ? "" : String(base)); };
  globalThis.URL = URL;

  // ---- Blob / File ----
  function toBytes(part) {
    if (part instanceof Blob) return part._b;
    if (part instanceof ArrayBuffer) return new Uint8Array(part.slice(0));
    if (ArrayBuffer.isView(part)) return new Uint8Array(part.buffer.slice(part.byteOffset, part.byteOffset + part.byteLength));
    return utf8Encode(String(part));
  }
  function Blob(parts, opts) {
    opts = opts || {};
    var chunks = [], total = 0;
    if (parts != null) {
      if (typeof parts !== 'object' || typeof parts.length !== 'number')
        throw new TypeError("Blob parts must be a sequence");
      for (var i = 0; i < parts.length; i++) { var b = toBytes(parts[i]); chunks.push(b); total += b.length; }
    }
    var all = new Uint8Array(total), off = 0;
    for (var j = 0; j < chunks.length; j++) { all.set(chunks[j], off); off += chunks[j].length; }
    this._b = all;
    this.size = total;
    var t = opts.type === undefined ? '' : String(opts.type);
    this.type = /[^ -~]/.test(t) ? '' : t.toLowerCase();
  }
  Blob.prototype.text = function() { var self = this; return Promise.resolve(utf8Decode(self._b)); };
  Blob.prototype.arrayBuffer = function() { return Promise.resolve(this._b.slice(0).buffer); };
  Blob.prototype.slice = function(start, end, type) {
    var s = this._b.slice(start || 0, end === undefined ? this._b.length : end);
    var b = new Blob([], { type: type || '' }); b._b = s; b.size = s.length; return b;
  };
  globalThis.Blob = Blob;

  function File(parts, name, opts) {
    if (name === undefined) throw new TypeError("File requires a name");
    Blob.call(this, parts, opts);
    this.name = String(name);
    this.lastModified = (opts && opts.lastModified != null) ? (opts.lastModified | 0) : 0;
  }
  File.prototype = Object.create(Blob.prototype);
  File.prototype.constructor = File;
  globalThis.File = File;

  // ---- FormData ----
  function FormData() { this._l = []; }
  FormData.prototype.append = function(name, value, filename) {
    name = String(name);
    if (value instanceof Blob) {
      var fv = value;
      if (filename !== undefined && !(value instanceof File)) {
        fv = new File([value], filename, { type: value.type });
      } else if (filename !== undefined) {
        fv = new File([value], filename, { type: value.type });
      }
      this._l.push([name, fv]);
    } else this._l.push([name, String(value)]);
  };
  FormData.prototype.set = function(name, value, filename) {
    name = String(name); this['delete'](name); this.append(name, value, filename);
  };
  FormData.prototype['delete'] = function(name) { name = String(name); this._l = this._l.filter(function(p) { return p[0] !== name; }); };
  FormData.prototype.get = function(name) { name = String(name); for (var i = 0; i < this._l.length; i++) if (this._l[i][0] === name) return this._l[i][1]; return null; };
  FormData.prototype.getAll = function(name) { name = String(name); var o = []; for (var i = 0; i < this._l.length; i++) if (this._l[i][0] === name) o.push(this._l[i][1]); return o; };
  FormData.prototype.has = function(name) { name = String(name); for (var i = 0; i < this._l.length; i++) if (this._l[i][0] === name) return true; return false; };
  FormData.prototype.forEach = function(cb, thisArg) { for (var i = 0; i < this._l.length; i++) cb.call(thisArg, this._l[i][1], this._l[i][0], this); };
  FormData.prototype.entries = function() { return makeIterator(this._l.map(function(p) { return [p[0], p[1]]; })); };
  FormData.prototype.keys = function() { return makeIterator(this._l.map(function(p) { return p[0]; })); };
  FormData.prototype.values = function() { return makeIterator(this._l.map(function(p) { return p[1]; })); };
  if (hasSym) FormData.prototype[Symbol.iterator] = FormData.prototype.entries;
  globalThis.FormData = FormData;

  // Serialize a FormData to a multipart/form-data body + content-type. File field
  // content is spliced as text here (the one remaining lossy spot for binary file
  // parts); text fields and filenames round-trip exactly.
  function serializeFormData(fd) {
    var boundary = '----genet' + Math.floor(Math.random() * 0x100000000).toString(16) + Math.floor(Math.random() * 0x100000000).toString(16);
    var s = '';
    for (var i = 0; i < fd._l.length; i++) {
      var name = fd._l[i][0], value = fd._l[i][1];
      s += '--' + boundary + '\r\n';
      if (value instanceof Blob) {
        var fn = (value instanceof File) ? value.name : 'blob';
        s += 'Content-Disposition: form-data; name="' + name + '"; filename="' + fn + '"\r\n';
        s += 'Content-Type: ' + (value.type || 'application/octet-stream') + '\r\n\r\n';
        s += utf8Decode(value._b) + '\r\n';
      } else {
        s += 'Content-Disposition: form-data; name="' + name + '"\r\n\r\n';
        s += value + '\r\n';
      }
    }
    s += '--' + boundary + '--\r\n';
    return { body: s, type: 'multipart/form-data; boundary=' + boundary };
  }

  // WHATWG "extract a body": returns { bytes: Uint8Array|null, type: string|null }.
  // Blob / ArrayBuffer / typed-array bodies carry their bytes directly, so binary
  // is exact end-to-end (the body crosses __fetch as a lossless binary string).
  function extractBody(v) {
    if (v == null) return { bytes: null, stream: null, type: null };
    if (typeof v === 'string') return { bytes: utf8Encode(v), stream: null, type: 'text/plain;charset=UTF-8' };
    if (v instanceof URLSearchParams) return { bytes: utf8Encode(v.toString()), stream: null, type: 'application/x-www-form-urlencoded;charset=UTF-8' };
    // Blob / buffers carry their bytes directly: no text round-trip, so binary is exact.
    if (v instanceof Blob) return { bytes: v._b.slice(0), stream: null, type: v.type ? v.type : null };
    if (v instanceof FormData) { var r = serializeFormData(v); return { bytes: utf8Encode(r.body), stream: null, type: r.type }; }
    if (ReadableOps.isReadable(v)) {
      // A stream body stays a live stream (consumed lazily): a stream already
      // locked or disturbed is not a usable body (the from-stream tests).
      if (ReadableOps.isLocked(v) || ReadableOps.isDisturbed(v)) throw new TypeError("Body stream is already locked or disturbed");
      return { bytes: null, stream: v, type: null };
    }
    if (Core.isArrayBuffer(v)) return { bytes: Core.copyBytes(new Uint8Array(v)), stream: null, type: null };
    if (Core.call(nativeArrayBufferIsView, ArrayBuffer, [v])) return { bytes: Core.copyBytes(v), stream: null, type: null };
    return { bytes: utf8Encode(String(v)), stream: null, type: 'text/plain;charset=UTF-8' };
  }

  // Parse a multipart/form-data body (already decoded to text) into a FormData.
  // Text fields round-trip exactly; a file part becomes a File (its content via
  // the UTF-8 text, the one lossy spot for binary parts).
  function parseMultipart(text, boundary) {
    var fd = new FormData();
    var segments = text.split("--" + boundary);
    for (var i = 0; i < segments.length; i++) {
      var part = segments[i];
      if (part === "" || /^--\r?\n?$/.test(part)) continue; // preamble / closing delim
      part = part.replace(/^\r\n/, "").replace(/\r\n$/, "");
      var sep = part.indexOf("\r\n\r\n");
      if (sep < 0) continue;
      var head = part.slice(0, sep), content = part.slice(sep + 4);
      var nameM = /name="([^"]*)"/i.exec(head);
      if (!nameM) continue;
      var fileM = /filename="([^"]*)"/i.exec(head);
      var ctM = /content-type:\s*([^\r\n]+)/i.exec(head);
      if (fileM) {
        fd.append(nameM[1], new File([utf8Encode(content)], fileM[1], { type: ctM ? ctM[1].replace(/\s+$/, "") : "" }));
      } else {
        fd.append(nameM[1], content);
      }
    }
    return fd;
  }
  // Parse a consumed body back to a FormData for `.formData()`:
  // application/x-www-form-urlencoded or multipart/form-data.
  function parseFormData(text, contentType) {
    var ct = String(contentType || '').toLowerCase();
    if (ct.indexOf('multipart/form-data') === 0) {
      var bM = /boundary=("?)([^";]+)\1/i.exec(String(contentType));
      if (!bM) throw new TypeError("multipart/form-data without boundary");
      return parseMultipart(text, bM[2]);
    }
    if (ct.indexOf('application/x-www-form-urlencoded') === 0 || ct === '') {
      var fd = new FormData();
      new URLSearchParams(text).forEach(function(val, key) { fd.append(key, val); });
      return fd;
    }
    throw new TypeError("Unsupported content-type for formData(): " + ct);
  }

  // Convert only the final public result into a promise. Intermediate reads
  // use trusted callbacks; null bodies stay undisturbed and can be read again.
  var bodyMixin = Core.record();
  bodyMixin.text = function text() {
    return consumeBody(this, utf8Decode);
  };
  bodyMixin.json = function json() {
    return consumeBody(this, function (bytes) { return Core.call(nativeJsonParse, undefined, [utf8Decode(bytes)]); });
  };
  bodyMixin.arrayBuffer = function arrayBuffer() {
    return consumeBody(this, function (bytes) { return Core.viewInfo(bytes).buffer; });
  };
  bodyMixin.bytes = function bytes() {
    return consumeBody(this, function (bytes) { return bytes; });
  };
  bodyMixin.blob = function blob() {
    var self = this;
    return consumeBody(self, function (bytes) {
      var options = Core.record(); options.type = self.headers.get('content-type') || '';
      return new Blob([bytes], options);
    });
  };
  bodyMixin.formData = function formData() {
    var self = this;
    return consumeBody(self, function (bytes) { return parseFormData(utf8Decode(bytes), self.headers.get('content-type')); });
  };
  function installBodyMixin(prototype) {
    var names = ['text', 'json', 'arrayBuffer', 'bytes', 'blob', 'formData'];
    for (var i = 0; i < names.length; ++i) Core.define(prototype, names[i], {
      value: bodyMixin[names[i]], enumerable: true, configurable: true, writable: true
    });
    Core.define(prototype, 'body', {
      get: function () { return bodyStream(this); }, enumerable: true, configurable: true
    });
    Core.define(prototype, 'bodyUsed', {
      get: function () { return bodyUsed(this); }, enumerable: true, configurable: true
    });
  }

  // ---- AbortController / AbortSignal ----
  function AbortSignal() { throw new TypeError("Illegal constructor"); }
  AbortSignal.prototype = Core.create(NativeEventTarget.prototype);
  var signalPrototype = AbortSignal.prototype;
  Core.define(signalPrototype, 'constructor', { value: AbortSignal, configurable: true, writable: true });
  function signalRecord(signal) {
    var state = Core.get(signalSlots, signal);
    if (state === undefined) throw new TypeError('Illegal AbortSignal receiver');
    return state;
  }
  function isSignal(signal) { return Core.has(signalSlots, signal); }
  function signalAborted(signal) { return signalRecord(signal).aborted; }
  function signalReason(signal) { return signalRecord(signal).reason; }
  Core.define(signalPrototype, 'aborted', { get: function () { return signalAborted(this); }, enumerable: true, configurable: true });
  Core.define(signalPrototype, 'reason', { get: function () { return signalReason(this); }, enumerable: true, configurable: true });
  Core.define(signalPrototype, 'onabort', {
    get: function () { return signalRecord(this).onabort; },
    set: function (value) { signalRecord(this).onabort = typeof value === 'function' ? value : null; },
    enumerable: true, configurable: true
  });
  Core.define(signalPrototype, 'throwIfAborted', {
    value: function throwIfAborted() { var state = signalRecord(this); if (state.aborted) throw state.reason; },
    enumerable: true, configurable: true, writable: true
  });
  Core.define(signalPrototype, Core.Symbol.toStringTag, { value: 'AbortSignal', configurable: true });
  function makeSignal() {
    var s = Core.create(signalPrototype), state = Core.record();
    Core.call(NativeEventTarget, s, []);
    state.aborted = false; state.reason = undefined; state.onabort = null;
    state.algorithms = Core.queue(); state.dependents = Core.queue();
    Core.set(signalSlots, s, state);
    return s;
  }
  function abortReason(reason) {
    return reason !== undefined ? reason : new NativeDOMException("signal is aborted without reason", "AbortError");
  }
  function addAbortAlgorithm(signal, algorithm) {
    var state = signalRecord(signal), token = Core.record();
    token.algorithm = algorithm; token.active = !state.aborted;
    if (token.active) Core.push(state.algorithms, token);
    return token;
  }
  function removeAbortAlgorithm(signal, token) {
    signalRecord(signal);
    if (token !== undefined) { token.active = false; token.algorithm = undefined; }
  }
  function signalAbort(signal, reason) {
    var initial = signalRecord(signal);
    if (initial.aborted) return;
    var pending = Core.queue(), all = Core.queue(), actualReason = abortReason(reason);
    Core.push(pending, signal);
    // All dependent states change before the source's author listeners run.
    while (Core.length(pending)) {
      var current = Core.shift(pending), state = signalRecord(current);
      if (state.aborted) continue;
      state.aborted = true; state.reason = actualReason; Core.push(all, current);
      while (Core.length(state.dependents)) {
        var ref = Core.shift(state.dependents);
        var child = NativeWeakRef === undefined ? ref : Core.call(nativeDeref, ref, []);
        if (child !== undefined) Core.push(pending, child);
      }
    }
    while (Core.length(all)) {
      var target = Core.shift(all), targetState = signalRecord(target);
      while (Core.length(targetState.algorithms)) {
        var token = Core.shift(targetState.algorithms);
        if (token.active) {
          var algorithm = token.algorithm; token.active = false; token.algorithm = undefined;
          Core.call(algorithm, undefined, []);
        }
      }
      // Initialize without an author prototype; the existing Event constructor
      // assigns type and would otherwise hit the C3 inherited setter.
      var event = Core.record(); Core.call(NativeEvent, event, ['abort', Core.record()]);
      event.isTrusted = true;
      Core.setPrototypeOf(event, nativeEventPrototype);
      if (targetState.onabort !== null) { try { Core.call(targetState.onabort, target, [event]); } catch (_) {} }
      Core.call(nativeDispatch, target, [event]);
    }
  }
  // A fresh signal that follows its source signals: aborts (with the source's
  // reason) when any source aborts. A Request's signal and a clone's signal are
  // new dependent signals (WHATWG), never the same object as the input's.
  function dependentSignal(sources) {
    var s = makeSignal();
    for (var i = 0; i < sources.length; i++) {
      var src = sources[i];
      if (!isSignal(src)) throw new TypeError('Expected an AbortSignal');
      var sourceState = signalRecord(src);
      if (sourceState.aborted) {
        var state = signalRecord(s); state.aborted = true; state.reason = sourceState.reason; break;
      }
      Core.push(sourceState.dependents, NativeWeakRef === undefined ? s : new NativeWeakRef(s));
    }
    return s;
  }
  AbortSignal.abort = function(reason) {
    var s = makeSignal(), state = signalRecord(s); state.aborted = true; state.reason = abortReason(reason); return s;
  };
  AbortSignal.timeout = function(ms) {
    ms = +ms;
    if (!(ms >= 0) || ms > 9007199254740991 || ms % 1 !== 0) throw new RangeError('Invalid abort timeout');
    var s = makeSignal();
    Core.call(nativeSetTimeout, globalThis, [function() { signalAbort(s, new NativeDOMException("signal timed out", "TimeoutError")); }, ms]);
    return s;
  };
  AbortSignal.any = function(signals) {
    var sources = [];
    for (var signal of signals) {
      if (!isSignal(signal)) throw new TypeError('Expected an AbortSignal');
      sources[sources.length] = signal;
    }
    return dependentSignal(sources);
  };
  defineInterface('AbortSignal', AbortSignal, 0);

  function AbortController() {
    if (!new.target) throw new TypeError('AbortController requires new');
    Core.set(controllerSlots, this, makeSignal());
  }
  function controllerSignal(controller) {
    if (!Core.has(controllerSlots, controller)) throw new TypeError('Illegal AbortController receiver');
    return Core.get(controllerSlots, controller);
  }
  Core.define(AbortController.prototype, 'signal', {
    get: function () { return controllerSignal(this); }, enumerable: true, configurable: true
  });
  Core.define(AbortController.prototype, 'abort', {
    value: function abort(reason) { signalAbort(controllerSignal(this), reason); },
    enumerable: true, configurable: true, writable: true
  });
  Core.define(AbortController.prototype.abort, 'length', { value: 0, configurable: true });
  Core.define(AbortController.prototype, Core.Symbol.toStringTag, { value: 'AbortController', configurable: true });
  defineInterface('AbortController', AbortController, 0);
  Core.signal.makeSignal = makeSignal; Core.signal.abort = signalAbort;
  Core.signal.isSignal = isSignal; Core.signal.isAborted = signalAborted;
  Core.signal.reason = signalReason; Core.signal.addAbortAlgorithm = addAbortAlgorithm;
  Core.signal.removeAbortAlgorithm = removeAbortAlgorithm;

  // ---- Request init validation (WHATWG) ----
  var NORMALIZE_METHODS = { DELETE: 1, GET: 1, HEAD: 1, OPTIONS: 1, POST: 1, PUT: 1 };
  var FORBIDDEN_METHODS = { CONNECT: 1, TRACE: 1, TRACK: 1 };
  var SIMPLE_METHODS = { GET: 1, HEAD: 1, POST: 1 };
  function normalizeMethod(m) {
    m = String(m);
    if (!TOKEN_RE.test(m)) throw new TypeError("Invalid method: '" + m + "'");
    var up = m.toUpperCase();
    if (FORBIDDEN_METHODS[up]) throw new TypeError("Forbidden method: " + m);
    return NORMALIZE_METHODS[up] ? up : m;
  }
  var ENUMS = {
    mode: { 'same-origin': 1, 'no-cors': 1, 'cors': 1, 'navigate': 1 },
    credentials: { 'omit': 1, 'same-origin': 1, 'include': 1 },
    cache: { 'default': 1, 'no-store': 1, 'reload': 1, 'no-cache': 1, 'force-cache': 1, 'only-if-cached': 1 },
    redirect: { 'follow': 1, 'error': 1, 'manual': 1 },
    referrerPolicy: {
      '': 1, 'no-referrer': 1, 'no-referrer-when-downgrade': 1, 'same-origin': 1, 'origin': 1,
      'strict-origin': 1, 'origin-when-cross-origin': 1, 'strict-origin-when-cross-origin': 1, 'unsafe-url': 1
    }
  };
  function checkEnum(kind, v) {
    var s = String(v);
    if (!ENUMS[kind][s]) throw new TypeError("Invalid " + kind + ": '" + s + "'");
    return s;
  }

  // ---- Request ----
  function Request(input, init) {
    if (!new.target) throw new TypeError("Failed to construct 'Request': use 'new'");
    init = init || {};
    var inputBody = Core.get(bodySlots, input);
    if (inputBody !== undefined && inputBody.kind !== 'request') inputBody = undefined;
    var requestSignal, bytes = null, stream = null;
    if (init.window !== undefined && init.window !== null) throw new TypeError("RequestInit window must be null");
    if (inputBody !== undefined) {
      this.url = input.url; this.method = input.method; this.headers = new Headers(input.headers);
      this.mode = input.mode; this.credentials = input.credentials;
      this.redirect = input.redirect; this.cache = input.cache; this.destination = input.destination;
      this.referrer = input.referrer; this.referrerPolicy = input.referrerPolicy; this.integrity = input.integrity;
      this.keepalive = input.keepalive;
    } else {
      // Resolve leniently (relative URLs resolve at fetch when there is no base),
      // then validate: a resolved absolute URL must not carry credentials; an
      // unresolvable URL is an error only when a real document base exists (with no
      // base, a relative URL legitimately stays unresolved).
      this.url = __resolve_url(String(input));
      var pj = __url_parse(this.url, "");
      if (pj) {
        var pc = JSON.parse(pj);
        if (pc.username || pc.password) throw new TypeError("Request URL cannot have credentials");
        this.url = pc.href;
      } else if (typeof location !== 'undefined' && location && location.href && location.href !== 'about:blank') {
        throw new TypeError("Failed to construct 'Request': invalid URL");
      }
      this.method = 'GET'; this.headers = new Headers();
      this.mode = 'cors'; this.credentials = 'same-origin'; this.redirect = 'follow'; this.cache = 'default'; this.destination = '';
      // Default referrer is the client (the document URL, resolved at fetch).
      this.referrer = 'about:client'; this.referrerPolicy = ''; this.integrity = '';
      this.keepalive = false;
    }
    // The request's signal is a fresh dependent signal following a single source
    // (WHATWG): init.signal if present (even null removes it), else the input
    // request's signal — never a shared reference.
    var __sigSource = (init.signal !== undefined) ? init.signal
                    : inputBody !== undefined ? inputBody.signal : null;
    requestSignal = dependentSignal(__sigSource !== null ? [__sigSource] : []);
    if (init.method !== undefined) this.method = normalizeMethod(init.method);
    if (init.mode !== undefined) { if (String(init.mode) === 'navigate') throw new TypeError("Cannot construct a Request with mode 'navigate'"); this.mode = checkEnum('mode', init.mode); }
    if (init.credentials !== undefined) this.credentials = checkEnum('credentials', init.credentials);
    if (init.cache !== undefined) this.cache = checkEnum('cache', init.cache);
    if (init.redirect !== undefined) this.redirect = checkEnum('redirect', init.redirect);
    if (init.referrerPolicy !== undefined) this.referrerPolicy = checkEnum('referrerPolicy', init.referrerPolicy);
    if (init.integrity !== undefined) this.integrity = String(init.integrity);
    if (init.keepalive !== undefined) this.keepalive = !!init.keepalive;
    var duplex = init.duplex;
    if (duplex !== undefined && String(duplex) !== 'half') throw new TypeError('Invalid duplex');
    // referrer: "" = no referrer; "about:client" = default (the document); else a URL.
    if (init.referrer !== undefined) this.referrer = String(init.referrer);
    // no-cors restricts the method to GET/HEAD/POST.
    if (this.mode === 'no-cors' && !SIMPLE_METHODS[this.method]) throw new TypeError("Method '" + this.method + "' not allowed in no-cors mode");
    // only-if-cached requires same-origin mode.
    if (this.cache === 'only-if-cached' && this.mode !== 'same-origin') throw new TypeError("only-if-cached requires same-origin mode");
    if (init.headers !== undefined) this.headers = new Headers(init.headers);
    var bodyInput = init.body;
    if (bodyInput !== undefined && bodyInput !== null) {
      // A ReadableStream body is an upload stream; it requires duplex: "half".
      if (ReadableOps.isReadable(bodyInput) && duplex === undefined)
        throw new TypeError("Request with a ReadableStream body requires 'duplex: \"half\"'");
      var eb = extractBody(bodyInput);
      bytes = eb.bytes; stream = eb.stream;
      if (eb.type && !this.headers.has('content-type')) this.headers.set('content-type', eb.type);
    } else if (inputBody !== undefined) {
      bytes = inputBody.bytes; stream = inputBody.stream;
    }
    if (stream !== null && bytes === null) {
      if (this.keepalive) throw new TypeError('A stream upload cannot use keepalive');
      if (this.mode !== 'same-origin' && this.mode !== 'cors') throw new TypeError('A stream upload requires same-origin or cors mode');
    }
    // The request header guard (from the mode) drops forbidden / non-safelisted
    // headers and governs later append/set/delete.
    this.headers._guard = (this.mode === 'no-cors') ? 'request-no-cors' : 'request';
    var __g = this.headers._guard, __hs = this.headers;
    __hs._h = __hs._h.filter(function(p) {
      try { return guardAllows(__g, p[0], p[1]); } catch (e) { return false; }
    });
    if ((this.method === 'GET' || this.method === 'HEAD') && (bytes !== null || stream !== null))
      throw new TypeError("Request with GET/HEAD method cannot have body.");
    if (inputBody !== undefined && (bodyInput === undefined || bodyInput === null) && stream !== null) {
      if (bodyUnusable(inputBody)) throw new TypeError('Input request body is disturbed or locked');
      stream = PipingOps.createProxy(stream);
    }
    var body = setBody(this, 'request', bytes, stream); body.signal = requestSignal;
  }
  var requestPrototype = Request.prototype;
  Request.prototype.clone = function() {
    var original = bodyRecord(this);
    if (original.kind !== 'request') throw new TypeError('Illegal Request receiver');
    var r = Core.create(requestPrototype);
    var fields = ['url', 'method', 'mode', 'credentials', 'redirect', 'cache', 'destination',
      'referrer', 'referrerPolicy', 'integrity', 'keepalive'];
    for (var i = 0; i < fields.length; ++i) Core.define(r, fields[i], {
      value: this[fields[i]], enumerable: true, configurable: true, writable: true
    });
    r.headers = new Headers(this.headers); r.headers._guard = this.headers._guard;
    cloneBodyInto(this, r);
    bodyRecord(r).signal = dependentSignal([original.signal]);
    return r;
  };
  installBodyMixin(requestPrototype);
  Core.define(requestPrototype, 'signal', { get: function () {
    var b = bodyRecord(this); if (b.kind !== 'request') throw new TypeError('Illegal Request receiver'); return b.signal;
  }, enumerable: true, configurable: true });
  Core.define(requestPrototype, 'duplex', { get: function () {
    var b = bodyRecord(this); if (b.kind !== 'request') throw new TypeError('Illegal Request receiver'); return 'half';
  }, enumerable: true, configurable: true });
  globalThis.Request = Request;

  // ---- Response ----
  function Response(body, init) {
    if (!new.target) throw new TypeError('Response requires new');
    init = init || {};
    var status = (init.status !== undefined) ? (init.status | 0) : 200;
    if (status < 200 || status > 599) throw new RangeError("Response status " + status + " out of range");
    this.status = status;
    this.statusText = (init.statusText !== undefined) ? String(init.statusText) : "";
    this.ok = this.status >= 200 && this.status < 300;
    Core.define(this, 'type', { value: 'default', writable: true, enumerable: true, configurable: true });
    this.url = ""; this.redirected = false;
    this.headers = new Headers(init.headers);
    // Response guard: a script-built Response cannot carry set-cookie / set-cookie2
    // (a network Response sets its headers with the guard bypassed; see
    // responseFromOutcome). Filter the init headers, then govern later writes.
    this.headers._h = this.headers._h.filter(function(p) { return !isForbiddenResponseHeader(p[0]); });
    this.headers._guard = 'response';
    var bytes = null, stream = null;
    if (body != null) {
      if (isNullBodyStatus(status)) throw new TypeError('A null-body response status cannot have a body');
      var eb = extractBody(body);
      bytes = eb.bytes; stream = eb.stream;
      if (eb.type && !this.headers.has('content-type')) this.headers.set('content-type', eb.type);
    }
    setBody(this, 'response', bytes, stream);
  }
  var responsePrototype = Response.prototype;
  installBodyMixin(responsePrototype);
  Response.prototype.clone = function() {
    if (bodyRecord(this).kind !== 'response') throw new TypeError('Illegal Response receiver');
    var r = Core.create(responsePrototype);
    r.status = this.status; r.statusText = this.statusText; r.ok = this.ok;
    Core.define(r, 'type', { value: this.type, writable: true, enumerable: true, configurable: true });
    r.url = this.url; r.redirected = this.redirected;
    r.headers = new Headers(this.headers); r.headers._guard = this.headers._guard;
    cloneBodyInto(this, r);
    return r;
  };
  Response.error = function() {
    var r = new Response(null, { status: 200 });
    r.status = 0; r.ok = false; r.type = "error"; r.headers._guard = 'immutable'; return r;
  };
  Response.redirect = function(url, status) {
    status = (status === undefined) ? 302 : (status | 0);
    if ([301, 302, 303, 307, 308].indexOf(status) === -1) throw new RangeError("Invalid redirect status " + status);
    var r = new Response(null, { status: status });
    r.headers.set("location", String(url));
    return r;
  };
  Response.json = function(data, init) {
    init = init || {};
    var r = new Response(JSON.stringify(data), init);
    // application/json wins unless the caller's init explicitly set a type (the
    // string-body extraction defaults to text/plain, which must not stick here).
    if (!new Headers(init.headers).has("content-type")) r.headers.set("content-type", "application/json");
    return r;
  };
  globalThis.Response = Response;

  // Build a Response from the native __fetch outcome. The body crossed as a binary
  // string; decode it straight to bytes (bypassing extractBody, which would treat
  // it as text) so binary responses are exact.
  // A status-0 (opaqueredirect / opaque) filtered response cannot go through the
  // Response ctor, which rejects < 200. Build a 200 placeholder then override —
  // the same trick as Response.error. `o.status || 200` would also mis-map 0 to
  // 200 (0 is falsy), so route every outcome through here.
  function makeFilteredShell(o) {
    var st = (o.status == null) ? 200 : (o.status | 0);
    if (st >= 200 && st <= 599) {
      return new Response(null, { status: st, statusText: o.statusText || "" });
    }
    var r = new Response(null, { status: 200 });
    r.status = st; r.statusText = o.statusText || ""; r.ok = st >= 200 && st < 300;
    return r;
  }
  // A null-body status (WHATWG): the response's body is null regardless of any
  // bytes on the wire, so Response.body reads as null.
  function isNullBodyStatus(s) { return s === 204 || s === 205 || s === 304; }
  function responseFromOutcome(o) {
    var r = makeFilteredShell(o);
    // Network headers are set with the guard bypassed (a real response keeps
    // set-cookie, readable via getSetCookie); the guard only blocks later writes.
    r.headers = new Headers(o.headers); r.headers._guard = 'response';
    setBody(r, 'response', isNullBodyStatus(r.status) ? null : binaryStringToBytes(o.body != null ? o.body : ""), null);
    r.type = o.type || "default"; r.url = o.url || ""; r.redirected = !!o.redirected; return r;
  }
  function headersFlat(h) {
    var flat = [];
    for (var i = 0; i < h._h.length; i++) { flat.push(h._h[i][0]); flat.push(h._h[i][1]); }
    return flat.join("\n");
  }

  // ---- Deferred fetch registry ----
  // Each in-flight fetch() owns an entry keyed by a monotonic id. A synchronous
  // host answers inline (__fetch_start returns the outcome JSON, resolved in this
  // tick). A deferred host returns "" and the entry stays pending until Rust drives
  // __fetchSettle / __fetchFail (or __fetchPushChunk / __fetchClose for streams).
  // Single authority, delete-once: every terminal removes the entry exactly once.
  var __pending = Object.create(null);
  globalThis.__pending = __pending;          // Object.keys count drives the host's quiescence probe
  var fetchIds = globalThis.__agentTimers;
  if (!fetchIds.nextFetchId) fetchIds.nextFetchId = 1;

  function finishPull(entry) {
    if (entry.pull !== undefined) {
      var pull = entry.pull; entry.pull = undefined; pull.resolve(undefined);
    }
    entry.awaiting = false;
  }
  function removeEntry(id, entry) {
    delete __pending[id];
    removeAbortAlgorithm(entry.signal, entry.abortToken);
    if (entry.response !== null) bodyRecord(entry.response).nativeLive = false;
  }
  function settleEntry(e, o) {
    if (o.networkError) e.reject(new TypeError('Failed to fetch'));
    else {
      var response = responseFromOutcome(o);
      if (e.method === 'HEAD') setBody(response, 'response', null, null);
      e.resolve(response);
    }
  }

  globalThis.fetch = function(input, init) {
    var req;
    try { req = new Request(input, init); } catch (e) { return Core.reject(e); }
    if (!req.headers.has('accept')) req.headers.append('accept', '*/*');
    if (!req.headers.has('accept-language')) req.headers.append('accept-language', '*');
    var body = bodyRecord(req), signal = body.signal;
    if (signalAborted(signal)) {
      var pre = signalReason(signal);
      if (body.stream !== null) Core.handled(ReadableOps.cancel(body.stream, pre));
      return Core.reject(pre);
    }
    var id = fetchIds.nextFetchId++, result = Core.deferred(), entry = Core.record();
    entry.resolve = result.resolve; entry.reject = result.reject;
    entry.signal = signal; entry.stream = null; entry.response = null;
    entry.pull = undefined; entry.uploadReader = undefined;
    entry.settled = false; entry.started = false; entry.awaiting = false; entry.method = req.method;
    __pending[id] = entry;
    entry.abortToken = addAbortAlgorithm(signal, function () {
      if (__pending[id] !== entry) return;
      var reason = signalReason(signal);
      removeEntry(id, entry);
      if (entry.started) { try { Core.call(nativeFetchAbort, undefined, [id]); } catch (_) {} }
      if (entry.stream !== null) ReadableOps.error(entry.stream, reason);
      finishPull(entry);
      if (!entry.settled) { entry.settled = true; entry.reject(reason); }
      if (entry.uploadReader !== undefined) Core.handled(ReadableOps.readerCancel(entry.uploadReader, reason));
    });
    // The native seam still accepts one complete upload. Drain the actual body
    // through private readers before registering work with that seam.
    collectBody(req, function (bytes) {
      entry.uploadReader = undefined;
      if (__pending[id] !== entry) return;
      try {
        var docHref = (typeof location !== 'undefined' && location && location.href) ? location.href : '';
        var referrer = req.referrer;
        if (referrer === undefined || referrer === 'about:client') referrer = docHref;
        else if (referrer !== '') {
          try { referrer = new URL(referrer, docHref || undefined).href; } catch (_) { referrer = ''; }
        }
        entry.started = true;
        var inline = Core.call(nativeFetchStart, undefined, [id, req.method, req.url, headersFlat(req.headers),
          bytesToBinaryString(bytes), req.cache || 'default', req.redirect || 'follow',
          req.mode || 'cors', referrer, req.referrerPolicy || '',
          req.credentials || 'same-origin', req.integrity || '', body.stream !== null ? '1' : '0']);
        if (inline && __pending[id] === entry && !entry.settled) {
          entry.settled = true; removeEntry(id, entry);
          settleEntry(entry, Core.call(nativeJsonParse, undefined, [inline]));
        }
      } catch (e) {
        if (__pending[id] === entry) { removeEntry(id, entry); entry.settled = true; entry.reject(e); }
      }
    }, function (reason) {
      if (__pending[id] === entry) { removeEntry(id, entry); entry.settled = true; entry.reject(reason); }
    }, function (reader) { entry.uploadReader = reader; });
    return result.promise;
  };

  // Rust invokes these existing hooks. Response delivery remains headers-first
  // and demand-driven; Streams changes the private controller implementation.
  globalThis.__fetchSettle = function(id, ojson) {
    var e = __pending[id]; if (!e || e.settled) return;
    e.settled = true; removeEntry(id, e);
    settleEntry(e, Core.call(nativeJsonParse, undefined, [ojson]));
  };
  globalThis.__fetchFail = function(id, msg) {
    var e = __pending[id]; if (!e || e.settled) return;
    e.settled = true; removeEntry(id, e); e.reject(new TypeError(msg || 'Failed to fetch'));
  };
  globalThis.__fetchStartStream = function(id, ojson) {
    var e = __pending[id]; if (!e || e.settled) return; e.settled = true;
    var o = Core.call(nativeJsonParse, undefined, [ojson]);
    if (o.networkError) { removeEntry(id, e); e.reject(new TypeError('Failed to fetch')); return; }
    var r = makeFilteredShell(o);
    r.headers = new Headers(o.headers); r.headers._guard = 'response';
    r.type = o.type || 'default'; r.url = o.url || ''; r.redirected = !!o.redirected;
    if (isNullBodyStatus(r.status) || e.method === 'HEAD') {
      setBody(r, 'response', null, null);
    } else {
      var algorithms = Core.record();
      algorithms.pull = function () {
        var pe = __pending[id];
        if (pe !== e) return Core.resolve(undefined);
        if (pe.pull === undefined) {
          pe.pull = Core.deferred(); pe.awaiting = true;
          var pending = pe.pull.promise;
          try { Core.call(nativeFetchPull, undefined, [id]); }
          catch (reason) { ReadableOps.error(pe.stream, reason); removeEntry(id, pe); finishPull(pe); }
          return pending;
        }
        return pe.pull.promise;
      };
      algorithms.cancel = function () {
        if (__pending[id] === e) {
          removeEntry(id, e); finishPull(e); Core.call(nativeFetchAbort, undefined, [id]);
        }
        return Core.resolve(undefined);
      };
      e.stream = ReadableOps.createByte(algorithms, byteStrategy());
      var body = setBody(r, 'response', null, e.stream); body.nativeLive = true;
    }
    e.response = r; e.resolve(r);
  };
  globalThis.__fetchPushChunk = function(id, arr) {
    var e = __pending[id]; if (!e || e.stream === null) return;
    var u8 = new Uint8Array(arr.length);
    for (var i = 0; i < arr.length; i++) u8[i] = arr[i] & 0xFF;
    try { if (arr.length !== 0) ReadableOps.enqueue(e.stream, u8); }
    finally { finishPull(e); }
  };
  globalThis.__fetchClose = function(id) {
    var e = __pending[id]; if (!e) return;
    removeEntry(id, e);
    if (e.stream !== null) ReadableOps.close(e.stream);
    finishPull(e);
  };
  globalThis.__fetchError = function(id) {
    var e = __pending[id]; if (!e) return;
    removeEntry(id, e);
    if (e.stream !== null) ReadableOps.error(e.stream, new TypeError('Failed to read response body'));
    finishPull(e);
  };

  // ---- ProgressEvent ----
  function ProgressEvent(type, init) {
    if (arguments.length < 1) throw new TypeError("ProgressEvent requires a type");
    Event.call(this, String(type), init);
    init = init || {};
    this.lengthComputable = !!init.lengthComputable;
    this.loaded = init.loaded === undefined ? 0 : Number(init.loaded);
    this.total = init.total === undefined ? 0 : Number(init.total);
  }
  ProgressEvent.prototype = Object.create(Event.prototype);
  ProgressEvent.prototype.constructor = ProgressEvent;
  // WebIDL shape: an interface object is a non-enumerable global property (a
  // `for (p in window)` must not see it), its `prototype` is non-writable, and
  // its `length` counts only the required arguments.
  function defineInterface(name, ctor, len) {
    Object.defineProperty(ctor, 'prototype', { writable: false });
    Object.defineProperty(ctor, 'length', { value: len, configurable: true });
    Object.defineProperty(globalThis, name, {
      value: ctor, writable: true, enumerable: false, configurable: true
    });
  }
  defineInterface('ProgressEvent', ProgressEvent, 1);

  // ---- XMLHttpRequest ----
  // A state machine over the same fetch seam, not a second network path: an async
  // send runs through globalThis.fetch (so abort, streaming and the host mailbox
  // come free) and a sync send through the blocking __fetch_sync sink.

  // An `on<type>` IDL attribute. The forwarding listener is registered once, at
  // first assignment, so reassigning the handler keeps its position in the
  // listener list (the event-order tests depend on that).
  function defineHandler(proto, type) {
    var slot = '__on_' + type;
    Object.defineProperty(proto, 'on' + type, {
      configurable: true, enumerable: true,
      get: function() { return this[slot] || null; },
      set: function(v) {
        var fn = (typeof v === 'function') ? v : null;
        if (!this.__hooked) this.__hooked = {};
        if (!this.__hooked[type]) {
          this.__hooked[type] = true;
          var self = this;
          this.addEventListener(type, function(ev) { var h = self[slot]; if (h) h.call(self, ev); });
        }
        this[slot] = fn;
      }
    });
  }
  var XHR_EVENTS = ['loadstart', 'progress', 'abort', 'error', 'load', 'timeout', 'loadend'];

  function XMLHttpRequestEventTarget() { EventTarget.call(this); }
  XMLHttpRequestEventTarget.prototype = Object.create(EventTarget.prototype);
  XMLHttpRequestEventTarget.prototype.constructor = XMLHttpRequestEventTarget;
  for (var xe = 0; xe < XHR_EVENTS.length; xe++) defineHandler(XMLHttpRequestEventTarget.prototype, XHR_EVENTS[xe]);
  defineInterface('XMLHttpRequestEventTarget', XMLHttpRequestEventTarget, 0);

  function XMLHttpRequestUpload() { XMLHttpRequestEventTarget.call(this); }
  XMLHttpRequestUpload.prototype = Object.create(XMLHttpRequestEventTarget.prototype);
  XMLHttpRequestUpload.prototype.constructor = XMLHttpRequestUpload;
  defineInterface('XMLHttpRequestUpload', XMLHttpRequestUpload, 0);

  var XU = 0, XO = 1, XH = 2, XL = 3, XD = 4;   // UNSENT / OPENED / HEADERS_RECEIVED / LOADING / DONE
  var XHR_UPPER = { DELETE: 1, GET: 1, HEAD: 1, OPTIONS: 1, POST: 1, PUT: 1 };
  var XHR_BAD_METHOD = { CONNECT: 1, TRACE: 1, TRACK: 1 };
  var XHR_TYPES = { '': 1, 'arraybuffer': 1, 'blob': 1, 'document': 1, 'json': 1, 'text': 1 };
  // Labels this tier decodes as one byte per code point. Everything else falls
  // back to UTF-8; there is no encoding registry in the scripted tier.
  var XHR_LATIN1 = { 'windows-1252': 1, 'iso-8859-1': 1, 'latin1': 1, 'us-ascii': 1, 'ascii': 1 };

  function xhrErr(name, msg) { return new DOMException(msg || name, name); }
  function xhrDocHref() {
    return (typeof location !== 'undefined' && location && location.href) ? location.href : '';
  }
  // Resolve against the document base. `null` = a genuinely invalid URL; with no
  // real base (disk mode) a relative URL legitimately stays unresolved.
  function xhrResolve(input) {
    var href = __resolve_url(input);
    var pj = __url_parse(href, "");
    if (pj) return JSON.parse(pj).href;
    var doc = xhrDocHref();
    return (doc && doc !== 'about:blank') ? null : href;
  }
  // Split a MIME type into its lower-cased essence and charset parameter.
  function parseMime(s) {
    s = String(s).replace(/^[\t\n\r ]+|[\t\n\r ]+$/g, "");
    var semi = s.indexOf(';');
    var essence = (semi < 0 ? s : s.slice(0, semi)).replace(/[\t\n\r ]+$/, "");
    if (!/^[!#$%&'*+\-.^_`|~0-9A-Za-z]+\/[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/.test(essence)) return null;
    var cm = /;[\t ]*charset[\t ]*=[\t ]*("?)([^";]*)\1/i.exec(s);
    return { essence: essence.toLowerCase(), charset: cm ? cm[2].toLowerCase() : '' };
  }
  function latin1Decode(b) {
    var out = '', CH = 0x8000;
    for (var i = 0; i < b.length; i += CH) out += String.fromCharCode.apply(null, b.subarray(i, i + CH));
    return out;
  }
  function hasAnyListener(t) {
    var L = t.__listeners;
    if (!L) return false;
    for (var k in L) { if (L[k] && L[k].length) return true; }
    return false;
  }
  function xhrFire(target, type) { target.dispatchEvent(new Event(type)); }
  function xhrProgress(target, type, loaded, total, computable) {
    target.dispatchEvent(new ProgressEvent(type, {
      lengthComputable: !!computable, loaded: loaded || 0, total: total || 0
    }));
  }
  function xhrClearTimer(x) { if (x._timer !== null) { clearTimeout(x._timer); x._timer = null; } }
  function xhrArmTimeout(x) {
    xhrClearTimer(x);
    if (!x._timeout || !x._sendFlag) return;
    var rem = x._timeout - (Date.now() - x._sendTime);
    if (rem < 0) rem = 0;
    x._timer = setTimeout(function() {
      if (!x._sendFlag) return;
      x._timedOut = true;
      x._terminate();
      xhrRequestError(x, 'timeout', false);
    }, rem);
  }
  // The spec's request error steps. `allowThrow` is set only on the synchronous
  // send path, where an error throws instead of firing events.
  function xhrRequestError(x, type, allowThrow) {
    x._state = XD;
    x._sendFlag = false;
    x._resp = null; x._bytes = null; x._respObj = undefined; x._respDoc = undefined;
    xhrClearTimer(x);
    if (x._sync && allowThrow) {
      throw xhrErr(type === 'timeout' ? 'TimeoutError' : type === 'abort' ? 'AbortError' : 'NetworkError',
                   'XMLHttpRequest ' + type);
    }
    xhrFire(x, 'readystatechange');
    if (!x._uploadComplete) {
      x._uploadComplete = true;
      if (x._uploadListener) { xhrProgress(x.upload, type, 0, 0, false); xhrProgress(x.upload, 'loadend', 0, 0, false); }
    }
    xhrProgress(x, type, 0, 0, false);
    xhrProgress(x, 'loadend', 0, 0, false);
  }

  function XMLHttpRequest() {
    if (!(this instanceof XMLHttpRequest)) throw new TypeError("Failed to construct 'XMLHttpRequest': use 'new'");
    XMLHttpRequestEventTarget.call(this);
    this._state = XU;
    this._timeout = 0; this._sendTime = 0; this._timer = null;
    this._withCred = false; this._respType = ''; this._sync = false;
    this._sendFlag = false; this._uploadListener = false; this._uploadComplete = false;
    this._timedOut = false;
    this._method = ''; this._url = ''; this._reqHeaders = []; this._override = null;
    this._resp = null; this._bytes = null; this._respObj = undefined; this._respDoc = undefined;
    this._ctl = null;
    this.upload = new XMLHttpRequestUpload();
  }
  XMLHttpRequest.prototype = Object.create(XMLHttpRequestEventTarget.prototype);
  XMLHttpRequest.prototype.constructor = XMLHttpRequest;
  defineHandler(XMLHttpRequest.prototype, 'readystatechange');

  XMLHttpRequest.prototype._terminate = function() {
    xhrClearTimer(this);
    if (this._ctl) { var c = this._ctl; this._ctl = null; try { c.abort(); } catch (e) {} }
  };

  XMLHttpRequest.prototype.open = function(method, url, async, username, password) {
    if (arguments.length < 2) throw new TypeError("open requires a method and a URL");
    method = String(method);
    if (!TOKEN_RE.test(method)) throw xhrErr('SyntaxError', "Invalid method '" + method + "'");
    var up = method.toUpperCase();
    if (XHR_BAD_METHOD[up]) throw xhrErr('SecurityError', "Method '" + method + "' is not allowed");
    if (XHR_UPPER[up]) method = up;
    var href = xhrResolve(String(url));
    if (href === null) throw xhrErr('SyntaxError', "Invalid URL");
    var isAsync = (async === undefined) ? true : !!async;
    if (!isAsync && (this._timeout !== 0 || this._respType !== ''))
      throw xhrErr('InvalidAccessError', "A synchronous request cannot use timeout or responseType");
    this._terminate();
    this._sync = !isAsync;
    this._method = method; this._url = href;
    this._reqHeaders = []; this._override = null;
    this._sendFlag = false; this._uploadListener = false; this._uploadComplete = false;
    this._resp = null; this._bytes = null; this._respObj = undefined; this._respDoc = undefined;
    this._timedOut = false;
    // Only a state that was not already OPENED fires readystatechange.
    if (this._state !== XO) { this._state = XO; xhrFire(this, 'readystatechange'); }
  };

  XMLHttpRequest.prototype.setRequestHeader = function(name, value) {
    if (this._state !== XO || this._sendFlag)
      throw xhrErr('InvalidStateError', "setRequestHeader: the request is not OPENED");
    name = String(name);
    var v = String(value).replace(/^[\t\n\r ]+|[\t\n\r ]+$/g, "");
    if (!TOKEN_RE.test(name) || /[\r\n\0]/.test(v)) throw xhrErr('SyntaxError', "Invalid header name or value");
    var lower = name.toLowerCase();
    if (isForbiddenRequestHeader(lower, v)) return;
    for (var i = 0; i < this._reqHeaders.length; i++) {
      if (this._reqHeaders[i][0] === lower) { this._reqHeaders[i][1] += ", " + v; return; }
    }
    this._reqHeaders.push([lower, v]);
  };

  XMLHttpRequest.prototype.overrideMimeType = function(mime) {
    if (this._state === XL || this._state === XD)
      throw xhrErr('InvalidStateError', "overrideMimeType: the request is LOADING or DONE");
    var m = parseMime(String(mime));
    if (!m) throw xhrErr('SyntaxError', "Invalid MIME type");
    this._override = m;
  };

  XMLHttpRequest.prototype.abort = function() {
    this._terminate();
    var s = this._state;
    if ((s === XO && this._sendFlag) || s === XH || s === XL) xhrRequestError(this, 'abort', false);
    if (this._state === XD) {
      this._state = XU; this._resp = null; this._bytes = null; this._respObj = undefined; this._respDoc = undefined;
    }
  };

  XMLHttpRequest.prototype.send = function(body) {
    if (this._state !== XO) throw xhrErr('InvalidStateError', "send: the request is not OPENED");
    if (this._sendFlag) throw xhrErr('InvalidStateError', "send: the request has already been sent");
    if (this._method === 'GET' || this._method === 'HEAD') body = null;
    var eb = (body == null) ? { bytes: null, stream: null, type: null } : extractBody(body);
    var headers = new Headers();
    for (var i = 0; i < this._reqHeaders.length; i++) headers.append(this._reqHeaders[i][0], this._reqHeaders[i][1]);
    if (eb.type && !headers.has('content-type')) headers.set('content-type', eb.type);
    var bodyBytes = eb.bytes;
    var total = bodyBytes ? bodyBytes.length : 0;

    this._uploadComplete = (bodyBytes == null);
    this._uploadListener = !this._sync && hasAnyListener(this.upload);
    this._sendFlag = true;
    this._timedOut = false;
    this._resp = null; this._bytes = null; this._respObj = undefined; this._respDoc = undefined;
    this._sendTime = Date.now();

    if (this._sync) return xhrSendSync(this, headers, bodyBytes);

    var self = this;
    xhrProgress(this, 'loadstart', 0, 0, false);
    if (!this._uploadComplete && this._uploadListener) xhrProgress(this.upload, 'loadstart', 0, total, true);
    if (!this._sendFlag) return;   // a loadstart listener aborted us

    var ctl = new AbortController();
    this._ctl = ctl;
    var init = {
      method: this._method, headers: headers, signal: ctl.signal, mode: 'cors',
      credentials: this._withCred ? 'include' : 'same-origin'
    };
    if (bodyBytes != null) init.body = bodyBytes;
    xhrArmTimeout(this);

    fetch(this._url, init).then(function(res) {
      if (!self._sendFlag) return;
      if (!self._uploadComplete) {
        self._uploadComplete = true;
        if (self._uploadListener) {
          xhrProgress(self.upload, 'progress', total, total, true);
          xhrProgress(self.upload, 'load', total, total, true);
          xhrProgress(self.upload, 'loadend', total, total, true);
        }
      }
      self._resp = res;
      self._state = XH;
      xhrFire(self, 'readystatechange');
      if (!self._sendFlag) return;
      return res.arrayBuffer().then(function(buf) {
        if (!self._sendFlag) return;
        var bytes = new Uint8Array(buf);
        self._bytes = bytes;
        var cl = res.headers.get('content-length');
        var len = cl ? parseInt(cl, 10) : NaN;
        var computable = !isNaN(len) && len >= 0;
        if (!computable) len = bytes.length;
        self._state = XL;
        xhrFire(self, 'readystatechange');
        if (!self._sendFlag) return;
        xhrProgress(self, 'progress', bytes.length, len, computable);
        if (!self._sendFlag) return;
        self._state = XD;
        self._sendFlag = false;
        xhrClearTimer(self);
        xhrFire(self, 'readystatechange');
        xhrProgress(self, 'load', bytes.length, len, computable);
        xhrProgress(self, 'loadend', bytes.length, len, computable);
      });
    }).then(null, function() {
      if (!self._sendFlag) return;   // abort / timeout already ran the error steps
      xhrRequestError(self, 'error', false);
    });
  };

  // The synchronous path: the host answers in place (a sync FetchHandler, or the
  // no-handler network error), so send() returns with the response in hand.
  function xhrSendSync(x, headers, bodyBytes) {
    var flat = [];
    if (!headers.has('accept')) { flat.push('accept'); flat.push('*/*'); }
    for (var i = 0; i < headers._h.length; i++) { flat.push(headers._h[i][0]); flat.push(headers._h[i][1]); }
    var json = Core.call(nativeFetchSync, undefined, [x._method, x._url, flat.join('\n'),
                            bodyBytes != null ? bytesToBinaryString(bodyBytes) : '',
                            'default', 'follow', 'cors', xhrDocHref(), '',
                            x._withCred ? 'include' : 'same-origin', '', bodyBytes !== null ? '1' : '0']);
    var o = json ? JSON.parse(json) : { networkError: true };
    if (o.networkError) { xhrRequestError(x, 'error', true); return; }
    var res = responseFromOutcome(o);
    x._resp = res;
    x._bytes = bodyRecord(res).bytes || new Uint8Array(0);
    x._state = XD;
    x._sendFlag = false;
    xhrFire(x, 'readystatechange');
    var n = x._bytes.length;
    xhrProgress(x, 'load', n, n, true);
    xhrProgress(x, 'loadend', n, n, true);
  }

  XMLHttpRequest.prototype.getResponseHeader = function(name) {
    if (!this._resp) return null;
    try { return this._resp.headers.get(String(name)); } catch (e) { return null; }
  };
  XMLHttpRequest.prototype.getAllResponseHeaders = function() {
    if (!this._resp) return '';
    var s = this._resp.headers._sorted(), out = '';
    for (var i = 0; i < s.length; i++) {
      if (isForbiddenResponseHeader(s[i][0])) continue;
      out += s[i][0] + ': ' + s[i][1] + '\r\n';
    }
    return out;
  };

  // The final MIME type: the override if one was set, else the response's
  // Content-Type. The override's charset wins only when it has one.
  function xhrMime(x) {
    var ct = x._resp ? x._resp.headers.get('content-type') : null;
    var m = ct ? parseMime(ct) : null;
    if (x._override) return { essence: x._override.essence, charset: x._override.charset || (m ? m.charset : '') };
    return { essence: m ? m.essence : '', charset: m ? m.charset : '' };
  }
  function xhrText(x) {
    if (!x._bytes || x._bytes.length === 0) return '';
    var cs = xhrMime(x).charset;
    return XHR_LATIN1[cs] ? latin1Decode(x._bytes) : utf8Decode(x._bytes);
  }

  function xhrGetter(name, fn) {
    Object.defineProperty(XMLHttpRequest.prototype, name, { configurable: true, enumerable: true, get: fn });
  }
  xhrGetter('readyState', function() { return this._state; });
  xhrGetter('status', function() { return this._resp ? this._resp.status : 0; });
  xhrGetter('statusText', function() { return this._resp ? this._resp.statusText : ''; });
  xhrGetter('responseURL', function() {
    var u = this._resp ? (this._resp.url || '') : '';
    var h = u.indexOf('#');
    return h < 0 ? u : u.slice(0, h);
  });
  xhrGetter('responseText', function() {
    if (this._respType !== '' && this._respType !== 'text')
      throw xhrErr('InvalidStateError', 'responseText requires responseType "" or "text"');
    if (this._state !== XL && this._state !== XD) return '';
    return xhrText(this);
  });
  // XHR "response document": HTML and the XML essences parse through DOMParser
  // (the same host parsers), anything else is null. Parsed once and cached.
  var XHR_XML_ESSENCE = {
    'text/xml': 1, 'application/xml': 1, 'application/xhtml+xml': 1, 'image/svg+xml': 1
  };
  xhrGetter('responseXML', function() {
    if (this._respType !== '' && this._respType !== 'document')
      throw xhrErr('InvalidStateError', 'responseXML requires responseType "" or "document"');
    if (this._state !== XD) return null;
    if (this._respDoc !== undefined) return this._respDoc;
    var essence = xhrMime(this).essence;
    var isXml = !!XHR_XML_ESSENCE[essence] || /\+xml$/.test(essence);
    var doc = null;
    if ((essence === 'text/html' || isXml) && globalThis.DOMParser) {
      try {
        doc = new DOMParser().parseFromString(xhrText(this),
          essence === 'text/html' ? 'text/html' : 'application/xml');
        // A failed XML parse is a null response document, not a parsererror one.
        if (isXml && doc.documentElement &&
            doc.documentElement.localName === 'parsererror') doc = null;
      } catch (e) { doc = null; }
    }
    this._respDoc = doc;
    return doc;
  });
  xhrGetter('response', function() {
    var t = this._respType;
    if (t === '' || t === 'text') {
      if (this._state !== XL && this._state !== XD) return '';
      return xhrText(this);
    }
    if (this._state !== XD || !this._resp) return null;
    if (this._respObj !== undefined) return this._respObj;
    var bytes = this._bytes || new Uint8Array(0);
    var v = null;
    if (t === 'arraybuffer') v = bytes.slice(0).buffer;
    else if (t === 'blob') v = new Blob([bytes], { type: xhrMime(this).essence || '' });
    else if (t === 'json') { try { v = JSON.parse(utf8Decode(bytes)); } catch (e) { v = null; } }
    this._respObj = v;
    return v;
  });

  Object.defineProperty(XMLHttpRequest.prototype, 'responseType', {
    configurable: true, enumerable: true,
    get: function() { return this._respType; },
    set: function(v) {
      var s = String(v);
      if (!XHR_TYPES[s] && s !== '') return;   // an invalid enum value is ignored
      if (this._state === XL || this._state === XD)
        throw xhrErr('InvalidStateError', 'responseType cannot be set while LOADING or DONE');
      if (this._sync) throw xhrErr('InvalidAccessError', 'responseType on a synchronous request');
      this._respType = s;
    }
  });
  Object.defineProperty(XMLHttpRequest.prototype, 'timeout', {
    configurable: true, enumerable: true,
    get: function() { return this._timeout; },
    set: function(v) {
      if (this._sync) throw xhrErr('InvalidAccessError', 'timeout on a synchronous request');
      var n = Number(v);
      this._timeout = (isNaN(n) || n < 0) ? 0 : (n >>> 0);
      if (this._sendFlag) xhrArmTimeout(this);
    }
  });
  Object.defineProperty(XMLHttpRequest.prototype, 'withCredentials', {
    configurable: true, enumerable: true,
    get: function() { return this._withCred; },
    set: function(v) {
      if (this._state !== XU && this._state !== XO)
        throw xhrErr('InvalidStateError', 'withCredentials requires UNSENT or OPENED');
      if (this._sendFlag) throw xhrErr('InvalidStateError', 'withCredentials after send()');
      this._withCred = !!v;
    }
  });

  var XHR_STATES = { UNSENT: XU, OPENED: XO, HEADERS_RECEIVED: XH, LOADING: XL, DONE: XD };
  for (var xk in XHR_STATES) {
    Object.defineProperty(XMLHttpRequest, xk, { value: XHR_STATES[xk], enumerable: true });
    Object.defineProperty(XMLHttpRequest.prototype, xk, { value: XHR_STATES[xk], enumerable: true });
  }
  defineInterface('XMLHttpRequest', XMLHttpRequest, 0);
})();
"#;
