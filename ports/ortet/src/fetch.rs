// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Ortet's fetch seam: two lanes and a scheme split between them.
//!
//! `data:`, `file:` and bare paths are answered by `genet-documents`'
//! [`LocalFetcher`]. `http(s)` goes to netfetcher's default transport on a
//! background tokio runtime this host owns. Every other scheme is a miss —
//! ortet has no trust store, no protocol registry, and no smolweb lane, and
//! adding one is a product decision that belongs to Mere.

use std::sync::Arc;

use genet_documents::LocalFetcher;
use genet_host_api::{ResourceFetchResponse, ResourceFetcher, ResourceRequest, ResourceResponse};

/// Which lane serves an address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FetchLane {
    /// `data:`, `file:`, or a bare filesystem path.
    Local,
    /// `http:` or `https:`.
    Remote,
    /// Anything else. Ortet answers `None`; it does not guess a transport.
    Unsupported,
}

/// The scheme split, as a pure function so it is testable without a network.
pub fn lane_for(url: &str) -> FetchLane {
    match crate::args::url_scheme(url) {
        // No scheme is a filesystem path, which is what `LocalFetcher` reads.
        None => FetchLane::Local,
        Some(scheme) => {
            let scheme = scheme.to_ascii_lowercase();
            match scheme.as_str() {
                "data" | "file" => FetchLane::Local,
                "http" | "https" => FetchLane::Remote,
                _ => FetchLane::Unsupported,
            }
        },
    }
}

/// The one fetcher the session engine is constructed with.
#[derive(Clone)]
pub struct OrtetFetcher {
    remote: Option<Arc<RemoteFetcher>>,
}

impl OrtetFetcher {
    /// Local schemes only. The receipts run this way, and so does any run whose
    /// document never names a remote resource.
    pub fn local_only() -> Self {
        Self { remote: None }
    }

    /// Local schemes plus http(s) over netfetcher.
    pub fn with_network() -> Result<Self, String> {
        Ok(Self {
            remote: Some(Arc::new(RemoteFetcher::new()?)),
        })
    }
}

impl ResourceFetcher for OrtetFetcher {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.fetch_response(url).map(|response| response.bytes)
    }

    fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
        match lane_for(&request.url) {
            FetchLane::Local if request.is_plain_get() => {
                LocalFetcher.fetch_response(&request.url).map(Into::into)
            },
            FetchLane::Remote => self.remote.as_ref()?.fetch_request(request),
            _ => None,
        }
    }

    fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
        match lane_for(url) {
            FetchLane::Local => LocalFetcher.fetch_response(url),
            FetchLane::Remote => self.remote.as_ref()?.fetch_response(url),
            FetchLane::Unsupported => None,
        }
    }
}

/// http(s) over netfetcher's default transport.
///
/// The document session boundary is synchronous, so the async fetch is run to
/// completion on a runtime this host owns rather than on the UI thread's own
/// executor — the arrangement netfetcher's README describes for its consumers.
pub struct RemoteFetcher {
    runtime: tokio::runtime::Runtime,
    context: netfetcher::FetchContext,
}

impl RemoteFetcher {
    pub fn new() -> Result<Self, String> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("ortet-fetch")
            .enable_all()
            .build()
            .map_err(|error| format!("could not start the ortet fetch runtime: {error}"))?;
        Ok(Self {
            runtime,
            context: netfetcher::FetchContext::permissive(),
        })
    }
}

