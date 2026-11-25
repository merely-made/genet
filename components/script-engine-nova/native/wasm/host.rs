// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The wasmi side of the bridge: one store per engine and the handle tables
//! script reaches it through. Nothing here touches the JavaScript heap except
//! the rooted import wrappers and memory buffers it holds.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use wasmi::{
    Config, Engine, Error, Extern, ExternType, Func, FuncType, Instance, Module, Store, Val,
    ValType, errors::HostError,
};

use super::super::NovaValue;
use super::memory::MemoryEntry;
use super::sections::{self, FUNC_KIND, START_EXPORT};

/// Raised by every imported function. The call suspends at it and the bridge
/// runs the script function for `slot`, then resumes with its result.
#[derive(Debug)]
pub(super) struct ImportCall {
    pub(super) slot: usize,
    pub(super) params: Vec<Val>,
}

impl fmt::Display for ImportCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "script import {} called", self.slot)
    }
}

impl HostError for ImportCall {}

/// What script supplies for one import, in the module's import order.
pub(super) enum ImportValue {
    Func(NovaValue),
    Memory(usize),
}

pub(super) struct ModuleEntry {
    module: Module,
    has_start: bool,
    /// `(name, kind, index)` for every export.
    pub(super) exports: Vec<(String, u8, u32)>,
}

pub(super) struct InstanceEntry {
    pub(super) instance: Instance,
    pub(super) module: usize,
    /// Handles of the memories it imported, by memory index.
    pub(super) imported_memories: Vec<usize>,
}

struct FuncEntry {
    func: Func,
    /// Index in its module's function space, which names the script function.
    index: Option<u32>,
}

/// Fuel a store holds when the host has set no budget.
const UNBOUNDED_FUEL: u64 = u64::MAX;

/// One engine's WebAssembly state. Handles are indices that never move.
pub(crate) struct WasmHost {
    engine: Engine,
    pub(super) store: RefCell<Store<()>>,
    pub(super) modules: RefCell<Vec<ModuleEntry>>,
    pub(super) instances: RefCell<Vec<InstanceEntry>>,
    funcs: RefCell<Vec<FuncEntry>>,
    /// Script wrappers for imported functions, by import slot.
    pub(super) imports: RefCell<Vec<NovaValue>>,
    pub(super) memories: RefCell<Vec<MemoryEntry>>,
    /// Handles of memories an instance defines, by `(instance, memory index)`.
    pub(super) defined_memories: RefCell<HashMap<(usize, u32), usize>>,
}

pub(super) fn val_type_name(ty: ValType) -> &'static str {
    match ty {
        ValType::I32 => "i32",
        ValType::I64 => "i64",
        ValType::F32 => "f32",
        ValType::F64 => "f64",
        ValType::V128 => "v128",
        ValType::FuncRef => "funcref",
        ValType::ExternRef => "externref",
    }
}

fn json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn json_types(out: &mut String, types: &[ValType]) {
    out.push('[');
    for (i, ty) in types.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        json_string(out, val_type_name(*ty));
    }
    out.push(']');
}

fn json_signature(out: &mut String, ty: &FuncType) {
    out.push_str("\"params\":");
    json_types(out, ty.params());
    out.push_str(",\"results\":");
    json_types(out, ty.results());
}

fn kind_name(ty: &ExternType) -> &'static str {
    match ty {
        ExternType::Func(_) => "function",
        ExternType::Table(_) => "table",
        ExternType::Memory(_) => "memory",
        ExternType::Global(_) => "global",
    }
}

impl WasmHost {
    pub(crate) fn new() -> Self {
        let mut config = Config::default();
        config.consume_fuel(true);
        let engine = Engine::new(&config);
        let mut store = Store::new(&engine, ());
        store.set_fuel(UNBOUNDED_FUEL).expect("fuel is enabled");
        Self {
            engine,
            store: RefCell::new(store),
            modules: RefCell::new(Vec::new()),
            instances: RefCell::new(Vec::new()),
            funcs: RefCell::new(Vec::new()),
            imports: RefCell::new(Vec::new()),
            memories: RefCell::new(Vec::new()),
            defined_memories: RefCell::new(HashMap::new()),
        }
    }

    /// Whether script holds anything that lives in this store.
    pub(crate) fn is_live(&self) -> bool {
        !self.modules.borrow().is_empty() || !self.memories.borrow().is_empty()
    }

    /// Set the fuel every wasm call on this engine draws from. `None` is
    /// unbounded. Exhausted fuel stays exhausted until the host sets it again.
    pub(crate) fn set_budget(&self, fuel: Option<u64>) {
        let fuel = fuel.unwrap_or(UNBOUNDED_FUEL);
        self.store
            .borrow_mut()
            .set_fuel(fuel)
            .expect("fuel is enabled");
    }

    pub(super) fn compile(&self, bytes: Vec<u8>) -> Result<usize, String> {
        let prepared = sections::prepare(bytes);
        let module = Module::new(&self.engine, &prepared.bytes).map_err(|e| e.to_string())?;
        let mut modules = self.modules.borrow_mut();
        modules.push(ModuleEntry {
            module,
            has_start: prepared.has_start,
            exports: prepared.exports,
        });
        Ok(modules.len() - 1)
    }

