// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! W1 and W2 of `design_docs/2026-11-22_webassembly_object_plan.md`: a module called
//! from script on Vano, with script called back from the module, and memory the
//! two share.
#![cfg(all(target_pointer_width = "64", feature = "wasm"))]

mod common;

use common::{engine_with, run};
use script_engine_api::ScriptEngine;

const CALLS_BACK: &str = r#"(module
  (import "env" "add1" (func $add1 (param i32) (result i32)))
  (func (export "twice") (param i32) (result i32)
    local.get 0 call $add1 i32.const 2 i32.mul))"#;

#[test]
fn an_export_calls_an_imported_script_function() {
    let mut engine = engine_with(CALLS_BACK);
    let out = run(
        &mut engine,
        "const seen = [];
         const env = { add1: (x) => { seen.push(typeof x + ' ' + x); return x + 1; } };
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes), { env });
         return [exports.twice(20), exports.twice(-4.7), seen.join(','), exports.twice.length].join('|');",
    );
    assert_eq!(out, "42|-6|number 20,number -4|1");
}

#[test]
fn i64_crosses_as_bigint_both_ways() {
    let mut engine = engine_with(
        r#"(module
          (import "env" "echo" (func $echo (param i64) (result i64)))
          (func (export "inc") (param i64) (result i64)
            local.get 0 i64.const 1 i64.add call $echo))"#,
    );
    let out = run(
        &mut engine,
        "let got;
         const env = { echo: (x) => { got = x; return x * 2n; } };
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes), { env });
         const r = exports.inc(2n ** 40n);
         const first = got;
         const wrap = exports.inc(2n ** 63n - 1n);
         let number;
         try { exports.inc(1); } catch (e) { number = e.name; }
         return [typeof first, first, typeof r, r, got, wrap, number].join('|');",
    );
    assert_eq!(
        out, "bigint|1099511627777|bigint|2199023255554|-9223372036854775808|0|TypeError",
        "2^63 - 1 + 1 wraps to -2^63 in wasm; doubled in script it wraps to 0 on the way back"
    );
}

#[test]
fn compile_link_and_runtime_errors_have_their_classes() {
    let mut engine = engine_with(
        r#"(module
          (import "env" "f" (func))
          (func (export "boom") unreachable))"#,
    );
    let out = run(
        &mut engine,
        "const W = WebAssembly;
         const names = [];
         try { new W.Module(new Uint8Array([0, 97, 115, 109, 9])); }
         catch (e) { names.push(e instanceof W.CompileError && e instanceof Error && e.name); }
         try { new W.Instance(new W.Module(bytes), { env: {} }); }
         catch (e) { names.push(e instanceof W.LinkError && e.name); }
         const { exports } = new W.Instance(new W.Module(bytes), { env: { f() {} } });
         try { exports.boom(); }
         catch (e) { names.push(e instanceof W.RuntimeError && e.name); }
         return names.join(',');",
    );
    assert_eq!(out, "CompileError,LinkError,RuntimeError");
}

#[test]
fn a_runaway_module_is_cut_off_and_the_engine_survives() {
    let mut engine = engine_with(
        r#"(module
          (func (export "spin") (loop br 0))
          (func (export "one") (result i32) i32.const 1))"#,
    );
    engine.set_wasm_fuel(Some(1_000_000)).unwrap();
    let out = run(
        &mut engine,
        "globalThis.inst = new WebAssembly.Instance(new WebAssembly.Module(bytes));
         try { inst.exports.spin(); return 'returned'; }
         catch (e) { return e.name + ': ' + e.message; }",
    );
    assert_eq!(out, "RuntimeError: fuel exhausted");
    assert_eq!(run(&mut engine, "return 6 * 7;"), "42");
    assert_eq!(
        run(&mut engine, "return inst.exports.one();"),
        "threw RuntimeError: fuel exhausted",
        "exhausted fuel stays exhausted until the host sets it again"
    );
    engine.set_wasm_fuel(Some(1_000_000)).unwrap();
    assert_eq!(run(&mut engine, "return inst.exports.one();"), "1");
}

#[test]
fn script_called_from_wasm_can_call_back_into_the_same_instance() {
    let mut engine = engine_with(
        r#"(module
          (import "env" "back" (func $back (param i32) (result i32)))
          (func (export "f") (param i32) (result i32)
            local.get 0 i32.eqz
            if (result i32) i32.const 100
            else local.get 0 i32.const 1 i32.sub call $back i32.const 1 i32.add end))"#,
    );
    let out = run(
        &mut engine,
        "let inst;
         const env = { back: (n) => inst.exports.f(n) };
         inst = new WebAssembly.Instance(new WebAssembly.Module(bytes), { env });
         return inst.exports.f(5);",
    );
    assert_eq!(out, "105");
}

