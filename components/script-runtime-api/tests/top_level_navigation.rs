// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The **top-level** browsing context navigates the way a child does.
//!
//! The child receipt lives next door in `navigation.rs` and can watch from the
//! parent. Nothing watches a top-level navigation from inside the agent - every
//! realm the document can reach is replaced - so both observers here are
//! outside it:
//!
//! - The `WindowProxy` is held in the agent's **bootstrap realm**, which is not
//!   a browsing context and survives every navigation. Comparing the proxy
//!   taken before the first navigation against `window` after the third asks
//!   the child test's identity question from the one realm that outlives the
//!   answer.
//! - The event log is a `FetchHandler`, because the fetch route is one of the
//!   seams a navigation carries from the outgoing document to the incoming one.
//!   Each document's `load` / `pagehide` / `unload` handlers call `fetch`, and
//!   the handler records the order on the Rust side, where no navigation can
//!   clear it.

use std::cell::RefCell;
use std::rc::Rc;

use script_engine_api::{MAIN_REALM, RealmId, ScriptEngine};
use script_runtime_api::{
    FetchHandler, FetchOutcome, FetchRequest, NoScriptLoader, Runtime, ScriptResourceLoader,
};

/// Records every `fetch` a document makes, across documents.
#[derive(Clone, Default)]
struct Journal(Rc<RefCell<Vec<String>>>);

impl Journal {
    fn read(&self) -> String {
        self.0.borrow().join(",")
    }
    fn clear(&self) {
        self.0.borrow_mut().clear();
    }
}

impl FetchHandler for Journal {
    fn fetch(&self, request: FetchRequest) -> FetchOutcome {
        if let Some((_, entry)) = request.url.split_once("/log/") {
            self.0.borrow_mut().push(entry.to_owned());
        }
        FetchOutcome::network_error()
    }
}

/// Four pages differing only in their name, each announcing its own `load`,
/// `pagehide` and `unload` through the fetch route.
struct Pages;

impl Pages {
    fn page(name: &str) -> String {
        format!(
            "<body><p id=mark>{name}</p><script>\
             window.__page = '{name}';\
             function note(e) {{ fetch('https://top.test/log/{name}:' + e); }}\
             window.addEventListener('load', function(){{ note('load'); }});\
             window.addEventListener('pagehide', function(){{ note('pagehide'); }});\
             window.addEventListener('unload', function(){{ note('unload'); }});\
             </script></body>"
        )
    }
}

impl ScriptResourceLoader for Pages {
    fn load(&self, url: &str) -> Option<String> {
        let name = url
            .strip_prefix("https://top.test/")?
            .strip_suffix(".html")?;
        Some(Pages::page(name))
    }
}

fn runtime<E: ScriptEngine>(journal: &Journal) -> Runtime<E> {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://top.test/one.html")
        .expect("base URL");
    runtime.set_script_resource_loader(Box::new(Pages));
    runtime.set_fetch_handler(Box::new(journal.clone()));
    runtime.parse_document_interleaved(&Pages::page("one"), &NoScriptLoader);
    runtime.run_event_loop(100).expect("first load");
    runtime
}

/// Park the top context's `WindowProxy` in the bootstrap realm under `name`.
fn park<E: ScriptEngine>(runtime: &mut Runtime<E>, name: &str) {
    let top = runtime.top_realm();
    let window = runtime
        .eval_in_realm(top, "window")
        .expect("the top realm's global this");
    runtime
        .engine_mut()
        .set_global_in_realm(MAIN_REALM, name, &window)
        .expect("park in the bootstrap realm");
}

/// Ask the bootstrap realm a yes/no question about what it is holding.
fn bootstrap_says<E: ScriptEngine>(runtime: &mut Runtime<E>, expression: &str) -> bool {
    let value = runtime
        .engine_mut()
        .eval(&format!("String(!!({expression}))"))
        .expect(expression);
    runtime.value_to_string(&value).expect("stringify") == "true"
}

fn read<E: ScriptEngine>(runtime: &mut Runtime<E>, expression: &str) -> String {
    let value = runtime.eval(expression).expect(expression);
    runtime.value_to_string(&value).expect("stringify")
}

