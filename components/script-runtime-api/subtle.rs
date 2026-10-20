/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Web Crypto's `crypto.subtle`: native key storage plus the operations,
//! each delegated to a maintained RustCrypto crate.
//!
//! The split follows the Encoding surface. The JS bootstrap owns the WebIDL
//! conversions, algorithm normalisation, the error names and the promises. This
//! file owns everything that touches key material: a per-realm registry
//! of keys behind opaque ids, digests, and MACs. The registry is the authority
//! on a key's extractability and usages, so a forged JS-side brand can't export
//! a non-extractable key or sign with a verify-only one. Key bytes are zeroised
//! when released. No algorithm is implemented here; every primitive comes from
//! `sha1`, `sha2` and `hmac`.
//!
//! Bytes cross as Latin-1 binary strings (one char per byte), the convention
//! `__crypto_random` and `TextDecoder` already use.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use hmac::{Hmac, KeyInit, Mac};
use script_engine_api::{CallCx, NativeFn, ScriptEngine};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};
use zeroize::Zeroizing;

use crate::HostState;

static NEXT_KEY_ID: AtomicU64 = AtomicU64::new(1);

/// Process-wide and never recycled, so a stale wrapper (a snapshot clone's, or
/// a late finalizer) can never name a later key.
fn allocate_key_id() -> Option<u64> {
    NEXT_KEY_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .ok()
}

/// The SHA family Web Crypto registers for `digest` and as HMAC's hash.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HashAlg {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl HashAlg {
    /// The registered (already normalised) name.
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "SHA-1" => Some(Self::Sha1),
            "SHA-256" => Some(Self::Sha256),
            "SHA-384" => Some(Self::Sha384),
            "SHA-512" => Some(Self::Sha512),
            _ => None,
        }
    }

    fn digest(self, data: &[u8]) -> Vec<u8> {
        match self {
            Self::Sha1 => Sha1::digest(data).to_vec(),
            Self::Sha256 => Sha256::digest(data).to_vec(),
            Self::Sha384 => Sha384::digest(data).to_vec(),
            Self::Sha512 => Sha512::digest(data).to_vec(),
        }
    }
}

/// Run `$body` with `$mac` keyed by `$key` and fed `$data`, for the hash named.
/// HMAC accepts a key of any length, so `new_from_slice` cannot fail for it.
macro_rules! with_hmac {
    ($hash:expr, $key:expr, $data:expr, |$mac:ident| $body:expr) => {
        match $hash {
            HashAlg::Sha1 => with_hmac!(@one Sha1, $key, $data, $mac, $body),
            HashAlg::Sha256 => with_hmac!(@one Sha256, $key, $data, $mac, $body),
            HashAlg::Sha384 => with_hmac!(@one Sha384, $key, $data, $mac, $body),
            HashAlg::Sha512 => with_hmac!(@one Sha512, $key, $data, $mac, $body),
        }
    };
    (@one $digest:ty, $key:expr, $data:expr, $mac:ident, $body:expr) => {{
        let mut $mac = <Hmac<$digest> as KeyInit>::new_from_slice($key).expect("HMAC takes any key length");
        $mac.update($data);
        $body
    }};
}

fn hmac_sign(hash: HashAlg, key: &[u8], data: &[u8]) -> Vec<u8> {
    with_hmac!(hash, key, data, |mac| mac.finalize().into_bytes().to_vec())
}

/// Constant-time comparison, through the crate's own `verify_slice`.
fn hmac_verify(hash: HashAlg, key: &[u8], data: &[u8], signature: &[u8]) -> bool {
    with_hmac!(hash, key, data, |mac| mac.verify_slice(signature).is_ok())
}

/// The key usages, as Web Crypto's `KeyUsage` enum spells them.
const USAGES: [&str; 12] = [
    "encrypt",
    "decrypt",
    "sign",
    "verify",
    "deriveKey",
    "deriveBits",
    "wrapKey",
    "unwrapKey",
    "encapsulateKey",
    "encapsulateBits",
    "decapsulateKey",
    "decapsulateBits",
];

fn usage_bit(name: &str) -> Option<u16> {
    USAGES.iter().position(|u| *u == name).map(|i| 1 << i)
}