#[test]
fn an_import_exception_passes_through_and_the_store_survives() {
    let mut engine = engine_with(CALLS_BACK);
    let out = run(
        &mut engine,
        "const thrown = { marker: 1 };
         let fail = true;
         const env = { add1: (x) => { if (fail) throw thrown; return x + 1; } };
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes), { env });
         let same;
         try { exports.twice(1); } catch (e) { same = e === thrown; }
         fail = false;
         return [same, exports.twice(1)].join('|');",
    );
    assert_eq!(out, "true|4");
}

#[test]
fn a_start_function_calls_script_during_instantiation() {
    let mut engine = engine_with(
        r#"(module
          (import "env" "hello" (func $hello (param i32)))
          (func $start i32.const 7 call $hello)
          (start $start)
          (func (export "after") (result i32) i32.const 9))"#,
    );
    let out = run(
        &mut engine,
        "const seen = [];
         const env = { hello: (x) => { seen.push(x); } };
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes), { env });
         return [seen.join(), exports.after(), Object.keys(exports).join()].join('|');",
    );
    assert_eq!(out, "7|9|after");
}

#[test]
fn an_import_reached_without_a_wasm_frame_returns_its_result() {
    let mut engine = engine_with(
        r#"(module
          (import "env" "id" (func $id (param i32) (result i32)))
          (export "reexported" (func $id))
          (func (export "tail") (param i32) (result i32) local.get 0 return_call $id))"#,
    );
    let out = run(
        &mut engine,
        "const env = { id: (x) => x * 3 };
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes), { env });
         return [exports.reexported(2), exports.tail(5)].join('|');",
    );
    assert_eq!(out, "6|15");
}

#[test]
fn the_namespace_and_its_promises() {
    let mut engine = engine_with(r#"(module (func (export "seven") (result i32) i32.const 7))"#);
    engine
        .eval(
            "globalThis.result = 'pending';
             WebAssembly.instantiate(bytes).then(({ module, instance }) => {
               globalThis.result = [
                 instance.exports.seven(),
                 module instanceof WebAssembly.Module,
                 WebAssembly.validate(bytes),
                 WebAssembly.validate(new Uint8Array([1, 2])),
                 Object.prototype.toString.call(WebAssembly),
                 Object.keys(globalThis).includes('WebAssembly'),
                 typeof __genetWasmCompile,
               ].join('|');
             });",
        )
        .unwrap();
    engine.pump_microtasks();
    assert_eq!(
        run(&mut engine, "return result;"),
        "7|true|true|false|[object WebAssembly]|false|undefined"
    );
}

#[test]
fn a_snapshot_clone_is_refused_while_wasm_objects_live() {
    let mut engine = engine_with(r#"(module)"#);
    assert!(
        engine.snapshot_clone().is_ok(),
        "installed but unused clones"
    );
    engine
        .eval("globalThis.m = new WebAssembly.Module(bytes);")
        .unwrap();
    let refusal = engine
        .snapshot_clone()
        .err()
        .expect("a live module refuses the clone");
    assert!(refusal.contains("WebAssembly"), "{refusal}");
}

const MEMORY: &str = r#"(module
  (memory (export "mem") 1 4)
  (func (export "store") (param i32 i32) local.get 0 local.get 1 i32.store8)
  (func (export "load") (param i32) (result i32) local.get 0 i32.load8_u)
  (func (export "grow") (param i32) (result i32) local.get 0 memory.grow))"#;

#[test]
fn an_exported_memory_buffer_is_the_linear_memory() {
    let mut engine = engine_with(MEMORY);
    let out = run(
        &mut engine,
        "const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes));
         const m = exports.mem;
         const view = new Uint8Array(m.buffer);
         exports.store(5, 77);
         const wasmToScript = view[5];
         view[6] = 99;
         const scriptToWasm = exports.load(6);
         new Float64Array(m.buffer)[1] = 1.5;
         let transfer;
         try { m.buffer.transfer(); } catch (e) { transfer = e.name; }
         return [wasmToScript, scriptToWasm, exports.load(15), m.buffer === m.buffer,
                 m.buffer.byteLength, m instanceof WebAssembly.Memory, exports.mem === m,
                 transfer].join('|');",
    );
    assert_eq!(out, "77|99|63|true|65536|true|true|TypeError");
}

#[test]
fn growth_from_wasm_detaches_the_old_buffer() {
    let mut engine = engine_with(MEMORY);
    let out = run(
        &mut engine,
        "const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes));
         const m = exports.mem;
         exports.store(5, 77);
         const old = m.buffer;
         const before = exports.grow(1);
         const now = m.buffer;
         return [before, old.byteLength, now.byteLength, now !== old,
                 new Uint8Array(now)[5], exports.grow(9)].join('|');",
    );
    assert_eq!(out, "1|0|131072|true|77|-1");
}

#[test]
fn growth_from_script_detaches_even_by_zero_pages() {
    let mut engine = engine_with(MEMORY);
    let out = run(
        &mut engine,
        "const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes));
         const m = exports.mem;
         const first = m.buffer;
         const grew = m.grow(1);
         const second = m.buffer;
         const secondLength = second.byteLength;
         const zero = m.grow(0);
         const third = m.buffer;
         let failed;
         try { m.grow(5); } catch (e) { failed = e.name; }
         return [grew, first.byteLength, secondLength, second.byteLength, zero,
                 third.byteLength, third !== second, failed, m.buffer === third,
                 exports.load(131071)].join('|');",
    );
    assert_eq!(out, "1|0|131072|0|2|131072|true|RangeError|true|0");
}

#[test]
fn a_memory_made_by_script_is_imported_and_exported_as_itself() {
    let mut engine = engine_with(
        r#"(module
          (import "env" "mem" (memory 1))
          (export "mem" (memory 0))
          (func (export "store") (param i32 i32) local.get 0 local.get 1 i32.store8)
          (func (export "grow") (param i32) (result i32) local.get 0 memory.grow))"#,
    );
    let out = run(
        &mut engine,
        "const mem = new WebAssembly.Memory({ initial: 1, maximum: 3 });
         const view = new Uint8Array(mem.buffer);
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes), { env: { mem } });
         exports.store(100, 42);
         const throughView = view[100];
         const old = mem.buffer;
         exports.grow(1);
         let notMemory;
         try { new WebAssembly.Instance(new WebAssembly.Module(bytes), { env: { mem: {} } }); }
         catch (e) { notMemory = e.name; }
         return [exports.mem === mem, throughView, old.byteLength, mem.buffer.byteLength,
                 new Uint8Array(mem.buffer)[100], notMemory].join('|');",
    );
    assert_eq!(out, "true|42|0|131072|42|LinkError");
}

