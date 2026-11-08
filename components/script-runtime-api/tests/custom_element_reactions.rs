// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! HTML's custom element reactions: a callback owed by a `[CEReactions]` member
//! has run by the time that member returns, in the order the arena recorded the
//! mutations, with callbacks taken from the definition. Reactions recorded with
//! no member on the stack (the parser) are delivered through the backup element
//! queue. Each body runs on both backends.

use genet_static_dom::StaticDocument;
use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime};

fn read<E: ScriptEngine>(rt: &mut Runtime<E>, expr: &str) -> String {
    let v = rt.eval(expr).expect("eval");
    rt.value_to_string(&v).expect("stringify")
}

fn loaded<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.load_dom(&StaticDocument::parse("<html><body></body></html>"));
    rt
}

fn define_reads_options_and_lifecycle_properties_in_order<E: ScriptEngine>() {
    let mut rt = loaded::<E>();
    assert_eq!(
        read(
            &mut rt,
            r#"(() => {
                const log = [];
                function Button() {}
                Button.prototype = new Proxy(Button.prototype, {
                    get(target, key) {
                        log.push(key);
                        return key === 'attributeChangedCallback' ? () => {} : undefined;
                    }
                });
                const constructor = new Proxy(Button, {
                    get(target, key) {
                        log.push(key);
                        return target[key];
                    }
                });
                customElements.define('x-button', constructor, {
                    get extends() { log.push('extends'); return 'button'; }
                });
                log.push(customElements.get('x-button') === constructor);
                return log.join('|');
            })()"#,
        ),
        "extends|prototype|connectedCallback|disconnectedCallback|connectedMoveCallback|\
         adoptedCallback|attributeChangedCallback|observedAttributes|disabledFeatures|formAssociated|true"
    );
}

fn define_rejects_unknown_extensions<E: ScriptEngine>() {
    let mut rt = loaded::<E>();
    assert_eq!(
        read(
            &mut rt,
            r#"(() => {
                const log = [];
                const constructor = new Proxy(class extends HTMLElement {}, {
                    get(target, key) { log.push(key); return target[key]; }
                });
                for (const tag of ['BUTTON', 'bgsound', '?', 'x-custom']) {
                    try {
                        customElements.define('x-invalid', constructor, { extends: tag });
                        log.push('accepted');
                    } catch (error) {
                        log.push(error.name);
                    }
                }
                return log.join('|');
            })()"#,
        ),
        "NotSupportedError|NotSupportedError|NotSupportedError|NotSupportedError"
    );
}

fn define_rejects_null_callbacks_and_releases_guard<E: ScriptEngine>() {
    let mut rt = loaded::<E>();
    assert_eq!(
        read(
            &mut rt,
            r#"(() => {
                class ElementWithCallback extends HTMLElement {}
                ElementWithCallback.prototype.connectedCallback = null;
                let result = 'accepted';
                try {
                    customElements.define('x-callback', ElementWithCallback);
                } catch (error) {
                    result = error.name;
                }
                if (result !== 'TypeError') return result;
                delete ElementWithCallback.prototype.connectedCallback;
                customElements.define('x-callback', ElementWithCallback);
                return result + '|' + (customElements.get('x-callback') === ElementWithCallback);
            })()"#,
        ),
        "TypeError|true"
    );
}

