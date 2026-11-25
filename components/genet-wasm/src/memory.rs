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
//! (D7); one from script always detaches. A resizable buffer
//! (`toResizableBuffer`) is not detached: it is repointed at the moved bytes
//! and their new length, and keeps its identity.

use wasmi::{Memory, MemoryType};

use crate::host::WasmHost;

/// A buffer script holds, and the bytes it was made over.
struct BufferView<V> {
    value: V,
    ptr: *mut u8,
    len: usize,
    resizable: bool,
}

/// What the caller must do to the buffers after a memory may have moved.
pub struct Refresh<V> {
    /// Fixed-length buffers to detach.
    pub detach: Vec<V>,
    /// Memories whose resizable buffer must be repointed at these bytes.
    pub repoint: Vec<(usize, *mut u8, usize)>,
}

impl<V> Default for Refresh<V> {
    fn default() -> Self {
        Self {
            detach: Vec::new(),
            repoint: Vec::new(),
        }
    }
}

pub struct MemoryEntry<V> {
    memory: Memory,
    buffer: Option<BufferView<V>>,
}

impl<V> WasmHost<V> {
    pub fn push_memory(&self, memory: Memory) -> usize {
        let mut memories = self.memories.borrow_mut();
        memories.push(MemoryEntry {
            memory,
            buffer: None,
        });
        memories.len() - 1
    }

    pub fn memory(&self, handle: usize) -> Option<Memory> {
        self.memories.borrow().get(handle).map(|e| e.memory)
    }

    /// `new WebAssembly.Memory({ initial, maximum })`, in pages.
    ///
    /// # Errors
    ///
    /// wasmi's message, for the native to throw.
    pub fn memory_new(&self, initial: u32, maximum: Option<u32>) -> Result<usize, String> {
        let ty = MemoryType::new(initial, maximum);
        let memory = Memory::new(&mut *self.store.borrow_mut(), ty).map_err(|e| e.to_string())?;
        Ok(self.push_memory(memory))
    }

    /// Grow by `delta` pages and return the old size. The caller detaches the
    /// buffer whether or not any page was added, as the spec requires.
    ///
    /// # Errors
    ///
    /// wasmi's message, for the native to throw.
    pub fn memory_grow(&self, handle: usize, delta: u32) -> Result<u64, String> {
        let memory = self.memory(handle).ok_or("not a memory")?;
        let mut store = self.store.borrow_mut();
        memory
            .grow(&mut *store, u64::from(delta))
            .map_err(|e| e.to_string())
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

    /// Run `f` on the buffer script holds for `handle`, if it is current.
    pub fn with_buffer<R>(&self, handle: usize, f: impl FnOnce(&V) -> R) -> Option<R> {
        let memories = self.memories.borrow();
        memories.get(handle)?.buffer.as_ref().map(|b| f(&b.value))
    }

    pub fn set_buffer(
        &self,
        handle: usize,
        value: V,
        (ptr, len): (*mut u8, usize),
        resizable: bool,
    ) {
        if let Some(entry) = self.memories.borrow_mut().get_mut(handle) {
            entry.buffer = Some(BufferView {
                value,
                ptr,
                len,
                resizable,
            });
        }
    }

    /// Whether the memory's current buffer is resizable; `None` if it has none.
    pub fn buffer_is_resizable(&self, handle: usize) -> Option<bool> {
        let memories = self.memories.borrow();
        memories.get(handle)?.buffer.as_ref().map(|b| b.resizable)
    }

    /// Take the memory's current buffer out, for the caller to detach.
    pub fn take_buffer(&self, handle: usize) -> Option<V> {
        let mut memories = self.memories.borrow_mut();
        memories
            .get_mut(handle)?
            .buffer
            .take()
            .map(|view| view.value)
    }

    /// Every buffer whose memory has moved or changed length, and `force`'s
    /// fixed-length buffer in any case: fixed-length ones taken out to be
    /// detached, resizable ones to be repointed.
    pub fn refresh(&self, force: Option<usize>) -> Refresh<V> {
        let store = self.store.borrow();
        let mut memories = self.memories.borrow_mut();
        let mut refresh = Refresh::default();
        for (handle, entry) in memories.iter_mut().enumerate() {
            let Some(view) = &mut entry.buffer else {
                continue;
            };
            let now = (
                entry.memory.data_ptr(&*store),
                entry.memory.data_size(&*store),
            );
            let moved = now != (view.ptr, view.len);
            if view.resizable {
                if moved {
                    (view.ptr, view.len) = now;
                    refresh.repoint.push((handle, now.0, now.1));
                }
            } else if moved || force == Some(handle) {
                refresh
                    .detach
                    .extend(entry.buffer.take().map(|view| view.value));
            }
        }
        refresh
    }

    /// The memory whose current buffer `is` picks out.
    pub fn memory_with_buffer(&self, mut is: impl FnMut(&V) -> bool) -> Option<usize> {
        let memories = self.memories.borrow();
        memories
            .iter()
            .position(|e| e.buffer.as_ref().is_some_and(|b| is(&b.value)))
    }
}
