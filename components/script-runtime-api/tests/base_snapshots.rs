// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime, ScriptResourceLoader};
struct Resources;
impl ScriptResourceLoader for Resources {
    fn load(&self, url: &str) -> Option<String> {
        match url {
            "https://a.test/assets/code.js" => Some("globalThis.loaded = true".into()),
            "https://a.test/assets/child" => Some("<body>loaded</body>".into()),
            _ => None,
        }
    }
}
fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://a.test/dir/page").unwrap();
    rt.set_script_resource_loader(Box::new(Resources));
    rt.parse_document_interleaved("<head></head><body></body>", &NoScriptLoader);
    rt
}
fn check<E: ScriptEngine>(rt: &mut Runtime<E>, js: &str) {
    let value = rt.eval(js).expect(js);
    assert_eq!(rt.value_to_string(&value).unwrap(), "true", "{js}");
}
fn snapshot<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    check(
        &mut rt,
        "var base=document.createElement('base'); base.setAttribute('href','/assets/'); document.head.append(base); var f=document.createElement('iframe'); document.body.append(f); var s=document.createElement('iframe'); s.srcdoc='<body>srcdoc</body>'; document.body.append(s); f.contentDocument.baseURI === 'https://a.test/assets/' && s.contentDocument.baseURI === 'https://a.test/assets/'",
    );
    check(
        &mut rt,
        "base.setAttribute('href','/later/'); var next=document.createElement('iframe'); document.body.append(next); f.contentDocument.baseURI === 'https://a.test/assets/' && s.contentDocument.baseURI === 'https://a.test/assets/' && next.contentDocument.baseURI === 'https://a.test/later/'",
    );
    rt.eval("f.src='about:blank'; base.setAttribute('href','/latest/');")
        .unwrap();
    rt.run_event_loop(100).unwrap();
    check(
        &mut rt,
        "f.contentDocument.baseURI === 'https://a.test/later/'",
    );
}
fn cross_origin_base<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    check(
        &mut rt,
        "document.head.innerHTML='<base href=https://cdn.test/assets/>'; var f=document.createElement('iframe'); document.body.append(f); origin === 'https://a.test' && f.contentWindow.origin === 'https://a.test' && f.contentDocument.baseURI === 'https://cdn.test/assets/' && new f.contentWindow.Request('asset').url === 'https://cdn.test/assets/asset'",
    );
}
fn frozen<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    // No baseURI read before changing the document URL.
    rt.eval("var base=document.createElement('base'); base.setAttribute('href','relative/'); document.head.append(base); history.replaceState(null,'','/moved/page');").unwrap();
    check(
        &mut rt,
        "document.baseURI === 'https://a.test/dir/relative/'",
    );
    check(
        &mut rt,
        "document.body.append(document.createElement('div')); document.baseURI === 'https://a.test/dir/relative/'",
    );
    check(
        &mut rt,
        "base.setAttribute('href','relative/'); document.baseURI === 'https://a.test/moved/relative/'",
    );
    rt.eval(
        "history.replaceState(null,'','/again/page'); base.remove(); document.head.append(base);",
    )
    .unwrap();
    check(
        &mut rt,
        "document.baseURI === 'https://a.test/again/relative/'",
    );
    rt.eval("history.replaceState(null,'','/atomic/page'); document.head.append(base);")
        .unwrap();
    check(
        &mut rt,
        "document.baseURI === 'https://a.test/atomic/relative/'",
    );
    rt.eval(
        "history.replaceState(null,'','/preserved/page'); document.head.moveBefore(base,null);",
    )
    .unwrap();
    check(
        &mut rt,
        "document.baseURI === 'https://a.test/atomic/relative/'",
    );
}
fn first_and_invalid<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    check(
        &mut rt,
        "document.head.innerHTML='<base target=x><base href=/first/><base href=/second/>'; document.baseURI === 'https://a.test/first/'",
    );
    check(
        &mut rt,
        "document.head.children[1].setAttribute('href','relative/'); document.head.children[2].setAttribute('href','second/'); document.head.children[1].href === 'https://a.test/dir/relative/' && document.head.children[2].href === 'https://a.test/dir/second/'",
    );
    check(
        &mut rt,
        "document.head.children[2].setAttribute('href','/second/'); true",
    );
    for href in ["http://[", "javascript:1", "data:text/plain,test"] {
        check(
            &mut rt,
            &format!(
                "document.head.children[1].setAttribute('href','{href}'); document.baseURI === 'https://a.test/dir/page'"
            ),
        );
    }
    check(
        &mut rt,
        "document.head.children[1].removeAttribute('href'); document.baseURI === 'https://a.test/second/'",
    );
    check(
        &mut rt,
        "document.head.children[0].setAttribute('href','../first/'); document.baseURI === 'https://a.test/first/'",
    );
    check(
        &mut rt,
        "document.head.insertBefore(document.head.children[2],document.head.children[0]); document.baseURI === 'https://a.test/second/'",
    );
    check(
        &mut rt,
        "document.head.innerHTML=''; var foreign=document.createElementNS('http://www.w3.org/2000/svg','base'); foreign.setAttribute('href','/foreign/'); document.head.append(foreign); document.baseURI === 'https://a.test/dir/page'",
    );
}
fn resources<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("document.head.innerHTML='<base href=/assets/>'; var f=document.createElement('iframe'); f.srcdoc='<script src=code.js></script>'; document.body.append(f);").unwrap();
    rt.run_event_loop(100).unwrap();
    check(
        &mut rt,
        "f.contentWindow.loaded === true && new Request('child').url === 'https://a.test/assets/child'",
    );
    rt.eval("var c=document.createElement('iframe'); c.src='child'; document.body.append(c);")
        .unwrap();
    rt.run_event_loop(100).unwrap();
    check(
        &mut rt,
        "c.contentWindow.location.href === 'https://a.test/assets/child'",
    );
}
macro_rules! both { ($($test:ident => ($boa:ident,$nova:ident)),*) => { $(
    #[test] fn $boa() { $test::<script_engine_boa::BoaEngine>(); }
    #[cfg(target_pointer_width="64")] #[test] fn $nova() { $test::<script_engine_nova::NovaEngine>(); }
)* }; }
both! { cross_origin_base => (cross_origin_boa,cross_origin_nova), snapshot => (snapshot_boa,snapshot_nova), frozen => (frozen_boa,frozen_nova), first_and_invalid => (first_boa,first_nova), resources => (resources_boa,resources_nova) }
