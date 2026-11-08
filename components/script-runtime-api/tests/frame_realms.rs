// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Browsing-context acceptance through observable iframe APIs on both engines.
//! These tests do not manually create realms or install child globals.

use std::{cell::RefCell, rc::Rc};

use script_engine_api::ScriptEngine;
use script_runtime_api::{
    FetchHandler, FetchOutcome, FetchRequest, NoScriptLoader, Runtime, ScriptResourceLoader,
};

struct Documents;

impl ScriptResourceLoader for Documents {
    fn load(&self, url: &str) -> Option<String> {
        match url {
            "https://other.test/child.html" => Some(
                "<body><p id=child>foreign</p><script>\
                 addEventListener('message', function(event) {\
                   parent.postMessage('echo:' + event.data, '*');\
                 });\
                 </script></body>"
                    .to_owned(),
            ),
            _ => None,
        }
    }
}

fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://parent.test/page.html")
        .expect("base URL");
    runtime.set_script_resource_loader(Box::new(Documents));
    runtime.parse_document_interleaved("<html><body></body></html>", &NoScriptLoader);
    runtime
}

fn run<E: ScriptEngine>(runtime: &mut Runtime<E>, source: &str) {
    runtime.eval(source).expect("frame script");
}

fn check<E: ScriptEngine>(runtime: &mut Runtime<E>, expression: &str) {
    let value = runtime.eval(expression).expect("frame assertion");
    let result = runtime.value_to_string(&value).expect("assertion result");
    assert_eq!(result, "true", "{expression}");
}

fn drain<E: ScriptEngine>(runtime: &mut Runtime<E>) {
    // All fixture tasks have zero delay. A second pass also catches accidental
    // duplicate load events and replies posted while delivering another message.
    runtime.run_event_loop(100).expect("frame tasks");
    runtime.run_event_loop(100).expect("reply tasks");
}

fn dynamic_insertion_exposes_initial_window_synchronously<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    check(
        &mut runtime,
        r#"
        var frame = document.createElement('iframe');
        document.body.appendChild(frame);
        var win = frame.contentWindow;
        win !== null && win === frame.contentWindow && win.window === win &&
        win.document === frame.contentDocument && win.Object !== Object &&
        win.document !== document
    "#,
    );
}

fn child_script_and_parent_share_node_identity<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        r#"
        var frame = document.createElement('iframe');
        frame.srcdoc = '<body><p id="child">hello</p><script>globalThis.held = document.getElementById("child");</script></body>';
        document.body.appendChild(frame);
    "#,
    );
    drain(&mut runtime);
    check(
        &mut runtime,
        r#"
        var win = frame.contentWindow;
        var node = win.document.getElementById('child');
        node !== null && node === win.held && node.ownerDocument === win.document &&
        document.getElementById('child') === null && node instanceof win.Element &&
        !(node instanceof Element) && win.document.defaultView === win
    "#,
    );
}

fn nested_windows_expose_parent_top_frames_and_container<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    check(
        &mut runtime,
        r#"
        var outer = document.createElement('iframe');
        document.body.appendChild(outer);
        var child = outer.contentWindow;
        var inner = child.document.createElement('iframe');
        child.document.body.appendChild(inner);
        var grandchild = inner.contentWindow;
        frames === window && frames[0] === child && window.length === 1 &&
        child.parent === window && child.top === window && child.frameElement === outer &&
        child.frames[0] === grandchild && child.length === 1 &&
        grandchild.parent === child && grandchild.top === window &&
        grandchild.frameElement === inner && window.frameElement === null
    "#,
    );
}

fn cross_origin_window_rejects_non_whitelisted_reads<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        r#"
        var frame = document.createElement('iframe');
        frame.src = 'https://other.test/child.html';
        document.body.appendChild(frame);
    "#,
    );
    drain(&mut runtime);
    check(
        &mut runtime,
        r#"
        var win = frame.contentWindow;
        var failures = [];
        try { win.document; } catch (error) { failures.push(error.name); }
        try { win.secret; } catch (error) { failures.push(error.name); }
        frame.contentDocument === null && failures.join(',') === 'SecurityError,SecurityError' &&
        frame.contentWindow === win && win.window === win && win.self === win &&
        win.frames === win && typeof win.postMessage === 'function' &&
        win.parent === window && win.top === window && win.closed === false
    "#,
    );
}

