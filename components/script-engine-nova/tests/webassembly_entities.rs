// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! W4 of `design_docs/2026-11-22_webassembly_object_plan.md`: globals,
//! tables, reference values, multi-value, linked functions and type
//! reflection, on Vano.
#![cfg(all(target_pointer_width = "64", feature = "wasm"))]

mod common;

use common::{engine_with, run};

#[test]
fn globals_cross_both_ways() {
    let mut engine = engine_with(
        r#"(module
          (import "env" "g" (global $g i32))
          (import "env" "big" (global $big i64))
          (global $count (export "count") (mut i32) (i32.const 1))
          (func (export "read") (result i32) global.get $g)
          (func (export "bump") global.get $count i32.const 1 i32.add global.set $count)
          (func (export "big") (result i64) global.get $big))"#,
    );
    let out = run(
        &mut engine,
        "const g = new WebAssembly.Global({ value: 'i32' }, 41);
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes),
           { env: { g, big: 7n } });
         exports.bump();
         exports.count.value = exports.count.value + 10;
         exports.bump();
         let immutable;
         try { g.value = 1; } catch (e) { immutable = e.name; }
         return [exports.read(), exports.count.value, exports.count instanceof WebAssembly.Global,
                 exports.count === exports.count, typeof exports.big(), immutable,
                 JSON.stringify(exports.count.type())].join('|');",
    );
    assert_eq!(
        out,
        r#"41|13|true|true|bigint|TypeError|{"mutable":true,"value":"i32"}"#
    );
}

#[test]
fn a_table_holds_functions_with_their_identity() {
    let mut engine = engine_with(
        r#"(module
          (type $t (func (result i32)))
          (table (export "table") 2 funcref)
          (func $seven (export "seven") (result i32) i32.const 7)
          (elem (i32.const 0) $seven)
          (func (export "callAt") (param i32) (result i32) local.get 0 call_indirect (type $t)))"#,
    );
    let out = run(
        &mut engine,
        "const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes));
         const t = exports.table;
         const nine = new WebAssembly.Function({ parameters: [], results: ['i32'] }, () => 9);
         t.set(1, nine);
         const grew = t.grow(1);
         let out;
         try { t.get(5); } catch (e) { out = e.name; }
         return [t.get(0) === exports.seven, exports.callAt(0), exports.callAt(1), t.get(1) === nine,
                 grew, t.length, t.get(2), out, JSON.stringify(t.type())].join('|');",
    );
    assert_eq!(
        out,
        r#"true|7|9|true|2|3||RangeError|{"minimum":3,"element":"funcref"}"#
    );
}

#[test]
fn externref_keeps_the_script_value() {
    let mut engine = engine_with(
        r#"(module
          (func (export "id") (param externref) (result externref) local.get 0)
          (table (export "refs") 1 externref))"#,
    );
    let out = run(
        &mut engine,
        "const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes));
         const object = { marker: 1 };
         exports.refs.set(0, object);
         return [exports.id(object) === object, exports.id(null), exports.id(undefined),
                 exports.id('text'), exports.refs.get(0) === object].join('|');",
    );
    assert_eq!(out, "true|||text|true");
}

#[test]
fn multi_value_crosses_both_ways() {
    let mut engine = engine_with(
        r#"(module
          (import "env" "pair" (func $pair (result i32 i64)))
          (func (export "swap") (param i32 i64) (result i64 i32) local.get 1 local.get 0)
          (func (export "viaScript") (result i64 i32) (local $a i32) (local $b i64)
            call $pair local.set $b local.set $a local.get $b local.get $a))"#,
    );
    let out = run(
        &mut engine,
        "const env = { pair: () => [5, 6n] };
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes), { env });
         const [a, b] = exports.swap(1, 2n);
         const [c, d] = exports.viaScript();
         return [typeof a, a, b, c, d].join('|');",
    );
    assert_eq!(out, "bigint|2|1|6|5");
}

#[test]
fn a_wasm_function_imported_elsewhere_is_linked_and_checked() {
    let mut engine = engine_with(
        r#"(module
          (import "env" "f" (func $f (param i32) (result i32)))
          (export "again" (func $f))
          (func (export "useIt") (result i32) i32.const 4 call $f))"#,
    );
    let out = run(
        &mut engine,
        "const source = new WebAssembly.Instance(new WebAssembly.Module(bytes),
           { env: { f: (x) => x * 10 } });
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes),
           { env: { f: source.exports.useIt.length === 0 ? source.exports.again : null } });
         const wrongType = new WebAssembly.Function({ parameters: [], results: [] }, () => {});
         let mismatch;
         try { new WebAssembly.Instance(new WebAssembly.Module(bytes), { env: { f: wrongType } }); }
         catch (e) { mismatch = e.name; }
         return [exports.useIt(), exports.again === source.exports.again, mismatch].join('|');",
    );
    assert_eq!(out, "40|true|LinkError");
}

#[test]
fn one_function_exported_twice_is_one_script_function() {
    let mut engine = engine_with(
        r#"(module (func $f (result i32) i32.const 1) (export "a" (func $f)) (export "b" (func $f)))"#,
    );
    let out = run(
        &mut engine,
        "const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes));
         return [exports.a === exports.b, exports.a instanceof WebAssembly.Function,
                 JSON.stringify(exports.a.type())].join('|');",
    );
    assert_eq!(out, r#"true|true|{"parameters":[],"results":["i32"]}"#);
}

#[test]
fn imports_and_exports_keep_the_module_order() {
    let mut engine = engine_with(
        r#"(module
          (import "m" "g" (global i32))
          (import "m" "f" (func))
          (import "m" "t" (table 1 funcref))
          (memory (export "mem") 1)
          (global (export "glob") i32 (i32.const 0))
          (func (export "fn")))"#,
    );
    let out = run(
        &mut engine,
        "const m = new WebAssembly.Module(bytes);
         const imports = WebAssembly.Module.imports(m).map((i) => i.kind).join();
         const exports = WebAssembly.Module.exports(m).map((e) => e.name).join();
         return [imports, exports].join('|');",
    );
    assert_eq!(out, "global,function,table|mem,glob,fn");
}

#[test]
fn instantiating_a_module_reads_imports_at_once() {
    let mut engine = engine_with(r#"(module (import "m" "f" (func)))"#);
    let out = run(
        &mut engine,
        "const order = [];
         const imports = { get m() { order.push('m'); return { f() {} }; } };
         WebAssembly.instantiate(new WebAssembly.Module(bytes), imports);
         return order.join();",
    );
    assert_eq!(out, "m");
}
