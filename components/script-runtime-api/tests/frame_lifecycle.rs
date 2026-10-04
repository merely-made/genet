// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Browsing-context lifecycle: what HTML's iframe removing steps destroy, in
//! what order, and what the parent can still see afterwards.
//!
//! The three cycles matter. One removal proves the teardown runs; three prove it
//! runs *to completion* each time - a realm, a host registration, an arena or a
//! rooting entry that was only mostly released would show up here as a growing
//! host table, a reused realm id, or an arena that never returns to baseline.

use script_engine_api::{RealmId, ScriptEngine};
use script_runtime_api::{NoScriptLoader, Runtime, ScriptResourceLoader};

struct XhtmlRemovalDocument;

impl ScriptResourceLoader for XhtmlRemovalDocument {
    fn load(&self, url: &str) -> Option<String> {
        (url == "https://parent.test/remove-child.xhtml").then(|| {
            "<?xml version=\"1.0\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><body>\
             <script>parent.removeXmlChild(); parent.xmlScriptFinished = true;</script>\
             </body></html>"
                .to_owned()
        })
    }
}

fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://parent.test/page.html")
        .expect("base URL");
    runtime.parse_document_interleaved(
        "<html><head></head><body><div id='slot'></div></body></html>",
        &NoScriptLoader,
    );
    runtime.eval("globalThis.log = [];").expect("log");
    runtime
}

fn read<E: ScriptEngine>(runtime: &mut Runtime<E>, expression: &str) -> String {
    let value = runtime.eval(expression).expect("eval");
    runtime.value_to_string(&value).expect("stringify")
}

/// A child holding one nested grandchild, both recording page lifecycle events
/// into the parent's log. `srcdoc` keeps them same-origin, which is what lets a
/// child reach `parent.log` at all.
const ATTACH: &str = concat!(
    "var frame = document.createElement('iframe'); ",
    "frame.srcdoc = '<html><body><iframe id=inner srcdoc=\"<body>leaf</body>\">",
    "</iframe></body></html>'; ",
    "document.getElementById('slot').appendChild(frame);"
);

const INSTRUMENT: &str = concat!(
    "var childWindow = frame.contentWindow; ",
    "var innerWindow = childWindow.document.getElementById('inner').contentWindow; ",
    "function watch(win, tag) { ",
    "  win.addEventListener('pagehide', function() { parent.log.push(tag + ':pagehide'); }); ",
    "  win.addEventListener('unload', function() { parent.log.push(tag + ':unload'); }); ",
    // Far enough out that the teardown task, queued at 0ms when the frame is
    // removed, is unambiguously ahead of it: if this ever fires, the context's
    // timers survived its destruction rather than losing a race.
    "  win.setTimeout(function() { parent.log.push(tag + ':timer-ran'); }, 50); ",
    "} ",
    "watch(childWindow, 'child'); watch(innerWindow, 'inner');"
);

fn realms<E: ScriptEngine>(runtime: &Runtime<E>) -> Vec<RealmId> {
    let mut all: Vec<_> = runtime
        .frame_realms(runtime.top_realm())
        .into_iter()
        .map(|(_, realm)| realm)
        .collect();
    for realm in all.clone() {
        all.extend(runtime.frame_realms(realm).into_iter().map(|(_, r)| r));
    }
    all
}

