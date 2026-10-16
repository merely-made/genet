# Architecture reduction implementation plan

**Status:** in progress, 2026-10-16. Source baseline `8fa73497952ecda37cf234c4a845354dab1af599`.

**Goal:** remove competing ownership and false success paths identified by the architecture audit, retaining actual semantic contracts and evidence.

**Spec:** `genet/design_docs/2026-10-16_architecture_removal_audit.md` in the primary checkout. The approved twelve-lane decomposition is recorded below. This is the implementation ledger entry point, not a claim that the complete audit is already implemented.

## Constraints

- Work in this isolated checkout; preserve both local Forms repairs and all existing control/supplier worktrees.
- Cheap workers receive bounded tasks and disjoint named files. Parent owns integration, commits, source identities and build admission. Workers never launch Cargo builds/tests, browsers, downloads or child agents without admission.
- At most three workers, reduced when memory pressure warrants. Source reads and lightweight Python/shell tests can overlap. One heavy job holds `/home/markik/Code/testing/machine-heavy.lock`; use one Cargo job, reduced debug information and a measured memory cap, leaving headroom for the desktop.
- Reuse existing local dependency/target artifacts when compatible; do not clone suppliers, refresh dependencies or rebuild all engines to qualify a narrow change.
- Test retained code after cuts; add regressions for actual changed behavior. Test earlier only when a specific uncertainty determines whether a cut is safe. Mechanical moves and hypothetical APIs do not need separate testing campaigns.
- No push or Checkpoint C acceptance is implied. Local review, tests, browser/hardware qualification and publication remain separately recorded.
- Every task receives an independent review. New architecture remains parent-owned; workers implement the settled seam rather than inventing frameworks.

## Lanes and dependencies

| Lane | Audit IDs | Outcome / dependency |
|---|---|---|
| 1 Trustworthy gates | H6-H8 | Setup/command failure cannot pass; current CI owns dependency witnesses. |
| 2 Dead code/tools | H1, H11, I14-I15, X1 | Delete verified unused surfaces, preserve assets and historical evidence. |
| 3 Live documents | I1-I2, H3-H4, R15 | Production input/lifecycle/scheduling owns semantics; test clock remains separate. |
| 4 Runtime boundaries | R4-R6, R12-R14 | Callable/error/module/serialization/registration ownership; coordinate shared runtime files. |
| 5 Execution limits | R7-R8 | Enforced bounds or explicit refusal; external boundary for hang tests. |
| 6 Queries | R1-R2, R11 | Shared selectors, single query traversal, faithful XPath adapter. |
| 7 CSS parsing | R3, L1 | One lexical/declaration authority; coordinates runtime bootstrap with lane 4. |
| 8 Resources | I3-I5, L3 | Retained identity/payloads and non-stealing journal observations; coordinates live document owner. |
| 9 Network | I6-I9, H5 | Canonical identity, metadata and enforced host policy. |
| 10 WebGL | W1, I11-I13, R9-R10 | Correct pair layout before compiler/bridge simplification. |
| 11 Layout/projections | I10, L2, L4-L13 | Owner geometry/facts; preserve backend and unsupported-feature contracts. |
| 12 Presentation | H2, H9-H10 | Checked custody and actual outcomes; platform proof for conditional cuts. |

Forms qualification follows its production dependencies; independent graphics/platform work does not block it. All lanes receive small tasks, not one unbounded worker assignment.

## First task packets

- [ ] **T1, lane 1 / H7:** migrate static-DOM dependency exclusion to existing CI dependency-cone owner, retire obsolete profile launchers and stale instructions. Write a synthetic resolved-graph witness for forbidden edges and command failure. Keep existing Fleece/Ortet witnesses. Parent reviews current static-tier identity before integration; the absent old `genet-static-html` package must not remain a gate target. Workers can run Python unittest only.
- [ ] **T2, lane 2 / H1:** remove `GlCapability` / `GlCapabilityHandle` no-op definitions and references from compositor source after Genet/Mere/Turnstone consumer search. Keep `RenderingContextCore` and real WgpuCapability. No replacement interface, new dependency or broad formatting. Parent runs admitted compositor check.
- [ ] **T3, lane 3 / H4:** preserve full typed SessionInputResult in Ortet browser dispatch and share its default-scroll decision with the native host. Exercise handled/cancelled/editable/ignored outcomes in a host-neutral regression; preserve capture/navigation effects. Parent owns the shared predicate location and test/build admission. Worker first traces and proposes the smallest seam; no runtime/DOM editing redesign in this patch.
- [ ] **T4, lane 3 / I1-I2/H3:** after T3, trace actual editable-state APIs and production pump contracts; parent specifies shared editing/lifecycle ownership before implementation. Do not create another form-specific editing shim.

