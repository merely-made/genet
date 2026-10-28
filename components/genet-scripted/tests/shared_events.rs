// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A host shares the document's listener registry: a Rust-owned listener sits
//! between two JS listeners on one node and one script-driven dispatch runs all
//! three in registration order, on both engines.

use genet_events::{Invocation, InvokeResult, Invoker, ListenerSpec, Owner, TargetKey};
use genet_scripted::ScriptedDocument;
use layout_dom_api::LayoutDom;
use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime};

const HOST: Owner = Owner(7);

/// The host's listener: it writes into the page's own console, so the order of
/// all three listeners is read from one place.
struct Host<H: Fn(&str)>(H);
impl<H: Fn(&str)> Invoker for Host<H> {
    fn invoke(&mut self, call: Invocation<'_>) -> InvokeResult {
        (self.0)(&format!("rust{}", call.listener.token));
        InvokeResult::default()
    }
}

fn button<E: ScriptEngine>(runtime: &Runtime<E>) -> genet_scripted_dom::NodeId {
    let host = runtime.host().borrow();
    let mut pending = vec![host.dom.document()];
    loop {
        let node = pending.pop().expect("button");
        if host.dom.element_name(node).is_some_and(|n| n.local.as_str() == "button") {
            return node;
        }
        pending.extend(host.dom.dom_children(node));
    }
}

fn the_run_order_is_registration_order<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime.parse_document_interleaved("<body><button id=b>go</button></body>", &NoScriptLoader);
    let node = button(&runtime);
    let shared = runtime.host().clone();
    runtime.set_foreign_invoker(Some(Box::new(Host(move |mark: &str| {
        shared.borrow_mut().console.push(mark.to_string());
    }))));
    runtime.host().borrow_mut().console.clear();
    runtime
        .eval(
            "var b = document.getElementById('b'); \
             b.addEventListener('click', function() { console.log('js1'); });",
        )
        .unwrap();
    runtime.events().borrow_mut().add(
        TargetKey::Node(node),
        ListenerSpec {
            owner: HOST,
            token: 1,
            event_type: "click".into(),
            capture: false,
            once: false,
            passive: false,
        },
    );
    runtime
        .eval("b.addEventListener('click', function() { console.log('js2'); });")
        .unwrap();
    runtime.eval("b.dispatchEvent(new Event('click'));").unwrap();
    assert_eq!(runtime.host().borrow().console, ["js1", "rust1", "js2"]);
}

fn a_document_hands_its_registry_to_the_host<E: ScriptEngine>() {
    let mut doc = ScriptedDocument::<E>::parse("<body><p id=p>x</p></body>").expect("document");
    let events = doc.events();
    let before = events.borrow().target_count();
    doc.evaluate("document.getElementById('p').addEventListener('click', function() {});")
        .expect("script");
    assert_eq!(events.borrow().target_count(), before + 1);
}

macro_rules! both_engines {
    ($name:ident) => {
        mod $name {
            #[test]
            fn boa() {
                super::$name::<script_engine_boa::BoaEngine>();
            }
            #[cfg(feature = "scripted-nova")]
            #[test]
            fn nova() {
                super::$name::<script_engine_nova::NovaEngine>();
            }
        }
    };
}

both_engines!(the_run_order_is_registration_order);
both_engines!(a_document_hands_its_registry_to_the_host);
