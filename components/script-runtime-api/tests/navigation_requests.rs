//! Network document navigation uses the host fetch seam when a source loader misses.
use layout_dom_api::LayoutDom;
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

// TASK5 DRAFT RUNTIME FIXTURES BEGIN. Actual form request; supplied response contract.
#[derive(Clone)]
struct T5RedirectReply {
    final_url: String,
    requests: Rc<RefCell<Vec<FetchRequest>>>,
}
impl FetchHandler for T5RedirectReply {
    fn fetch(&self, request: FetchRequest) -> FetchOutcome {
        let request_url = request.url.clone();
        self.requests.borrow_mut().push(request);
        let url = if request_url.starts_with("https://nav.test/start.html") {
            request_url
        } else { self.final_url.clone() };
        FetchOutcome { network_error: false, status: 200, status_text: "OK".into(),
            response_type: "basic".into(), url, redirected: true,
            headers: vec![("content-type".into(), "text/html".into())],
            body: b"<html><body><p id=final>FINAL</p></body></html>".to_vec() }
    }
}
fn t5_form_redirect_fragment_identity<E: ScriptEngine>() {
    // Explicit controls first; absence case must fail after real request/source controls.
    for (supplied, want) in [
        ("https://nav.test/final.html#other", "https://nav.test/final.html#other"),
        ("https://nav.test/final.html#", "https://nav.test/final.html#"),
        ("https://nav.test/final.html", "https://nav.test/final.html#done"),
    ] {
        let requests = Rc::new(RefCell::new(Vec::new()));
        let mut rt = Runtime::<E>::new().unwrap();
        rt.set_base_url("https://nav.test/start.html#start").unwrap();
        rt.set_fetch_handler(Box::new(T5RedirectReply { final_url: supplied.into(), requests: requests.clone() }));
        rt.parse_document_interleaved(
            "<html><body><form id=f action='https://nav.test/old#done' method=post><input name=q value='a b'><button id=b name=go value=send type=submit>send</button></form><script>var events=[];var f=document.getElementById('f');f.addEventListener('submit',function(){events.push('submit')});f.addEventListener('formdata',function(){events.push('formdata')});</script></body></html>",
            &NoScriptLoader);
        rt.run_event_loop(100).unwrap();
        let realm = rt.top_realm();
        let document = rt.host().borrow().dom.document();
        let held = rt.eval("window").unwrap();
        rt.engine_mut().set_global_in_realm(MAIN_REALM, "t5Held", &held).unwrap();
        rt.eval("document.getElementById('f').requestSubmit(document.getElementById('b'))").unwrap();
        assert_eq!(read(&mut rt, "events.join(',')"), "submit,formdata");
        assert_eq!(rt.top_realm(), realm, "real form navigation remains queued");
        assert!(requests.borrow().is_empty());
        rt.run_event_loop(100).unwrap();
        {
            let log = requests.borrow();
            assert_eq!(log.len(), 1, "one actual host navigation fetch");
            let request = &log[0];
            assert_eq!(request.url, "https://nav.test/old#done");
            assert_eq!(request.method, "POST");
            assert_eq!(request.body.as_deref(), Some(b"q=a+b&go=send".as_slice()));
            assert!(request.headers.iter().any(|(k,v)| k.eq_ignore_ascii_case("content-type") && v=="application/x-www-form-urlencoded"));
            assert_eq!(request.referrer, "https://nav.test/start.html#start");
            assert_eq!((&*request.mode, &*request.credentials, &*request.redirect), ("navigate", "include", "follow"));
            assert_eq!(request.cache,"default");assert_eq!(request.referrer_policy,"");assert_eq!(request.integrity,"");
        }
        assert_ne!(rt.top_realm(), realm);
        assert_ne!(rt.host().borrow().dom.document(), document);
        assert_eq!(read(&mut rt, "document.getElementById('final').textContent"), "FINAL");
        let current = rt.eval("window").unwrap();
        rt.engine_mut().set_global_in_realm(MAIN_REALM, "t5Current", &current).unwrap();
        let same = rt.engine_mut().eval("String(t5Held===t5Current)").unwrap();
        assert_eq!(rt.value_to_string(&same).unwrap(), "true");
        assert_eq!(rt.host().borrow().base_url.as_deref(), Some(want), "real navigation must retain null-versus-empty fragment identity");
        assert_eq!(read(&mut rt, "history.length"), "2");
        rt.eval("history.back()").unwrap();
        rt.run_event_loop(100).unwrap();
        assert_eq!(rt.host().borrow().base_url.as_deref(), Some("https://nav.test/start.html#start"));
        rt.eval("history.forward()").unwrap();
        rt.run_event_loop(100).unwrap();
        assert_eq!(rt.host().borrow().base_url.as_deref(), Some(want));
        assert!(requests.borrow().iter().any(|r| r.url==want), "history traverses the retained final identity");
    }
}
#[test]
fn boa_phase_c_t5_runtime_form_redirect_fragment_identity() {
    t5_form_redirect_fragment_identity::<script_engine_boa::BoaEngine>();
}
#[cfg(target_pointer_width="64")]
#[test]
fn nova_phase_c_t5_runtime_form_redirect_fragment_identity() {
    t5_form_redirect_fragment_identity::<script_engine_nova::NovaEngine>();
}
// TASK5 DRAFT RUNTIME FIXTURES END.
