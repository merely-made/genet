// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A `WebAssembly` global for scripts on Vano, backed by wasmi (Genet's
//! WebAssembly object plan, D2 and D4). Off unless the `wasm` feature is on and
//! the host calls [`NovaEngine::install_webassembly`].

mod bridge;
mod entities;
mod entities_natives;
mod host;
mod json;
mod memory;
mod memory_natives;
mod natives;
mod sections;

use std::cell::RefCell;
use std::rc::Rc;

use nova_vm::ecmascript::{Behaviour, InternalMethods, PropertyDescriptor, PropertyKey, RegularFn};
use nova_vm::engine::Bindable;

pub(crate) use host::WasmHost;
pub(crate) use memory_natives::resize_hook;

use super::{NovaEngine, ScriptEngine, create_named_host_function};

/// What `GenetHostHooks::get_host_data` hands the natives.
pub(crate) type WasmSlot = RefCell<Option<Rc<WasmHost>>>;

const BOOTSTRAP: &str = include_str!("bootstrap.js");

const NATIVES: [(&str, RegularFn, u32); 27] = [
    ("__genetWasmCompile", natives::compile, 1),
    ("__genetWasmDescribe", natives::describe, 1),
    ("__genetWasmInstantiate", natives::instantiate, 2),
    ("__genetWasmExportOf", natives::export_of, 3),
    ("__genetWasmSignature", natives::signature, 1),
    ("__genetWasmCall", natives::call, 2),
    ("__genetWasmTrapMessage", natives::trap_message, 0),
    ("__genetWasmResultAt", natives::result_at, 1),
    ("__genetWasmStage", natives::stage, 0),
    ("__genetWasmFuncNew", natives::func_new, 3),
    ("__genetWasmCustomSection", natives::custom_section, 3),
    ("__genetWasmMemoryNew", memory_natives::memory_new, 2),
    ("__genetWasmMemoryGrow", memory_natives::memory_grow, 2),
    ("__genetWasmMemoryBuffer", memory_natives::memory_buffer, 1),
    ("__genetWasmMemoryType", memory_natives::memory_type, 1),
    (
        "__genetWasmMemoryToFixed",
        memory_natives::memory_to_fixed,
        1,
    ),
    (
        "__genetWasmMemoryToResizable",
        memory_natives::memory_to_resizable,
        1,
    ),
    ("__genetWasmGlobalNew", entities_natives::global_new, 3),
    ("__genetWasmGlobalGet", entities_natives::global_get, 1),
    ("__genetWasmGlobalSet", entities_natives::global_set, 2),
    ("__genetWasmGlobalType", entities_natives::global_type, 1),
    ("__genetWasmTableNew", entities_natives::table_new, 4),
    ("__genetWasmTableGet", entities_natives::table_get, 2),
    ("__genetWasmTableSet", entities_natives::table_set, 3),
    ("__genetWasmTableGrow", entities_natives::table_grow, 3),
    ("__genetWasmTableLength", entities_natives::table_length, 1),
    ("__genetWasmTableType", entities_natives::table_type, 1),
];

impl NovaEngine {
    /// Give the main realm a `WebAssembly` global. Wasm calls draw on the fuel
    /// [`set_wasm_fuel`](Self::set_wasm_fuel) sets, unbounded until it is set.
    ///
    /// # Errors
    ///
    /// If the natives cannot be defined or the bootstrap throws.
    pub fn install_webassembly(&mut self) -> Result<(), String> {
        if self.hook_owner.wasm.borrow().is_none() {
            *self.hook_owner.wasm.borrow_mut() = Some(Rc::new(WasmHost::new()));
        }
        let mut out = Ok(());
        self.agent.run_in_realm(&self.realm, |agent, mut gc| {
            let global = agent.current_realm(gc.nogc()).global_object(agent).unbind();
            for (name, behaviour, length) in NATIVES {
                let func = create_named_host_function(
                    agent,
                    Behaviour::Regular(behaviour),
                    name,
                    length,
                    None,
                    gc.nogc(),
                );
                let key = PropertyKey::from_str(agent, name, gc.nogc()).unbind();
                let desc = PropertyDescriptor {
                    value: Some(func.unbind().into()),
                    writable: Some(true),
                    configurable: Some(true),
                    ..Default::default()
                };
                if global
                    .internal_define_own_property(agent, key, desc, gc.reborrow())
                    .is_err()
                {
                    out = Err(format!("could not install {name}"));
                }
            }
        });
        out?;
        self.eval(BOOTSTRAP).map(drop)
    }

    /// Set the fuel all wasm calls on this engine draw from; `None` is
    /// unbounded. A call that runs out throws a `WebAssembly.RuntimeError`, and
    /// fuel stays exhausted until this is called again.
    ///
    /// # Errors
    ///
    /// If `WebAssembly` has not been installed.
    pub fn set_wasm_fuel(&mut self, fuel: Option<u64>) -> Result<(), String> {
        match self.hook_owner.wasm.borrow().as_ref() {
            Some(host) => {
                host.set_budget(fuel);
                Ok(())
            },
            None => Err("WebAssembly is not installed".to_string()),
        }
    }

    /// A clone cannot share the donor's store, so a live wasm object refuses
    /// the clone; an installed but unused `WebAssembly` gets a fresh store.
    pub(super) fn wasm_clone_slot(&self) -> Result<Option<Rc<WasmHost>>, String> {
        match self.hook_owner.wasm.borrow().as_ref() {
            Some(host) if host.is_live() => {
                Err("cannot snapshot clone NovaEngine with live WebAssembly objects".to_string())
            },
            Some(_) => Ok(Some(Rc::new(WasmHost::new()))),
            None => Ok(None),
        }
    }
}
