// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A `WebAssembly` global for scripts on Vano, backed by wasmi (Genet's
//! WebAssembly object plan, D2 and D4). Off unless the `wasm` feature is on and
//! the host calls [`NovaEngine::install_webassembly`].

mod bridge;
mod entities_natives;
mod memory_natives;
mod natives;

use std::cell::RefCell;
use std::rc::Rc;

use nova_vm::ecmascript::{Behaviour, InternalMethods, PropertyDescriptor, PropertyKey, RegularFn};
use nova_vm::engine::Bindable;
use script_engine_api::{MAIN_REALM, RealmId};

pub(crate) use memory_natives::resize_hook;

use super::{NovaEngine, NovaValue, ScriptEngine, create_named_host_function};

/// The engine's wasm state over Vano's rooted values.
pub(crate) type WasmHost = genet_wasm::WasmHost<NovaValue>;

/// What `GenetHostHooks::get_host_data` hands the natives.
pub(crate) type WasmSlot = RefCell<Option<Rc<WasmHost>>>;

/// The native behind each name the bootstrap expects (`genet_wasm::NATIVES`).
fn native(name: &str) -> RegularFn {
    match name {
        "Compile" => natives::compile,
        "Describe" => natives::describe,
        "Instantiate" => natives::instantiate,
        "ExportOf" => natives::export_of,
        "Signature" => natives::signature,
        "Call" => natives::call,
        "TrapMessage" => natives::trap_message,
        "ResultAt" => natives::result_at,
        "Stage" => natives::stage,
        "FuncNew" => natives::func_new,
        "CustomSection" => natives::custom_section,
        "MemoryNew" => memory_natives::memory_new,
        "MemoryGrow" => memory_natives::memory_grow,
        "MemoryBuffer" => memory_natives::memory_buffer,
        "MemoryType" => memory_natives::memory_type,
        "MemoryToFixed" => memory_natives::memory_to_fixed,
        "MemoryToResizable" => memory_natives::memory_to_resizable,
        "GlobalNew" => entities_natives::global_new,
        "GlobalGet" => entities_natives::global_get,
        "GlobalSet" => entities_natives::global_set,
        "GlobalType" => entities_natives::global_type,
        "TableNew" => entities_natives::table_new,
        "TableGet" => entities_natives::table_get,
        "TableSet" => entities_natives::table_set,
        "TableGrow" => entities_natives::table_grow,
        "TableLength" => entities_natives::table_length,
        "TableType" => entities_natives::table_type,
        other => unreachable!("genet-wasm names a native genet-vano lacks: {other}"),
    }
}

impl NovaEngine {
    /// Give the main realm a `WebAssembly` global. Wasm calls draw on the fuel
    /// [`set_wasm_fuel`](Self::set_wasm_fuel) sets, unbounded until it is set.
    ///
    /// # Errors
    ///
    /// If the natives cannot be defined or the bootstrap throws.
    pub fn install_webassembly(&mut self) -> Result<(), String> {
        self.install_webassembly_in_realm(MAIN_REALM)
    }

    /// [`install_webassembly`](Self::install_webassembly) on `realm`'s global,
    /// for a page whose scripts run in a child realm. The store is the
    /// engine's, shared by every realm that installs it.
    ///
    /// # Errors
    ///
    /// If the realm is unknown, the natives cannot be defined or the bootstrap
    /// throws.
    pub fn install_webassembly_in_realm(&mut self, realm: RealmId) -> Result<(), String> {
        if !self.registry.realms.borrow().contains_key(&realm) {
            return Err(format!("no such realm: {realm}"));
        }
        let registry = self.registry.clone();
        if self.hook_owner.wasm.borrow().is_none() {
            *self.hook_owner.wasm.borrow_mut() = Some(Rc::new(WasmHost::new()));
        }
        let mut out = Ok(());
        self.agent.run_in_realm(&self.realm, |agent, mut gc| {
            let target = registry
                .realms
                .borrow()
                .get(&realm)
                .expect("checked realm")
                .get(agent, gc.nogc())
                .unbind();
            let global = target.bind(gc.nogc()).global_object(agent).unbind();
            for (short, length) in genet_wasm::NATIVES {
                let name = &format!("__genetWasm{short}");
                let func = create_named_host_function(
                    agent,
                    Behaviour::Regular(native(short)),
                    name,
                    length,
                    Some(target.bind(gc.nogc())),
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
        self.eval_in_realm(realm, genet_wasm::BOOTSTRAP)
            .map(drop)
            .map_err(|e| format!("{e:?}"))
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