#[test]
fn define_property_order_on_boa() {
    define_reads_options_and_lifecycle_properties_in_order::<script_engine_boa::BoaEngine>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn define_property_order_on_nova() {
    define_reads_options_and_lifecycle_properties_in_order::<script_engine_nova::NovaEngine>();
}

#[test]
fn define_unknown_extensions_on_boa() {
    define_rejects_unknown_extensions::<script_engine_boa::BoaEngine>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn define_unknown_extensions_on_nova() {
    define_rejects_unknown_extensions::<script_engine_nova::NovaEngine>();
}

#[test]
fn define_null_callback_on_boa() {
    define_rejects_null_callbacks_and_releases_guard::<script_engine_boa::BoaEngine>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn define_null_callback_on_nova() {
    define_rejects_null_callbacks_and_releases_guard::<script_engine_nova::NovaEngine>();
}

fn reactions_run_before_the_member_returns<E: ScriptEngine>() {
    let mut rt = loaded::<E>();
    rt.eval(
        "var log = []; var other;\
         class XA extends HTMLElement {\
           static get observedAttributes() { return ['id', 'Mixed']; }\
           attributeChangedCallback(n, o, v, ns) { log.push('attr:' + n + ':' + o + ':' + v + ':' + ns); }\
           connectedCallback() { log.push('connected'); }\
           disconnectedCallback() { log.push('disconnected'); }\
           adoptedCallback(o, n) { log.push('adopted:' + (o === document) + ':' + (n === other)); }\
         }\
         customElements.define('x-a', XA);\
         var el = document.createElement('x-a');\
         el.setAttribute('id', 'one'); log.push('after-set');\
         el.setAttributeNS('urn:x', 'id', 'two'); log.push('after-setns');\
         el.setAttribute('mixed', 'm'); log.push('after-unobserved');\
         document.body.appendChild(el); log.push('after-append');\
         document.body.removeChild(el); log.push('after-remove');\
         other = document.implementation.createHTMLDocument('');\
         other.body.appendChild(el); log.push('after-adopt');\
         XA.prototype.connectedCallback = function() { log.push('replaced'); };\
         document.body.appendChild(el); log.push('after-reinsert');",
    )
    .expect("reaction script");
    assert_eq!(
        read(&mut rt, "log.join('|')"),
        "attr:id:null:one:null|after-set|attr:id:null:two:urn:x|after-setns|after-unobserved|\
         connected|after-append|disconnected|after-remove|\
         adopted:true:true|connected|after-adopt|\
         disconnected|adopted:false:false|connected|after-reinsert"
    );
}

fn a_throwing_callback_is_reported_and_the_queue_continues<E: ScriptEngine>() {
    let mut rt = loaded::<E>();
    rt.eval(
        "var log = [];\
         window.addEventListener('error', function(e) { log.push('error:' + e.message); e.preventDefault(); });\
         customElements.define('x-throws', class extends HTMLElement {\
           connectedCallback() { log.push('throws'); throw new Error('boom'); }\
         });\
         customElements.define('x-after', class extends HTMLElement {\
           connectedCallback() { log.push('after'); }\
         });\
         var div = document.createElement('div');\
         div.appendChild(document.createElement('x-throws'));\
         div.appendChild(document.createElement('x-after'));\
         document.body.appendChild(div); log.push('returned');",
    )
    .expect("throwing callback script");
    assert_eq!(
        read(&mut rt, "log.join('|')"),
        "throws|error:boom|after|returned"
    );
}

/// A parser insertion goes through no DOM member, so its connected reaction
/// reaches the element through the backup element queue.
fn a_parser_inserted_custom_element_is_connected<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.parse_document_interleaved(
        "<body><script>var log = [];\
         customElements.define('x-p', class extends HTMLElement {\
           connectedCallback() { log.push('connected:' + this.parentNode.localName); }\
         });</script>\
         <div><x-p></x-p></div>\
         <script>log.push('second-script');</script></body>",
        &NoScriptLoader,
    );
    rt.run_microtasks();
    assert_eq!(read(&mut rt, "log.join('|')"), "connected:div|second-script");
}

/// A failed upgrade leaves the element failed: the next definition's tree walk
/// must not rerun its constructor.
fn a_failed_upgrade_is_not_retried<E: ScriptEngine>() {
    let mut rt = loaded::<E>();
    rt.eval(
        "var runs = 0;         document.body.appendChild(document.createElement('x-fails'));         try { customElements.define('x-fails', class extends HTMLElement {           constructor() { runs++; throw new Error('no'); }         }); } catch (e) {}         customElements.define('x-later', class extends HTMLElement {});",
    )
    .expect("failed upgrade script");
    assert_eq!(read(&mut rt, "String(runs)"), "1");
}

#[test]
fn a_failed_upgrade_is_not_retried_on_boa() {
    a_failed_upgrade_is_not_retried::<script_engine_boa::BoaEngine>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_failed_upgrade_is_not_retried_on_nova() {
    a_failed_upgrade_is_not_retried::<script_engine_nova::NovaEngine>();
}

/// document.write runs the written source through the parser inside the call;
/// a custom element it inserts is connected before its children are parsed.
fn a_written_custom_element_is_connected_before_its_children<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.parse_document_interleaved(
        "<body><script>var seen = -1;         customElements.define('x-w', class extends HTMLElement {           connectedCallback() { seen = this.childNodes.length; }         });         document.write('<x-w>hello <b>world</b></x-w>');</script></body>",
        &NoScriptLoader,
    );
    assert_eq!(read(&mut rt, "String(seen)"), "0");
    assert_eq!(
        read(&mut rt, "document.querySelector('x-w').childNodes.length + ''"),
        "2"
    );
}

#[test]
fn a_written_custom_element_is_connected_before_its_children_on_boa() {
    a_written_custom_element_is_connected_before_its_children::<script_engine_boa::BoaEngine>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_written_custom_element_is_connected_before_its_children_on_nova() {
    a_written_custom_element_is_connected_before_its_children::<script_engine_nova::NovaEngine>();
}

#[test]
fn reactions_run_before_the_member_returns_on_boa() {
    reactions_run_before_the_member_returns::<script_engine_boa::BoaEngine>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn reactions_run_before_the_member_returns_on_nova() {
    reactions_run_before_the_member_returns::<script_engine_nova::NovaEngine>();
}

#[test]
fn a_throwing_callback_is_reported_and_the_queue_continues_on_boa() {
    a_throwing_callback_is_reported_and_the_queue_continues::<script_engine_boa::BoaEngine>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_throwing_callback_is_reported_and_the_queue_continues_on_nova() {
    a_throwing_callback_is_reported_and_the_queue_continues::<script_engine_nova::NovaEngine>();
}

#[test]
fn a_parser_inserted_custom_element_is_connected_on_boa() {
    a_parser_inserted_custom_element_is_connected::<script_engine_boa::BoaEngine>();
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_parser_inserted_custom_element_is_connected_on_nova() {
    a_parser_inserted_custom_element_is_connected::<script_engine_nova::NovaEngine>();
}
