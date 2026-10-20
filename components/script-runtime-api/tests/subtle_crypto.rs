// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `crypto.subtle`, phase 1 of the Web Crypto plan
//! (design_docs/2026-10-20_webcrypto_subtle_plan.md): the interface shape,
//! digest, HMAC, raw and JWK import/export, the error names, and the native
//! registry's authority over extractability. Each body runs on both backends.

use std::time::{Duration, Instant};

use script_engine_api::ScriptEngine;
use script_runtime_api::{NoScriptLoader, Runtime, ScriptResourceLoader, WorkerPolicy};

fn read<E: ScriptEngine>(rt: &mut Runtime<E>, expr: &str) -> String {
    let v = rt.eval(expr).expect("eval");
    rt.value_to_string(&v).expect("stringify")
}

/// Evaluate a promise-valued `expr`, drain the loop, and read its outcome:
/// the fulfilled value as a string, or `E:<name>` for a rejection.
fn settle<E: ScriptEngine>(rt: &mut Runtime<E>, expr: &str) -> String {
    rt.eval(&format!(
        "globalThis.__out = 'pending'; Promise.resolve().then(function(){{ return {expr}; }})\
         .then(function(v){{ __out = String(v); }}, function(e){{ __out = 'E:' + e.name; }});"
    ))
    .expect("eval");
    rt.run_event_loop(10).expect("drain");
    read(rt, "__out")
}

const HELPERS: &str = "globalThis.hex = function(buf) { var b = new Uint8Array(buf), s = ''; \
     for (var i = 0; i < b.length; i++) s += (b[i] + 0x100).toString(16).slice(1); return s; }; \
     globalThis.bytes = function(text) { var b = new Uint8Array(text.length); \
     for (var i = 0; i < text.length; i++) b[i] = text.charCodeAt(i); return b; };";

fn subtle_shape<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().expect("runtime");
    assert_eq!(read(&mut rt, "typeof crypto.subtle"), "object");
    assert_eq!(read(&mut rt, "String(crypto.subtle === crypto.subtle)"), "true");
    assert_eq!(read(&mut rt, "String(crypto.subtle instanceof SubtleCrypto)"), "true");
    assert_eq!(read(&mut rt, "Object.prototype.toString.call(crypto.subtle)"), "[object SubtleCrypto]");
    for ctor in ["SubtleCrypto", "CryptoKey"] {
        assert_eq!(
            read(&mut rt, &format!("(function(){{ try {{ new {ctor}(); }} catch (e) {{ return e.name; }} return 'none'; }})()")),
            "TypeError",
            "{ctor} has no constructor"
        );
    }
    for (name, length) in [("digest", 2), ("sign", 3), ("verify", 4), ("importKey", 5), ("unwrapKey", 7)] {
        assert_eq!(read(&mut rt, &format!("String(SubtleCrypto.prototype.{name}.length)")), length.to_string());
    }
    // A method on the wrong receiver rejects rather than throws.
    assert_eq!(settle(&mut rt, "SubtleCrypto.prototype.digest.call({}, 'SHA-1', new Uint8Array(0))"), "E:TypeError");
}

