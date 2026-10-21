// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The document policy container: what a document's requests carry.
//!
//! The runtime owns it because only the runtime sees the document's `<meta>`
//! elements and its frames; the host that loaded the document supplies the
//! initial response headers (`Runtime::set_document_response_headers`) because
//! only it saw them. It holds the referrer policy (a request whose own policy is
//! empty takes the document's) and the two Content Security Policy directives
//! that govern mixed content, `upgrade-insecure-requests` and
//! `block-all-mixed-content`. The rest of CSP is its own lane and adds to this
//! container.
//!
//! Residual: the policy is read from the tree when a request starts, so a
//! `<meta>` removed after it applied no longer applies (the specification keeps
//! the policy it set).

use layout_dom_api::{LayoutDom, LocalName, Namespace};

use crate::HostState;

/// The eight Referrer Policy keywords.
const POLICIES: [&str; 8] = [
    "no-referrer",
    "no-referrer-when-downgrade",
    "same-origin",
    "origin",
    "strict-origin",
    "origin-when-cross-origin",
    "strict-origin-when-cross-origin",
    "unsafe-url",
];

fn keyword(token: &str) -> Option<&'static str> {
    let token = token.trim().to_ascii_lowercase();
    POLICIES.iter().copied().find(|p| *p == token)
}

/// `Referrer-Policy` header value: a comma-separated list, the last token that is a
/// known policy wins; none known means no policy.
pub(crate) fn parse_header(value: &str) -> Option<&'static str> {
    value.split(',').rev().find_map(keyword)
}

/// `<meta name=referrer content=…>`: one keyword, plus the legacy spellings the
/// HTML Standard still maps (`never`, `always`, `origin-when-crossorigin`,
/// `default`). `default` is the empty (default) policy; anything else is ignored.
pub(crate) fn parse_meta(content: &str) -> Option<&'static str> {
    let token = content.trim().to_ascii_lowercase();
    match token.as_str() {
        "never" => Some("no-referrer"),
        "always" => Some("unsafe-url"),
        "origin-when-crossorigin" => Some("origin-when-cross-origin"),
        "default" => Some(""),
        other => keyword(other),
    }
}

/// Whether a serialized Content Security Policy (a header or `<meta>` value: policies
/// separated by commas, directives by semicolons) names `directive`.
pub(crate) fn csp_names(value: &str, directive: &str) -> bool {
    value.split(',').any(|policy| {
        policy.split(';').any(|d| {
            d.split_ascii_whitespace()
                .next()
                .is_some_and(|name| name.eq_ignore_ascii_case(directive))
        })
    })
}

/// What a document's requests carry.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DocumentPolicy {
    /// The referrer policy; empty is the default policy.
    pub referrer_policy: String,
    pub upgrade_insecure_requests: bool,
    pub block_all_mixed_content: bool,
}

/// The policy of `host`'s document: the initial response headers, then the
/// `<meta>` elements in tree order. For the referrer policy the last valid value
/// wins; each CSP directive, once named by any enforced policy, stays on.
pub(crate) fn document_policy(host: &HostState) -> DocumentPolicy {
    let mut found = DocumentPolicy::default();
    let mut csp: Vec<&str> = Vec::new();
    let mut policy = host
        .document_response_headers
        .iter()
        .rev()
        .find(|(name, _)| name.eq_ignore_ascii_case("referrer-policy"))
        .and_then(|(_, value)| parse_header(value))
        .unwrap_or("");
    // The report-only header never enforces anything.
    csp.extend(
        host.document_response_headers
            .iter()
            .filter(|(name, _)| name.eq_ignore_ascii_case("content-security-policy"))
            .map(|(_, value)| value.as_str()),
    );
    let dom = &host.dom;
    let html = Namespace::from("");
    let (name_attr, content_attr, equiv_attr) = (
        LocalName::from("name"),
        LocalName::from("content"),
        LocalName::from("http-equiv"),
    );
    let mut stack = vec![dom.document()];
    while let Some(node) = stack.pop() {
        if dom
            .element_name(node)
            .is_some_and(|name| name.local.as_ref() == "meta")
        {
            let content = dom.attribute(node, &html, &content_attr);
            let is_referrer = dom
                .attribute(node, &html, &name_attr)
                .is_some_and(|name| name.eq_ignore_ascii_case("referrer"));
            if is_referrer {
                if let Some(parsed) = content.and_then(parse_meta) {
                    policy = parsed;
                }
            }
            let is_csp = dom
                .attribute(node, &html, &equiv_attr)
                .is_some_and(|equiv| equiv.eq_ignore_ascii_case("content-security-policy"));
            if is_csp {
                if let Some(content) = content {
                    csp.push(content);
                }
            }
        }
        let children: Vec<_> = dom.dom_children(node).collect();
        stack.extend(children.into_iter().rev());
    }
    found.referrer_policy = policy.to_owned();
    found.upgrade_insecure_requests = csp
        .iter()
        .any(|v| csp_names(v, "upgrade-insecure-requests"));
    found.block_all_mixed_content = csp.iter().any(|v| csp_names(v, "block-all-mixed-content"));
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_known_header_token_wins() {
        assert_eq!(parse_header("origin"), Some("origin"));
        assert_eq!(parse_header("no-referrer, unsafe-url"), Some("unsafe-url"));
        assert_eq!(parse_header("unsafe-url, bogus"), Some("unsafe-url"));
        assert_eq!(parse_header("  Same-Origin "), Some("same-origin"));
        assert_eq!(parse_header("bogus"), None);
        assert_eq!(parse_header(""), None);
    }

    #[test]
    fn csp_directives_are_found_by_name_across_policies() {
        assert!(csp_names(
            "upgrade-insecure-requests",
            "upgrade-insecure-requests"
        ));
        assert!(csp_names(
            "default-src 'self'; Upgrade-Insecure-Requests",
            "upgrade-insecure-requests"
        ));
        assert!(csp_names(
            "img-src *, block-all-mixed-content",
            "block-all-mixed-content"
        ));
        assert!(!csp_names(
            "default-src 'self'",
            "upgrade-insecure-requests"
        ));
        // A source expression that merely contains the words is not the directive.
        assert!(!csp_names(
            "script-src upgrade-insecure-requests.example",
            "upgrade-insecure-requests"
        ));
        assert!(!csp_names("", "block-all-mixed-content"));
    }

    #[test]
    fn meta_accepts_the_legacy_spellings() {
        assert_eq!(parse_meta("never"), Some("no-referrer"));
        assert_eq!(parse_meta("always"), Some("unsafe-url"));
        assert_eq!(
            parse_meta("origin-when-crossorigin"),
            Some("origin-when-cross-origin")
        );
        assert_eq!(parse_meta("default"), Some(""));
        assert_eq!(parse_meta("strict-origin"), Some("strict-origin"));
        assert_eq!(parse_meta("no-referrer, origin"), None);
        assert_eq!(parse_meta("bogus"), None);
    }
}
