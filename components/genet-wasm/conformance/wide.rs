// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! P6: SIMD and 64-bit addresses. `v128` never crosses the boundary, and an
//! `address` of `"i64"` makes limits, deltas and sizes BigInts.

use super::Harness;

pub fn v128_throws_a_type_error_at_the_boundary<H: Harness>() {
    let mut engine = H::with_module(
        r#"(module
          (import "env" "vec" (func $vec (param v128)))
          (global (export "g") (mut v128) (v128.const i32x4 1 2 3 4))
          (func (export "id") (param v128) (result v128) local.get 0)
          (func (export "make") (result v128) v128.const i32x4 1 2 3 4)
          (func (export "callImport") v128.const i32x4 0 0 0 0 call $vec))"#,
    );
    let out = engine.run(
        "const caught = (f) => { try { f(); return 'no throw'; } catch (e) { return e.name; } };
         let called = false;
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes),
           { env: { vec: () => { called = true; } } });
         return [caught(() => exports.id()), caught(() => exports.make()),
                 caught(() => exports.callImport()), called,
                 caught(() => exports.g.value), caught(() => { exports.g.value = 1; }),
                 caught(() => new WebAssembly.Global({ value: 'v128' })),
                 exports.g.type().value].join('|');",
    );
    assert_eq!(
        out,
        "TypeError|TypeError|TypeError|false|TypeError|TypeError|TypeError|v128"
    );
}

pub fn simd_runs_inside_a_module<H: Harness>() {
    let mut engine = H::with_module(
        r#"(module
          (func (export "sum") (param i32) (result i32) (local $v v128)
            local.get 0 i32x4.splat
            v128.const i32x4 1 2 3 4
            i32x4.add
            local.set $v
            local.get $v i32x4.extract_lane 0
            local.get $v i32x4.extract_lane 1 i32.add
            local.get $v i32x4.extract_lane 2 i32.add
            local.get $v i32x4.extract_lane 3 i32.add))"#,
    );
    let out = engine.run(
        "const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes));
         return [exports.sum(10), WebAssembly.validate(bytes)].join('|');",
    );
    assert_eq!(out, "50|true");
}

pub fn an_i64_memory_uses_bigints<H: Harness>() {
    let mut engine = H::with_module(
        r#"(module
          (memory (export "mem") i64 1 4)
          (func (export "size") (result i64) memory.size)
          (func (export "grow") (param i64) (result i64) local.get 0 memory.grow))"#,
    );
    let out = engine.run(
        "const caught = (f) => { try { f(); return 'no throw'; } catch (e) { return e.name; } };
         const made = new WebAssembly.Memory({ address: 'i64', initial: 1n, maximum: 4n });
         const first = made.buffer;
         const old = made.grow(1n);
         const type = made.type();
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes));
         const mem = exports.mem;
         const grown = exports.grow(1n);
         const exported = mem.type();
         return [typeof old, old, first.byteLength, made.buffer.byteLength, type.address,
                 typeof type.minimum, type.minimum, type.maximum,
                 exports.size(), grown, mem.buffer.byteLength, exported.address, exported.minimum,
                 caught(() => made.grow(9n)), caught(() => made.grow(-1n)),
                 caught(() => new WebAssembly.Memory({ address: 'i64', initial: 5n, maximum: 4n })),
                 caught(() => new WebAssembly.Memory({ address: 'i64', initial: 1n << 64n })),
                 caught(() => new WebAssembly.Memory({ address: 'i16', initial: 1 })),
                 new WebAssembly.Memory({ initial: 1 }).type().address].join('|');",
    );
    assert_eq!(
        out,
        "bigint|1|0|131072|i64|bigint|2|4|2|1|131072|i64|2|RangeError|TypeError|RangeError|TypeError|TypeError|i32"
    );
}

pub fn an_i64_table_uses_bigints<H: Harness>() {
    let mut engine = H::with_module(
        r#"(module
          (table (export "table") i64 2 funcref)
          (func $seven (export "seven") (result i32) i32.const 7)
          (elem (i64.const 0) $seven))"#,
    );
    let out = engine.run(
        "const caught = (f) => { try { f(); return 'no throw'; } catch (e) { return e.name; } };
         const t = new WebAssembly.Table({ element: 'anyfunc', address: 'i64', initial: 2n, maximum: 5n });
         const old = t.grow(1n);
         const nine = new WebAssembly.Function({ parameters: [], results: ['i32'] }, () => 9);
         t.set(2n, nine);
         const type = t.type();
         const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes));
         return [typeof old, old, typeof t.length, t.length, t.get(2n) === nine, t.get(0n),
                 type.address, type.minimum, type.maximum, caught(() => t.get(9n)),
                 caught(() => t.grow(9n)),
                 exports.table.get(0n) === exports.seven, exports.table.length,
                 exports.table.type().address].join('|');",
    );
    assert_eq!(
        out,
        "bigint|2|bigint|3|true||i64|3|5|RangeError|RangeError|true|2|i64"
    );
}
