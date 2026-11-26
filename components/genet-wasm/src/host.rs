// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The wasmi side of the bridge: one store per engine and the handle tables
//! script reaches it through. Nothing here touches the JavaScript heap. The
//! script values it holds (import wrappers, `externref` payloads, memory
//! buffers) are the engine's rooted values, `V`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use wasmi::{
    Config, Engine, Error, Extern, ExternType, Func, FuncType, Global, Instance, Module, Store,
    Table, Val, errors::HostError,
};

use crate::json;
use crate::memory::MemoryEntry;
use crate::sections::{self, FUNC_KIND, GLOBAL_KIND, MEMORY_KIND, START_EXPORT, TABLE_KIND};

/// Raised by every function made from script. The call suspends at it and
/// the bridge runs the script function for `slot`, then resumes.
#[derive(Debug)]
pub struct ImportCall {
    pub slot: usize,
    pub params: Vec<Val>,
}

impl fmt::Display for ImportCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "script function {} called", self.slot)
    }
}

impl HostError for ImportCall {}

/// What script supplies for one import, in the module's import order.
pub enum ImportValue<V> {
    /// A script function's wrapper, made into a host function here.
    Script(V),
    /// An entity that already lives in the store, by kind and handle.
    Entity(u8, usize),
}

pub struct ModuleEntry {
    pub module: Module,
    has_start: bool,
    /// For each import in the module's order, its place in wasmi's.
    import_order: Vec<usize>,
    /// `(name, kind, index)` for every export.
    pub exports: Vec<(String, u8, u32)>,
}

pub struct InstanceEntry {
    pub instance: Instance,
    pub module: usize,
    /// Handles of what it imported, by kind and then by index.
    pub imported: [Vec<usize>; 4],
}

struct FuncEntry {
    func: Func,
    /// Index in its module's function space, which names the script function.
    index: Option<u32>,
}

/// Fuel a store holds when the host has set no budget.
const UNBOUNDED_FUEL: u64 = u64::MAX;

/// One engine's WebAssembly state. Handles are indices that never move.
pub struct WasmHost<V> {
    engine: Engine,
    pub store: RefCell<Store<()>>,
    pub modules: RefCell<Vec<ModuleEntry>>,
    pub instances: RefCell<Vec<InstanceEntry>>,
    funcs: RefCell<Vec<FuncEntry>>,
    /// Handles by function identity, so a function has one handle however it
    /// reaches script.
    func_handles: RefCell<HashMap<String, usize>>,
    /// Script wrappers for functions made from script, by slot.
    pub imports: RefCell<Vec<V>>,
    /// The script values `externref`s hold, by slot.
    pub externs: RefCell<Vec<V>>,
    pub memories: RefCell<Vec<MemoryEntry<V>>>,
    pub globals: RefCell<Vec<Global>>,
    pub tables: RefCell<Vec<Table>>,
    /// Handles of what an instance defines, by `(instance, kind, index)`.
    defined: RefCell<HashMap<(usize, u8, u32), usize>>,
    /// Results of the last multi-value call, read one by one.
    pub results: RefCell<Vec<Val>>,
    /// Values a script function returned for a multi-value import.
    pub staged: RefCell<Vec<V>>,
    /// The message of the last trap `call` reported.
    pub trap: RefCell<String>,
}

fn kind_of(ty: &ExternType) -> u8 {
    match ty {
        ExternType::Func(_) => FUNC_KIND,
        ExternType::Table(_) => TABLE_KIND,
        ExternType::Memory(_) => MEMORY_KIND,
        ExternType::Global(_) => GLOBAL_KIND,
    }
}

/// wasmi yields imports grouped by kind. Match each import in the module's
/// order to wasmi's by module, name and kind, or keep wasmi's order if the
/// two lists do not line up.
fn import_order(module: &Module, original: &[(String, String, u8)]) -> Vec<usize> {
    let wasmi: Vec<(String, String, u8)> = module
        .imports()
        .map(|i| {
            (
                i.module().to_string(),
                i.name().to_string(),
                kind_of(i.ty()),
            )
        })
        .collect();
    let mut used = vec![false; wasmi.len()];
    let mut order = Vec::with_capacity(original.len());
    for entry in original {
        match (0..wasmi.len()).find(|&w| !used[w] && wasmi[w] == *entry) {
            Some(w) => {
                used[w] = true;
                order.push(w);
            },
            None => return (0..wasmi.len()).collect(),
        }
    }
    if order.len() == wasmi.len() {
        order
    } else {
        (0..wasmi.len()).collect()
    }
}

/// wasmi gives `Func` no equality; its debug form names the store and index.
fn func_key(func: &Func) -> String {
    format!("{func:?}")
}