/// A comma-separated usage list to a bit set; `None` if any entry is unknown.
fn parse_usages(list: &str) -> Option<u16> {
    list.split(',')
        .filter(|u| !u.is_empty())
        .try_fold(0u16, |bits, u| usage_bit(u).map(|b| bits | b))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyAlgorithm {
    Hmac(HashAlg),
}

struct KeyEntry {
    algorithm: KeyAlgorithm,
    extractable: bool,
    usages: u16,
    material: Zeroizing<Vec<u8>>,
}

impl KeyEntry {
    fn allows(&self, usage: &str) -> bool {
        usage_bit(usage).is_some_and(|bit| self.usages & bit != 0)
    }
}

/// Native key material keyed by wrapper id. Entries hold no script references;
/// finalization or runtime teardown removes them, zeroising the bytes.
#[derive(Default)]
pub(crate) struct KeyRegistry {
    entries: HashMap<u64, KeyEntry>,
}

impl KeyRegistry {
    fn insert(&mut self, entry: KeyEntry) -> Option<u64> {
        let id = allocate_key_id()?;
        self.entries.insert(id, entry);
        Some(id)
    }

    fn get(&self, id: u64) -> Option<&KeyEntry> {
        self.entries.get(&id)
    }

    /// The key's bytes, only for a known, extractable key.
    fn export(&self, id: u64) -> Option<Zeroizing<Vec<u8>>> {
        let entry = self.get(id)?;
        entry
            .extractable
            .then(|| Zeroizing::new(entry.material.to_vec()))
    }

    /// A MAC over `data`, only for a known key holding the `sign` usage.
    fn sign(&self, id: u64, data: &[u8]) -> Option<Vec<u8>> {
        let entry = self.get(id).filter(|entry| entry.allows("sign"))?;
        match entry.algorithm {
            KeyAlgorithm::Hmac(hash) => Some(hmac_sign(hash, &entry.material, data)),
        }
    }

    /// Whether `signature` is `data`'s MAC, only for a known key holding the
    /// `verify` usage.
    fn verify(&self, id: u64, data: &[u8], signature: &[u8]) -> Option<bool> {
        let entry = self.get(id).filter(|entry| entry.allows("verify"))?;
        match entry.algorithm {
            KeyAlgorithm::Hmac(hash) => Some(hmac_verify(hash, &entry.material, data, signature)),
        }
    }

    fn release(&mut self, id: u64) {
        self.entries.remove(&id);
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}

fn arg_string<E: ScriptEngine>(cx: &mut E::CallCx<'_>, index: usize) -> Result<String, E::Error> {
    let arg = cx.arg(index);
    cx.value_to_string(&arg)
}

fn arg_bytes<E: ScriptEngine>(cx: &mut E::CallCx<'_>, index: usize) -> Result<Zeroizing<Vec<u8>>, E::Error> {
    let binary = Zeroizing::new(arg_string::<E>(cx, index)?);
    Ok(Zeroizing::new(binary.chars().map(|ch| ch as u32 as u8).collect()))
}

fn make_binary<E: ScriptEngine>(cx: &mut E::CallCx<'_>, bytes: &[u8]) -> Result<E::Value, E::Error> {
    let s: String = bytes.iter().map(|&b| b as char).collect();
    cx.make_string(&s)
}

fn with_keys<E: ScriptEngine, R>(
    cx: &mut E::CallCx<'_>,
    f: impl FnOnce(&mut KeyRegistry) -> Option<R>,
) -> Option<R> {
    let data = cx.host_data()?;
    let host = data.downcast_ref::<RefCell<HostState>>()?;
    let mut host = host.borrow_mut();
    f(&mut host.crypto_keys)
}

/// `__subtle_digest(hash, data)` → the digest as a binary string, or `null`
/// for an unregistered hash name.
struct SubtleDigest;
impl<E: ScriptEngine> NativeFn<E> for SubtleDigest {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let hash = HashAlg::from_name(&arg_string::<E>(cx, 0)?);
        let data = arg_bytes::<E>(cx, 1)?;
        match hash {
            Some(hash) => make_binary::<E>(cx, &hash.digest(&data)),
            None => Ok(cx.make_null()),
        }
    }
}

/// `__subtle_hmac_store(hash, extractable, usages, bytes)` → the new key's id,
/// or `null`. Imports supply `bytes`; validation already ran in the bootstrap.
struct SubtleHmacStore;
impl<E: ScriptEngine> NativeFn<E> for SubtleHmacStore {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let hash = HashAlg::from_name(&arg_string::<E>(cx, 0)?);
        let extractable = arg_string::<E>(cx, 1)? == "1";
        let usages = parse_usages(&arg_string::<E>(cx, 2)?);
        let material = arg_bytes::<E>(cx, 3)?;
        let (Some(hash), Some(usages)) = (hash, usages) else {
            return Ok(cx.make_null());
        };
        let entry = KeyEntry {
            algorithm: KeyAlgorithm::Hmac(hash),
            extractable,
            usages,
            material,
        };
        match with_keys::<E, _>(cx, |keys| keys.insert(entry)) {
            Some(id) => cx.make_string(&id.to_string()),
            None => Ok(cx.make_null()),
        }
    }
}

