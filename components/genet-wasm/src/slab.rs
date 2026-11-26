// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! A handle table with a free list. A handle is an index that stays put until
//! the entry's last owner releases it; only then can it be handed out again.

struct Slot<T> {
    item: Option<T>,
    refs: usize,
}

pub struct Slab<T> {
    slots: Vec<Slot<T>>,
    free: Vec<usize>,
    live: usize,
}

impl<T> Default for Slab<T> {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            free: Vec::new(),
            live: 0,
        }
    }
}

impl<T> Slab<T> {
    /// Store `item` with no owner yet and return its handle.
    pub fn insert(&mut self, item: T) -> usize {
        self.live += 1;
        let slot = Slot {
            item: Some(item),
            refs: 0,
        };
        match self.free.pop() {
            Some(handle) => {
                self.slots[handle] = slot;
                handle
            },
            None => {
                self.slots.push(slot);
                self.slots.len() - 1
            },
        }
    }

    pub fn get(&self, handle: usize) -> Option<&T> {
        self.slots.get(handle)?.item.as_ref()
    }

    pub fn get_mut(&mut self, handle: usize) -> Option<&mut T> {
        self.slots.get_mut(handle)?.item.as_mut()
    }

    /// Add an owner; `false` if there is no such entry.
    pub fn retain(&mut self, handle: usize) -> bool {
        match self.slots.get_mut(handle) {
            Some(slot) if slot.item.is_some() => {
                slot.refs += 1;
                true
            },
            _ => false,
        }
    }

    /// Drop an owner. `None` if there is no such entry; otherwise the entry
    /// itself when this was the last owner, now freed for reuse.
    pub fn release(&mut self, handle: usize) -> Option<Option<T>> {
        let slot = self.slots.get_mut(handle)?;
        slot.item.as_ref()?;
        slot.refs = slot.refs.saturating_sub(1);
        if slot.refs > 0 {
            return Some(None);
        }
        self.live -= 1;
        self.free.push(handle);
        Some(slot.item.take())
    }

    /// Free an entry whatever its owners.
    pub fn remove(&mut self, handle: usize) -> Option<T> {
        let item = self.slots.get_mut(handle)?.item.take()?;
        self.slots[handle].refs = 0;
        self.live -= 1;
        self.free.push(handle);
        Some(item)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(handle, slot)| Some((handle, slot.item.as_mut()?)))
    }

    /// Entries held now.
    pub fn len(&self) -> usize {
        self.live
    }

    pub fn is_empty(&self) -> bool {
        self.live == 0
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

impl<T: Copy> Slab<T> {
    pub fn copied(&self, handle: usize) -> Option<T> {
        self.get(handle).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::Slab;

    #[test]
    fn a_handle_is_reused_only_after_its_last_release() {
        let mut slab = Slab::default();
        let a = slab.insert("a");
        slab.retain(a);
        slab.retain(a);
        assert_eq!(slab.release(a), Some(None));
        assert_eq!(slab.insert("b"), a + 1, "still owned");
        assert_eq!(slab.release(a), Some(Some("a")));
        assert_eq!(slab.insert("c"), a, "freed, so reused");
        assert_eq!(slab.len(), 3 - 1);
        assert_eq!(slab.release(7), None);
    }
}
