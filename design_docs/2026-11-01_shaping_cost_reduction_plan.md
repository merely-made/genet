# Shaping cold-path and memory cost reduction

> **For agentic workers:** use superpowers:subagent-driven-development. Preserve the worktree and evidence. Steps use checkbox syntax.

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
- Use Terra for diagnosis/implementation and substantive review; Luna for bounded mechanical checks. No concurrent implementation agents.

## Review focus

1. Full exact key equality and current source slots after deduplication or sharing.
2. Cache disabled/evicting/oversized groups and active result lifetime.
3. New or replaced fonts, atom size/topology, and K7/late geometry paths.
4. Parallel execution actually admitted, bounded scratch, panic/error publication and teardown.
5. Memory budgets versus allocated and live bytes; long-edit historical retention versus current active output.

## Tasks

### Task 1: reconcile dependency closure and diagnose costs

Files: existing measurement example and diagnostic instrumentation only where necessary; `components/genet-livery/src/text.rs`, `text/cache.rs`, `text/prepared.rs`, `text/parallel.rs`; preserved controls under testing/genet. Do not implement speculative optimizations.

- [ ] Confirm source merge/lock provenance and narrowly repair locked closure if required by the new workspace member.
- [ ] Trace and measure cold owner preparation, key construction/hash/equality/cloning, cache/LRU activity, unbroken layout construction/cloning and publication. Identify counts/bytes/CPU evidence using an available profiler or small opt-in probes; distinguish live payload from allocation churn.
- [ ] Diagnose long-edit memory: current active output, retained old unbroken layouts, worker/font scratch, and harness/provenance contributions. Run bounded cache-off/default/capacity controls sufficient to distinguish causes. Account for process RSS high-water behavior.
- [ ] Reproduce essential old/new controls with pinned real font bytes and matching output. Start with one-sample sanity; use only targeted cases until the hypothesis is established. Reuse prior full matrix as prior-source evidence, not current-source qualification.
- [ ] Write diagnosis report with exact source/lock/binary/script identities, commands, raw evidence, confirmed and rejected hypotheses, and a concrete minimal implementation brief. Commit any instrumentation; task review before optimization.

### Task 2: reduce confirmed costs

Exact files/interfaces and tests are finalized from Task 1's measured diagnosis in its reviewed brief, rather than prescribing a guessed optimization now.

- [ ] For each confirmed cause, create a behavioral/allocation-work regression where meaningful, observe failure, make one minimal change, and compare the targeted control before proceeding.
- [ ] Preserve exact semantics and configurable settings; never hide the cold regression by relabeling the baseline or removing correctness work without equivalent proof.
- [ ] Run covering default/parallel tests and format/diff checks. Report costs separately for keys, retained layouts and current active output. Review each independently rejectable change before the next.

### Task 3: qualification and documentation

- [ ] Reuse the preserved balanced controls, five warmups/30 measured samples for final headline cases, real font-byte identities, matched paint and actual pool-job observations. Compare original, reconciled pre-optimization and final where needed to attribute changes.
- [ ] Establish cold layout against original, repeated resize reuse, one-key edits and long-edit memory; include small edit and lifecycle controls relevant to changed paths. Extend measurements only for new failures or unresolved questions.
- [ ] Run appropriate default/parallel package and consumer/wasm gates for changed source; reuse unchanged WPT evidence with explicit source limits or rerun affected cases. Update README, original receipt and this plan with exact evidence and remaining limitations.
- [ ] Final whole-phase review, one cohesive fix wave if required, scoped re-review, clean local checkpoint and preserved worktree.

**Done:** the accepted control matrix shows cold performance matches original and retains resize/pool gains; additional retained memory is measured and controlled. If a result remains unmet, keep it explicit and continue diagnosis rather than claim a universal improvement.
