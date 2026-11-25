# WebAssembly object plan: a `WebAssembly` global for scripts, probed on Vano

**Status:** 2026-11-25. W0 to W3 are met; W4 and W5 wait on the open lanes (D10). The probe
runs modules from script on Vano behind `genet-vano`'s `wasm` feature (Findings 25 to 31), with
`Memory.buffer` as the module's linear memory through Vano `c2c0eaa4` (Findings 32 to 37). It
passes 360 of 777 subtests of the tree's JS API tests (Findings 40). D8 and D9 grow it to what
wasmi can reach, on Vano and then on Bole.

**For:** pages that use WebAssembly. Neither of Genet's JavaScript engines runs it: Vano says of
itself "The engine does not support WebAssembly execution" and Bole has no `WebAssembly` object. Mark
asked, during Graphshell's guest runtime plan
(`graphshell/design_docs/graphshell_docs/implementation_strategy/2026-11-21_guest_runtime_plan.md`,
D11), whether an embedded wasm runtime could give them one. This plan is a probe that finds out on the
harder engine first, memory included.

**Not here:** Genet built for a browser, where the page's own `WebAssembly` is the backend (D2);
streaming compilation, threads, exceptions, GC types, JSPI and ES module integration (D8); passing
any count of the web platform tests. Bole came in with D9 (W5).

## Constraints

- The probe is native. An older note stands for browser builds: Genet's browser target "bars a nested
  JIT and defers to the host `WebAssembly` object" (`docs/2026-06-24_formal_web_lessons.md:46`). D2
  keeps it.
- Vano is ours to change; changes to it are additive and named here before they are made.
- `script-engine-api` is engine-neutral. Anything the probe adds to the trait is added for every
  backend or kept out of the trait; a backend that cannot honour it says so, it does not pretend.
- The spec is the guide: behaviour follows the WebAssembly JavaScript Interface, and the yardstick is
  the web platform tests already in the tree (`tests/wpt/tests/wasm/jsapi`), none of which Genet runs
  today.
- A wasm module from a page is untrusted. It runs under the same script budget the page's JavaScript
  does, through the runtime's own fuel, and a runaway module is a tested case.
- Conclusions are scoped to what ran.

## Phases

### W0. What Vano lacks, written against its source (gate)

**Done-conditions:** for each of the three gaps in Findings 3 to 5, the files and types a change
would touch are named, with the smallest additive change that closes it; whether Vano's upstream
(Nova) has moved on any of them is checked and recorded; the changes come back to Mark as forks where
more than one shape is defensible.

### W1. A module called from script, on Vano

**Done-conditions:** from a script running on Vano in a native test, `new WebAssembly.Module(bytes)`
and `new WebAssembly.Instance(module, imports)` work for a module with one imported JavaScript
function and one exported function; calling the export with numbers returns the right number, and the
import is called with the right arguments; an `i64` crosses as a BigInt in both directions; a module
that does not validate throws `WebAssembly.CompileError`, a missing import throws
`WebAssembly.LinkError`, and a trap throws `WebAssembly.RuntimeError`; a module that loops forever is
cut off by the script budget and the engine survives it.

### W2. Memory that the script and the module share

**Done-conditions:** an exported `WebAssembly.Memory`'s `buffer` is the module's linear memory, not a
copy: a byte the module writes is read by the script through a typed array with no call between, and
a byte the script writes is read by the module; after `memory.grow` the old `buffer` is detached and
the new one has the new length, as the spec requires; the same holds for a `Memory` the script
creates and imports. If Vano cannot be made to do this additively, the plan says exactly why and what
a copy-based stand-in gets wrong, and that is the result.

### W3. Readout

