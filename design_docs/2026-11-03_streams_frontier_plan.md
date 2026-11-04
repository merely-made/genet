# Streams and Fetch body frontier

**Status: active, adopted 2026-11-03.** Successor to the [historical original](archive_docs/2026-11-03/2026-10-06_whatwg_streams_plan.md). Review baseline `725e6c4fa30dfb1e7ea39a383ddac912374e27e7`; approved owner carry is included. Historical counts and receipts retain their recorded qualification. Adoption does not implement remaining work or decide open design questions.

## Established contract and rulings

S1 selected standard Vano transfer and narrowly enabled Boa transfer, separately qualified and pinned; S2 approved reader-based collection of Request and Response data at the existing whole-body upload seam, preserving native response hooks. S3/S4 qualified private Promise-reaction and cleanup-listener mechanisms. S5 approved exactly twelve identified experimental owning/transfer old-pass rows, not a general waiver. The original S1-S5 checkpoints were recorded complete, with 24 label equivalents and no unexplained losses in the qualified comparison. `components/script-runtime-api/streams.rs`, `streams/{core,piping,readable,writable}.js`, `fetch.rs`, `structured_clone.rs` and `tests/streams*.rs` are current entry points. Historical `e410a1e1dab8` is not an ancestor of this snapshot; source implementation `65baf9427e0` is. Do not relabel the old receipt as current qualification.

### Landed stream contracts

The ruled implemented scope includes default and byte/BYOB readable streams, readers and controllers, writable and transform streams, backpressure, cancellation, tee, async iteration, `ReadableStream.from`, AbortSignal piping, queuing strategies, and Window/Worker installation. Preserve agent-shared weak brands, captured per-realm operations, private queues/readers/disturbance, and `streams.rs` bootstrap installation before Fetch. BYOB depends on genuine buffer transfer/detachment and correct returned-view behavior; this established ArrayBuffer prerequisite is distinct from the deferred transfer of a ReadableStream, WritableStream or TransformStream object in phase S2.

For changes touching these seams, require focused Boa and Vano controls for cross-realm private brands, BYOB fill and EOF, returned views, tee and cancellation, reader locks, error propagation and failing controls; apply matched WPT accounting to the selected behavior. The original exact twelve S5 row identities and raw receipts remain unchanged; no wider waiver follows. These are preservation conditions, not a new test result.

## Phase S1: current conformance inventory

Freeze matching current runners for Boa and Vano and compare Streams and relevant Fetch request/response/body directories with explicit errors, skips, assertion identity and C3 rows, preserving reporting controls and source/lock/corpus provenance. Attribute all movements, including prior-pass losses, rather than treating historical counts as a fresh baseline. Select the highest-value bounded algorithm family from the actual failures; done for that slice requires negative controls, focused runtime tests and matched comparisons on both engines, including private brand, cancellation, backpressure and pipe behavior where affected.

## Phase S2: transfer and clone

`structured_clone.rs:133,414` currently rejects stream serialization. Specify transfer custody, sender/receiver and error semantics against the existing cross-agent clone/port boundary, then test current engine support including real ArrayBuffer/view detachment. Done when transferable readable/writable/transform streams have an explicitly selected scope, observable ownership and failure controls across both engines and relevant agents, no leaked locks, and WPT attribution. The old twelve-row S5 ruling does not authorize any broader loss.

## Phase S3: Encoding streams

`TextDecoderStream` / `TextEncoderStream` were outside the Encoding and original Streams slice. Work with the existing Encoding owner to define incremental state, flush, BOM, fatal errors and boundaries. Done when streaming split-input and cancellation/error controls pass on both engines, matched Encoding/Streams cases are attributed, and ordinary TextDecoder/TextEncoder behavior stays intact.

## Phase S4: native incremental network bodies

Existing reader-based collection is not incremental network delivery. Preserve native hook and live-clone rejection until a separately scoped transport/host decision. Done when the chosen producer-to-stream path proves chunk ordering, backpressure, abort, errors, lifecycle, body disturbance and clone behavior with an actual network host and bounded memory, as well as matched Fetch tests. A new transport or additional native body change retains its original explicit checkpoint. Across all phases, new dependencies, version changes, engine revisions and copying the Streams reference implementation retain their respective original checkpoints.

## Open questions

Current remaining conformance and disk-mode C3 Fetch failures need a fresh owner-attributed map. Which transferable scope and incremental host boundary should be implemented first? The original plan's earlier pending S1-S5 statements are superseded by its later recorded rulings and final publication; do not reopen them or claim full Streams conformance.

## Owner carry, 2026-11-03

The approved original-plan carry assigns the census lane's two historical stream-getter records to S1 for exact case identification, current-source reconciliation and refusal/exception controls. Neither old maps nor the approved loss accounting authorizes a baseline repin or broad Streams acceptance. The Encoding carry confirms TextDecoderStream and TextEncoderStream remain owned by S3.

## Adoption record

Mark approved all eight reviewed cuts on 2026-11-03: "Yep". The original is archived whole; the reviewed technical scope, historical qualifications and open decisions are retained.
