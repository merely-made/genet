// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime, ScriptResourceLoader};

struct Documents;
impl ScriptResourceLoader for Documents {
    fn load(&self, url: &str) -> Option<String> {
        (url == "https://parent.test/next.html").then(|| "<body><p id=next>next</p></body>".into())
    }
}

fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://parent.test/page.html").unwrap();
    rt.set_script_resource_loader(Box::new(Documents));
    rt.parse_document_interleaved("<html><body></body></html>", &NoScriptLoader);
    rt
}

fn check<E: ScriptEngine>(rt: &mut Runtime<E>, expression: &str) {
    let value = rt.eval(expression).expect(expression);
    assert_eq!(rt.value_to_string(&value).unwrap(), "true", "{expression}");
}

fn initial_load_is_synchronous_and_once<E: ScriptEngine>() {
    for attrs in [
        "",
        "f.src = '';",
        "f.src = 'about:blank';",
        "f.src = 'about:blank?x#y';",
        "f.loading = 'lazy';",
    ] {
        let mut rt = runtime::<E>();
        rt.eval(&format!(
            "var log = [], f = document.createElement('iframe'); {attrs}
             f.onload = function(e) {{
               log.push('load');
               globalThis.seen = e.target === f && !e.bubbles &&
                 f.contentDocument.readyState === 'complete';
               f.contentWindow.addEventListener('load', function() {{ log.push('window'); }});
             }};
             log.push('before'); document.body.appendChild(f); log.push('after');"
        ))
        .unwrap();
        check(&mut rt, "log.join(',') === 'before,load,after' && seen");
        rt.run_event_loop(100).unwrap();
        check(&mut rt, "log.join(',') === 'before,load,after'");
    }
}

fn navigation_listener_cannot_catch_initial_load<E: ScriptEngine>() {
    for navigate in [
        "f.src = '/next.html'",
        "f.contentWindow.location.assign('/next.html')",
    ] {
        let mut rt = runtime::<E>();
        rt.eval(&format!(
            "var f = document.createElement('iframe'); f.src = 'about:blank';
             document.body.appendChild(f); var original = f.contentDocument, held = f.contentWindow;
             var urls = []; {navigate};
             f.addEventListener('load', function() {{ urls.push(f.contentWindow.location.href); }}, {{once:true}});"
        )).unwrap();
        rt.run_event_loop(100).unwrap();
        check(
            &mut rt,
            "urls.join(',') === 'https://parent.test/next.html' && held === f.contentWindow && original !== f.contentDocument && !!f.contentDocument.getElementById('next')",
        );
    }
}

fn initial_callback_can_remove_or_navigate<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval(
        "var log = [], f = document.createElement('iframe');
      f.onload = function() {
        log.push('load');
        f.contentWindow.addEventListener('unload', function() { log.push('unload'); });
        f.remove(); log.push(f.contentWindow === null ? 'detached' : 'attached');
      };
      document.body.appendChild(f); log.push('returned');",
    )
    .unwrap();
    check(&mut rt, "log.join(',') === 'load,detached,returned'");
    rt.run_event_loop(100).unwrap();
    check(&mut rt, "log.join(',') === 'load,detached,returned,unload'");
    assert!(rt.frame_realms(rt.top_realm()).is_empty());

    let mut rt = runtime::<E>();
    rt.eval("var log = [], f = document.createElement('iframe');
      f.addEventListener('load', function() { log.push('initial'); f.src = '/next.html'; }, {once:true});
      document.body.appendChild(f); log.push('returned');
      f.addEventListener('load', function() { log.push(f.contentWindow.location.href); });").unwrap();
    check(&mut rt, "log.join(',') === 'initial,returned'");
    rt.run_event_loop(100).unwrap();
    check(
        &mut rt,
        "log.join(',') === 'initial,returned,https://parent.test/next.html'",
    );
}

fn parser_initial_load_precedes_following_script<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("globalThis.log = []").unwrap();
    rt.parse_document_interleaved(
        "<html><body><script>document.addEventListener('load', function(e) { if (e.target.localName === 'iframe') log.push('load'); }, true)</script><iframe></iframe><script>log.push('script')</script></body></html>",
        &NoScriptLoader,
    );
    check(&mut rt, "log.join(',') === 'load,script'");
    rt.run_event_loop(100).unwrap();
    check(&mut rt, "log.join(',') === 'load,script'");
}

fn parser_load_callback_can_remove_its_frame<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.parse_document_interleaved(
        "<html><body><script>var removed = 0; document.addEventListener('load', function(e) { if (e.target.localName === 'iframe') { e.target.remove(); removed++; } }, true)</script><iframe></iframe><script>var clean = removed === 1 && window.length === 0 && window[0] === undefined;</script></body></html>",
        &NoScriptLoader,
    );
    check(&mut rt, "clean");
    rt.run_event_loop(100).unwrap();
    check(&mut rt, "window.length === 0 && window[0] === undefined");
}

fn later_blank_navigation_and_srcdoc_stay_queued<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval(
        "var log = [], f = document.createElement('iframe');
      document.body.appendChild(f); var original = f.contentDocument;
      f.onload = function() { log.push('load'); };
      f.src = 'about:blank'; log.push('requested');",
    )
    .unwrap();
    check(
        &mut rt,
        "log.join(',') === 'requested' && original === f.contentDocument",
    );
    rt.run_event_loop(100).unwrap();
    check(
        &mut rt,
        "log.join(',') === 'requested,load' && original !== f.contentDocument",
    );
    rt.eval(
        "var g = document.createElement('iframe'); g.srcdoc = '';
      g.onload = function() { log.push('srcdoc'); }; document.body.appendChild(g);",
    )
    .unwrap();
    check(&mut rt, "log.join(',') === 'requested,load'");
    rt.run_event_loop(100).unwrap();
    check(&mut rt, "log.join(',') === 'requested,load,srcdoc'");
}

macro_rules! both {
    ($($body:ident => ($boa:ident, $nova:ident)),* $(,)?) => { $(
        #[test] fn $boa() { $body::<script_engine_boa::BoaEngine>(); }
        #[cfg(target_pointer_width = "64")]
        #[test] fn $nova() { $body::<script_engine_nova::NovaEngine>(); }
    )* };
}
both! {
    initial_load_is_synchronous_and_once => (initial_boa, initial_nova),
    navigation_listener_cannot_catch_initial_load => (navigation_boa, navigation_nova),
    initial_callback_can_remove_or_navigate => (reentrant_boa, reentrant_nova),
    parser_initial_load_precedes_following_script => (parser_boa, parser_nova),
    parser_load_callback_can_remove_its_frame => (parser_removal_boa, parser_removal_nova),
    later_blank_navigation_and_srcdoc_stay_queued => (later_boa, later_nova),
}
