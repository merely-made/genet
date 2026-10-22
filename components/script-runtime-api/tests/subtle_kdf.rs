// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `crypto.subtle`, phase 2 of the Web Crypto plan
//! (design_docs/2026-10-20_webcrypto_subtle_plan.md): HKDF and PBKDF2 base
//! keys, `deriveBits`, `deriveKey` into HMAC and AES keys, and AES raw
//! import/export as far as `deriveKey` needs it. Each body runs on both
//! backends. The registry's own refusals are unit-tested in subtle.rs.

use script_engine_api::ScriptEngine;
use script_runtime_api::Runtime;

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
     globalThis.unhex = function(h) { return new Uint8Array(h.match(/../g).map(function(x){ return parseInt(x, 16); })); }; \
     globalThis.bytes = function(text) { var b = new Uint8Array(text.length); \
     for (var i = 0; i < text.length; i++) b[i] = text.charCodeAt(i); return b; }; \
     globalThis.kdfKey = function(name, data, usages) { \
     return crypto.subtle.importKey('raw', data, {name: name}, false, usages || ['deriveBits', 'deriveKey']); };";

fn runtime<E: ScriptEngine>() -> Runtime<E> {
    let mut rt = Runtime::<E>::new().expect("runtime");
    rt.eval(HELPERS).expect("helpers");
    rt
}

/// RFC 5869 appendix A.1 and A.3 through `deriveBits`, RFC 6070 and a SHA-256
/// pair for PBKDF2.
fn derive_bits_vectors<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval(
        "globalThis.ikm = unhex('0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b');\
         globalThis.hkdfSalt = unhex('000102030405060708090a0b0c');\
         globalThis.hkdfInfo = unhex('f0f1f2f3f4f5f6f7f8f9');",
    )
    .expect("setup");
    assert_eq!(
        settle(&mut rt, "kdfKey('HKDF', ikm).then(function(k){ return crypto.subtle.deriveBits({name: 'HKDF', hash: 'SHA-256', salt: hkdfSalt, info: hkdfInfo}, k, 336); }).then(hex)"),
        "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865"
    );
    assert_eq!(
        settle(&mut rt, "kdfKey('HKDF', ikm).then(function(k){ return crypto.subtle.deriveBits({name: 'hkdf', hash: {name: 'sha-256'}, salt: new Uint8Array(0), info: new Uint8Array(0)}, k, 336); }).then(hex)"),
        "8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8"
    );
    assert_eq!(
        settle(&mut rt, "kdfKey('PBKDF2', bytes('password')).then(function(k){ return crypto.subtle.deriveBits({name: 'PBKDF2', hash: 'SHA-1', salt: bytes('salt'), iterations: 4096}, k, 160); }).then(hex)"),
        "4b007901b765489abead49d926f721d065a429c1"
    );
    assert_eq!(
        settle(&mut rt, "kdfKey('PBKDF2', bytes('password')).then(function(k){ return crypto.subtle.deriveBits({name: 'PBKDF2', hash: 'SHA-256', salt: bytes('salt'), iterations: 1}, k, 256); }).then(hex)"),
        "120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b"
    );
    // A zero length derives an empty buffer; views over an offset copy the right bytes.
    assert_eq!(
        settle(&mut rt, "kdfKey('PBKDF2', bytes('password')).then(function(k){ return crypto.subtle.deriveBits({name: 'PBKDF2', hash: 'SHA-1', salt: new DataView(bytes('xsaltx').buffer, 1, 4), iterations: 1}, k, 0); }).then(function(b){ return b.byteLength; })"),
        "0"
    );
    assert_eq!(
        settle(&mut rt, "kdfKey('PBKDF2', bytes('password')).then(function(k){ return crypto.subtle.deriveBits({name: 'PBKDF2', hash: 'SHA-1', salt: new DataView(bytes('xsaltx').buffer, 1, 4), iterations: 4096}, k, 160); }).then(hex)"),
        "4b007901b765489abead49d926f721d065a429c1"
    );
}

