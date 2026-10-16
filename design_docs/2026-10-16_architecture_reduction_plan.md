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
