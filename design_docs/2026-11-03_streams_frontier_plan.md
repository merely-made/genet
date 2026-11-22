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

The S3 implementation from `fed4552db20` shipped in `efef89ec882`. The
November 17 gates passed with Bole main `b9240e98` and Vano `298c22b7`.
They cover 43 Encoding, six stream-specific, four transform, two readable
and two native decoder-ownership checks. The lock and logs live in
`Code/testing/genet/priorities-20261117/`.

`TextDecoderStream` / `TextEncoderStream` were outside the Encoding and original Streams slice. Work with the existing Encoding owner to define incremental state, flush, BOM, fatal errors and boundaries. Done when streaming split-input and cancellation/error controls pass on both engines, matched Encoding/Streams cases are attributed, and ordinary TextDecoder/TextEncoder behavior stays intact.

The 2026-11-08 implementation uses the existing native decoder and private TransformStream operations. It carries split UTF-16 surrogates and decoder input across writes, releases decoder state on termination, and checks for pending reads before reapplying transform backpressure.

Locked, offline runtime controls passed on Boa and Vano with diagnostic Bole `ddf4e31b` and Vano `298c22b7`. They cover split input, BOM, fatal write and flush, cancellation, backpressure, Worker use, cross-realm brands and output byte ownership, and the existing Encoding integration cases. The fresh Bole `aa14721c` resolution blocks compilation because it lacks `SimpleJobExecutor::with_continue_on_error`.

A borrowed reader method creates the read result object in the method realm
on both engines. The endpoints and output bytes follow the constructor realm.
The generic read result realm needs separate Streams work.

On 2026-11-18, matched disk-mode WPT compared `fe42f351a24` and `efef89ec882`
with the same lock and corpus and a matching Worker process helper for each
runner. Each engine evaluated 25 files, including one skipped crashtest.

| Engine | Control assertions | Candidate assertions | Candidate files passing / failing / error |
|---|---:|---:|---:|
| Boa | 0/228 | 168/221 | 15 / 6 / 3 |
| Vano | 0/227 | 172/225 | 16 / 6 / 2 |

Remaining failures include worker helper functions unavailable to tests and
four borrowed-reader result-object realm assertions on each engine. Both
comparisons recorded zero lost passes. The Boa batch reached the deadline on
the detached-realm file, which accounts for four missing labels. An isolated
rerun passed all four assertions with the same settings. Three other missing
labels on Boa and two on Vano were duplicate baseline failure occurrences.
Exact assertion comparisons, excluded manifest variants and runner provenance
are in `Code/testing/genet/priorities-20261118/wpt/`.

## Phase S4: native incremental network bodies

Existing reader-based collection is not incremental network delivery. Preserve native hook and live-clone rejection until a separately scoped transport/host decision. Done when the chosen producer-to-stream path proves chunk ordering, backpressure, abort, errors, lifecycle, body disturbance and clone behavior with an actual network host and bounded memory, as well as matched Fetch tests. A new transport or additional native body change retains its original explicit checkpoint. Across all phases, new dependencies, version changes, engine revisions and copying the Streams reference implementation retain their respective original checkpoints.

Ortet now opts its native HTTP(S) document Fetch route into the existing
headers-first runtime hooks. Each reader pull permits one delivery of at most
16 KiB. The host can retain the remainder of one decoded transport chunk and
keeps its existing deadline, decoded-body cap, and concurrency policy.
Aborting the signal drops the host transfer, and bodyless responses settle
without an open stream.

On 2026-11-22, a native loopback HTTP fixture held the final byte until
script consumed the first. Both pre-fix controls at `109238e1a49` stalled.
Bole and Vano passed that fixture on the candidate based on `4394693ff84`.
Each engine also drained 49,153 bytes in order with no delivery while the
reader paused, at most 16 KiB per delivery, and one pull for EOF after five
chunk deliveries. Each rejected a pending body read with `AbortError`,
closed the server connection, and completed a bodyless follow-up request
with a one-request admission limit. The frozen binaries, locks, and logs
are in `Code/output/genet-incremental-fetch/`.

S4 remains active for live response cloning, broader error and lifecycle
coverage, and the matched Fetch WPT comparison.

## Open questions

Current remaining conformance and disk-mode C3 Fetch failures need a fresh owner-attributed map. Which transferable scope should be implemented first? The original plan's earlier pending S1-S5 statements are superseded by its later recorded rulings and final publication; do not reopen them or claim full Streams conformance.

## Owner carry, 2026-11-03

The approved original-plan carry assigns the census lane's two historical stream-getter records to S1 for exact case identification, current-source reconciliation and refusal/exception controls. Neither old maps nor the approved loss accounting authorizes a baseline repin or broad Streams acceptance. The Encoding carry confirms TextDecoderStream and TextEncoderStream remain owned by S3.

## Adoption record

Mark approved all eight reviewed cuts on 2026-11-03: "Yep". The original is archived whole; the reviewed technical scope, historical qualifications and open decisions are retained.