/// Three top-level navigations: the four assertions the child receipt makes,
/// asked of the context that used to be the agent's bootstrap realm.
fn the_top_level_navigates_three_times<E: ScriptEngine>() {
    let journal = Journal::default();
    let mut runtime = runtime::<E>(&journal);

    // What a freshly loaded copy of one of these pages occupies. Every page has
    // the same shape, so this is what the arena must hold again at the end; a
    // top-level navigation that leaked its outgoing document would leave more.
    let baseline_nodes = runtime.host().borrow().dom.live_node_count();
    assert_eq!(read(&mut runtime, "window.__page"), "one");
    assert_eq!(journal.read(), "one:load", "the first load fired once");
    journal.clear();

    // The bootstrap realm is not the document's origin - it has no document at
    // all - so what it may ask of the parked proxy is exactly what any
    // cross-origin holder may ask: identity, and the short cross-origin
    // property list. That is the right question here anyway; the document's
    // contents are read from inside the document's own realm below.
    park(&mut runtime, "held");
    assert!(
        bootstrap_says(&mut runtime, "held.closed === false"),
        "the parked proxy answers the cross-origin surface"
    );
    assert!(
        !bootstrap_says(
            &mut runtime,
            "(function(){try{held.document;return true;}catch(e){return false;}})()"
        ),
        "the bootstrap realm is not same-origin with the top document"
    );

    let mut seen: Vec<RealmId> = vec![runtime.top_realm()];
    for (pass, page) in ["two", "three", "four"].iter().enumerate() {
        let outgoing = runtime.top_realm();
        assert_ne!(
            outgoing, MAIN_REALM,
            "pass {pass}: the top document is never the bootstrap realm"
        );
        runtime
            .eval(&format!("location.href = 'https://top.test/{page}.html';"))
            .expect("navigate");
        runtime.run_event_loop(100).expect("navigation tasks");

        // 1. One identity across every navigation, reading the new document -
        //    asked from the realm that has held it since before the first.
        park(&mut runtime, "current");
        assert!(
            bootstrap_says(&mut runtime, "held === current"),
            "pass {pass}: the top WindowProxy is not the same object"
        );
        assert!(
            bootstrap_says(&mut runtime, "held.closed === false"),
            "pass {pass}: the parked proxy stopped answering"
        );
        // And the new realm agrees it is that same object.
        assert_eq!(read(&mut runtime, "window === globalThis"), "true");
        assert_eq!(read(&mut runtime, "window.__page"), *page);
        assert_eq!(
            read(&mut runtime, "document.getElementById('mark').textContent"),
            *page
        );
        assert_eq!(
            read(&mut runtime, "location.href"),
            format!("https://top.test/{page}.html")
        );

        // 2. The realm behind it is a new one, the outgoing realm is gone, and
        //    no realm id is reused.
        let incoming = runtime.top_realm();
        assert_ne!(
            incoming, outgoing,
            "pass {pass}: the realm was not replaced"
        );
        assert!(
            runtime.host_in_realm(outgoing).is_err(),
            "pass {pass}: the outgoing top realm {outgoing} was not discarded"
        );
        assert!(
            !seen.contains(&incoming),
            "pass {pass}: realm {incoming} was reused"
        );
        seen.push(incoming);
    }

    // 3. Unload order, once per navigation, pagehide before unload, and exactly
    //    one `load` per document.
    assert_eq!(
        journal.read(),
        "one:pagehide,one:unload,two:load,\
         two:pagehide,two:unload,three:load,\
         three:pagehide,three:unload,four:load",
        "unload sequence and load count"
    );

    // 4. The arena is back where one document leaves it: three were built and
    //    three were discarded, and nothing of the first three survives.
    runtime.run_event_loop(100).expect("settle");
    runtime.collect_garbage();
    let live = runtime.host().borrow().dom.live_node_count();
    assert_eq!(
        live, baseline_nodes,
        "the top arena did not return to one document's baseline"
    );
}

/// The host has the last word, and a refusal changes nothing: no unload, no new
/// realm, no history entry. A child is never asked.
fn the_policy_hook_can_refuse<E: ScriptEngine>() {
    let journal = Journal::default();
    let mut runtime = runtime::<E>(&journal);
    journal.clear();
    runtime.set_top_level_navigation_policy(|url| !url.ends_with("two.html"));
    let before = runtime.top_realm();

    runtime
        .eval("location.href = 'https://top.test/two.html';")
        .expect("refused navigation");
    runtime.run_event_loop(100).expect("tasks");
    assert_eq!(runtime.top_realm(), before, "a refusal replaced the realm");
    assert_eq!(read(&mut runtime, "window.__page"), "one");
    assert_eq!(journal.read(), "", "a refusal unloaded the document");

    // And the same hook lets the next one through.
    runtime
        .eval("location.href = 'https://top.test/three.html';")
        .expect("allowed navigation");
    runtime.run_event_loop(100).expect("tasks");
    assert_ne!(runtime.top_realm(), before);
    assert_eq!(read(&mut runtime, "window.__page"), "three");
    assert_eq!(journal.read(), "one:pagehide,one:unload,three:load");
}