fn sandbox_script_permission_and_origin_are_independent<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        r#"
        var messages = [], blockedRan = 0;
        addEventListener('message', function(event) { messages.push(event.data + ':' + event.origin); });
        var scripted = document.createElement('iframe');
        scripted.setAttribute('sandbox', 'allow-scripts');
        scripted.srcdoc = '<script>parent.postMessage("ran", "*");</script>';
        document.body.appendChild(scripted);
        var readable = document.createElement('iframe');
        readable.setAttribute('sandbox', 'allow-same-origin');
        readable.srcdoc = '<p id="visible">visible</p><script>parent.blockedRan++;</script>';
        document.body.appendChild(readable);
    "#,
    );
    drain(&mut runtime);
    check(
        &mut runtime,
        r#"
        messages.join(',') === 'ran:null' && blockedRan === 0 &&
        scripted.contentDocument === null && readable.contentDocument !== null &&
        readable.contentDocument.getElementById('visible') !== null
    "#,
    );
}

fn iframe_load_is_delivered_once_to_parent_element<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        r#"
        var loads = 0, correctTarget = false;
        var frame = document.createElement('iframe');
        frame.onload = function(event) { loads++; correctTarget = event.target === frame; };
        frame.srcdoc = '<body><p>loaded</p></body>';
        document.body.appendChild(frame);
    "#,
    );
    drain(&mut runtime);
    check(&mut runtime, "loads === 1 && correctTarget");
    drain(&mut runtime);
    check(&mut runtime, "loads === 1");
}

fn realm_message_clones_in_recipient_and_identifies_sender<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        r#"
        var frame = document.createElement('iframe');
        frame.srcdoc = '<script>addEventListener("message", function(event) { var d = event.data; globalThis.received = d; globalThis.correct = d.self === d && d.a === d.b && Object.getPrototypeOf(d) === Object.prototype && Object.getPrototypeOf(d.array) === Array.prototype && event.source === parent && event.origin === "https://parent.test"; });</script>';
        document.body.appendChild(frame);
    "#,
    );
    drain(&mut runtime);
    run(
        &mut runtime,
        r#"
        var shared = { value: 7 }, message = { a: shared, b: shared, array: [shared] };
        message.self = message;
        frame.contentWindow.postMessage(message, '*');
    "#,
    );
    drain(&mut runtime);
    check(
        &mut runtime,
        r#"
        frame.contentWindow.correct === true &&
        frame.contentWindow.received !== message &&
        frame.contentWindow.received.a !== shared
    "#,
    );
}

fn cross_origin_default_target_rejects_and_star_delivers<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        r#"
        var frame = document.createElement('iframe');
        var replies = [], correctSource = false, replyOrigin = '';
        addEventListener('message', function(event) {
            replies.push(event.data);
            correctSource = event.source === frame.contentWindow;
            replyOrigin = event.origin;
        });
        frame.src = 'https://other.test/child.html';
        document.body.appendChild(frame);
    "#,
    );
    drain(&mut runtime);
    run(
        &mut runtime,
        r#"
        frame.contentWindow.postMessage('default');
        frame.contentWindow.postMessage('slash', '/');
        frame.contentWindow.postMessage('wrong', 'https://wrong.test');
        frame.contentWindow.postMessage('star', '*');
    "#,
    );
    drain(&mut runtime);
    check(
        &mut runtime,
        "replies.join(',') === 'echo:star' && correctSource && replyOrigin === 'https://other.test'",
    );
}

fn borrowed_post_message_uses_receiver_and_authored_sender<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        r#"
        var frame = document.createElement('iframe');
        document.body.appendChild(frame);
        var received = [], fromParent = false;
        addEventListener('message', function(e) { received.push(e.data); fromParent = e.source === window; });
    "#,
    );
    drain(&mut runtime);
    run(
        &mut runtime,
        r#"
        frame.contentWindow.postMessage.call(window, 'call', '*');
        frame.contentWindow.postMessage.apply(window, ['apply', '*']);
    "#,
    );
    drain(&mut runtime);
    check(&mut runtime, "received.join(',') === 'call,apply'");
    check(&mut runtime, "fromParent");
}

fn parser_only_frame_loads_without_window_access<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        "var parentLoads = 0, readyAtLoad = false; addEventListener('load', function() { parentLoads++; readyAtLoad = globalThis.childLoaded === true && document.readyState === 'complete'; });",
    );
    runtime.parse_document_interleaved(
        r#"<body><iframe srcdoc='<script>parent.childLoaded = true;</script>'></iframe></body>"#,
        &NoScriptLoader,
    );
    assert_eq!(runtime.frame_realms(runtime.top_realm()).len(), 1);
    check(
        &mut runtime,
        "parentLoads === 0 && document.readyState === 'interactive'",
    );
    drain(&mut runtime);
    check(
        &mut runtime,
        "globalThis.childLoaded === true && parentLoads === 1 && readyAtLoad",
    );
    drain(&mut runtime);
    check(&mut runtime, "parentLoads === 1");
}

