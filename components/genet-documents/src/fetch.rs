/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Host-owned local resource fetching shared by every document engine.
//!
//! [`LocalFetcher`] answers the schemes an engine can serve without a network:
//! `data:` decodes the inline payload, `file://` and a bare filesystem path
//! read from disk. Every other scheme is a host's to supply: compose one with
//! [`LocalFetcher::with_fallback`] and a remote fetcher (Mere's
//! `mere-document-lanes` provides one over netfetcher and errand). Genet's
//! engine components never link a transport.

pub use genet_host_api::ResourceFetchPolicy;
use genet_host_api::{ResourceFetchResponse, ResourceFetcher, ResourceRequest, ResourceResponse};

/// The local-scheme [`ResourceFetcher`]: `data:`, `file://`, and bare paths.
#[derive(Clone, Copy, Debug, Default)]
pub struct LocalFetcher;

/// [`LocalFetcher`] with a host-supplied fetcher for every scheme it does not
/// serve itself.
#[derive(Clone, Debug)]
pub struct LocalFetcherWith<F> {
    fallback: F,
}

impl LocalFetcher {
    /// Serve the local schemes here and hand every other scheme to `fallback`.
    pub fn with_fallback<F: ResourceFetcher>(self, fallback: F) -> LocalFetcherWith<F> {
        LocalFetcherWith { fallback }
    }
}

/// Whether `url` is one of the schemes this crate serves without a network: an
/// inline `data:` payload, a `file://` URL, or a bare filesystem path.
fn is_local(url: &str) -> bool {
    if url.starts_with("data:") || url.starts_with("file://") {
        return true;
    }
    let Some((prefix, rest)) = url.split_once(':') else {
        return true;
    };
    let bytes = prefix.as_bytes();
    // A Windows drive is a path even though it has one letter before ':'.
    if bytes.len() == 1
        && bytes[0].is_ascii_alphabetic()
        && (rest.starts_with('/') || rest.starts_with('\\'))
    {
        return true;
    }
    !(bytes.first().is_some_and(u8::is_ascii_alphabetic)
        && bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.')))
}

impl ResourceFetcher for LocalFetcher {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.fetch_response(url).map(|response| response.bytes)
    }

    fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
        is_local(url).then(|| fetch_local_response(url)).flatten()
    }

    fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
        (is_local(&request.url) && request.is_plain_get())
            .then(|| fetch_local_response(&request.url))
            .flatten()
            .map(ResourceFetchResponse::from_local_plain_get)
    }
}

impl<F: ResourceFetcher> ResourceFetcher for LocalFetcherWith<F> {
    fn fetch(&self, url: &str) -> Option<Vec<u8>> {
        self.fetch_response(url).map(|response| response.bytes)
    }

    fn max_concurrent_fetches(&self) -> usize {
        self.fallback.max_concurrent_fetches()
    }

    fn fetch_response(&self, url: &str) -> Option<ResourceResponse> {
        if is_local(url) {
            fetch_local_response(url)
        } else {
            self.fallback.fetch_response(url)
        }
    }

    fn fetch_request(&self, request: &ResourceRequest) -> Option<ResourceFetchResponse> {
        if is_local(&request.url) {
            LocalFetcher.fetch_request(request)
        } else {
            self.fallback.fetch_request(request)
        }
    }
}

fn fetch_local_response(url: &str) -> Option<ResourceResponse> {
    if url.starts_with("data:") {
        let parsed = data_url::DataUrl::process(url).ok()?;
        return parsed
            .decode_to_vec()
            .ok()
            .map(|(bytes, _fragment)| ResourceResponse::new(url, bytes));
    }
    if let Some(rest) = url.strip_prefix("file://") {
        let path = rest.split_once('?').map_or(rest, |(path, _)| path);
        return std::fs::read(file_url_to_path(path))
            .ok()
            .map(|bytes| ResourceResponse::new(url, bytes));
    }
    let path = url.split_once('?').map_or(url, |(path, _)| path);
    std::fs::read(path)
        .ok()
        .map(|bytes| ResourceResponse::new(url, bytes))
}

fn file_url_to_path(after_scheme: &str) -> String {
    let path = match after_scheme.split_once('/') {
        Some((auth, rest)) if auth.is_empty() || auth.eq_ignore_ascii_case("localhost") => {
            format!("/{rest}")
        },
        _ => after_scheme.to_string(),
    };
    #[cfg(windows)]
    if let Some(rest) = path.strip_prefix('/')
        && rest.as_bytes().get(1) == Some(&b':')
    {
        return rest.to_string();
    }
    path
}

#[cfg(test)]
mod tests {
    use std::fs;

    use genet_host_api::{ResourceFetcher, ResourceRequest, ResourceResponse};

    use super::{LocalFetcher, is_local};

    #[test]
    fn missing_local_resource_is_a_clean_miss() {
        assert!(LocalFetcher.fetch("/no/such/pelt/file.html").is_none());
    }

    #[test]
    fn local_get_query_does_not_become_part_of_the_filename() {
        let fixture = std::env::temp_dir().join(format!(
            "pelt-local-form-{}-{}.html",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock after epoch")
                .as_nanos()
        ));
        fs::write(&fixture, b"submitted").expect("write local form target");
        let addressed = format!("{}?note=cedar", fixture.display());
        let response = LocalFetcher
            .fetch_response(&addressed)
            .expect("query-addressed local target");
        assert_eq!(response.final_url, addressed);
        assert_eq!(response.bytes, b"submitted");
        fs::remove_file(fixture).expect("remove local form target");
    }

    /// A remote scheme never reaches the filesystem: alone it is a miss, and
    /// with a fallback it is the fallback's answer.
    #[test]
    fn remote_schemes_are_the_fallbacks_and_never_the_filesystem() {
        struct Canned;
        impl ResourceFetcher for Canned {
            fn fetch(&self, url: &str) -> Option<Vec<u8>> {
                Some(url.as_bytes().to_vec())
            }
        }
        assert!(LocalFetcher.fetch("https://example.invalid/page").is_none());
        assert!(LocalFetcher.fetch("gemini://capsule.invalid/").is_none());
        let composed = LocalFetcher.with_fallback(Canned);
        assert_eq!(
            composed.fetch_response("https://example.invalid/page"),
            Some(ResourceResponse::new(
                "https://example.invalid/page",
                b"https://example.invalid/page".to_vec()
            ))
        );
        assert!(composed.fetch("/no/such/pelt/file.html").is_none());
        assert!(composed
            .fetch_request(&ResourceRequest::plain_get("https://example.invalid/page"))
            .is_none(), "a byte-only remote fallback cannot invent HTTP metadata");
    }

    #[test]
    fn local_plain_get_explicitly_supplies_synthetic_metadata() {
        let request = ResourceRequest::plain_get("data:text/plain,hello");
        let response = LocalFetcher.fetch_request(&request).expect("local plain GET");
        assert_eq!(response.bytes, b"hello");
        assert_eq!(response.status, 200);
        assert_eq!(response.response_type, "basic");
        assert_eq!(response.final_url, request.url);
        let mut unsupported = request;
        unsupported.method = "POST".into();
        assert!(LocalFetcher.fetch_request(&unsupported).is_none());
    }

    #[test]
    fn colon_only_schemes_do_not_receive_local_success_metadata() {
        for address in ["about:blank", "mailto:user@example.test", "custom+scheme:item"] {
            assert!(!is_local(address));
            assert!(LocalFetcher.fetch_request(&ResourceRequest::plain_get(address)).is_none());
        }
        assert!(is_local("C:\\pages\\a.html"));
        assert!(is_local("/tmp/a:b.html"));
    }
}
