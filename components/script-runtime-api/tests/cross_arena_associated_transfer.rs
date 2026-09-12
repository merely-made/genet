// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Adoption of subtrees that carry associated state: shadow trees, template
//! contents, and a subtree leaving a document the parser is still working in.
//!
//! DOM's adopt steps move a node's **shadow-including** inclusive descendants,
//! so a host's shadow root, that root's nodes and its nested hosts travel with
//! it, closed roots included. HTML's "adopt the template's contents" re-homes a
//! template's contents fragment onto the destination's inert template-contents
//! owner document. And HTML forbids moving only what the tree builder still
//! holds a handle on - a completed sibling subtree may leave mid-parse.

use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime};

fn in_two_documents<E: ScriptEngine>(source: &str) {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://parent.test/").unwrap();
    rt.parse_document_interleaved("<body><iframe id=frame></iframe></body>", &NoScriptLoader);
    rt.run_event_loop(20).unwrap();
    rt.eval("var child = document.getElementById('frame').contentWindow;")
        .unwrap();
    rt.eval(source).expect(source);
}

/// As above, but `after` runs once the event loop has drained - custom element
/// reactions are queued, not synchronous, so an `adoptedCallback` is only
/// observable on the far side of a turn.
fn in_two_documents_then<E: ScriptEngine>(source: &str, after: &str) {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://parent.test/").unwrap();
    rt.parse_document_interleaved("<body><iframe id=frame></iframe></body>", &NoScriptLoader);
    rt.run_event_loop(20).unwrap();
    rt.eval("var child = document.getElementById('frame').contentWindow;")
        .unwrap();
    rt.eval(source).expect(source);
    rt.run_event_loop(20).unwrap();
    rt.eval(after).expect(after);
}

/// A host carries its whole shadow tree: the root, its nodes, its nested hosts
/// (open or closed), the slot assignment table, and the adopted-callback fan-out
/// for custom elements living inside it. Every reflector survives the move.
fn a_shadow_tree_travels_with_its_host<E: ScriptEngine>() {
    in_two_documents_then::<E>(
        r#"
        globalThis.adopted = [];
        customElements.define('x-adoptee', class extends HTMLElement {
            adoptedCallback(oldDoc, newDoc) { adopted.push([this, oldDoc, newDoc]); }
        });
        var host = document.createElement('div');
        var root = host.attachShadow({mode: 'open'});
        var slot = document.createElement('slot'); root.appendChild(slot);
        var innerHost = document.createElement('span'); root.appendChild(innerHost);
        var innerRoot = innerHost.attachShadow({mode: 'closed'});
        var deep = document.createElement('x-adoptee');
        innerRoot.appendChild(deep);
        var light = document.createElement('p'); light.textContent = 'slotted';
        host.appendChild(light);
        document.body.appendChild(host);
        if (slot.assignedNodes()[0] !== light || light.assignedSlot !== slot)
            throw new Error('fixture did not assign the slottable');
        if (root.ownerDocument !== document) throw new Error('fixture shadow root owner');

        if (child.document.body.appendChild(host) !== host)
            throw new Error('a shadow host must adopt');
        // The host and every node of its shadow tree, nested roots included,
        // now belong to the destination - as the same reflectors.
        if (host.ownerDocument !== child.document || host.parentNode !== child.document.body)
            throw new Error('the host did not follow the destination document');
        if (host.shadowRoot !== root || root.ownerDocument !== child.document)
            throw new Error("the shadow root's ownerDocument did not follow its host");
        if (root.host !== host || root.firstChild !== slot || root.lastChild !== innerHost)
            throw new Error('the shadow tree lost its structure or its reflectors');
        if (innerHost.ownerDocument !== child.document || deep.ownerDocument !== child.document)
            throw new Error('a nested shadow tree did not travel');
        // A closed root stays closed on the far side.
        if (innerHost.shadowRoot !== null) throw new Error('a closed root was revealed by adoption');
        if (innerRoot.host !== innerHost || innerRoot.firstChild !== deep)
            throw new Error('the closed root lost its contents');
        // The slot assignment table came across intact.
        if (slot.assignedNodes().length !== 1 || slot.assignedNodes()[0] !== light ||
            light.assignedSlot !== slot)
            throw new Error('slot assignment did not survive adoption');
        globalThis.deep = deep;
    "#,
        // The adopted callback is owed to a custom element inside the shadow
        // tree exactly as to one in the light tree. Reactions are queued, so
        // this reads on the far side of a turn.
        r#"
        if (adopted.length !== 1 || adopted[0][0] !== deep ||
            adopted[0][1] !== document || adopted[0][2] !== child.document)
            throw new Error('adoptedCallback did not fan out into the shadow tree: ' + adopted.length);
    "#,
    );
}

