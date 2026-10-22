# Web Cryptography: `crypto.subtle` in the scripted tier

**Status:** In progress, 2026-10-20. P1 (framework, digest, HMAC and Web Crypto's
secure-context gate) and P2 (HKDF, PBKDF2, with AES keys created from raw bytes
for `deriveKey`) are implemented and measured on branch `webcrypto-subtle`.
They are **not merged**. P1's matched WebCryptoAPI maps lose zero passes on both
engines: Boa 81 → 5,472 and Vano 80 → 7,980 passing subtests. P2's lose zero on
Vano (7,980 → 17,698) and two `setup` passes on Boa (5,472 → 6,909), which Mark
waived as Boa's F11 defect. P3 (AES ciphers and wrap/unwrap) is next by the ruled
order. Mark's answers from three card rounds are recorded below.

Authority: [standards ledger](2026-09-07_standards_to_features_ledger.md) row 10
(Web Crypto, registry id `webcrypto-2`), with WebCrypto picked as a conformance
target on 2026-10-20. Mark's standing security rule applies: privacy, security
and crypto code uses maintained libraries and standards only. No own crypto math
and no crypto forks. The interface is taken from the vendored WebIDL
`tests/wpt/tests/interfaces/webcrypto.idl` (Level 2; `CryptoKey` and
`SubtleCrypto` are `[SecureContext, Exposed=(Window,Worker)]`), plus
`webcrypto-secure-curves.idl` and `webcrypto-modern-algos.idl` for the drafts.
Behaviour comes from [Web Cryptography Level 2](https://www.w3.org/TR/webcrypto-2/)
and the vendored WPT `tests/wpt/tests/WebCryptoAPI/`. No source is copied from
another engine.

## Rulings (Mark's words; unnumbered until Mark numbers them)

Put to Mark as one card round on 2026-10-20, each card carrying its evidence and
a recommendation. The full evidence is under Findings.

- **Crates.** Mark: **"RustCrypto 0.11 line (Recommended)"**. The digest-0.11
  generation of RustCrypto, pure Rust, all MIT OR Apache-2.0 except the dalek
  crates (BSD-3-Clause, already on deny.toml's allow list):
  - Phase 1 adds `sha1` 0.11.0, `sha2` 0.11.1 and `hmac` 0.13.0.
  - The existing `sha2` 0.10 consumers (fleece, genet-scripted, genet-wpt) move
    to 0.11 in the same change, so cargo-deny's multiple-versions ban holds.
  - tungstenite's transitive `sha1` 0.10 stays and is recorded.
  - Later phases name `hkdf`/`pbkdf2` 0.13.0, `aes` 0.9.3, `aes-gcm` 0.11.1,
    `aes-kw` 0.3.1, `ctr` 0.10.1, `cbc` 0.2.1, `p256`/`p384`/`p521` 0.14.0,
    `ecdsa` 0.17.0, and `ed25519-dalek`/`x25519-dalek` 3.0.0.
  - RSA waits for `rsa` 0.10 stable. 0.9.10 carries the unfixed
    RUSTSEC-2023-0071; 0.10 is at rc.19.
  - ring was the declined alternative.
- **Phase order.** Mark: **"Weight order (Recommended)"**. Framework, digest and
  HMAC; then HKDF and PBKDF2; AES with wrap/unwrap; EC P-curves; Ed25519/X25519;
  RSA last, gated on `rsa` 0.10 stable.
- **Algorithms beyond the product's needs.** Mark: **"Standard + drafts with
  crates"**. The card text: also take draft algorithms where a maintained stable
  crate exists (ChaCha20-Poly1305, Ed448/X448, SHA-3/KMAC); ML-KEM and ML-DSA
  wait for crates without advisories. This covers every standard algorithm,
  legacy ones included (SHA-1, RSASSA-PKCS1-v1_5, AES-CBC/CTR). Each draft
  algorithm's crate is checked for maintenance and advisories when its phase
  opens, and a crate that fails comes back as a fork.
- **Byte transport.** Mark: **"Binary strings in P1 (Recommended)"**. Phase 1
  uses the established Latin-1 binary-string convention (`__crypto_random`,
  Encoding). Key material stays native behind an opaque id. The cost is measured
  in P1, and if it matters, BufferSource-read and typed-array-create primitives
  on `CallCx` for both engines come back as their own proposal.

- **Secure contexts** (second round, 2026-10-20, after the first matched maps
  showed `historical.any.html` losing its file and three subtests on both
  engines). Mark: **"Gate Web Crypto now (Recommended)"**. The card text:
  - a per-runtime secure-context flag;
  - hosted pages derive it from the creation URL by the Secure Contexts
    "potentially trustworthy" rule, workers inherit their owner's, and it
    defaults to secure until a URL says otherwise;
  - only `crypto.subtle`, `SubtleCrypto` and `CryptoKey` are gated;
  - genet-wpt marks a test secure when its name carries `.https.`, WPT's own
    serving rule;
  - `randomUUID` and `isSecureContext` are untouched.
- **AES keys in P2** (third round, 2026-10-20, after the assessment showed 12 of
  the 16 derived-key types in the vendored `hkdf`/`pbkdf2` files are AES, which
  gate 4,644 of 12,291 window-scope subtests on an AES key existing). Mark:
  **"Raw import/export only (Recommended)"**. The card text: register AES-CBC,
  AES-CTR, AES-GCM and AES-KW for `importKey` and "get key length"; raw and
  `raw-secret` import and raw export; no new crate. `jwk`, `generateKey` and
  every cipher stay P3.
- **Boa's two lost passes** (fourth round, 2026-10-20). Mark: **"Keep AES import,
  waive the 2 (Recommended)"**. The two are the `setup` subtests of
  `encrypt_decrypt/aes_gcm` and `aes_gcm_256_iv`, errored on Boa by F11 (see F14).
  They are recorded as F11-attributed and re-measured when the F11 fix lands.
- **P1's lane-level choices** (same round as the secure-context ruling). Mark: **"Keep both
  (Recommended)"**. `zeroize` stays a direct dependency, clearing released key
  material. The draft "Modern Algorithms" KeyFormat values are accepted, with
  `raw-secret` aliasing `raw` for HMAC, and later for AES and the KDFs.

## Phases

Weights are reference subtest counts over window and dedicated-worker scopes
(Findings F1). They are not census counts.

1. **P1: framework, digest, HMAC.** About 16,200 reference subtests:
   - algorithm normalisation 14,312;
   - HMAC 1,582;
   - digest SHA-1/2 232;
   - the getRandomValues residual 84.

   It adds `SubtleCrypto` on `crypto.subtle` in both Window and Worker scopes,
   and a `CryptoKey` interface with cached `algorithm`/`usages`. Every standard
   operation is present: an unimplemented algorithm is normalised and rejects
   with `NotSupportedError`. P1 implements `digest` for SHA-1, SHA-256, SHA-384
   and SHA-512, plus HMAC `generateKey`, `importKey`/`exportKey` (`raw`,
   `raw-secret`, `jwk`), and `sign`/`verify`. Key material lives in a per-realm
   native registry (the Encoding pattern) and is zeroised on release.
   `crypto.subtle`, `SubtleCrypto` and `CryptoKey` exist only in a secure
   context (`secure_context.rs`, `Runtime::set_secure_context`). The runner
   applies WPT's `.https.` serving rule.

   Done when:
   - the crate gates for script-runtime-api pass on Boa and Vano, with new
     regressions at the runtime layer;
   - cargo-deny's bans and licences hold for the new graph;
   - before/after WebCryptoAPI maps on both engines show zero lost passes, with
     every movement attributed.
2. **P2: HKDF and PBKDF2.** 24,678 reference subtests, 12,291 of them in the
   window scope the runner can measure (F12). Import `raw`/`raw-secret`,
   `deriveBits`, and `deriveKey` into HMAC and AES keys, with `hkdf` and
   `pbkdf2` 0.13.0. AES key *creation* through `deriveKey` needs only a raw AES
   import, which Mark ruled into this phase (third round); the rest of AES is P3.
   The derived bytes of a `deriveKey` go from the KDF into the registry without
   reaching script.

   Done when the KDF operations pass the vendored `hkdf` and `pbkdf2` files on
   Vano, new both-engine regressions sit beside `tests/subtle_crypto.rs`,
   cargo-deny's bans and licences hold, and the matched maps meet the same
   zero-loss bar. The files cannot pass whole before P4: 2,652 of their window
   subtests need an ECDH P-256 key (F13).
3. **P3: AES-CBC, AES-CTR, AES-GCM, AES-KW, with wrapKey/unwrapKey.** 7,780.
4. **P4: EC P-256/P-384/P-521, ECDSA and ECDH, with `spki`/`pkcs8`/`jwk`/`raw`.** 4,434.
5. **P5: Ed25519 and X25519.** 2,892.
6. **P6: RSA-OAEP, RSA-PSS, RSASSA-PKCS1-v1_5.** 4,164. Gated on `rsa` 0.10
   stable.
7. **Drafts with maintained crates** (the third ruling), ordered by the same
   weight table when reached. AES-OCB is not in the card's list; it comes back
   as a question.

Each phase's done-conditions repeat P1's three. Structured cloning of
`CryptoKey` (Worker and storage transport) belongs to whichever phase first
needs it, at the latest P5.

## Findings

**F1 (2026-10-20). The census undercounts WebCryptoAPI by more than seven
times.** The 2026-09-29 Vano census reports 107 of 9,652 subtests over 274 files.
149 files are `no-results`, because their top level reaches `crypto.subtle`
before registering any test. Files such as `hkdf` and `pbkdf2` report one
`setup` subtest per variant.

The vendored tests were enumerated under Node 24.11's WebCrypto with the
vendored `testharness.js` in its shell environment. That is a counting
instrument, not a conformance claim. Each `.any.js` was run once with no
variant (all subtests), and the total doubled for its `.any.html` and
`.any.worker.html` scopes. The result is 72,172 subtests, of which Node passes
70,612. `idlharness` (needs fetch) and the two window-only files are counted
from the census.

Each subtest was bucketed by the algorithm in its name, and failing that by
its file. Weights by family:

| Family | Subtests |
|---|---:|
| HKDF/PBKDF2 | 24,678 |
| Framework: algorithm normalisation (generateKey "Bad algorithm") | 14,312 |
| AES CBC/CTR/GCM/KW (+ wrap with them) | 7,780 |
| EC P-curves (ECDSA, ECDH) | 4,434 |
| RSA (three schemes) | 4,164 |
| Draft: ML-KEM/ML-DSA | 4,142 |
| Ed25519/X25519 | 2,892 |
| Draft: Ed448/X448 | 2,838 |
| Draft: SHA-3/cSHAKE/KMAC/Argon2 | 2,126 |
| HMAC | 1,582 |
| Draft: AES-OCB | 1,188 |
| Unclassified | 602 |
| Draft: ChaCha20-Poly1305 | 578 |
| Draft: supports()/getPublicKey() | 522 |
| digest SHA-1/2 | 232 |
| getRandomValues/randomUUID | 84 |
| Framework, cross-cutting | 18 |

Instrument and per-file output: `Code/testing/genet/webcrypto/assessment-20261020/`.

**F2 (2026-10-20). What already exists.** `crypto` is a real `Crypto` object with
`getRandomValues` and `randomUUID` over the `RandomSource` host seam
(`components/script-runtime-api/crypto.rs`). Native targets fill from the
operating system through `getrandom` 0.4. There is no wasm default; the host
installs one. The surface installs unconditionally, so Window, frames and
dedicated workers all get it. Census: getRandomValues 20/39 per scope,
randomUUID 3/3, historical 3/3.

**F3 (2026-10-20). Binding layout facts P1 builds on.**
- Bytes cross as Latin-1 binary strings. `CallCx::copy_uint8_array` accepts only
  a genuine `Uint8Array` and serves WebGL alone, and no native creates a typed
  array.
- Native state uses the Encoding pattern:
  - a `HostState` registry;
  - process-wide never-recycled ids (snapshot clones get a fresh `HostState`);
  - an agent-shared `WeakMap` brand;
  - a `FinalizationRegistry` holding only the id;
  - native sinks captured and deleted from the global before author code.
- Promises are made in the bootstrap around synchronous native sinks. Fetch and
  Encoding do this; nothing in the runtime uses `new_host_promise`.
- `DOMException` is the bootstrap's JS constructor.
- There was no `isSecureContext` and no `[SecureContext]` gating anywhere in the
  scripted tier. Disk and server modes both give pages an `http://` URL, so
  without the runner's `.https.` rule every test would be non-secure. The
  second card round added gating for Web Crypto's three members only.
- The IDL interface table generator reads only `html`, `dom`, `cssom` and
  `selection-api`. `SubtleCrypto`/`CryptoKey` are hand-shaped in the bootstrap.

**F4 (2026-10-20). Dependencies on record.** The lock holds `sha2` 0.10.9 (fleece,
genet-scripted, genet-wpt), `sha1` 0.10.7 (through tungstenite) and `ring`
0.17.14 (netfetcher SRI, rustls, quinn). The root manifest's `rsa` 0.9.10,
`x25519-dalek` 2.0.1, `p384`/`p521` 0.13, `argon2` 0.5 and `ocb3` 0.1 entries
are Servo leftovers: no member uses them and the lock has none of them. deny.toml
already ignores RUSTSEC-2023-0071 (`rsa`) and RUSTSEC-2025-0144 (`ml-dsa`
0.0.4), both inherited.

**F5 (2026-10-20). No Web Crypto spec prose is vendored.** Only the WPT IDL is in the
tree. Algorithms are implemented from the published Level 2 text, read rather
than copied. Vendoring the prose would be a separate download decision.

**F6 (2026-10-20). The product consumer's algorithms.** mere's design docs name
Ed25519, X25519, HKDF and ChaCha20-Poly1305 (plus BLAKE3, which is outside Web
Crypto). ChaCha20-Poly1305 is only in the draft "Modern Algorithms" text.

**F7 (2026-10-20). On Vano, a native sink outlives its `delete` unless its
global slot is pre-defined as configurable.** Nova's `set_function` keeps an
existing property's attributes, and a fresh one is non-configurable. Streams
already pre-defines its bridge's slot (`streams.rs`, `install_streams_surface`);
`install_subtle_surface` does the same for its seven `__subtle_*` sinks. A probe
of every `__`-prefixed own global on both engines shows Encoding's
`__text_decoder_create/decode/release` still visible on Vano and absent on Boa.
That is a pre-existing Encoding-lane defect, left for that lane. The first
version of this lane's privacy test accepted "no shared state" as an outcome and
so passed vacuously on both engines. It now asserts the absence directly and
fails without the fix.

**F8 (2026-10-20). cargo-deny.**
- **Licences pass.**
- **Bans already fail on main** with 28 duplicate crates. The digest-0.11 move
  adds exactly four: `sha1`, `digest`, `block-buffer` and `crypto-common`, all
  held at 0.10 by tungstenite's `sha1` 0.10.7. They are recorded as deny skips
  under the crates ruling, so the duplicate set is identical to main's.
- **Advisories touch no new crate.**
- The wasm32 cone of script-runtime-api carries no `getrandom` of any version.
- **Advisories outside this lane.** rustls 0.23.45, used through netfetcher,
  murm-wire and quinn, carries RUSTSEC-2026-0346 to 0349 (session `Debug`
  leaking TLS secrets, ECH rejection, Brotli certificate allocation, ServerHello
  downgrade), all fixed in 0.23.46. This is for whoever owns netfetcher.

**F9 (2026-10-20). getRandomValues' eight "Large length" subtests per scope need
WebIDL's `QuotaExceededError` interface.** The interface derives from
`DOMException` and carries `quota` and `requested`; testharness checks
`self.QuotaExceededError`. It is a WebIDL/DOM surface used by storage and
messaging too, so it is out of this lane. The subclass and Float16 failures were
`constructor.name` lookups and are fixed here by the intrinsic
`%TypedArray%.prototype[@@toStringTag]`.

**F10 (2026-10-20). A pre-cargo instrument for the bootstrap.** The bootstrap was
run under Node with Node's hash and HMAC standing in for the native sinks, and
the vendored WPT files were driven against it. Results:

| File | Pass |
|---|---|
| `digest` | 116/116 |
| `generateKey` HMAC successes | 192/192 |
| `generateKey` HMAC failures | 616/616 |
| HMAC sign/verify | 57/65 (the other 8 need an ECDSA key) |
| `symmetric_importKey` HMAC rows | every one |

This checks JS logic only. The engine results are the maps below. Instrument:
`Code/testing/genet/webcrypto/p1-20261020/instruments/`.

**Lane-level choices in P1, for Mark's review.** Neither was on the card.
- **`zeroize` as a direct dependency** of script-runtime-api. It was already in
  the lock and the workspace table, and it clears released key material.
- **`raw-secret` accepted as an alias of `raw` for HMAC.** That format is from
  the draft "Modern Algorithms" text; the KeyFormat enum in that draft is
  accepted so its values are not TypeErrors.

**F11 (2026-10-20). Boa gives up on WebCryptoAPI files with roughly 300 or
more promise tests.** On Boa, 29 files end in `evaluation-threw`. Vano runs the
same files. Instrumenting the runner's turn errors with `describe_error`
(reverted) shows `RuntimeLimit(StackSize)`. The sequence is:
1. testharness's harness-level timeout (`Tests.prototype.set_timeout`'s
   callback) fires;
2. it times out the remaining tests recursively;
3. that recursion overflows Boa's default value-stack limit.

The run lasts about two seconds, so this is not slowness in the operations.
Evidence that it's scale and not correctness:
- Each algorithm's slice of `symmetric_importKey` passes on Boa alone (HMAC
  240/240).
