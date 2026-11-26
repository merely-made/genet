# WebAssembly pages plan: the `WebAssembly` object in documents and workers

**Status:** 2026-11-25. Met: P1 to P6. P3 is met as far as disk mode reaches. Open: the items under Open questions and Findings 7, 13 and 15.

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

- **D6. Keepers and an idle reset (Mark, 2026-11-25):**
  - **Evidence:** Findings 11. wasmi frees nothing short of dropping the store, and Genet pins every
    wrapper, import and externref.
  - **Offered:** keepers and an idle reset (recommended); one store for each linked group; releasing
    handles only.
  - **Answer:** "Keepers + idle reset (Recommended)".
  - **What follows:**
    - Wrappers register with a `FinalizationRegistry` that releases their handles and module bytes.
      The handle tables reuse released slots.
    - Import functions and externref values are held from script, by a keeper for each linked
      group. The tracing collector then reclaims cycles, and no root is added per crossing.
    - Every buffer script can see keeps its keeper alive.
    - When no wrapper or buffer is live, the host drops its wasmi engine and store and starts fresh.
  - **What stays:** a long-lived instance still lets the store's small entries grow, and memory bytes
    are freed only at a reset.
  - **Outcome:** new; answers P4's design fork.

  Links: area Scripting · plans webassembly-pages P4 · summary Dropped WebAssembly is freed through FinalizationRegistry, keepers held from script, and a reset of the wasmi store once nothing is live.

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
9. 2026-11-25: with wasmi's `simd` and `memory64` on, the standalone runner still reads 946 of
   1066 on both engines. This checkout's `wasm/jsapi` has no tests that use `address` or SIMD, so
   the new paths are covered by the shared tests in `conformance/wide.rs` alone. An i32 `Memory` or
   `Table` still takes a BigInt for `initial`, `delta` and `index`, where the spec says `TypeError`.
   No test checks this, so it is left open.
10. 2026-11-25: WebAssembly in frame realms made no difference to genet-wpt's `wasm/jsapi`
    results.
    - **`proto-from-ctor-realm.html`:** still errors. On Nova, `bindOther.call(proxy)` throws before
      any subtest runs. On Bole, a constructor called with a `NewTarget` from another realm doesn't
      take its prototype from that realm. Neither cause is confirmed as the whole story.
    - **`functions/entry*` and `incumbent`:** these test entry and incumbent settings objects with
      `window.open`, and they are still open.
11. 2026-11-25, P4 design read:
    - **wasmi frees nothing short of dropping the store.** A wasmi 2.0.3 store's arenas have no
      remove (`wasmi_collections` `arena/mod.rs:64-190`). That covers funcs, memories, tables,
      globals, instances, segments and externrefs. The engine's `CodeMap` only grows
      (`engine/code_map/mod.rs:73-78`), so dropping a `Module` doesn't free its code. Dropping the
      store and engine is the only release.
    - **What things cost.** An instance with one page of memory costs about 65 KiB, and one without
      memory about 0.9 KiB.
    - **Genet pins everything on top of that.** The bootstrap's `funcObjects` and the entity
      `objects` maps are strong. Import functions and externref values are strong roots on the Rust
      side. Each externref that crosses into wasm adds a root and a wasmi externref
      (`script-engine-nova/native/wasm/bridge.rs:186`, `script-engine-boa/wasm/bridge.rs:141`). A
      loop calling `f(obj)` therefore grows without bound and keeps `obj` alive.
    - **`FinalizationRegistry` works on both engines as built in Genet,** so no fork change is
      needed. `script-runtime-api` already uses it with native callbacks (`fetch.rs:933`).
    - **GC differs by engine.** Vano collects only when the embedder asks: pages once per frame, and
      workers have no cadence. Bole collects on its own allocation threshold.
12. 2026-11-25: `compileStreaming` and `instantiateStreaming` live in `script-runtime-api`
    (`wasm_streaming.js`), beside `Response`. Each realm evaluates them once `WebAssembly` is
    installed. The MIME check is a case-insensitive exact match on `application/wasm`, as
    `contenttype.any.js` expects, so parameters are refused. The body is read whole through
    `arrayBuffer()`, with no true streaming.