/// `adoptNode` reaches the same place as an insertion, and a second hop back
/// returns everything to where it started.
fn a_shadow_tree_round_trips_through_adopt_node<E: ScriptEngine>() {
    in_two_documents::<E>(
        r#"
        var host = document.createElement('div');
        var root = host.attachShadow({mode: 'open'});
        var inside = document.createElement('b'); inside.textContent = 'shadow';
        root.appendChild(inside);
        document.body.appendChild(host);

        if (child.document.adoptNode(host) !== host) throw new Error('adoptNode must return the host');
        if (host.parentNode !== null) throw new Error('adoptNode must detach');
        if (host.ownerDocument !== child.document || inside.ownerDocument !== child.document ||
            host.shadowRoot !== root || root.firstChild !== inside)
            throw new Error('adoptNode did not carry the shadow tree');
        child.document.body.appendChild(host);
        if (document.adoptNode(host) !== host || host.ownerDocument !== document ||
            inside.ownerDocument !== document || host.shadowRoot !== root ||
            root.firstChild !== inside || inside.textContent !== 'shadow')
            throw new Error('the return hop lost the shadow tree');
    "#,
    );
}

/// A template's contents fragment is re-homed onto the destination's inert
/// template-contents owner document, recursively for nested templates, and the
/// fragment and its nodes keep their reflectors.
fn template_contents_are_rehomed_recursively<E: ScriptEngine>() {
    in_two_documents::<E>(
        r#"
        var outer = document.createElement('template');
        outer.innerHTML = '<i>one</i><template><b>two</b></template>';
        var contents = outer.content;
        var inner = contents.lastChild, innerContents = inner.content;
        var leaf = innerContents.firstChild;
        var sourceOwner = contents.ownerDocument, sourceMarkup = outer.innerHTML;
        if (innerContents.ownerDocument !== sourceOwner)
            throw new Error('fixture: two templates in one arena share one inert owner');
        document.body.appendChild(outer);

        child.document.body.appendChild(outer);
        var destinationOwner = child.document.createElement('template').content.ownerDocument;
        if (outer.ownerDocument !== child.document) throw new Error('the template did not adopt');
        if (outer.content !== contents || contents.lastChild !== inner ||
            inner.content !== innerContents || innerContents.firstChild !== leaf ||
            leaf.textContent !== 'two')
            throw new Error('adoption minted new reflectors for template contents');
        if (contents.ownerDocument !== destinationOwner ||
            innerContents.ownerDocument !== destinationOwner)
            throw new Error('nested template contents were not re-homed');
        if (destinationOwner === sourceOwner || destinationOwner === child.document)
            throw new Error('the destination inert owner is the wrong document');
        if (outer.innerHTML !== sourceMarkup)
            throw new Error('adoption changed the template serialization');
    "#,
    );
}

/// HTML refuses a move only for what the tree builder still holds: the stack of
/// open elements and the form element pointer. A script running during the parse
/// may adopt a *completed* sibling subtree into another document - which is what
/// `dom/nodes/Node-isConnected.html: Test with iframes` does.
fn a_completed_sibling_subtree_may_leave_a_parsing_document<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().unwrap();
    rt.set_base_url("https://parent.test/").unwrap();
    rt.parse_document_interleaved(
        "<body><iframe id=frame></iframe><div id=done><span>inner</span></div>\
         <script>\
         var log = {};\
         var frame = document.getElementById('frame');\
         var done = document.getElementById('done');\
         var open = document.body;\
         log.parsing = document.readyState;\
         try { frame.contentDocument.body.appendChild(done); log.moved = 'yes'; }\
         catch (e) { log.moved = String(e); }\
         try { frame.contentDocument.body.appendChild(open); log.openMove = 'admitted'; }\
         catch (e) { log.openMove = e.name || String(e); }\
         </script><p id=after>after</p></body>",
        &NoScriptLoader,
    );
    rt.run_event_loop(20).unwrap();
    rt.eval(
        r#"
        if (log.parsing !== 'loading')
            throw new Error('fixture did not run the script during the parse: ' + log.parsing);
        // The completed sibling left the parsing document.
        if (log.moved !== 'yes') throw new Error('a completed subtree was refused: ' + log.moved);
        if (done.ownerDocument !== frame.contentDocument ||
            done.parentNode !== frame.contentDocument.body ||
            done.firstChild.textContent !== 'inner')
            throw new Error('the adopted subtree did not arrive intact');
        if (done.isConnected !== true) throw new Error('the adopted subtree is not connected');
        // `document.body` was on the stack of open elements, so it was refused,
        // and the refusal moved nothing.
        if (log.openMove === 'admitted')
            throw new Error("the parser's stack of open elements was robbed");
        if (document.body.ownerDocument !== document || document.body.parentNode !== document.documentElement)
            throw new Error('a refused transfer moved the open element anyway');
        // The parse finished normally on the far side of both.
        if (!document.getElementById('after') || document.getElementById('done'))
            throw new Error('the parse did not complete over the transferred subtree');
    "#,
    )
    .unwrap();
}

macro_rules! backend {
    ($name:ident, $engine:ty) => {
        mod $name {
            #[test]
            fn shadow_tree_travels_with_host() {
                super::a_shadow_tree_travels_with_its_host::<$engine>();
            }
            #[test]
            fn shadow_tree_round_trip() {
                super::a_shadow_tree_round_trips_through_adopt_node::<$engine>();
            }
            #[test]
            fn template_contents_rehomed() {
                super::template_contents_are_rehomed_recursively::<$engine>();
            }
            #[test]
            fn completed_sibling_leaves_a_parsing_document() {
                super::a_completed_sibling_subtree_may_leave_a_parsing_document::<$engine>();
            }
        }
    };
}
backend!(boa, script_engine_boa::BoaEngine);
#[cfg(target_pointer_width = "64")]
backend!(nova, script_engine_nova::NovaEngine);
