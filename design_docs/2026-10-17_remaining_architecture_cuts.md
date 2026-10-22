# Remaining architecture cuts: five slices

User-authorized continuation of the 2026-10-16 source audit. Start: c1b4100dca7. Each slice is reviewed, tested on its retained path, committed and pushed to main before the next slice. Fetch and integrate concurrent main changes before publishing. Preserve unrelated worktree edits and historical receipts.

| Slice | Findings | Owner boundary and done condition |
|---|---|---|
| 1 | I13, L10, L11, L12 | Compiler-owned shader normalization/precision; prospective fragmentation gated; shared axis facts; honestly historical census. Retain linked shader layouts, unsupported fragmentation markers, continuation repros and property classification. |
| 2 | R7, R8, R14, I2 | Truthful execution bounds and worker lifetime, owned registrations, shared live/headless lifecycle. Unsupported required bounds must refuse rather than run unbounded. |
| 3 | I6, H5, I9, R12 | Shared host request/response contract, enforceable native fetch policy, one cookie-policy owner, structured history serialization. |
| 4 | R10, R11, L5, I10 | Private WebGL identity; faithful XPath adapter; retained geometry and semantic projections consumed by automation. Preserve caller contracts or migrate actual callers. |
| 5 | L9, H2, H9, H10 | Honest backend boundary, checked browser rendering, same-queue synchronization, observable presentation outcomes. Hardware/browser proof remains required where the audit requires it. |

## Verification and resources

Cheaper implementation agents; controller alone runs builds and commits. Test after each slice's cuts, using relevant existing tests and regressions for changed behavior. Mechanical moves do not need a new testing campaign. Serialize Cargo using /home/markik/Code/testing/machine-heavy.lock; one build job, one test thread, nice 10, MemoryMax=5G, MemorySwapMax=0, CPUQuota=150%, bounded gates. Preserve test receipts and active targets. Delete only identified disposable artifacts if storage pressure warrants it.

## Scope and evidence

The four previously partial findings are outside this twenty-finding sequence. A finding is addressed only when its stated ownership cut and required evidence are established. Record any remaining platform/dependency/caller limits explicitly; do not relabel incomplete work to meet the batch count. Publication of a slice does not establish unrelated Forms or full browser conformance.

## Progress

Slice 1 published as `ec5d8b997aa`: four findings addressed, 591 passing tests and six retained ignores. Slice 2 published: I2/R8/R14 addressed locally; R7 partial because VM cancellation remains unavailable. 131 passing tests, Ortet host compile and actual GPU draw; source hashes and excluded attempts in the slice 2 receipt.

Slice 3 source cuts: I9/R12 addressed locally; I6/H5 partial. The default cookie jar enforces public-suffix and prefix rules. History state uses shared structured records and an explicit bare-runtime owner that transfers at context initialization. External byte-only callers still need I6 migration. Native H5 byte, deadline and admission limits are implemented, but the browser adapter does not yet consume the same configured policy. Cross-document history traversal preserves state and final redirect URL; realm-install failure after outgoing realm release remains an inherited navigation limit, not atomic recovery. 163 tests passed with no failures or ignores; Ortet scripted-Nova all-target check passed. Source hashes and gates are in the slice 3 receipt. Slices 4 and 5 retain their listed done conditions.

Slice 4 verified: R10/R11/L5/I10 addressed locally, with 102 passing cases across corrected focused runs and Piccolo, Ortet and Graphshell Wasm checks. Cross-repository source hashes and limits are in the slice 4 receipt. Mere/Turnstone publication adopts the supplier image API and portable Git-only pins. Slice 5 remains L9/H2/H9/H10.

Slice 4 published on Genet `cb6161cc6cbdad2f80d0e0433add3120e05e47ae`, Mere `4490ae1651f88b4f22f7e7fdd1ef12b81c6b3008`, and Turnstone `65ccf70f0e0e8be7ae40c6f4d2ae06124024d6ac`. Git-only dependency graphs match tested graphs.

Slice 5 verified: L9/H2/H9/H10 addressed within recorded limits. 294 unit/regression cases passed, Graphshell Wasm checked, Chrome AMD WebGPU actual draw/lifecycle witness passed, and Wayland master handoff passed 60 frames both before and after the timeline cut. Seven partial findings remain across the complete audit; three (R7/I6/H5) are from this twenty-finding sequence. Platform and fallback limitations remain in the slice 5 receipt.