/// Attach, instrument, remove; three times over. Everything asserted here is
/// per-cycle: the same facts must hold on the third pass as on the first.
fn removing_a_frame_destroys_its_context_and_releases_every_registration<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    let baseline_nodes = runtime.host().borrow().dom.live_node_count();
    let mut seen: Vec<RealmId> = Vec::new();

    for cycle in 1..=3 {
        runtime.eval(ATTACH).expect("attach");
        runtime.run_event_loop(100).expect("load");
        runtime.eval(INSTRUMENT).expect("instrument");

        let live = realms(&runtime);
        assert_eq!(
            live.len(),
            2,
            "cycle {cycle}: the child and its nested grandchild are both live"
        );
        for realm in &live {
            assert!(
                !seen.contains(realm),
                "cycle {cycle}: realm {realm} was reused after being discarded - a fresh \
                 insertion must create a fresh browsing context, not revive the old one"
            );
            assert!(runtime.host_in_realm(*realm).is_ok());
        }

        runtime
            .eval(
                "globalThis.savedDocument = childWindow.document;                  globalThis.log = []; frame.remove();",
            )
            .expect("remove");

        // 1. The removing steps run no script. HTML destroys the child navigable
        //    in a queued task, which is what
        //    `dom/nodes/insertion-removing-steps/insertion-removing-steps-iframe`
        //    is checking, so nothing has been dispatched yet.
        assert_eq!(
            read(&mut runtime, "log.join(',')"),
            "",
            "cycle {cycle}: iframe destruction ran script synchronously"
        );
        // ...but the container already has no content navigable.
        assert_eq!(read(&mut runtime, "String(frame.contentWindow)"), "null");
        assert_eq!(read(&mut runtime, "String(window.length)"), "0");
        assert!(realms(&runtime).is_empty(), "cycle {cycle}: live records");

        runtime.run_event_loop(100).expect("drain");

        // 2. Removing an iframe destroys its child navigable without dispatching
        //    `pagehide` or `unload`.
        assert_eq!(
            read(&mut runtime, "log.join(',')"),
            "",
            "cycle {cycle}: iframe removal must not dispatch unload events"
        );

        // 3. Nothing the destroyed contexts had scheduled runs at all.
        assert!(
            !read(&mut runtime, "log.join(',')").contains("timer-ran"),
            "cycle {cycle}: a destroyed context's timer still ran"
        );

        // 4. What a parent still holding the window sees. `closed` flips; the
        //    document deliberately does not - a Window's document is its
        //    document, and WPT's `document-attribute.window.js` asserts it keeps
        //    answering after the context is discarded.
        assert_eq!(
            read(&mut runtime, "String(childWindow.closed)"),
            "true",
            "cycle {cycle}: a discarded context reports closed"
        );
        assert_eq!(
            read(
                &mut runtime,
                "String(childWindow.document === savedDocument)"
            ),
            "true",
            "cycle {cycle}: a discarded context lost its document"
        );

        // 5. Every host registration, rooting entry and realm is gone.
        for realm in &live {
            assert!(
                runtime.host_in_realm(*realm).is_err(),
                "cycle {cycle}: realm {realm} kept its host registration"
            );
        }
        seen.extend(live);

        // 6. No arena or reflector is left behind: after a collection tick the
        //    parent is back at the node count it started the cycle with.
        runtime
            .eval(
                "frame = undefined; childWindow = undefined;                  innerWindow = undefined; savedDocument = undefined;",
            )
            .expect("drop");
        runtime.collect_garbage();
        assert_eq!(
            runtime.host().borrow().dom.live_node_count(),
            baseline_nodes,
            "cycle {cycle}: the parent arena did not return to its baseline"
        );
    }
}

/// `moveBefore` is the exception HTML carves out: the nested browsing context
/// survives the move, so the same realm is still there afterwards.
fn move_before_preserves_the_nested_context<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    runtime
        .eval(
            "var a = document.createElement('div'), b = document.createElement('div'); \
             document.body.appendChild(a); document.body.appendChild(b); \
             var frame = document.createElement('iframe'); frame.srcdoc = '<body>x</body>'; \
             a.appendChild(frame);",
        )
        .expect("attach");
    runtime.run_event_loop(100).expect("load");
    let before = realms(&runtime);
    assert_eq!(before.len(), 1);
    runtime
        .eval("globalThis.held = frame.contentWindow; b.moveBefore(frame, null);")
        .expect("moveBefore");
    assert_eq!(read(&mut runtime, "String(frame.parentNode === b)"), "true");
    assert_eq!(
        realms(&runtime),
        before,
        "moveBefore must not destroy and recreate the nested browsing context"
    );
    assert_eq!(read(&mut runtime, "String(held.closed)"), "false");
    assert_eq!(
        read(&mut runtime, "String(held === frame.contentWindow)"),
        "true",
        "the preserved context keeps the window the parent already held"
    );
}

/// A child parser script is run to completion when it removes its own iframe.
/// Destruction then wins over the child's remaining load work: the detached
/// owner receives no `load`, while the completed script does not panic.
fn self_removal_during_child_parse_finishes_script_without_owner_load<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    runtime
        .eval(
            "globalThis.childScriptFinished = false; globalThis.afterRemovedParserScript = false; \
             globalThis.detachedLoads = 0; \
             var frame = document.createElement('iframe'); \
             frame.addEventListener('load', function() { detachedLoads++; }); \
             frame.srcdoc = '<script>window.frameElement.remove(); parent.childScriptFinished = true;<\\/script><script>parent.afterRemovedParserScript = true;<\\/script>'; \
             document.body.appendChild(frame);",
        )
        .expect("insert self-removing frame");

    runtime
        .run_event_loop(100)
        .expect("finish child parser and drain teardown");
    assert_eq!(read(&mut runtime, "String(childScriptFinished)"), "true");
    assert_eq!(
        read(&mut runtime, "String(afterRemovedParserScript)"),
        "false",
        "removal stops the open-stream parser after its current script"
    );
    assert_eq!(read(&mut runtime, "String(detachedLoads)"), "0");
    assert_eq!(read(&mut runtime, "String(frame.contentWindow)"), "null");
    assert!(realms(&runtime).is_empty());
}

