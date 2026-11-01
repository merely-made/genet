// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{prepared::ShapeInput, shaping::UnbrokenText};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(super) struct ShapeKey {
    pub generation: u64,
    pub input: ShapeInput,
}

#[derive(Default)]
pub(super) struct ShapeCache {
    entries: HashMap<ShapeKey, (Arc<UnbrokenText>, u128)>,
    order: BTreeMap<u128, ShapeKey>,
    clock: u128,
    bytes: usize,
}

impl ShapeCache {
    pub fn get(&mut self, key: &ShapeKey) -> Option<Arc<UnbrokenText>> {
        let (value, stamp) = self.entries.get_mut(key)?;
        self.order.remove(stamp);
        self.clock += 1;
        *stamp = self.clock;
        self.order.insert(*stamp, key.clone());
        Some(Arc::clone(value))
    }

    pub fn insert(
        &mut self,
        key: ShapeKey,
        value: Arc<UnbrokenText>,
        max_entries: usize,
        max_bytes: usize,
    ) {
        let bytes = key.input.text.len();
        if max_entries == 0 || max_bytes == 0 || bytes > max_bytes {
            return;
        }
        if let Some((_, stamp)) = self.entries.remove(&key) {
            self.order.remove(&stamp);
            self.bytes -= bytes;
        }
        self.clock += 1;
        self.order.insert(self.clock, key.clone());
        self.entries.insert(key, (value, self.clock));
        self.bytes += bytes;
        self.trim(max_entries, max_bytes);
    }

    pub fn trim(&mut self, max_entries: usize, max_bytes: usize) {
        while self.entries.len() > max_entries
            || self.bytes > max_bytes
            || (max_bytes == 0 && !self.entries.is_empty())
        {
            let (_, key) = self.order.pop_first().expect("cache order matches entries");
            self.bytes -= key.input.text.len();
            self.entries.remove(&key);
        }
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.bytes = 0;
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}