fn eager_iframe_carries_document_policy<E: ScriptEngine>() {
    #[derive(Clone, Default)]
    struct Requests(Rc<RefCell<Vec<FetchRequest>>>);

    impl FetchHandler for Requests {
        fn fetch_iframe(&self, request: FetchRequest) -> Option<FetchOutcome> {
            Some(self.fetch(request))
        }

        fn fetch(&self, request: FetchRequest) -> FetchOutcome {
            let body = if request.mode == "navigate" {
                br#"<script>
                  parent.loadedFrames++;
                  var xhr = new XMLHttpRequest();
                  xhr.open('GET', '/probe', false);
                  xhr.send();
                </script><iframe src="/nested"></iframe>"#.to_vec()
            } else {
                Vec::new()
            };
            let url = request.url.clone();
            self.0.borrow_mut().push(request);
            FetchOutcome {
                network_error: false,
                status: 200,
                status_text: "OK".into(),
                response_type: "basic".into(),
                url,
                redirected: false,
                headers: vec![
                    ("Content-Type".into(), "text/html; charset=utf-8".into()),
                    ("Referrer-Policy".into(), "same-origin".into()),
                ],
                body,
            }
        }
    }

    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://parent.test/page.html")
        .expect("base URL");
    let requests = Requests::default();
    runtime.set_fetch_handler(Box::new(requests.clone()));
    run(&mut runtime, "var loadedFrames = 0, dynamicLoads = 0;");
    runtime.parse_document_interleaved(
        r#"<head><base href="https://parent.test/assets/">
        <meta name="referrer" content="origin">
        <meta http-equiv="Content-Security-Policy"
              content="upgrade-insecure-requests; block-all-mixed-content"></head>
        <body><iframe src="parser" referrerpolicy="No-Referrer"></iframe>
        <iframe loading="lazy" src="/lazy"></iframe>
        <iframe srcdoc="" src="/srcdoc-fallback"></iframe>
        <script>
          var dynamic = document.createElement('iframe');
          dynamic.setAttribute('src', 'dynamic');
          dynamic.setAttribute('referrerpolicy', 'unsafe-url, no-referrer');
          dynamic.onload = function() { dynamicLoads++; };
          document.body.appendChild(dynamic);
        </script></body>"#,
        &NoScriptLoader,
    );
    drain(&mut runtime);
    let requests = requests.0.borrow();
    let navigations: Vec<_> = requests.iter().filter(|r| r.mode == "navigate").collect();
    assert_eq!(navigations.len(), 2);
    for (path, policy) in [("parser", "no-referrer"), ("dynamic", "origin")] {
        let request = navigations
            .iter()
            .find(|r| r.url == format!("https://parent.test/assets/{path}"))
            .expect("frame navigates relative to the base element");
        assert_eq!(request.referrer, "https://parent.test/page.html");
        assert_eq!(request.referrer_policy, policy);
        assert_eq!(request.credentials, "include");
        assert!(request.upgrade_insecure_requests);
        assert!(request.block_all_mixed_content);
    }
    let probes: Vec<_> = requests.iter().filter(|r| r.url.ends_with("/probe")).collect();
    assert_eq!(probes.len(), 2);
    assert!(probes.iter().all(|r| r.referrer_policy == "same-origin"));
    check(&mut runtime, "loadedFrames === 2 && dynamicLoads === 1");
}

fn eager_iframe_failure_does_not_fall_back_to_url_loader<E: ScriptEngine>() {
    struct FailedRequest;
    impl FetchHandler for FailedRequest {
        fn fetch_iframe(&self, _: FetchRequest) -> Option<FetchOutcome> {
            Some(FetchOutcome::network_error())
        }
    }

    struct Legacy;
    impl ScriptResourceLoader for Legacy {
        fn load(&self, _: &str) -> Option<String> {
            Some("<script>parent.usedLegacyLoader = true;</script>".into())
        }
    }

    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://parent.test/page.html")
        .expect("base URL");
    runtime.set_fetch_handler(Box::new(FailedRequest));
    runtime.set_script_resource_loader(Box::new(Legacy));
    runtime.parse_document_interleaved("<body></body>", &NoScriptLoader);
    run(
        &mut runtime,
        r#"
        var loads = 0;
        var frame = document.createElement('iframe');
        frame.src = '/blocked';
        frame.onload = function() { loads++; };
        document.body.appendChild(frame);
        "#,
    );
    drain(&mut runtime);
    check(
        &mut runtime,
        "loads === 1 && globalThis.usedLegacyLoader === undefined",
    );
}