/// A synchronously running child parser script calls into its parent, which
/// removes the XML-named iframe before that child's load function returns.
/// The containing document must finish after the removed child releases its
/// load barrier, and the current script must run to completion.
fn parser_callback_removal_releases_parent_load_barrier<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://parent.test/page.html")
        .expect("base URL");
    runtime.set_script_resource_loader(Box::new(XhtmlRemovalDocument));
    runtime.parse_document_interleaved(
        "<html><head></head><body><iframe id='xml-child' src='/remove-child.xhtml'></iframe></body></html>",
        &NoScriptLoader,
    );
    runtime
        .eval(
            "globalThis.xmlScriptFinished = false; globalThis.detachedChildLoad = false; \
             globalThis.removeXmlChild = function() { document.getElementById('xml-child').remove(); }; \
             document.getElementById('xml-child').addEventListener('load', function() { detachedChildLoad = true; }); \
             globalThis.topLoad = false; window.addEventListener('load', function() { topLoad = true; });",
        )
        .expect("install parent callbacks before child loading");

    runtime
        .run_event_loop(100)
        .expect("finish parser callback, teardown, and top-level load");
    assert_eq!(read(&mut runtime, "String(xmlScriptFinished)"), "true");
    assert_eq!(read(&mut runtime, "String(detachedChildLoad)"), "false");
    assert_eq!(read(&mut runtime, "String(topLoad)"), "true");
    assert_eq!(
        read(&mut runtime, "String(document.getElementById('xml-child'))"),
        "null"
    );
}

/// A child Window `load` handler can remove the owner after the child event has
/// started. The owner-element load stays suppressed, and the parent's load
/// event follows destruction rather than racing ahead of it.
fn removal_during_window_load_defers_parent_load_until_destroyed<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://parent.test/page.html")
        .expect("base URL");
    runtime.parse_document_interleaved(
        r#"<html><head></head><body><script>
          globalThis.childLoadHandlerFinished = false;
          globalThis.ownerLoads = 0;
          globalThis.topLoad = false;
          globalThis.topLoadSawClosed = false;
          var frame = document.createElement('iframe');
          frame.addEventListener('load', function() { ownerLoads++; });
          frame.srcdoc = '<script>window.addEventListener("load", function() { window.frameElement.remove(); parent.childLoadHandlerFinished = true; });<\/script>';
          document.body.appendChild(frame);
          globalThis.heldChildWindow = frame.contentWindow;
          window.addEventListener('load', function() {
            topLoadSawClosed = heldChildWindow.closed;
            topLoad = true;
          });
        </script></body></html>"#,
        &NoScriptLoader,
    );
    runtime
        .run_event_loop(100)
        .expect("remove during child Window load and release top load");

    assert_eq!(
        read(&mut runtime, "String(childLoadHandlerFinished)"),
        "true"
    );
    assert_eq!(read(&mut runtime, "String(ownerLoads)"), "0");
    assert_eq!(read(&mut runtime, "String(topLoad)"), "true");
    assert_eq!(
        read(&mut runtime, "String(topLoadSawClosed)"),
        "true",
        "top-level load waits until the removed child is destroyed"
    );
    assert_eq!(read(&mut runtime, "String(heldChildWindow.closed)"), "true");
}

