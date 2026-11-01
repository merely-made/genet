// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{
    ShapeKey, TextShapingOptions,
    shaping::{ShapeWorker, UnbrokenText},
    stats,
};
use rayon::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
pub(super) struct ParallelShaping {
    pub(super) pool: Option<Arc<rayon::ThreadPool>>,
    workers: Vec<ShapeWorker>,
    #[cfg(test)]
    pub(super) hook: Option<Arc<dyn Fn(usize) + Send + Sync>>,
}
impl ParallelShaping {
    pub(super) fn set_pool(&mut self, pool: Option<Arc<rayon::ThreadPool>>) {
        self.workers.clear();
        self.pool = pool;
    }
    pub(super) fn scratch_len(&self) -> usize {
        self.workers.len()
    }
    pub(super) fn clear(&mut self) {
        self.workers.clear();
    }
    pub(super) fn admits(&self, keys: &[ShapeKey], options: TextShapingOptions) -> bool {
        self.pool.is_some()
            && keys.len() >= 2
            && keys.len() >= options.parallel_min_groups
            && keys.iter().map(|k| k.input.text.len()).sum::<usize>()
                >= options.parallel_min_text_bytes
    }
    pub(super) fn shape(
        &mut self,
        keys: &[ShapeKey],
        fonts: &parley::FontContext,
        timing: bool,
    ) -> Vec<(UnbrokenText, u64)> {
        let pool = self.pool.as_ref().expect("admitted pool");
        let slots = keys.len().min(pool.current_num_threads());
        self.workers.resize_with(slots, || ShapeWorker {
            font_context: fonts.clone(),
            layout_context: parley::LayoutContext::new(),
        });
        let next = AtomicUsize::new(0);
        #[cfg(test)]
        let hook = &self.hook;
        let outputs: Vec<Vec<_>> = pool.install(|| {
            self.workers[..slots]
                .par_iter_mut()
                .map(|worker| {
                    let mut results = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        if index >= keys.len() {
                            break;
                        }
                        #[cfg(test)]
                        if let Some(hook) = hook {
                            hook(index);
                        }
                        let start = timing.then(std::time::Instant::now);
                        let shaped = worker.shape(&keys[index].input);
                        results.push((index, shaped, stats::elapsed_ns(start)));
                    }
                    results
                })
                .collect()
        });
        let mut ordered: Vec<_> = outputs.into_iter().flatten().collect();
        ordered.sort_unstable_by_key(|(index, _, _)| *index);
        ordered
            .into_iter()
            .map(|(_, result, ns)| (result, ns))
            .collect()
    }
}
