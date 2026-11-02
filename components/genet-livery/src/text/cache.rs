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
    input: ShapeInput,
    fingerprint: u64,
}

impl ShapeKey {
    pub(super) fn generation(&self) -> u64 {
        self.generation
    }
    pub(super) fn input(&self) -> &ShapeInput {
        &self.input
    }
    #[cfg(test)]
    pub(super) fn force_fingerprint(&mut self, fingerprint: u64) {
        self.fingerprint = fingerprint;
    }
    pub(super) fn new(generation: u64, input: ShapeInput, seed: &RandomState) -> Self {
        let fingerprint = seed.hash_one((&generation, &input));
        Self {
            generation,
            input,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::{Hash, Hasher};

    #[derive(Default)]
    struct WorkCounter(usize);
    impl Hasher for WorkCounter {
        fn finish(&self) -> u64 {
            0
        }
        fn write(&mut self, bytes: &[u8]) {
            self.0 += bytes.len();
        }
    }

    #[test]
    fn repeated_key_hashes_do_not_scan_complete_text() {
        let seed = RandomState::new();
        let key = ShapeKey::new(
            0,
            ShapeInput {
                text: "text payload ".repeat(1000),
                defaults: vec![],
                spans: vec![],
                slots: vec![],
                base_level: 0,
                scale: 1.0f32.to_bits(),
                quantize: true,
            },
            &seed,
        );
        let mut counter = WorkCounter::default();
        for _ in 0..10 {
            key.hash(&mut counter);
        }
        assert!(counter.0 <= 160, "hashed {} bytes", counter.0);
    }
}