13. 2026-11-25: genet-wpt `wasm/webapi` in disk mode reads 492 of 562 on both engines (19 of 56 files
    all-pass).
    - **Needs a server.** `abort`, `historical` and the accepting cases of `contenttype` fetch
      `incrementer.wasm`, which disk mode can't serve. The window halves of
      `wasm_stream_{compile,instantiate}_test` and their CORS cases are server-bound too.
    - **Out of scope.** `esm-integration`.
    - **Worker variants.** In workers, disk fetch works, and the ArrayBuffer, stream, blob and
      FormData cases pass. That is the evidence the checks are right.
    - **No server on this Mac.** `wpt serve` fails here: there is no `python` on PATH, it crashes
      importing `pywebsocket3` under Python 3.15, and there are no `web-platform.test` hosts entries.
      Until it runs, the server-bound subtests are untested and the `wasm_webapi_*` files hold disk
      results only.
    - **Lost in workers, likely from Findings 7.** Three "Empty module" subtests in
      `instantiateStreaming.any.worker.html`. Not confirmed.
14. 2026-11-25: Vano's `FinalizationRegistry` loses a held value when exactly two cells are
    queued. The cleanup job pops one value and then tests `queue.len() == 1`. That calls only the
    popped value and drops the one left
    (`nova_vm/src/ecmascript/execution/weak_ref_and_finalization_registry.rs:96`, Vano
    `a739d12d`). The fix is `queue.is_empty()`.
    - The bootstrap works around it. Each watched object also registers a companion token that
      shares its held record, so cells always die in pairs and the record acts once.
    - Registering one target twice doesn't help, because Vano keeps one cell per target.
    - Fixed in Vano `ed1e0e5e` (Mark, 2026-11-26), with `nova_vm/tests/finalization_registry_tests.rs`,
      which checks one to four cells dying together. test262 can't force a collection, so it never
      reached this path; its 76 `FinalizationRegistry` and `WeakRef` tests are unchanged. The
      companion tokens are gone from the bootstrap.
15. 2026-11-25: with P4, the host holds no buffer object. So the engines' resize hooks find a
    resizable buffer's memory by its pointer, length and maximum (`genet-wasm` `memory.rs`,
    `resizable_memory`).
    - Boa's hook can't read the maximum.
    - Empty memories all share wasmi's dangling pointer. When two empty resizable buffers are live,
      resizing the older one resizes the newer one's memory.
    - Telling them apart needs the hook to carry a key for each buffer, which means a change in
      both forks.
16. 2026-11-25: P4's held counts, after a forced collection:
    - Modules compiled, instantiated and dropped in rounds of 50, and fresh objects passed as
      externrefs in rounds of 50, both return to zero wrappers, buffers, handles and externs. The
      idle store resets once a round.
    - One object crossing 1000 times takes one slot.
    - An import that closes over its own instance is collected.
    - A buffer keeps its memory's bytes after the `Memory` and its instance are gone.
    - Externrefs that enter a long-lived instance stay until its group dies and the store resets
      (D6).

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
- 2026-11-25: P6 met (genet `f41dad83a30`).
  - wasmi's SIMD, relaxed SIMD and memory64 are on.
  - `v128` throws `TypeError` at the boundary.
  - `Memory` and `Table` take `address`, with BigInt limits for i64.
- 2026-11-25: P1 met in full (genet `e8d3da0f3a4`). `install_webassembly_from_call` gives realms made
  inside a native callback the object: iframes, and the document a top-level navigation opens.
  Iframe tests pass on both engines.
- 2026-11-25: P3 met in disk mode (genet `91e1cdd0b08`, `9f9e989a4dc`).
  - Streaming compile and instantiate run on both engines, in documents and dedicated workers.
  - Page tests cover the checks.
  - `wasm_webapi_{boa,nova}.json` record the directory in disk mode.
- 2026-11-25: P4 met (genet `07f220b41c7`).
  - Handles are released through a `FinalizationRegistry` and a free-list slab.
  - Import functions and externrefs are held by keepers in script, and buffers are recorded rather
    than held.
  - The wasmi store resets once nothing is live.
  - Conformance passes, 38 tests on Vano and 36 on Bole.
  - `wasm/jsapi` and `wasm/webapi` match their expectations on both engines, apart from Bole's
    known `js-string/constants` worker timing.
- 2026-11-26: Vano's two-cell `FinalizationRegistry` fix (Vano `ed1e0e5e`, `0fd6b2c6`). The
  bootstrap's companion tokens are removed. Conformance passes, 38 tests on Vano and 36 on Bole, and
  `wasm/jsapi` and `wasm/webapi` on Nova match their expectations.