impl ResourceFetcher for RemoteFetcher {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.fetch_response(url).map(|response| response.bytes)
    }

    fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
        self.fetch_request(&ResourceRequest {
            url: url.into(),
            method: "GET".into(),
            headers: vec![],
            body: None,
            cache: "default".into(),
            redirect: "follow".into(),
            mode: "cors".into(),
            referrer: String::new(),
            referrer_policy: String::new(),
            credentials: "same-origin".into(),
            integrity: String::new(),
        })
        .map(Into::into)
    }

    fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
        if lane_for(&request.url) != FetchLane::Remote {
            return None;
        }
        let parsed = url::Url::parse(&request.url).ok()?;
        let mut wire = netfetcher::Request::get(parsed);
        wire.method = match request.method.as_str() {
            "GET" => netfetcher::Method::Get,
            "HEAD" => netfetcher::Method::Head,
            "POST" => netfetcher::Method::Post,
            "PUT" => netfetcher::Method::Put,
            "DELETE" => netfetcher::Method::Delete,
            "PATCH" => netfetcher::Method::Patch,
            "OPTIONS" => netfetcher::Method::Options,
            other => netfetcher::Method::Other(other.into()),
        };
        wire.headers = request.headers.clone();
        wire.body = request.body.clone().map(Into::into);
        wire.cache = match request.cache.as_str() {
            "default" => netfetcher::CacheMode::Default,
            "no-store" => netfetcher::CacheMode::NoStore,
            "reload" => netfetcher::CacheMode::Reload,
            "no-cache" => netfetcher::CacheMode::NoCache,
            "force-cache" => netfetcher::CacheMode::ForceCache,
            "only-if-cached" => netfetcher::CacheMode::OnlyIfCached,
            _ => return None,
        };
        wire.redirect = match request.redirect.as_str() {
            "follow" => netfetcher::RedirectMode::Follow,
            "error" => netfetcher::RedirectMode::Error,
            "manual" => netfetcher::RedirectMode::Manual,
            _ => return None,
        };
        wire.mode = match request.mode.as_str() {
            "cors" => netfetcher::RequestMode::Cors,
            "no-cors" => netfetcher::RequestMode::NoCors,
            "same-origin" => netfetcher::RequestMode::SameOrigin,
            "navigate" => netfetcher::RequestMode::Navigate,
            _ => return None,
        };
        wire.referrer = if request.referrer.is_empty() {
            None
        } else {
            Some(url::Url::parse(&request.referrer).ok()?)
        };
        wire.origin = wire.referrer.as_ref().map(url::Url::origin);
        wire.referrer_policy = match request.referrer_policy.as_str() {
            "" => netfetcher::ReferrerPolicy::Empty,
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
            _ => return None,
        };
        wire.credentials = match request.credentials.as_str() {
            "omit" => netfetcher::Credentials::Omit,
            "same-origin" => netfetcher::Credentials::SameOrigin,
            "include" => netfetcher::Credentials::Include,
            _ => return None,
        };
        wire.integrity = request.integrity.clone();
        if request.mode == "navigate" {
            wire.destination = netfetcher::Destination::Document;
        }
        self.runtime.block_on(async {
            let response = netfetcher::fetch(wire, &self.context).await;
            if response.is_network_error() {
                return None;
            }
            let final_url = response
                .url_list
                .last()
                .map_or_else(|| request.url.clone(), ToString::to_string);
            let content_type = response
                .headers
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
                .map(|(_, value)| value.clone());
            let status = response.status;
            let headers = response.headers.clone();
            let response_type = match response.response_type {
                netfetcher::ResponseType::Basic => "basic",
                netfetcher::ResponseType::Cors => "cors",
                netfetcher::ResponseType::Opaque => "opaque",
                netfetcher::ResponseType::OpaqueRedirect => "opaqueredirect",
                netfetcher::ResponseType::Error => "error",
            }
            .into();
            let redirected = response.url_list.len() > 1;
            let bytes = response.bytes().await.ok()?.to_vec();
            let mut resource: ResourceFetchResponse =
                ResourceResponse::new(final_url, bytes).into();
            resource.content_type = content_type;
            resource.status = status;
            // netfetcher exposes the numeric status, not the transport reason phrase.
            resource.status_text = String::new();
            resource.headers = headers;
            resource.response_type = response_type;
            resource.redirected = redirected;
            Some(resource)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scheme_split_sends_local_addresses_to_the_filesystem() {
        assert_eq!(lane_for("a.html"), FetchLane::Local);
        assert_eq!(lane_for("./sub/a.css"), FetchLane::Local);
        assert_eq!(lane_for("C:\\pages\\a.html"), FetchLane::Local);
        assert_eq!(lane_for("file:///x/a.html"), FetchLane::Local);
        assert_eq!(lane_for("FILE:///x/a.html"), FetchLane::Local);
        assert_eq!(lane_for("data:text/html,<p>x</p>"), FetchLane::Local);
    }

    #[test]
    fn only_http_and_https_reach_the_network_lane() {
        assert_eq!(lane_for("http://example.invalid/a"), FetchLane::Remote);
        assert_eq!(lane_for("https://example.invalid/a"), FetchLane::Remote);
        assert_eq!(lane_for("HTTPS://example.invalid/a"), FetchLane::Remote);
    }

    /// Ortet is not a protocol registry. A scheme it does not serve is a clean
    /// miss, never a guess at a transport or a fall-through to the filesystem.
    #[test]
    fn every_other_scheme_is_an_honest_miss() {
        assert_eq!(
            lane_for("gemini://capsule.invalid/"),
            FetchLane::Unsupported
        );
        assert_eq!(lane_for("ws://example.invalid/"), FetchLane::Unsupported);
        assert_eq!(lane_for("about:blank"), FetchLane::Unsupported);

        let fetcher = OrtetFetcher::local_only();
        assert!(fetcher.fetch("gemini://capsule.invalid/").is_none());
        // With no network lane wired, http(s) is a miss rather than a panic.
        assert!(fetcher.fetch("https://example.invalid/a").is_none());
        assert!(fetcher.fetch("/no/such/ortet/file.html").is_none());
    }

    /// The local lane really reads bytes, so a passing "miss" test above is not
    /// merely an instrument that answers `None` to everything.
    #[test]
    fn the_local_lane_reads_a_real_file() {
        let fixture = std::env::temp_dir().join(format!(
            "ortet-fetch-{}-{}.html",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock after epoch")
                .as_nanos()
        ));
        std::fs::write(&fixture, b"<p>ortet</p>").expect("write fixture");
        let fetcher = OrtetFetcher::local_only();
        let address = crate::args::file_url_from_path(&fixture);
        assert_eq!(
            fetcher.fetch(&address).as_deref(),
            Some(b"<p>ortet</p>".as_slice()),
            "the local lane must read {address}"
        );
        std::fs::remove_file(&fixture).expect("remove fixture");
    }
}
