// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The document policy container reaches the host with every request. The referrer
//! policy comes from `<meta name=referrer>` (legacy spellings included) or the
//! `Referrer-Policy` header the host hands over, applies to a request that names
//! none, and loses to the request's own. The two mixed-content directives,
//! `upgrade-insecure-requests` and `block-all-mixed-content`, come from a
//! `<meta http-equiv=Content-Security-Policy>` or the header. Backends: Boa and Vano.

use std::cell::RefCell;
use std::rc::Rc;

use script_engine_api::ScriptEngine;
use script_runtime_api::{FetchHandler, FetchOutcome, FetchRequest, NoScriptLoader, Runtime};

struct Record(Rc<RefCell<Vec<FetchRequest>>>);

impl FetchHandler for Record {
    fn fetch(&self, req: FetchRequest) -> FetchOutcome {
        self.0.borrow_mut().push(req.clone());
        FetchOutcome {
            network_error: false,
            status: 200,
            status_text: "OK".to_owned(),
            response_type: "basic".to_owned(),
            url: req.url,
            redirected: false,
            headers: Vec::new(),
            body: Vec::new(),
        }
    }
    fn fetch_blocking(&self, req: FetchRequest) -> FetchOutcome {
        self.fetch(req)
    }
}

/// A runtime on `http://a.test/page` holding `html`, and the requests its handler sees.
fn page<E: ScriptEngine>(
    html: &str,
    headers: Vec<(String, String)>,
) -> (Runtime<E>, Rc<RefCell<Vec<FetchRequest>>>) {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("http://a.test/page").unwrap();
    rt.set_document_response_headers(headers);
    rt.parse_document_interleaved(html, &NoScriptLoader);
    rt.set_fetch_handler(Box::new(Record(seen.clone())));
    (rt, seen)
}

fn policy_of_fetch<E: ScriptEngine>(html: &str, headers: &[(&str, &str)], init: &str) -> String {
    let headers = headers
        .iter()
        .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
        .collect();
    let (mut rt, seen) = page::<E>(html, headers);
    rt.eval(&format!("fetch('http://b.test/x', {init});"))
        .unwrap();
    rt.run_microtasks();
    let requests = seen.borrow();
    assert_eq!(requests.len(), 1, "{html}");
    requests[0].referrer_policy.clone()
}

fn meta_and_header<E: ScriptEngine>() {
    let p = |html: &str, headers: &[(&str, &str)], init: &str| {
        policy_of_fetch::<E>(html, headers, init)
    };
    // No policy anywhere: the default, which is the empty string on the request.
    assert_eq!(p("<body></body>", &[], "{}"), "");
    for (content, expected) in [
        ("always", "unsafe-url"),
        ("never", "no-referrer"),
        ("origin-when-crossorigin", "origin-when-cross-origin"),
        ("default", ""),
        ("strict-origin", "strict-origin"),
        ("bogus", ""),
    ] {
        let html = format!("<head><meta name=referrer content='{content}'></head><body></body>");
        assert_eq!(p(&html, &[], "{}"), expected, "meta {content}");
    }
    // The header alone, last known token winning.
    assert_eq!(
        p(
            "<body></body>",
            &[("Referrer-Policy", "no-referrer, same-origin")],
            "{}"
        ),
        "same-origin"
    );
    // A meta after the header overrides it; an invalid meta changes nothing.
    assert_eq!(
        p(
            "<head><meta name=referrer content=never></head>",
            &[("referrer-policy", "origin")],
            "{}"
        ),
        "no-referrer"
    );
    assert_eq!(
        p(
            "<head><meta name=referrer content=bogus></head>",
            &[("referrer-policy", "origin")],
            "{}"
        ),
        "origin"
    );
    // The last valid meta wins; the name is matched without regard to case.
    assert_eq!(
        p(
            "<head><meta name=referrer content=never><META NAME=Referrer content=unsafe-url></head>",
            &[],
            "{}"
        ),
        "unsafe-url"
    );
    // The request's own policy beats the document's.
    assert_eq!(
        p(
            "<head><meta name=referrer content=never></head>",
            &[],
            "{referrerPolicy: 'origin'}"
        ),
        "origin"
    );
}

fn csp_directives<E: ScriptEngine>() {
    let flags = |html: &str, headers: &[(&str, &str)]| -> (bool, bool) {
        let headers = headers
            .iter()
            .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
            .collect();
        let (mut rt, seen) = page::<E>(html, headers);
        rt.eval("fetch('http://b.test/x');").unwrap();
        rt.run_microtasks();
        let requests = seen.borrow();
        assert_eq!(requests.len(), 1, "{html}");
        (
            requests[0].upgrade_insecure_requests,
            requests[0].block_all_mixed_content,
        )
    };
    assert_eq!(flags("<body></body>", &[]), (false, false));
    assert_eq!(
        flags(
            "<head><meta http-equiv='Content-Security-Policy' content='upgrade-insecure-requests'></head>",
            &[]
        ),
        (true, false)
    );
    assert_eq!(
        flags(
            "<head><META HTTP-EQUIV=content-security-policy CONTENT='img-src *; Block-All-Mixed-Content'></head>",
            &[]
        ),
        (false, true)
    );
    assert_eq!(
        flags(
            "<body></body>",
            &[(
                "Content-Security-Policy",
                "upgrade-insecure-requests; block-all-mixed-content"
            )]
        ),
        (true, true)
    );
    // A report-only policy enforces nothing, and an unrelated directive names neither.
    assert_eq!(
        flags(
            "<body></body>",
            &[(
                "Content-Security-Policy-Report-Only",
                "upgrade-insecure-requests"
            )]
        ),
        (false, false)
    );
    assert_eq!(
        flags(
            "<head><meta http-equiv=Content-Security-Policy content=\"script-src 'none'\"></head>",
            &[]
        ),
        (false, false)
    );
}

fn sync_xhr<E: ScriptEngine>() {
    let (mut rt, seen) = page::<E>(
        "<head><meta name=referrer content=same-origin></head><body></body>",
        Vec::new(),
    );
    rt.eval("var x = new XMLHttpRequest(); x.open('GET', 'http://b.test/x', false); x.send();")
        .unwrap();
    let requests = seen.borrow();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].referrer_policy, "same-origin");
    assert_eq!(requests[0].referrer, "http://a.test/page");
}

macro_rules! both {
    ($($f:ident => ($b:ident, $n:ident)),*) => { $(
        #[test]
        fn $b() { $f::<script_engine_boa::BoaEngine>(); }
        #[cfg(target_pointer_width = "64")]
        #[test]
        fn $n() { $f::<script_engine_nova::NovaEngine>(); }
    )* };
}
both! {
    meta_and_header => (meta_and_header_boa, meta_and_header_nova),
    csp_directives => (csp_directives_boa, csp_directives_nova),
    sync_xhr => (sync_xhr_boa, sync_xhr_nova)
}
