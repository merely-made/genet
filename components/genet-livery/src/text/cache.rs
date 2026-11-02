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

    fn key(text: &str, seed: &RandomState) -> ShapeKey {
        ShapeKey::new(
            0,
            ShapeInput {
                text: text.into(),
                defaults: vec![],
                spans: vec![],
                slots: vec![],
                base_level: 0,
                scale: 1f32.to_bits(),
                quantize: true,
            },
            seed,
        )
    }

    fn value(key: &ShapeKey) -> Arc<UnbrokenText> {
        Arc::new(super::super::shaping::ShapeWorker::new().shape(key.input()))
    }

    #[test]
    fn retained_key_text_is_shared_and_touch_keeps_its_allocation() {
        let seed = RandomState::new();
        let lookup = key(&"long key ".repeat(1000), &seed);
        let mut cache = ShapeCache::default();
        cache.insert(lookup.clone(), value(&lookup), 2, usize::MAX, usize::MAX);
        let entry_text = cache.entries.keys().next().unwrap().input.text.as_ptr();
        let order_text = cache.order.values().next().unwrap().input.text.as_ptr();
        assert_eq!(entry_text, order_text, "indexes must share retained text");
        cache.get(&lookup).unwrap();
        assert_eq!(
            cache.order.values().next().unwrap().input.text.as_ptr(),
            entry_text,
            "touch must reuse retained text"
        );
    }

    #[test]
    fn retention_replacement_collision_and_limits_keep_correct_results() {
        let seed = RandomState::new();
        let a = key("aaa", &seed);
        let mut b = key("bbbb", &seed);
        b.force_fingerprint(a.fingerprint);
        let c = key("cc", &seed);
        let va = value(&a);
        let vb = value(&b);
        let vc = value(&c);
        let mut cache = ShapeCache::default();
        cache.insert(a.clone(), va.clone(), 2, 7, usize::MAX);
        cache.insert(b.clone(), vb.clone(), 2, 7, usize::MAX);
        assert_eq!((cache.len(), cache.bytes()), (2, 7));
        assert!(Arc::ptr_eq(&cache.get(&a).unwrap(), &va));
        cache.insert(c.clone(), vc.clone(), 2, 7, usize::MAX);
        assert!(cache.get(&b).is_none());
        assert_eq!((cache.len(), cache.bytes()), (2, 5));
        cache.insert(a.clone(), vb.clone(), 2, 7, usize::MAX);
        assert_eq!((cache.len(), cache.bytes()), (2, 5));
        assert!(Arc::ptr_eq(&cache.get(&a).unwrap(), &vb));
        cache.insert(b.clone(), vb.clone(), 2, 7, usize::MAX);
        assert!(cache.get(&c).is_none());
        assert_eq!((cache.len(), cache.bytes()), (2, 7));
        cache.insert(c.clone(), vc.clone(), 0, 7, usize::MAX);
        cache.insert(c.clone(), vc.clone(), 2, 0, usize::MAX);
        cache.insert(key("oversized", &seed), vc, 2, 7, usize::MAX);
        assert_eq!((cache.len(), cache.bytes()), (2, 7));
        cache.trim(0, 7, usize::MAX);
        assert_eq!((cache.len(), cache.bytes()), (0, 0));
    }

    #[test]
    fn layout_budget_admits_evicts_and_rejects_replacements() {
        let seed = RandomState::new();
        let a = key("alpha", &seed);
        let b = key("beta beta", &seed);
        let va = value(&a);
        let vb = value(&b);
        let ca = va.layout.vector_storage_bytes();
        let cb = vb.layout.vector_storage_bytes();
        assert!(ca > 0 && cb > 0);
        let mut cache = ShapeCache::default();
        cache.insert(a.clone(), va.clone(), 2, 100, ca);
        assert_eq!(cache.layout_bytes(), ca);
        assert!(Arc::ptr_eq(&cache.get(&a).unwrap(), &va));
        cache.insert(b.clone(), vb.clone(), 2, 100, ca.max(cb));
        assert_eq!(cache.layout_bytes(), cb);
        assert!(cache.get(&a).is_none());
        assert!(Arc::ptr_eq(&cache.get(&b).unwrap(), &vb));
        cache.insert(b.clone(), va.clone(), 2, 100, ca);
        assert_eq!(cache.layout_bytes(), ca);
        assert!(Arc::ptr_eq(&cache.get(&b).unwrap(), &va));
        cache.insert(b.clone(), vb.clone(), 2, 100, ca.max(cb));
        assert_eq!(cache.layout_bytes(), cb);
        assert!(Arc::ptr_eq(&cache.get(&b).unwrap(), &vb));
        cache.insert(b.clone(), vb.clone(), 2, 100, cb - 1);
        assert_eq!((cache.len(), cache.layout_bytes()), (0, 0));
        // Active references remain usable after eviction and rejection.
        assert_eq!(vb.layout.vector_storage_bytes(), cb);
        for _ in 0..3 {
            cache.insert(a.clone(), va.clone(), 2, 100, usize::MAX);
            cache.insert(b.clone(), vb.clone(), 2, 100, usize::MAX);
            assert_eq!(cache.layout_bytes(), ca + cb);
            cache.trim(2, 100, ca.max(cb));
            assert!(cache.layout_bytes() <= ca.max(cb));
            cache.clear();
            assert_eq!(
                (cache.len(), cache.bytes(), cache.layout_bytes()),
                (0, 0, 0)
            );
        }
    }

    #[test]
    fn limits_disable_and_trim_retention_independently() {
        let seed = RandomState::new();
        let k = key("distinct text", &seed);
        let v = value(&k);
        let charge = v.layout.vector_storage_bytes();
        assert!(charge > 0);
        let mut cache = ShapeCache::default();
        for (entries, text, layout) in [
            (0, 100, charge),
            (1, 0, charge),
            (1, 100, 0),
            (1, 2, charge),
            (1, 100, charge - 1),
        ] {
            cache.insert(k.clone(), v.clone(), entries, text, layout);
            assert_eq!((cache.len(), cache.layout_bytes()), (0, 0));
        }
        for (entries, text, layout) in [
            (0, 100, charge),
            (1, 0, charge),
            (1, 100, 0),
            (1, 2, charge),
            (1, 100, charge - 1),
        ] {
            cache.insert(k.clone(), v.clone(), 1, 100, charge);
            assert_eq!(cache.layout_bytes(), charge);
            cache.trim(entries, text, layout);
            assert_eq!(
                (cache.len(), cache.bytes(), cache.layout_bytes()),
                (0, 0, 0)
            );
        }
    }

    #[test]
    fn admission_handles_arithmetic_boundary_without_overflow() {
        assert!(fits(usize::MAX - 5, 5, usize::MAX));
        assert!(!fits(usize::MAX - 5, 6, usize::MAX));
        assert!(!fits(usize::MAX, 0, usize::MAX - 1));
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

    #[test]
    fn repeated_map_operations_do_not_scan_complete_text() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        #[derive(Clone)]
        struct CountingBuild(Arc<AtomicUsize>);
        struct CountingHash {
            inner: std::collections::hash_map::DefaultHasher,
            bytes: Arc<AtomicUsize>,
        }
        impl Hasher for CountingHash {
            fn finish(&self) -> u64 {
                self.inner.finish()
            }
            fn write(&mut self, bytes: &[u8]) {
                self.bytes.fetch_add(bytes.len(), Ordering::Relaxed);
                self.inner.write(bytes);
            }
        }
        impl BuildHasher for CountingBuild {
            type Hasher = CountingHash;
            fn build_hasher(&self) -> Self::Hasher {
                CountingHash {
                    inner: Default::default(),
                    bytes: self.0.clone(),
                }
            }
        }
        let seed = RandomState::new();
        let key = ShapeKey::new(
            0,
            ShapeInput {
                text: "text payload ".repeat(1000),
                defaults: vec![],
                spans: vec![],
                slots: vec![],
                base_level: 0,
                scale: 1f32.to_bits(),
                quantize: true,
            },
            &seed,
        );
        let bytes = Arc::new(AtomicUsize::new(0));
        let mut map = HashMap::with_hasher(CountingBuild(bytes.clone()));
        for value in 0..4 {
            assert_eq!(map.insert(key.clone(), value), None);
            assert_eq!(map.get(&key), Some(&value));
            assert_eq!(map.remove(&key), Some(value));
            assert_eq!(map.insert(key.clone(), value + 10), None);
            assert_eq!(map.get(&key), Some(&(value + 10)));
            assert_eq!(map.remove(&key), Some(value + 10));
        }
        let work = bytes.load(Ordering::Relaxed);
        assert!(work > 0 && work <= 512, "map hashed {work} bytes");
    }

    #[test]
    fn independently_seeded_keys_obey_eq_hash_contract() {
        let input = ShapeInput {
            text: "same input".into(),
            defaults: vec![],
            spans: vec![],
            slots: vec![],
            base_level: 0,
            scale: 1f32.to_bits(),
            quantize: true,
        };
        let seeds: Vec<_> = (0..8).map(|_| RandomState::new()).collect();
        let keys: Vec<_> = seeds
            .iter()
            .flat_map(|seed| {
                [
                    ShapeKey::new(2, input.clone(), seed),
                    ShapeKey::new(2, input.clone(), seed),
                ]
            })
            .collect();
        let map_seed = RandomState::new();
        for pair in keys.chunks_exact(2) {
            assert_eq!(pair[0], pair[1]);
        }
        for a in &keys {
            for b in &keys {
                if a == b {
                    assert_eq!(map_seed.hash_one(a), map_seed.hash_one(b));
                }
            }
        }
    }
}