## Progress

2026-10-16: approved lane execution started. Native worktree tool has no Git repository at the chat's `/home/markik/Code` root; Git fallback created `codex/genet-audit-reduction-20261016` without modifying primary checkout or controls. Initial memory: approximately 9.9 GiB available, no current pressure. Builds not yet admitted.

## Resumed final cuts and tests

Pending Test262, dormant break projection, shader reflection, shared invalidation,
and compositor alias cuts finished. Final Cargo check passed for genet-wpt and
servo-webgl-wgpu. Livery library: 334 passed, 2 existing ignores; shader tests:
19 passed; Test262 standalone policy: 9 passed, 1 existing ignore; private
completion Node regression and six dependency-gate tests passed. See
[exact source hashes and results](receipts/2026-10-16_architecture_reduction_tests.json).
These tests do not close W1 shader linking, browser input, or Forms acceptance.

The user requested a fresh source audit after testing. Work first stopped at
the agreed 41% cutoff; the user then explicitly instructed proceeding. The
post-cut re-audit is now recorded at the start of the architecture audit,
including newly reproduced W2 and explicit partial/open dispositions.
Do not present this bounded reduction as completion of the twelve lanes.

## W1/W2 linked shader layout repair

**Status:** implemented and verified locally, 2026-10-16. Baseline `819dc26ffc6`.

The approved repair gives the ESSL frontend one linked-program contract: shared
varying locations and uniform member order/types for both stages, followed by
reflection of the emitted WGSL layout. The wgpu consumer copies those facts
into its upload descriptors instead of calculating another layout. Preserve the
single-stage compile entry point and the guarded Naga boundary.

Done conditions: reordered varyings and shared/disjoint uniforms link coherently;
conflicting uniform types fail linking; the scalar after a mat2 uses the emitted
offset. Existing frontend/backend tests and added pixel readbacks must pass on
the retained implementation. Implementation workers do not build. Parent admits
one capped job at a time, then reviews the combined changes and records actual
results. This scope does not close unrelated compiler caching or browser gates.

W1/W2 final gate: 582 passed, zero failed/ignored across the full ESSL and wgpu
crates, including native Vulkan draw/readback regressions. Independent review
approved after fixing unused-bool indexing. The audit receipt records source
hashes, resource caps, raw logs and adapter identity. Other lanes remain open.

## I3 retained resource acquisition

**Status:** implemented and verified locally, 2026-10-16. Baseline `95ebacf2a4e`.

Selected as the next highest-value bounded ownership cut: unrelated DOM changes
currently let computed-style/frame synchronization reacquire all resources.
I1/I2/H3 remain important but need a broader host input/lifecycle contract.

The existing document-resource owner will retain acquisition results, including
misses, under document/dependency identity. Live CSSOM consumes reconciled
resources and deltas. Preserve redirect/import bases, dependency replacement,
negative results and navigation reset; evict dead dependencies. Do not replace
the resolver's grammar with a CSSOM-specific parser or add a permanent URL cache.

Done conditions: a counting fetcher through real Boa/Livery CSSOM proves no
additional fetches for unrelated edits/style reads/frames; relevant dependency
changes update resource ledgers and computed styles, removed owners can reload,
and replacement documents cannot reuse stale acquisitions. Run retained-path
regressions and relevant existing suites after implementation, with the shared
lock, one Cargo job, serial tests, 5 GiB memory, zero swap, 150% CPU, nice 10.
Workers edit disjoint production/test files and do not launch builds. Parent
reviews the integrated ownership/invalidation contract and records exact results.

I3 final focused gates: 45 distinct tests passed across nine new retained-path
regressions, 18 resource unit tests and 18 CSSOM tests. One old CSSOM assertion
required the removed reacquisition behavior; updated retention and explicit
owner-reinsertion assertions passed on rerun. Production code did not change
between final gates. The initial broad scripted run timed out and remains
unqualified. Independent source review approved. Exact hashes, logs and resource
limits are in the audit receipt. Journal gaps still reset acquisition; I4/I5/L3
and broader host/input findings remain open.

## I5 mutation journal ownership

**Status:** implemented and verified locally, 2026-10-16. Baseline `bcc46a093d4`.

Capture currently drains the same pending facts used by retained rendering and
resource reconciliation. Remove recorder drain authority. Keep one compaction
boundary for the rendering path and publish/observe every capture batch before
it is discarded. Capture baseline and repeat flushes must not change the live
journal, duplicate records, or silently omit mutations. Retain the conservative
fallback for genuinely missing history. Preserve wire discriminants, identity
translation, replay and the render-free document path.

