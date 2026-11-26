// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `design_docs/archive_docs/2026-11-25/2026-11-22_webassembly_object_plan.md` on Vano: the shared
//! tests in `genet-wasm/conformance`, and the snapshot clone, which only Vano
//! has.
#![cfg(all(target_pointer_width = "64", feature = "wasm"))]

#[path = "../../genet-wasm/conformance/mod.rs"]
mod conformance;

#[path = "harness/mod.rs"]
mod harness;

use harness::Vano;
use script_engine_nova::NovaEngine;

conformance::conformance_tests!(Vano);

fn engine_with(wat: &str) -> NovaEngine {
    <Vano as conformance::Harness>::with_module(wat).0
}

use script_engine_api::ScriptEngine;

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

#[test]
fn a_live_memory_refuses_a_snapshot_clone() {
    let mut engine = engine_with(r#"(module)"#);
    engine
        .eval("globalThis.m = new WebAssembly.Memory({ initial: 1 });")
        .unwrap();
    assert!(engine.snapshot_clone().is_err());
}