fn digest_vectors<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.eval(HELPERS).expect("helpers");
    // FIPS 180-4 examples for the one-block message "abc".
    for (alg, expected) in [
        ("SHA-1", "a9993e364706816aba3e25717850c26c9cd0d89d"),
        ("sha-256", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
        ("SHA-384", "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7"),
        ("SHA-512", "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"),
    ] {
        assert_eq!(
            settle(&mut rt, &format!("crypto.subtle.digest({{name: '{alg}'}}, bytes('abc')).then(hex)")),
            expected,
            "{alg}"
        );
    }
    // Views over an offset, DataViews and bare buffers all copy the right bytes.
    assert_eq!(
        settle(&mut rt, "crypto.subtle.digest('SHA-1', new DataView(bytes('xabcx').buffer, 1, 3)).then(hex)"),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
    assert_eq!(settle(&mut rt, "crypto.subtle.digest('MD5', new Uint8Array(1))"), "E:NotSupportedError");
    assert_eq!(settle(&mut rt, "crypto.subtle.digest({}, new Uint8Array(1))"), "E:TypeError");
    assert_eq!(settle(&mut rt, "crypto.subtle.digest('SHA-1', 'abc')"), "E:TypeError");
    // Only ASCII case-folds: the Kelvin sign is not a K.
    assert_eq!(settle(&mut rt, "crypto.subtle.digest('S\\u212AA-1', new Uint8Array(1))"), "E:NotSupportedError");
}

fn hmac_vectors_and_rules<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.eval(HELPERS).expect("helpers");
    // RFC 4231 test case 2.
    rt.eval(
        "globalThis.keyP = crypto.subtle.importKey('raw', bytes('Jefe'), {name: 'HMAC', hash: 'SHA-256'}, true, ['verify', 'sign', 'sign']);\
         globalThis.msg = bytes('what do ya want for nothing?');",
    )
    .expect("import");
    let tag = "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843";
    assert_eq!(settle(&mut rt, "keyP.then(function(k){ return crypto.subtle.sign('HMAC', k, msg); }).then(hex)"), tag);
    assert_eq!(
        settle(&mut rt, "keyP.then(function(k){ return [k.type, k.extractable, k.algorithm.name, k.algorithm.hash.name, k.algorithm.length, k.usages.join('+'), k.algorithm === k.algorithm, k.usages === k.usages].join(); })"),
        "secret,true,HMAC,SHA-256,32,sign+verify,true,true"
    );
    rt.eval(&format!("globalThis.sig = new Uint8Array({tag:?}.match(/../g).map(function(h){{ return parseInt(h, 16); }}));"))
        .expect("signature");
    assert_eq!(settle(&mut rt, "keyP.then(function(k){ return crypto.subtle.verify('HMAC', k, sig, msg); })"), "true");
    assert_eq!(
        settle(&mut rt, "keyP.then(function(k){ var s = sig.slice(); s[0] ^= 1; return crypto.subtle.verify('HMAC', k, s, msg); })"),
        "false"
    );
    assert_eq!(
        settle(&mut rt, "keyP.then(function(k){ return crypto.subtle.verify('HMAC', k, sig.slice(0, 16), msg); })"),
        "false"
    );
    // Raw and JWK export round trips; JWK carries alg, key_ops and ext.
    assert_eq!(settle(&mut rt, "keyP.then(function(k){ return crypto.subtle.exportKey('raw', k); }).then(hex)"), "4a656665");
    assert_eq!(
        settle(&mut rt, "keyP.then(function(k){ return crypto.subtle.exportKey('jwk', k); }).then(function(j){ return [j.kty, j.k, j.alg, j.key_ops.join('+'), j.ext].join(); })"),
        "oct,SmVmZQ,HS256,sign+verify,true"
    );
    assert_eq!(
        settle(&mut rt, "crypto.subtle.importKey('jwk', {kty: 'oct', k: 'SmVmZQ', alg: 'HS256'}, {name: 'HMAC', hash: 'SHA-256'}, false, ['sign']).then(function(k){ return crypto.subtle.sign('HMAC', k, msg); }).then(hex)"),
        tag
    );
    // The error names the spec assigns.
    for (expr, name) in [
        ("crypto.subtle.importKey('jwk', {kty: 'oct', k: 'SmVmZQ', alg: 'HS512'}, {name: 'HMAC', hash: 'SHA-256'}, false, ['sign'])", "DataError"),
        ("crypto.subtle.importKey('jwk', {kty: 'oct', k: 'Sm*mZQ'}, {name: 'HMAC', hash: 'SHA-256'}, false, ['sign'])", "DataError"),
        ("crypto.subtle.importKey('raw', new Uint8Array(0), {name: 'HMAC', hash: 'SHA-1'}, false, ['sign'])", "DataError"),
        ("crypto.subtle.importKey('raw', bytes('Jefe'), {name: 'HMAC', hash: 'SHA-1'}, false, [])", "SyntaxError"),
        ("crypto.subtle.importKey('raw', bytes('Jefe'), {name: 'HMAC', hash: 'SHA-1'}, false, ['encrypt'])", "SyntaxError"),
        ("crypto.subtle.importKey('raw', bytes('Jefe'), {name: 'HMAC'}, false, ['sign'])", "TypeError"),
        ("crypto.subtle.importKey('spki', bytes('Jefe'), {name: 'HMAC', hash: 'SHA-1'}, false, ['sign'])", "NotSupportedError"),
        ("crypto.subtle.importKey('jwk', bytes('Jefe'), {name: 'HMAC', hash: 'SHA-1'}, false, ['sign'])", "TypeError"),
        ("crypto.subtle.importKey('raw', bytes('Jefe'), {name: 'AES-GCM'}, false, ['encrypt'])", "NotSupportedError"),
        ("crypto.subtle.generateKey({name: 'HMAC', hash: 'SHA-256'}, false, [])", "SyntaxError"),
        ("crypto.subtle.generateKey({name: 'HMAC', hash: 'SHA-256', length: 0}, false, ['sign'])", "OperationError"),
        ("crypto.subtle.generateKey({name: 'HMAC', hash: 'SHA-256'}, false, ['sign', 'bogus'])", "TypeError"),
        ("crypto.subtle.encrypt({name: 'AES-GCM'}, null, new Uint8Array(0))", "TypeError"),
        ("keyP.then(function(k){ return crypto.subtle.encrypt({name: 'AES-GCM'}, k, new Uint8Array(0)); })", "NotSupportedError"),
        ("crypto.subtle.importKey('raw', bytes('Jefe'), {name: 'HMAC', hash: 'SHA-1'}, false, ['verify']).then(function(k){ return crypto.subtle.sign('HMAC', k, msg); })", "InvalidAccessError"),
        ("crypto.subtle.importKey('raw', bytes('Jefe'), {name: 'HMAC', hash: 'SHA-1'}, false, ['sign']).then(function(k){ return crypto.subtle.exportKey('raw', k); })", "InvalidAccessError"),
    ] {
        assert_eq!(settle(&mut rt, expr), format!("E:{name}"), "{expr}");
    }
    // Generated keys take the hash's block size by default and sign.
    assert_eq!(
        settle(&mut rt, "crypto.subtle.generateKey({name: 'HMAC', hash: 'SHA-512'}, true, ['sign']).then(function(k){ return crypto.subtle.exportKey('raw', k).then(function(raw){ return k.algorithm.length + ':' + raw.byteLength; }); })"),
        "1024:128"
    );
}

