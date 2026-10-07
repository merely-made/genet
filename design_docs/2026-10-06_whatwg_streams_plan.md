# WHATWG Streams

**Status:** Candidate implemented; native fixes published, consumer qualification in progress, 2026-10-07.
S1's published engine pins and exact Genet consumer are qualified. Both engines
pass real buffer transfer; meaningful Streams algorithm controls still fail on
the unchanged foundation. The candidate passes those controls and all existing
focused Fetch, Encoding and Worker tests. A new Boa Promise-property trap and
plain-object finalizer delivery control fail. Independent WeakRef controls
confirm collection on both engines. Three bounded Luna agents drafted the
modules; root owns integration, review and gates. Mark approved S2 on 2026-10-06.
Mark approved S3/S4 below on 2026-10-07. Their qualification and the final local
pre-publication checkpoint remain open.

Authority is the [standards ledger](2026-09-07_standards_to_features_ledger.md#conformance-targets-2026-10-02)
rulings C1 and C3, the [census panic attribution](2026-09-06_web_platform_wpt_census.md),
and `Code/work/briefs/2026-10-02_genet_whatwg_streams.md`.
The [Streams Standard](https://streams.spec.whatwg.org/) and
[Fetch body algorithms](https://fetch.spec.whatwg.org/#body-mixin) govern the
implementation. Engine prerequisites follow
[ECMAScript buffer transfer](https://tc39.es/ecma262/multipage/structured-data.html#sec-arraybuffer.prototype.transfer).
The user has lifted the brief's offline/no-network requirement; new dependency
packages, version changes and engine revisions still require their stated
checkpoint. The previously authorized GPT substitutions apply, favoring Luna
for bounded delegated work. Copying the Streams reference implementation has
no ruling and remains a checkpoint.

## Scope and ownership

The ruled scope includes default and byte/BYOB readable streams, their readers
and controllers, writable and transform streams, backpressure, cancellation,
tee, async iteration, `ReadableStream.from`, piping with AbortSignal, queuing
strategies, and Window/Worker installation. Request and Response bodies consume
the Streams implementation. Transferable streams, Encoding streams, and new
incremental network-delivery work remain follow-ons. The Encoding block is
outside this lane's edits.

The implementation uses a separate `streams.rs` bootstrap installed immediately
before Fetch by `install_host_surface`. Agent-shared weak brands and captured
per-realm operations replace direct public queue, reader and disturbance fields.
S2 authorizes reader-based collection at the existing whole-body upload seam;
native incremental response delivery and its current live-clone rejection remain
preserved boundaries. A new transport or further native body behavior change
retains its checkpoint.

## Phases and done-conditions

1. **Foundation and baseline.** Resolve the engine checkpoint below. Preserve
   the pre-change optimized runner and establish full directory baselines on
   both engines for `streams`, `fetch/api/basic`, `fetch/api/response`,
   `fetch/api/request`, and `fetch/api/body` where present, with the two C3
   records explicit. Record exact source, lock, manifest, corpus and executable
   hashes. Use Livery, default collection, jobs four, outer timeout 120 seconds
   and drive deadline 15 seconds on matched before/after runs. E3's explicit
   completion reporting is retained. Qualify an engine repin separately if ruled.
2. **Streams algorithms and fetch seam.** Implement private slots and abstract
   operations in the owned module, with bootstrap order and realm coverage
   verified. Stop if engine capabilities or fetch body changes exceed the
   approved scope. Unit controls cover backpressure, cancellation, pipeTo error
   propagation and BYOB fills, with actual failing runs on the starting source.
3. **Acceptance.** Both C3 records cease throwing on evaluation; record their
   complete new statuses. Streams excluding transferable improves on both
   engines, with zero lost passing files or assertions across Streams and fetch
   directories and every movement attributed. All Runtime, Scripted and WPT
   tests pass, including fetch_binding and both engine fixtures. Count tests by
   exact executable source ownership. Commit locally, report receipts and
   forks, and stop before push or merge at the brief's final checkpoint.

## Findings, 2026-10-06

- Per-realm `install_host_surface` serves Window, Worker and child frame realms;
  it installs DOM before fetch (`script-runtime-api/lib.rs`, `frames.rs`). The
  separate-module install order is feasible without changing realm setup.
- Vano pin `47f8d4f9d6fca884e3472e405872ce9556ff9db0` implements internal
  detachment, but `ArrayBuffer.prototype.transfer` and `transferToFixedLength`
  return implementation TODOs
  (`Code/crates/vano/nova_vm/src/ecmascript/builtins/structured_data/array_buffer_objects/array_buffer_prototype.rs:369-391`).
  Its public Rust buffer API exposes fixed-buffer construction and detachment;
  resizable allocation is internal. A private Genet operation therefore also
  needs a complete resizable-buffer contract, not only fixed-buffer copying.
- Boa pin `52cfb6ff9efdaf0ff6d213ab107bbaefe5838ab0` contains transfer code but
  gates the transfer family behind `experimental`
  (`Code/crates/boa/core/engine/src/builtins/array_buffer/mod.rs:349-405, 754`). Genet leaves
  that feature off. It also enables unrelated APIs, including Array.fromAsync,
  Atomics.pause and experimental Promise methods; no transfer-only feature
  exists at this pin. Source presence did not establish compiled availability.
- Both frozen-runner prerequisite probes pass async-generator/for-await and
  Promise-order controls. The real transfer/detachment control fails: Boa says
  the method is not callable; Vano says transfer is not implemented. Each probe
  reports two passes out of three. These are prerequisite controls, not a
  directory conformance measurement.
- Structured clone explicitly documents its backend detachment gap and has a
  copy/marker fallback (`structured_clone.rs:19-22, 261-275`). That fallback
  does not detach the caller's actual backing buffer, so it cannot qualify BYOB.
- The brief's whole-body description is incomplete: the current Runtime also
  has host incremental response hooks (`FetchPull`, `start_stream`, `push_chunk`,
  `close_stream`). Existing `fetch_binding.rs` tests qualify that behavior and
  reject cloning a live body. Do not remove these hooks or claim that this lane
  adds native incremental delivery. A body-seam change requires its own evidence
  and checkpoint if it exceeds the stated scope.

### S1: engine-capability checkpoint, ruled

The brief requires: **"Checkpoint: stop and report if ... an engine feature
Streams needs ... is missing in one engine. That is a Vano or Boa question."**
The missing transfer capability is now reproduced on both engines.

The question put to Mark is: "Streams' byte readers need real buffer transfer.
The probe fails on both engines: Vano's method is unimplemented; Boa hides it
behind a broad experimental feature. Which route should I take?"

1. Fix Vano's standard transfer methods and add a transfer-only Boa feature,
   then qualify both and repin Genet (Recommended).
2. Add a private transfer operation to Genet's engine adapters, including
   resizable-buffer support.
3. Defer byte/BYOB readers and narrow Streams to a first phase.

At the original checkpoint no answer was recorded. Option one changes two engine forks and the exact
Git revisions while keeping unrelated experimental features disabled. Option
two expands Genet's adapter contract and still needs genuine resizable-buffer
support. Option three narrows the currently ruled done-conditions; it cannot
be reported as completion of the full Streams brief. No option is selected
by inference from the user's earlier continuation.

**Ruling S1, 2026-10-06:** after the recommendation was restated as completing
Vano's methods and enabling only Boa's transfer family, Mark answered
**"Ok, agreed. Proceed"**. This selects option one. Vano starts from its clean,
already pinned main 47f8d4f9; Boa starts from its clean, already pinned
52cfb6ff. Boa's checkout was initially on its `genet` branch at that revision;
its existing `main` was safely fast-forwarded to the same revision while
preserving the lane edits. Qualify standard buffer transfer and actual baseline failures in each
owner, preserve unrelated experimental APIs as disabled, then publish the
required engine revisions and repin Genet without changing package versions.
The engine prerequisite probe and Genet consumer gates must qualify those exact
revisions before Streams implementation proceeds. This does not select the
private adapter alternative or narrow the BYOB done-conditions. Any required
fetch-body behavior change remains a separate checkpoint.

### S2: fetch-body checkpoint, ruled

The brief requires: **"Checkpoint: stop and report if ... fetch's body handling
must change beyond wrapping the whole-body delivery in a real stream."**
Source inspection found that Request accepts a ReadableStream with `duplex:
"half"`, but the native call sends only `req.__bytes`; a stream body has no
such bytes and reaches the current sink as an empty string (`fetch.rs:1809,
1993-1994`). This is source evidence, not a new transport measurement. WPT's
`fetch/api/basic/request-upload.h2.any.js` requires consuming valid byte chunks
with `duplex: "half"`; the upload probes also reject invalid chunk types.
This does not constitute a new HTTP/2 transport measurement. Replacing the bootstrap's direct `_chunks`
drain with standard readers also changes Promise, pull, error and disturbance
timing for Request and buffered Response bodies.

Mark has been asked to approve reader-based consumption: validate and collect
Request stream chunks before calling the existing whole-body upload sink,
use reader-based Response consumption, and preserve the existing incremental
response hooks. The alternative is to defer stream-backed uploads and retain
current body behavior, which narrows the brief's done-conditions. No answer
was recorded before the pause; dependent body changes had not started. S1
engine qualification and frozen baseline accounting continued independently.

**Ruling S2, 2026-10-06:** after the status report explicitly identified
reader-based body consumption as the remaining decision, Mark answered
**"ok. proceed, orchestrating"**. This authorizes the described reader-based
Request and Response consumption, validation/collection before the existing
whole-body upload handler, and resulting asynchronous timing changes. Preserve
the existing native incremental response hooks. It does not authorize new
incremental network delivery or copying a reference implementation.

## Progress, 2026-10-06

- Encoding is qualified and published on main `90c5ef507db`; the tested code
  merge is `f97f0df038c`, passing 1,100 tests and eight optimized reporting
  controls. The main publication receipt remains under the Encoding evidence.
- The optimized merged runner is the pre-Streams code runner, frozen as
  `Code/testing/genet/encoding-textdecoder/integration-genet-wpt.exe`, SHA-256
  `C0739B278E1FA6D422E9A658E7956587FA30763BB25538ABD2DB377CCE008EA8`.
  Its accepted lock and committed Git sources are fixed. The prerequisite
  probes used the already qualified d406 runner at identical engine pins;
  their receipt explicitly distinguishes this from a full Streams baseline.
- Fresh C3 subset baselines use the merged runner on both engines:

  | Record | Boa | Vano |
  |---|---|---|
  | fetch/api/basic/stream-safe-creation.any.html | ERROR, evaluation-threw | ERROR, evaluation-threw |
  | streams/readable-streams/patched-global.any.html | ERROR, evaluation-threw | ERROR, evaluation-threw |

  Each record reports zero observed subtests. Vano's logs retain the expected
  Object.prototype.type and type-getter trap messages. Boa's error formatter
  retains an opaque exception rather than the message string. Raw records and
  logs are preserved. No after measurement exists.
- Evidence is under `Code/testing/genet/streams`: prerequisite fixtures,
  per-engine receipts and four C3 maps/receipts. Root ran probes sequentially
  after the completed Encoding gates. Luna preflight was read-only.
- Reuse the existing isolated checkout `Code/worktrees/genet-encoding` on
  `conformance/streams` and its stable `C:/t/cargo-targets/genet-encoding`
  cache. This preserves primary's concurrently used ignored local patches,
  especially its netrender revision, while qualification uses committed Git
  sources. Reuse avoids another worktree or cache; the name reflects its
  previous owner. At this initial baseline no isolated Cargo home existed. The checkpoint plan is local
  only; Streams is not published or merged.
- S1 is answered. Two bounded Luna agents own Vano's transfer algorithm and
  Boa's transfer-only feature respectively. They initially prepare source and
  tests without compiling while root captures the frozen pre-change directory
  baselines. Root serializes qualification gates and owns Genet repins/docs;
  the read-only Streams preflight audits the private fetch seam separately.
- Full pre-change directory gates are complete on both engines. The manifest,
  runner and 1,924 relevant corpus files remain unchanged. All five selectors
  have identical file-ID sets across engines. `published-before-summary.json`
  is derived from the frozen maps by `account_maps.py`; missing subtest arrays
  contribute zero rows, and repeated names use occurrence ordinals.

  | Selector | Engine | Pass / fail / error / skip files | Completed subtest pass / total |
  |---|---|---|---|
  | streams, excluding transferable | Boa | 13 / 89 / 47 / 6 | 334 / 1,111 |
  | streams, excluding transferable | Vano | 12 / 90 / 47 / 6 | 333 / 1,111 |
  | fetch/api/basic | Both | 11 / 50 / 8 / 0 | 222 / 598 |
  | fetch/api/response | Both | 24 / 28 / 3 / 1 | 391 / 515 |
  | fetch/api/request | Boa | 14 / 44 / 1 / 0 | 603 / 952 |
  | fetch/api/request | Vano | 14 / 42 / 3 / 0 | 604 / 872 |
  | fetch/api/body | Both | 4 / 0 / 0 / 0 | 46 / 46 |

  Completed totals above exclude errored files. On both engines, the errored
  non-transferable Streams records contain 204 passing, 204 failing, 41
  timed-out and 746 not-run subtest rows; fetch basic contains 14 passing,
  6 failing, 31 timed-out and 60 not-run rows. Response has three timed-out
  rows; Request error records have no subtest arrays. These remain errors,
  rather than conformance credit. Transferable Streams retain their separate
  13-file baseline (seven fail, five error, one skip).
- A seven-case transfer fixture fails 0/7 on each published engine. It covers
  fixed-buffer copying, shrinking, growth/zero-fill, resizable preservation,
  fixed-length conversion, and ToIndex resize/detachment reentry. Its SHA-256
  is `D885D618025D09917D03CE74373A6BC1047F1874736A66A816F95D19844DBE8D`;
  the before receipts use the unchanged f97 merged runner.
- Independent engine review found an unsafe optional `shrink_to_fit` in the
  Boa transfer path exposed by S1: locked aligned-vec 0.6.4 does not handle a
  null realloc result there. The candidate removes that unobservable shrink,
  applies the existing host allocation cap, and reserves fallibly before
  taking source bytes. It may retain excess capacity; it adds no cap/default
  change. Feature-only, experimental-only and combined test modes must all
  actually execute positive controls. Vano uses its existing allocation helper;
  that helper's pre-existing abort-on-failed-realloc behavior remains an engine
  limitation, not evidence of recoverable out-of-memory handling.
- Vano qualification is complete: 70 unit, 12 integration and seven doctests
  passed, with 10 existing ignores. The exact pinned Test262 corpus has 24
  files per transfer method. Both full directories ran with `--gc` and exit
  zero after removing only 36 verified expected failures: 46 files pass, and
  the two `transferToImmutable` prerequisite cases retain their expected
  failures. Initial unexpected-pass logs and post-update confirmations remain
  frozen; global metrics were preserved. A filtered metrics mismatch is
  diagnostic and the runner explicitly suppresses its exit, so the initial
  nonzero run is attributed to the 36 unexpected passes.
- Vano commit `a4415da20864daf8ec0ec85b26aaec189f8d20ae`, **Implement standard
  ArrayBuffer transfer methods**, is published on its main as authorized by
  S1. `vano-transfer/publication.receipt.json` verifies the remote transition
  from 47f8d4f9 and links the native gate receipt. That receipt explicitly
  distinguishes formatting-only changes after native tests from exact byte
  build provenance; Genet's published-pin qualification is still pending.
- The four authored Streams algorithm controls fail on both frozen published
  engines (0/4): writable desiredSize/backpressure, deferred cancellation,
  source cancellation after pipeTo sink error, and actual BYOB fills. Boa's
  BYOB negative initially reaches its missing transfer prerequisite; Vano's
  reaches the missing byte-controller request. The unchanged fixture must
  also run on the engine foundation before implementation, so BYOB's failure
  is distinguished from the resolved engine prerequisite. Fixture SHA-256:
  `B577F788EE18FA99156699FC30197266613FC8036C100B455A0745F95678674F`.
- Boa owns the next serialized gate. Its current maintained `origin/genet`
  is the accepted 52cfb6ff baseline. Its newer `origin/main` diverges and
  contains unrelated dependency changes; publish this qualified patch by
  fast-forwarding the maintained Genet ref without absorbing that drift.
  Source work remains on local main. Consumer receipts and the Genet repin
  remained pending at this point. S2 was subsequently answered as recorded above.
- Boa's native qualification passed: no-feature absence 1/1, transfer-only
  6/6, experimental-only 5/5, combined 5/5, and default features plus the
  narrow feature's owning ArrayBuffer module 19/19. The old allocation
  algorithm actually fails the host-cap control (0/1, exit 101); the candidate
  passes after a real rebuild. A restored source timestamp initially reused
  the control binary; that stale run is preserved and explicitly excluded.
  Root verified source, lock, patch and successful-binary hashes in
  `boa-transfer/native-qualification.receipt.json`, then committed
  `f9003484f6e83fb3d02024b82cd21a796fe8fb8d`, **Add narrow standard
  ArrayBuffer transfer feature**.
- The first Boa publication attempt ran its required pre-push hook with the
  shell's inherited shared `C:/t/graphshell-target`. Root stopped only that
  push's owned descendants before any remote movement; source and lock stayed
  unchanged. Required workspace lint checks passed with Rust
  1.97.1, locked dependencies, warnings denied and the approved reusable
  `C:/t/cargo-targets/boa` target. Existing foreign cache ownership is
  preserved. The Genet lock verifier permits exactly nine Boa and three Vano
  Git-source substitutions and rejects any other parsed lock change.
- Boa `f9003484` is published by fast-forwarding `origin/genet`; the normal
  pre-push hook also passed with an explicit reusable target and qualified
  toolchain. `boa-transfer/publication.receipt.json` seals the remote transition
  and native/lint receipts. The local Genet repin enables only Boa's
  `array-buffer-transfer` feature and updates both native and wasm64 Vano
  revision entries. No package version or dependency edge changed. Accepted
  lock SHA-256 `9B7C50EA...423D9` becomes `D73B7ADB...FAE89`; full hashes and
  all twelve changed source rows are in `engine-foundation-lock.receipt.json`.
  Locked metadata verifies all twelve packages resolve from Cargo's published
  Git checkouts; Boa's resolved features include `array-buffer-transfer` and
  exclude `experimental`. Metadata SHA-256 is `2BAE04D4...5F2840`; full source
  paths/features are sealed in `engine-foundation-metadata.receipt.json`.
  Consumer tests and the frozen optimized runner must still qualify the repin
  before the foundation is accepted.
- Mark paused the gate after compilation had finished and Runtime tests had
  begun. The `engine-foundation-crates` receipt has exit -1 and is explicitly
  unqualified; its output remains intact. Mark then resumed orchestration and
  answered S2 as recorded above. The prepared fetch-binding change shares all
  sixteen existing bodies across Boa and target-64 Vano while retaining every
  existing Boa test name. Reverse normalization verifies that assertions and
  script strings are unchanged; the source comparison receipt is sealed under
  `Code/testing/genet/streams`. Resume with a fresh named gate and preserve the
  interrupted one.
- The resumed online and cache-only Cargo attempts both stopped before
  compilation while waiting for the shared package-cache lock; each retains
  its exit -1 receipt and unchanged 914d790 source. Root interrupted only its
  verified Cargo pair, preserving other projects. A marker-owned temporary
  Cargo home at `C:/t/cargo-homes/genet-streams` copies existing locked caches
  for this qualification gate, with no new dependencies or configuration.
  Remove it after the gate's receipts are recorded. The stable target remains
  `C:/t/cargo-targets/genet-encoding`.
- Read-only Luna design agrees on the existing private `__agentTimers` root:
  agent-shared WeakMaps support borrowed methods, frame retention and snapshot
  cloning without adding an engine API. Keep operation closures realm-local;
  Fetch supplies trusted signal operations during synchronous installation.
- Four meaningful Rust controls run on each engine: deferred-write
  backpressure, cancellation waiting for its source, pipe error cancellation
  and lock release, and actual BYOB detachment/fills. They are initially ignored
  only to permit the unchanged-production foundation suite, then explicitly
  executed as starting negative controls. The candidate must enable all eight
  and pass them; an ignored control is not acceptance evidence.

## Progress, 2026-10-07

- Exact published-Git qualification at `d4800b63c656440ba628f058a3e4f06f9cb6521f`
  passed **1,142 tests, zero failures**, with eight temporarily ignored starting
  controls and three existing WPT ignores. Counts use exact depfile source and
  package ownership: Runtime 747, Scripted 120, WPT 77, Vano 43, Boa 26,
  Documents 56, Render 41 and Taproot 32. The eight starting controls then
  actually executed and failed (zero passes, eight failures, exit 101).
- A parallel compilation attempt ran out of memory in Naga before tests. Its
  failed receipt is preserved. The successful serial gate reused the same
  stable target and marker-owned Cargo home without changing source or lock.
  Keep `C:/t/cargo-homes/genet-streams` through final qualification, then remove
  it after receipts are sealed and its live owners have exited.
- The ordinary optimized production runner was built without test profile
  overrides and frozen as `Code/testing/genet/streams/engine-foundation-genet-wpt.exe`,
  SHA-256 `BB4B79B96CCB98C6793FB36333414D3B64FF14C407C959F20880B130B39BC7AA`.
  Its lock is `D73B7ADB010BD299D58F006B606FB69C7972A3951CE88E827E1D5A7870CFAE89`.
  Both engines pass seven transfer assertions and all three prerequisites.
  The unchanged algorithm fixture remains zero out of four on both engines.
- All ten engine-only WPT directory runs finished with unchanged corpus,
  manifest and file IDs. There are zero lost passing files or assertion rows.
  Completed file and assertion totals match the published baseline table.
  Each engine gains two assertions inside still-errored byte-stream records:
  actual detachment now makes the detached respondWithNewView view reject.
  These gains remain error-file evidence rather than completed conformance
  credit. `engine-foundation-vs-published-before.json` preserves raw movements;
  the separate attribution receipt explains all four.
- Implementation installs one captured bootstrap after DOM and before Fetch.
  Agent-shared WeakMaps retain brands across same-agent realms and snapshots;
  operation tables are captured and deleted during synchronous installation.
  Structured clone is captured afterward, before author script. Public Web IDL
  strategy conversion remains separate from algorithm extraction so source,
  sink and transformer getters keep their specified order.
- S2 uses private reader callbacks for Request upload collection and Response
  consumption, including byte validation and abort before native start. Existing
  incremental response hooks retain their native ownership and pull handshake.
  Request(input) uses the specified private identity-transform proxy. A native
  body-presence argument distinguishes an empty upload from an absent body.
  Response's own type field uses captured DefineOwnProperty to survive C3's
  inherited accessor. The Encoding block remains outside these edits.
- All three implementation modules are drafted. Source review fixed writable
  close reservation suppressing the queue sentinel, kept shutdown actions pending
  until all settle, and captured ReadableStream.from controller methods. Root
  added both-engine controls for prototype traps, frozen/forged stream objects,
  abort reentry, transform pressure, exact/empty/invalid uploads, abort before
  native registration, live authored clone independence, retained child streams,
  collection and Worker installation, plus a Vano snapshot control. These are
  candidate source, not yet passing receipts. The original eight controls are now
  enabled; their starting negative receipt remains unchanged.
- The authored BYOB control's EOF source now saves its pending request, calls
  `close()`, then `respond(0)`, as the byte-stream algorithms require. Its
  assertions are unchanged. The original HTML fixture and receipts remain
  intact; the separate corrected fixture is
  `streams-algorithm-controls-byte-eof.html`, SHA-256
  `DCD51AEC9A5F39A9CDAB7D0410F8CEA2723FB34458D56C9286D9DC3903BFD6CF`.
  Matched runs on the unchanged foundation still fail all four controls on
  both engines. `algorithm-controls-byte-eof-correction.receipt.json` records
  the precise correction and both fixture hashes.
- The first frozen candidate gate, `candidate-focused-controls`, failed during
  Runtime compilation before any tests ran. Its two Streams installers accepted
  the full engine instead of Runtime's realm-aware `Surface`. Both signatures
  now follow the existing installers, preserving Window, Worker and child-realm
  installation. The failed gate and its source/lock receipt remain intact.
- `candidate-surface-controls` compiled successfully and executed the existing
  Fetch tests: 28 passes, four failures, split equally between engines. A default
  read mistook `Core.shift`'s returned chunk for an entry record and returned
  `undefined`; it now forwards the chunk directly. The unchanged assertions
  detected this production defect. A new named gate runs every focused target
  with `--no-fail-fast` so a failure cannot hide later controls.
- A source audit found an additional reaction-attachment gap: captured public
  `Promise.prototype.then` still performs species construction. Internal
  reactions can therefore inspect authored Promise constructor/species getters.
  The separate six-case `streams-promise-reaction-properties.html` fixture,
  SHA-256 `39F8734A4444178BDC8A20C2BF0F3D261FFDF739B5FB94A455A1D24714B901B2`,
  passes all six cases on each unchanged foundation engine. The candidate has
  an enabled Rust control for the same edge. A private reaction operation is
  under read-only engine inspection for the brief's engine-feature checkpoint;
  no engine or dependency change is authorized by this observation alone.
- The frozen `candidate-queue-controls` gate completed all five targets:
  **126 passes and two failures**. All 32 existing Fetch, 39 Encoding and 24
  Worker tests passed. Streams had 17 passes and one Boa Promise-constructor
  trap failure; Vano passed that probe. Bridge controls had 14 passes and one
  Boa collection failure, with none of its four discarded streams finalized.
  Vano collection, both-engine child retention and Worker installation, and
  Vano snapshot independence passed. A plain-object/WeakMap-cycle diagnostic
  will distinguish a source retention defect from engine ephemeron behavior.
- The first plain-object diagnostic also lost every finalizer, including the
  object without a WeakMap. Boa's `SimpleJobExecutor` moves pending cleanup
  futures into a local group and drops that group at quiescence. An earlier
  microtask pump can therefore discard a cleanup listener before GC. The
  collection controls now observe WeakRefs directly, independently of finalizer
  delivery; a separate plain-object control retains the finalizer failure.
  This is read-only attribution pending a fresh diagnostic gate, not an engine
  fix or a change to GC-policy semantics.
- `candidate-weak-collection-diagnostic` executed all 17 bridge controls:
  **16 passes, one failure**. Both engines collect discarded stream cycles;
  Boa also collects ordinary self-cycles, a WeakMap self-key cycle, and a cycle
  spanning two maps. All body, realm, Worker and snapshot controls pass. The
  sole failure is the isolated plain-object finalizer after an earlier pump.

## S3: private Promise reaction operation, ruled

The brief's engine-feature checkpoint applies: **"an engine feature Streams
needs ... is missing in one engine"**. The candidate's Boa control records an
unexpected Promise constructor getter during default ReadableStream setup.
The six-case fixture passes on the unchanged foundation on both engines.
The existing in-scope WPT only poisons the public `then` method, so this new
control preserves evidence for the additional observed standards gap.

Proposed bounded route: expose each maintained engine's existing
`PerformPromiseThen` operation without a result capability. Add a defaulted
`CallCx` hook and two adapter implementations, capture a temporary native bridge
before Streams installation, then delete that global before authors. Core's
reaction attachment uses it and does not construct a derived promise. No
caller consumes `Core.react`'s current return value. Public Promise behavior
remains unchanged. Vano's algorithm and handler type are crate-private;
Boa's `JsPromise::then` follows the public species path, while its underlying
perform operation is crate-private. Native engine controls and matched Genet
qualification precede consumer repinning. Package versions, features and
dependency edges remain fixed.

**Ruling S3, 2026-10-07:** Mark answered "Yes and yes" to the two separately
presented engine questions. S3 authorizes the private hooks, adapter integration
and consumer repins after native qualification. Root delegated the bounded Vano
and Boa changes and Genet integration to the existing Luna agents; root owns
review, serial native gates, publication and consumer qualification.

## S4: Boa cleanup listener retention, ruled

The isolated plain-object finalizer control fails only after a microtask pump
precedes GC. The existing Encoding finalizer control, which collects before
its first pump, passes. WeakRef observations confirm actual collection, so
the failed callback does not demonstrate stream retention.

Proposed bounded route: preserve pending FinalizationRegistry cleanup work
across Boa `SimpleJobExecutor` drains without keeping registries strongly alive
or changing GC policy. Verify notification before/after a pump, multiple
registries, repeated collections, and registry collection before a qualified
repin. The source inspection identifies local pending cleanup futures dropped
at executor quiescence; implementation details await the ruling and native
controls. The existing approved Vano finalizer queue remains the owner there.

**Ruling S4, 2026-10-07:** the same explicit answer authorizes Boa cleanup-work
retention across pumps with the approved GC policy preserved. Actual collection
remains checked separately from callback delivery. No further confirmation is
needed for this bounded fix and its qualification.

- S3's Boa native hook gate passed all three controls on `dd0b5af58766b4ffbed65ed7bdbaf295c65b3f09`: constructor/species bypass, empty fulfillment, and thrown-handler identity through the job error path. Vano's first hook gate on `fafc895415e021cda91c2a2774443fb14e8dc0d5` failed compilation because a returned error retained a short GC borrow. Local `e4b85dc83ae3e2775a9d400552c5d8447e336578` consumes the GC scope when rebinding that error; fresh native qualification remains pending.
- S4's first native registry gate passed six controls and failed the multi-registry case: one drain returned after its first completed cleanup future. Local `4b6d316d` continues polling already-ready cleanup futures before returning at quiescence. A fresh whole-engine gate includes all hook and registry controls. Queued cleanup for a registry created while another waiter is pending remains available on the next drain, with notification buffered and weak ownership retained.
- Genet captures and deletes the native reaction bridge during each realm's bootstrap. A configurable placeholder preserves the deletion contract across both adapters. Enabled private-brand controls assert that the bridge has no remaining global property. New decoder controls collect after an earlier microtask checkpoint on both engines, preserving the existing Encoding source and tests.
- The fresh Boa whole-engine library gate on `4b6d316d1f363f4f483982fecfc62e7532835b89` passed **1,112 tests**, including the three reaction and three registry controls. Its native formatting check also passed. Required publication hooks and exact published consumer qualification remain pending.
- A delegated source audit identified synchronous Promise conversion failures outside callback catches in readable pull and sync-iterator value conversion. Both now relay failures through their existing stream error and operation rejection paths. Byte tee also captured branch cancellation flags before a pending BYOB read; it now reads current flags when that read completes. Enabled controls cover default/byte pull conversion errors, iterator conversion errors, and cancellation of either tee branch, alongside the unchanged local WPT cases in `streams/readable-byte-streams/tee.any.js`.
- Mark's prompt preference, 2026-10-07: ask and wait for an explicit response, with no response timer or default inferred from elapsed time. Technical gate deadlines remain as specified by their receipts.
- S3/S4's native revisions are qualified and published: Vano `be68ac01dc5f4f4829476146c7a034552dfe9fab` passed **95 tests, zero failures, 10 existing ignores**, including all six reaction controls, and is on `origin/main`. Boa `4b6d316d1f363f4f483982fecfc62e7532835b89` passed **1,112 tests** and its required formatting, all-features/all-targets Clippy and no-default-features Clippy hooks under Rust 1.97.1; its maintained `origin/genet` advanced from the accepted foundation. Neither native lock changed. Fresh publication receipts verify exact clean source, native gate hashes, remote before/after and enabled hooks. Genet now requests only those qualified revisions; the new lock receipt verifies all other versions, checksums and edges against the accepted foundation. Published-source metadata, consumer tests and WPT remain pending.

- Published-source metadata verifies all 12 Boa/Vano packages resolve from the
  qualified Git revisions with the foundation's feature sets unchanged. The
  first consumer gate, `candidate-reactions-focused`, failed compilation before
  tests: the Vano adapter addressed its private `builtins` module. It now uses
  the existing public `ecmascript` re-export. The failed receipt is preserved;
  consumer qualification restarts under a fresh name.

All candidate gates and failed diagnostics remain under `Code/testing/genet/streams`.
The bounded agents supply source review; root runs gates serially. Retain the reused
`worktrees/genet-encoding` checkout on `conformance/streams`, stable target
`C:/t/cargo-targets/genet-encoding`, and marker-owned
`C:/t/cargo-homes/genet-streams` for the pending qualification. The candidate
has not had its full crate suite, optimized runner or post-change WPT qualified.
