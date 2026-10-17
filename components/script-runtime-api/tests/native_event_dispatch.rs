// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The DOM event algorithm runs in `genet-events`; these pin what script sees.

use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime};

fn read<E: ScriptEngine>(source: &str) -> String {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.parse_document_interleaved("<body><div id=host><b id=light>l</b></div></body>", &NoScriptLoader);
    let value = rt.eval(source).expect("eval");
    rt.value_to_string(&value).expect("string")
}

fn phase_order_and_window<E: ScriptEngine>() {
    let out = read::<E>(
        r#"
        var log = [], host = document.getElementById('host'), light = document.getElementById('light');
        function at(name) { return function(e) { log.push(name + e.eventPhase); }; }
        window.addEventListener('t', at('wc'), true); window.addEventListener('t', at('wb'));
        document.addEventListener('t', at('d'));
        host.addEventListener('t', at('hc'), true); host.addEventListener('t', at('hb'));
        light.addEventListener('t', at('lb')); light.addEventListener('t', at('lc'), true);
        light.dispatchEvent(new Event('t', {bubbles: true}));
        log.join(',');
    "#,
    );
    assert_eq!(out, "wc1,hc1,lc2,lb2,hb3,d3,wb3");
}

fn flags_once_passive_and_handle_event<E: ScriptEngine>() {
    let out = read::<E>(
        r#"
        var log = [], light = document.getElementById('light');
        light.addEventListener('t', function() { log.push('once'); }, {once: true});
        light.addEventListener('t', function(e) { e.preventDefault(); log.push('passive:' + e.defaultPrevented); }, {passive: true});
        light.addEventListener('t', {handleEvent: function(e) { log.push('obj:' + (this !== light)); e.stopImmediatePropagation(); }});
        light.addEventListener('t', function() { log.push('never'); });
        var ev = new Event('t', {cancelable: true});
        var first = light.dispatchEvent(ev);
        var second = light.dispatchEvent(new Event('t', {cancelable: true}));
        log.join(',') + '|' + first + second;
    "#,
    );
    assert_eq!(out, "once,passive:false,obj:true,passive:false,obj:true|truetrue");
}

fn removal_during_dispatch_and_throwing_listener<E: ScriptEngine>() {
    let out = read::<E>(
        r#"
        var log = [], light = document.getElementById('light'), errors = 0;
        window.addEventListener('error', function() { errors++; });
        function later() { log.push('later'); }
        light.addEventListener('t', function() { log.push('first'); light.removeEventListener('t', later); });
        light.addEventListener('t', function() { throw new Error('boom'); });
        light.addEventListener('t', later);
        light.addEventListener('t', function() { log.push('last'); });
        light.dispatchEvent(new Event('t'));
        log.join(',') + '|' + errors;
    "#,
    );
    assert_eq!(out, "first,last|1");
}

fn shadow_retargeting_and_composed_path<E: ScriptEngine>() {
    let out = read::<E>(
        r#"
        var log = [], host = document.getElementById('host');
        var root = host.attachShadow({mode: 'closed'});
        root.innerHTML = '<p id=inner><slot></slot></p>';
        var inner = root.getElementById('inner'), light = document.getElementById('light');
        function name(n) { return n === window ? 'win' : n === document ? 'doc' : n === root ? 'root' : n.id || n.nodeName; }
        function full(tag) { return function(e) { log.push(tag + ':' + name(e.target) + ':' + e.composedPath().map(name).join('>')); }; }
        function short(tag) { return function(e) { log.push(tag + ':' + name(e.target)); }; }
        var current = full;
        host.addEventListener('t', function(e) { current('host')(e); });
        root.addEventListener('t', function(e) { current('root')(e); });
        inner.addEventListener('t', function(e) { current('inner')(e); });
        window.addEventListener('t', function(e) { current('win')(e); });
        inner.dispatchEvent(new Event('t', {bubbles: true, composed: true}));
        current = short;
        light.dispatchEvent(new Event('t', {bubbles: true}));
        var sealed = new Event('t', {bubbles: true});
        inner.dispatchEvent(sealed);
        log.push('sealed:' + (sealed.target === null) + ':' + sealed.composedPath().length);
        log.join(' ');
    "#,
    );
    let path = "BODY>HTML>doc>win";
    let expected = format!(
        "inner:inner:inner>root>host>{path} root:inner:inner>root>host>{path} \
         host:host:host>{path} win:host:host>{path} \
         inner:light root:light host:light win:light \
         inner:inner root:inner sealed:true:0"
    );
    assert_eq!(out, expected);
}

fn plain_event_targets_share_the_registry<E: ScriptEngine>() {
    let out = read::<E>(
        r#"
        var a = new EventTarget(), b = new EventTarget(), log = [];
        function f(e) { log.push(e.target === this ? 'self' : 'other', e.eventPhase, e.composedPath()[0] === this); }
        a.addEventListener('x', f); a.addEventListener('x', f);
        b.addEventListener('x', function() { log.push('b'); });
        a.dispatchEvent(new Event('x'));
        a.removeEventListener('x', f);
        a.dispatchEvent(new Event('x'));
        var late = new Event('x'); late.stopPropagation();
        b.dispatchEvent(late);
        log.join(',');
    "#,
    );
    assert_eq!(out, "self,2,true");
}

macro_rules! backend {
    ($module:ident, $engine:ty) => {
        mod $module {
            #[test]
            fn phase_order_and_window() {
                super::phase_order_and_window::<$engine>();
            }
            #[test]
            fn flags_once_passive_and_handle_event() {
                super::flags_once_passive_and_handle_event::<$engine>();
            }
            #[test]
            fn removal_during_dispatch_and_throwing_listener() {
                super::removal_during_dispatch_and_throwing_listener::<$engine>();
            }
            #[test]
            fn plain_event_targets_share_the_registry() {
                super::plain_event_targets_share_the_registry::<$engine>();
            }
            #[test]
            fn shadow_retargeting_and_composed_path() {
                super::shadow_retargeting_and_composed_path::<$engine>();
            }
        }
    };
}
backend!(boa, script_engine_boa::BoaEngine);
#[cfg(target_pointer_width = "64")]
backend!(nova, script_engine_nova::NovaEngine);
