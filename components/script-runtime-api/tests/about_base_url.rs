// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use script_engine_api::{RealmId, ScriptEngine};
use script_runtime_api::{NoScriptLoader, Runtime, ScriptResourceLoader};
struct Documents;
impl ScriptResourceLoader for Documents {
    fn load(&self, url: &str) -> Option<String> {
        match url {
            "https://a.test/dir/code.js" => Some("globalThis.loadedScript = true".into()),
            "https://a.test/dir/child" => Some("<body>child</body>".into()),
            _ => None,
        }
    }
}
fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://a.test/dir/page").unwrap();
    rt.set_script_resource_loader(Box::new(Documents));
    rt.parse_document_interleaved("<body></body>", &NoScriptLoader);
    rt
}
fn check<E: ScriptEngine>(rt: &mut Runtime<E>, realm: RealmId, js: &str) {
    let v = rt.eval_in_realm(realm, js).expect(js);
    assert_eq!(rt.value_to_string(&v).unwrap(), "true", "{js}");
}
fn initial<E: ScriptEngine>() {
    for (attrs, expected) in [("", "about:blank"), ("f.src='about:blank?q#h';", "about:blank?q#h"), ("f.srcdoc='<p>srcdoc</p>';", "about:srcdoc"), ("f.setAttribute('loading','lazy'); f.src='/later';", "about:blank")] {
        let mut rt=runtime::<E>();
        rt.eval(&format!("var f=document.createElement('iframe'); {attrs} document.body.append(f);")).unwrap();
        let child=rt.frame_realms(rt.top_realm())[0].1;
        check(&mut rt, child, &format!("document.URL === '{expected}' && document.documentURI === '{expected}' && location.href === '{expected}' && location.origin === 'null' && origin === 'https://a.test' && document.baseURI === 'https://a.test/dir/page'"));
        check(&mut rt, child, "new Request('asset').url === 'https://a.test/dir/asset'");
        rt.eval("history.replaceState(null, '', '/changed');").unwrap();
        check(&mut rt, child, "document.baseURI === 'https://a.test/dir/page'");
        rt.eval("var retained=f.contentDocument; f.remove();").unwrap();
        let top=rt.top_realm();
        check(&mut rt, top, &format!("retained.URL === '{expected}' && retained.baseURI === 'https://a.test/dir/page'"));
        rt.run_event_loop(100).unwrap();
        check(&mut rt, top, &format!("retained.URL === '{expected}' && retained.documentURI === '{expected}'"));
    }
}
fn resources<E: ScriptEngine>() {
    let mut rt=runtime::<E>();
    rt.eval("var f=document.createElement('iframe'); f.srcdoc='<script src=code.js></script>'; document.body.append(f);").unwrap();
    rt.run_event_loop(100).unwrap();
    let child=rt.frame_realms(rt.top_realm())[0].1;
    check(&mut rt, child, "loadedScript === true && document.URL === 'about:srcdoc'");
    rt.eval_in_realm(child, "var nested=document.createElement('iframe'); nested.src='child'; document.body.append(nested);").unwrap();
    rt.run_event_loop(100).unwrap();
    check(&mut rt, child, "nested.contentWindow.location.href === 'https://a.test/dir/child'");
    rt.eval_in_realm(child, "location.assign('child')").unwrap();
    rt.run_event_loop(100).unwrap();
    let next=rt.frame_realms(rt.top_realm())[0].1;
    check(&mut rt, next, "location.href === 'https://a.test/dir/child' && document.baseURI === location.href");
}
fn queued_blank<E: ScriptEngine>() {
    let mut rt=runtime::<E>();
    rt.eval("var f=document.createElement('iframe'); f.src='child'; document.body.append(f);").unwrap();
    rt.run_event_loop(100).unwrap();
    rt.eval("f.src='about:blank?query#fragment'; history.replaceState(null, '', '/changed');").unwrap();
    rt.run_event_loop(100).unwrap();
    let child=rt.frame_realms(rt.top_realm())[0].1;
    check(&mut rt, child, "location.href === 'about:blank?query#fragment' && document.baseURI === 'https://a.test/dir/page' && new Request('asset').url === 'https://a.test/dir/asset'");
}
macro_rules! both { ($($f:ident => ($b:ident,$n:ident)),*) => { $(
    #[test] fn $b() { $f::<script_engine_boa::BoaEngine>(); }
    #[cfg(target_pointer_width="64")] #[test] fn $n() { $f::<script_engine_nova::NovaEngine>(); }
)* }; }
both! { initial => (initial_boa,initial_nova), resources => (resources_boa,resources_nova), queued_blank => (queued_boa,queued_nova) }
