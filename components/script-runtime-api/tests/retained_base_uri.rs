// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime};

fn check<E: ScriptEngine>(rt: &mut Runtime<E>, js: &str) {
    let value = rt.eval(js).expect(js);
    assert_eq!(rt.value_to_string(&value).unwrap(), "true", "{js}");
}
fn retained<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://a.test/parent/page").unwrap();
    rt.parse_document_interleaved("<body></body>", &NoScriptLoader);
    rt.eval("var f=document.createElement('iframe'); f.srcdoc='<base href=\"/child/\"><p id=p title=x>text</p>'; document.body.append(f);").unwrap();
    rt.run_event_loop(100).unwrap();
    rt.eval("var d=f.contentDocument; var n=d.getElementById('p'); var attr=n.getAttributeNode('title'); var detached=d.createAttribute('loose'); var getter=Object.getOwnPropertyDescriptor(Node.prototype,'baseURI').get;").unwrap();
    check(
        &mut rt,
        "(()=>{try { getter.call({}); return false; } catch(e) { return e instanceof TypeError; }})()",
    );
    check(
        &mut rt,
        "getter.call(n)==='https://a.test/child/' && getter.call(attr)==='https://a.test/child/' && getter.call(detached)==='https://a.test/child/'",
    );
    rt.eval("f.remove(); d.querySelector('base').setAttribute('href','/retained/');")
        .unwrap();
    check(
        &mut rt,
        "d.baseURI==='https://a.test/retained/' && n.baseURI===d.baseURI",
    );
    rt.run_event_loop(100).unwrap();
    rt.collect_garbage();
    check(
        &mut rt,
        "d.baseURI==='https://a.test/retained/' && n.baseURI===d.baseURI && attr.baseURI===d.baseURI && detached.baseURI===d.baseURI && getter.call(d)===d.baseURI && getter.call(n)===d.baseURI",
    );
}
fn adopted<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://a.test/parent/page").unwrap();
    rt.parse_document_interleaved(
        "<head><base href='/parent/'></head><body></body>",
        &NoScriptLoader,
    );
    rt.eval("var f=document.createElement('iframe'); f.srcdoc='<base href=\"/child/\"><p id=p title=x>text</p>'; document.body.append(f);").unwrap();
    rt.run_event_loop(100).unwrap();
    rt.eval("var n=f.contentDocument.getElementById('p'); var a=n.getAttributeNode('title'); document.adoptNode(n);").unwrap();
    check(
        &mut rt,
        "n.baseURI==='https://a.test/parent/' && a.baseURI===n.baseURI",
    );
    rt.eval("n.removeAttributeNode(a); f.remove(); document.querySelector('base').setAttribute('href','/new/');")
        .unwrap();
    rt.run_event_loop(100).unwrap();
    rt.collect_garbage();
    check(
        &mut rt,
        "n.baseURI==='https://a.test/new/' && a.baseURI===n.baseURI",
    );
}
#[test]
fn retained_boa() {
    retained::<script_engine_boa::BoaEngine>();
}
#[cfg(target_pointer_width = "64")]
#[test]
fn retained_nova() {
    retained::<script_engine_nova::NovaEngine>();
}
#[test]
fn adopted_boa() {
    adopted::<script_engine_boa::BoaEngine>();
}
#[cfg(target_pointer_width = "64")]
#[test]
fn adopted_nova() {
    adopted::<script_engine_nova::NovaEngine>();
}

#[test]
#[cfg(target_pointer_width = "64")]
fn snapshot_reader_nova() {
    let mut rt = Runtime::<script_engine_nova::NovaEngine>::new().unwrap();
    rt.set_base_url("https://a.test/old/page").unwrap();
    rt.parse_document_interleaved("<body><p id=p>text</p></body>", &NoScriptLoader);
    rt.eval("var held=document.getElementById('p'); var baseGetter=Object.getOwnPropertyDescriptor(Node.prototype,'baseURI').get;").unwrap();
    let mut cloned = rt.snapshot_clone().unwrap();
    drop(rt);
    cloned.set_base_url("https://a.test/new/page").unwrap();
    check(
        &mut cloned,
        "document.baseURI==='https://a.test/new/page' && baseGetter.call(document)===document.baseURI",
    );
}

fn document_only<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://a.test/parent/page").unwrap();
    rt.parse_document_interleaved("<body></body>", &NoScriptLoader);
    rt.eval("var d; var f=document.createElement('iframe'); f.srcdoc='<base href=\"/only/\"><p>text</p>'; document.body.append(f);").unwrap();
    rt.run_event_loop(100).unwrap();
    rt.eval("d=f.contentDocument; f.remove(); f=null;").unwrap();
    rt.run_event_loop(100).unwrap();
    rt.collect_garbage();
    check(&mut rt, "d.baseURI==='https://a.test/only/'");
}
#[test]
fn document_only_boa() {
    document_only::<script_engine_boa::BoaEngine>();
}
#[cfg(target_pointer_width = "64")]
#[test]
fn document_only_nova() {
    document_only::<script_engine_nova::NovaEngine>();
}
