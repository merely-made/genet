use crate::{Device, LiveryDocument, StyleSet};
use genet_scripted_dom::ScriptedDom;

#[test]
fn shaping_stats_count_builds_separately_from_formats() {
    let dom = ScriptedDom::from_serialized_document(
        "<html><body><p>Plain text fixture</p></body></html>",
    );
    let mut document = LiveryDocument::new(
        dom,
        StyleSet::cambium(&["body { width: 30ch; }"]),
        Device::screen(640.0, 480.0),
    );
    document.frame(640, 480).expect("initial frame");
    let stats = document.text_system().shaping_stats();
    assert!(stats.format_requests > 0);
    assert!(stats.metric_layout_builds > 0);
    assert_eq!(
        stats.layout_builds,
        stats.cache_misses + stats.metric_layout_builds
    );
    assert!(stats.line_breaks >= stats.format_requests);
    document.frame(640, 480).expect("cached frame");
    assert_eq!(document.text_system().shaping_stats(), stats);
}

#[test]
fn shaping_options_survive_font_resource_rebuild() {
    let dom = ScriptedDom::from_serialized_document("<html><body>test</body></html>");
    let mut document =
        LiveryDocument::new(dom, StyleSet::cambium(&[]), Device::screen(100.0, 100.0));
    let options = crate::TextShapingOptions {
        retained_entries: 7,
        retained_text_bytes: 256,
        parallel_min_groups: 3,
        parallel_min_text_bytes: 100,
        record_timings: true,
    };
    document.set_text_shaping_options(options);
    document.set_font_resource("resource", vec![1]);
    assert_eq!(document.text_system().shaping_options(), options);
}

fn cached_fixture(system: &mut super::TextSystem, text: &str) {
    let input = super::prepared::ShapeInput {
        text: text.into(),
        defaults: vec![],
        spans: vec![],
        slots: vec![],
        base_level: 0,
        scale: 1.0f32.to_bits(),
        quantize: true,
    };
    let key = super::cache::ShapeKey {
        generation: system.font_generation,
        input,
    };
    let value = std::sync::Arc::new(system.worker.shape(&key.input));
    system.cache.insert(key, value, 2, 100);
}

#[test]
fn direct_font_registration_invalidates_cached_shapes() {
    let bytes = include_bytes!("../../../../tests/wpt/tests/fonts/Ahem.ttf").to_vec();
    let mut system = super::TextSystem::new();
    cached_fixture(&mut system, "before");
    let generation = system.font_generation;
    system.register_font_bytes(bytes.clone());
    assert_eq!(system.cache.len(), 0);
    assert_ne!(system.font_generation, generation);
    cached_fixture(&mut system, "again");
    let generation = system.font_generation;
    assert!(
        system
            .register_generic_font_bytes(vec![0], "sans-serif")
            .is_err()
    );
    assert_eq!(system.font_generation, generation);
    assert_eq!(system.cache.len(), 1);
    system
        .register_generic_font_bytes(bytes.clone(), "sans-serif")
        .unwrap();
    assert_eq!(system.cache.len(), 0);
    cached_fixture(&mut system, "face");
    system.register_font_face_bytes(bytes, "Fixture", &"\"liga\" 0".parse().unwrap());
    assert_eq!(system.cache.len(), 0);
}

#[test]
fn retained_lru_refreshes_on_hit() {
    let mut system = super::TextSystem::new();
    cached_fixture(&mut system, "one");
    cached_fixture(&mut system, "two");
    let key = |text: &str| super::cache::ShapeKey {
        generation: 0,
        input: super::prepared::ShapeInput {
            text: text.into(),
            defaults: vec![],
            spans: vec![],
            slots: vec![],
            base_level: 0,
            scale: 1.0f32.to_bits(),
            quantize: true,
        },
    };
    let active = system.cache.get(&key("one")).unwrap();
    let weak = std::sync::Arc::downgrade(&active);
    cached_fixture(&mut system, "three");
    assert!(system.cache.get(&key("two")).is_none());
    assert!(system.cache.get(&key("one")).is_some());
    assert!(system.cache.get(&key("three")).is_some());
    system.cache.clear();
    assert!(weak.upgrade().is_some());
    drop(active);
    assert!(weak.upgrade().is_none());
}

#[test]
fn prepared_batch_deduplicates_without_retention_and_rejects_stale_generation() {
    use super::{InlineGroupId, ShapeJob};
    let mut system = super::TextSystem::new();
    let mut options = system.shaping_options();
    options.retained_entries = 0;
    system.set_shaping_options(options);
    let input = super::prepared::ShapeInput {
        text: "same".into(),
        defaults: vec![],
        spans: vec![],
        slots: vec![],
        base_level: 0,
        scale: 1.0f32.to_bits(),
        quantize: true,
    };
    let generation = system.font_generation;
    let key = super::cache::ShapeKey { generation, input };
    let jobs = vec![
        ShapeJob {
            group: InlineGroupId(0),
            generation,
            key: key.clone(),
        },
        ShapeJob {
            group: InlineGroupId(1),
            generation,
            key: key.clone(),
        },
    ];
    let results = system.shape_batch(&jobs, generation).expect("valid batch");
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].group, InlineGroupId(0));
    assert_eq!(results[1].group, InlineGroupId(1));
    assert!(std::sync::Arc::ptr_eq(
        &results[0].shaped,
        &results[1].shaped
    ));
    assert_eq!(system.shaping_stats().unique_jobs, 1);
    assert_eq!(system.shaping_stats().batches, 1);
}