impl<V> Default for WasmHost<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> WasmHost<V> {
    /// A store with fuel metering on and unbounded.
    ///
    /// # Panics
    ///
    /// Never: fuel is enabled just before it is set.
    pub fn new() -> Self {
        let mut config = Config::default();
        config
            .consume_fuel(true)
            .wasm_simd(true)
            .wasm_relaxed_simd(true)
            .wasm_memory64(true);
        let engine = Engine::new(&config);
        let mut store = Store::new(&engine, ());
        store.set_fuel(UNBOUNDED_FUEL).expect("fuel is enabled");
        Self {
            engine,
            store: RefCell::new(store),
            modules: RefCell::default(),
            instances: RefCell::default(),
            funcs: RefCell::default(),
            func_handles: RefCell::default(),
            imports: RefCell::default(),
            externs: RefCell::default(),
            memories: RefCell::default(),
            globals: RefCell::default(),
            tables: RefCell::default(),
            defined: RefCell::default(),
            results: RefCell::default(),
            staged: RefCell::default(),
            trap: RefCell::default(),
        }
    }

    /// Whether script holds anything that lives in this store.
    pub fn is_live(&self) -> bool {
        !self.modules.borrow().is_empty()
            || !self.memories.borrow().is_empty()
            || !self.globals.borrow().is_empty()
            || !self.tables.borrow().is_empty()
            || !self.funcs.borrow().is_empty()
    }

    /// Set the fuel every wasm call on this engine draws from. `None` is
    /// unbounded. Exhausted fuel stays exhausted until the host sets it again.
    ///
    /// # Panics
    ///
    /// Never: the store meters fuel.
    pub fn set_budget(&self, fuel: Option<u64>) {
        let fuel = fuel.unwrap_or(UNBOUNDED_FUEL);
        let mut store = self.store.borrow_mut();
        store.set_fuel(fuel).expect("fuel is enabled");
    }

    /// Compile a module, moving its start function out (`sections`).
    ///
    /// # Errors
    ///
    /// wasmi's message, for the native to throw.
    pub fn compile(&self, bytes: Vec<u8>) -> Result<usize, String> {
        let prepared = sections::prepare(bytes);
        let module = Module::new(&self.engine, &prepared.bytes).map_err(|e| e.to_string())?;
        let import_order = import_order(&module, &prepared.imports);
        let mut modules = self.modules.borrow_mut();
        modules.push(ModuleEntry {
            module,
            has_start: prepared.has_start,
            import_order,
            exports: prepared.exports,
        });
        Ok(modules.len() - 1)
    }