fn removed_pending_frame_releases_parent_load<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        "var parentLoads = 0, childLoads = 0; addEventListener('load', function() { parentLoads++; });",
    );
    runtime.parse_document_interleaved(
        r#"<body><iframe id=f srcdoc='<script>parent.childRan = true;</script>'></iframe>
        <script>
          var removedFrame = document.getElementById('f');
          var retainedWindow = removedFrame.contentWindow;
          retainedWindow.addEventListener('load', function() { childLoads++; });
          removedFrame.onload = function() { childLoads++; };
          removedFrame.remove();
        </script></body>"#,
        &NoScriptLoader,
    );
    drain(&mut runtime);
    check(
        &mut runtime,
        "parentLoads === 1 && childLoads === 0 && globalThis.childRan === undefined && document.readyState === 'complete' && retainedWindow.document !== undefined",
    );
    drain(&mut runtime);
    check(&mut runtime, "parentLoads === 1 && childLoads === 0");
}

fn descendant_load_precedes_ancestor_load<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        r#"
        var order = [], completeAtLoad = false;
        var frame = document.createElement('iframe');
        frame.srcdoc = '<body><iframe id="nested" srcdoc="&lt;p id=ready&gt;ready&lt;/p&gt;"></iframe><script>' +
            'document.getElementById("nested").onload = function(){ parent.order.push("nested"); };' +
            'addEventListener("load", function(){ parent.order.push("window"); });' +
            '</scr' + 'ipt></body>';
        frame.onload = function() {
            order.push('outer');
            completeAtLoad = !!frame.contentWindow.frames[0].document.getElementById('ready');
        };
        document.body.appendChild(frame);
    "#,
    );
    drain(&mut runtime);
    check(
        &mut runtime,
        "completeAtLoad && order.join(',') === 'nested,window,outer'",
    );
    drain(&mut runtime);
    check(&mut runtime, "order.join(',') === 'nested,window,outer'");
}

macro_rules! backend {
    ($module:ident, $engine:ty) => {
        mod $module {
            #[test]
            fn removed_pending_frame_releases_parent_load() {
                super::removed_pending_frame_releases_parent_load::<$engine>();
            }
            #[test]
            fn parser_only_frame_loads_without_window_access() {
                super::parser_only_frame_loads_without_window_access::<$engine>();
            }
            #[test]
            fn eager_iframe_carries_document_policy() {
                super::eager_iframe_carries_document_policy::<$engine>();
            }
            #[test]
            fn eager_iframe_failure_does_not_fall_back_to_url_loader() {
                super::eager_iframe_failure_does_not_fall_back_to_url_loader::<$engine>();
            }
            #[test]
            fn descendant_load_precedes_ancestor_load() {
                super::descendant_load_precedes_ancestor_load::<$engine>();
            }
            #[test]
            fn borrowed_post_message_uses_receiver_and_authored_sender() {
                super::borrowed_post_message_uses_receiver_and_authored_sender::<$engine>();
            }
            #[test]
            fn dynamic_insertion_exposes_initial_window_synchronously() {
                super::dynamic_insertion_exposes_initial_window_synchronously::<$engine>();
            }
            #[test]
            fn child_script_and_parent_share_node_identity() {
                super::child_script_and_parent_share_node_identity::<$engine>();
            }
            #[test]
            fn nested_windows_expose_parent_top_frames_and_container() {
                super::nested_windows_expose_parent_top_frames_and_container::<$engine>();
            }
            #[test]
            fn cross_origin_window_rejects_non_whitelisted_reads() {
                super::cross_origin_window_rejects_non_whitelisted_reads::<$engine>();
            }
            #[test]
            fn sandbox_script_permission_and_origin_are_independent() {
                super::sandbox_script_permission_and_origin_are_independent::<$engine>();
            }
            #[test]
            fn iframe_load_is_delivered_once_to_parent_element() {
                super::iframe_load_is_delivered_once_to_parent_element::<$engine>();
            }
            #[test]
            fn realm_message_clones_in_recipient_and_identifies_sender() {
                super::realm_message_clones_in_recipient_and_identifies_sender::<$engine>();
            }
            #[test]
            fn cross_origin_default_target_rejects_and_star_delivers() {
                super::cross_origin_default_target_rejects_and_star_delivers::<$engine>();
            }
        }
    };
}

backend!(boa, script_engine_boa::BoaEngine);
#[cfg(target_pointer_width = "64")]
backend!(nova, script_engine_nova::NovaEngine);
