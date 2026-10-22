/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The Fetch-spec [`Request`] — mode, credentials, redirect policy, body.
//!
//! Distinct from the *wire-level* `http::Request`: this carries the
//! Fetch-algorithm concepts (request mode, credentials mode, response tainting
//! inputs). Whether to thin-wrap the `http` crate instead is **plan open
//! question #1** — deferred; for now netfetcher owns its types.

use bytes::Bytes;
use url::{Origin, Url};

/// A Fetch request.
#[derive(Clone, Debug)]
pub struct Request {
    pub url: Url,
    pub method: Method,
    /// Header list. Placeholder shape — becomes an ordered header map once the
    /// `http`-crate-vs-own-types question (plan OQ#1) is settled.
    pub headers: Vec<(String, String)>,
    pub body: Option<Bytes>,
    pub mode: RequestMode,
    pub credentials: Credentials,
    pub redirect: RedirectMode,
    /// The initiator's origin (the page/script that issued the fetch). Drives CORS
    /// gating and response tainting. `None` = no initiator (a top-level fetch),
    /// treated as same-origin with the target — no cross-origin checks apply.
    pub origin: Option<Origin>,
    /// What the response will be used for (WHATWG Fetch request *destination*).
    /// Drives the mixed-content active/passive split: optionally-blockable
    /// destinations (image/audio/video) are auto-upgraded http→https, the rest
    /// are blocked in a secure context.
    pub destination: Destination,
    /// The HTTP cache mode (WHATWG Fetch request *cache mode*): which cached
    /// response, if any, this request may use, and which request headers it adds.
    pub cache: CacheMode,
    /// The request's referrer URL (the initiator document), or `None` for no
    /// referrer. The `Referer` header is derived from this per [`referrer_policy`].
    pub referrer: Option<Url>,
    /// Referrer policy governing the `Referer` header (a redirect's
    /// `Referrer-Policy` response header can override it mid-chain).
    pub referrer_policy: ReferrerPolicy,
    /// Subresource Integrity metadata (`alg-base64 ...`), or empty for none. The
    /// response body is verified against it; a mismatch is a network error.
    pub integrity: String,
    /// The requesting document's `upgrade-insecure-requests` policy: this request's
    /// `http` URL, and any redirect hop's, is fetched as `https`.
    pub upgrade_insecure_requests: bool,
    /// The requesting document's `block-all-mixed-content` policy. Carried for the
    /// Content Security Policy lane: Mixed Content Level 2 already upgrades or
    /// blocks every `http` subresource of a secure context, so the directive
    /// changes no outcome today.
    pub block_all_mixed_content: bool,
}