/// `deriveKey` yields the same bytes `deriveBits` does, for every target the
/// phase supports, and the key reports the shape the spec gives it.
fn derive_key_targets<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval(
        "globalThis.pbkdf2 = {name: 'PBKDF2', hash: 'SHA-256', salt: bytes('salt'), iterations: 3};\
         globalThis.bits = kdfKey('PBKDF2', bytes('password')).then(function(k){ return crypto.subtle.deriveBits(pbkdf2, k, 512); }).then(hex);",
    )
    .expect("reference bits");
    for (target, usages, take) in [
        ("{name: 'AES-GCM', length: 128}", "['encrypt', 'decrypt']", 16),
        ("{name: 'AES-CBC', length: 192}", "['decrypt']", 24),
        ("{name: 'AES-CTR', length: 256}", "['encrypt']", 32),
        ("{name: 'AES-KW', length: 256}", "['wrapKey', 'unwrapKey']", 32),
        ("{name: 'HMAC', hash: 'SHA-256', length: 256}", "['sign', 'verify']", 32),
        ("{name: 'HMAC', hash: 'SHA-1', length: 128}", "['sign']", 16),
    ] {
        let expr = format!(
            "Promise.all([bits, kdfKey('PBKDF2', bytes('password')).then(function(k){{ return crypto.subtle.deriveKey(pbkdf2, k, {target}, true, {usages}); }})\
             .then(function(key){{ return crypto.subtle.exportKey('raw', key).then(hex); }})]).then(function(r){{ return r[0].slice(0, {}) === r[1]; }})",
            take * 2
        );
        assert_eq!(settle(&mut rt, &expr), "true", "{target}");
    }
    // An HMAC target without a length takes the hash's block size.
    assert_eq!(
        settle(&mut rt, "kdfKey('PBKDF2', bytes('password')).then(function(k){ return crypto.subtle.deriveKey(pbkdf2, k, {name: 'HMAC', hash: 'SHA-512'}, true, ['sign']); }).then(function(k){ return [k.algorithm.name, k.algorithm.hash.name, k.algorithm.length, k.type, k.extractable, k.usages.join()].join(); })"),
        "HMAC,SHA-512,1024,secret,true,sign"
    );
    assert_eq!(
        settle(&mut rt, "kdfKey('HKDF', bytes('secret')).then(function(k){ return crypto.subtle.deriveKey({name: 'HKDF', hash: 'SHA-384', salt: bytes('s'), info: bytes('i')}, k, {name: 'AES-GCM', length: 256}, false, ['decrypt', 'encrypt', 'decrypt']); }).then(function(k){ return [k.algorithm.name, k.algorithm.length, k.type, k.extractable, k.usages.join()].join(); })"),
        "AES-GCM,256,secret,false,encrypt,decrypt"
    );
    // The derived key is usable: an HMAC tag from it matches one from the same bytes imported.
    assert_eq!(
        settle(&mut rt, "kdfKey('PBKDF2', bytes('password')).then(function(k){ return crypto.subtle.deriveKey(pbkdf2, k, {name: 'HMAC', hash: 'SHA-256', length: 256}, false, ['sign']); }).then(function(k){ return crypto.subtle.sign('HMAC', k, bytes('m')); }).then(hex).then(function(tag){ return bits.then(function(b){ return crypto.subtle.importKey('raw', unhex(b.slice(0, 64)), {name: 'HMAC', hash: 'SHA-256'}, false, ['sign']); }).then(function(k){ return crypto.subtle.sign('HMAC', k, bytes('m')); }).then(hex).then(function(again){ return tag === again; }); })"),
        "true"
    );
}

