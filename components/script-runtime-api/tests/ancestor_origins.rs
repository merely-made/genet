// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use script_engine_api::{RealmId, ScriptEngine};
use script_runtime_api::{NoScriptLoader, Runtime, ScriptResourceLoader};

struct Documents;
impl ScriptResourceLoader for Documents {
    fn load(&self, _: &str) -> Option<String> {
        Some("<body>loaded</body>".into())
    }
}
fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://a.test/page").unwrap();
    rt.set_script_resource_loader(Box::new(Documents));
    rt.parse_document_interleaved("<body></body>", &NoScriptLoader);
    rt
}
fn check<E: ScriptEngine>(rt: &mut Runtime<E>, source: &str) {
    let value = rt.eval(source).expect(source);
    assert_eq!(rt.value_to_string(&value).unwrap(), "true", "{source}");
}
fn lifetime<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("var f = document.createElement('iframe'); document.body.append(f); var loc = f.contentWindow.location, list = loc.ancestorOrigins;").unwrap();
    check(
        &mut rt,
        "list === loc.ancestorOrigins && list.length === 1 && list[0] === 'https://a.test'",
    );
    rt.eval("var listProto = Object.getPrototypeOf(list), getter = Object.getOwnPropertyDescriptor(location, 'ancestorOrigins').get;").unwrap();
    check(
        &mut rt,
        "getter.call(loc) === list && DOMStringList.prototype.item.call(list, 0) === 'https://a.test'",
    );
    rt.eval("f.remove()").unwrap();
    check(
        &mut rt,
        "Object.getPrototypeOf(getter.call(loc)) === listProto",
    );
    check(
        &mut rt,
        "loc.ancestorOrigins.length === 0 && loc.ancestorOrigins === loc.ancestorOrigins && loc.ancestorOrigins !== list && list[0] === 'https://a.test'",
    );
    rt.run_event_loop(100).unwrap();
    rt.collect_garbage();
    check(
        &mut rt,
        "loc.ancestorOrigins.length === 0 && list[0] === 'https://a.test'",
    );
    rt.eval("document.body.append(f); var old = f.contentWindow.location, prior = old.ancestorOrigins; f.src = '/next';").unwrap();
    rt.run_event_loop(100).unwrap();
    check(
        &mut rt,
        "old.ancestorOrigins.length === 0 && prior[0] === 'https://a.test' && f.contentWindow.location.ancestorOrigins[0] === 'https://a.test'",
    );
}
fn string_list<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("var f = document.createElement('iframe'); document.body.append(f); var list = f.contentWindow.location.ancestorOrigins;").unwrap();
    check(
        &mut rt,
        "(function(){ list[1] = 'forged'; list.extra = 3; return list[1] === undefined && list.extra === 3 && !Reflect.defineProperty(list, '2', {value:'forged'}) && !Reflect.deleteProperty(list, '0') && Reflect.deleteProperty(list, '1'); })()",
    );
    check(
        &mut rt,
        "Object.prototype.toString.call(list) === '[object DOMStringList]' && !Array.isArray(list) && list.item(0) === 'https://a.test' && list.item(-1) === null && list.item(4294967296) === list[0] && list.contains('https://a.test') && !list.contains('https://b.test') && Array.from(list).join() === 'https://a.test'",
    );
    check(
        &mut rt,
        "(function(){ var threw = 0; try { new DOMStringList(); } catch(e) { threw++; } try { DOMStringList.prototype.item.call({},0); } catch(e) { threw++; } try { list.item(); } catch(e) { threw++; } list[0] = 'forged'; list.length = 8; return threw === 3 && list[0] === 'https://a.test' && list.length === 1; })()",
    );
}
fn child<E: ScriptEngine>(
    rt: &mut Runtime<E>,
    parent: RealmId,
    url: &str,
    policy: &str,
) -> RealmId {
    rt.eval_in_realm(parent, &format!("var f = document.createElement('iframe'); f.src = '{url}'; f.referrerPolicy = '{policy}'; document.body.append(f);")).unwrap();
    rt.run_event_loop(100).unwrap();
    rt.frame_realms(parent).last().unwrap().1
}
fn masks<E: ScriptEngine>() {
    for (middle, policy, expected) in [
        ("https://a.test/child", "no-referrer", "null,null"),
        ("https://b.test/child", "no-referrer", "null,https://a.test"),
        ("https://b.test/child", "same-origin", "null,https://a.test"),
        (
            "https://a.test/child",
            "same-origin",
            "https://a.test,https://a.test",
        ),
        (
            "https://b.test/child",
            "origin",
            "https://b.test,https://a.test",
        ),
    ] {
        let mut rt = runtime::<E>();
        let top = rt.top_realm();
        let mid = child(&mut rt, top, middle, "");
        let leaf = child(&mut rt, mid, "https://a.test/leaf", policy);
        let value = rt
            .eval_in_realm(leaf, "Array.from(location.ancestorOrigins).join(',')")
            .unwrap();
        assert_eq!(rt.value_to_string(&value).unwrap(), expected);
    }
}
fn policy_snapshot<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("var f = document.createElement('iframe'); f.referrerPolicy = 'no-referrer'; document.body.append(f); var list = f.contentWindow.location.ancestorOrigins; f.referrerPolicy = '';").unwrap();
    check(
        &mut rt,
        "list[0] === 'null' && list === f.contentWindow.location.ancestorOrigins",
    );
    rt.eval("f.referrerPolicy = 'no-referrer'; f.src = '/next'; f.referrerPolicy = '';")
        .unwrap();
    rt.run_event_loop(100).unwrap();
    check(
        &mut rt,
        "f.contentWindow.location.ancestorOrigins[0] === 'null'",
    );
    rt.eval("f.src = '/again';").unwrap();
    rt.run_event_loop(100).unwrap();
    check(
        &mut rt,
        "f.contentWindow.location.ancestorOrigins[0] === 'https://a.test'",
    );
}
fn inherited_mask_and_security<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    let top = rt.top_realm();
    let mid = child(&mut rt, top, "https://a.test/child", "no-referrer");
    rt.eval("f.referrerPolicy = ''").unwrap();
    let leaf = child(&mut rt, mid, "https://b.test/leaf", "");
    let value = rt
        .eval_in_realm(leaf, "Array.from(location.ancestorOrigins).join(',')")
        .unwrap();
    assert_eq!(rt.value_to_string(&value).unwrap(), "https://a.test,null");
    let value = rt.eval_in_realm(mid, "(function(){try { f.contentWindow.location.ancestorOrigins; } catch(e) { return e.name === 'SecurityError'; } return false; })()").unwrap();
    assert_eq!(rt.value_to_string(&value).unwrap(), "true");
}
fn opaque_parent<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval("var f = document.createElement('iframe'); f.setAttribute('sandbox', 'allow-scripts'); document.body.append(f);").unwrap();
    let mid = rt.frame_realms(rt.top_realm())[0].1;
    let leaf = child(&mut rt, mid, "about:blank", "");
    let value = rt
        .eval_in_realm(leaf, "Array.from(location.ancestorOrigins).join(',')")
        .unwrap();
    assert_eq!(rt.value_to_string(&value).unwrap(), "null,https://a.test");
}

#[test]
#[cfg(target_pointer_width = "64")]
fn snapshot_clone_retains_location_list_on_nova() {
    let mut rt = Runtime::<script_engine_nova::NovaEngine>::new().unwrap();
    rt.eval("var heldList = location.ancestorOrigins").unwrap();
    let mut cloned = rt.snapshot_clone().unwrap();
    drop(rt);
    check(
        &mut cloned,
        "heldList === location.ancestorOrigins && heldList.length === 0 && DOMStringList.prototype.item.call(heldList, 0) === null",
    );
}

macro_rules! both { ($($f:ident => ($b:ident,$n:ident)),*) => { $(
    #[test] fn $b() { $f::<script_engine_boa::BoaEngine>(); }
    #[cfg(target_pointer_width="64")] #[test] fn $n() { $f::<script_engine_nova::NovaEngine>(); }
)* }; }
both! { lifetime => (lifetime_boa,lifetime_nova), string_list => (list_boa,list_nova), masks => (masks_boa,masks_nova), policy_snapshot => (snapshot_boa,snapshot_nova), inherited_mask_and_security => (inherited_boa,inherited_nova), opaque_parent => (opaque_boa,opaque_nova) }
