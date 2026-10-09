// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Cross-realm constraint-validation regressions. These deliberately exercise
//! public wrappers and state across same-origin iframe arenas.

use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime};

fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut runtime = Runtime::<E>::new().expect("runtime");
    runtime
        .set_base_url("https://parent.test/forms.html")
        .expect("base URL");
    runtime.parse_document_interleaved("<html><body></body></html>", &NoScriptLoader);
    runtime
}

fn run<E: ScriptEngine>(runtime: &mut Runtime<E>, source: &str) {
    if let Err(error) = runtime.eval(source) {
        panic!("{source}: {}", runtime.describe_error(&error));
    }
}

fn drain<E: ScriptEngine>(runtime: &mut Runtime<E>) {
    runtime.run_event_loop(100).expect("iframe parser tasks");
}

fn child_form_document<E: ScriptEngine>() -> Runtime<E> {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        r#"globalThis.frame = document.createElement('iframe');
           frame.srcdoc = '<html><body><script>\
             parent.childWindow = window;\
             var childInput = document.createElement("input");\
             childInput.required = true;\
             document.body.appendChild(childInput);\
             parent.childInput = childInput;\
           <\/script></body></html>';
           document.body.appendChild(frame);"#,
    );
    drain(&mut runtime);
    runtime
}

fn borrowed_input_validation_methods_use_child_native_state<E: ScriptEngine>() {
    let mut runtime = child_form_document::<E>();
    run(
        &mut runtime,
        r#"
        const borrowedValidity = Object.getOwnPropertyDescriptor(
          HTMLInputElement.prototype, 'validity').get;
        const borrowedCheck = HTMLInputElement.prototype.checkValidity;
        const childValidity = borrowedValidity.call(childInput);
        if (!(childValidity instanceof childWindow.ValidityState) ||
            childValidity !== childInput.validity || !childValidity.valueMissing || childValidity.valid)
          throw new Error('borrowed validity getter did not read child input state');
        childInput.addEventListener('invalid', event => { parent.savedBorrowedInvalid = event; });
        if (borrowedCheck.call(childInput) !== false)
          throw new Error('borrowed checkValidity did not use child native state');
        const originalChildEvent = childWindow.Event;
        if (!(savedBorrowedInvalid instanceof childWindow.Event) ||
            savedBorrowedInvalid instanceof Event ||
            Object.getPrototypeOf(savedBorrowedInvalid) !== childWindow.Event.prototype ||
            !savedBorrowedInvalid.isTrusted || savedBorrowedInvalid.bubbles ||
            !savedBorrowedInvalid.cancelable)
          throw new Error('borrowed validation created invalid event in caller realm');

        childWindow.Event = function PoisonedEvent() {
          throw new Error('authored child Event constructor was invoked');
        };
        if (borrowedCheck.call(childInput) !== false)
          throw new Error('borrowed checkValidity failed after child Event replacement');
        if (!(savedBorrowedInvalid instanceof originalChildEvent) ||
            savedBorrowedInvalid instanceof childWindow.Event ||
            savedBorrowedInvalid instanceof Event ||
            Object.getPrototypeOf(savedBorrowedInvalid) !== originalChildEvent.prototype ||
            !savedBorrowedInvalid.isTrusted || savedBorrowedInvalid.bubbles ||
            !savedBorrowedInvalid.cancelable)
          throw new Error('invalid event creation used the exposed child Event constructor');
        childWindow.Event = originalChildEvent;
        let wrongKind = false;
        try { borrowedValidity.call(document.createElement('div')); }
        catch (error) { wrongKind = error instanceof TypeError; }
        if (!wrongKind) throw new Error('borrowed getter accepted wrong interface kind');
        "#,
    );
}

fn validity_identity_and_custom_error_survive_adoption_but_not_clone<E: ScriptEngine>() {
    let mut runtime = child_form_document::<E>();
    run(
        &mut runtime,
        r#"
        childInput.value = 'initial';
        const liveValidity = childInput.validity;
        childInput.setCustomValidity('child custom message');
        if (!liveValidity.customError || liveValidity.valid ||
            childInput.validationMessage !== 'child custom message')
          throw new Error('custom validity did not become live');

        const adopted = document.adoptNode(childInput);
        document.body.appendChild(adopted);
        if (adopted !== childInput || adopted.validity !== liveValidity ||
            !liveValidity.customError || adopted.validationMessage !== 'child custom message')
          throw new Error('adoption replaced or lost live validity state');

        const copy = adopted.cloneNode(false);
        if (copy.value !== adopted.value || copy.validity.customError || !copy.validity.valid ||
            copy.validationMessage !== '')
          throw new Error('clone copied runtime validity metadata');
        "#,
    );
}