Done conditions: capture enabled/disabled produces equal incremental decisions
and rendered output; unrelated mutations do not reacquire resources; frame-time
mutations and repeated flush/frame ordering preserve capture batches. Relevant
existing capture/replay and resource-retention tests pass after the cut. Cheaper
implementation workers do not build; parent serializes gates with the shared
lock, one job, serial tests, 5 GiB memory, zero swap, 150% CPU and nice 10.

I5 final gates: 22 distinct tests passed, zero failed/ignored. The three new
regressions establish capture-on/off restyle and scene-operation parity,
unchanged acquisitions, frame-time callback capture, exact-once observation,
headless compaction after successful/throwing scripts, and explicit gap/arena
refusal. Nine existing capture/replay cases, nine resource-retention cases and
the retained missing-history fallback passed. Independent review approved.
Only two unused imports changed after the first gate; subsequent gates compiled
the frozen source. Exact source/log hashes and resource limits are in the audit
receipt. Complete replay fidelity, child-realm capture, browser/wasm and Forms
acceptance remain outside this cut.

## I4 source-relative image identity

**Status:** implemented and verified locally, 2026-10-16. Baseline `5db13a54460`.

The resource ledger inserts authored image aliases that can collide across
stylesheet sources. Carry each declaration's source-relative image identity
through computed styles to layout/paint, and remove ambiguous aliases. Preserve
CSSOM authored text, final redirect/import bases, document-relative inline and
replaced images, and dynamic replacement. Reuse the existing URL resolver and
cascade owner rather than create a second offset/alias-style lookup table.

Done conditions: two sheets using the same relative image spelling retain
separate computed identities and paint their own decoded pixels; relevant
replacement updates the retained path; existing image/layout/style and resource
retention tests pass. Tests follow the cut. Cheaper workers do implementation
and regression files; parent serializes tests with the shared lock, one Cargo
job, serial tests, 5 GiB memory, zero swap, 150% CPU quota and nice 10. L3 payload
copying and broader CSS parser or browser acceptance remain outside this cut.

I4 final gates: 497 passed, zero failed, six existing ignores across cascade,
Livery library/paint, scripted identity/retention and native session paths. Three
new regressions inspect draw-command pixels, identity and placement; they cover
redirected import bases, variable consumption, link/base replacement and literal
image keyframes. Review caught and closed callback argument order, stale
keyframe fallback-base rebuilding and the intrinsic-size assertion gap. The
receipt retains the preliminary compile failure separately. Final code matches
all 24 reviewed source hashes. No new GPU readback, browser run, full-workspace
qualification, or Forms acceptance is claimed. CSSOM-only new resource discovery,
pending/custom keyframe values and L3 sharing remain separate work.

## L4 transform and containing-block ownership

**Status:** bounded repair implemented and verified locally, 2026-10-16. Baseline `9ed0bb563f4`.

L3 was inspected first. Its complete immutable-pixel-sharing cut crosses the
pinned NetRender paint_list_api boundary: ImageResource.data is an owned Vec,
and frame absorption copies it. Defer that supplier/API change rather than call
a Genet-only cache a complete L3 repair. No L3 production change is made here.

