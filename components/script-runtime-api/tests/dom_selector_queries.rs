// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Shared Cadency grammar and one-shot live DOM query snapshots.

use genet_static_dom::StaticDocument;
use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.load_dom(&StaticDocument::parse(
        "<!doctype html><html><body><div id='a:b' class='hit' data-note='x,y'></div><p class='hit'></p><p></p></body></html>",
    ));
    rt
}

fn read<E: ScriptEngine>(rt: &mut Runtime<E>, expression: &str) -> String {
    let value = rt.eval(expression).expect("selector expression");
    rt.value_to_string(&value).expect("selector result")
}

fn grammar_scope_and_errors<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    assert_eq!(
        read(&mut rt, r#"document.querySelectorAll('div + p, p ~ p').length"#),
        "2"
    );
    assert_eq!(
        read(&mut rt, r#"document.querySelector('#a\\:b').id"#),
        "a:b"
    );
    assert_eq!(
        read(&mut rt, r#"document.querySelectorAll('[data-note="x,y"], p').length"#),
        "3"
    );
    assert_eq!(
        read(&mut rt, r#"document.querySelector('DIV').id"#),
        "a:b"
    );
    assert_eq!(
        read(&mut rt, r#"document.querySelectorAll('svg linearGradient').length"#),
        "0"
    );
    assert_eq!(
        read(&mut rt, r#"var root = document.createElement('section'); root.innerHTML = '<p class="hit"></p>'; document.body.appendChild(root); root.querySelectorAll(':scope > p').length + ',' + root.querySelectorAll(':root > p').length + ',' + root.matches(':scope') + ',' + document.querySelector(':scope').tagName"#),
        "1,0,true,HTML"
    );
    assert_eq!(
        read(&mut rt, r#"function errorName(f) { try { f(); return 'none'; } catch (e) { return e.name; } }
            [errorName(function(){ document.querySelector('div,'); }),
             errorName(function(){ document.querySelectorAll('p, :unsupported'); }),
             errorName(function(){ root.matches('p +'); })].join(',')"#),
        "SyntaxError,SyntaxError,SyntaxError"
    );
    assert_eq!(
        read(&mut rt, r#"var required = document.createElement('input'); required.required = true;
            document.body.appendChild(required); [required.matches(':invalid'), document.querySelectorAll('input:invalid').length].join(',')"#),
        "true,1"
    );
}

fn case_shadow_and_snapshot<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    assert_eq!(
        read(&mut rt, r#"var xml = new DOMParser().parseFromString('<Root><Item DATA-X="A"/></Root>', 'text/xml');
            [xml.querySelectorAll('Item').length, xml.querySelectorAll('item').length,
             xml.querySelectorAll('[DATA-X]').length, xml.querySelectorAll('[data-x]').length].join(',')"#),
        "1,0,1,0"
    );
    assert_eq!(
        read(&mut rt, r#"var svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
            var gradient = document.createElementNS('http://www.w3.org/2000/svg', 'linearGradient');
            svg.appendChild(gradient); document.body.appendChild(svg);
            [document.querySelectorAll('linearGradient').length, document.querySelectorAll('lineargradient').length].join(',')"#),
        "1,0"
    );
    assert_eq!(
        read(&mut rt, r#"var host = document.createElement('div'); host.innerHTML = '<span class="hit"></span>';
            document.body.appendChild(host); var shadow = host.attachShadow({mode:'open'});
            shadow.innerHTML = '<span class="hit"></span>';
            [document.querySelectorAll('.hit').length, host.querySelectorAll('.hit').length,
             shadow.querySelectorAll('.hit').length, shadow.querySelectorAll('div span').length].join(',')"#),
        "3,1,1,0"
    );
    assert_eq!(
        read(&mut rt, r#"var snapshot = document.querySelectorAll('p'); var old = snapshot[0];
            old.remove(); [snapshot.length, snapshot[0] === old, document.querySelectorAll('p').length].join(',')"#),
        "2,true,1"
    );
    // The callback changes the DOM after the first handoff. The second member
    // must still arrive from the one selection made before either callback.
    assert_eq!(
        read(&mut rt, r#"var scope = document.createElement('div'); scope.innerHTML = '<i></i><i></i>';
            document.body.appendChild(scope); var first = scope.firstChild, last = scope.lastChild, seen = [];
            __querySelectorAll(scope.__ref, 'i', 'true', function(ref) {
                seen.push(ref === first.__ref ? 'first' : ref === last.__ref ? 'last' : 'other');
                if (seen.length === 1) scope.removeChild(last);
            });
            seen.join(',')"#),
        "first,last"
    );
}

#[test]
fn boa_selector_queries() {
    grammar_scope_and_errors::<script_engine_boa::BoaEngine>();
    case_shadow_and_snapshot::<script_engine_boa::BoaEngine>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn nova_selector_queries() {
    grammar_scope_and_errors::<script_engine_nova::NovaEngine>();
    case_shadow_and_snapshot::<script_engine_nova::NovaEngine>();
}
