// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! P4: what script drops is freed (the pages plan's D6). Wrappers release
//! their handles when collected, keepers hold what wasm refers to, buffers
//! keep their memory, and the store is rebuilt once nothing is live.

use super::Harness;

/// `(wrappers, buffers, handles, externs)`.
fn held<H: Harness>(engine: &mut H) -> (usize, usize, usize, usize) {
    let stats = engine.stats();
    (stats.wrappers, stats.buffers, stats.handles, stats.externs)
}

pub fn compiled_and_dropped_modules_stay_bounded<H: Harness>() {
    let mut engine = H::with_module(
        r#"(module (memory (export "m") 1) (func (export "f") (result i32) i32.const 1))"#,
    );
    engine.eval(
        "globalThis.spin = (n) => { for (let i = 0; i < n; i++) {
           const instance = new WebAssembly.Instance(new WebAssembly.Module(bytes));
           instance.exports.f();
           instance.exports.m.buffer.byteLength;
         } };",
    );
    for round in 0..4 {
        engine.eval("spin(50)");
        let (wrappers, buffers, ..) = held(&mut engine);
        assert!(
            wrappers > 0 && buffers > 0,
            "round {round}: held until collected"
        );
        engine.collect();
        assert_eq!(held(&mut engine), (0, 0, 0, 0), "round {round}");
    }
    assert_eq!(engine.stats().resets, 4, "each round left nothing live");
}

pub fn externrefs_in_a_loop_stay_bounded<H: Harness>() {
    let mut engine = H::with_module(
        r#"(module (func (export "id") (param externref) (result externref) local.get 0))"#,
    );
    engine.eval(
        "globalThis.spin = (n) => { let same = 0; for (let i = 0; i < n; i++) {
           const instance = new WebAssembly.Instance(new WebAssembly.Module(bytes));
           const object = { i };
           if (instance.exports.id(object) === object) same++;
         } return same; };",
    );
    for round in 0..4 {
        assert_eq!(engine.run("return spin(50);"), "50");
        assert!(
            engine.stats().externs > 0,
            "round {round}: held until collected"
        );
        engine.collect();
        assert_eq!(held(&mut engine), (0, 0, 0, 0), "round {round}");
    }
}

pub fn an_object_crossing_again_reuses_its_slot<H: Harness>() {
    let mut engine = H::with_module(
        r#"(module (func (export "id") (param externref) (result externref) local.get 0))"#,
    );
    let out = engine.run(
        "const { id } = new WebAssembly.Instance(new WebAssembly.Module(bytes)).exports;
         const object = {};
         let same = 0;
         for (let i = 0; i < 1000; i++) if (id(object) === object) same++;
         return same;",
    );
    assert_eq!(out, "1000");
    assert_eq!(engine.stats().externs, 1, "one slot for one object");
}

pub fn wasm_keeps_an_object_script_dropped<H: Harness>() {
    let mut engine = H::with_module(
        r#"(module (table (export "t") 1 externref)
             (func (export "get") (result externref) i32.const 0 table.get 0))"#,
    );
    engine.eval(
        "globalThis.exports = new WebAssembly.Instance(new WebAssembly.Module(bytes)).exports;
         exports.t.set(0, { tag: 'kept' });",
    );
    engine.collect();
    assert_eq!(engine.run("return exports.get().tag;"), "kept");
    engine.eval("delete globalThis.exports;");
    engine.collect();
    assert_eq!(
        held(&mut engine),
        (0, 0, 0, 0),
        "and frees it with the group"
    );
}

pub fn an_import_closing_over_its_own_instance_is_collected<H: Harness>() {
    let mut engine = H::with_module(
        r#"(module (import "env" "back" (func $back))
             (func (export "f") call $back))"#,
    );
    engine.eval(
        "globalThis.probe = (() => {
           let instance;
           const back = () => instance.exports.f;
           instance = new WebAssembly.Instance(new WebAssembly.Module(bytes), { env: { back } });
           return new WeakRef(instance.exports.f);
         })();",
    );
    engine.collect();
    assert_eq!(engine.run("return probe.deref() === undefined;"), "true");
    assert_eq!(held(&mut engine), (0, 0, 0, 0));
}

pub fn the_store_is_rebuilt_once_nothing_is_live<H: Harness>() {
    let mut engine = H::with_module(
        r#"(module (func (export "f") (result i32) i32.const 1)
             (func (export "spin") (loop br 0)))"#,
    );
    engine.eval(
        "globalThis.keep = new WebAssembly.Instance(new WebAssembly.Module(bytes));
         for (let i = 0; i < 30; i++) new WebAssembly.Instance(new WebAssembly.Module(bytes));",
    );
    engine.collect();
    assert_eq!(engine.stats().resets, 0, "one instance is still held");
    assert!(engine.stats().wrappers > 0);
    engine.set_fuel(Some(100_000));
    engine.eval("delete globalThis.keep;");
    engine.collect();
    assert_eq!(engine.stats().resets, 1);
    assert_eq!(held(&mut engine), (0, 0, 0, 0));
    let out = engine.run(
        "const { f, spin } = new WebAssembly.Instance(new WebAssembly.Module(bytes)).exports;
         let out = f();
         try { spin(); } catch (e) { out += ' ' + e.name + ': ' + e.message; }
         return out;",
    );
    assert_eq!(
        out, "1 RuntimeError: fuel exhausted",
        "wasm works, and the fuel carried over"
    );
}

pub fn a_buffer_holds_the_store_and_its_memory<H: Harness>() {
    let mut engine = H::with_module(
        r#"(module (memory (export "m") 1)
             (func (export "grow") (result i32) i32.const 1 memory.grow))"#,
    );
    engine.eval(
        "(() => {
           const { exports } = new WebAssembly.Instance(new WebAssembly.Module(bytes));
           globalThis.held = { buffer: exports.m.buffer, grow: exports.grow };
         })();
         new Uint8Array(held.buffer)[5] = 7;",
    );
    engine.collect();
    assert_eq!(engine.stats().resets, 0);
    assert_eq!(engine.stats().buffers, 1);
    assert_eq!(
        engine.run("return [held.buffer.byteLength, new Uint8Array(held.buffer)[5]].join();"),
        "65536,7",
        "the bytes outlive the memory object and the instance"
    );
    assert_eq!(
        engine.run("held.grow(); return held.buffer.byteLength;"),
        "0",
        "growth from wasm detaches a buffer whose memory object is gone"
    );
    engine.collect();
    assert_eq!(engine.stats().buffers, 0, "a detached buffer holds nothing");
    engine.eval("delete globalThis.held;");
    engine.collect();
    assert_eq!(held(&mut engine), (0, 0, 0, 0));
    assert_eq!(engine.stats().resets, 1);
}
