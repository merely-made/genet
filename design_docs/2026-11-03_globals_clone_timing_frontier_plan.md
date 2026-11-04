# Globals, structured clone and timing frontier

**Status: active, adopted 2026-11-03.** Successor to the [historical original](archive_docs/2026-11-03/2026-09-07_cheap_globals_plan.md). Review baseline `725e6c4fa30dfb1e7ea39a383ddac912374e27e7`; approved owner carry is included. Historical counts and receipts retain their recorded qualification. Adoption does not implement remaining work or decide open design questions. The nine-family owner check at `Code/testing/genet/closure-audit/cheap-globals-owner-check-2026-11-03.md` is a source comparison, not a test receipt.

## Established contract

Runtime provides performance/marks, queueMicrotask, structured clone, channels and crypto; native crypto uses the OS source and wasm requires a host source, failing closed when absent (`components/script-runtime-api/crypto.rs`). SubtleCrypto and CryptoKey exist in `subtle.rs` and `subtle/bootstrap.js`, with incomplete algorithm/conformance coverage owned by `2026-10-20_webcrypto_subtle_plan.md`. FileList is implemented and its clone branch and regression are in `structured_clone.rs`, `form_data/bootstrap.js`, and `tests/form_data_bindings.rs`. Worker and frame postMessage and cross-agent MessagePort have later implementations; do not repeat the original “absent” claims. All current test files are source evidence, not fresh passes.

## Phase G1: transfer and clone custody

`structured_clone.rs` tries a backend's `ArrayBuffer.transfer()` and otherwise copies and shadows length without releasing storage. Determine actual Boa/Vano capabilities, including existing typed views, resizable and out-of-bounds cases; coordinate with `2026-09-07_worker_plan.md` and deferred worker scoping. `messaging.rs` has a separate cross-agent serialization path that moves queued messages and constructs a receiver, while same-agent transfer returns the same port and later clears its flag. Done when the chosen scope has both-engine negative and positive tests for storage/view detachment, distinct sender/receiver identity and queued-message custody across same-agent, cross-realm and cross-agent cases, with matched messaging/clone WPT attribution. Do not claim the worker round-trip proves every custody mode.

## Phase G2: platform serialization and RegExp verification

Maintain a type-by-type matrix of interface availability versus serialization for FileList, ImageData, ImageBitmap, SharedArrayBuffer, CryptoKey and DOMException. FileList has a clone path; CryptoKey exists but clone is a WebCrypto-plan item; canvas/ImageBitmap belongs with the deferred canvas owner. Existing `structured_clone.rs` comment is stale about interface existence. For RegExp, reproduce slash-source escaping on each current engine and clone round-trip before deciding whether an issue exists; the old observation is historical, not a confirmed current defect. Done when the selected type's owning interface, clone and realm tests pass with failure controls and the remaining types keep named owners and evidence.

## Phase G3: timing

`timing.rs` provides only `mark`/`measure` entry types with a frozen SameObject array; `navigationStart` now uses time origin, while other legacy fields initialize to zero. Its conversion of every reserved navigation name throws `SyntaxError`; the historical residual says a zero-valued navigation-timing attribute should instead produce `InvalidAccessError`. Recheck that exact distinction, nonzero-name behavior and current WPT before changing it. Coordinate resource/navigation/paint/longtask/element producers with `2026-10-20_observers_and_timing_plan.md` and network/render hosts. Done for each selected entry family when producer data, identity/lifecycle and error controls pass on both engines, with matched timing results. The SameObject fix is not evidence that those entry producers exist.

## Phase G4: cross-agent reach

Worker and frame messaging are present; BroadcastChannel remains per-agent per `worker.rs`. Coordinate cross-agent BroadcastChannel with `2026-09-07_worker_plan.md` and `2026-09-07_deferred_web_platform_lanes_scoping.md`, and realm origin/lifecycle checks with [Realms and document lifetime](2026-11-03_realms_frontier_plan.md). Done when admitted peers, isolation, delivery order, teardown and clone failures have both-engine and host-specific controls; keep nested relay ordering and browser-hosted proof separate.

## Questions and limits

Which of G1-G4 is the next owner-approved slice, and what exact runtime/host scope is required? Preserve the wasm random-source contract as an implemented boundary, not a new defect. The approved phases above retain the open items; their historical source is archived whole. No WPT totals or test status here are fresh.

## Owner carry, 2026-11-03

The observers/timing plan now records the census lane's historical ResizeObserver case; G3 should reference that owner for exact-case reconciliation and must not assert a current ResizeObserver defect or completion from the old panic-repair receipt. The completed-lane carry in deferred web-platform scoping retains follow-on owner boundaries and distinguishes historical observations from current reproduced defects.

## Adoption record

Mark approved all eight reviewed cuts on 2026-11-03: "Yep". The original is archived whole; the reviewed technical scope, historical qualifications and open decisions are retained.