impl Request {
    /// Map the shared host request into Fetch semantics in one place. Unknown
    /// policy values are refused instead of silently becoming defaults.
    pub fn from_host(host: &genet_host_api::ResourceRequest) -> Option<Self> {
        let url = Url::parse(&host.url).ok()?;
        if !matches!(url.scheme(), "http" | "https") {
            return None;
        }
        let mut request = Self::get(url);
        request.method = match host.method.as_str() {
            "GET" => Method::Get,
            "HEAD" => Method::Head,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "DELETE" => Method::Delete,
            "PATCH" => Method::Patch,
            "OPTIONS" => Method::Options,
            other if valid_method_token(other) => Method::Other(other.into()),
            _ => return None,
        };
        request.headers = host.headers.clone();
        request.body = host.body.clone().map(Into::into);
        request.cache = match host.cache.as_str() {
            "default" => CacheMode::Default,
            "no-store" => CacheMode::NoStore,
            "reload" => CacheMode::Reload,
            "no-cache" => CacheMode::NoCache,
            "force-cache" => CacheMode::ForceCache,
            "only-if-cached" => CacheMode::OnlyIfCached,
            _ => return None,
        };
        request.redirect = match host.redirect.as_str() {
            "follow" => RedirectMode::Follow,
            "error" => RedirectMode::Error,
            "manual" => RedirectMode::Manual,
            _ => return None,
        };
        request.mode = match host.mode.as_str() {
            "cors" => RequestMode::Cors,
            "no-cors" => RequestMode::NoCors,
            "same-origin" => RequestMode::SameOrigin,
            "navigate" => RequestMode::Navigate,
            _ => return None,
        };
        request.referrer = if host.referrer.is_empty() {
            None
        } else {
            Some(Url::parse(&host.referrer).ok()?)
        };
        request.origin = request.referrer.as_ref().map(Url::origin);
        request.referrer_policy = match host.referrer_policy.as_str() {
            "" => ReferrerPolicy::Empty,
            "no-referrer" => ReferrerPolicy::NoReferrer,
            "no-referrer-when-downgrade" => ReferrerPolicy::NoReferrerWhenDowngrade,
            "same-origin" => ReferrerPolicy::SameOrigin,
            "origin" => ReferrerPolicy::Origin,
            "strict-origin" => ReferrerPolicy::StrictOrigin,
            "origin-when-cross-origin" => ReferrerPolicy::OriginWhenCrossOrigin,
            "strict-origin-when-cross-origin" => ReferrerPolicy::StrictOriginWhenCrossOrigin,
            "unsafe-url" => ReferrerPolicy::UnsafeUrl,
            _ => return None,
        };
        request.credentials = match host.credentials.as_str() {
            "omit" => Credentials::Omit,
            "same-origin" => Credentials::SameOrigin,
            "include" => Credentials::Include,
            _ => return None,
        };
        request.integrity = host.integrity.clone();
        request.upgrade_insecure_requests = host.upgrade_insecure_requests;
        request.block_all_mixed_content = host.block_all_mixed_content;
        if host.mode == "navigate" {
            request.destination = Destination::Document;
        }
        Some(request)
    }

    /// A plain `GET` with default modes — the increment-1 starting point.
    pub fn get(url: Url) -> Self {
        Self {
            url,
            method: Method::Get,
            headers: Vec::new(),
            body: None,
            mode: RequestMode::default(),
            credentials: Credentials::default(),
            redirect: RedirectMode::default(),
            origin: None,
            destination: Destination::default(),
            cache: CacheMode::default(),
            referrer: None,
            referrer_policy: ReferrerPolicy::default(),
            integrity: String::new(),
            upgrade_insecure_requests: false,
            block_all_mixed_content: false,
        }
    }

    /// Set the request destination (drives the mixed-content active/passive split).
    pub fn with_destination(mut self, destination: Destination) -> Self {
        self.destination = destination;
        self
    }

    /// Set the initiator origin (cross-origin requests need this for CORS).
    pub fn with_origin(mut self, origin: Origin) -> Self {
        self.origin = Some(origin);
        self
    }
}

fn valid_method_token(token: &str) -> bool {
    !token.is_empty()
        && token.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

/// HTTP cache mode (WHATWG Fetch §2.2.5 request cache mode).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CacheMode {
    /// RFC 9111: serve a fresh stored response, revalidate a stale one.
    #[default]
    Default,
    /// Never read or write the cache; force a network fetch.
    NoStore,
    /// Bypass the stored response (always network) but store the result.
    Reload,
    /// Always revalidate a stored response before using it.
    NoCache,
    /// Use a stored response even if stale; only network on a miss.
    ForceCache,
    /// Use a stored response even if stale; a miss is a network error.
    OnlyIfCached,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Patch,
    Options,
    /// Any other valid method token (e.g. a custom `REPORT` / `patcH`), preserved
    /// verbatim. Non-simple, so a cross-origin CORS request with one is preflighted.
    Other(String),
}

impl Method {
    /// The method token as sent on the wire; a custom token verbatim.
    pub fn as_str(&self) -> &str {
        match self {
            Method::Get => "GET",
            Method::Head => "HEAD",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
            Method::Patch => "PATCH",
            Method::Options => "OPTIONS",
            Method::Other(m) => m,
        }
    }
}

/// Fetch request mode (WHATWG Fetch §2.2.5) — gates CORS and response tainting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RequestMode {
    #[default]
    Cors,
    SameOrigin,
    NoCors,
    Navigate,
}

/// Credentials mode — whether cookies/auth travel with the request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Credentials {
    #[default]
    SameOrigin,
    Omit,
    Include,
}