    /// The module's imports and exports with their types, as JSON.
    pub fn describe(&self, handle: usize) -> Option<String> {
        let modules = self.modules.borrow();
        let entry = modules.get(handle)?;
        let imports: Vec<_> = entry.module.imports().collect();
        let mut out = String::from("{\"imports\":[");
        for (i, import) in entry.import_order.iter().map(|&w| &imports[w]).enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"module\":");
            json::string(&mut out, import.module());
            out.push_str(",\"name\":");
            json::string(&mut out, import.name());
            out.push(',');
            json::extern_type(&mut out, import.ty());
            out.push('}');
        }
        out.push_str("],\"exports\":[");
        let exports = entry
            .exports
            .iter()
            .filter(|(name, _, _)| name != START_EXPORT);
        for (i, (name, _, _)) in exports.enumerate() {
            let ty = entry.module.get_export(name)?;
            if i > 0 {
                out.push(',');
            }
            out.push_str("{\"name\":");
            json::string(&mut out, name);
            out.push(',');
            json::extern_type(&mut out, &ty);
            out.push('}');
        }
        out.push_str("]}");
        Some(out)
    }

    /// The bytes of the `nth` custom section called `name`.
    pub fn custom_section(&self, module: usize, name: &str, nth: usize) -> Option<Vec<u8>> {
        let modules = self.modules.borrow();
        let module = &modules.get(module)?.module;
        let section = module
            .custom_sections()
            .filter(|s| s.name() == name)
            .nth(nth)?;
        Some(section.data().to_vec())
    }

    /// A host function of type `ty` that calls the script wrapper `wrapper`.
    pub fn script_func(&self, store: &mut Store<()>, ty: FuncType, wrapper: V) -> Func {
        let mut imports = self.imports.borrow_mut();
        let slot = imports.len();
        imports.push(wrapper);
        Func::new(store, ty, move |_, params, _| {
            Err(Error::host(ImportCall {
                slot,
                params: params.to_vec(),
            }))
        })
    }

    fn entity(&self, kind: u8, handle: usize) -> Option<Extern> {
        Some(match kind {
            FUNC_KIND => Extern::Func(self.func(handle)?),
            TABLE_KIND => Extern::Table(*self.tables.borrow().get(handle)?),
            MEMORY_KIND => Extern::Memory(self.memory(handle)?),
            GLOBAL_KIND => Extern::Global(*self.globals.borrow().get(handle)?),
            _ => return None,
        })
    }

    /// Link a module against what script supplied, one value per import in
    /// order; wasmi checks every type. Returns the instance and the start
    /// function still to be run.
    ///
    /// # Errors
    ///
    /// The message, prefixed `link:` or `trap:` for the bootstrap.
    pub fn instantiate(
        &self,
        module: usize,
        values: Vec<ImportValue<V>>,
    ) -> Result<(usize, Option<Func>), String> {
        let modules = self.modules.borrow();
        let entry = modules.get(module).ok_or("link:not a module")?;
        let imports: Vec<_> = entry.module.imports().collect();
        if values.len() != imports.len() {
            return Err("link:the wrong number of imports".to_string());
        }
        let mut externs: Vec<Option<Extern>> = vec![None; imports.len()];
        let mut imported: [Vec<usize>; 4] = Default::default();
        for (value, &w) in values.into_iter().zip(&entry.import_order) {
            let ext = match (imports[w].ty(), value) {
                (ExternType::Func(ty), ImportValue::Script(wrapper)) => {
                    let mut store = self.store.borrow_mut();
                    let func = self.script_func(&mut store, ty.clone(), wrapper);
                    Extern::Func(func)
                },
                (_, ImportValue::Entity(kind, handle)) => {
                    let ext = self
                        .entity(kind, handle)
                        .ok_or("link:not a WebAssembly object")?;
                    if kind != FUNC_KIND {
                        imported[kind as usize].push(handle);
                    }
                    ext
                },
                (ty, ImportValue::Script(_)) => {
                    return Err(format!(
                        "link:a {} import needs a WebAssembly object",
                        json::kind_name(ty)
                    ));
                },
            };
            externs[w] = Some(ext);
        }
        let externs: Vec<Extern> = externs.into_iter().flatten().collect();
        let mut store = self.store.borrow_mut();
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
            imported,
        });
        Ok((instances.len() - 1, start))
    }

    pub fn export_index(&self, module: usize, name: &str, kind: u8) -> Option<u32> {
        self.modules.borrow()[module]
            .exports
            .iter()
            .find(|(export, k, _)| export == name && *k == kind)
            .map(|(_, _, index)| *index)
    }

    /// The handle of an instance's export of `kind`. An import passed through
    /// keeps its handle, and a defined entity gets one handle however often
    /// it is exported, so script sees one object for it.
    pub fn export_of(&self, instance: usize, name: &str, kind: u8) -> Option<usize> {
        let (module, inst) = {
            let instances = self.instances.borrow();
            let entry = instances.get(instance)?;
            (entry.module, entry.instance)
        };
        let index = self.export_index(module, name, kind)?;
        if kind == FUNC_KIND {
            let func = inst.get_func(&*self.store.borrow(), name)?;
            return Some(self.func_handle(func, Some(index)));
        }
        let imported = self.instances.borrow()[instance].imported[kind as usize]
            .get(index as usize)
            .copied();
        if imported.is_some() {
            return imported;
        }
        if let Some(handle) = self.defined.borrow().get(&(instance, kind, index)) {
            return Some(*handle);
        }
        let handle = {
            let store = self.store.borrow();
            match kind {
                MEMORY_KIND => self.push_memory(inst.get_memory(&*store, name)?),
                TABLE_KIND => self.push(&self.tables, inst.get_table(&*store, name)?),
                GLOBAL_KIND => self.push(&self.globals, inst.get_global(&*store, name)?),
                _ => return None,
            }
        };
        self.defined
            .borrow_mut()
            .insert((instance, kind, index), handle);
        Some(handle)
    }

    pub fn push<T>(&self, list: &RefCell<Vec<T>>, item: T) -> usize {
        let mut list = list.borrow_mut();
        list.push(item);
        list.len() - 1
    }

    /// The one handle for `func`, minted the first time it reaches script.
    pub fn func_handle(&self, func: Func, index: Option<u32>) -> usize {
        let key = func_key(&func);
        if let Some(handle) = self.func_handles.borrow().get(&key) {
            return *handle;
        }
        let handle = self.push(&self.funcs, FuncEntry { func, index });
        self.func_handles.borrow_mut().insert(key, handle);
        handle
    }

    pub fn func(&self, handle: usize) -> Option<Func> {
        self.funcs.borrow().get(handle).map(|e| e.func)
    }

    /// `{"index":…,"parameters":[…],"results":[…]}` for a function.
    pub fn signature(&self, handle: usize) -> Option<String> {
        let funcs = self.funcs.borrow();
        let entry = funcs.get(handle)?;
        let ty = entry.func.ty(&*self.store.borrow());
        let mut out = String::from("{\"index\":");
        match entry.index {
            Some(index) => out.push_str(&index.to_string()),
            None => out.push_str("null"),
        }
        out.push(',');
        json::func(&mut out, &ty);
        out.push('}');
        Some(out)
    }

    pub fn func_type(&self, func: &Func) -> FuncType {
        func.ty(&*self.store.borrow())
    }
}
