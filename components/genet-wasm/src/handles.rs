// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Who owns what (D6). Script's bootstrap retains a handle for every wrapper it
//! makes and for every buffer it attaches, and releases it when a
//! `FinalizationRegistry` reports the wrapper or buffer collected. The last
//! release frees the entry for reuse. wasmi frees nothing short of dropping its
//! store, so when no wrapper and no buffer is live the host drops its engine and
//! store and starts fresh.

use wasmi::ExternRef;

use crate::host::{UNBOUNDED_FUEL, WasmHost, fresh_store};
use crate::sections::{FUNC_KIND, GLOBAL_KIND, MEMORY_KIND, TABLE_KIND};

pub const MODULE_KIND: u8 = 4;
pub const INSTANCE_KIND: u8 = 5;
/// A buffer retains its memory. It counts as a buffer, not a wrapper.
pub const BUFFER_KIND: u8 = 6;
/// A keeper slot holding a script value an `externref` carries.
pub const EXTERN_KIND: u8 = 7;

/// What the host holds, for tests and the curious.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// Wrappers script holds that have not been collected.
    pub wrappers: usize,
    /// Buffers over wasm memory that are still attached.
    pub buffers: usize,
    /// Entries in the handle tables.
    pub handles: usize,
    /// `externref` slots.
    pub externs: usize,
    /// Times the store has been dropped and rebuilt.
    pub resets: usize,
}

impl<V> WasmHost<V> {
    pub fn stats(&self) -> Stats {
        Stats {
            wrappers: self.wrappers.get(),
            buffers: self.buffers.get(),
            handles: self.modules.borrow().len()
                + self.instances.borrow().len()
                + self.funcs.borrow().len()
                + self.memories.borrow().len()
                + self.globals.borrow().len()
                + self.tables.borrow().len(),
            externs: self.externs.borrow().len(),
            resets: self.resets.get(),
        }
    }

    /// Whether script holds anything that lives in this store.
    pub fn is_live(&self) -> bool {
        let stats = self.stats();
        stats.wrappers + stats.buffers + stats.handles > 0
    }

    /// Add an owner to a handle.
    pub fn retain(&self, kind: u8, handle: usize) {
        let held = match kind {
            FUNC_KIND => self.funcs.borrow_mut().retain(handle),
            TABLE_KIND => self.tables.borrow_mut().retain(handle),
            MEMORY_KIND | BUFFER_KIND => self.memories.borrow_mut().retain(handle),
            GLOBAL_KIND => self.globals.borrow_mut().retain(handle),
            MODULE_KIND => self.modules.borrow_mut().retain(handle),
            _ => false,
        };
        if held {
            let count = if kind == BUFFER_KIND {
                &self.buffers
            } else {
                &self.wrappers
            };
            count.set(count.get() + 1);
        }
    }

    /// Drop an owner of a handle, freeing the entry after the last. A buffer's
    /// release (`current`: it was the memory's current one) drops the memory's
    /// record of it. An instance, which nothing retains, is freed outright, and
    /// so is an `externref` slot.
    ///
    /// When that leaves no wrapper and no buffer the store is rebuilt. The
    /// memories that still had a buffer recorded then are returned, for their
    /// buffers to be detached before they can dangle.
    pub fn release(&self, kind: u8, handle: usize, current: bool) -> Vec<usize> {
        let dropped = match kind {
            FUNC_KIND => self
                .funcs
                .borrow_mut()
                .release(handle)
                .inspect(|freed| {
                    if let Some(entry) = freed {
                        self.func_handles.borrow_mut().remove(&entry.key);
                    }
                })
                .is_some(),
            TABLE_KIND => self.tables.borrow_mut().release(handle).is_some(),
            GLOBAL_KIND => self.globals.borrow_mut().release(handle).is_some(),
            MODULE_KIND => self.modules.borrow_mut().release(handle).is_some(),
            MEMORY_KIND => self.memories.borrow_mut().release(handle).is_some(),
            BUFFER_KIND => {
                if current {
                    self.clear_record(handle);
                }
                self.memories.borrow_mut().release(handle).is_some()
            },
            INSTANCE_KIND => {
                self.instances.borrow_mut().remove(handle);
                self.defined
                    .borrow_mut()
                    .retain(|&(i, _, _), _| i != handle);
                return Vec::new();
            },
            EXTERN_KIND => {
                self.externs.borrow_mut().remove(&handle);
                return Vec::new();
            },
            _ => false,
        };
        if !dropped {
            return Vec::new();
        }
        let count = if kind == BUFFER_KIND {
            &self.buffers
        } else {
            &self.wrappers
        };
        count.set(count.get().saturating_sub(1));
        if self.wrappers.get() == 0 && self.buffers.get() == 0 {
            return self.reset();
        }
        Vec::new()
    }

    /// Drop the engine and store and start fresh, keeping the fuel left.
    fn reset(&self) -> Vec<usize> {
        let leftover = self.recorded();
        let fuel = self.store.borrow().get_fuel().unwrap_or(UNBOUNDED_FUEL);
        let (engine, store) = fresh_store(fuel);
        *self.engine.borrow_mut() = engine;
        *self.store.borrow_mut() = store;
        self.modules.borrow_mut().clear();
        self.instances.borrow_mut().clear();
        self.funcs.borrow_mut().clear();
        self.memories.borrow_mut().clear();
        self.globals.borrow_mut().clear();
        self.tables.borrow_mut().clear();
        self.func_handles.borrow_mut().clear();
        self.defined.borrow_mut().clear();
        self.externs.borrow_mut().clear();
        self.results.borrow_mut().clear();
        self.trap.borrow_mut().clear();
        self.resets.set(self.resets.get() + 1);
        leftover
    }

    /// A fresh `externref` slot, for script to hold a value in.
    pub fn extern_new(&self) -> usize {
        let slot = self.next_extern.get();
        self.next_extern.set(slot + 1);
        let extern_ref = ExternRef::new(&mut *self.store.borrow_mut(), slot);
        self.externs.borrow_mut().insert(slot, extern_ref);
        slot
    }

    pub fn extern_ref(&self, slot: usize) -> Option<ExternRef> {
        self.externs.borrow().get(&slot).copied()
    }

    /// The keeper slot an `externref` carries.
    pub fn extern_slot(&self, extern_ref: &ExternRef) -> Option<usize> {
        let store = self.store.borrow();
        extern_ref.data(&*store).downcast_ref::<usize>().copied()
    }
}
