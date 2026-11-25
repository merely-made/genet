// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Memories and the buffers script sees them through (W2, D6 and D7).
//!
//! A memory's buffer is a Vano `ArrayBuffer` over wasmi's own bytes, made only
//! when script asks for it. wasmi may move those bytes when the memory grows,
//! so each buffer is kept beside the pointer and length it was made over, and
//! a buffer whose memory no longer matches is detached before script runs
//! again. A grow that adds no pages from wasm moves nothing and goes unseen
//! (D7); one from script always detaches.

use wasmi::{Memory, MemoryType};

use super::super::NovaValue;
use super::host::WasmHost;
use super::sections::MEMORY_KIND;

/// A buffer script holds, and the bytes it was made over.
struct BufferView {
    value: NovaValue,
    ptr: *mut u8,
    len: usize,
}

pub(super) struct MemoryEntry {
    memory: Memory,
    buffer: Option<BufferView>,
}

impl WasmHost {
    fn push_memory(&self, memory: Memory) -> usize {
        let mut memories = self.memories.borrow_mut();
        memories.push(MemoryEntry {
            memory,
            buffer: None,
        });
        memories.len() - 1
    }

    pub(super) fn memory(&self, handle: usize) -> Option<Memory> {
        self.memories.borrow().get(handle).map(|e| e.memory)
    }

    /// `new WebAssembly.Memory({ initial, maximum })`, in pages.
    pub(super) fn memory_new(&self, initial: u32, maximum: Option<u32>) -> Result<usize, String> {
        let ty = MemoryType::new(initial, maximum);
        let memory = Memory::new(&mut *self.store.borrow_mut(), ty).map_err(|e| e.to_string())?;
        Ok(self.push_memory(memory))
    }

    /// Grow by `delta` pages and return the old size. The caller detaches the
    /// buffer whether or not any page was added, as the spec requires.
    pub(super) fn memory_grow(&self, handle: usize, delta: u32) -> Result<u64, String> {
        let memory = self.memory(handle).ok_or("not a memory")?;
        let mut store = self.store.borrow_mut();
        memory
            .grow(&mut *store, u64::from(delta))
            .map_err(|e| e.to_string())
    }

    /// The handle for a memory an instance exports. A memory below the
    /// imported count is the import itself, so it keeps the import's handle.
    pub(super) fn export_memory(&self, instance: usize, name: &str) -> Option<usize> {
        let instances = self.instances.borrow();
        let entry = instances.get(instance)?;
        let index = self.export_index(entry.module, name, MEMORY_KIND)?;
        if let Some(handle) = entry.imported_memories.get(index as usize) {
            return Some(*handle);
        }
        if let Some(handle) = self.defined_memories.borrow().get(&(instance, index)) {
            return Some(*handle);
        }
        let memory = entry.instance.get_memory(&*self.store.borrow(), name)?;
        let handle = self.push_memory(memory);
        self.defined_memories
            .borrow_mut()
            .insert((instance, index), handle);
        Some(handle)
    }

    /// The memory's bytes as they are now.
    pub(super) fn memory_bytes(&self, handle: usize) -> Option<(*mut u8, usize)> {
        let memory = self.memory(handle)?;
        let store = self.store.borrow();
        Some((memory.data_ptr(&*store), memory.data_size(&*store)))
    }

    /// Run `f` on the buffer script holds for `handle`, if it is current.
    pub(super) fn with_buffer<R>(
        &self,
        handle: usize,
        f: impl FnOnce(&NovaValue) -> R,
    ) -> Option<R> {
        let memories = self.memories.borrow();
        memories.get(handle)?.buffer.as_ref().map(|b| f(&b.value))
    }

    pub(super) fn set_buffer(&self, handle: usize, value: NovaValue, ptr: *mut u8, len: usize) {
        if let Some(entry) = self.memories.borrow_mut().get_mut(handle) {
            entry.buffer = Some(BufferView { value, ptr, len });
        }
    }

    /// Take out every buffer whose memory has moved or changed length, and
    /// `force`'s buffer in any case, for the caller to detach.
    pub(super) fn stale_buffers(&self, force: Option<usize>) -> Vec<NovaValue> {
        let store = self.store.borrow();
        let mut memories = self.memories.borrow_mut();
        let mut stale = Vec::new();
        for (handle, entry) in memories.iter_mut().enumerate() {
            let Some(view) = &entry.buffer else { continue };
            let now = (
                entry.memory.data_ptr(&*store),
                entry.memory.data_size(&*store),
            );
            if force == Some(handle) || now != (view.ptr, view.len) {
                stale.extend(entry.buffer.take().map(|view| view.value));
            }
        }
        stale
    }
}