- The AES `generateKey` files (288 to 774 tests) already errored on Boa before
  this lane.

No pass is lost, but Boa's WebCryptoAPI gains stay below Vano's until the
runner or Boa limit is addressed. That belongs to the runner/Boa lane, not here.
The `boa-harness-timeout` lane has since traced the cause below the timeout
(F14); read the mechanism there.

**F12 (2026-10-20). Only the window scope is measurable.** Every `*.any.worker.html`
file errors on both engines, before and after: `harness-error: NotSupportedError:
Worker cancellation is unsupported by this script engine` (136 files on each).
The reference counts double each file for its worker scope, so half of the
24,678 KDF subtests are unreachable in the runner whatever the operations do.
The window scope is 3,660 (`hkdf`) plus 8,631 (`pbkdf2`) subtests, setup
excluded. The worker scope runs on the product's own worker path, as
`tests/subtle_crypto.rs` exercises it, not through this runner.

**F13 (2026-10-20). The `wrongKey` subtests need an EC key.** Both KDF drivers
build `wrongKey` with `generateKey({name: "ECDH", namedCurve: "P-256"})` and
expect `InvalidAccessError` when it is used as the KDF base key. Without EC,
`wrongKey` is `null` and the call rejects with `TypeError`. The window scope has
816 such subtests in `hkdf` and 1,836 in `pbkdf2`, 2,652 in all. After P2 on Vano
they are the *only* failures in either file (checked by name), so both files
finish the moment P4 lands `generateKey` for ECDH. 2,496 of the 2,652 are
`deriveKey` rows. Separately, 4,644 other `deriveKey` rows (AES targets: success
and missing-usage) needed an AES key to exist, which is the third-round ruling;
the AES bad-hash and non-digest rows passed without it.

