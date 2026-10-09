/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Raw engine host contracts for Genet.
//!
//! Hosts keep browser chrome, platform integration, protocol UI, and automation
//! routing distinct from whichever Genet engine profile they embed. What lives
//! here is what an engine host must supply or may read: the engine profile
//! and its capabilities, the resource-fetch seam, and href resolution. The
//! application half, settings projection and surface admission, is
//! `mere-surface-api`, split out under the platform boundary plan (P1) so
//! this crate depends on nothing in the workspace and Genet never names an
//! application type.

use std::fmt;
use std::str::FromStr;

pub mod navigation;

pub use navigation::resolve_href;

/// Coarse engine selection for diagnostic hosts such as standalone Pelt.
///
/// Product hosts route individual content through Inker engine ids. This enum
/// deliberately names only the two HTML capabilities that still need a CLI
/// override; windowing mode, protocol, and historical implementation names are
/// separate concerns.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EngineProfile {
    /// Script-free HTML through the owned Livery/Buckram engine. Its Inker
    /// identity is `genet.livery`.
    #[default]
    Livery,
    /// Live HTML whose scripts mutate the DOM while Livery/Buckram retain style,
    /// layout, and paint state.
    Scripted,
}

impl fmt::Display for EngineProfile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Livery => "livery",
            Self::Scripted => "scripted",
        };
        f.write_str(name)
    }
}

impl FromStr for EngineProfile {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            // Keep the shipped command spellings as input aliases while making
            // their canonical identity truthful in logs and host state.
            "livery" | "viewer" | "static" => Ok(Self::Livery),
            "scripted" | "livery-scripted" => Ok(Self::Scripted),
            other => Err(format!(
                "unknown engine profile '{other}'; expected livery or scripted (legacy aliases: viewer, static, livery-scripted)"
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ShellEngineCapabilities {
    pub javascript: bool,
    pub webdriver: bool,
    pub devtools: bool,
    pub webgpu: bool,
    pub webxr: bool,
}

pub trait ShellEngine {
    fn profile(&self) -> EngineProfile;
    fn capabilities(&self) -> ShellEngineCapabilities;
}

pub struct DeferredShellEngine {
    profile: EngineProfile,
}

impl DeferredShellEngine {
    pub fn new(profile: EngineProfile) -> Self {
        Self { profile }
    }

    pub fn unavailable_message(&self) -> String {
        format!(
            "engine profile {} is reserved for a future Genet validation path",
            self.profile
        )
    }
}

impl ShellEngine for DeferredShellEngine {
    fn profile(&self) -> EngineProfile {
        self.profile
    }

    fn capabilities(&self) -> ShellEngineCapabilities {
        ShellEngineCapabilities {
            javascript: matches!(self.profile, EngineProfile::Scripted),
            ..ShellEngineCapabilities::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EngineProfile;

    #[test]
    fn legacy_profile_spellings_canonicalize_to_owned_engines() {
        assert_eq!("viewer".parse(), Ok(EngineProfile::Livery));
        assert_eq!("static".parse(), Ok(EngineProfile::Livery));
        assert_eq!("livery-scripted".parse(), Ok(EngineProfile::Scripted));
        assert_eq!(EngineProfile::default().to_string(), "livery");
    }

    #[test]
    fn retired_profiles_are_rejected() {
        assert!("browser".parse::<EngineProfile>().is_err());
        assert!("headless".parse::<EngineProfile>().is_err());
    }
}

/// A host-owned request, independent of the runtime and transport implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceRequest {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub cache: String,
    pub redirect: String,
    pub mode: String,
    pub referrer: String,
    pub referrer_policy: String,
    pub credentials: String,
    pub integrity: String,
}

impl ResourceRequest {
    /// Requests a URL-only host can honor without silently losing semantics.
    pub fn is_plain_get(&self) -> bool {
        self.method == "GET"
            && self.body.is_none()
            && self.headers.is_empty()
            && self.cache == "default"
            && self.redirect == "follow"
            && self.mode == "cors"
            && self.referrer.is_empty()
            && self.referrer_policy.is_empty()
            && self.credentials == "same-origin"
            && self.integrity.is_empty()
    }
}

/// One host-owned resource response. Engines receive the final identity and a
/// bounded metadata surface, never a transport handle or mutable cache entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceResponse {
    pub final_url: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}
impl ResourceResponse {
    pub fn new(final_url: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            final_url: final_url.into(),
            content_type: None,
            bytes,
        }
    }
    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }
}

/// Metadata from the request-aware fetch seam. Kept distinct from the legacy
/// resource record so existing host struct literals and destructuring stay valid.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceFetchResponse {
    pub final_url: String,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub response_type: String,
    pub redirected: bool,
}
impl From<ResourceResponse> for ResourceFetchResponse {
    fn from(response: ResourceResponse) -> Self {
        let headers = response
            .content_type
            .as_ref()
            .map(|value| vec![("content-type".into(), value.clone())])
            .unwrap_or_default();
        Self {
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
}
impl From<ResourceFetchResponse> for ResourceResponse {
    fn from(response: ResourceFetchResponse) -> Self {
        Self {
            final_url: response.final_url,
            content_type: response.content_type,
            bytes: response.bytes,
        }
    }
}

/// The host-owned policy shared by one remote document-resource client: the
/// caps a host puts on redirects, concurrency, body size and time. A host
/// contract even though its enforcement is transport work, so it lives here
/// beside the fetch seam rather than with any one transport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceFetchPolicy {
    pub max_redirects: u32,
    pub max_concurrent_fetches: usize,
    pub max_response_bytes: usize,
    pub timeout: std::time::Duration,
}

impl Default for ResourceFetchPolicy {
    fn default() -> Self {
        Self {
            max_redirects: 20,
            max_concurrent_fetches: 6,
            max_response_bytes: 8 * 1024 * 1024,
            timeout: std::time::Duration::from_secs(15),
        }
    }
}

/// Host-shell resource-fetch contract: turn a URL into bytes for whichever engine
/// is hosted underneath. Networking is a *platform-integration* concern the shell
/// owns — kept off the engine, which only consumes host-owned response records.
/// Impls live in the ports (a local-file fetcher, a netfetcher-backed fetcher, …);
/// an engine's own byte seams (e.g. genet's `ImageLoader`) delegate to whichever
/// the shell supplies.
pub trait ResourceFetcher {
    /// Fetch `url` to bytes, or `None` on failure / unsupported scheme.
    fn fetch(&self, url: &str) -> Option<Vec<u8>>;

