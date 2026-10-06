# WHATWG Streams

**Status:** Assessment in progress, 2026-10-06. The required engine-capability
checkpoint is open: neither locked engine currently exposes working
ArrayBuffer transfer to the Streams bootstrap. No Streams implementation,
engine change, feature enablement or dependency repin has been made.

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

*Reading, not ruled:* use a separate `streams.rs` bootstrap installed immediately
before fetch by `install_host_surface`. Closure-owned private slots and captured
intrinsics keep author-patched globals and prototypes out of internal algorithms.
Fetch will need a trusted bootstrap bridge in place of its direct `_chunks`,
`_reader`, `_disturbed`, `_closed`, `_errored`, and `_error` field accesses.
Changes to body-delivery behavior beyond the brief's wrapping scope remain a
checkpoint; the existing native contracts must first be measured and preserved.

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

### S1: engine-capability checkpoint, pending

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

No answer is recorded yet. Option one changes two engine forks and the exact
Git revisions while keeping unrelated experimental features disabled. Option
two expands Genet's adapter contract and still needs genuine resizable-buffer
support. Option three narrows the currently ruled done-conditions; it cannot
be reported as completion of the full Streams brief. No option is selected
by inference from the user's earlier continuation.

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
  logs are preserved. No after measurement exists and no full directory
  baseline has run yet.
- Evidence is under `Code/testing/genet/streams`: prerequisite fixtures,
  per-engine receipts and four C3 maps/receipts. Root ran probes sequentially
  after the completed Encoding gates. Luna preflight was read-only.
- Reuse the existing isolated checkout `Code/worktrees/genet-encoding` on
  `conformance/streams` and its stable `C:/t/cargo-targets/genet-encoding`
  cache. This preserves primary's concurrently used ignored local patches,
  especially its netrender revision, while qualification uses committed Git
  sources. Reuse avoids another worktree or cache; the name reflects its
  previous owner. No isolated Cargo home exists. The checkpoint plan is local
  only; Streams is not published or merged.