/// The error names the spec assigns, in the order it assigns them.
fn kdf_rules_and_errors<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval(
        "globalThis.hk = {name: 'HKDF', hash: 'SHA-256', salt: bytes('s'), info: bytes('i')};\
         globalThis.pb = {name: 'PBKDF2', hash: 'SHA-256', salt: bytes('s'), iterations: 2};\
         globalThis.noBits = kdfKey('PBKDF2', bytes('p'), ['deriveKey']);\
         globalThis.noKey = kdfKey('PBKDF2', bytes('p'), ['deriveBits']);\
         globalThis.hkdfKey = kdfKey('HKDF', bytes('p'));\
         globalThis.aesKey = crypto.subtle.importKey('raw', new Uint8Array(16), {name: 'AES-GCM'}, true, ['encrypt']);",
    )
    .expect("setup");
    for (expr, name) in [
        // Import rules for a KDF base key.
        ("crypto.subtle.importKey('raw', bytes('p'), {name: 'HKDF'}, true, ['deriveBits'])", "SyntaxError"),
        ("crypto.subtle.importKey('raw', bytes('p'), {name: 'PBKDF2'}, false, ['sign'])", "SyntaxError"),
        ("crypto.subtle.importKey('raw', bytes('p'), {name: 'PBKDF2'}, false, [])", "SyntaxError"),
        ("crypto.subtle.importKey('jwk', {kty: 'oct', k: 'cA'}, {name: 'HKDF'}, false, ['deriveBits'])", "NotSupportedError"),
        ("crypto.subtle.importKey('spki', bytes('p'), {name: 'PBKDF2'}, false, ['deriveBits'])", "NotSupportedError"),
        // deriveBits.
        ("noBits.then(function(k){ return crypto.subtle.deriveBits(pb, k, 256); })", "InvalidAccessError"),
        ("noKey.then(function(k){ return crypto.subtle.deriveKey(pb, k, {name: 'AES-GCM', length: 128}, true, ['encrypt']); })", "InvalidAccessError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveBits(pb, k, 256); })", "InvalidAccessError"),
        ("aesKey.then(function(k){ return crypto.subtle.deriveBits(pb, k, 256); })", "InvalidAccessError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveBits(hk, k, 44); })", "OperationError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveBits(hk, k); })", "OperationError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveBits(hk, k, 255 * 32 * 8 + 8); })", "OperationError"),
        ("kdfKey('PBKDF2', bytes('p')).then(function(k){ return crypto.subtle.deriveBits({name: 'PBKDF2', hash: 'SHA-1', salt: bytes('s'), iterations: 0}, k, 128); })", "OperationError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveBits({name: 'HKDF', hash: 'SHA-256', info: bytes('i')}, k, 8); })", "TypeError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveBits({name: 'HKDF', hash: 'SHA-256', salt: bytes('s')}, k, 8); })", "TypeError"),
        ("kdfKey('PBKDF2', bytes('p')).then(function(k){ return crypto.subtle.deriveBits({name: 'PBKDF2', hash: 'SHA-1', salt: bytes('s')}, k, 8); })", "TypeError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveBits({name: 'HKDF', hash: 'SHA-256', salt: 'text', info: bytes('i')}, k, 8); })", "TypeError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveBits({name: 'HKDF', hash: 'SHA-2-56', salt: bytes('s'), info: bytes('i')}, k, 8); })", "NotSupportedError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveBits({name: 'HKDF', hash: 'PBKDF2', salt: bytes('s'), info: bytes('i')}, k, 8); })", "NotSupportedError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveBits({name: 'HMAC', hash: 'SHA-256'}, k, 8); })", "NotSupportedError"),
        // deriveKey.
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveKey(hk, k, {name: 'AES-GCM', length: 100}, true, ['encrypt']); })", "OperationError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveKey(hk, k, {name: 'AES-GCM'}, true, ['encrypt']); })", "TypeError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveKey(hk, k, {name: 'HMAC', hash: 'SHA-1', length: 0}, true, ['sign']); })", "TypeError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveKey(hk, k, {name: 'AES-GCM', length: 128}, true, ['sign']); })", "SyntaxError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveKey(hk, k, {name: 'AES-KW', length: 128}, true, ['encrypt']); })", "SyntaxError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveKey(hk, k, {name: 'AES-GCM', length: 128}, true, []); })", "SyntaxError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveKey(hk, k, {name: 'HKDF'}, false, ['deriveBits']); })", "OperationError"),
        ("hkdfKey.then(function(k){ return crypto.subtle.deriveKey(hk, k, {name: 'RSA-OAEP'}, true, ['encrypt']); })", "NotSupportedError"),
        // The AES import the derive path relies on.
        ("crypto.subtle.importKey('raw', new Uint8Array(20), {name: 'AES-CBC'}, true, ['encrypt'])", "DataError"),
        ("crypto.subtle.importKey('raw', new Uint8Array(16), {name: 'AES-KW'}, true, ['encrypt'])", "SyntaxError"),
        ("crypto.subtle.importKey('raw', new Uint8Array(16), {name: 'AES-CTR'}, true, [])", "SyntaxError"),
        ("crypto.subtle.importKey('jwk', {kty: 'oct', k: 'AAAAAAAAAAAAAAAAAAAAAA'}, {name: 'AES-CTR'}, true, ['encrypt'])", "NotSupportedError"),
        ("crypto.subtle.generateKey({name: 'AES-GCM', length: 128}, true, ['encrypt'])", "NotSupportedError"),
        ("aesKey.then(function(k){ return crypto.subtle.encrypt({name: 'AES-GCM', iv: new Uint8Array(12)}, k, new Uint8Array(4)); })", "NotSupportedError"),
        ("aesKey.then(function(k){ return crypto.subtle.sign('HMAC', k, new Uint8Array(1)); })", "InvalidAccessError"),
    ] {
        assert_eq!(settle(&mut rt, expr), format!("E:{name}"), "{expr}");
    }
    // A KDF key never exports, whatever the format; an AES key exports raw.
    assert_eq!(
        settle(&mut rt, "hkdfKey.then(function(k){ return crypto.subtle.exportKey('raw', k); })"),
        "E:NotSupportedError"
    );
    assert_eq!(
        settle(&mut rt, "aesKey.then(function(k){ return crypto.subtle.exportKey('raw-secret', k); }).then(hex)"),
        "00000000000000000000000000000000"
    );
    assert_eq!(
        settle(&mut rt, "aesKey.then(function(k){ return [k.algorithm.name, k.algorithm.length, k.usages.join()].join(); })"),
        "AES-GCM,128,encrypt"
    );
    assert_eq!(
        settle(&mut rt, "kdfKey('HKDF', new Uint8Array(0)).then(function(k){ return crypto.subtle.deriveBits(hk, k, 16).then(function(b){ return b.byteLength + ':' + k.extractable + ':' + k.algorithm.name + ':' + Object.keys(k.algorithm).join(); }); })"),
        "2:false:HKDF:name",
        "an empty secret is a valid base key"
    );
    // Both usages are filtered into enum order, without repeats.
    assert_eq!(
        settle(&mut rt, "kdfKey('PBKDF2', bytes('p'), ['deriveBits', 'deriveKey', 'deriveBits']).then(function(k){ return k.usages.join(); })"),
        "deriveKey,deriveBits"
    );
}