**F14 (2026-10-20). F11 is a value-stack leak, and AES import brings two more
files under it.** The `boa-harness-timeout` lane showed at runtime that Boa's VM
does not truncate the value stack when a frame above the thrower catches an
exception, so a run of caught cross-frame throws (a failing `promise_test` per
test) overruns the 10,240-slot limit. The overrun drops every queued promise job;
the harness timeout seen in F11 is a symptom. Vano has no such limit. P2's AES
import lets `encrypt_decrypt/aes_gcm` and `aes_gcm_256_iv` define their real
tests instead of failing at import. Both then error on Boa and report no
subtests, so their `setup` passes read as lost. Same file, same runner, vectors
trimmed in the worktree and restored:

| Tests the file defines | Boa result |
|---:|---|
| 37 | runs, 1 pass |
| 109 | runs, 1 pass |
| 187 | runs, 1 pass |
| 265 | `evaluation-threw`, no subtests |
| 577 (the file as shipped) | `evaluation-threw`, no subtests |

Mark waived the two passes (fourth round). The Boa lane's fix would also bring
back `hkdf`/`pbkdf2` files that now error on Boa (3 of 4 `hkdf` and 8 of 9
`pbkdf2` window variants) and cap Boa's P2 gains at 1,024 passes in those files.

**F15 (2026-10-20). P2's dependencies and what it leaves open.**
- `hkdf` 0.13.0 and `pbkdf2` 0.13.0, on `digest` 0.11 and `hmac` 0.13, both MIT OR
  Apache-2.0, both already in the registry cache. cargo-deny: licences pass; bans
  fail on exactly main's 28 duplicates and add none; advisories name neither crate.