#[test]
fn memory_moved_under_a_suspended_call_is_seen_by_both_sides() {
    let mut engine = engine_with(
        r#"(module
          (import "env" "growFromScript" (func $grow))
          (import "env" "check" (func $check))
          (memory (export "mem") 1 4)
          (func (export "growThenWrite") (result i32)
            call $grow
            i32.const 70000 i32.const 5 i32.store8
            i32.const 70000 i32.load8_u)
          (func (export "growThenCall")
            i32.const 1 memory.grow drop
            call $check))"#,
    );
    let out = run(
        &mut engine,
        "let inst, held, seenInImport;
         const env = {
           growFromScript: () => { inst.exports.mem.grow(1); },
           check: () => { seenInImport = [held.byteLength, inst.exports.mem.buffer.byteLength]; },
         };
         inst = new WebAssembly.Instance(new WebAssembly.Module(bytes), { env });
         const written = inst.exports.growThenWrite();
         const read = new Uint8Array(inst.exports.mem.buffer)[70000];
         held = inst.exports.mem.buffer;
         inst.exports.growThenCall();
         return [written, read, seenInImport.join('/')].join('|');",
    );
    assert_eq!(out, "5|5|0/196608");
}

#[test]
fn an_empty_memory_has_an_empty_buffer_until_it_grows() {
    let mut engine = engine_with(r#"(module)"#);
    let out = run(
        &mut engine,
        "const m = new WebAssembly.Memory({ initial: 0 });
         const empty = m.buffer.byteLength;
         m.grow(1);
         const limits = [];
         for (const d of [{ initial: 2, maximum: 1 }, { initial: 70000 }]) {
           try { new WebAssembly.Memory(d); } catch (e) { limits.push(e.name); }
         }
         return [empty, m.buffer.byteLength, limits.join()].join('|');",
    );
    assert_eq!(out, "0|65536|RangeError,RangeError");
}

#[test]
fn a_live_memory_refuses_a_snapshot_clone() {
    let mut engine = engine_with(r#"(module)"#);
    engine
        .eval("globalThis.m = new WebAssembly.Memory({ initial: 1 });")
        .unwrap();
    assert!(engine.snapshot_clone().is_err());
}
