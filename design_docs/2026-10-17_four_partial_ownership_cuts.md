# Finish the four partial ownership findings

Base: Genet `4db9557c9c1`, fast-forwarded during verification to `44657594dc4` (event-plan/baseline updates, standalone event module and Rust 1.99.1; tested source hashes unchanged). User authorizes I1, I11, H6 and L6 completion, then commit and push at the end. Preserve the unrelated reduction-plan edit.

## Done conditions

- I1: shared DOM editing defaults reached by live scripted input; text, IME/preedit, selection, key release, cancellable Tab/default actions and trusted input event ordering. Native/scripted same-fixture witness, not setter-only tests.
- I11: shader object retains source/stage-owned compiled artifacts. Links consume parsed/checked/validated artifacts and own only pair-dependent layout/lowering. Preserve diagnostics, source invalidation, reflection and actual GPU draw.
- H6: Test262 distinguishes trustworthy phase/type from infrastructure and completion failures using engine outcomes. Combined async/module routing remains correct. Qualify the classifier against real engine fixtures and available corpus; conformance failures must remain failures, never be relabeled as passes.
- L6: one mutation dependency classifier supplies affected semantic owners to distinct style and layout consumers. Preserve former/current parents, radios, form association, validity/select aggregation and conservative fallbacks. Compare retained selectors and geometry with fresh results on the same mutation fixtures.

## Execution and resource limits

Cheaper implementation agents, controller-owned review/tests/publication. No concurrent builds. Use `/home/markik/Code/testing/machine-heavy.lock`, one Cargo job and test thread, nice 10, MemoryMax=5G, MemorySwapMax=0, CPUQuota=150%. Tests after cuts, except a specific safety uncertainty. Reuse existing targets; no deletion of active worktrees/evidence. Approximately 460 GiB free at start.

## Preflight and rulings

| Task pair / task | Interface and decision |
|---|---|
| I1 / H6 | Both may use ScriptEngine/runtime APIs. H6 owns typed diagnostic additions; I1 must consume without editing those files concurrently. |
| I1 / L6 | Input mutates DOM state; L6 owns Livery mutation classification, I1 owns session/runtime input translation and shared DOM edits. |
| I11 / others | Compiler/backend isolated from input, mutation, runner changes. |
| I1 | Live host entry points and cancellation are required, not merely value setters. |
| I11 | Pair-dependent lowering remains necessary; eliminate repeated semantic compilation, not linked layout. |
| H6 | Classifier qualification does not imply either VM passes all ECMAScript conformance tests. Preserve complete run accounting and failures. |
| L6 | Shared semantic facts do not merge style roots with formatting roots. |

Ruling: continue already-authorized designs and direct publication, retaining evidence. User instructions override workflow templates requiring earlier tests, intermediate commits, or further publication approval.

## Progress

Source cuts and Genet/Mere/Turnstone retained gates are complete. See the audit's four-partial section for exact counts and corrected failures. Turnstone passed all eight retained-pane tests on Rust 1.99.1. Supplier revisions are published; downstream adoption follows this Genet commit with graph equivalence checks. Independent final source reviews found no actionable remaining defect; raw failure receipts were preserved.

The live Chromium keyboard witness produced `keydown, microtask, beforeinput, input, keyup`. Preserve the checkpoint between the host key and its produced text. `KeyboardText` explicitly pairs text with key cancellation; independent `Text` (paste/accessibility) must not inherit stale key cancellation. Physical IME and complete InputEvent/CompositionEvent IDL remain separate qualification limits.

The separately approved native event-dispatch plan (`docs/2026-10-17_native_event_dispatch_plan.md`) remains its own implementation/qualification work. I1 closes the live scripted editing route identified in this audit; it does not claim that native event convergence or all Forms/platform acceptance gates are finished. Turnstone was fast-forwarded to `428deb5` before its consumer gate, adopting the concurrent Rust 1.99.1 toolchain update.
