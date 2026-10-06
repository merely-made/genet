# WHATWG Encoding: TextDecoder and TextEncoder

**Status:** Verified locally, 2026-10-05; final integration checkpoint.
Encoding and the ruled E3 harness correction at d406ebe2ba1 pass 971 crate
tests, with three existing WPT tests ignored. Matched optimized runs over
all 1,313 Encoding records gain 5,054 assertions on Boa and 5,053 on Vano,
with zero existing passing identities lost on either engine. All movements
are attributed below and in external evidence. E1, C2, E2 and E3 are answered;
the accepted dependency edge, versions, lock and corpus remain fixed. The
known fatal UTF-8 standard/WPT mismatch remains an existing failure. Vano is
pinned to 47f8d4f9 at the d7f08fecdc7 baseline. Code is committed locally on
conformance/encoding; this lane is not published or integrated.

Authority: the [standards ledger](2026-09-07_standards_to_features_ledger.md#conformance-targets-2026-10-02)
rulings C1, C2 and C4; the execution brief
`Code/work/briefs/2026-10-02_genet_encoding_textdecoder.md`; the
[WHATWG Encoding Standard](https://encoding.spec.whatwg.org/#interface-textdecoder)
and vendored WPT. The existing workspace dependency is `encoding_rs` 0.8.35.
Its [Decoder API](https://docs.rs/encoding_rs/0.8.35/encoding_rs/struct.Decoder.html)
provides stateful native decoding. No source is copied from another engine.

## Scope and done-conditions

This lane owns the TextDecoder/TextEncoder API on Window and Worker:
label resolution and canonical names, unknown/replacement RangeError,
fatal TypeError, BOM behavior, stream state and flush/reset, accepted
ArrayBuffer and ArrayBufferView inputs, and UTF-8 encodeInto counts and
character boundaries. Supported SharedArrayBuffer-backed views are included.
`read` counts UTF-16 code units; `written` counts UTF-8 destination bytes.
Astral code points and lone surrogates require explicit regressions.

Only `encoding_rs = { workspace = true }` may be added to script-runtime-api.
That crate is already locked; C2 does not authorize version or graph changes
beyond the direct workspace edge. URL query and form encoding in a document's
legacy charset belong to form submission. Legacy-charset iframe parsing and
TextDecoderStream/TextEncoderStream remain outside this lane. Shared UTF-8
helpers also serve Blob, Response and XHR; avoid changing those callers here.

## Phases

1. **Resolve the native-state checkpoint.** The brief fences fetch.rs edits to
   the encoder/decoder block and requires a checkpoint for a missing host seam.
   Record Mark's choice before native registration, host storage or adapter
   changes. Preserve the census R1 invariant: policy bookkeeping runs no author
   code and never rematerializes custom elements.
2. **Freeze a controlled baseline.** After Vano repin acceptance, freeze the
   release/netfetch runner, lock, source and WPT manifest. Measure the entire
   encoding directory on Boa and Vano, jobs 4, timeout 120, drive deadline 15,
   Livery, isolated workers and collection on. Classify the API subset separately
   and record which historical 30-second kills finish at these settings.
3. **Control and implement.** Add both-engine owning-runtime fixtures, then
   record their failures on unchanged production. Cover labels with whitespace
   and case, unknown/replacement labels, malformed fatal input, both BOM modes,
   multibyte stream splits/flush, view offsets, astral and lone-surrogate output,
   and partial destinations. Add the ruled native lifetime slice and narrow API
   implementation. Assert flush/reset release and collection/teardown lifetime.
4. **Accept and report.** Run script-runtime-api, genet-scripted and genet-wpt
   gates on the supported engines. Rebuild and freeze the final release runner,
   repeat the same WPT scope/settings, and match individual subtests including
   missing entries. Require API improvement on both engines and zero previously
   passing file or subtest losses across the whole directory. Attribute every
   movement, keeping timing and missing-feature outcomes separate. Commit and
   report at the brief's final checkpoint before further integration.

## Findings

- 2026-10-05: fetch.rs:850-870 remains the UTF-8 stub. TextDecoder ignores
  label and options, and encodeInto reports byte counts for read and can write
  part of a character. The shared utf8Encode helper also needs lone-surrogate
  care, but changing it would alter out-of-scope callers. Prefer an encoder
  helper inside the owned block.
- 2026-10-05: install_host_surface installs the same fetch bootstrap for Window,
  iframe realms and workers; worker_main owns and drops a separate Runtime.
  Native set_function registrations are in install_fetch_surface outside the
  fenced block. HostState provides per-realm native state ownership, but no
  decoder registry or arbitrary-object death hook.
- 2026-10-05: ScriptEngine dead/minted inventories are DOM-reflector-specific.
  Reusing fake node identities for decoder handles would cross that ownership
  boundary. No general native-resource finalization API exists today.
- 2026-10-05: both relevant engines contain FinalizationRegistry. Boa drains its
  cleanup jobs through run_jobs. Vano emits cleanup jobs, but GenetHostHooks
  currently inherits the default that ignores them. Its generic, promise and
  timeout jobs already use one queue, and the cleanup hook accepts that same
  Job type. This is a narrow adapter delivery gap rather than a missing VM
  primitive. Source inspection: Vano `47f8d4f9`, execution/agent.rs:424-449;
  Genet script-engine-nova/lib.rs:105-116 and pump; Boa `52cfb6ff`, finalization
  registry builtin and the adapter pump. No source is translated or copied.

## E1 native lifetime ruling, 2026-10-05

Mark selected "Private finalizer and Vano queue (Recommended)". Add a per-realm native decoder-handle map, with native
creation/decode/release sinks registered by the existing fetch installer.
Inside the trusted bootstrap, capture FinalizationRegistry and register, keep
only each opaque integer as the held value, and call a captured native release
sink from its trusted cleanup callback. Never capture the decoder target in
the held value or cleanup closure. Add Vano's missing cleanup-job override to
its existing queue. Cleanup occurs later during pump; it does not run in the
opaque-root policy. Flush releases decoder state immediately and subsequent
calls recreate it; realm/worker/runtime teardown drops leftovers. Releases are
idempotent and ids must not be recycled into stale finalizer callbacks.

The unselected alternative, if later needed for direct after-GC release without a JS cleanup job, is to add a generic engine weak-native-resource inventory/drain hook parallel to DOM
reflectors, implement it in both adapters, and drain it after force_gc. This is
a larger API change and needs a wider regression surface.

Both choices cross the brief's block fence. E1 explicitly authorizes native
registration/per-realm storage and the narrow Vano cleanup-queue override.
The selected option must prove collection and cleanup delivery on both engines,
worker and iframe lifetime, harmless author intrinsic replacement, no strong
reference from native state back to its wrapper, and R1 remaining intact.

Other checkpoints remain: specification/library disagreement; lock changes
beyond C2; and an excluded test becoming a dependency of this lane. Network
access is allowed by Mark's later instruction, superseding the brief's offline
and no-download rule while retaining dependency limits.

## Progress

- 2026-10-05: two read-only agents mapped the API ownership and native lifetime
  seams. Root verified the registration boundary and Vano Job-hook signature.
  The required boundary/lifetime question is pending. Existing census/Vano
  integration work continues independently; Encoding implementation is paused
  at this checkpoint. No Encoding worktree or target has been created.

- 2026-10-05: Mark answered E1 with the private finalizer and Vano queue option.
  Primary now has foreign image-decoding source WIP, in addition to its local
  Cargo overrides. Encoding is isolated at the brief's approved worktree and
  branch to preserve that actual collision; repin source remains frozen in the
  existing census checkout. The accepted repin lock is copied into the new
  checkout rather than importing primary's differing override resolution.

- 2026-10-05: Encoding checkout completed at `d7f08fecdc7` on
  `conformance/encoding`, without primary local patches. Portable lock SHA-256
  `070C2ABCC1F2C326BE560D4CCA053D5F526E65F872EC8ADF541818EAAC460907`;
  it differs from the preceding census lock only in three Vano git sources,
  not package versions or dependency edges. The plan/index/ledger edits were
  moved from primary with exact-block removal, preserving image-feature WIP.
  Owning-runtime fixture preparation is delegated separately from JS/native
  design; all production remains unchanged until failing controls are recorded.

- 2026-10-05: repin acceptance passed all 836 affected tests and paired WPT
  guards; exact repin `d7f08fecdc7` and receipt `2312c800c0c` are published.
  Encoding release/netfetch baseline is frozen from d7f, with runner SHA-256
  `A5E4B784C6550B7A18E7EED5F6082A39662AE05F2F35278E85EFE4A564FFBFA0`.
  Both full 1,313-record encoding runs are executing, same manifest/settings,
  engines in parallel. The 24 owning-runtime controls are compiling against
  unchanged production. No native code or workspace edge has changed yet.
  The reproducible API selector is saved in external evidence; it yields
  67 historical records/12,587 reported subtests, not the brief's unrecovered
  101/13,111 grouping. Fresh counts will use that named selector, including
  supported any-global variants, rather than equating different groupings.
  Native IDs will use a checked crate-wide monotonic allocator: snapshots
  retain JS associations while replacing HostState, so a per-host counter
  would allow stale cleanup callbacks to release unrelated new entries.

- 2026-10-05: both immutable baseline runs completed. Whole directory: 1,313
  records each; Boa 5 passing files and 10,577/1,333,564 passing reported
  subtests, Vano 6 passing files and 10,901/1,333,612. Named API subset: 67
  records each; Boa 4 passing files and 10,576/15,749 subtests, Vano 5 passing
  files and 10,900/16,123. Counts retain timeout/not-run/missing distinctions;
  they are not percentages of implemented Encoding behavior. External maps,
  counts and exact provenance are retained under encoding-textdecoder.
- 2026-10-05: owning-runtime controls ran against unchanged d7f production:
  24 tests, 23 expected assertion failures and the Boa finalizer control pass.
  Vano's finalizer queue control fails as expected. Each API behavior group
  fails on both engines. The fixture, SHA and log are frozen externally.
- 2026-10-05: adding only C2's direct workspace edge makes locked metadata
  reject the old ignored lock (exit 101). Source lock is unchanged. The exact
  proposed patch adds encoding_rs to script-runtime-api's dependencies only;
  all 867 package identities, versions and sources remain unchanged. Mark's
  required lock checkpoint is pending; implementation remains paused.
- 2026-10-05: cross-realm brands can share private WeakMap/WeakSet state through
  the existing per-agent __agentTimers value, with each decoder retaining its
  captured creation-realm native decode callable. Metadata never stores its
  wrapper. This avoids new engine API and preserves borrowed methods/getters.
  Navigable retirement does not itself clear retained wrappers' native state;
  actual HostState/runtime destruction is the teardown backstop. Flush frees
  the active decoder and pending byte buffer, retaining only reusable metadata.

- 2026-10-05: expanded baseline fixtures qualified on unchanged production:
  30 tests, 27 expected failures and 3 passing regression controls. The added
  cross-realm tests expose the missing prototype getter and lost streaming
  state after iframe retirement; the two captured public intrinsic controls
  pass already and guard against regression. Fixture SHA and log are frozen
  externally. The C2 manifest edit was temporarily removed for this locked
  baseline run; the ignored lock remains unchanged pending Mark's answer.
- 2026-10-05: the recovered historical map has 816 error records, all outside
  the canonical API selector. Fresh Boa reports 800 FAIL and 16 no-results;
  fresh Vano reports 798 FAIL, 16 no-results and 2 errors for those records.
  That separates reporting/speed changes from implementation work. The brief's
  two killed API files are not recoverable from this map; they are not equated
  with the two fresh out-of-scope Vano errors. The named API matrices now have
  ordinary records for every query variant, with missing subtests explicit.
- 2026-10-05: comparison helper positive controls pass for unchanged records,
  pass loss, missing records and duplicate-name occurrence identity. Actual
  after-run attribution remains pending implementation and matched runs.

## C2 lock checkpoint, 2026-10-05

Mark approved the exact one-edge ignored-lock update with "Approved, continue".
The patch adds encoding_rs to script-runtime-api's dependency row only. All 867
packages, versions, source revisions, checksums and other dependency edges are
unchanged. Locked metadata now succeeds. Accepted lock SHA-256:
`9B7C50EA33A181C424E871129A0CFB071CC8B65EE5BCECCD42CCD656118423D9`.
The original/proposed/accepted locks, metadata and graph-delta receipt remain in
external evidence. Phase 2 is open under E1 and C2; remaining checkpoints persist.

- 2026-10-05: Phase 2 implementation is assembled. The new native module owns
  per-realm decoder metadata and streaming state; process-wide checked IDs
  prevent stale cleanup aliasing after HostState resets. The JS block captures
  intrinsics and native callables, shares weak receiver brands across agent
  realms, and registers ID-only finalizers. Vano cleanup jobs now enter its
  existing queue. Native state holds no JS wrappers, and the R1 GC-policy code
  is unchanged. No source is imported or translated from another engine.
- 2026-10-05: root review corrected draft astral UTF-8 conversion, primitive
  dictionary conversion, required-argument ordering, nonconstructible getters,
  decoder reset borrowing and fatal-read accounting before runtime acceptance.
  The first compile caught an undefined-value API-name error; it was fixed
  using the existing CallCx::undefined contract. The failed log and original
  source are retained. A second focused gate is compiling.
- 2026-10-05: independent read-only review found no actionable issues. An exact
  outer-bootstrap comparison confirms unchanged code outside the assigned
  Encoding block plus the authorized registration/helper-visibility edits.
  Shared Blob/Response/XHR UTF-8 helpers are byte-for-byte unchanged. This is
  review evidence, not a passing implementation or WPT receipt.

- 2026-10-05: the focused gate passes all 164 runtime unit tests, including
  existing census R1 controls, decoder collection/flush/teardown, and snapshot
  stale-handle protection. It passes 30/32 Encoding integration controls. Both
  Worker failures were traced to the fixture's relative loader key while
  Worker resolves an absolute URL. The loader key is corrected and an explicit
  script-error assertion added; the next gate is running. The original Worker
  before failures are unqualified as Encoding behavior controls, because the
  script did not load. Window behavior controls and full WPT baseline remain
  qualified. All failed runs remain in evidence.

- 2026-10-05: a bounded read-only check found no specification/library mismatch
  in the GB18030-2022 cases. encoding_rs 0.8.35 records 2022 support; the relevant
  two-byte tables and four-byte range mappings match all listed updated WPT
  examples inspected. This qualifies those examples only; full API acceptance
  still requires the actual matched WPT run.

- 2026-10-05: all 32 Encoding integration controls now pass, including actual
  Boa/Vano worker scripts with corrected absolute fixture routing. The prior
  164-unit run also passed. Root is committing the reviewed implementation
  before broad crate gates and frozen post-change WPT acceptance. This is a
  candidate commit, not the lane's final acceptance or publication.

- 2026-10-05: a further capture audit confirmed that private ordinary-array
  writes can call an inherited numeric setter, and captured slice still invokes
  Array species. The correction uses captured Object.create with null-prototype
  array-like byte containers, own numeric entries and own length. It stays in
  the owned Encoding block. The initial run named array-hook-control-before
  accidentally includes the applied fix, as its immutable source.patch shows;
  both engine regressions pass, but that run is positive evidence only. Root
  restored the exact 9f9d22 production source for a separately recorded negative
  control. Every original artifact is retained.
- 2026-10-05: root interrupted the runtime-crate gate during compilation after
  this defect was confirmed. Its receipt records exit -1 and no test acceptance;
  the broad gate will run against the final corrected source. No foreign
  compiler or task was interrupted.

- 2026-10-05: the qualified array-hook negative control fails on both engines
  against exact 9f9d22 production: encode('A') emits byte 0, two inherited setter
  calls occur, and Array species is accessed once and throws during decode.
  The identical corrected source previously passes both controls with byte 65,
  text A and zero hook calls. The fix is reapplied without additional changes.
- 2026-10-05: the post-change corpus runner uses the brief's default optimized
  release build. Broad crate tests will reuse release dependencies while setting
  only script-runtime-api, genet-scripted and genet-wpt test-package optimization
  to zero, with two build jobs and two test threads. This limits the cost of the
  45 runtime integration binaries; it is ordinary test evidence, not a timing
  comparison. The frozen WPT runner keeps the original release settings.

- 2026-10-05: final corrected production source is e23f2db1b14. Its default
  release/netfetch runner builds successfully and is frozen with SHA-256
  A045054AF9DEE8791A86152E3B9FE08E39BE587E9E47C529DD32D0096BF8298B.
  Corpus manifest and accepted C2 lock are unchanged. Full matched Encoding
  runs on Boa and Vano and the broad crate gates have started. This is not yet
  acceptance. Final outer-bootstrap comparison still matches the baseline
  after normalizing only the authorized registration/helper-visibility hunks.

## Candidate measurements and E2 checkpoint, 2026-10-05

The complete gate against e23f2db1b14 passes script-runtime-api 735,
genet-scripted 120, genet-wpt 72 and script-engine-nova 22 tests. Three existing
WPT tests are ignored. The frozen WPT executable uses the default optimized
release profile; the broad crate gate uses the recorded per-package test
optimization overrides. The accepted C2 lock and corpus manifest are unchanged.
Evidence is under `Code/testing/genet/encoding-textdecoder`, including
`final-crates.receipt.json`, `final-crates-counts.json`, both `after-*.json`
maps, their frozen-run receipts, and `compare-*.json` individual movements.

These are unaccepted first post-change measurements. Each full map contains
1,313 records; the controlled API selector contains 67 records. Historical
brief totals use a different selector and are not substituted for this one.

| Engine | Scope | Passing files before / candidate | Passing subtests before / candidate |
| --- | --- | --- | --- |
| Boa | API | 4 / 42 | 10,576 / 15,117 |
| Vano | API | 5 / 43 | 10,900 / 14,846 |
| Boa | Whole encoding | 5 / 43 | 10,577 / 15,118 |
| Vano | Whole encoding | 6 / 43 | 10,901 / 14,846 |

Boa's comparator reports 422 losses of previously passing file/subtest
identities: 415 subtests absent, two not-run, four timeout and one file failing.
Vano reports 1,282 such losses: 1,278 subtests absent, one fail, one timeout,
one not-run and one lost passing file. These are actual unaccepted movements;
missing names are not inferred to pass. There are also 16 Boa and 18 Vano new
error records outside the API selector. Their full attribution remains open.
The two full runs overlapped the broad build, and foreign compilers were also
active. Contention is a hypothesis rather than a demonstrated explanation.
Root started sequential frozen-before/frozen-after runs of
`textdecoder-fatal-single-byte.any.js`, with the same jobs, timeout, drive
deadline and corpus. They have no concurrent root gate; foreign compiler
owners at each start are recorded. These diagnostic pairs will be retained
separately rather than replacing the initial full maps.

**Confirmed library defect.** Local WPT `textdecoder-mistakes.any.js:614-624`
inserts an empty streaming chunk between a lead byte and its trail. A standalone
probe linked to the exact existing 0.8.35 rlib reproduces wrong output for
Big5 (`@` instead of U+9442), Shift_JIS (U+FFFD instead of U+221E) and EUC-KR
(`A` instead of U+AC02). Skipping only that empty non-final library call yields
all three expected results. Source inspection shows the two-byte decoder
prolog clears its lead before checking whether source input is available.
The probe source, output and SHA are `library-empty-stream-probe.rs` and its
log. This is independent of the native wrapper and confirms the brief's
specification/library checkpoint. Upstream [issue 126](https://github.com/hsivonen/encoding_rs/issues/126)
and [merged fix 128](https://github.com/hsivonen/encoding_rs/pull/128) concern
this defect; no upstream code was copied.

E2 asks Mark whether to add a narrow adapter guard: if both new input and
pending input are empty and `stream` is true, return an empty string without
calling the library, preserving existing decoder/BOM state. A final empty
call still flushes normally; nonempty pending bytes still run. Keep 0.8.35,
the accepted lock and dependency graph. The alternative is a separately
scoped library upgrade. Production has not been changed past this checkpoint.

**Fatal UTF-8 conflict, reading rather than a new ruling.** WPT lines 652-658
expect an empty successful flush after fatal streaming input `[FD, EF]` throws.
The [current TextDecoder algorithm](https://encoding.spec.whatwg.org/#dom-textdecoder-decode)
sets do-not-flush before processing and does not clear its I/O queue when a
fatal error throws. The UTF-8 handler rejects FD immediately, leaving EF
queued. The next flush must therefore reject incomplete EF. Both root and a
read-only agent checked the queue, decode and UTF-8 steps against the existing
library's consumed-byte accounting. Current pending-byte behavior is consistent
with that reading, including preserving a valid tail after `[FF, 41]`. Clearing
all remaining input to satisfy this assertion would drop specified queued
bytes. Preserve the frozen WPT and report the mismatch separately; it is not
attributed to a decoder implementation improvement.

The Vano `sharedarraybuffer.https.html` case is a concrete regression: its
previously passing file and `decoding SharedArrayBuffer` subtest now fail with
`TextDecoder input is not an ArrayBuffer or view`. This differs from the
existing Boa SAB-constructor limitations. Read-only source tracing confirms
Vano's `array_buffer_constructor.rs:127-141` omits its separate SharedTypedArray
variants from ArrayBuffer.isView, although its intrinsic typed-array tag getter
recognizes them. The Encoding block relies on the captured isView at
fetch.rs:936. A proposed correction classifies using the already captured
intrinsic typed-array tag getter and DataView buffer getter, then uses the
existing captured buffer/offset/length copy path. The separate encodeInto
Uint8Array validation needs the same shared-view regression coverage; C4
already includes SAB-backed views supported by the engine. No Vano source
change is required for the Encoding-owned workaround. Production edits are
held while the E2 library checkpoint is pending.

Other remaining API failures include existing Boa SAB support, ArrayBuffer
transfer, Worker helper-loading and IDL-fetch limitations. Those are not
assigned new implementation scope by this measurement. Candidate acceptance still requires
the narrow streaming correction after E2, qualified pass-loss repair, and a
new frozen final runner and matched full maps with every movement attributed.

- 2026-10-05: all four sequential fatal-single-byte diagnostic runs finish.
  The paired comparator still reports 54 lost passing subtests on Boa and 160
  on Vano, all in Worker variants with incomplete results. Window variants
  finish all 1,000 subtests (168 in the final range). Removing root's broad
  compilation/other-engine overlap reduces losses but does not eliminate
  them. Six to eleven foreign cargo/rustc processes were recorded at each
  start. The pair does not establish a contention-only cause or satisfy the
  zero-pass-loss condition. Production e23f2db1b14 remains unchanged while E2
  is pending; an independent read-only review is examining private native
  call overhead and the Worker reporting deadline.

## E2 ruling and correction pass, 2026-10-05

Mark answered "Ok" to the narrow empty-chunk adapter workaround while keeping
encoding_rs 0.8.35. Add the guard described above, preserving pending input,
existing stream state and final flush behavior. No library upgrade, new
dependency or lock update is authorized by this answer.

The same pass repairs shared-view detection through captured intrinsic brands
inside the owned Encoding block. Capture remains before author execution.
Reduce bridge allocation by using a private tagged-string result instead of
JSON and a one-byte input-copy fast path. Native names/arities, handle identity,
per-wrapper metadata, private finalizer delivery and teardown stay as ruled in
E1. This protocol is private plumbing selected during implementation, not a
new author-visible API or policy ruling. Preserve label/options coercion order,
RangeError/TypeError distinctions, NUL/BOM/astral output and trusted intrinsic
capture. Initial FinalizationRegistry allocation stays unchanged in this pass;
deferring it is not evidence-qualified as a Worker pass-loss repair.

Record new empty-chunk and shared-view fixture failures against e23 production
before applying the correction. Output/intrinsic-poisoning controls may already
pass the candidate and are regression controls, not claimed failing controls.
Keep all original full maps and quiet diagnostic maps. After focused checks,
freeze a separately named corrected runner, inspect the affected WPT files,
then require a full comparison with every old pass retained before acceptance.

- 2026-10-05: the new fixtures on unchanged e23 production give 36 passes and
  exactly three expected failures: empty non-final chunks on Boa and Vano, and
  Vano shared-view detection. The special-output and replaced-string-intrinsic
  controls pass already and are regression controls. The immutable source patch
  contains only fixtures and docs, preserving the candidate production source.
- 2026-10-05: the first corrected native gate passes 165 unit tests but fails
  one new fixture whose Big5 A440 expected character was mistakenly written as
  U+4E2D instead of the correct U+4E00. Cargo stops before integration tests.
  Root replaces that vector with the exact WPT FE40/U+9442 case and includes
  Shift_JIS 8187/U+221E and EUC-KR 8141/U+AC02. The failed log and source patch
  remain under e2-native-api-controls; they are not implementation acceptance.
- 2026-10-05: e2-corrected-controls passes 166 runtime unit tests and all 39
  Encoding integration controls. The empty guard retains native queued tails,
  normal final flush and split BOM behavior. Vano direct SAB, shared Uint8Array
  and DataView offsets, and shared encodeInto destinations now pass. Captured
  String helpers preserve NUL, punctuation, astral and BOM output even when
  author code replaces slice/indexOf/JSON.parse. Existing private-array hook,
  lifecycle, stale-handle, iframe-retirement and actual Worker controls pass.
  A read-only agent finds no actionable defect in the JS correction. The native
  module no longer needs fetch's JSON helper, so its authorized visibility
  expansion is reverted to the baseline private function. The corrected code
  is a candidate; WPT pass-loss and broad crate acceptance remain open.

## E3 harness ruling and matched remeasurement, 2026-10-05

The corrected default-profile runner is frozen from 22cc7e17b92 with SHA256
`9180CB07251526FB0CEF5ADE39064AD2203FFC0ABD2E7A71771462EE2B26AB13`.
Its accepted C2 lock and frozen corpus manifest are unchanged. The complete
corrected crate gate passes script-runtime-api 742, genet-scripted 120,
genet-wpt 72 and script-engine-nova 22 tests, total 956, with three existing WPT
ignores. `corrected-crates.receipt.json` and its counts preserve this completed
source gate before any E3 source edits.

The corrected pilot selects 25 distinct records per engine. Shared-buffer,
ISO-2022-JP and surrogate cases retain every old pass; mistakes cases improve
13 to 86 passing of 87 in both globals, leaving the documented fatal-tail
specification/WPT mismatch. Fatal single-byte Window shards all finish their
1,000 assertions (168 final shard). Worker shards are incomplete: the selected
comparison loses 574 old passing identities on Boa and 3,673 on Vano, all in
those Worker records. A lowercase `encodeinto` selector accidentally matched
no tests; its failed log and receipt remain, and the exact `encodeInto` selector
was run separately. `corrected-pilot-analysis.json` matches selected IDs and
duplicate-safe subtest names; it is diagnostic, not whole-directory acceptance.

Paired one-shard probes use both frozen source runners and the same corpus,
jobs 4, outer 120 seconds and drive 15/60, with no concurrent root gate. Boa
before reports 358/1,000 at 15 and 385/1,000 at 60; corrected reports 420/1,000
and 542/1,000. Vano finishes all 1,000 at either drive setting, improving
988 passing to 1,000. Boa's 60-second runs finish after about 30 seconds, so a
driver-deadline-only explanation is insufficient. Eleven to twenty foreign
compilers are recorded at probe starts; contention is not proven as the sole
cause. All raw `worker-deadline-*` maps and receipts are retained.

Root and independent read-only review find the parent harness timer seam:
`ports/genet-wpt/src/main.rs` synthesizes a Worker host without timeout metadata;
`harness.rs` evaluates testharness before parsing document metadata. Vendored
WPT caches a Window timeout of 10 seconds unless long metadata is visible.
WorkerTestEnvironment itself has no default timeout. The completion bridge in
`components/script-runtime-api/harness.rs` drops the callback's overall status,
and the port labels a nonempty collected array passing if all its assertions
passed. A parent TIMEOUT can therefore produce a false passing partial file.

Frozen-runner controls on both engines show long metadata failing an
eleven-second virtual timer, a timeout after one passing assertion reporting
false PASS, and explicit-done quiescence reporting no-results without a
completion signal. The retained uncaught-timer fixture also reports PASS, but
the driver swallows that timer exception without notifying WPT; it does not
qualify an overall harness ERROR. A separate invalid-return fixture uses WPT's
own ERROR path after a passing assertion. The fixtures and receipts are retained.

The initial E3 runtime control gate passed six tests and failed two because its
uncaught-timer fixture used an API that propagates the exception immediately.
The corrected fixture exercises WPT's invalid-return ERROR status without
changing runtime exception handling. `e3-native-controls.*` retains that failed
gate. `e3-observed-controls.*` passes ten checks: completion and reset, retained
assertions without completion, TIMEOUT and ERROR despite a passing assertion,
captured native sinks, and restoration of final harness array order without
duplicate reports. Both runtime and port source reviews are complete with no
actionable findings, including the incremental-reporting gap and its correction.
Broad crate and optimized runner gates remain open.

The first clean E3 broad gate, `e3-crates.*`, passed genet-scripted 120 and
genet-wpt 70 tests but failed three existing WPT harness guards. Two miniature
harness fixtures omitted the new result callback and overall status argument;
their fixtures now implement those public WPT callback contracts. The animation
smoke guard only pins non-panic reporting, but its match rejected the newly
explicit harness TIMEOUT. The frozen pre-E3 runner already reports the same
three timed-out subtests (`e3-animation-before-classification.*`); the guard now
accepts that exact known timeout outcome while preserving failure on thrown,
incomplete or other stopped outcomes. No animation behavior or expectations
were changed. The separate invalid-return controls qualify false PASS on both
pre-E3 engines (`harness-controls-pre-harness-*-error.*`).

`e3-fixture-crates.*` completes the first E3 broad gate from 5dd234d3143:
script-runtime-api 752, genet-scripted 120, genet-wpt 75 and script-engine-nova
22, total 969 passed, zero failed, three existing WPT ignores. Its default
release runner is frozen with SHA256
`312216537328A8CEF2D477B887F2EF3F26AED296BF1CF55317451EF348D6D2C3`.
The first optimized long-timeout control rejects this runner: the multiplier
workaround also scales authored `step_timeout(11000)` to 66000ms, beyond the
60000ms harness timeout. That failed receipt and executable are retained; the
fixture is unchanged.

The correction loads minimal real long-timeout META into the host DOM before
WPT initializes, selected by the actual wrapper's first timeout META. Authored HTML parsing clears the seed
children before scripts run; the XML route retains its actual preloaded DOM.
Authored timer delays keep WPT's default multiplier of one. The first metadata
gate, `e3-metadata-crates.*` from 111b9a9a150, passes the real Boa eleven-second
guard but fails the real Vano snapshot guard with `harness-incomplete` and zero
results (genet-scripted 120 passed; genet-wpt 74 passed, one failed, three
ignored; later crate gates did not run). The isolated worker already bypasses
this documented snapshot/host-state limitation, so the bounded remedy sends
long-timeout template calls through the same fresh-runtime path. Normal
snapshot behavior is unchanged and remains guarded by the miniature harness;
real normal/long deadline checks use the matched fresh path. Snapshot/realm
repair is outside E3. Two real-harness guards check the eleven-second timer,
absence of leaked seed metadata, and preservation of the normal deadline.
Fresh gates for this correction remain open. The matched corpus byte snapshot covers 952
Encoding/shared resource/interface/common files and will be verified at both
ends of each paired run.

Mark selected "Approve harness correction and matched 60-second runs
(Recommended)". E3 authorizes correct long-timeout selection before authored
scripts, separate overall completion/status reporting, explicit incomplete
outcomes retaining observed named subtests, and matching before/corrected
Encoding sources with exactly the same harness correction. The driver default
stays unchanged; the matched gate explicitly uses 60 seconds. Jobs 4, outer
120 seconds, Livery, collection, frozen corpus and dependency limits remain.
Freeze new runners and raw maps instead of replacing any previous artifact.
Zero old passing identities lost and complete movement attribution remain the
acceptance requirement. Commit locally only; the final push checkpoint remains.

## Matched E3 acceptance and final checkpoint, 2026-10-05

All four full-directory runs finish with exit zero and unchanged bytes for
the 952-file corpus snapshot. Each contains exactly the same 1,313 result
IDs. Root ran the gates sequentially, with Livery, collection enabled, jobs
4, a 120-second outer timeout and the ruled 60-second drive deadline. The
default deadline remains unchanged. The before source is the unchanged
Encoding baseline d7f08fecdc7 plus precisely the same E3 harness patch as
the corrected source. It is preserved locally as c36eb9800b8 under the
annotated tag `encoding-e3-baseline`. The candidate code source is
d406ebe2ba1. Final documentation does not change either tested code tree.

The fixed API selection contains 67 result IDs and 19,408 reported subtests
in every run:

| Engine | Stage | Passing files | Failed files | Error files | Passing assertions | Failed assertions |
|---|---|---:|---:|---:|---:|---:|
| Boa | Before | 4 | 54 | 9 | 14,156 | 5,252 |
| Boa | Corrected | 47 | 11 | 9 | 19,210 | 198 |
| Vano | Before | 4 | 54 | 9 | 14,156 | 5,252 |
| Vano | Corrected | 47 | 11 | 9 | 19,209 | 199 |

The whole-directory raw measurements remain separate from that API subset:

| Engine | Stage | Passing files | Failed files | Error files | Skipped files | Passing assertions | Failed assertions | Reported subtests |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| Boa | Before | 5 | 322 | 981 | 5 | 14,157 | 222,792 | 787,170 |
| Boa | Corrected | 48 | 279 | 981 | 5 | 19,211 | 217,738 | 787,170 |
| Vano | Before | 5 | 320 | 983 | 5 | 14,157 | 222,466 | 786,844 |
| Vano | Corrected | 48 | 278 | 982 | 5 | 19,210 | 217,571 | 787,002 |

Each whole map additionally retains 550,053 NOTRUN and 168 TIMEOUT subtests;
there are zero `no-results` file classifications. Error records remain
errors even when they retain passing assertions. This is acceptance of the
bounded Encoding change, not a claim that the whole directory passes.

**Every movement.** The duplicate-aware comparison matches result ID,
subtest name and duplicate occurrence. Boa has 58 changed records: 43 API
files fail-to-pass, seven API files remain failed while gaining assertions,
one GB18030 Worker remains a harness error while gaining 61 assertions, and
seven non-API ISO-2022-JP records remain errors with no assertions and change
only between `harness-timeout` and `drive-deadline`. Its sole assertion
movement is 5,054 fail-to-pass; every prior pass remains a pass.

Vano has 52 changed records: the same 51 API records, gaining 5,053 assertions,
plus `unsupported-labels.window.html`, which changes from a killed run with
no report to a failed file containing 158 failed assertions. Targeted runs
of that exact record on both frozen binaries return **identical** records:
the same 158 assertions fail because child Document charset metadata is
undefined. The source creates iframes using a Python server resource and
calls neither TextDecoder nor TextEncoder. The full baseline's timeout hid
existing document-metadata failures; the recovered report is not a new
Encoding dependency or a conformance improvement. Preserve the original
full counts rather than substituting targeted outcomes.

The single Boa/Vano API gain difference is `encodeInto.any.html`'s invalid
Float16Array destination backed by ArrayBuffer. Vano fails while creating
that unsupported constructor, before entering encodeInto; Boa reaches and
passes the new destination validation. All other API movement identities
agree. Per-record notes with source pointers are in
`e3-metadata-boa-attribution.md` and `e3-metadata-nova-attribution.md` under
`Code/testing/genet/encoding-textdecoder`. Raw maps and comparisons remain
untouched. No old passing file or assertion is lost on either engine.

**Fresh code gates.** `e3-qualified-metadata-crates.*` verifies exactly
d406ebe2ba1 with the accepted lock and both engine features:

| Crate | Passed | Failed | Ignored |
|---|---:|---:|---:|
| script-runtime-api | 752 | 0 | 0 |
| genet-scripted | 120 | 0 | 0 |
| genet-wpt | 77 | 0 | 3 |
| script-engine-nova | 22 | 0 | 0 |
| Total | 971 | 0 | 3 |

The suite includes 166 native runtime unit tests, 39 Encoding integration
controls and ten harness completion controls. Own test packages use the
recorded release opt-level override; dependencies stay optimized. Both
frozen production runners use the normal release configuration. All 16
production harness fixtures pass their expected classifications across
before/corrected and both engines: an unchanged eleven-second timer passes;
TIMEOUT, incomplete and ERROR cases retain their one observed passing
assertion and never become passing files. The matched fatal-single-byte
Worker probe reports all 1,000 assertions on both engines: 988 pass/12 fail
before, 1,000 pass corrected, with zero passing identities lost.

An unnecessary supplementary default-optimized build without scripted-nova
was stopped before tests. Its aborted receipt is preserved separately;
it is neither a code failure nor the qualified 971-test gate.

**Failing controls.** The original baseline is 1 pass/23 failures for the
first 24 controls and 3 pass/27 failures for the expanded 30-control gate.
The pre-fix array-hook probe is 0 pass/2 failures. Rolling back E2 produces
36 pass/3 failures in integration controls (both empty legacy stream cases
and Vano shared views), plus the native empty-stream test failure at
165 pass/1 failure. Restoring E2 passes all 166 native tests and 39 Encoding
controls. These receipts preserve actual failures rather than assuming
every new test failed on the baseline.

**Frozen inputs.** Before runner SHA-256:
`9013CEF3C9A03707CDD949D7E3CEA560902CD0119775A03079F0D408D310BB8F`.
Corrected runner SHA-256:
`81557C1FEB2816C1FADEF7C77ED03A27B20A661D3F7A34351C7CD74794B29027`.
Shared harness patch SHA-256:
`C4407E9E73F134D148DFEA64A5E8EA4EFA09E081451A58935721FBD1B7E0C51B`.
The manifest, corpus snapshot and original/accepted lock hashes are retained
in the frozen receipts and `e3-matched-final-audit.receipt.json`. That audit
checks source cleanliness, exact ID sets, serial gates, binary/source/lock
tuples, settings and zero lost passes. The C2 lock change remains only the
existing workspace dependency edge; package versions and sources are fixed.

**Forks and integration limit.** E1 chooses per-realm native storage and
captured private finalizer delivery through Vano's existing queue rather
than a new engine weak-resource hook. C2 accepts only the existing
encoding_rs workspace edge. E2 keeps 0.8.35 and chooses the empty nonfinal
adapter guard rather than a library upgrade. E3 chooses actual early
long-timeout metadata and separate completion reporting, with matching
60-second runs rather than accepting partial reports. Those rulings are
executed. The fatal UTF-8 queue/WPT conflict remains an existing failure;
dropping queued bytes or altering the frozen corpus is not part of this
acceptance. Encoding streams, network byte streaming, URL/form encoding,
document charset and engine capability gaps remain separate work.

The execution brief's final checkpoint says **"Do not push."** Report and
stop with local commits on `conformance/encoding`. Keep the worktree
`Code/worktrees/genet-encoding` and stable target
`C:/t/cargo-targets/genet-encoding` for the pending integration; no isolated
Cargo home exists. The external frozen binaries, receipts and baseline tag
remain reproducibility evidence. Current main includes separate image work;
integration must preserve that work and its ignored lock when carrying C2's
runtime edge forward, then qualify the merged source. Streams is next in
the approved queue after this integration checkpoint.