/// If the child load handler removes its parent frame, completion must resume
/// at the nearest surviving ancestor rather than skipping to the top and
/// leaving the surviving outer frame's load barrier pending.
fn ancestor_removal_during_window_load_releases_surviving_ancestor<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://parent.test/page.html")
        .expect("base URL");
    runtime.parse_document_interleaved(
        r#"<html><head></head><body><script>
          globalThis.middleHandlerDone = false;
          globalThis.middleOwnerLoads = 0;
          globalThis.outerOwnerLoads = 0;
          globalThis.topLoad = false;
          globalThis.topLoadSawClosed = false;
          const closeScript = '</' + 'script>';
          const jsLiteral = source => JSON.stringify(source).replace(/</g, '\\u003c');
          const innerSource = '<html><body><script>globalThis.innerScriptRan=true;window.addEventListener("load",function(){globalThis.innerHandlerStarted=true;try{var owner=parent.frameElement;globalThis.ownerElementFound=!!owner;owner.remove();globalThis.afterOwnerRemoval=true;globalThis.frameElementCleared=parent.frameElement===null;parent.parent.parent.middleHandlerDone=true;globalThis.afterParentTraversal=true;}catch(error){globalThis.innerHandlerError=String(error);}});' + closeScript + '</body></html>';
          const middleSource = '<html><body><script>globalThis.middleScriptRan=true;var inner=document.createElement("iframe");inner.srcdoc=' + jsLiteral(innerSource) + ';document.body.appendChild(inner);parent.parent.heldInnerWindow=inner.contentWindow;' + closeScript + '</body></html>';
          const outerSource = '<html><body><script>globalThis.outerScriptRan=true;var middle=document.createElement("iframe");middle.addEventListener("load",function(){parent.middleOwnerLoads++;});middle.srcdoc=' + jsLiteral(middleSource) + ';document.body.appendChild(middle);parent.heldMiddleWindow=middle.contentWindow;' + closeScript + '</body></html>';
          var outer = document.createElement('iframe');
          outer.addEventListener('load', function() { outerOwnerLoads++; });
          outer.srcdoc = outerSource;
          document.body.appendChild(outer);
          globalThis.heldOuterWindow = outer.contentWindow;
          window.addEventListener('load', function() {
            topLoad = true;
            topLoadSawClosed = heldMiddleWindow.closed;
          });
        </script></body></html>"#,
        &NoScriptLoader,
    );
    runtime
        .run_event_loop(100)
        .expect("remove a nested parent during child Window load");

    assert_eq!(
        read(&mut runtime, "String(heldOuterWindow.outerScriptRan)"),
        "true"
    );
    assert_eq!(
        read(&mut runtime, "String(heldMiddleWindow.middleScriptRan)"),
        "true"
    );
    assert_eq!(
        read(&mut runtime, "String(heldInnerWindow.innerScriptRan)"),
        "true"
    );
    assert_eq!(
        read(&mut runtime, "String(heldInnerWindow.innerHandlerStarted)"),
        "true"
    );
    assert_eq!(
        read(&mut runtime, "String(heldInnerWindow.ownerElementFound)"),
        "true"
    );
    assert_eq!(
        read(&mut runtime, "String(heldInnerWindow.afterOwnerRemoval)"),
        "true"
    );
    assert_eq!(
        read(&mut runtime, "String(heldInnerWindow.frameElementCleared)"),
        "true"
    );
    assert_eq!(
        read(&mut runtime, "String(heldInnerWindow.afterParentTraversal)"),
        "true"
    );
    assert_eq!(
        read(&mut runtime, "String(heldInnerWindow.innerHandlerError)"),
        "undefined"
    );
    assert_eq!(read(&mut runtime, "String(middleHandlerDone)"), "true");
    assert_eq!(read(&mut runtime, "String(middleOwnerLoads)"), "0");
    assert_eq!(read(&mut runtime, "String(outerOwnerLoads)"), "1");
    assert_eq!(read(&mut runtime, "String(topLoad)"), "true");
    assert_eq!(
        read(&mut runtime, "String(topLoadSawClosed)"),
        "true",
        "the top load follows destruction while the surviving outer load completes"
    );
}

macro_rules! both_engines {
    ($($body:ident => ($boa:ident, $nova:ident)),* $(,)?) => {
        $(
            #[test]
            fn $boa() { $body::<script_engine_boa::BoaEngine>(); }

            #[cfg(target_pointer_width = "64")]
            #[test]
            fn $nova() { $body::<script_engine_nova::NovaEngine>(); }
        )*
    };
}

both_engines! {
    removing_a_frame_destroys_its_context_and_releases_every_registration =>
        (teardown_releases_everything_on_boa, teardown_releases_everything_on_nova),
    move_before_preserves_the_nested_context =>
        (move_before_preserves_context_on_boa, move_before_preserves_context_on_nova),
    self_removal_during_child_parse_finishes_script_without_owner_load =>
        (self_removal_finishes_without_load_on_boa, self_removal_finishes_without_load_on_nova),
    parser_callback_removal_releases_parent_load_barrier =>
        (parser_callback_releases_barrier_on_boa, parser_callback_releases_barrier_on_nova),
    removal_during_window_load_defers_parent_load_until_destroyed =>
        (window_load_removal_defers_parent_on_boa, window_load_removal_defers_parent_on_nova),
    ancestor_removal_during_window_load_releases_surviving_ancestor =>
        (ancestor_removal_releases_outer_on_boa, ancestor_removal_releases_outer_on_nova),
}
