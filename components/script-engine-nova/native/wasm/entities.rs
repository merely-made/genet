// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `WebAssembly.Global` and `WebAssembly.Table` on the store (W4).

use wasmi::{Global, GlobalType, Mutability, Ref, Table, TableType, Val};

use super::host::WasmHost;

/// A table element as a value, or `None` for a value that is not a reference.
pub(super) fn val_ref(value: Val) -> Option<Ref> {
    match value {
        Val::FuncRef(r) => Some(Ref::Func(r)),
        Val::ExternRef(r) => Some(Ref::Extern(r)),
        _ => None,
    }
}

impl WasmHost {
    pub(super) fn global_new(&self, value: Val, mutable: bool) -> usize {
        let mutability = if mutable {
            Mutability::Var
        } else {
            Mutability::Const
        };
        let global = Global::new(&mut *self.store.borrow_mut(), value, mutability);
        self.push(&self.globals, global)
    }

    fn global(&self, handle: usize) -> Option<Global> {
        self.globals.borrow().get(handle).copied()
    }

    pub(super) fn global_type(&self, handle: usize) -> Option<GlobalType> {
        Some(self.global(handle)?.ty(&*self.store.borrow()))
    }

    pub(super) fn global_get(&self, handle: usize) -> Option<Val> {
        Some(self.global(handle)?.get(&*self.store.borrow()))
    }

    pub(super) fn global_set(&self, handle: usize, value: Val) -> Result<(), String> {
        let global = self.global(handle).ok_or("not a global")?;
        global
            .set(&mut *self.store.borrow_mut(), value)
            .map_err(|e| e.to_string())
    }

    pub(super) fn table_new(&self, ty: TableType, init: Ref) -> Result<usize, String> {
        let table =
            Table::new(&mut *self.store.borrow_mut(), ty, init).map_err(|e| e.to_string())?;
        Ok(self.push(&self.tables, table))
    }

    fn table(&self, handle: usize) -> Option<Table> {
        self.tables.borrow().get(handle).copied()
    }

    pub(super) fn table_type(&self, handle: usize) -> Option<TableType> {
        Some(self.table(handle)?.ty(&*self.store.borrow()))
    }

    pub(super) fn table_size(&self, handle: usize) -> Option<u64> {
        Some(self.table(handle)?.size(&*self.store.borrow()))
    }

    /// `None` for a bad handle, `Some(None)` for an index out of bounds.
    pub(super) fn table_get(&self, handle: usize, index: u64) -> Option<Option<Ref>> {
        Some(self.table(handle)?.get(&*self.store.borrow(), index))
    }

    pub(super) fn table_set(&self, handle: usize, index: u64, value: Ref) -> Result<(), String> {
        let table = self.table(handle).ok_or("not a table")?;
        table
            .set(&mut *self.store.borrow_mut(), index, value)
            .map_err(|e| e.to_string())
    }

    pub(super) fn table_grow(&self, handle: usize, delta: u64, init: Ref) -> Result<u64, String> {
        let table = self.table(handle).ok_or("not a table")?;
        table
            .grow(&mut *self.store.borrow_mut(), delta, init)
            .map_err(|e| e.to_string())
    }
}
