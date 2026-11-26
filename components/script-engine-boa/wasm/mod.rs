// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A `WebAssembly` global for scripts on Bole, backed by wasmi (Genet's
//! WebAssembly object plan, W5). The engine-free half is `genet-wasm`'s
//! (D12); this module converts Bole's values and runs the call bridge. Off
//! unless the `wasm` feature is on and the host calls
//! [`BoaEngine::install_webassembly`].

mod bridge;
mod entities_natives;
mod handle_natives;
mod memory_natives;
mod natives;

use std::cell::RefCell;
use std::rc::Rc;

use boa_engine::context::HostHooks;
use boa_engine::object::builtins::JsArrayBuffer;
use boa_engine::{Context, JsNativeError, JsResult, JsString, JsValue, NativeFunction, Source};
use script_engine_api::{MAIN_REALM, RealmId};

use crate::BoaEngine;

/// The engine's wasm state over Bole's values.
pub(crate) type WasmHost = genet_wasm::WasmHost<JsValue>;

/// The context data the natives reach the store through.
pub(crate) type WasmSlot = RefCell<Option<Rc<WasmHost>>>;

type Native = fn(&JsValue, &[JsValue], &mut Context) -> JsResult<JsValue>;

/// The native behind each name the bootstrap expects (`genet_wasm::NATIVES`).
fn native(name: &str) -> Native {
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
        "Stale" => memory_natives::stale,
        "Detach" => memory_natives::detach,
        "Repoint" => memory_natives::repoint,
        "Retain" => handle_natives::retain,
        "Release" => handle_natives::release,
        "Extern" => handle_natives::extern_new,
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
        other => unreachable!("genet-wasm names a native genet-bole lacks: {other}"),
    }
}

/// Bole's host hooks with the wasm feature: `HostResizeArrayBuffer` for a
/// memory's resizable buffer, the defaults otherwise.
#[derive(Debug, Default)]
pub(crate) struct WasmHooks;

impl HostHooks for WasmHooks {
    fn resize_array_buffer(
        &self,
        buffer: &JsArrayBuffer,
        new_byte_length: u64,
        context: &mut Context,
    ) -> JsResult<bool> {
        memory_natives::resize_hook(buffer, new_byte_length, context)
    }
}

/// Create the engine-wide wasm state if absent, register the natives on the
/// context's current realm and run the bootstrap there.
pub(crate) fn install_in_context(ctx: &mut Context) -> Result<(), String> {
    if ctx.get_data::<WasmSlot>().is_none() {
        ctx.insert_data::<WasmSlot>(RefCell::new(Some(Rc::new(WasmHost::new()))));
    }
    (|| {
        for (short, length) in genet_wasm::NATIVES {
            ctx.register_global_builtin_callable(
                JsString::from(format!("__genetWasm{short}")),
                length as usize,
                NativeFunction::from_fn_ptr(native(short)),
            )?;
        }
        ctx.eval(Source::from_bytes(genet_wasm::BOOTSTRAP))
            .map(drop)
    })()
    .map_err(|e| format!("{e:?}"))
}

impl BoaEngine {
    /// Give the main realm a `WebAssembly` global. Wasm calls draw on the fuel
    /// [`set_wasm_fuel`](Self::set_wasm_fuel) sets, unbounded until it is set.
    ///
    /// # Errors
    ///
    /// If the natives cannot be defined or the bootstrap throws.
    pub fn install_webassembly(&mut self) -> JsResult<()> {
        self.install_webassembly_in_realm(MAIN_REALM)
            .map_err(|e| JsNativeError::error().with_message(e).into())
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
        self.with_realm(realm, install_in_context)
            .map_err(|e| format!("{e:?}"))?
    }

    /// What the engine's wasm store holds: live wrappers and buffers, handle
    /// table entries, `externref` slots and store resets. `None` if
    /// `WebAssembly` has not been installed.
    pub fn wasm_stats(&self) -> Option<genet_wasm::Stats> {
        bridge::host(&self.ctx).map(|host| host.stats())
    }

    /// Set the fuel all wasm calls on this engine draw from; `None` is
    /// unbounded (D13). A call that runs out throws a
    /// `WebAssembly.RuntimeError`, and fuel stays exhausted until this is
    /// called again.
    ///
    /// # Errors
    ///
    /// If `WebAssembly` has not been installed.
    pub fn set_wasm_fuel(&mut self, fuel: Option<u64>) -> Result<(), String> {
        match bridge::host(&self.ctx) {
            Some(host) => {
                host.set_budget(fuel);
                Ok(())
            },
            None => Err("WebAssembly is not installed".to_string()),
        }
    }
}
