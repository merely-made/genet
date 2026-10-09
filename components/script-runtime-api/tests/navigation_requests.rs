//! Network document navigation uses the host fetch seam when a source loader misses.
use script_engine_api::{MAIN_REALM, ScriptEngine};
use script_runtime_api::{FetchHandler, FetchOutcome, FetchRequest, NoScriptLoader, Runtime};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Default)]
struct Network(Rc<RefCell<Vec<String>>>);
impl FetchHandler for Network {
    fn fetch(&self, request: FetchRequest) -> FetchOutcome {
        self.0.borrow_mut().push(request.url.clone());
        if request.url.contains("/log/") {
            return FetchOutcome::network_error();
        }
        FetchOutcome { network_error: false, status: 200, status_text: "OK".into(),
            response_type: "basic".into(), url: "https://nav.test/final.html".into(), redirected: true,
            headers: vec![("content-type".into(), "text/html".into())],
            body: b"<p id=mark>final</p><script>addEventListener('load',function(){fetch('https://nav.test/log/incoming-load')})</script>".to_vec() }
    }
}
fn read<E: ScriptEngine>(r: &mut Runtime<E>, source: &str) -> String {
    let v = r.eval(source).unwrap();
    r.value_to_string(&v).unwrap()
}
fn network_redirect_navigation<E: ScriptEngine>() {
    let network = Network::default();
    let mut runtime = Runtime::<E>::new().unwrap();
    runtime.set_base_url("https://nav.test/start.html").unwrap();
    runtime.set_fetch_handler(Box::new(network.clone()));
    runtime.parse_document_interleaved("<p>start</p><script>addEventListener('pagehide',function(){fetch('https://nav.test/log/pagehide')});addEventListener('unload',function(){fetch('https://nav.test/log/unload')})</script>", &NoScriptLoader);
    runtime.run_event_loop(100).unwrap();
    let outgoing = runtime.top_realm();
    let window = runtime.eval("window").unwrap();
    runtime
        .engine_mut()
        .set_global_in_realm(MAIN_REALM, "held", &window)
        .unwrap();
    runtime
        .eval("location.href='https://nav.test/requested.html'")
        .unwrap();
    assert_eq!(runtime.top_realm(), outgoing, "navigation is queued");
    runtime.run_event_loop(100).unwrap();
    assert_eq!(
        read(&mut runtime, "location.href"),
        "https://nav.test/final.html"
    );
    assert_eq!(
        read(&mut runtime, "document.getElementById('mark').textContent"),
        "final"
    );
    assert_ne!(runtime.top_realm(), outgoing);
    let window = runtime.eval("window").unwrap();
    runtime
        .engine_mut()
        .set_global_in_realm(MAIN_REALM, "current", &window)
        .unwrap();
    let same = runtime.engine_mut().eval("String(held===current)").unwrap();
    assert_eq!(runtime.value_to_string(&same).unwrap(), "true");
    assert_eq!(
        &*network.0.borrow(),
        &[
            "https://nav.test/requested.html",
            "https://nav.test/log/pagehide",
            "https://nav.test/log/unload",
            "https://nav.test/log/incoming-load"
        ]
    );
    assert_eq!(read(&mut runtime, "history.length"), "2");
    runtime.eval("history.back()").unwrap();
    runtime.run_event_loop(100).unwrap();
    runtime.eval("history.forward()").unwrap();
    runtime.run_event_loop(100).unwrap();
    assert_eq!(
        network
            .0
            .borrow()
            .iter()
            .filter(|url| *url == "https://nav.test/final.html")
            .count(),
        1,
        "history retains the response's final URL"
    );
}
#[test]
fn boa_network_redirect_navigation() {
    network_redirect_navigation::<script_engine_boa::BoaEngine>();
}
#[cfg(target_pointer_width = "64")]
#[test]
fn nova_network_redirect_navigation() {
    network_redirect_navigation::<script_engine_nova::NovaEngine>();
}
