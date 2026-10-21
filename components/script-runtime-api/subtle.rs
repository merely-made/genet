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
//! `sha1`, `sha2`, `hmac`, `hkdf` and `pbkdf2`. A derived key (`deriveKey`) is
//! made and stored without its bytes ever reaching script.
//!
//! Bytes cross as Latin-1 binary strings (one char per byte), the convention
//! `__crypto_random` and `TextDecoder` already use.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use hkdf::Hkdf;
use hmac::{Hmac, KeyInit, Mac};
use pbkdf2::pbkdf2_hmac;
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

/// HKDF-Expand over the HKDF-Extract of `ikm` and `salt` (RFC 5869). `false`
/// when `out` is longer than 255 hash lengths.
fn hkdf_derive(hash: HashAlg, ikm: &[u8], salt: &[u8], info: &[u8], out: &mut [u8]) -> bool {
    macro_rules! go {
        ($digest:ty) => {
            Hkdf::<$digest>::new(Some(salt), ikm).expand(info, out).is_ok()
        };
    }
    match hash {
        HashAlg::Sha1 => go!(Sha1),
        HashAlg::Sha256 => go!(Sha256),
        HashAlg::Sha384 => go!(Sha384),
        HashAlg::Sha512 => go!(Sha512),
    }
}

/// PBKDF2 with HMAC as the PRF (RFC 8018). `false` for zero iterations.
fn pbkdf2_derive(hash: HashAlg, password: &[u8], salt: &[u8], iterations: u32, out: &mut [u8]) -> bool {
    if iterations == 0 {
        return false;
    }
    match hash {
        HashAlg::Sha1 => pbkdf2_hmac::<Sha1>(password, salt, iterations, out),
        HashAlg::Sha256 => pbkdf2_hmac::<Sha256>(password, salt, iterations, out),
        HashAlg::Sha384 => pbkdf2_hmac::<Sha384>(password, salt, iterations, out),
        HashAlg::Sha512 => pbkdf2_hmac::<Sha512>(password, salt, iterations, out),
    }
    true
}

/// What a key is for. HKDF and PBKDF2 keys only derive; an AES key is stored
/// here for `deriveKey` and holds no cipher state (the ciphers are a later phase).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyAlgorithm {
    Hmac(HashAlg),
    Hkdf,
    Pbkdf2,
    Aes,
}

/// The KDF a derive request names, with its algorithm-specific input.
enum Kdf {
    Hkdf { info: Zeroizing<Vec<u8>> },
    Pbkdf2 { iterations: u32 },
}

/// One `deriveBits` or `deriveKey` request, already decoded.
struct DeriveRequest {
    id: u64,
    usage: String,
    kdf: Kdf,
    hash: HashAlg,
    salt: Zeroizing<Vec<u8>>,
    bytes: usize,
}