/// A top-level fragment navigation keeps the document - same realm, same
/// `Window`, one `hashchange` - exactly as a child's does.
fn a_top_level_fragment_keeps_the_document<E: ScriptEngine>() {
    let journal = Journal::default();
    let mut runtime = runtime::<E>(&journal);
    journal.clear();
    let realm = runtime.top_realm();
    runtime
        .eval(
            "globalThis.__hash = [];\
             window.addEventListener('hashchange', function(e){ __hash.push(e.oldURL + '>' + e.newURL); });\
             location.href = 'https://top.test/one.html#alpha';",
        )
        .expect("fragment");
    runtime.run_event_loop(20).expect("tasks");

    assert_eq!(runtime.top_realm(), realm, "a fragment replaced the realm");
    assert_eq!(read(&mut runtime, "window.__page"), "one");
    assert_eq!(
        read(&mut runtime, "location.href"),
        "https://top.test/one.html#alpha"
    );
    assert_eq!(read(&mut runtime, "location.hash"), "#alpha");
    assert_eq!(read(&mut runtime, "__hash.length"), "1");
    assert_eq!(
        read(&mut runtime, "__hash[0]"),
        "https://top.test/one.html>https://top.test/one.html#alpha"
    );
    assert_eq!(journal.read(), "", "a fragment navigation unloaded nothing");
}

/// Session history at the top level: a navigation joins it, `replace` takes the
/// current entry's place, and `back` lands on the entry before.
fn top_level_history_joins_and_traverses<E: ScriptEngine>() {
    let journal = Journal::default();
    let mut runtime = runtime::<E>(&journal);

    runtime
        .eval("location.href = 'https://top.test/two.html';")
        .expect("push");
    runtime.run_event_loop(100).expect("tasks");
    runtime
        .eval("location.replace('https://top.test/three.html');")
        .expect("replace");
    runtime.run_event_loop(100).expect("tasks");
    assert_eq!(read(&mut runtime, "window.__page"), "three");

    // one -> two, replaced by three: two entries, so `back` lands on one.
    runtime.eval("history.back();").expect("back");
    runtime.run_event_loop(100).expect("tasks");
    assert_eq!(
        read(&mut runtime, "location.href"),
        "https://top.test/one.html",
        "replace did not take its own entry's place"
    );
}

/// A top document whose host was never given a base URL still has a document
/// URL - `about:blank`, which is what `location.href` reports - and a
/// fragment-only navigation resolves against it. The WPT corpus runs in disk
/// mode with exactly this shape, and `location.assign('#x')` there is a
/// same-document navigation, not a navigation to the opaque string `#x`.
fn a_base_less_top_document_still_resolves_a_fragment<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime.parse_document_interleaved("<body><p>bare</p></body>", &NoScriptLoader);
    runtime.run_event_loop(20).expect("first load");
    let realm = runtime.top_realm();
    assert_eq!(read(&mut runtime, "location.href"), "about:blank");

    runtime
        .eval(
            "globalThis.__hash = 0;             window.addEventListener('hashchange', function(){ __hash++; });             location.assign('#x');",
        )
        .expect("fragment assign");
    runtime.run_event_loop(20).expect("tasks");

    assert_eq!(
        runtime.top_realm(),
        realm,
        "a base-less fragment navigation replaced the realm"
    );
    assert_eq!(read(&mut runtime, "location.href"), "about:blank#x");
    assert_eq!(read(&mut runtime, "location.hash"), "#x");
    assert_eq!(read(&mut runtime, "__hash"), "1");
    assert_eq!(
        read(&mut runtime, "document.querySelector('p').textContent"),
        "bare"
    );
}

