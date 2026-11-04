# Shaping cold-path and memory cost reduction

> **For agentic workers:** use superpowers:subagent-driven-development. Preserve the worktree and evidence. Steps use checkbox syntax.

**Status (2026-11-03):** selected cost reductions are qualified for the user-authorized main publication; whole-workload cold equivalence remains open. The [borrowed validation plan](2026-11-02_borrowed_shaping_validation_plan.md#publication-reconciliation-2026-11-03) owns current reconciliation/publication. Earlier isolation restrictions below describe the historical stages.

**Goal:** remove the measured cold-layout regression and make retained layout memory understood and controllable while preserving resize reuse, changed-key edits, and parallel gains.

**Architecture:** retain whole inline groups, exact builder-input equality, owner-thread DOM/source projection, and synchronous optional caller-owned workers. Diagnose costs before altering ownership or cache representation. Optimize only experimentally confirmed waste.

**Tech stack:** existing Rust/Parley/Fontique/Rayon; no dependency upgrade or replacement engine.

**Spec:** user-approved follow-up: reconcile current main, profile allocation and owner-thread costs, remove confirmed waste, and rerun preserved controls. Original semantic contract: `2026-11-01_retained_parallel_shaping_plan.md`.

## Global constraints

- Reconciled local merge `7df296b6b9274cf3512cd267c0dd5ad5534f4c59`, previous shaping head `411f47b65a5`, upstream `0cceb7e2fbc`. No shared-main merge, push or consumer repins.
- Existing exact glyph/font/source/line/paint semantics and all font invalidation, atom geometry, generation/error and cache active-result guarantees remain binding.
- Rust floor 1.97.1; default/wasm serial; caller pool only; no unsafe Send/Sync, no DOM worker ownership, no paragraph splitting.
- Preserve old binaries, scripts, raw rows, locks and failures. Source-text budget is not a heap budget. Do not infer causes from peak RSS alone or sum nested timers.
- Shared `/home/markik/Code/testing/machine-heavy.lock`; build/test scope MemoryMax=3G, MemorySwapMax=0, CPUQuota=150%, nice10, one Cargo job and test thread. Matched two-worker performance controls CPUQuota=200%, same memory/swap limits for every mode. Host swap is not disabled.
- Freeze and record reconciled dependency closure before measurement. Narrow workspace-member lock reconciliation only; preserve original lock and explain graph changes, no broad source/version churn.
- Use Terra for diagnosis/implementation and substantive review; Luna for bounded mechanical checks. The final whole-phase review uses the most capable available model under the SDD review requirement. No concurrent implementation agents.

## Review focus

1. Full exact key equality and current source slots after deduplication or sharing.
2. Cache disabled/evicting/oversized groups and active result lifetime.
3. New or replaced fonts, atom size/topology, and K7/late geometry paths.
4. Parallel execution actually admitted, bounded scratch, panic/error publication and teardown.
5. Memory budgets versus allocated and live bytes; long-edit historical retention versus current active output.

## Tasks

### Task 1: reconcile dependency closure and diagnose costs

Files: existing measurement example and diagnostic instrumentation only where necessary; `components/genet-livery/src/text.rs`, `text/cache.rs`, `text/prepared.rs`, `text/parallel.rs`; preserved controls under testing/genet. Do not implement speculative optimizations.

- [x] Confirm source merge and lock provenance; narrowly reconcile the lock for the new workspace member. Complete offline metadata still lacks unrelated sources.
- [x] Measure owner preparation, key/hash/clone, cache, layout and publication work with bounded profiling and probes. The whole cold gap remains unattributed; cumulative allocation traffic is not live payload or a CPU attribution.
- [x] Run bounded long-edit cache-off/default/capacity controls and inspect live heap and RSS high-water behavior. Retained layout vectors were identified and bounded; current output, worker/font scratch, shared resources and the total process heap are not completely accounted for.
- [x] Reproduce targeted original and reconciled controls with pinned real font bytes and matching output, including one-sample sanity. Prior-source full-matrix evidence remains source-bound.
- [x] Write and review the diagnosis and implementation briefs with source, lock, binary, script and raw-evidence provenance. The controller accepted external hashed profiler artifacts rather than committing diagnostic instrumentation; no tracked production instrumentation was introduced.

### Task 2: reduce confirmed costs

Exact files/interfaces and tests were finalized from the reviewed diagnosis and subsequent measured briefs.

- [x] Implement and independently review the selected measured cuts, with behavioral or allocation-work regressions and targeted controls where applicable. Task 2C's initial RED was a missing-API compile failure; its behavioral tests were added after implementation. The controller accepted this historical chronology exception; later tests cannot reconstruct a pre-implementation behavioral RED.
- [x] Preserve exact semantics and configurable settings in the reviewed implementation; original controls and their remaining performance limits are retained.
- [x] Run the recorded covering default/parallel tests and format/diff checks and review each selected cut. Report measured key and layout costs separately; attribution of the entire cold gap and total active-output/process memory remains incomplete.

### Task 3: qualification and documentation

- [x] Reuse the preserved balanced controls, five warmups/30 measured samples for final headline cases, real font-byte identities, matched paint and actual pool-job observations. Compare original, reconciled pre-optimization and final where needed to attribute changes.
- [x] Qualify repeated resize reuse, one-key edits, bounded direct-vector history and lifecycle controls for the exercised modes and settings.
- [ ] Establish the whole-matrix cold outcome against original; no acceptance tolerance has been decided.
- [x] Run appropriate default/parallel package and consumer/wasm gates for changed source; reuse unchanged WPT evidence with explicit source limits or rerun affected cases. Update README, original receipt and this plan with exact evidence and remaining limitations.
- [x] Final whole-phase review, one cohesive documentation fix wave, scoped re-review, clean local checkpoint and preserved worktree. Source/API/test checkpoint accepted for local retention; the separate cold-outcome condition remains open.

**Done:** the accepted control matrix shows cold performance matches original and retains resize/pool gains; additional retained memory is measured and controlled. If a result remains unmet, keep it explicit and continue diagnosis rather than claim a universal improvement.


### Qualification disposition at selected source

Source `24f743517497bec0d44316ab0794572441f70177` passed full Livery default/parallel packages (802/808, eight ignored each), 101 `genet-documents` tests and Livery wasm no-default compile. Current-source WPT, browser/GPU and platform runtime remain UNTESTED; the earlier WPT results are bound to `30901973a284c265044d649c5d3ec95095e3d7c4`. Rust 1.97.1 and sanitizer are not freshly qualified. The original receipt records the build cap, the failed mixed-face validator and byte-for-byte revalidation, the continuation's one 677-microsecond throttle, and the separate source-specific measurements.

Three CPU 0,2 cold cohorts included two independent original processes. Final cached-serial article layout versus original A/B had paired medians `-4.346/-6.719`, `-8.495/+0.089`, and `-2.621/+0.355` ms; the original A/B article medians were `-2.987/+7.815/+2.696` ms. This empirical process control leaves the cold-original goal unresolved, not proven by a universal performance claim. Small cached-serial layout comparisons favored final in these samples, and raised-cap final cached-serial resize reused all 2,048 small keys. Default direct-vector history plateaued below 32 MiB; that is not a heap or RSS bound. Task 2J also remains materially slower than 2I for atomic layout in its CPU 0 samples. Full ranges, signs, order strata, current gates, precise mode/cap distinctions and artifact hashes are in the [receipt](archive_docs/receipts/2026-11-01_retained_parallel_shaping_plan_receipts.md) and preserved Task 3 report. Whole-phase review and the single documentation fix wave passed scoped re-review. The checkpoint is retained locally; the cold-goal disposition remains open pending the user's acceptance criterion.

The non-headline default cold controls also remain relevant: final cached-serial minus original paired layout medians were +0.754 ms for mixed-256 (27/30 positive; both order medians positive) and +0.793 ms for single-long (23/30 positive). Each is one cohort without an original twin; neither establishes a general or causal regression. Small-2048 pool-minus-serial layout medians favored pooling in all three headline cohorts, while frame medians were +2.004/-1.082/-11.152 ms and cycle medians +8.009/+2.527/-10.861 ms. A consistent small end-to-end pool gain is not established. The cold goal remains open pending the user's acceptance criterion.