/// `__subtle_hmac_generate(hash, extractable, usages, byteLength)` → the new
/// key's id; `"!R"` when this host has no random source; `null` otherwise.
/// The bytes come from the same source as `crypto.getRandomValues`.
struct SubtleHmacGenerate;
impl<E: ScriptEngine> NativeFn<E> for SubtleHmacGenerate {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let hash = HashAlg::from_name(&arg_string::<E>(cx, 0)?);
        let extractable = arg_string::<E>(cx, 1)? == "1";
        let usages = parse_usages(&arg_string::<E>(cx, 2)?);
        let length = arg_string::<E>(cx, 3)?.trim().parse::<usize>().ok();
        let (Some(hash), Some(usages), Some(length)) = (hash, usages, length) else {
            return Ok(cx.make_null());
        };
        let mut material = Zeroizing::new(vec![0u8; length]);
        if !crate::crypto::fill_random::<E>(cx, &mut material) {
            return cx.make_string("!R");
        }
        let entry = KeyEntry {
            algorithm: KeyAlgorithm::Hmac(hash),
            extractable,
            usages,
            material,
        };
        match with_keys::<E, _>(cx, |keys| keys.insert(entry)) {
            Some(id) => cx.make_string(&id.to_string()),
            None => Ok(cx.make_null()),
        }
    }
}

fn key_id<E: ScriptEngine>(cx: &mut E::CallCx<'_>) -> Result<Option<u64>, E::Error> {
    Ok(arg_string::<E>(cx, 0)?.parse::<u64>().ok())
}

/// `__subtle_export(id)` → the key bytes, or `null` for an unknown or
/// non-extractable key.
struct SubtleExport;
impl<E: ScriptEngine> NativeFn<E> for SubtleExport {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id = key_id::<E>(cx)?;
        let bytes = with_keys::<E, _>(cx, |keys| keys.export(id?));
        match bytes {
            Some(bytes) => make_binary::<E>(cx, &bytes),
            None => Ok(cx.make_null()),
        }
    }
}

/// `__subtle_sign(id, data)` → the MAC, or `null` when the key is unknown or
/// lacks the `sign` usage.
struct SubtleSign;
impl<E: ScriptEngine> NativeFn<E> for SubtleSign {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id = key_id::<E>(cx)?;
        let data = arg_bytes::<E>(cx, 1)?;
        let tag = with_keys::<E, _>(cx, |keys| keys.sign(id?, &data));
        match tag {
            Some(tag) => make_binary::<E>(cx, &tag),
            None => Ok(cx.make_null()),
        }
    }
}

/// `__subtle_verify(id, data, signature)` → `"1"` or `"0"`, or `null` when the
/// key is unknown or lacks the `verify` usage.
struct SubtleVerify;
impl<E: ScriptEngine> NativeFn<E> for SubtleVerify {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id = key_id::<E>(cx)?;
        let data = arg_bytes::<E>(cx, 1)?;
        let signature = arg_bytes::<E>(cx, 2)?;
        let ok = with_keys::<E, _>(cx, |keys| keys.verify(id?, &data, &signature));
        match ok {
            Some(ok) => cx.make_string(if ok { "1" } else { "0" }),
            None => Ok(cx.make_null()),
        }
    }
}