/// Referrer policy (W3C Referrer Policy) — governs the `Referer` header value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ReferrerPolicy {
    /// No policy set: apply the default (`strict-origin-when-cross-origin`).
    #[default]
    Empty,
    NoReferrer,
    NoReferrerWhenDowngrade,
    SameOrigin,
    Origin,
    StrictOrigin,
    OriginWhenCrossOrigin,
    StrictOriginWhenCrossOrigin,
    UnsafeUrl,
}

/// WHATWG Fetch request *destination* — what the fetched bytes become. Only the
/// distinctions the mixed-content split needs are modeled; every other
/// destination (script, style, font, worker, the empty fetch()/XHR destination,
/// …) is "blockable" and collapses to [`Destination::Other`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Destination {
    /// The empty destination (a plain `fetch()` / XHR) — blockable.
    #[default]
    None,
    /// `image` — optionally-blockable (auto-upgraded).
    Image,
    /// `audio` — optionally-blockable.
    Audio,
    /// `video` — optionally-blockable.
    Video,
    /// `document` (a nested navigable) — blockable.
    Document,
    /// Any other destination (script, style, font, worker, …) — blockable.
    Other,
}

impl Destination {
    /// "optionally-blockable" per the Mixed Content spec: `image` / `audio` /
    /// `video`. These are auto-upgraded http→https; everything else is blockable.
    pub(crate) fn is_optionally_blockable(self) -> bool {
        matches!(
            self,
            Destination::Image | Destination::Audio | Destination::Video
        )
    }
}

/// Redirect handling (WHATWG Fetch §2.2.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RedirectMode {
    #[default]
    Follow,
    Error,
    Manual,
}

#[cfg(test)]
mod host_request_mapping {
    use super::*;

    #[test]
    fn host_request_keeps_all_transport_semantics() {
        let mut host = genet_host_api::ResourceRequest::plain_get("https://site.test/submit");
        host.method = "REPORT".into();
        host.headers = vec![("content-type".into(), "application/octet-stream".into())];
        host.body = Some(vec![0, 128, 255]);
        host.cache = "no-store".into();
        host.redirect = "manual".into();
        host.mode = "navigate".into();
        host.referrer = "https://site.test/start".into();
        host.referrer_policy = "strict-origin".into();
        host.credentials = "include".into();
        host.integrity = "sha256-example".into();
        host.upgrade_insecure_requests = true;
        host.block_all_mixed_content = true;
        let request = Request::from_host(&host).expect("supported host request");
        assert_eq!(request.url.as_str(), host.url);
        assert_eq!(request.method, Method::Other("REPORT".into()));
        assert_eq!(request.headers, host.headers);
        assert_eq!(request.body.unwrap().as_ref(), &[0, 128, 255]);
        assert_eq!(request.cache, CacheMode::NoStore);
        assert_eq!(request.redirect, RedirectMode::Manual);
        assert_eq!(request.mode, RequestMode::Navigate);
        assert_eq!(request.referrer.unwrap().as_str(), host.referrer);
        assert_eq!(
            request.origin.unwrap(),
            Url::parse(&host.referrer).unwrap().origin()
        );
        assert_eq!(request.referrer_policy, ReferrerPolicy::StrictOrigin);
        assert_eq!(request.credentials, Credentials::Include);
        assert_eq!(request.destination, Destination::Document);
        assert_eq!(request.integrity, host.integrity);
        assert!(request.upgrade_insecure_requests && request.block_all_mixed_content);
        let plain = Request::from_host(&genet_host_api::ResourceRequest::plain_get(
            "https://site.test/",
        ))
        .unwrap();
        assert!(!plain.upgrade_insecure_requests && !plain.block_all_mixed_content);
    }

    #[test]
    fn unknown_host_policy_is_refused() {
        let mut host = genet_host_api::ResourceRequest::plain_get("https://site.test/");
        host.cache = "stale-if-convenient".into();
        assert!(Request::from_host(&host).is_none());
        host.cache = "default".into();
        host.credentials = "ambient".into();
        assert!(Request::from_host(&host).is_none());
        host.credentials = "same-origin".into();
        host.method = "BAD METHOD".into();
        assert!(Request::from_host(&host).is_none());
    }
}
