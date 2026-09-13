// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime, ScriptResourceLoader};

struct Documents;
impl ScriptResourceLoader for Documents {
    fn load(&self, url: &str) -> Option<String> {
        Some(if url.ends_with("worker.js") {
            "postMessage(self.origin);".into()
        } else { "<body>loaded</body>".into() })
    }
}
fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://a.test:443/page").unwrap();
    rt.set_script_resource_loader(Box::new(Documents));
    rt.parse_document_interleaved("<body></body>", &NoScriptLoader);
    rt
}
fn check<E: ScriptEngine>(rt: &mut Runtime<E>, source: &str) {
    let value = rt.eval(source).expect(source);
    assert_eq!(rt.value_to_string(&value).unwrap(), "true", "{source}");
}
fn inherited<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    check(&mut rt, "origin === 'https://a.test' && self.origin === window.origin");
    rt.eval("var f = document.createElement('iframe'); document.body.append(f); var getter = Object.getOwnPropertyDescriptor(window, 'origin').get;").unwrap();
    check(&mut rt, "f.contentWindow.origin === 'https://a.test' && getter.call(f.contentWindow) === 'https://a.test'");
    rt.eval("var held = f.contentWindow; f.remove()").unwrap();
    rt.run_event_loop(100).unwrap();
    check(&mut rt, "held.origin === 'https://a.test'");
}
fn opaque<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("var f = document.createElement('iframe'); f.src = 'https://a.test/child'; f.setAttribute('sandbox', 'allow-scripts'); document.body.append(f);").unwrap();
    let child = rt.frame_realms(rt.top_realm())[0].1;
    let v = rt.eval_in_realm(child, "origin === 'null' && location.origin === 'https://a.test'").unwrap();
    assert_eq!(rt.value_to_string(&v).unwrap(), "true");
    check(&mut rt, "(function(){try { return f.contentWindow.origin; } catch(e) { return e.name === 'SecurityError'; }})()");
    for url in ["about:blank", "data:text/html,hello", "file:///tmp/test.html"] {
        let mut bare = Runtime::<E>::new().unwrap();
        bare.set_base_url(url).unwrap();
        check(&mut bare, "origin === 'null'");
    }
}
fn replaceable<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("var d = Object.getOwnPropertyDescriptor(window, 'origin'); var marker = {}; window.origin = marker;").unwrap();
    check(&mut rt, "origin === marker && Object.getOwnPropertyDescriptor(window, 'origin').writable && d.get.call(window) === 'https://a.test'");
    check(&mut rt, "(function(){try { d.get.call({}); } catch(e) { return e instanceof TypeError; } return false;})()");
    rt.eval("Object.defineProperty(window, 'origin', d); history.replaceState(null, '', '/next');").unwrap();
    check(&mut rt, "origin === 'https://a.test'");
}
fn navigation<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("var f = document.createElement('iframe'); document.body.append(f); var held = f.contentWindow; f.src = 'https://b.test/next';").unwrap();
    rt.run_event_loop(100).unwrap();
    let child = rt.frame_realms(rt.top_realm())[0].1;
    let v = rt.eval_in_realm(child, "origin === 'https://b.test'").unwrap();
    assert_eq!(rt.value_to_string(&v).unwrap(), "true");
    check(&mut rt, "(function(){try { Object.getOwnPropertyDescriptor(window, 'origin').get.call(held); } catch(e) { return e.name === 'SecurityError'; } return false;})()");
}
fn worker<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("var answer; var w = new Worker('/worker.js'); w.onmessage = function(e) { answer = e.data; w.terminate(); }; w.onerror = function(e) { answer = 'error: ' + e.message; };").unwrap();
    let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        rt.run_event_loop(100).unwrap();
        rt.pump_workers();
        rt.run_timers(64, 10000.0);
        let v = rt.eval("answer === 'https://a.test'").unwrap();
        if rt.value_to_string(&v).unwrap() == "true" { break; }
        let debug = rt.eval("String(answer)").unwrap();
        assert!(std::time::Instant::now() < until, "worker origin was not delivered: {}", rt.value_to_string(&debug).unwrap());
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
macro_rules! both { ($($f:ident => ($b:ident,$n:ident)),*) => { $(
    #[test] fn $b() { $f::<script_engine_boa::BoaEngine>(); }
    #[cfg(target_pointer_width="64")] #[test] fn $n() { $f::<script_engine_nova::NovaEngine>(); }
)* }; }
both! { inherited => (inherited_boa,inherited_nova), opaque => (opaque_boa,opaque_nova), replaceable => (replaceable_boa,replaceable_nova), navigation => (navigation_boa,navigation_nova), worker => (worker_boa,worker_nova) }
