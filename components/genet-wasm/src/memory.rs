// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Memories and the buffers script sees them through (W2, D6 and D7).
//!
//! A memory's buffer is an engine `ArrayBuffer` over wasmi's own bytes, made
//! only when script asks for it. The host never holds the buffer: script's
//! bootstrap does, weakly, so a dropped buffer can be collected. The host keeps
//! only a record of the bytes it was made over. wasmi may move those bytes when
//! the memory grows, so at every point control passes between script and wasm
//! the bootstrap asks [`WasmHost::stale`] which buffers no longer match their
//! memory, and the engine detaches them (a fixed-length one) or repoints them
//! (a resizable one, `toResizableBuffer`, which keeps its identity). A grow that
//! adds no pages from wasm moves nothing and goes unseen (D7); one from script
//! always detaches.

use wasmi::{Memory, MemoryType};

use crate::host::WasmHost;

/// The bytes a memory's current buffer was made over.
struct BufferRecord {
    /// Order of creation, to pick between empty buffers that look alike.
    seq: u64,
    ptr: *mut u8,
    len: usize,
    resizable: bool,
    /// A resizable buffer's maximum length, in bytes.
    max: usize,
}

pub struct MemoryEntry {
    memory: Memory,
    record: Option<BufferRecord>,
}

impl<V> WasmHost<V> {
    pub fn push_memory(&self, memory: Memory) -> usize {
        self.memories.borrow_mut().insert(MemoryEntry {
            memory,
            record: None,
        })
    }

    pub fn memory(&self, handle: usize) -> Option<Memory> {
        self.memories.borrow().get(handle).map(|e| e.memory)
    }

    /// `new WebAssembly.Memory({ initial, maximum, address })`, in pages.
    ///
    /// # Errors
    ///
    /// wasmi's message, for the native to throw.
    pub fn memory_new(
        &self,
        initial: u64,
        maximum: Option<u64>,
        is64: bool,
    ) -> Result<usize, String> {
        let mut builder = MemoryType::builder();
        builder.memory64(is64).min(initial).max(maximum);
        let ty = builder.build().map_err(|e| e.to_string())?;
        let memory = Memory::new(&mut *self.store.borrow_mut(), ty).map_err(|e| e.to_string())?;
        Ok(self.push_memory(memory))
    }

    /// Grow by `delta` pages and return the old size. The bootstrap detaches
    /// the buffer whether or not any page was added, as the spec requires.
    ///
    /// # Errors
    ///
    /// wasmi's message, for the native to throw.
    pub fn memory_grow(&self, handle: usize, delta: u64) -> Result<u64, String> {
        let memory = self.memory(handle).ok_or("not a memory")?;
        let mut store = self.store.borrow_mut();
        memory.grow(&mut *store, delta).map_err(|e| e.to_string())
    }

    pub fn memory_type(&self, handle: usize) -> Option<MemoryType> {
        Some(self.memory(handle)?.ty(&*self.store.borrow()))
    }

    /// The memory's bytes as they are now.
    pub fn memory_bytes(&self, handle: usize) -> Option<(*mut u8, usize)> {
        let memory = self.memory(handle)?;
        let store = self.store.borrow();
        Some((memory.data_ptr(&*store), memory.data_size(&*store)))
    }

    /// Record the buffer just made over the memory's current bytes. A
    /// resizable one holds `max` bytes at most.
    pub fn set_record(&self, handle: usize, resizable: bool, max: usize) {
        let Some((ptr, len)) = self.memory_bytes(handle) else {
            return;
        };
        if let Some(entry) = self.memories.borrow_mut().get_mut(handle) {
            self.record_seq.set(self.record_seq.get() + 1);
            entry.record = Some(BufferRecord {
                seq: self.record_seq.get(),
                ptr,
                len,
                resizable,
                max,
            });
        }
    }

    /// Note that the memory's resizable buffer was repointed at its bytes.
    pub fn sync_record(&self, handle: usize) {
        let Some((ptr, len)) = self.memory_bytes(handle) else {
            return;
        };
        if let Some(record) = self
            .memories
            .borrow_mut()
            .get_mut(handle)
            .and_then(|e| e.record.as_mut())
        {
            (record.ptr, record.len) = (ptr, len);
        }
    }

    pub fn clear_record(&self, handle: usize) {
        if let Some(entry) = self.memories.borrow_mut().get_mut(handle) {
            entry.record = None;
        }
    }

    /// Whether the memory's current buffer is resizable; `None` if it has none.
    pub fn buffer_is_resizable(&self, handle: usize) -> Option<bool> {
        let memories = self.memories.borrow();
        Some(memories.get(handle)?.record.as_ref()?.resizable)
    }

    /// Every memory whose buffer no longer matches its bytes, and `force`'s
    /// fixed-length buffer in any case. `true` means detach it, and its record
    /// is dropped; `false` means repoint it, and its record is updated.
    pub fn stale(&self, force: Option<usize>) -> Vec<(usize, bool)> {
        let store = self.store.borrow();
        let mut memories = self.memories.borrow_mut();
        let mut out = Vec::new();
        for (handle, entry) in memories.iter_mut() {
            let Some(record) = &mut entry.record else {
                continue;
            };
            let now = (
                entry.memory.data_ptr(&*store),
                entry.memory.data_size(&*store),
            );
            let moved = now != (record.ptr, record.len);
            if record.resizable {
                if moved {
                    (record.ptr, record.len) = now;
                    out.push((handle, false));
                }
            } else if moved || force == Some(handle) {
                entry.record = None;
                out.push((handle, true));
            }
        }
        out
    }

    /// The memory with a resizable buffer of `len` bytes at `ptr`, at most
    /// `max` if the engine can say. An empty buffer has no pointer of its own
    /// to tell by, and the engine's resize hook gives nothing else, so of
    /// several empty memories with the same maximum the newest wins. A buffer
    /// that script dropped still has its record until it is collected, so this
    /// is wrong only if an older empty buffer is resized while a newer one
    /// like it exists.
    pub fn resizable_memory(
        &self,
        ptr: *const u8,
        len: usize,
        max: Option<usize>,
    ) -> Option<usize> {
        let mut memories = self.memories.borrow_mut();
        memories
            .iter_mut()
            .filter_map(|(handle, e)| Some((handle, e.record.as_ref()?)))
            .filter(|(_, r)| {
                r.resizable
                    && max.is_none_or(|m| r.max == m)
                    && r.len == len
                    && (len == 0 || r.ptr.cast_const() == ptr)
            })
            .max_by_key(|(_, r)| r.seq)
            .map(|(handle, _)| handle)
    }

    /// Handles of the memories that still have a buffer recorded.
    pub(crate) fn recorded(&self) -> Vec<usize> {
        let mut memories = self.memories.borrow_mut();
        memories
            .iter_mut()
            .filter(|(_, e)| e.record.is_some())
            .map(|(handle, _)| handle)
            .collect()
    }
}