/// Page script can reach neither the native sinks nor the brand records:
/// both are captured by the bootstrap and removed from the global. (The
/// registry's own refusal to export or misuse a key is tested in subtle.rs.)
fn subtle_internals_are_private<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().expect("runtime");
    assert_eq!(
        read(&mut rt, "Object.getOwnPropertyNames(globalThis).filter(function(n){ return /^__subtle|^__agentTimers$/.test(n); }).join()"),
        ""
    );
    assert_eq!(read(&mut rt, "typeof __agentTimers"), "undefined");
    assert_eq!(
        settle(&mut rt, "crypto.subtle.generateKey({name: 'HMAC', hash: 'SHA-256'}, false, ['sign']).then(function(k){ return Object.getOwnPropertyNames(k).length + ':' + Object.getOwnPropertySymbols(k).length; })"),
        "0:0",
        "a CryptoKey carries no own state"
    );
}

fn get_random_values_accepts_subclasses<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().expect("runtime");
    assert_eq!(
        read(&mut rt, "(function(){ class Buf extends Uint8Array {} var b = new Buf(16); return String(crypto.getRandomValues(b) === b); })()"),
        "true"
    );
    assert_eq!(
        read(&mut rt, "(function(){ try { crypto.getRandomValues(new DataView(new ArrayBuffer(4))); } catch (e) { return e.name; } return 'none'; })()"),
        "TypeMismatchError"
    );
    assert_eq!(
        read(&mut rt, "(function(){ try { crypto.getRandomValues({}); } catch (e) { return e.name; } return 'none'; })()"),
        "TypeError"
    );
}

const WEB_CRYPTO_MEMBERS: &str = "[typeof crypto.subtle, 'subtle' in crypto, typeof SubtleCrypto, 'CryptoKey' in self].join()";
const PRESENT: &str = "object,true,function,true";
const ABSENT: &str = "undefined,false,undefined,false";

