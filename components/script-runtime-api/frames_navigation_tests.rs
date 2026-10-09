//! Test-only native entry drives private request navigation without a page API.
use super::*;
use crate::{
    FetchHandler, FetchOutcome, FetchRequest, NoScriptLoader, Runtime, ScriptResourceLoader,
};

#[derive(Clone, Default)]
struct Capture(Rc<RefCell<Vec<FetchRequest>>>);
impl FetchHandler for Capture {
    fn fetch(&self, request: FetchRequest) -> FetchOutcome {
        let failed = request.url.ends_with("/fail");
        self.0.borrow_mut().push(request);
        if failed {
            return FetchOutcome::network_error();
        }
        FetchOutcome {
            network_error: false,
            status: 200,
            status_text: "OK".into(),
            response_type: "basic".into(),
            url: "https://nav.test/final.html".into(),
            redirected: true,
            headers: vec![(
                "content-type".into(),
                "text/html; charset=windows-1252".into(),
            )],
            body: b"<p id=mark>caf\xe9</p>".to_vec(),
        }
    }
}
#[derive(Clone, Default)]
struct Loader(Rc<RefCell<Vec<String>>>);
impl ScriptResourceLoader for Loader {
    fn load(&self, url: &str) -> Option<String> {
        self.0.borrow_mut().push(url.into());
        Some("<p id=mark>loader GET</p>".into())
    }
}
struct Post;
impl<E: ScriptEngine> NativeFn<E> for Post {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let url = cx.arg(0);
        let url = cx.value_to_string(&url)?;
        let name = cx.arg(1);
        let name = cx.value_to_string(&name)?;
        let from = cx.current_realm();
        let Some(target) = navigation_target::<E>(cx, from, &name) else {
            return Ok(cx.undefined());
        };
        let request = FetchRequest {
            url,
            method: "POST".into(),
            headers: vec![("Content-Type".into(), "application/octet-stream".into())],
            body: Some(vec![0, 128, 255]),
            cache: "no-store".into(),
            redirect: "follow".into(),
            mode: "navigate".into(),
            referrer: "https://nav.test/form.html".into(),
            referrer_policy: "origin".into(),
            credentials: "include".into(),
            integrity: String::new(),
        };
        navigate_request::<E>(cx, from, target, request, false)
    }
}
fn setup<E: ScriptEngine>() -> (Runtime<E>, Capture, Loader) {
    let mut runtime = Runtime::<E>::new().unwrap();
    runtime.set_base_url("https://nav.test/form.html").unwrap();
    let capture = Capture::default();
    let loader = Loader::default();
    runtime.set_fetch_handler(Box::new(capture.clone()));
    runtime.set_script_resource_loader(Box::new(loader.clone()));
    runtime.parse_document_interleaved("<p>original</p>", &NoScriptLoader);
    runtime.run_event_loop(100).unwrap();
    let top = runtime.top_realm();
    runtime
        .engine_mut()
        .set_function_in_realm::<Post>(top, "testPost", 2)
        .unwrap();
    (runtime, capture, loader)
}
fn read<E: ScriptEngine>(runtime: &mut Runtime<E>, source: &str) -> String {
    let value = runtime.eval(source).unwrap();
    runtime.value_to_string(&value).unwrap()
}
fn post_fragment_and_failure<E: ScriptEngine>() {
    let (mut runtime, capture, loader) = setup::<E>();
    let initial = runtime.top_realm();
    runtime.eval("location.hash='get'").unwrap();
    assert_eq!(runtime.top_realm(), initial);
    assert!(capture.0.borrow().is_empty());
    runtime
        .eval("testPost('https://nav.test/form.html#post','_self')")
        .unwrap();
    assert_eq!(runtime.top_realm(), initial);
    assert!(capture.0.borrow().is_empty());
    runtime.run_event_loop(100).unwrap();
    assert_ne!(runtime.top_realm(), initial);
    assert!(
        loader.0.borrow().is_empty(),
        "POST cannot become loader GET"
    );
    assert_eq!(
        read(&mut runtime, "location.href"),
        "https://nav.test/final.html"
    );
    assert_eq!(
        read(&mut runtime, "document.getElementById('mark').textContent"),
        "caf\u{e9}"
    );
    let requests = capture.0.borrow();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method, "POST");
    assert_eq!(requests[0].url, "https://nav.test/form.html#post");
    assert_eq!(requests[0].body.as_deref(), Some(&[0, 128, 255][..]));
    assert_eq!(
        requests[0].headers,
        [("Content-Type".into(), "application/octet-stream".into())]
    );
    assert_eq!(requests[0].referrer, "https://nav.test/form.html");
    assert_eq!(requests[0].referrer_policy, "origin");
    assert_eq!(requests[0].cache, "no-store");
    assert_eq!(requests[0].redirect, "follow");
    assert_eq!(requests[0].mode, "navigate");
    assert_eq!(requests[0].credentials, "include");
    drop(requests);
    let agent = runtime.host().borrow().agent.upgrade().unwrap();
    let a = agent.borrow();
    let context = a.frames.contexts[&runtime.top_realm()];
    assert_eq!(
        a.frames
            .tree
            .as_ref()
            .unwrap()
            .get(context)
            .unwrap()
            .history()
            .current()
            .url,
        "https://nav.test/final.html"
    );
    drop(a);
    let incoming = runtime.top_realm();
    runtime
        .engine_mut()
        .set_function_in_realm::<Post>(incoming, "testPost", 2)
        .unwrap();
    runtime.eval("testPost('/fail','_self')").unwrap();
    runtime.run_event_loop(100).unwrap();
    assert_eq!(
        runtime.top_realm(),
        incoming,
        "transport failure retains the active document"
    );
    assert_eq!(
        read(&mut runtime, "document.getElementById('mark').textContent"),
        "caf\u{e9}"
    );
}
fn top_policy_blocks_post<E: ScriptEngine>() {
    let (mut runtime, capture, loader) = setup::<E>();
    runtime.set_top_level_navigation_policy(|_| false);
    let initial = runtime.top_realm();
    runtime.eval("testPost('/blocked','_top')").unwrap();
    runtime.run_event_loop(100).unwrap();
    assert_eq!(runtime.top_realm(), initial);
    assert!(capture.0.borrow().is_empty());
    assert!(loader.0.borrow().is_empty());
}
fn named_child_and_destroyed_target<E: ScriptEngine>() {
    let (mut runtime, capture, loader) = setup::<E>();
    runtime.eval("var frame=document.createElement('iframe');frame.name='Receipt';frame.src='/child.html';document.body.appendChild(frame)").unwrap();
    runtime.run_event_loop(100).unwrap();
    loader.0.borrow_mut().clear();
    runtime
        .eval("var held=frame.contentWindow; testPost('/posted','Receipt')")
        .unwrap();
    runtime.run_event_loop(100).unwrap();
    assert_eq!(read(&mut runtime, "held===frame.contentWindow"), "true");
    assert_eq!(
        read(&mut runtime, "frame.contentWindow.location.href"),
        "https://nav.test/final.html"
    );
    assert_eq!(
        read(
            &mut runtime,
            "frame.contentWindow.document.getElementById('mark').textContent"
        ),
        "caf\u{e9}"
    );
    assert_eq!(capture.0.borrow().len(), 1);
    assert!(loader.0.borrow().is_empty());
    runtime.eval("testPost('/wrong-case','receipt');testPost('/auxiliary','_blank');testPost('/missing','missing')").unwrap();
    runtime.run_event_loop(100).unwrap();
    assert_eq!(capture.0.borrow().len(), 1);
    runtime
        .eval("testPost('/dead','Receipt');frame.remove()")
        .unwrap();
    runtime.run_event_loop(100).unwrap();
    assert_eq!(
        capture.0.borrow().len(),
        1,
        "dead target is discarded before fetch"
    );
}
macro_rules! backend {
    ($name:ident,$engine:ty) => {
        mod $name {
            #[test]
            fn post_fragment_bytes_redirect_failure() {
                super::post_fragment_and_failure::<$engine>();
            }
            #[test]
            fn policy_blocks_post() {
                super::top_policy_blocks_post::<$engine>();
            }
            #[test]
            fn named_child_proxy_and_dead_target() {
                super::named_child_and_destroyed_target::<$engine>();
            }
        }
    };
}
backend!(boa, script_engine_boa::BoaEngine);
#[cfg(target_pointer_width = "64")]
backend!(nova, script_engine_nova::NovaEngine);