/// A derived key is a real `CryptoKey` over native bytes: a non-extractable
/// one cannot be read back, and the base key's usages still gate it.
fn derived_keys_stay_native<E: ScriptEngine>() {
    let mut rt = runtime::<E>();
    rt.eval(
        "kdfKey('HKDF', bytes('secret')).then(function(base){          return crypto.subtle.deriveKey({name: 'HKDF', hash: 'SHA-256', salt: bytes('s'), info: bytes('i')}, base, {name: 'AES-GCM', length: 128}, false, ['encrypt']); })         .then(function(k){ globalThis.derived = k; });",
    )
    .expect("derive");
    rt.run_event_loop(10).expect("settle");
    assert_eq!(read(&mut rt, "String(derived instanceof CryptoKey) + String(derived.extractable)"), "truefalse");
    assert_eq!(settle(&mut rt, "crypto.subtle.exportKey('raw', derived)"), "E:InvalidAccessError");
    assert_eq!(settle(&mut rt, "crypto.subtle.exportKey('jwk', derived)"), "E:InvalidAccessError");
    assert_eq!(read(&mut rt, "Object.getOwnPropertyNames(derived).length + ':' + Object.getOwnPropertySymbols(derived).length"), "0:0");
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
    derive_bits_vectors => (derive_bits_vectors_on_boa, derive_bits_vectors_on_nova),
    derive_key_targets => (derive_key_targets_on_boa, derive_key_targets_on_nova),
    kdf_rules_and_errors => (kdf_rules_on_boa, kdf_rules_on_nova),
    derived_keys_stay_native => (derived_keys_native_on_boa, derived_keys_native_on_nova),
}
