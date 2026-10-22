# Remaining R7, I6 and H5 ownership cuts

Continuation authorized after the five-slice audit sequence. Base Genet `7f13b85f53c`, Mere `8c299283f`, Turnstone `ca1534e`. Retain unrelated reduction-plan edits. Commit and publish completed cuts directly to main, with downstream adoption pins where needed.

## Ownership and done conditions

- **I6:** the host request/response contract owns actual remote fetch semantics. Migrate active external producers, preserve arbitrary valid methods, headers/body, response status, redirects and final identity. Byte consumers choose an explicit lossy boundary. Local synthetic responses remain explicitly local; remote responses must not acquire invented success metadata. Audit all remaining implementations before claiming closure.
- **H5:** one configured `ResourceFetchPolicy` reaches native and browser adapters. Enforce streamed decoded bytes, elapsed deadlines and shared admission. Preserve navigation cancellation/generation and aggregate document budgets. Refuse custom browser redirect caps that browser Fetch cannot enforce. Verify chunked, oversized, slow, redirected and overlapping requests with the actual browser path.
- **R7:** a native supervised process owns each required-cancellation worker tree. Independent worker engines/heaps and page-owned resource service remain. Nested workers stay inside that process. Terminate and runtime/document drop kill and reap the process and join IPC bridge threads; parent-pipe loss retires the child. The helper executable/backend is configured explicitly; unavailable support refuses. Explicit trusted cooperative mode may remain, without a reclamation claim. Prove startup/task/timer/microtask/descendant hangs under an external watchdog on both Boa and Nova.

## Execution

Cheaper implementation agents own disjoint source areas; controller owns review, all builds/tests, commits and pushes. Relevant retained tests and regressions run after each cut. Serialize heavy work with `/home/markik/Code/testing/machine-heavy.lock`, one Cargo job/test thread, nice 10, MemoryMax=5G, MemorySwapMax=0 and CPUQuota=150%. Preserve failed attempts and source hashes. No cleanup of active worktrees or evidence.

Ruling: this continues the already authorized audit designs and publication workflow; no new approval checkpoint is introduced by workflow templates. Keep the evidence ledger rather than deleting it. Tests run after cuts as explicitly requested. Process isolation follows the audit's stated R7 option because neither pinned VM supplies enforceable asynchronous interruption.

## Review decisions and retained boundaries

- The native `ResourceRequest` to Netfetcher conversion belongs to Netfetcher. Ortet and both Mere HTTP producers call that owner. Invalid custom method tokens are refused rather than silently becoming GET.
- `ResourceResponse` remains a deliberate byte-document view. `ResourceFetchResponse` is the live HTTP response. Byte snapshots, held inventory, empty fetchers and non-HTTP receipt sources explicitly decline rich requests; they cannot invent HTTP 200 metadata. Local files/data and controlled local receipt fixtures retain explicit synthetic responses.
- Native admission consumes the same elapsed deadline as headers/body. The new scoped rich-fetch path has one elapsed deadline; the existing range-streaming patience contract remains separate.
- Browser Fetch follows at most its platform redirect limit and does not expose intermediate redirects. A different configured redirect cap is refused. Receipt-only hooks share the actual admission pool and fetch implementation.
- The native process helper is a trusted, explicitly configured executable containing the selected engine. The lifetime guarantee covers script-created worker descendants inside that helper, not arbitrary replacement executables which spawn independent OS processes.

## Verification progress

The browser Wasm build and actual HTTP receipt passed locally. Twelve policy checks passed, including decoded-byte overflow, exact byte boundary, redirected response bytes, elapsed body deadline, zero admission, queued deadline before wire, and permit release. The retained page rendered via the physical AMD gcn-5 WebGPU adapter: 640×400, 136128 card pixels, 8960 border pixels, 24462 glyph pixels and 3744 image pixels; redirect, nested CSS import, authored font, image, denied response and budget refusal checks passed. Evidence is in `/home/markik/Code/testing/audits/genet-architecture-20261017/remaining-three/browser.json`.

The first browser attempt exposed missing generated JavaScript snippet serving in the receipt server. The server now serves only a restricted snippet path pattern; the failed receipt is retained as `browser-missing-snippets.json`. Native and worker gates subsequently passed: 227 unit/retained/regression tests and 36 distinct isolated scenarios across Boa and Nova. The report-only crashed/reaped distinction and invalid-queue early refusal received focused final checks. Ortet/scripted/runtime all-target checks, full Graphshell Wasm and the separate Boa scripted-worker Wasm compile also passed. See the remaining-three receipt for exact source hashes and corrected attempts. Downstream Git-only publication is verified separately during adoption.