#[test]
fn prepared_batch_keeps_evicted_results_and_order() {
    use super::{InlineGroupId, ShapeJob};
    let mut system = super::TextSystem::new();
    let mut options = system.shaping_options();
    options.retained_entries = 1;
    system.set_shaping_options(options);
    let make = |text: &str| super::cache::ShapeKey {
        generation: 0,
        input: super::prepared::ShapeInput {
            text: text.into(),
            defaults: vec![],
            spans: vec![],
            slots: vec![],
            base_level: 0,
            scale: 1.0f32.to_bits(),
            quantize: true,
        },
    };
    let first = make("first");
    let second = make("second");
    let jobs = [
        ShapeJob {
            group: InlineGroupId(3),
            generation: 42,
            key: first.clone(),
        },
        ShapeJob {
            group: InlineGroupId(1),
            generation: 42,
            key: second,
        },
        ShapeJob {
            group: InlineGroupId(7),
            generation: 42,
            key: first.clone(),
        },
    ];
    let results = system.shape_batch(&jobs, 42).expect("valid batch");
    assert_eq!(
        results.iter().map(|r| r.group).collect::<Vec<_>>(),
        vec![InlineGroupId(3), InlineGroupId(1), InlineGroupId(7)]
    );
    assert_eq!(system.shaping_stats().unique_jobs, 2);
    assert!(system.cache.get(&first).is_none());
    assert!(std::sync::Arc::ptr_eq(
        &results[0].shaped,
        &results[2].shaped
    ));
    assert_eq!(std::sync::Arc::strong_count(&results[0].shaped), 2);
}

#[test]
fn prepared_batch_rejects_all_jobs_atomically() {
    use super::{InlineGroupId, ShapeBatchError, ShapeBatchErrorKind, ShapeJob};
    let input = super::prepared::ShapeInput {
        text: "ready".into(),
        defaults: vec![],
        spans: vec![],
        slots: vec![],
        base_level: 0,
        scale: 1.0f32.to_bits(),
        quantize: true,
    };
    for stale_index in [0, 1] {
        for stale_font in [false, true] {
            let mut system = super::TextSystem::new();
            let key = super::cache::ShapeKey {
                generation: system.font_generation,
                input: input.clone(),
            };
            let mut jobs = (0..3)
                .map(|i| ShapeJob {
                    group: InlineGroupId(i),
                    generation: 17,
                    key: key.clone(),
                })
                .collect::<Vec<_>>();
            if stale_font {
                jobs[stale_index].key.generation = key.generation.wrapping_add(1);
            } else {
                jobs[stale_index].generation = 18;
            }
            cached_fixture(&mut system, "ready");
            cached_fixture(&mut system, "other");
            let before_stats = system.shaping_stats();
            let before_count = system.shape_count();
            let kind = if stale_font {
                ShapeBatchErrorKind::FontGeneration
            } else {
                ShapeBatchErrorKind::PassGeneration
            };
            assert!(
                matches!(system.shape_batch(&jobs, 17), Err(ShapeBatchError { index, kind: actual }) if index == stale_index && actual == kind)
            );
            assert_eq!(system.shaping_stats(), before_stats);
            assert_eq!(system.shape_count(), before_count);
            assert_eq!(system.cache.len(), 2);
            cached_fixture(&mut system, "third");
            assert!(
                system.cache.get(&key).is_none(),
                "rejected work must not refresh LRU order"
            );
        }
    }
}

#[test]
fn prepared_batch_late_counter_counts_only_cache_misses() {
    use super::{InlineGroupId, ShapeJob};
    let mut system = super::TextSystem::new();
    let input = super::prepared::ShapeInput {
        text: "late".into(),
        defaults: vec![],
        spans: vec![],
        slots: vec![],
        base_level: 0,
        scale: 1.0f32.to_bits(),
        quantize: true,
    };
    let key = super::cache::ShapeKey {
        generation: system.font_generation,
        input,
    };
    let job = ShapeJob {
        group: InlineGroupId(0),
        generation: 7,
        key,
    };
    system
        .shape_late_batch(std::slice::from_ref(&job), 7)
        .unwrap();
    assert_eq!(system.shaping_stats().late_misses, 1);
    system
        .shape_late_batch(std::slice::from_ref(&job), 7)
        .unwrap();
    assert_eq!(system.shaping_stats().late_misses, 1);
    let before = system.shaping_stats();
    assert!(system.shape_late_batch(&[job], 8).is_err());
    assert_eq!(system.shaping_stats(), before);
}