- The registry refuses a KDF key that is extractable, so a script cannot export
  an HKDF or PBKDF2 base key even through a forged brand.
- **No cap on PBKDF2 iterations or on `deriveBits` length.** The Web Crypto text
  sets none. A call with 4,294,967,295 iterations blocks the script thread until
  it finishes, and a 2^32-bit PBKDF2 output tries to allocate 512 MiB, which the
  registry turns into an `OperationError` if the allocation fails. A policy cap
  would be a ruling; none is proposed here.
- AES-GCM's `iv` and the other AES parameters are not read yet. Importing an AES
  key does not make it usable: `encrypt` and the rest still reject with
  `NotSupportedError` until P3.

## Progress

- 2026-10-20: assessment, card round, plan. Worktree `Code/worktrees/genet-webcrypto`
  on `webcrypto-subtle` from main `16b971188f3`.
- **2026-10-20: P1 implemented on the branch, not merged.**
  - **What landed.**
    - `subtle.rs`: the native key registry, digest, HMAC, export, and the
      sinks.
    - `subtle/bootstrap.js`: WebIDL conversions, normalisation, `CryptoKey`,
      `SubtleCrypto`, JWK handling, promises.
    - `secure_context.rs`: potentially-trustworthy URLs, document and frame
      inheritance.
    - `Runtime::set_secure_context`, and URL derivation in `set_base_url`.
    - Workers inherit their owner's flag over threads and the process hello.
    - getRandomValues checks the intrinsic element type.
    - The `.https.` serving rule in genet-wpt.
    - sha2 0.10 → 0.11 across fleece, genet-scripted and genet-wpt, with
      explicit hex where `LowerHex` left.
    - Four deny skips (F8).
  - **Matched WebCryptoAPI maps.** Same box, frozen before runner
    `c6f85bd7…`, after runner `b60c8eb0…`, identical manifest. Receipt:
    `Code/testing/genet/webcrypto/p1-20261020/attribution.md`.

    | Engine | Subtests passing | Files all-pass | Lost | Gained |
    |---|---|---|---|---|
    | Boa | 81/6,947 → 5,472/14,863 | 3 → 6 | **0** | 5,394 |
    | Vano | 80/6,947 → 7,980/35,645 | 3 → 6 | **0** | 7,903 |

    Every other movement is a non-pass becoming another non-pass. Those
    belong to later phases' tests now registering, or to Boa's F11
    harness-timeout files.
  - **The first candidate lost `historical.any.html`** (the file and three
    subtests on each engine). That went to Mark as a fork, and the secure-context
    gate is his ruling.
  - **Crate gates, `--no-fail-fast`, debug info off.**

    | Crate | Passed | Failed | Ignored |
    |---|---|---|---|
    | fleece | 65 | 0 | 0 |
    | genet-scripted | 133 | 0 | 0 |
    | genet-wpt | 87 | 0 | 2 |
    | script-runtime-api | 1,156 | 10 | 0 |

    The script-runtime-api total includes 16 new `subtle_crypto` tests, 8
    `subtle`/`secure_context` unit tests, and the HMAC RFC vectors.

    The 10 failures reproduce identically against main's own script-runtime-api
    sources in this worktree, so they are pre-existing:
    - `cross_arena_replacement::atomic_refusal` ×2 and
      `frame_arena_boundary::associated_subtree_boundary` ×2: a canvas
      `getContext('webgl')` mints no context without a WebGL handler, so the
      transfer is admitted.
    - `forms_submission` implicit-submission ×6.

    `dom::tests::opaque_root_policy_cost_is_bounded_on_boa` failed once under
    concurrent load and passes alone on both main and the candidate.
  - **Other gates.** `cargo check -p ortet --all-targets` and
    `cargo check --target wasm32-unknown-unknown -p script-runtime-api` both
    pass, the latter with no new warnings. cargo-deny is covered in F8.
  - **Follow-ups outside this lane:**
    - Encoding's Vano sink leak (F7).
    - rustls 0.23.46 (F8).
    - WebIDL `QuotaExceededError` (F9).
    - Boa's harness-timeout stack overflow (F11).
    - Structured clone of `CryptoKey` (Phases).
