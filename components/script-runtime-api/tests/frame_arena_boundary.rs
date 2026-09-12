// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime};

fn associated_subtrees_travel_with_their_owner<E: ScriptEngine>() {
    let mut runtime = Runtime::<E>::new().unwrap();
    runtime
        .set_base_url("https://parent.test/index.html")
        .unwrap();
    runtime.parse_document_interleaved(
        "<body><section id=kept><template><b>hidden</b></template><p>original</p>         <object></object><embed></section><canvas id=painted></canvas>         <iframe></iframe></body>",
        &NoScriptLoader,
    );
    runtime.run_event_loop(20).unwrap();
    runtime.eval(r#"
        var child = document.querySelector('iframe').contentWindow;
        var kept = document.getElementById('kept');
        var originalDocument = kept.ownerDocument;
        var originalMarkup = kept.outerHTML, childMarkup = child.document.body.innerHTML;
        var template = kept.querySelector('template');
        var contents = template.content, hidden = contents.firstChild;
        var sourceOwner = contents.ownerDocument;
        // The inert owner is a document of its own, shared by every template in
        // the source arena and distinct from the page's document.
        if (!sourceOwner || sourceOwner === originalDocument || sourceOwner.nodeType !== 9)
            throw new Error('template contents must report the inert owner document');

        // A template-bearing subtree, and one carrying `object` and `embed`, is
        // no longer refused: nothing in it holds host state with no ownership
        // transaction.
        if (child.document.body.appendChild(kept) !== kept)
            throw new Error('associated-tree adoption must succeed');
        if (kept.ownerDocument !== child.document || kept.parentNode !== child.document.body)
            throw new Error('adopted host did not follow the destination document');
        if (kept.outerHTML !== originalMarkup)
            throw new Error('adoption changed the transferred markup');
        // Reflector identity holds through the move, for the element, its
        // contents fragment and a node inside that fragment.
        if (kept.querySelector('template') !== template || template.content !== contents ||
            contents.firstChild !== hidden || hidden.textContent !== 'hidden')
            throw new Error('adoption minted new reflectors for associated nodes');
        // HTML adopts the contents into the *destination's* inert template
        // document, not into the destination document and not into the source's.
        var destinationOwner = child.document.createElement('template').content.ownerDocument;
        if (contents.ownerDocument !== destinationOwner)
            throw new Error('template contents were not re-homed to the destination inert document');
        if (contents.ownerDocument === sourceOwner || contents.ownerDocument === child.document)
            throw new Error('template contents landed in the wrong document');

        // The residual refusal is a canvas that minted a drawing context: its
        // registry index and texture producer answer to the source host only.
        var painted = document.getElementById('painted');
        painted.getContext('webgl');
        var error;
        try { child.document.body.appendChild(painted); } catch (e) { error = e; }
        if (!error || String(error).indexOf('NotSupportedError') === -1)
            throw new Error('a canvas with a live context must refuse explicitly');
        if (painted.ownerDocument !== document || painted.parentNode !== document.body)
            throw new Error('a refused adoption moved the canvas anyway');
        // A canvas that never asked for a context carries nothing.
        var blank = document.createElement('canvas');
        document.body.appendChild(blank);
        if (child.document.body.appendChild(blank) !== blank || blank.ownerDocument !== child.document)
            throw new Error('a context-free canvas must adopt like any element');

        var movable = document.createElement('p'); movable.textContent='ordinary';
        document.body.appendChild(movable);
        var moveError;
        try { child.document.body.moveBefore(movable, null); } catch (e) { moveError = e; }
        if (!moveError || moveError.name !== 'HierarchyRequestError' ||
            movable.parentNode !== document.body)
            throw new Error('cross-document moveBefore must refuse without moving');
        if (child.document.body.appendChild(movable) !== movable ||
            movable.ownerDocument !== child.document || child.document.body.lastChild !== movable)
            throw new Error('ordinary cross-arena insertion lost identity');
        child.Element.prototype.setAttribute.call(movable,'data-owner','destination');
        if (movable.getAttribute('data-owner') !== 'destination')
            throw new Error('borrowed operation did not route physical owner');
        var local = child.document.createElement('p'); local.textContent='child-owned';
        child.document.body.appendChild(local);
        if (child.document.body.lastChild !== local || local.ownerDocument !== child.document)
            throw new Error('same-origin access lost child-owned identity');
        if (childMarkup === child.document.body.innerHTML)
            throw new Error('the destination document never changed');
        var secondary = new DOMParser().parseFromString('<p>secondary</p>', 'text/html');
        var sameArena = secondary.querySelector('p');
        document.body.appendChild(sameArena);
        if (sameArena.ownerDocument !== document || sameArena.textContent !== 'secondary')
            throw new Error('same-arena adoption regressed');
    "#).unwrap();
}

#[test]
fn associated_subtree_boundary_on_boa() {
    associated_subtrees_travel_with_their_owner::<script_engine_boa::BoaEngine>();
}

#[cfg(target_pointer_width = "64")]
#[test]
fn associated_subtree_boundary_on_nova() {
    associated_subtrees_travel_with_their_owner::<script_engine_nova::NovaEngine>();
}

fn full_width_host_identity<E: ScriptEngine>() {
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::LayoutDom;

    // Force actual allocated identities beyond the JS integer precision limit.
    // Each arena is dropped immediately; this does not retain thousands of DOMs.
    for _ in 0..8193 {
        drop(ScriptedDom::new());
    }
    let mut runtime = Runtime::<E>::new().unwrap();
    runtime.parse_document_interleaved(
        "<body><!--offset--><p id=kept>before</p></body>",
        &NoScriptLoader,
    );
    runtime.run_event_loop(20).unwrap();
    let node = {
        let host = runtime.host();
        let host = host.borrow();
        let mut pending = vec![host.dom.document()];
        loop {
            let id = pending.pop().unwrap();
            if host
                .dom
                .element_name(id)
                .is_some_and(|name| name.local.as_ref() == "p")
            {
                break id;
            }
            pending.extend(host.dom.dom_children(id));
        }
    };
    assert!(node.raw() > (1u64 << 53));
    assert_eq!(node.raw() & 1, 1, "fixture must expose loss of the low bit");
    runtime
        .eval(&format!(
            r#"
        var kept = document.getElementById('kept');
        if (__nodeRawId(kept.__ref) !== '{}') throw new Error('identity narrowed');
        if (__reflectNode('0') !== null || __reflectNode('18446744073709551615') !== null)
            throw new Error('unknown identity minted a reflector');
        kept.addEventListener('click', function() {{ kept.textContent = 'after'; }});
    "#,
            node.raw()
        ))
        .unwrap();
    runtime.dispatch_event(node.raw(), "click").unwrap();
    runtime.collect_garbage();
    runtime
        .eval(
            r#"
        if (document.getElementById('kept') !== kept || kept.textContent !== 'after')
            throw new Error('host dispatch or collection lost full-width identity');
    "#,
        )
        .unwrap();
}

#[test]
fn full_width_host_identity_on_boa() {
    full_width_host_identity::<script_engine_boa::BoaEngine>();
}

#[cfg(target_pointer_width = "64")]
#[test]
fn full_width_host_identity_on_nova() {
    full_width_host_identity::<script_engine_nova::NovaEngine>();
}
