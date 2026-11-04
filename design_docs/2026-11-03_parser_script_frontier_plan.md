# Parser and script frontier

**Status: active, adopted 2026-11-03.** Successor to the [historical original](archive_docs/2026-11-03/2026-09-08_parser_script_interleaving_plan.md). Review baseline `725e6c4fa30dfb1e7ea39a383ddac912374e27e7`; approved owner carry is included. Historical counts and receipts retain their recorded qualification. Adoption does not implement remaining work or decide open design questions.

## Established contract

Mark ruled that scripted document parsing interleaves parsing and execution according to the HTML model. HTML parsing pauses for script with correct current-script and readiness ordering; the WPT runner and Livery-backed route use the interleaved path, with CSSOM installed over the parse. Post-parse write appends rather than recreating earlier nodes and inserted scripts can run; parse-time custom-element preparation occurs at creation. The original part-two phase and its later residual closure supersede earlier claims that these are absent. See `components/script-runtime-api/tests/parser_script_interleaving.rs`, `components/genet-scripted/document.rs`, `components/script-runtime-api/dom/markup_insertion.rs`, and `ports/genet-wpt/src/harness.rs`. Exact old-run and engine limits remain in the original.

### Live parser ownership and reentrancy

`components/genet-scripted-dom/parser.rs` owns `ScriptedTreeSink` and the tokenizer driver; it writes the actual `ScriptedDom` through `DomAccess` and `LayoutDomMut`, preserving mutation-observer records, slot assignment and structural-mutation epochs rather than copying a private tree afterward. The driver parks the real arena in the parser around resume and returns it before engine/script calls. It must park that real arena before parser construction because the tree builder caches the document handle. The tokenizer/sink makes no engine calls: `script-runtime-api/parse.rs` owns timing, and `ParserPolicy` answers declarative-shadow questions from a plain table refreshed **after** each script pause. The separate static, script-free route retains its post-parse declarative pass. Coordinate changes to these seams with the active layout/inline owners.

During parsing `document.write` inserts and processes written input at the active insertion point without consuming subsequent document input; stalled written scripts return to the driver's timing model. After parsing, `document.open` replaces the old document; subsequent writes to that new open stream append and preserve earlier newly written node identities. This does not retain the pre-open tree. These are recorded implementation contracts; recheck current source and qualification before changing them.

## Phase P1: async scheduling and dynamic attributes

Reproduce `execution-timing/112.html` and adjacent async/defer mutation cases on current Boa and Vano, with source-matched controls. Determine how fetch completion and task scheduling interact when `async` is removed from a deferred script; distinguish network completion from the synchronous loader. Done when the selected semantics are implemented through the existing scheduler, the named case and controls pass in an appropriate runner, and matched comparisons attribute all movements without lost prior passes. The later `async_runs_after_parser_blocking` regression is a bounded improvement, not proof of case 112.

## Phase P2: XML and foreign scripts

The historical runner explicitly preserves parse-then-run for XML because xml5ever lacks an equivalent pause driver. Foreign-namespace scripts were collected until the next pause rather than executed exactly at their end tag. First reproduce both current routes with ordering and negative controls, then decide the parser/driver seam with its owner. Done when the chosen XML and foreign-script scope executes at the required observable point on both engines, correctly handles document lifecycle and resource errors, and has matched HTML/XML and SVG regressions. If capability or parser architecture requires a design fork, record the precise checkpoint rather than simulate success.

## Phase P3: browsing-context and headed integration boundaries

`document.open(url, name, features)` delegates to `window.open`; the original lane reported refusal. Confirm current implementation and place missing behavior with the navigation/realms owner. Establish a scripted headed witness that visibly depends on interleaved external and inline scripts, ordering and a completion signal, with exact source, engine, host and viewport recorded; script-free Ortet is only a control. Keep external module loading and async/defer stream scheduling separate from the bounded blocking-classic receipt in the Realms inventory.

## Open decisions and qualifications

Current source and matches may have moved since the September measurements; remeasure before implementation. Do not reopen as unresolved: runner and Livery routes, post-parse written-script execution and stream identity, creation-time custom elements, MathML integration point (part two), quirks mode (line-box follow-up), invalid-to-valid script preparation, no-context script suppression, or the test-surface restoration, all recorded as later closed. `execution-timing/112` is specifically retained in the final closure note. Original rulings, source identities and negative controls remain in the original plan and its receipts. No old passing count constitutes a current gate.

## Cross-owner carry, 2026-11-03

The approved Realms carry assigns legacy-charset iframe navigation/parsing and the census lane's XML navigable parsing and `document.open()` exception to iframe/Realms and parser owners. P2/P3 must identify the actual original cases and current source before labeling any gap open or fixed. A historical Encoding or Vano receipt is not fresh XML, origin/security or native qualification.

## Adoption record

Mark approved all eight reviewed cuts on 2026-11-03: "Yep". The original is archived whole; the reviewed technical scope, historical qualifications and open decisions are retained.
