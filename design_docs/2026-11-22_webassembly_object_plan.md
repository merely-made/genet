# WebAssembly object plan: a `WebAssembly` global for scripts, probed on Vano

**Status:** 2026-11-22. Written; nothing built. D1 to D3 ruled the same day. W0 is next.

**For:** pages that use WebAssembly. Neither of Genet's JavaScript engines runs it: Vano says of
itself "The engine does not support WebAssembly execution" and Bole has no `WebAssembly` object. Mark
asked, during Graphshell's guest runtime plan
(`graphshell/design_docs/graphshell_docs/implementation_strategy/2026-11-21_guest_runtime_plan.md`,
D11), whether an embedded wasm runtime could give them one. This plan is a probe that finds out on the
harder engine first, memory included.

**Not here:** a complete `WebAssembly` implementation; Bole; Genet built for a browser, where the
page's own `WebAssembly` is the backend (D2); streaming compilation, threads, exceptions, GC types,
JSPI; passing any count of the web platform tests.

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

## Decisions

Each decision keeps its evidence, what was offered, the answer in Mark's words, what follows, and its
outcome. D1 to D3 are in answer order.

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

## Open questions

1. How a `WebAssembly.Module` or a live instance behaves under `ScriptEngineSnapshot::snapshot_clone`,
   which clones an engine heap: cloned, or refused.
2. Where the wasm-facing glue lives: beside `script-runtime-api`'s other host classes, written once
   over the engine-neutral trait, or inside each backend. The trait has no way to make a number, a
   BigInt, an object or an ArrayBuffer today (Findings 2), which decides much of this.
3. Vano on wasm64 (its experimental browser lane) is a browser build, so D2 sends it to the page's
   object; whether that lane's worker can reach it is unchecked.

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

## Progress

- 2026-11-22: plan written from Mark's rulings D1 to D3 and a read of the three sources.
