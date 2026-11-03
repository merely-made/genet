// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{prepared::ShapeInput, shaping::UnbrokenText};
use std::{
    collections::{BTreeMap, HashMap, hash_map::RandomState},
    hash::{BuildHasher, Hash, Hasher},
    sync::Arc,
};

// The fingerprint is scoped to one TextSystem's random seed and participates
// in equality. Keys from different systems may therefore compare unequal even
// for identical inputs; matching fingerprints still require exact input equality.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ShapeKey {
    generation: u64,
    input: Arc<ShapeInput>,
    fingerprint: u64,
}

impl ShapeKey {
    pub(super) fn fingerprint_matches(&self, generation: u64, seed: &RandomState) -> bool {
        self.generation == generation
            && self.fingerprint == seed.hash_one((&generation, self.input()))
    }
    pub(super) fn generation(&self) -> u64 {
        self.generation
    }
    pub(super) fn input(&self) -> &ShapeInput {
        self.input.as_ref()
    }
    #[cfg(test)]
    pub(super) fn force_fingerprint_from(&mut self, other: &Self) {
        self.fingerprint = other.fingerprint;
    }
    #[cfg(test)]
    pub(super) fn force_fingerprint(&mut self, fingerprint: u64) {
        self.fingerprint = fingerprint;
    }
    pub(super) fn new(generation: u64, input: ShapeInput, seed: &RandomState) -> Self {
        let fingerprint = seed.hash_one((&generation, &input));
        Self {
            generation,
            input: Arc::new(input),
            fingerprint,
        }
    }
}

impl Hash for ShapeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fingerprint.hash(state);
    }
}

#[derive(Default)]
pub(super) struct ShapeCache {
    entries: HashMap<Arc<ShapeKey>, (Arc<UnbrokenText>, u128)>,
    order: BTreeMap<u128, Arc<ShapeKey>>,
    clock: u128,
    bytes: usize,
    layout_bytes: usize,
}

fn fits(current: usize, incoming: usize, limit: usize) -> bool {
    current <= limit && incoming <= limit - current
}

impl ShapeCache {
    pub fn get(&mut self, key: &ShapeKey) -> Option<Arc<UnbrokenText>> {
        let (value, stamp) = self.entries.get_mut(key)?;
        let retained_key = self
            .order
            .remove(stamp)
            .expect("cache order matches entries");
        self.clock += 1;
        *stamp = self.clock;
        self.order.insert(*stamp, retained_key);
        Some(Arc::clone(value))
    }

    pub fn insert(
        &mut self,
        key: ShapeKey,
        value: Arc<UnbrokenText>,
        max_entries: usize,
        max_bytes: usize,
        max_layout_bytes: usize,
    ) {
        // Remove an old value even when the replacement cannot be retained.
        if let Some((old, stamp)) = self.entries.remove(&key) {
            self.order.remove(&stamp);
            self.bytes -= key.input.text.len();
            self.layout_bytes -= old.layout.vector_storage_bytes();
        }
        let bytes = key.input.text.len();
        let layout_bytes = value.layout.vector_storage_bytes();
        if max_entries == 0
            || max_bytes == 0
            || max_layout_bytes == 0
            || bytes > max_bytes
            || layout_bytes == usize::MAX
            || layout_bytes > max_layout_bytes
        {
            return;
        }
        while self.entries.len() >= max_entries
            || !fits(self.bytes, bytes, max_bytes)
            || !fits(self.layout_bytes, layout_bytes, max_layout_bytes)
        {
            self.evict_oldest();
        }
        self.clock += 1;
        let key = Arc::new(key);
        self.order.insert(self.clock, Arc::clone(&key));
        self.entries.insert(key, (value, self.clock));
        self.bytes += bytes;
        self.layout_bytes += layout_bytes;
    }

    fn evict_oldest(&mut self) {
        let (_, key) = self.order.pop_first().expect("cache order matches entries");
        self.bytes -= key.input.text.len();
        let (value, _) = self
            .entries
            .remove(key.as_ref())
            .expect("cache order matches entries");
        self.layout_bytes -= value.layout.vector_storage_bytes();
    }

    pub fn trim(&mut self, max_entries: usize, max_bytes: usize, max_layout_bytes: usize) {
        while !self.entries.is_empty()
            && (self.entries.len() > max_entries
                || self.bytes > max_bytes
                || self.layout_bytes > max_layout_bytes
                || max_entries == 0
                || max_bytes == 0
                || max_layout_bytes == 0)
        {
            self.evict_oldest();
        }
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.bytes = 0;
        self.layout_bytes = 0;
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn layout_bytes(&self) -> usize {
        self.layout_bytes
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
}

#[cfg(test)]
mod tests;