    /// The module's imports and exports as JSON, for the bootstrap.
    pub(super) fn describe(&self, handle: usize) -> Option<String> {
        let modules = self.modules.borrow();
        let entry = modules.get(handle)?;
        let mut out = String::from("{\"imports\":[");
        for (i, import) in entry.module.imports().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"module\":");
            json_string(&mut out, import.module());
            out.push_str(",\"name\":");
            json_string(&mut out, import.name());
            out.push_str(",\"kind\":");
            json_string(&mut out, kind_name(import.ty()));
            if let ExternType::Func(ty) = import.ty() {
                out.push(',');
                json_signature(&mut out, ty);
            }
            out.push('}');
        }
        out.push_str("],\"exports\":[");
        let exports = entry.module.exports().filter(|e| e.name() != START_EXPORT);
        for (i, export) in exports.enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"name\":");
            json_string(&mut out, export.name());
            out.push_str(",\"kind\":");
            json_string(&mut out, kind_name(export.ty()));
            out.push('}');
        }
        out.push_str("]}");
        Some(out)
    }

    /// Link a module against what script supplied, one value per import in
    /// order. Returns the instance and the start function still to be run.
    /// Errors are prefixed `link:` or `trap:` for the bootstrap.
    pub(super) fn instantiate(
        &self,
        module: usize,
        values: Vec<ImportValue>,
    ) -> Result<(usize, Option<Func>), String> {
        let modules = self.modules.borrow();
        let entry = modules.get(module).ok_or("link:not a module")?;
        let mut store = self.store.borrow_mut();
        let mut values = values.into_iter();
        let mut externs = Vec::new();
        let mut imported_memories = Vec::new();
        for import in entry.module.imports() {
            let value = values.next().ok_or("link:too few imports")?;
            match (import.ty(), value) {
                (ExternType::Func(ty), ImportValue::Func(wrapper)) => {
                    let mut imports = self.imports.borrow_mut();
                    let slot = imports.len();
                    imports.push(wrapper);
                    let func = Func::new(&mut *store, ty.clone(), move |_, params, _| {
                        Err(Error::host(ImportCall {
                            slot,
                            params: params.to_vec(),
                        }))
                    });
                    externs.push(Extern::Func(func));
                },
                (ExternType::Memory(_), ImportValue::Memory(handle)) => {
                    let memory = self.memory(handle).ok_or("link:not a memory")?;
                    imported_memories.push(handle);
                    externs.push(Extern::Memory(memory));
                },
                (ty, _) => {
                    return Err(format!(
                        "link:{} imports are not supported yet",
                        kind_name(ty)
                    ));
                },
            }
        }
        let instance = Instance::new(&mut *store, &entry.module, &externs).map_err(|e| {
            let prefix = if e.as_trap_code().is_some() {
                "trap"
            } else {
                "link"
            };
            format!("{prefix}:{e}")
        })?;
        let start = entry
            .has_start
            .then(|| instance.get_func(&*store, START_EXPORT))
            .flatten();
        let mut instances = self.instances.borrow_mut();
        instances.push(InstanceEntry {
            instance,
            module,
            imported_memories,
        });
        Ok((instances.len() - 1, start))
    }

    /// A handle for an exported function. The bootstrap asks once per export
    /// and caches the script function, so an instance's export keeps its
    /// identity. One function exported under two names gets two handles:
    /// wasmi's `Func` has no equality to merge them by.
    pub(super) fn export_func(&self, instance: usize, name: &str) -> Option<usize> {
        let instances = self.instances.borrow();
        let entry = instances.get(instance)?;
        let func = entry.instance.get_func(&*self.store.borrow(), name)?;
        let index = self.export_index(entry.module, name, FUNC_KIND);
        let mut funcs = self.funcs.borrow_mut();
        funcs.push(FuncEntry { func, index });
        Some(funcs.len() - 1)
    }

    pub(super) fn export_index(&self, module: usize, name: &str, kind: u8) -> Option<u32> {
        self.modules.borrow()[module]
            .exports
            .iter()
            .find(|(export, k, _)| export == name && *k == kind)
            .map(|(_, _, index)| *index)
    }

    pub(super) fn func(&self, handle: usize) -> Option<Func> {
        self.funcs.borrow().get(handle).map(|e| e.func)
    }

    /// `{"index":…,"params":[…],"results":[…]}` for an exported function.
    pub(super) fn signature(&self, handle: usize) -> Option<String> {
        let funcs = self.funcs.borrow();
        let entry = funcs.get(handle)?;
        let ty = entry.func.ty(&*self.store.borrow());
        let mut out = String::from("{\"index\":");
        match entry.index {
            Some(index) => out.push_str(&index.to_string()),
            None => out.push_str("null"),
        }
        out.push(',');
        json_signature(&mut out, &ty);
        out.push('}');
        Some(out)
    }

    pub(super) fn func_type(&self, func: &Func) -> FuncType {
        func.ty(&*self.store.borrow())
    }
}