L4 is the next bounded correctness cut. Box-tree containing-block establishment
recognizes only the transform list, while paint recognizes individual transforms
and applies a different inline predicate. Give supported transform/context facts
one owner with distinct applicability, containing-block, stacking and paint
projections. Check perspective separately from an element's own matrix. Preserve
containment, fixed/absolute selection, table/grid ownership and retained-motion
admission. Primary terminology: [CSS Transforms 1 transformable elements](https://www.w3.org/TR/css-transforms-1/#transformable-element)
and [CSS Transforms 2](https://www.w3.org/TR/css-transforms-2/) individual
transforms, perspective and computed preserve-3d context establishment.

Done conditions: static ancestors with individual transforms correctly contain
absolute/fixed descendants; none/non-none transitions update retained geometry;
identity transforms count, and non-transformable inline cases remain excluded.
Run relevant existing geometry/transform/paint tests after the cut and add only
changed-behavior regressions. Cheaper workers do implementation and test files;
parent serializes gates under the shared lock, one job, 5 GiB memory, zero swap,
150% CPU quota, nice 10 and serial tests. Preserve explicit 3D rendering limits.

L4 source review found parent perspective/3D-context reads bypassing applicability,
missing supported transform properties in the table wrapper/grid split, and
flex/grid blockification and numeric stacking levels through display: contents.
These are included in this cut; broader projective rendering and used-value 3D
grouping behavior remain outside its qualification.

Remaining L4 boundaries: paint still follows the immediate DOM parent when
applying parent perspective/preserve-3d, so a display: contents intermediary
does not forward the generated parent. The existing affine projection limit
also remains. First regression run compiled and passed six cases, but three
paint assertions found no colored rectangle: positioned spans still enter the
inline-decoration path in begin_node. Their geometry assertions passed. Those
fixtures now explicitly make the positioned children block-level while keeping
the tested parent inline; the positioned-inline decoration defect is recorded
separately rather than silently included in L4 qualification.

L4 final gates passed 453 tests, zero failed, two existing ignores: 335 library,
109 existing integration and nine new retained regressions. The retained source
was frozen before these gates; only explicit child display in the new fixtures
changed after the first run. The review and receipt record exact source hashes,
limits and failure history. No new GPU/browser or Forms acceptance is claimed.
L3 supplier sharing, contents perspective-parent traversal and positioned-inline
decoration remain open with the source boundaries above.

## L3 image sharing and remaining L4 paint boundaries

**Status:** locally implemented and verified, 2026-10-16. Genet baseline `2bf6ff015aa`; NetRender
baseline `f4f9c0d87`. User authorized continuing through all three remaining
items. Supplier work is isolated at
`/home/markik/Code/worktrees/netrender-image-sharing-20261016`.

1. Replace DOM-parent transform traversal and computed-inline decoration checks
   with generated-box ownership. Preserve perspective gap reporting and table
   wrappers; prove contents/no-contents equivalence, restored positioned-span
   draw witnesses, and hit testing.
2. Change supplier ImageResource pixel storage to shared `Arc<Vec<u8>>`, matching
   peniko Blob's backing so frame clones, rekeying and renderer registration
   share the allocation. Preserve serialized byte shape and renderer identity.
3. Give the retained document one versioned encoded/decoded image owner. Layout
   and paint share lazy decoded metadata/pixels; remove the output's encoded
   ledger. Preserve equal-byte entries on replacement, invalidate changed or
   removed resources, bound data-URL cache lifetime, and cache decode failure.

Done conditions: meaningful sharing/decode-count witnesses through intrinsic
layout, new paint, cached frames, scroll, replacement/removal, data URLs and frame
composition; retained draw data and supplier renderer ingestion remain correct.
Run relevant existing suites and feature-off checks after cuts. Parent alone
runs heavy gates with the shared lock, one Cargo job, serial tests, 5 GiB memory,
zero swap, 150% CPU and nice 10. Keep shared build caches and other worktrees.
Local supplier/consumer commits and exact lock provenance are required; publication
is not implied by local verification. No push is included in this continuation.

Supplier `e8cda06efc11c11595f2fa1bb30bce7e72979537` is committed locally.
The manifest now pins all four NetRender crates to that exact revision because
Cargo.lock is intentionally ignored here. The local lock was updated offline
from supplier Git objects imported into Cargo's cache; no path patch or remote
publication is implied. Supplier gates passed 46 focused tests including an
actual CPU image draw; corpus has three passes and one existing border-command
mismatch. A freshly compiled pristine `f4f9c0d87` control reproduced that mismatch.
The temporary control worktree was removed after saving source/log/dependency
provenance. The first control attempt reused a shared artifact and is explicitly
not counted as baseline evidence.

L4 review found and fixed anonymous table-wrapper ownership for captions and
overflow-clip transfer for hoisted descendants of positioned inline boxes.
L3 review found full-layout-only data-URL retirement insufficient: repeated
repaint-only image changes could retain old rasters indefinitely. Retirement now uses
current DOM/computed-style image references after successful paint, replacing
the begin/cancel full-layout sweep machinery.


Final source review and post-cut validation passed 506 Genet tests, zero failed,
two existing ignored. Supplier validation passed 49 tests including actual CPU
raster pixels, with one baseline-reproduced corpus border mismatch retained.
Final gates cover library, transform/paint/layout, images, shadow flat tree,
scripted identity/retention, native document sessions and feature-off decoding.
The new retained-root fixture needed parser mutation draining and a supported
flex ancestor mutation batch: empty image roots have Inline context and are
excluded by the existing local formatter. Its final exact damage, generation,
cache and pixel assertions pass; production layout admission was not changed.
Compile failures and intermediate fixture failures are preserved in the receipt.
Both source reviews are complete. No supplier/consumer publication, renderer
atlas-identity reuse, fresh GPU/browser qualification or Forms acceptance claimed.
See `image_sharing_and_paint_followups` in the maintained post-cut receipt for
source hashes, exact commands, results, resource limits and dependency provenance.