**Done-conditions:** the changes made to Vano and to `script-engine-api`, listed; one run of
`tests/wpt/tests/wasm/jsapi` on Vano with the pass count recorded, as a measure of the distance left
and not a target; what Bole would need, from Findings 6; the questions put to Mark (whether to
implement the object in full, on which engines, and in what order against Genet's other spec gaps).

### W4. The reachable set on Vano (D8, D10)

Starts once the scripting and layout lanes already open have landed (D10).

**Done-conditions:** behind the same feature, on Vano:
- `WebAssembly.Global` and `WebAssembly.Table`;
- every import and export kind, including a wasm function imported from another instance, which
  is linked directly and fails with `LinkError` on a type mismatch;
- multi-value results in both directions, and `externref` and `funcref` values;
- one script function per wasm function, however many names it is exported under;
- `toResizableBuffer` and `toFixedLengthBuffer`, with the Vano change they need named here first;
- type reflection (`type()` and `WebAssembly.Function`).

The runner from W3 is run again and its count recorded, as a measure and not a target.
Exceptions, GC types, promise integration and ES module integration are not in W4.

### W5. The object on Bole (D9)

**Done-conditions:** the W1, W2 and W4 tests pass on Bole as they do on Vano, through the same
bootstrap; the Bole fork change of D6's shape is named here before it is made; whether wasm fuel
shares Bole's script budget is put to Mark; the runner's count on Bole is recorded.

## Decisions

Each decision keeps its evidence, what was offered, the answer in Mark's words, what follows, and its
outcome. D1 to D10 are in answer order.

- **D1. Vano first (Mark, 2026-11-22):** Evidence: Bole's native functions can carry captured state,
  it is the default engine in Genet's builds and tools, and it builds for wasm32; Vano's native
  functions are bare function pointers with no data slot, so each wasm function exposed to script
  needs a workaround or a change to the fork, and Vano is 64-bit only; neither engine can make an
  ArrayBuffer over memory it does not own. Offered: Bole first (recommended); Vano first; both in the
  probe. Answer: "Vano first". What follows: the probe runs on Vano and begins with what Vano lacks
  (W0); Bole is assessed in the readout. Outcome: new.

  Links: area Scripting · plans webassembly-object W0, webassembly-object W1 · summary The WebAssembly object is probed on Vano first, starting with the changes the fork needs.

- **D2. wasmi natively; the page's own `WebAssembly` in a browser (Mark, 2026-11-22):** Evidence:
  Genet's browser target already defers to the host object (`docs/2026-06-24_formal_web_lessons.md:46`);
  for native builds nothing was decided; in Graphshell's probes wasmi ran a 50 million turn loop
  natively in about 120 ms against 336 ms for Wasmtime's interpreter, it translates a module from
  bytes itself, where Wasmtime's interpreter needs a compiler a page's wasm cannot assume, and it is a
  small pure-Rust dependency with fuel. Offered: wasmi natively, the host's own in a browser
  (recommended); wasmi everywhere; Wasmtime natively. Answer: "wasmi natively, the host's own in a
  browser (Recommended)". What follows: the probe embeds wasmi behind a seam a browser build can
  fill differently, and builds only the native half. Outcome: new; keeps the 2026-06-24 note.

  Links: area Scripting · plans webassembly-object W1 · summary Native Genet backs WebAssembly with wasmi; Genet in a browser passes through to the page's own.

- **D3. The probe includes memory aliasing (Mark, 2026-11-22):** Evidence: `WebAssembly.Memory`'s
  `buffer` must alias the module's linear memory, and both engines' ArrayBuffers own their bytes, so a
  probe either copies, which is wrong once a script writes, or changes a fork; the tree holds 805 web
  platform wasm test files, none enabled. Offered: the smallest probe, a call into wasm and back
  (recommended); that plus a first test count; include memory aliasing. Answer: "Include memory
  aliasing". What follows: phase W2, and the fork change it needs is in scope; the test count moves to
  the readout. Outcome: new.

  Links: area Scripting · plans webassembly-object W2, webassembly-object W3 · summary The probe makes Memory.buffer alias wasm linear memory in Vano, which means changing how the fork's ArrayBuffer holds its bytes.

- **D4. The glue lives inside `genet-vano` (Mark, 2026-11-25):** Evidence: the trait's call context
  cannot make a number, a BigInt, an object or an ArrayBuffer (Findings 2), so glue written once
  over it means growing the trait for every backend; the bridge holds the Vano agent across a
  suspended wasm call (Findings 21). Offered: inside `genet-vano` behind an off-by-default feature
  (recommended); over the trait; engine-neutral wasmi handling in its own crate with the bridging
  in `genet-vano`. Answer: "Inside genet-vano (Recommended)". What follows: the wasmi store and the
  rooted callbacks live with the Vano agent behind a feature that is off by default;
  `script-engine-api` does not change. Outcome: new; answers open question 2.

  Links: area Scripting · plans webassembly-object W1, webassembly-object W2 · summary The WebAssembly glue and its wasmi store live inside genet-vano behind an off-by-default feature; the engine-neutral trait is unchanged.

- **D5. Exports are bootstrap closures first; a Vano slot only if they fall short (Mark,
  2026-11-25):** Evidence: Vano's builtins carry a bare `fn` and no data (Findings 11); a `u64` slot
  is a small fork change (Findings 12); a closure over one trampoline needs none and differs from
  the spec in `toString`, `name` and `length` (Findings 13). Offered: the bootstrap closure
  (recommended); the `u64` slot; the closure first, the slot if needed. Answer: "Closure first, slot
  if needed". What follows: W1 exposes exports as closures branded in a `WeakMap`; the slot is added
  only if W1 or W3's test count shows the closure falls short, and is named here first. Outcome: new.

  Links: area Scripting · plans webassembly-object W1, webassembly-object W3 · summary Wasm exports reach Vano scripts as bootstrap closures over one trampoline; a host slot on Vano builtins waits on evidence that closures fall short.

- **D6. A non-owning `DataBlock` for `Memory.buffer` (Mark, 2026-11-25):** Evidence: Vano's
  `DataBlock` owns and frees its bytes (Findings 14); wasmi's grow can move them, so a stale buffer
  is a use after free (Findings 16); alignment is not promised (Findings 15). Offered: a non-owning
  block through an `unsafe` constructor, alignment checked (recommended); a counted backing handle
  in JavaScriptCore's shape; a copy at each boundary. Answer: "Non-owning DataBlock (Recommended)".
  What follows: W2 adds a `DataBlock` that does not free on drop and
  `ArrayBuffer::new_over_foreign(ptr, len, detach_key)`, which refuses a pointer not aligned to 8;
  the bridge detaches the old buffer at every wasm-to-script transition after a grow. Outcome: new;
  the fork change D3 foresaw.

  Links: area Scripting · plans webassembly-object W2 · summary Vano gains a non-owning ArrayBuffer backing over wasm memory, detached by the bridge before script runs after any grow.

- **D7. Growth by zero pages is accepted and recorded (Mark, 2026-11-25):** Evidence: wasmi reports
  neither form of a zero grow, so its old buffer is not detached as the spec requires; no bytes
  move, and no test in the tree sees it (Findings 19, 20). Offered: accept and record (recommended);
  a pull request to wasmi; a wasmi fork in the org. Answer: "Accept and record (Recommended)". What
  follows: the probe tracks grows that add pages, and a grow of zero from script detaches through
  our own code; W3 records the wasm-side gap. Outcome: new.

  Links: area Scripting · plans webassembly-object W2, webassembly-object W3 · summary A wasm-side memory.grow of zero leaves the old buffer attached under wasmi; the probe records the gap rather than changing wasmi.

- **D8. Grow the object to the reachable set (Mark, 2026-11-25):** Evidence: 360 of 777 subtests
  pass on Vano. 652 are within today's design, and `Global` and `Table` are 206 of the 308 left.
  125 wait on proposals wasmi does not implement or on ES module integration (Findings 40).
  Offered: grow to the reachable set (recommended); stop at the probe; take the blocked proposals
  on as well. Answer: "Grow to the reachable set (Recommended)". What follows: phase W4. Exception
  handling, GC types, promise integration and ES module integration stay out. Outcome: new;
  answers open question 4 in part.

  Links: area Scripting · plans webassembly-object W4 · summary Genet's WebAssembly object grows to everything wasmi and today's design can reach: Global, Table, every import and export kind, multi-value, reference types, resizable buffers and type reflection.

- **D9. Vano, then Bole (Mark, 2026-11-25):** Evidence: the object exists on Vano. Bole is the
  default engine in Genet's builds, worker and test runner (Findings 9). It would need the natives
  rewritten and an `ArrayBuffer` change of D6's shape, but its native functions already carry
  state (Findings 42). Offered: Vano, then Bole (recommended); Vano only; Bole in parallel now.
  Answer: "Vano, then Bole (Recommended)". What follows: W4 on Vano, then W5 on Bole. Outcome: new;
  answers open question 4 in part.

  Links: area Scripting · plans webassembly-object W4, webassembly-object W5 · summary The WebAssembly object is finished on Vano first and then ported to Bole, so default builds get it too.

- **D10. After the open lanes (Mark, 2026-11-25):** Evidence: W0 to W3 are met, and the work left
  is W4 and W5. Offered: after the scripting and layout lanes already open (recommended); next in
  scripting; when a page needs it. Answer: "After the open lanes (Recommended)". What follows: W4
  starts once the lanes already in flight have landed, and no lane is paused for it. Outcome: new;
  answers open question 4.

  Links: area Scripting · plans webassembly-object W4 · summary The rest of the WebAssembly object waits until Genet's scripting and layout lanes already in flight have landed.

## Open questions

1. Answered by evidence 2026-11-25 (Findings 23): `snapshot_clone` refuses an engine that holds a
   live wasm object, since the store cannot be cloned beside the heap.
2. Answered 2026-11-25 (D4): inside `genet-vano`, behind an off-by-default feature.
3. Vano on wasm64 (its experimental browser lane) is a browser build, so D2 sends it to the page's
   object; whether that lane's worker can reach it is unchecked.
4. Answered 2026-11-25 (D8 to D10): grow to the reachable set, on Vano and then Bole, after the open
   lanes.

## Findings

Read 2026-11-22 from the three repositories' sources; nothing was run.

1. Genet hosts each engine behind `script-engine-api`'s `ScriptEngine` trait
   (`components/script-engine-api/lib.rs:264`): `set_global` (`:422`), `set_function` over a
   zero-sized `NativeFn` that captures nothing (`:440`, `:897`), reflectors carrying a `u64`
   (`:32`, `:963`), and host promises by token (`:478`, `:489`). Backends are `genet-vano`
   (`components/script-engine-nova`) and `genet-bole` (`components/script-engine-boa`).
2. The trait's call context can read arguments as strings and numbers and make strings, null,
   undefined, reflectors and promises. It cannot make a number, a boolean, a BigInt, an object, an
   array, an ArrayBuffer or a function with state. Existing host classes pass numbers as strings and
   keep native state in a host-side table behind a JavaScript class in a bootstrap (the text decoder,
   `components/script-runtime-api/text_encoding.rs:162`).
3. Vano gap, functions: a builtin's behaviour is a bare `fn` pointer and its heap record has no data
   slot (`nova_vm/src/ecmascript/builtins/builtin_function.rs:376`, `:390`;
   `types/language/function/data.rs:41`), and the active function object is crate-private
   (`execution/agent.rs:1533`). A wasm export exposed as a function has nowhere to keep which export
   it is.
4. Vano gap, memory: an ArrayBuffer's bytes are a `DataBlock` the engine allocates and frees itself,
   with no constructor over outside memory (`types/spec/data_block.rs:56`, `:73`, `:151`). The only
   public constructor makes a zeroed buffer (`builtins/array_buffer.rs:52`); `detach` is public and
   `resize` is crate-private.
5. Vano has what the rest needs: BigInt to and from `i64` (`types/language/bigint.rs:169`, `:214`),
   promise capabilities the host can resolve (`promise_capability_records.rs:50`, `:159`), and
   `Global` roots that must be released by hand (`engine/rootable/global.rs:17`), which Genet's
   `NovaValue` already manages.
6. Bole, for the readout: native functions with traced captures exist
   (`core/engine/src/native_function/mod.rs:236`), classes register globally
   (`context/mod.rs:362`), and its ArrayBuffer also owns its bytes (`builtins/array_buffer/mod.rs:201`)
   but can be detached and rebuilt from a block it is handed (`object/builtins/jsarraybuffer.rs:103`,
   `:219`).
7. Earlier notes: Nova's lack of WebAssembly is called "a gated fullweb feature"
   (`docs/2026-05-20_genet_script_engine_plan.md:48`) and deferred
   (`docs/2026-06-10_nova_conformance_campaign_plan.md:159`). No plan or ruling in `design_docs/`
   covers a script-visible `WebAssembly`.
8. The web platform tests are in the tree at `tests/wpt/tests/wasm` (805 files; `jsapi` covers
   constructors, instances, memory, tables, globals and functions) with Servo's synced expectations;
   Genet's own expectation files have none for wasm. No Cargo manifest in Genet names wasmi or
   Wasmtime.
9. The two engines' standing is stated two ways in the tree: comments call Vano "the primary backend
   on 64-bit targets", while the `scripted` feature, the worker's default and the test runner's
   default are Bole.

W0, read 2026-11-25 from Vano `a4eff25b`, Genet `b558fae3326` and wasmi 2.0.3. Two small probes
ran outside the tree (`Code/output/genet-wasm-prior-art/`): wasmi on its own, with Rust standing in
for JavaScript, and Node 26.12 as the reference for the JS API. Nothing ran on Vano.

*Upstream.*

10. Nova has not moved on any of the three gaps. Our merge of `591bf7ba` is upstream's tip. Its open
    branches touch `builtin_function.rs` only for lifetimes and GC, not for data on a builtin.
    `data_block.rs` stays as it is. Nova once had a wasm crate and removed it (`c9a4344b`, #222).
    Its wasm interpreter issue (#48, 2023) has had no work since.

*Functions (Findings 3).*

11. The gap is as stated. A builtin's `Behaviour` is a bare `fn` (`builtin_function.rs:376`,
    `:390`). `BuiltinFunctionArgs` takes a length, a name, a realm, a prototype and a prefix, and
    nothing else (`:468`). GC tracing of the heap record names no other field (`:890`). The callee
    reaches a native call only through `active_function_object`, which is `pub(crate)`
    (`execution/agent.rs:1533`).
12. The smallest additive Vano change: a `u64` host slot on `BuiltinFunctionHeapData`, set through a
    new field of `BuiltinFunctionArgs`, and a public `Agent::active_builtin_host_data()`. A `u64`
    needs no tracing and matches Genet's reflector data (Findings 1).
13. A shape with no Vano change: a bootstrap closure over one native trampoline. It captures the
    export's handle as a number and is branded in a `WeakMap` so `Table.set` can recognise it. This
    is the text decoder's pattern (Findings 2). It costs an ECMAScript function where the spec has a
    built-in one. `toString` shows source, and `name` and `length` have to be defined by hand.

*Memory (Findings 4).*

14. The gap is as stated, and its edges are sharper. `DataBlock` (`types/spec/data_block.rs:56`)
    frees its pointer on drop at alignment 8 (`:73`). Clone copies into a fresh allocation (`:86`).
    Only `new` makes one, zeroed (`:155`). `ArrayBuffer::new` is the public way in
    (`builtins/array_buffer.rs:52`). `detach` and `set_detach_key` are public (`:115`, `:101`), so
    the `[[ArrayBufferDetachKey]]` that keeps script from transferring a memory's buffer is already
    reachable. `resize` is crate-private (`:127`).
15. Vano's typed-array access panics when a buffer is not aligned for its element type
    (`array_buffer.rs:182`, `:211`). wasmi allocates linear memory as `Layout::array::<u8>`, which
    guarantees alignment 1 only (`wasmi_core` `memory/buffer.rs:91`). The system allocator gives
    more in practice, but nothing promises it. A buffer over wasm memory must check its alignment
    when it is made, or wasmi must allocate aligned.
16. wasmi grows memory by `Vec::try_reserve` and `resize` (`buffer.rs:150`), so a successful grow
    can move the bytes. A buffer still pointing at the old bytes is a use after free, not only a
    spec fault. So the old buffer must be detached before any script runs after a growing call:
    at an import's entry, at an export's return, and at a trap.
17. The smallest additive Vano change: a non-owning `DataBlock` that does not free on drop. Clone
    keeps copying into an owned block. It is made through an `unsafe` public
    `ArrayBuffer::new_over_foreign(ptr, len, detach_key)` whose contract is that the owner detaches
    it before the bytes move or are freed. The other shape is JavaScriptCore's: the block holds a
    counted handle to the memory (Findings 22). That is safer about who frees the bytes, but since
    the bytes still move on grow it does not remove the detach. It is also a larger change to a type
    every buffer uses.
18. Out of W2 as written: `Memory.prototype.toResizableBuffer` and `toFixedLengthBuffer` are in the
    tree's tests (`tests/wpt/tests/wasm/jsapi/memory/`). A resizable buffer whose resize grows the
    memory would also need `resize` opened.

*Growth by zero pages (new).*

19. Every successful grow detaches the old buffer, zero pages included. Node confirms it at the
    first callback after a wasm `memory.grow 0`. wasmi translates a constant zero grow to
    `memory.size` (`wasmi` `engine/translator/func/visit.rs:732`). For a dynamic zero it returns
    before the resource limiter is consulted (`wasmi_core` `memory/mod.rs:184`, limiter at `:206`).
    So no hook wasmi offers sees either form. A grow counter misses both.
20. A zero grow moves no bytes, so this is spec conformance only, not safety (Findings 16). The JS
    side's `memory.grow(0)` goes through our own code and can detach correctly. The tree's `jsapi`
    tests grow only from script (`memory/grow.any.js`), so no test in the tree sees the gap. Closing
    it needs a wasmi change: our own fork, or a hook and an untranslated zero grow upstream.

*Calling script from wasm (new).*

21. wasmi's resumable calls give the bridge its shape. An import returns a marker error. The call
    suspends and the store borrow ends. The bridge calls the rooted JavaScript function with the
    Vano agent it already holds, then resumes with the result. Run with fuel on: two nested
    suspended calls on one store resumed in order, and dropping a suspended call left the store
    usable. That covers script that calls back into the same instance.
22. Two paths escape as plain errors. A root-frame tail call to an import is harmless: the export's
    result is the import's, so the bridge calls the import and returns. A start function is the
    real gap. wasmi runs it inside `instantiate` with a plain call (`module/instantiate/mod.rs:89`,
    `:115`), so an import there cannot be resumed. The fix is to rewrite the module before it
    compiles, the way Graphshell writes a memory maximum into a module (its plan's D5). Drop the
    start section, export the start function under a private name, then call it resumably after
    instantiation. The tree's tests set an imported JS function as the start function
    (`jsapi/functions/helper.js`), which then escapes at the root and completes as in the tail
    case.
23. Snapshot: `NovaEngine::snapshot_clone` gives the clone fresh host hooks and release queues and
    copies the heap whole (`components/script-engine-nova/lib.rs:1165`). A wasmi `Store` cannot be
    cloned, so a cloned `Memory` or exported function would point into the donor's store. The probe
    refuses the clone while the engine holds a live wasm object.

*Prior art.*

24. Other embedders have solved the same parts. OxiBrowser (Boa with wasmi) lends wasmi a raw
    thread-local pointer to its JS context and refuses calls back into the same instance. Its
    `Memory` exposes a snapshot of `byteLength`. txiki.js (QuickJS with WAMR) caches export wrappers
    by function identity. It checks for stale buffers before every imported callback and after every
    export, and rethrows an import's own JS value. JavaScriptCore keeps a handle to the memory's
    backing in its `ArrayBuffer`. A grow notification detaches the old buffer and the getter makes a
    new one lazily. Dialog DB's WCMP puts a component layer over wasmi with the same resumable
    technique. It is prior art for the layer Graphshell closed (its D13), not a reason to reopen it.

W1, run 2026-11-25 on this Mac with wasmi 2.0.3 and Vano `a4eff25b`. The code is
`components/script-engine-nova/native/wasm/` and the tests are
`components/script-engine-nova/tests/webassembly.rs`.

25. What landed. `genet-vano` has a `wasm` feature, off by default, that adds wasmi as its only
    dependency. `NovaEngine::install_webassembly` gives the main realm a `WebAssembly` global with
    `Module`, `Instance`, the three error classes, `validate`, `compile` and `instantiate`.
    `set_wasm_fuel` sets the budget. The namespace is a bootstrap script over six natives that it
    takes off the global object and deletes. Neither Vano nor `script-engine-api` changed.
26. The ten tests pass, and the crate's other suites pass with the feature on and with it off. They
    cover each done-condition:
    - an export calls an imported script function with the right arguments and returns the right
      number;
    - `i64` crosses as a BigInt both ways, wrapping at 64 bits, and a Number passed for one throws
      `TypeError`;
    - bad bytes throw `CompileError`, a missing import throws `LinkError` and `unreachable` throws
      `RuntimeError`;
    - an endless loop ends with `RuntimeError: fuel exhausted`, and the engine goes on evaluating.

    Four further cases test the bridge:
    - script called from wasm calls the same instance again, five calls deep;
    - an import's thrown object reaches the caller as the same object, and the instance works
      afterwards;
    - a start function calls script during `new WebAssembly.Instance` (Findings 22's rewrite);
    - an import exported directly, and one tail-called from the root frame, return their result.

    A snapshot clone is refused once a module exists and allowed before.
27. The budget is wasm's own. Vano has no way to bound script: `eval_bounded` with a step count is
    `Unsupported` there. So the fuel is a separate allotment the host sets, shared by every wasm
    call on the engine. Running out throws, and fuel stays exhausted until the host sets it again.
    Fuel stays on even when unbounded, at the cost Graphshell measured (its Findings 23).
28. Errors cross as strings. Vano lets an embedder throw only errors it constructs itself:
    `JsError::new` is crate-private and `throw_exception` takes a built-in error kind. So a native
    returns a message, and the bootstrap throws `CompileError`, `LinkError` or `RuntimeError`. An
    exception thrown by script passes through untouched, because the native returns the engine's
    own `JsError`.
29. Exports are arrow closures over one trampoline (D5). Each closure is cached per handle, so an
    export keeps its identity, and its `name` is the module's function index. Arrow closures are not
    constructors, as the spec requires. `toString` shows their source.
30. Not covered yet:
    - imports and exports of memories, tables and globals (W2 covers memory);
    - reference types and `v128` values;
    - multi-value results (they throw `TypeError`);
    - a wasm function passed as another instance's import, which crosses as a script function, so a
      type mismatch is not a `LinkError`;
    - one function exported under two names, which gives two script functions;
    - the promise forms, which resolve on the calling turn instead of a later task;
    - freeing: modules and instances live as long as the engine.
31. A start function's import has no wasm frame left to suspend when the start function is the
    import itself. The tree's tests make exactly that start function (`jsapi/functions/helper.js`),
    so the rewrite and the root-frame path between them cover both shapes.

W2, run 2026-11-25 on this Mac with wasmi 2.0.3 and Vano `c2c0eaa4`. The code is
`components/script-engine-nova/native/wasm/memory.rs`, with the transitions in `natives.rs`.

32. The fork change D6 named, Vano `c2c0eaa4`, is three parts:
    - a `DataBlock` can be foreign: it never frees its pointer, it asserts rather than reallocate,
      and a clone of it is an ordinary owned copy;
    - `ArrayBuffer::new_over_foreign(ptr, len, key)` makes a fixed-length buffer over such a block.
      It sets the detach key and refuses a pointer that is not aligned to 8, except at length zero,
      where no pointer is used. wasmi's empty memory has a pointer aligned to 1.
    - `DetachKey` is public. It was reachable through `set_detach_key` but could not be named.

    Vano's library tests pass (70). Its test262 suite was not run. The change leaves alone the
    host-hook and builtin-name contracts that `genet-vano`'s manifest names.
33. In Genet: `WebAssembly.Memory` with its constructor, `grow` and `buffer`; memory imports and
    exports; and one script object per memory. Bad limits throw `RangeError`, a non-number page
    count throws `TypeError`, and a non-`Memory` import throws `LinkError`.
34. The seven new tests pass. The crate's suites pass with the feature on and off, against the
    pushed Vano. Each of W2's done-conditions is covered:
    - A byte wasm writes is read through a `Uint8Array` with no call between, and a byte script
      writes is read by wasm. A `Float64Array` works over the buffer, the buffer is the same object
      each time, and `transfer()` throws `TypeError`.
    - After a grow from wasm the old buffer is detached and the new one has the new length and the
      old bytes. A failed grow returns -1.
    - After a grow from script the same holds, including for zero pages. A failed grow throws
      `RangeError` and leaves the buffer as it was.
    - A memory script creates and imports is the module's re-export of it, aliases both ways, and
      detaches when wasm grows it.
35. A buffer is kept beside the pointer and length it was made over. The bridge detaches any buffer
    whose memory no longer matches at four points: an import's entry; the end of every call, return
    or trap or thrown exception; a grow from script, which always detaches; and the `buffer`
    getter. Two cases show this is needed and holds:
    - Wasm grew memory and then called script. Inside the import, a buffer taken before the call
      was already detached.
    - Script grew memory inside an import while wasm was suspended. When wasm resumed, it wrote and
      read past the old end, and script read the byte through the new buffer, so a resumed frame
      sees the moved bytes.
36. An exported memory's identity comes from its index in the export section (Findings 22's
    scanner now keeps every export). An index below the imported count is the import, so it keeps
    the import's object. A defined memory exported twice gets one handle per instance and index.
37. Not covered yet:
    - a grow by zero pages from wasm, which leaves the old buffer attached (D7);
    - shared memory and 64-bit memory;
    - `toResizableBuffer` and `toFixedLengthBuffer` (Findings 18);
    - one instance's exported memory imported by another, which should work through the same
      handle but has no test;
    - freeing: memories and the latest buffer of each live as long as the engine.

    The alignment refusal never fired on this Mac, because its allocator returned 8-aligned memory
    every time.

W3, run 2026-11-25 on this Mac. The runner is `components/script-engine-nova/examples/wasm_jsapi.rs`.

38. What changed outside `genet-vano`: Vano `c2c0eaa4` only (Findings 32). `script-engine-api`
    did not change. `genet-vano` gained wasmi 2 as an optional dependency and `wat` for its tests.
39. How the count was taken. Each `.any.js` file runs on a fresh Vano engine with `WebAssembly`
    installed, in `testharness.js`'s shell environment: the tests' `jsshell` global, with no DOM,
    no timers and no fetch. Each file runs in its own process with a 60-second limit. 78 of the 80
    files declare that global; `idlharness.any.js` and one `esm-integration` file do not and were
    not run. Genet's own WPT runner (`ports/genet-wpt`) was not used: it runs the window variant
    and would have needed the feature plumbed through its runtime. Two runs gave the same count.
40. 360 of 777 subtests pass, a measure of distance and not a target. By directory:

    | Directory | Pass | Of | What fails |
    | --- | --- | --- | --- |
    | `constructor` | 137 | 163 | non-function imports; multi-value |
    | `instance` | 52 | 64 | non-function imports and exports |
    | `memory` | 48 | 72 | shared memory, resizable buffers, type reflection |
    | `module` | 30 | 38 | `customSections` (Findings 41); import and export kinds |
    | top level | 43 | 59 | the constructors below missing from the namespace |
    | `global` | 13 | 145 | no `WebAssembly.Global` |
    | `table` | 21 | 95 | no `WebAssembly.Table` |
    | `exception`, `tag` | 6 | 33 | no `Tag` or `Exception`; wasmi has no exception handling |
    | `function` | 0 | 16 | no `WebAssembly.Function` (type reflection, tentative) |
    | `js-string` | 7 | 41 | string builtins; the polyfill needs GC arrays, which wasmi lacks |
    | `jspi` | 3 | 21 | promise integration, which needs stack switching |
    | `esm-integration` | 0 | 30 | wasm as an ES module, through Genet's module loader |
    | `gc` | 0 | 0 | all three files fail to compile: wasmi has no GC types |

    The most common failure message, 231 times, is `'undefined' is not a constructor`, which is
    `Global`, `Table`, `Tag`, `Exception` and `Function` missing. Of the 777, 652 are in directories
    the present design can reach, type reflection included. 344 of those pass, and `Global` and
    `Table` account for 206 of the 308 left. The other 125 wait on proposals wasmi does not
    implement (exception handling, GC arrays, stack switching) or on loading wasm as an ES module.
41. A Vano bug the run found: `unescape("yee%F0%90%90%B7eey")` panics in `wtf8` (`begin <= end`,
    from `GlobalObject::unescape` at `global_object.rs:1479`). `wasm-module-builder.js` reaches it
    when it encodes a custom section's name, so `module/customSections.any.js` counts none of its
    subtests. Shorter strings with the same escapes do not panic.
42. What Bole would need, from Findings 6 and its source at `932470b6`:
    - The bootstrap carries over unchanged. The natives would be rewritten against Boa's API.
    - Its native functions carry traced captures (`from_copy_closure_with_captures`), so an export
      needs no closure trick and D5's question does not arise there.
    - Its `ArrayBuffer` owns an `AlignedVec<u8>` (`builtins/array_buffer/mod.rs:205`), the same gap
      Vano had. It would need a fork change of D6's shape. Its detach key is already an arbitrary
      `JsValue`.
    - Bole can bound script (`eval_bounded`), so wasm fuel could share the page's budget there, as
      this plan first intended (Constraints), where Vano could not (Findings 27).

## Progress

- 2026-11-22: plan written from Mark's rulings D1 to D3 and a read of the three sources.
- 2026-11-25: W0 met. The assessment is Findings 10 to 24; D4 to D7 ruled; open questions 1 and 2
  answered.
- 2026-11-25: W1 met (Findings 25 to 31).
- 2026-11-25: W2 met (Findings 32 to 37); Vano `c2c0eaa4` carries D6's change.
- 2026-11-25: W3 met. 360 of 777 subtests pass (Findings 38 to 42); D8 to D10 ruled and W4 and W5
  added.