/// The key a `deriveKey` stores: HMAC with its hash, or AES.
struct DerivedKey {
    algorithm: KeyAlgorithm,
    extractable: bool,
    usages: u16,
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
            _ => None,
        }
    }

    /// Whether `signature` is `data`'s MAC, only for a known key holding the
    /// `verify` usage.
    fn verify(&self, id: u64, data: &[u8], signature: &[u8]) -> Option<bool> {
        let entry = self.get(id).filter(|entry| entry.allows("verify"))?;
        match entry.algorithm {
            KeyAlgorithm::Hmac(hash) => Some(hmac_verify(hash, &entry.material, data, signature)),
            _ => None,
        }
    }

    /// The derived bytes, only for a known key of the named KDF holding the
    /// usage the operation needs. `None` also covers a refusal by the KDF
    /// (zero iterations, an HKDF output past 255 hash lengths) and an
    /// allocation that fails.
    fn derive(&self, request: &DeriveRequest) -> Option<Zeroizing<Vec<u8>>> {
        if !matches!(request.usage.as_str(), "deriveBits" | "deriveKey") {
            return None;
        }
        let entry = self.get(request.id).filter(|entry| entry.allows(&request.usage))?;
        let mut out = Zeroizing::new(Vec::new());
        out.try_reserve_exact(request.bytes).ok()?;
        out.resize(request.bytes, 0);
        let derived = match (entry.algorithm, &request.kdf) {
            (KeyAlgorithm::Hkdf, Kdf::Hkdf { info }) => {
                hkdf_derive(request.hash, &entry.material, &request.salt, info, &mut out)
            }
            (KeyAlgorithm::Pbkdf2, Kdf::Pbkdf2 { iterations }) => {
                pbkdf2_derive(request.hash, &entry.material, &request.salt, *iterations, &mut out)
            }
            _ => false,
        };
        derived.then_some(out)
    }

    /// Derive and store in one step, so the new key's bytes stay native. An
    /// HMAC key needs a non-empty secret and an AES key 128, 192 or 256 bits.
    fn derive_key(&mut self, request: &DeriveRequest, key: DerivedKey) -> Option<u64> {
        let lengths_fit = match key.algorithm {
            KeyAlgorithm::Hmac(_) => request.bytes > 0,
            KeyAlgorithm::Aes => matches!(request.bytes, 16 | 24 | 32),
            KeyAlgorithm::Hkdf | KeyAlgorithm::Pbkdf2 => false,
        };
        if !lengths_fit {
            return None;
        }
        let material = self.derive(request)?;
        self.insert(KeyEntry {
            algorithm: key.algorithm,
            extractable: key.extractable,
            usages: key.usages,
            material,
        })
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

/// `__subtle_secret_store(kind, extractable, usages, bytes)` → the new key's
/// id, or `null`. `kind` is `HKDF`, `PBKDF2` or `AES`. A KDF base key is never
/// extractable: the registry refuses to store one that is.
struct SubtleSecretStore;
impl<E: ScriptEngine> NativeFn<E> for SubtleSecretStore {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let algorithm = match arg_string::<E>(cx, 0)?.as_str() {
            "HKDF" => Some(KeyAlgorithm::Hkdf),
            "PBKDF2" => Some(KeyAlgorithm::Pbkdf2),
            "AES" => Some(KeyAlgorithm::Aes),
            _ => None,
        };
        let extractable = arg_string::<E>(cx, 1)? == "1";
        let usages = parse_usages(&arg_string::<E>(cx, 2)?);
        let material = arg_bytes::<E>(cx, 3)?;
        let derive_only = matches!(algorithm, Some(KeyAlgorithm::Hkdf | KeyAlgorithm::Pbkdf2));
        let (Some(algorithm), Some(usages)) = (algorithm, usages) else {
            return Ok(cx.make_null());
        };
        if derive_only && extractable {
            return Ok(cx.make_null());
        }
        let entry = KeyEntry {
            algorithm,
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

/// Decode a derive call's first seven arguments:
/// `(id, usage, algorithm, hash, salt, info | iterations, bits)`.
fn derive_request<E: ScriptEngine>(cx: &mut E::CallCx<'_>) -> Result<Option<DeriveRequest>, E::Error> {
    let id = key_id::<E>(cx)?;
    let usage = arg_string::<E>(cx, 1)?;
    let algorithm = arg_string::<E>(cx, 2)?;
    let hash = HashAlg::from_name(&arg_string::<E>(cx, 3)?);
    let salt = arg_bytes::<E>(cx, 4)?;
    let kdf = match algorithm.as_str() {
        "HKDF" => Some(Kdf::Hkdf { info: arg_bytes::<E>(cx, 5)? }),
        "PBKDF2" => arg_string::<E>(cx, 5)?
            .parse::<u32>()
            .ok()
            .map(|iterations| Kdf::Pbkdf2 { iterations }),
        _ => None,
    };
    let bits = arg_string::<E>(cx, 6)?.trim().parse::<usize>().ok();
    let (Some(id), Some(hash), Some(kdf), Some(bits)) = (id, hash, kdf, bits) else {
        return Ok(None);
    };
    // Whole bytes only; the bootstrap has already refused anything else.
    Ok((bits % 8 == 0).then_some(DeriveRequest {
        id,
        usage,
        kdf,
        hash,
        salt,
        bytes: bits / 8,
    }))
}

/// `__subtle_derive(id, usage, algorithm, hash, salt, info | iterations, bits)`
/// → the derived bytes, or `null` on any refusal.
struct SubtleDerive;
impl<E: ScriptEngine> NativeFn<E> for SubtleDerive {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let Some(request) = derive_request::<E>(cx)? else {
            return Ok(cx.make_null());
        };
        match with_keys::<E, _>(cx, |keys| keys.derive(&request)) {
            Some(bytes) => make_binary::<E>(cx, &bytes),
            None => Ok(cx.make_null()),
        }
    }
}

/// `__subtle_derive_key(…the derive arguments…, kind, hash, extractable,
/// usages)` → the new key's id, or `null`. The derived bytes go straight into
/// the registry; script sees only the id.
struct SubtleDeriveKey;
impl<E: ScriptEngine> NativeFn<E> for SubtleDeriveKey {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let request = derive_request::<E>(cx)?;
        let kind = arg_string::<E>(cx, 7)?;
        let key_hash = HashAlg::from_name(&arg_string::<E>(cx, 8)?);
        let extractable = arg_string::<E>(cx, 9)? == "1";
        let usages = parse_usages(&arg_string::<E>(cx, 10)?);
        let algorithm = match (kind.as_str(), key_hash) {
            ("HMAC", Some(hash)) => Some(KeyAlgorithm::Hmac(hash)),
            ("AES", _) => Some(KeyAlgorithm::Aes),
            _ => None,
        };
        let (Some(request), Some(algorithm), Some(usages)) = (request, algorithm, usages) else {
            return Ok(cx.make_null());
        };
        let key = DerivedKey {
            algorithm,
            extractable,
            usages,
        };
        match with_keys::<E, _>(cx, |keys| keys.derive_key(&request, key)) {
            Some(id) => cx.make_string(&id.to_string()),
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
const SINKS: [&str; 11] = [
    "__subtle_digest",
    "__subtle_hmac_store",
    "__subtle_hmac_generate",
    "__subtle_export",
    "__subtle_sign",
    "__subtle_verify",
    "__subtle_release",
    "__subtle_secure",
    "__subtle_secret_store",
    "__subtle_derive",
    "__subtle_derive_key",
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
    engine.set_function::<SubtleSecretStore>("__subtle_secret_store", 4)?;
    engine.set_function::<SubtleDerive>("__subtle_derive", 7)?;
    engine.set_function::<SubtleDeriveKey>("__subtle_derive_key", 11)?;
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

    /// RFC 5869 appendix A.1 and A.3 (SHA-256), A.4 (SHA-1).
    #[test]
    fn hkdf_matches_the_rfc_vectors() {
        let unhex = |text: &str| -> Vec<u8> {
            (0..text.len() / 2).map(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).unwrap()).collect()
        };
        let mut okm = vec![0u8; 42];
        assert!(hkdf_derive(
            HashAlg::Sha256,
            &unhex("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"),
            &unhex("000102030405060708090a0b0c"),
            &unhex("f0f1f2f3f4f5f6f7f8f9"),
            &mut okm,
        ));
        assert_eq!(
            hex(&okm),
            "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865"
        );
        // A.3: empty salt and info.
        assert!(hkdf_derive(HashAlg::Sha256, &unhex("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b"), b"", b"", &mut okm));
        assert_eq!(
            hex(&okm),
            "8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8"
        );
        // A.4, SHA-1.
        let mut okm = vec![0u8; 42];
        assert!(hkdf_derive(
            HashAlg::Sha1,
            &unhex("0b0b0b0b0b0b0b0b0b0b0b"),
            &unhex("000102030405060708090a0b0c"),
            &unhex("f0f1f2f3f4f5f6f7f8f9"),
            &mut okm,
        ));
        assert_eq!(
            hex(&okm),
            "085a01ea1b10f36933068b56efa5ad81a4f14b822f5b091568a9cdd4f155fda2c22e422478d305f3f896"
        );
        // 255 hash lengths is the ceiling; one byte more is refused.
        assert!(hkdf_derive(HashAlg::Sha256, b"k", b"", b"", &mut vec![0u8; 255 * 32]));
        assert!(!hkdf_derive(HashAlg::Sha256, b"k", b"", b"", &mut vec![0u8; 255 * 32 + 1]));
        assert!(hkdf_derive(HashAlg::Sha256, b"k", b"", b"", &mut []), "zero length derives nothing");
    }

    /// RFC 6070 test vectors (SHA-1) and the widely published SHA-256 pair.
    #[test]
    fn pbkdf2_matches_the_published_vectors() {
        let mut out = [0u8; 20];
        assert!(pbkdf2_derive(HashAlg::Sha1, b"password", b"salt", 1, &mut out));
        assert_eq!(hex(&out), "0c60c80f961f0e71f3a9b524af6012062fe037a6");
        assert!(pbkdf2_derive(HashAlg::Sha1, b"password", b"salt", 4096, &mut out));
        assert_eq!(hex(&out), "4b007901b765489abead49d926f721d065a429c1");
        let mut out = [0u8; 32];
        assert!(pbkdf2_derive(HashAlg::Sha256, b"password", b"salt", 1, &mut out));
        assert_eq!(hex(&out), "120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b");
        assert!(!pbkdf2_derive(HashAlg::Sha256, b"password", b"salt", 0, &mut out), "zero iterations");
        assert!(pbkdf2_derive(HashAlg::Sha256, b"", b"", 1, &mut []), "empty password, salt and output");
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

    fn kdf_request(id: u64, usage: &str, kdf: Kdf, bytes: usize) -> DeriveRequest {
        DeriveRequest {
            id,
            usage: usage.to_owned(),
            kdf,
            hash: HashAlg::Sha256,
            salt: Zeroizing::new(b"salt".to_vec()),
            bytes,
        }
    }

    /// The registry decides who may derive: the key's algorithm must be the
    /// KDF named, and it must hold the usage the operation needs.
    #[test]
    fn registry_enforces_kdf_algorithm_and_usages() {
        let mut registry = KeyRegistry::default();
        let mut insert = |algorithm, usages: &str| {
            registry
                .insert(KeyEntry {
                    algorithm,
                    extractable: false,
                    usages: parse_usages(usages).unwrap(),
                    material: Zeroizing::new(b"secret".to_vec()),
                })
                .unwrap()
        };
        let hkdf_key = insert(KeyAlgorithm::Hkdf, "deriveBits");
        let pbkdf2_key = insert(KeyAlgorithm::Pbkdf2, "deriveKey,deriveBits");
        let hmac_key = insert(KeyAlgorithm::Hmac(HashAlg::Sha256), "sign");
        let info = || Kdf::Hkdf { info: Zeroizing::new(b"info".to_vec()) };
        let iterations = || Kdf::Pbkdf2 { iterations: 2 };

        assert_eq!(registry.derive(&kdf_request(hkdf_key, "deriveBits", info(), 16)).map(|b| b.len()), Some(16));
        assert!(registry.derive(&kdf_request(hkdf_key, "deriveKey", info(), 16)).is_none(), "missing usage");
        assert!(registry.derive(&kdf_request(hkdf_key, "sign", info(), 16)).is_none(), "not a derive usage");
        assert!(registry.derive(&kdf_request(hkdf_key, "deriveBits", iterations(), 16)).is_none(), "wrong KDF");
        assert!(registry.derive(&kdf_request(hmac_key, "deriveBits", info(), 16)).is_none(), "not a KDF key");
        assert!(registry.derive(&kdf_request(u64::MAX, "deriveBits", info(), 16)).is_none(), "unknown key");
        assert!(registry.derive(&kdf_request(pbkdf2_key, "deriveBits", Kdf::Pbkdf2 { iterations: 0 }, 16)).is_none());
        assert!(
            registry.derive(&kdf_request(hkdf_key, "deriveBits", info(), usize::MAX)).is_none(),
            "an allocation that cannot be made is a refusal"
        );

        // deriveKey stores natively: AES takes 16, 24 or 32 bytes, HMAC any non-empty.
        let aes = || DerivedKey { algorithm: KeyAlgorithm::Aes, extractable: true, usages: usage_bit("encrypt").unwrap() };
        let hmac = || DerivedKey { algorithm: KeyAlgorithm::Hmac(HashAlg::Sha1), extractable: false, usages: usage_bit("sign").unwrap() };
        let derived = registry.derive_key(&kdf_request(pbkdf2_key, "deriveKey", iterations(), 32), aes()).unwrap();
        assert_eq!(registry.export(derived).map(|bytes| bytes.len()), Some(32));
        assert!(registry.derive_key(&kdf_request(pbkdf2_key, "deriveKey", iterations(), 20), aes()).is_none());
        assert!(registry.derive_key(&kdf_request(pbkdf2_key, "deriveKey", iterations(), 0), hmac()).is_none());
        let sealed = registry.derive_key(&kdf_request(pbkdf2_key, "deriveKey", iterations(), 20), hmac()).unwrap();
        assert!(registry.export(sealed).is_none(), "a non-extractable derived key stays native");
        assert!(registry.sign(sealed, b"data").is_some());
        assert!(registry.sign(derived, b"data").is_none(), "an AES key signs nothing");
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

    /// A key made by `deriveKey` is registered natively and released with its
    /// wrapper, like any other; the base key's wrapper is independent.
    fn runtime_derived_key_lifecycle<E: ScriptEngine>() {
        let mut runtime = crate::Runtime::<E>::new().expect("runtime");
        runtime
            .eval(
                "crypto.subtle.importKey('raw', new Uint8Array(8), {name: 'HKDF'}, false, ['deriveKey'])                 .then(function(base){ globalThis.base = base;                  return crypto.subtle.deriveKey({name: 'HKDF', hash: 'SHA-256', salt: new Uint8Array(1), info: new Uint8Array(1)},                  base, {name: 'AES-GCM', length: 128}, false, ['encrypt']); })                 .then(function(k){ globalThis.derived = k; });",
            )
            .expect("derive");
        runtime.run_event_loop(10).expect("settle");
        assert_eq!(runtime.host().borrow().crypto_keys.len(), 2, "base key and derived key");
        runtime.eval("derived = null").expect("drop the derived wrapper");
        for _ in 0..6 {
            runtime.collect_garbage();
            runtime.run_event_loop(0).expect("pump cleanup jobs");
            if runtime.host().borrow().crypto_keys.len() == 1 {
                break;
            }
        }
        assert_eq!(runtime.host().borrow().crypto_keys.len(), 1, "only the derived key was released");
        let retained_host = runtime.host().clone();
        drop(runtime);
        assert_eq!(retained_host.borrow().crypto_keys.len(), 0, "teardown clears the rest");
    }

    #[test]
    fn runtime_derived_key_lifecycle_on_boa() {
        runtime_derived_key_lifecycle::<script_engine_boa::BoaEngine>();
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn runtime_derived_key_lifecycle_on_vano() {
        runtime_derived_key_lifecycle::<script_engine_nova::NovaEngine>();
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