/// `__subtle_secure()` → `"1"` when the calling realm is a secure context, so
/// the bootstrap installs its `[SecureContext]` members only where they exist.
struct SubtleSecure;
impl<E: ScriptEngine> NativeFn<E> for SubtleSecure {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let secure = cx
            .host_data()
            .and_then(|data| data.downcast_ref::<RefCell<HostState>>().map(|host| host.borrow().secure_context.0))
            .unwrap_or(true);
        cx.make_string(if secure { "1" } else { "0" })
    }
}

struct SubtleRelease;
impl<E: ScriptEngine> NativeFn<E> for SubtleRelease {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id = key_id::<E>(cx)?;
        with_keys::<E, _>(cx, |keys| {
            keys.release(id?);
            Some(())
        });
        Ok(cx.undefined())
    }
}

/// Every native sink the bootstrap captures and then deletes.
const SINKS: [&str; 8] = [
    "__subtle_digest",
    "__subtle_hmac_store",
    "__subtle_hmac_generate",
    "__subtle_export",
    "__subtle_sign",
    "__subtle_verify",
    "__subtle_release",
    "__subtle_secure",
];

pub(crate) fn install_subtle_surface<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    // A configurable slot first, as Streams does: Nova's set_function keeps an
    // existing property's attributes, so the bootstrap can then capture and
    // delete every sink on both engines before author code runs.
    for name in SINKS {
        engine.eval(&format!(
            "Object.defineProperty(globalThis, '{name}', {{ value: undefined, writable: true, configurable: true }});"
        ))?;
    }
    engine.set_function::<SubtleDigest>("__subtle_digest", 2)?;
    engine.set_function::<SubtleHmacStore>("__subtle_hmac_store", 4)?;
    engine.set_function::<SubtleHmacGenerate>("__subtle_hmac_generate", 4)?;
    engine.set_function::<SubtleExport>("__subtle_export", 1)?;
    engine.set_function::<SubtleSign>("__subtle_sign", 2)?;
    engine.set_function::<SubtleVerify>("__subtle_verify", 3)?;
    engine.set_function::<SubtleRelease>("__subtle_release", 1)?;
    engine.set_function::<SubtleSecure>("__subtle_secure", 0)?;
    engine.eval(SUBTLE_BOOTSTRAP)?;
    Ok(())
}

