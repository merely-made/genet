# Web Cryptography: `crypto.subtle` in the scripted tier

**Status:** In progress, 2026-10-20. P1 (framework, digest, HMAC and Web Crypto's
secure-context gate) is implemented and measured on branch `webcrypto-subtle`.
It is **not merged**. Matched WebCryptoAPI maps lose zero passes on both engines:
Boa 81 → 5,472 and Vano 80 → 7,980 passing subtests. P2 (HKDF, PBKDF2) is next
by the ruled order. Mark's answers from two card rounds are recorded below.

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
- **P1's lane-level choices** (same round). Mark: **"Keep both
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
2. **P2: HKDF and PBKDF2.** 24,678. Import `raw`, `deriveBits`, and `deriveKey`
   into HMAC and AES keys. AES key *creation* through deriveKey needs only the
   AES import of P3's framework, which is brought forward if the after-map
   shows it gating PBKDF2 subtests.

   Done when the matched maps meet the same zero-loss bar.
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