    /// Rich requests are optional. URL-only implementations must decline any
    /// request whose semantics their legacy URL seam cannot preserve.
    fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
        if request.is_plain_get() {
            self.fetch_response(&request.url).map(Into::into)
        } else {
            None
        }
    }

    /// Fetch `url` with its redirect-final identity and response type. Existing
    /// byte-only hosts remain valid: their requested URL is the final identity and
    /// their content type is unknown until they opt into this richer response.
    fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
        self.fetch(url)
            .map(|bytes| ResourceResponse::new(url, bytes))
    }
}

#[cfg(test)]
mod request_compatibility {
    use super::*;
    use std::cell::Cell;
    struct UrlOnly(Cell<usize>);
    impl ResourceFetcher for UrlOnly {
        fn fetch(&self, _: &str) -> Option<Vec<u8>> {
            self.0.set(self.0.get() + 1);
            Some(vec![0, 128, 255])
        }
    }
    fn plain() -> ResourceRequest {
        ResourceRequest {
            url: "https://host.test/resource".into(),
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
        }
    }
    #[test]
    fn url_only_default_serves_get_and_declines_semantic_changes() {
        let fetcher = UrlOnly(Cell::new(0));
        let response = fetcher.fetch_request(&plain()).unwrap();
        assert_eq!(response.bytes, [0, 128, 255]);
        assert_eq!(response.final_url, "https://host.test/resource");
        for change in 0..10 {
            let mut request = plain();
            match change {
                0 => request.method = "POST".into(),
                1 => request.body = Some(vec![]),
                2 => request
                    .headers
                    .push(("content-type".into(), "text/plain".into())),
                3 => request.cache = "no-store".into(),
                4 => request.redirect = "manual".into(),
                5 => request.mode = "navigate".into(),
                6 => request.referrer = "https://host.test/page".into(),
                7 => request.referrer_policy = "origin".into(),
                8 => request.credentials = "include".into(),
                _ => request.integrity = "sha256-example".into(),
            }
            assert!(
                fetcher.fetch_request(&request).is_none(),
                "changed field {change} cannot be discarded"
            );
        }
        assert_eq!(fetcher.0.get(), 1);
    }
}
