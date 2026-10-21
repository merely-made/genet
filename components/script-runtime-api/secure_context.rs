/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Whether an environment is a secure context (W3C Secure Contexts, HTML's
//! "secure context" for windows and workers), which decides whether its
//! `[SecureContext]` members exist. Today that is Web Crypto's `crypto.subtle`,
//! `SubtleCrypto` and `CryptoKey`; `randomUUID` and `isSecureContext` are not
//! gated yet (design_docs/2026-10-20_webcrypto_subtle_plan.md).
//!
//! A document derives its context from its URL, nested documents also need a
//! secure parent, `about:` and `data:` documents inherit their creator's, and a
//! worker inherits its owner's. A host may override the result through
//! [`crate::Runtime::set_secure_context`].

use std::net::IpAddr;

/// Defaults to secure, so an embedder that never names a URL keeps every
/// member it had before secure contexts were tracked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SecureContext(pub(crate) bool);

impl Default for SecureContext {
    fn default() -> Self {
        Self(true)
    }
}

/// Secure Contexts' "potentially trustworthy" test for a document's URL, or
/// `None` when the document takes its creator's context instead (`about:blank`,
/// `about:srcdoc`, `data:`) or the URL does not parse.
pub(crate) fn potentially_trustworthy(url: &str) -> Option<bool> {
    let url = url::Url::parse(url).ok()?;
    match url.scheme() {
        "about" | "data" => None,
        "https" | "wss" | "file" => Some(true),
        // A blob URL's origin is its creator's, which the inner URL names.
        "blob" => potentially_trustworthy(url.path()),
        _ => Some(match url.host() {
            Some(url::Host::Domain(name)) => {
                let name = name.to_ascii_lowercase();
                name == "localhost" || name.ends_with(".localhost")
            },
            Some(url::Host::Ipv4(ip)) => IpAddr::V4(ip).is_loopback(),
            Some(url::Host::Ipv6(ip)) => IpAddr::V6(ip).is_loopback(),
            None => false,
        }),
    }
}

/// A document's context: its URL decides, a nested document also needs a
/// secure parent, and an inheriting URL keeps `creator`'s context.
pub(crate) fn for_document(url: &str, parent: Option<SecureContext>, creator: SecureContext) -> SecureContext {
    match potentially_trustworthy(url) {
        Some(trusted) => SecureContext(trusted && parent.is_none_or(|parent| parent.0)),
        None => creator,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trustworthy_urls_follow_secure_contexts() {
        for (url, expected) in [
            ("https://example.test/", Some(true)),
            ("wss://example.test/", Some(true)),
            ("file:///tmp/page.html", Some(true)),
            ("http://localhost:8000/", Some(true)),
            ("http://app.localhost/", Some(true)),
            ("http://127.0.0.1/", Some(true)),
            ("http://[::1]/", Some(true)),
            ("http://web-platform.test/", Some(false)),
            ("ws://example.test/", Some(false)),
            ("blob:https://example.test/0d5a", Some(true)),
            ("blob:http://example.test/0d5a", Some(false)),
            ("about:blank", None),
            ("about:srcdoc", None),
            ("data:text/html,hi", None),
            ("not a url", None),
        ] {
            assert_eq!(potentially_trustworthy(url), expected, "{url}");
        }
    }

    #[test]
    fn nested_documents_need_a_secure_parent() {
        let (secure, insecure) = (SecureContext(true), SecureContext(false));
        assert_eq!(for_document("https://a.test/", None, insecure), secure, "top level: the URL decides");
        assert_eq!(for_document("https://a.test/", Some(insecure), secure), insecure);
        assert_eq!(for_document("https://a.test/", Some(secure), insecure), secure);
        assert_eq!(for_document("http://a.test/", Some(secure), secure), insecure);
        assert_eq!(for_document("about:blank", Some(secure), insecure), insecure, "about:blank inherits its creator");
    }
}
