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