const SUBTLE_BOOTSTRAP: &str = include_str!("subtle/bootstrap.js");

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// RFC 2202 test case 2 (SHA-1) and RFC 4231 test case 2 (SHA-2).
    #[test]
    fn hmac_matches_the_rfc_vectors() {
        let (key, data) = (b"Jefe".as_slice(), b"what do ya want for nothing?".as_slice());
        let cases = [
            (HashAlg::Sha1, "effcdf6ae5eb2fa2d27416d5f184df9c259a7c79"),
            (HashAlg::Sha256, "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"),
            (HashAlg::Sha384, "af45d2e376484031617f78d2b58a6b1b9c7ef464f5a01b47e42ec3736322445e8e2240ca5e69e2c78b3239ecfab21649"),
            (HashAlg::Sha512, "164b7a7bfcf819e2e395fbe73b56e0a387bd64222e831fd610270cd7ea2505549758bf75c05a994a6d034f65f8f0e6fdcaeab1a34d4a6b4b636e070a38bce737"),
        ];
        for (hash, expected) in cases {
            let tag = hmac_sign(hash, key, data);
            assert_eq!(hex(&tag), expected, "{hash:?}");
            assert!(hmac_verify(hash, key, data, &tag));
            assert!(!hmac_verify(hash, key, data, &tag[..tag.len() - 1]), "truncated tag");
            let mut altered = tag.clone();
            altered[0] ^= 1;
            assert!(!hmac_verify(hash, key, data, &altered), "altered tag");
        }
    }

    #[test]
    fn usages_parse_strictly() {
        assert_eq!(parse_usages(""), Some(0));
        assert_eq!(parse_usages("sign,verify"), usage_bit("sign").zip(usage_bit("verify")).map(|(a, b)| a | b));
        assert_eq!(parse_usages("sign,bogus"), None);
        assert_eq!(HashAlg::from_name("sha-256"), None, "names arrive normalised");
    }

    #[test]
    fn registry_ids_do_not_recycle_and_release_is_idempotent() {
        let entry = || KeyEntry {
            algorithm: KeyAlgorithm::Hmac(HashAlg::Sha256),
            extractable: false,
            usages: usage_bit("sign").unwrap(),
            material: Zeroizing::new(vec![7; 32]),
        };
        let mut registry = KeyRegistry::default();
        let first = registry.insert(entry()).unwrap();
        registry.release(first);
        registry.release(first);
        let second = registry.insert(entry()).unwrap();
        assert_ne!(first, second);
        assert!(registry.get(first).is_none());
        let kept = registry.get(second).unwrap();
        assert!(kept.allows("sign") && !kept.allows("verify"));
    }

    /// The registry is the authority: whatever the JS side believes, bytes
    /// leave only an extractable key and each operation needs its usage.
    #[test]
    fn registry_enforces_extractability_and_usages() {
        let mut registry = KeyRegistry::default();
        let mut insert = |extractable, usages: &str| {
            registry
                .insert(KeyEntry {
                    algorithm: KeyAlgorithm::Hmac(HashAlg::Sha256),
                    extractable,
                    usages: parse_usages(usages).unwrap(),
                    material: Zeroizing::new(b"Jefe".to_vec()),
                })
                .unwrap()
        };
        let sealed_verifier = insert(false, "verify");
        let open_signer = insert(true, "sign");
        let registry = registry;
        let data = b"what do ya want for nothing?";

        assert!(registry.export(sealed_verifier).is_none(), "non-extractable bytes stay native");
        assert!(registry.sign(sealed_verifier, data).is_none(), "verify-only key cannot sign");
        assert_eq!(registry.export(open_signer).as_deref().map(Vec::as_slice), Some(b"Jefe".as_slice()));
        let tag = registry.sign(open_signer, data).unwrap();
        assert!(registry.verify(open_signer, data, &tag).is_none(), "sign-only key cannot verify");
        assert_eq!(registry.verify(sealed_verifier, data, &tag), Some(true));
        assert!(registry.export(u64::MAX).is_none(), "unknown ids name nothing");
    }

    fn runtime_key_lifecycle<E: ScriptEngine>() {
        let mut runtime = crate::Runtime::<E>::new().expect("runtime");
        runtime
            .eval(
                "crypto.subtle.generateKey({name: 'HMAC', hash: 'SHA-256'}, false, ['sign'])\
                 .then(function(k){ globalThis.key = k; });",
            )
            .expect("generate");
        runtime.run_event_loop(10).expect("settle");
        assert_eq!(runtime.host().borrow().crypto_keys.len(), 1);
        runtime.eval("key = null").expect("drop the wrapper");
        for _ in 0..6 {
            runtime.collect_garbage();
            runtime.run_event_loop(0).expect("pump cleanup jobs");
            if runtime.host().borrow().crypto_keys.len() == 0 {
                break;
            }
        }
        assert_eq!(
            runtime.host().borrow().crypto_keys.len(),
            0,
            "collection plus job pumping releases the key material"
        );

        runtime
            .eval("crypto.subtle.importKey('raw', new Uint8Array(16), {name: 'HMAC', hash: 'SHA-1'}, true, ['verify']).then(function(k){ globalThis.kept = k; });")
            .expect("import");
        runtime.run_event_loop(10).expect("settle");
        let retained_host = runtime.host().clone();
        assert_eq!(retained_host.borrow().crypto_keys.len(), 1);
        drop(runtime);
        assert_eq!(
            retained_host.borrow().crypto_keys.len(),
            0,
            "runtime teardown clears key material even while HostState is retained"
        );
    }

    #[test]
    fn runtime_key_lifecycle_on_boa() {
        runtime_key_lifecycle::<script_engine_boa::BoaEngine>();
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn runtime_key_lifecycle_on_vano() {
        runtime_key_lifecycle::<script_engine_nova::NovaEngine>();
    }
}
