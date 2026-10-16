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

The user requested a fresh source audit after testing. That audit remains
unperformed: the account meter reached the agreed 41% cutoff after tests.
Do not present this bounded reduction as completion of the twelve lanes.
