# WebAssembly pages plan: the `WebAssembly` object in documents and workers

**Status:** 2026-11-25. In progress: P2 and P5 met; P1 reopened for frame realms; P6 in progress.

**For:** pages and workers that use WebAssembly. The object exists on both Genet JavaScript engines
behind a `wasm` feature, but no document, worker or WPT run turns it on, so no page can reach it.
This plan is cut from the open items of the WebAssembly object plan, which is archived whole at
`archive_docs/2026-11-25/2026-11-22_webassembly_object_plan.md`. Its Findings 1 to 54 are the
background. In particular, Findings 25 to 37 describe the object's design, and Findings 43 to 54 give
the counts and what is missing.

**Not here:**
- **Blocked on wasmi.** Exception handling and GC types: wasmi 2.0.3 implements neither.
  Shared memory: wasmi has no threads.
- **Left out by D1.** JSPI, the js-string builtins and ES module integration.
- **Engine-level work.** Bounding plain script on Vano or Bole.
- **A browser build of Genet,** where the page's own `WebAssembly` is the backend
  (the old plan's D2).

## Constraints

- `script-engine-api` stays engine-neutral. The hook D2 adds has a default that refuses, as
  `eval_bounded` does, and a backend built without wasm refuses rather than pretending.
- The JavaScript interface spec is the guide, and the WPT `wasm/jsapi` tests are the yardstick. The
  standalone runner's count is the baseline to match: 946 of 1066 on both engines, as of Vano
  `a739d12d` and Bole `85dd7391`.
- Vano and Bole are ours to change. Changes are additive and named here before they are made.
- Builds go through `tools/build_storage.py`, one lane per phase.

## Phases

### P1. A hook and the wiring (D2, D3, D4)

**Done-conditions:**
- `ScriptEngine` gains an install hook for `WebAssembly` that takes an optional fuel allotment. It
  refuses by default. Vano and Bole implement it when built with `wasm`.
- `genet-documents`' `scripted` and `scripted-nova` features turn on each engine's `wasm` feature
  (D3), and so does Ortet through them.
- Each scripted document session installs the object when its realm is made, with no fuel unless the
  embedder sets some (D4).
- Dedicated workers do the same.
- A document test on each engine compiles and calls a module from page script.

### P2. `wasm/jsapi` in genet-wpt

**Done-conditions:**
- genet-wpt runs `wasm/jsapi` as `.any.html` and `.any.worker.html` on both engines.
- The results are in expectation files beside the existing ones.
- The pass count on the window scope matches the standalone runner's 946 of 1066 on each engine, or
  each difference is explained in Findings.
- The standalone `wasm_jsapi` examples stay as fast checks.

### P3. Streaming compilation

**Done-conditions:**
- `WebAssembly.compileStreaming` and `instantiateStreaming` take a `Response` or a promise of one. They
  follow the spec's checks on MIME type, `ok` status and a used body, and they reject with the
  spec's errors.
- Documents and workers have them on both engines.
- `wasm/webapi` in genet-wpt has expectation files. Any subtests it can't pass are listed in Findings
  with the reason.

### P4. Freeing what script has dropped

**Done-conditions:**
- A module, instance, memory, table, global or function handle that script can no longer reach is
  released from wasmi's store, or from the handle tables that keep it alive, during the engine's own
  collection. The same goes for externref slots.
- The design comes back to Mark as a fork before it is built, where more than one shape is
  defensible.
- A test on each engine compiles and drops modules in a loop and shows that the held count stays
  bounded.

### P5. Annex B on Bole

**Done-conditions:**
- genet-bole builds Boa with `annex-b`, so Bole in Genet has `escape`, `unescape` and the rest of the
  web-compatibility annex, as Vano has.
- The `wasm_jsapi` runner no longer needs its `unescape` polyfill on Bole, and still reads 946.
- genet-bole's tests pass.
- Every Bole WPT expectation file that changes is regenerated, and each change is explained in
  Findings.

### P6. wasmi's SIMD and 64-bit memory

**Done-conditions:**
- genet-wasm turns on wasmi's `simd` and `memory64`.
- At the JavaScript boundary, `v128` behaves as the spec says: it throws a `TypeError` when it would
  cross.
- `WebAssembly.Memory` and `WebAssembly.Table` take the `address` member and use BigInt where the
  interface's 64-bit additions say to.
- The shared conformance tests cover both features.
- The `wasm_jsapi` count is re-read and recorded.

## Decisions

- **D1. Lane scope (Mark, 2026-11-25):** Evidence: the open items of the archived plan (its
  Findings 48 and 54). wasmi 2.0.3's configuration has no switch for exceptions or GC types.
  Offered: wiring, lifetime and fixes (recommended); that plus JSPI and the string builtins; wiring
  only, first. Answer: "Wiring, lifetime, fixes (Recommended)". What follows: P1 to P6. JSPI and the
  string builtins stay open (see Open questions). Outcome: new.

  Links: area Scripting · plans webassembly-pages P1, webassembly-pages P2, webassembly-pages P3, webassembly-pages P4, webassembly-pages P5, webassembly-pages P6 · summary The WebAssembly follow-up covers wiring into pages and workers, freeing, Bole's Annex B and wasmi's SIMD and 64-bit memory.

- **D2. An install hook on `ScriptEngine`:** Evidence: document sessions are generic over
  `ScriptEngine` (`components/genet-documents/src/engines/scripted.rs:42`), so a session can't
  reach an engine's inherent `install_webassembly`. Taken as the D3 wiring requires, not offered
  as a fork. What follows: one trait method with a refusing default, the same pattern as
  `eval_bounded`. Outcome: revises the archived plan's D4 and D12, which kept the trait unchanged
  while nothing generic had to install the object.

  Links: area Scripting · plans webassembly-pages P1 · summary ScriptEngine gains a refusing-by-default hook that installs WebAssembly, so generic document sessions can turn it on.

- **D3. On with scripted (Mark, 2026-11-25):** Offered: on with `scripted` (recommended); a
  separate `wasm` feature on genet-documents and Ortet; a setting for each document. Answer: "On
  with scripted (Recommended)". What follows: every scripted page sees `WebAssembly`, as in a
  browser, and the scripted build now includes wasmi. Outcome: new.

  Links: area Scripting · plans webassembly-pages P1 · summary Scripted documents get WebAssembly by default through genet-documents' scripted features.

- **D4. Unbounded unless set (Mark, 2026-11-25):** Evidence: neither JavaScript engine can bound
  plain script (archived plan, D13), so a fuel limit on wasm alone stops no hostile page.
  Offered: unbounded unless set (recommended); an allotment for each task; a fixed budget for each
  document. Answer: "Unbounded unless set (Recommended)". What follows: documents install with no
  fuel, and embedders may pass some through the hook. Outcome: new.

  Links: area Scripting · plans webassembly-pages P1 · summary Documents run WebAssembly without a fuel limit unless the embedder sets one.

- **D5. Two agents at a time (Mark, 2026-11-25):** Offered: two at a time (recommended); three;
  one. Answer: "Two at a time (Recommended)". What follows: P1 and P5 first, then P4 and P6, with
  P2 and P3 following P1. Each agent has its own worktree and build lane, and each result is
  reviewed before it lands on main. Outcome: new.

  Links: area Scripting · plans webassembly-pages P1, webassembly-pages P4, webassembly-pages P5, webassembly-pages P6 · summary The WebAssembly pages lane runs two agents at a time.

## Open questions

1. **JSPI.** wasmi's resumable calls already suspend the wasm stack at an import, which is the hard
   part of promise integration. Whether to take JSPI up is open.
2. **The js-string builtins.** They are a compile option backed by host imports. Whether to take
   them up is open.
3. **D7 of the archived plan.** A grow by zero pages from wasm still can't be seen. The fix needs a
   change in wasmi (Findings 1). Whether to propose that change upstream is Mark's call.

## Findings

1. 2026-11-25: wasmi 2.0.3 returns early on a grow by zero (`wasmi_core` `memory/mod.rs:184`,
   `if additional == 0 { return Ok(self.size()) }`). It does this before the resource limiter's
   `memory_growing` runs, so no host hook sees the grow. The archived plan's D7 can't be fixed from
   Genet's side.
2. 2026-11-25: wasmi 2.0.3's `Config` has switches for SIMD, relaxed SIMD, multi-memory, tail
   calls, extended constants, custom page sizes and wide arithmetic, plus a `memory64` cargo feature.
   It has none for exception handling, GC types or threads. genet-wasm turns wasmi's default features
   off, so `memory64` and `simd` are off as well.
3. 2026-11-25: pages never run in the engine's main realm. `open_top_realm` goes through
   `Runtime::create_child_realm` (`components/script-runtime-api/lib.rs`), so the install hook
   takes a realm, and each window realm, iframes included, evaluates the bootstrap once. All the
   realms share the engine's one wasm store and fuel. Fuel is therefore a setter of its own
   (`set_webassembly_fuel`). Before that change, the next realm's install would reset it.
4. 2026-11-25: installing at realm creation doesn't conflict with Vano's snapshot clones. Vano
   refuses to clone only when wasm objects are live. An object that is installed but unused clones
   with a fresh store.

5. 2026-11-25: realms made inside a native callback had no `WebAssembly`. These come from
   `create_realm_from_call` in `components/script-runtime-api/frames.rs`, which makes every iframe
   realm and the document a top-level navigation opens. A second hook installs into the callback's
   realm (P1, reopened).
6. 2026-11-25: genet-wpt `wasm/jsapi`, before the frame-realm fix:
   - **Window scope:** 946 of 1072 on both engines. Every file the standalone runner reads gives the
     same count. The extra subtests are `idlharness` (it fetches `WebIDLParser.js`, and the disk mode
     has no server) and `reserved-import-names`.
   - **Workers:** 934 of 1071 on Vano and 934 of 1059 on Bole.
   - **Bole timing:** Bole's `js-string/constants` needs about 20 s, which overruns a worker's
     10-second testharness limit. Bole is recorded at `--jobs 3`, because `instantiate-bad-imports`
     flakes at 6.
7. 2026-11-25: `importScripts` evaluates each script with an indirect eval
   (`components/script-runtime-api/worker.rs:1657`). A top-level `let` or `const` in one imported
   script is therefore invisible to the next, where classic scripts share the global lexical
   environment. That costs 12 worker subtests in `wasm/jsapi` (`constructor/instantiate`,
   `instance/constructor`), and it affects every worker test that splits top-level bindings across
   scripts. The fix needs an eval of a global script from a callback that keeps the thrown value.
   `eval_from_call` turns errors into strings. This is a worker runtime bug, not a wasm one, so it is
   left open here.
8. 2026-11-25: Bole's `annex-b` feature turns on `boa_ast/annex-b` and `boa_parser/annex-b`, and it
   gates about 40 sites in the engine. It moved no result in the 24 Bole WPT expectation files that
   name a subset. A control run built without the feature gave the same per-test results. The
   exceptions are two `dom` files that are killed for hanging depending on load. Even so, 17 of those
   files no longer match their checked-in baselines, for reasons other than this lane. Newly passing
   HTMLCollection and `getElementsBy*` tests account for some of it, and timeouts for the rest. They
   need re-recording on an idle machine.

## Progress

- 2026-11-25: plan cut from the WebAssembly object plan's open items. The old plan is archived whole.
- 2026-11-25: P1 met (genet `cbe8d3ed5d5`, `57fbac97e50`).
  - `ScriptEngine::install_webassembly(realm)` and `set_webassembly_fuel` added.
  - `Runtime` installs in window realms and worker realms.
  - The `scripted` features and `genet-worker-process` turn the engines' `wasm` on.
  - Page tests pass on Boa and Nova, and a dedicated-worker test passes on Boa.
- 2026-11-25: P2 met (genet `7a175209bb8`). genet-wpt builds both engines with `wasm`, and
  `wasm_jsapi_{boa,nova}.json` pin the directory.
- 2026-11-25: P5 met (genet `beef06a1875`). genet-bole builds Boa with `annex-b`. The `wasm/jsapi`
  runner drops its `unescape` polyfill and still reads 946 of 1066 on both engines.