fn redispatching_child_invalid_events_clears_trust_for_all_public_paths<E: ScriptEngine>() {
    let mut runtime = child_form_document::<E>();
    run(
        &mut runtime,
        r#"
        childInput.addEventListener('invalid', event => {
          parent.savedByNode = event;
          parent.initialNodeTrust = event.isTrusted;
        });
        const windowInput = childWindow.document.createElement('input');
        windowInput.required = true;
        childWindow.document.body.appendChild(windowInput);
        windowInput.addEventListener('invalid', event => {
          parent.savedByWindow = event;
          parent.initialWindowTrust = event.isTrusted;
        });
        const prototypeInput = childWindow.document.createElement('input');
        prototypeInput.required = true;
        childWindow.document.body.appendChild(prototypeInput);
        prototypeInput.addEventListener('invalid', event => {
          parent.savedByPrototype = event;
          parent.initialPrototypeTrust = event.isTrusted;
        });
        parent.windowInput = windowInput;
        parent.prototypeInput = prototypeInput;
        "#,
    );
    run(
        &mut runtime,
        r#"
        if (childInput.checkValidity() || windowInput.checkValidity() || prototypeInput.checkValidity())
          throw new Error('required controls unexpectedly valid');
        if (initialNodeTrust !== true || initialWindowTrust !== true || initialPrototypeTrust !== true)
          throw new Error('UA invalid event was not initially trusted');

        // Use the parent Node method on a child-owned target, then the parent
        // Window method, then the child's EventTarget prototype on parent window.
        Node.prototype.dispatchEvent.call(childInput, savedByNode);
        if (savedByNode.isTrusted !== false) throw new Error('Node redispatch retained trust');
        window.dispatchEvent(savedByWindow);
        if (savedByWindow.isTrusted !== false) throw new Error('window redispatch retained trust');
        childWindow.EventTarget.prototype.dispatchEvent.call(window, savedByPrototype);
        if (savedByPrototype.isTrusted !== false)
          throw new Error('foreign EventTarget redispatch retained trust');
        "#,
    );
}

fn normalized_option_values_and_datetime_bounds_use_native_state<E: ScriptEngine>() {
    let mut runtime = runtime::<E>();
    run(
        &mut runtime,
        r#"
        const select = document.createElement('select'); select.required = true;
        const option = document.createElement('option'); option.textContent = ' \t\n ';
        select.appendChild(option); document.body.appendChild(select);
        if (select.value !== '' || option.value !== '' || option.text !== '' ||
            !select.validity.valueMissing || select.checkValidity())
          throw new Error('whitespace-only native placeholder is not empty');
        option.textContent = ' child \n oak \t ';
        const script = document.createElement('script'); script.type = 'application/json'; script.textContent = 'ignored';
        option.appendChild(script);
        if (select.value !== 'child oak' || option.value !== 'child oak' || option.text !== 'child oak')
          throw new Error('option and select disagree on HTML-aware text');
        select.selectedIndex = -1; select.value = 'child oak';
        if (select.selectedIndex !== 0 || !select.checkValidity())
          throw new Error('native select setter does not use normalized option value');
        option.value = ' child ';
        if (select.value !== ' child ' || option.value !== ' child ')
          throw new Error('explicit option value was normalized');
        option.text = '\u00a0'; option.removeAttribute('value');
        if (select.value !== '\u00a0' || !select.checkValidity())
          throw new Error('non-ASCII option whitespace was collapsed');

        const date = document.createElement('input'); date.type = 'datetime-local';
        date.min = '2024-01-01 12:00'; date.max = '2024-01-01 13:00';
        date.value = '2024-01-01T11:00';
        if (!date.validity.rangeUnderflow) throw new Error('space min bound ignored');
        date.value = '2024-01-01T14:00';
        if (!date.validity.rangeOverflow) throw new Error('space max bound ignored');
        date.removeAttribute('max'); date.min = '2024-01-01 12:00:30'; date.step = '60';
        date.value = '2024-01-01T12:01:30';
        if (date.validity.stepMismatch) throw new Error('space min step base ignored');
    "#,
    );
}

macro_rules! both_engines {
    ($($body:ident => ($boa:ident, $vano:ident)),* $(,)?) => {
        $(
            #[test]
            fn $boa() { $body::<script_engine_boa::BoaEngine>(); }

            #[cfg(target_pointer_width = "64")]
            #[test]
            fn $vano() { $body::<script_engine_nova::NovaEngine>(); }
        )*
    };
}

both_engines! {
    normalized_option_values_and_datetime_bounds_use_native_state =>
        (normalized_validation_on_boa, normalized_validation_on_vano),
    borrowed_input_validation_methods_use_child_native_state =>
        (borrowed_validation_on_boa, borrowed_validation_on_vano),
    validity_identity_and_custom_error_survive_adoption_but_not_clone =>
        (adopted_validity_on_boa, adopted_validity_on_vano),
    redispatching_child_invalid_events_clears_trust_for_all_public_paths =>
        (invalid_redispatch_on_boa, invalid_redispatch_on_vano),
}