/// Web Crypto's `[SecureContext]` members exist only in a secure context: the
/// document URL decides, the host may override, and `getRandomValues` stays.
fn secure_context_gates_web_crypto<E: ScriptEngine>() {
    let mut rt = Runtime::<E>::new().expect("runtime");
    assert_eq!(read(&mut rt, WEB_CRYPTO_MEMBERS), PRESENT, "no URL yet: secure by default");
    rt.set_base_url("http://example.test/page.html").unwrap();
    assert_eq!(read(&mut rt, WEB_CRYPTO_MEMBERS), ABSENT, "an http document is not secure");
    assert_eq!(read(&mut rt, "typeof crypto.getRandomValues"), "function");
    rt.set_secure_context(true).unwrap();
    assert_eq!(read(&mut rt, WEB_CRYPTO_MEMBERS), PRESENT, "the host's override restores them");
    assert_eq!(read(&mut rt, "String(crypto.subtle === crypto.subtle)"), "true");
    rt.set_base_url("http://localhost:8000/").unwrap();
    assert_eq!(read(&mut rt, WEB_CRYPTO_MEMBERS), PRESENT, "localhost is potentially trustworthy");
    rt.set_base_url("http://example.test/").unwrap();
    rt.set_base_url("https://example.test/").unwrap();
    assert_eq!(read(&mut rt, WEB_CRYPTO_MEMBERS), PRESENT);
    assert_eq!(read(&mut rt, "typeof __subtleExposure"), "undefined", "the host hook is private");
}

/// A nested document needs a secure parent; `about:blank` inherits its creator.
fn frames_inherit_secure_context<E: ScriptEngine>() {
    for (page, expected) in [("https://parent.test/", PRESENT), ("http://parent.test/", ABSENT)] {
        let mut rt = Runtime::<E>::new().expect("runtime");
        rt.set_base_url(page).unwrap();
        rt.parse_document_interleaved("<body><iframe id=frame></iframe></body>", &NoScriptLoader);
        rt.run_event_loop(20).unwrap();
        let child = WEB_CRYPTO_MEMBERS
            .replace("crypto.", "child.crypto.")
            .replace("in crypto", "in child.crypto")
            .replace("typeof SubtleCrypto", "typeof child.SubtleCrypto")
            .replace("in self", "in child");
        rt.eval("var child = document.getElementById('frame').contentWindow;").unwrap();
        assert_eq!(read(&mut rt, &child), expected, "about:blank child of {page}");
    }
}

struct Scripts(&'static str);
impl ScriptResourceLoader for Scripts {
    fn load(&self, url: &str) -> Option<String> {
        url.ends_with("probe.js").then(|| self.0.to_owned())
    }
}

/// A worker inherits its owner's context, not its script URL's.
fn workers_inherit_secure_context<E: ScriptEngine>() {
    for (secure, expected) in [(true, PRESENT), (false, ABSENT)] {
        let mut rt = Runtime::<E>::new().expect("runtime");
        rt.set_worker_policy(WorkerPolicy::TrustedCooperative {
            resource_timeout: Duration::from_secs(30),
            shutdown_grace: Duration::from_millis(500),
        });
        rt.set_base_url("http://example.test/").unwrap();
        rt.set_secure_context(secure).unwrap();
        rt.set_script_resource_loader(Box::new(Scripts(
            "postMessage([typeof crypto.subtle, 'subtle' in crypto, typeof SubtleCrypto, 'CryptoKey' in self].join());",
        )));
        rt.eval("var got = null; var w = new Worker('probe.js'); w.onmessage = function(e) { got = e.data; };")
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        let mut now_ms = 0.0f64;
        while read(&mut rt, "String(got !== null)") != "true" && Instant::now() < deadline {
            rt.run_microtasks();
            rt.run_timers(64, now_ms).unwrap();
            rt.pump_workers();
            now_ms += 1.0;
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(read(&mut rt, "String(got)"), expected, "worker of a secure={secure} owner");
    }
}

macro_rules! both_engines {
    ($($body:ident => ($boa:ident, $nova:ident)),* $(,)?) => {
        $(
            #[test]
            fn $boa() { $body::<script_engine_boa::BoaEngine>(); }

            #[cfg(not(target_arch = "wasm32"))]
            #[test]
            fn $nova() { $body::<script_engine_nova::NovaEngine>(); }
        )*
    };
}

both_engines! {
    subtle_shape => (subtle_shape_on_boa, subtle_shape_on_nova),
    digest_vectors => (digest_vectors_on_boa, digest_vectors_on_nova),
    hmac_vectors_and_rules => (hmac_on_boa, hmac_on_nova),
    subtle_internals_are_private => (internals_private_on_boa, internals_private_on_nova),
    get_random_values_accepts_subclasses => (get_random_values_subclass_on_boa, get_random_values_subclass_on_nova),
    secure_context_gates_web_crypto => (secure_context_on_boa, secure_context_on_nova),
    frames_inherit_secure_context => (frame_secure_context_on_boa, frame_secure_context_on_nova),
    workers_inherit_secure_context => (worker_secure_context_on_boa, worker_secure_context_on_nova),
}