- **2026-10-20: P2 implemented on the branch, not merged.**
  - **What landed.**
    - `subtle.rs`: HKDF (RFC 5869) and PBKDF2-HMAC (RFC 8018) over `hkdf` and
      `pbkdf2` 0.13.0 for SHA-1/256/384/512; HKDF, PBKDF2 and AES entries in the
      key registry; sinks `__subtle_secret_store`, `__subtle_derive` and
      `__subtle_derive_key`. `deriveKey` derives and stores in one native step,
      so the derived bytes never reach script. The registry refuses an
      extractable KDF key, checks the KDF named, the usage, the AES length
      (16, 24 or 32 bytes) and turns a failed allocation into a refusal.
    - `subtle/bootstrap.js`: `HkdfParams`, `Pbkdf2Params` and
      `AesDerivedKeyParams`; "get key length"; `deriveBits`; `deriveKey`; HKDF,
      PBKDF2 and AES-CBC/CTR/GCM/KW registered for `importKey`; raw and
      `raw-secret` import for all three families; AES raw export. A key derived
      from another realm's base key keeps that realm's sinks and finalizer.
    - Tests: `tests/subtle_kdf.rs` (8, both engines: RFC 5869/6070 vectors
      through `deriveBits`, every `deriveKey` target against `deriveBits`, 41
      error-name rules, native-only derived keys), 5 new `subtle.rs` unit tests,
      and one P1 probe changed from AES-GCM, which now imports, to RSA-OAEP.
  - **Pre-cargo check.** The bootstrap run under Node with Node's HKDF/PBKDF2
    as the sinks: `hkdf` 2,845 and `pbkdf2` 6,796 of 3,661 and 8,632, failing
    exactly the 2,652 `wrong (ECDH) key` rows (F13). The engines agree.
  - **Matched WebCryptoAPI maps.** Same box, frozen before runner `2da14d1f…`
    built from the clean P1 tip, after runner `444b5eaa…`, identical manifest.
    The before runner reproduces P1's published counts exactly. Receipt:
    `Code/testing/genet/webcrypto/p2-20261020/attribution.md`.

    | Engine | Subtests passing | Files all-pass | Lost | Gained |
    |---|---|---|---|---|
    | Vano | 7,980/35,645 → 17,698/35,662 | 6 → 7 | **0** | 9,719 |
    | Boa | 5,472/14,863 → 6,909/15,529 | 6 → 7 | **2**, waived (F14) | 1,440 |

    On Vano, `hkdf` and `pbkdf2` pass 2,848 and 6,804 window subtests and fail
    only the 2,652 that need an EC key. Every other movement is a non-pass
    becoming a non-pass: AES tests that used to fail at import now fail at
    `encrypt` under their own names (on Vano 1,263 identities retire and 1,280
    appear, the extra 17 being `wrapKey_unwrapKey` tests that now register).
  - **Gates, debug info off, `--no-fail-fast`.** `script-runtime-api` passes
    1,166 and fails 13. Ten are P1's pre-existing failures (`atomic_refusal` ×2,
    `associated_subtree_boundary` ×2, forms implicit submission ×6). Three timed
    out under a machine at 100% CPU and pass alone: `worker_boa`,
    `worker_error_on_boa` and `opaque_root_policy_cost_is_bounded_on_boa`.
    `cargo check --target wasm32-unknown-unknown -p script-runtime-api` passes with
    no new warnings, and `hkdf` and `pbkdf2` add only the digest-0.11 crates
    already present. fleece, genet-scripted and genet-wpt are not re-gated: no
    input of theirs moved. cargo-deny: F15.
  - **Rulings taken to Mark:** the AES scope (third round) and the two Boa passes
    (fourth round), both recorded under Rulings.
  - **Follow-ups outside this lane:**
    - The F11 fix (F14) recovers the two Boa passes and the KDF files that
      error on Boa; re-run the matched maps once it lands.
    - The runner's worker scope (F12).
    - A policy cap on PBKDF2 iterations and output length, if Mark wants one (F15).