/// A `Location` whose browsing context has been removed navigates nothing -
/// and, in particular, does not navigate the *top* level. The top-level route
/// is reached by having no `FrameRecord`, and a discarded frame's realm has
/// none either; only the top realm itself may take it.
fn a_context_less_location_navigates_nothing<E: ScriptEngine>() {
    let journal = Journal::default();
    let mut runtime = runtime::<E>(&journal);
    journal.clear();
    let top = runtime.top_realm();

    runtime
        .eval(
            "var f = document.createElement('iframe');             document.body.appendChild(f);             globalThis.__win = f.contentWindow;             globalThis.__loc = __win.location;             f.remove();",
        )
        .expect("removed frame");
    runtime.run_event_loop(50).expect("teardown");
    assert_eq!(
        read(&mut runtime, "__win.closed"),
        "true",
        "the removed frame's context is not reported discarded"
    );

    for call in [
        "__loc.assign('about:blank')",
        "__loc.replace('about:blank')",
        "__loc.assign('https://top.test/two.html')",
        "__loc.href = 'https://top.test/two.html'",
    ] {
        runtime.eval(call).unwrap_or_else(|_| panic!("{call}"));
        runtime.run_event_loop(50).expect("tasks");
        assert_eq!(
            runtime.top_realm(),
            top,
            "{call} navigated the top-level context"
        );
        assert_eq!(read(&mut runtime, "window.__page"), "one");
        assert_eq!(journal.read(), "", "{call} unloaded the top document");
    }
}

/// A top-level navigation to a URL that does not parse is abandoned: HTML
/// stops at "parse a URL", and there is no container to rebuild a top-level
/// document from anyway. The document, its realm and its history stay.
///
/// The shape that matters is the base-less one - the whole WPT disk corpus -
/// where the document URL is `about:blank`, which cannot be a base, so every
/// relative input survives resolution unparsed.
fn an_unparseable_top_level_url_is_abandoned<E: ScriptEngine>() {
    let journal = Journal::default();
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime.set_script_resource_loader(Box::new(Pages));
    runtime.set_fetch_handler(Box::new(journal.clone()));
    runtime.parse_document_interleaved(&Pages::page("one"), &NoScriptLoader);
    runtime.run_event_loop(50).expect("first load");
    journal.clear();
    let realm = runtime.top_realm();
    assert_eq!(read(&mut runtime, "location.href"), "about:blank");

    for input in ["http://:", "relative.html", "not a url"] {
        runtime
            .eval(&format!("location.assign({input:?});"))
            .unwrap_or_else(|_| panic!("{input}"));
        runtime.run_event_loop(50).expect("tasks");
        assert_eq!(runtime.top_realm(), realm, "{input} replaced the realm");
        assert_eq!(read(&mut runtime, "location.href"), "about:blank");
        assert_eq!(read(&mut runtime, "window.__page"), "one");
        assert_eq!(journal.read(), "", "{input} unloaded the document");
    }

    // The positive control: an absolute URL still navigates from the same
    // base-less document.
    runtime
        .eval("location.assign('https://top.test/two.html');")
        .expect("parseable");
    runtime.run_event_loop(100).expect("tasks");
    assert_ne!(runtime.top_realm(), realm);
    assert_eq!(read(&mut runtime, "window.__page"), "two");
}

macro_rules! backend {
    ($name:ident, $engine:ty) => {
        mod $name {
            #[test]
            fn navigates_three_times() {
                super::the_top_level_navigates_three_times::<$engine>();
            }
            #[test]
            fn policy_hook_refuses() {
                super::the_policy_hook_can_refuse::<$engine>();
            }
            #[test]
            fn fragment_keeps_the_document() {
                super::a_top_level_fragment_keeps_the_document::<$engine>();
            }
            #[test]
            fn history_joins_and_traverses() {
                super::top_level_history_joins_and_traverses::<$engine>();
            }
            #[test]
            fn base_less_fragment_resolves() {
                super::a_base_less_top_document_still_resolves_a_fragment::<$engine>();
            }
            #[test]
            fn context_less_location_navigates_nothing() {
                super::a_context_less_location_navigates_nothing::<$engine>();
            }
            #[test]
            fn unparseable_top_level_url_abandoned() {
                super::an_unparseable_top_level_url_is_abandoned::<$engine>();
            }
        }
    };
}
backend!(boa, script_engine_boa::BoaEngine);
#[cfg(target_pointer_width = "64")]
backend!(nova, script_engine_nova::NovaEngine);
