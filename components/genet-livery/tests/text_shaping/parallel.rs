#[cfg(all(feature = "parallel-text", not(target_arch = "wasm32")))]
use super::*;

#[cfg(all(feature = "parallel-text", not(target_arch = "wasm32")))]
#[test]
fn parallel_shape_matches_serial() {
    use std::sync::Arc;
    let html = "<html><body><p>Combining e\u{301}, emoji 👩‍👩‍👧 and العربية.</p><p><span style='vertical-align:super'>nested <span style='vertical-align:sub'>office</span></span> text.</p><p>fallback 漢字 🦀</p></body></html>";
    let css = "@font-face { font-family: Fixture; src: url('fixture.ttf'); font-feature-settings: 'liga' 0; } body { font: 20px Fixture, sans-serif; } p { margin: 0; }";
    let make = || {
        let mut doc = LiveryDocument::new(
            ScriptedDom::from_serialized_document(html),
            StyleSet::cambium(&[css]),
            Device::screen(320.0, 200.0),
        );
        doc.set_text_shaping_options(genet_livery::TextShapingOptions {
            parallel_min_groups: 2,
            parallel_min_text_bytes: 0,
            ..Default::default()
        });
        doc.set_font_resource(
            "fixture.ttf",
            include_bytes!("../../../../tests/wpt/tests/fonts/Lato-Medium-Liga.ttf").to_vec(),
        );
        doc
    };
    for threads in [1, 2] {
        let mut serial = make();
        let mut parallel = make();
        parallel.set_text_shaping_pool(Some(Arc::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap(),
        )));
        for font in [
            include_bytes!("../../../../tests/wpt/tests/fonts/Lato-Medium-Liga.ttf").as_slice(),
            include_bytes!("../../../../tests/wpt/tests/fonts/Ahem.ttf").as_slice(),
        ] {
            serial.set_font_resource("fixture.ttf", font.to_vec());
            parallel.set_font_resource("fixture.ttf", font.to_vec());
            let before = parallel.text_system().shaping_stats();
            assert_eq!(font_output(&mut serial), font_output(&mut parallel));
            let after = parallel.text_system().shaping_stats();
            assert!(
                after.parallel_batches > before.parallel_batches,
                "threads={threads}, font generation did not enter pool"
            );
            assert!(after.parallel_jobs >= before.parallel_jobs + 2);
            assert_eq!(after.worker_scratch_slots, threads);
            let (serial_node, serial_offset) = serial.text_position_at_point(12.0, 12.0).unwrap();
            let (parallel_node, parallel_offset) =
                parallel.text_position_at_point(12.0, 12.0).unwrap();
            assert_eq!(serial_offset, parallel_offset);
            assert_eq!(
                serial.caret_rect(serial_node, serial_offset),
                parallel.caret_rect(parallel_node, parallel_offset)
            );
        }
    }
}

#[cfg(all(feature = "parallel-text", not(target_arch = "wasm32")))]
#[test]
fn parallel_shape_accepts_non_sync_dom() {
    use layout_dom_api::{AttributeView, LayoutDom, LocalName, Namespace, NodeKind, QualName};
    struct LocalDom(ScriptedDom, std::marker::PhantomData<std::rc::Rc<()>>);
    impl LayoutDom for LocalDom {
        type NodeId = <ScriptedDom as LayoutDom>::NodeId;
        fn document(&self) -> Self::NodeId {
            self.0.document()
        }
        fn parent(&self, id: Self::NodeId) -> Option<Self::NodeId> {
            self.0.parent(id)
        }
        fn prev_sibling(&self, id: Self::NodeId) -> Option<Self::NodeId> {
            self.0.prev_sibling(id)
        }
        fn next_sibling(&self, id: Self::NodeId) -> Option<Self::NodeId> {
            self.0.next_sibling(id)
        }
        fn dom_children(&self, id: Self::NodeId) -> impl Iterator<Item = Self::NodeId> + '_ {
            self.0.dom_children(id)
        }
        fn kind(&self, id: Self::NodeId) -> NodeKind {
            self.0.kind(id)
        }
        fn element_name(&self, id: Self::NodeId) -> Option<&QualName> {
            self.0.element_name(id)
        }
        fn attribute(&self, id: Self::NodeId, ns: &Namespace, local: &LocalName) -> Option<&str> {
            self.0.attribute(id, ns, local)
        }
        fn attributes(&self, id: Self::NodeId) -> impl Iterator<Item = AttributeView<'_>> + '_ {
            self.0.attributes(id)
        }
        fn text(&self, id: Self::NodeId) -> Option<&str> {
            self.0.text(id)
        }
    }
    let dom = LocalDom(
        ScriptedDom::from_serialized_document("<html><body><p>one</p><p>two</p></body></html>"),
        std::marker::PhantomData,
    );
    let mut doc = LiveryDocument::new(dom, StyleSet::cambium(&[]), Device::screen(320.0, 200.0));
    doc.set_text_shaping_options(genet_livery::TextShapingOptions {
        parallel_min_groups: 2,
        parallel_min_text_bytes: 0,
        ..Default::default()
    });
    doc.set_text_shaping_pool(Some(std::sync::Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap(),
    )));
    doc.frame(320, 200).unwrap();
}

#[cfg(all(feature = "parallel-text", not(target_arch = "wasm32")))]
#[test]
fn parallel_shape_font_snapshots_match() {
    use std::sync::Arc;
    let ahem = include_bytes!("../../../../tests/wpt/tests/fonts/Ahem.ttf").to_vec();
    let lato = include_bytes!("../../../../tests/wpt/tests/fonts/Lato-Medium-Liga.ttf").to_vec();
    let html = "<html><body><p>office e\u{301} 漢字 🦀 one</p><p>office e\u{301} 漢字 🦀 two</p><p>office e\u{301} 漢字 🦀 three</p></body></html>";
    let css = "@font-face { font-family: Fixture; src: url('fixture.ttf'); font-feature-settings: 'liga' 0; } body { font: 20px Fixture, sans-serif; } p { margin: 0; }";
    let options = genet_livery::TextShapingOptions {
        parallel_min_groups: 2,
        parallel_min_text_bytes: 0,
        ..Default::default()
    };
    let make = || {
        let mut d = LiveryDocument::new(
            ScriptedDom::from_serialized_document(html),
            StyleSet::cambium(&[css]),
            Device::screen(320.0, 200.0),
        );
        d.set_text_shaping_options(options);
        d
    };
    let mut serial = make();
    let mut parallel = make();
    let pool = Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap(),
    );
    let weak = Arc::downgrade(&pool);
    parallel.set_text_shaping_pool(Some(pool));
    for font in [&lato, &ahem] {
        serial.set_font_resource("fixture.ttf", font.clone());
        parallel.set_font_resource("fixture.ttf", font.clone());
        assert!(weak.upgrade().is_some());
        assert_eq!(parallel.text_system().shaping_options(), options);
        let before = parallel.text_system().shaping_stats();
        let a = font_output(&mut serial);
        let b = font_output(&mut parallel);
        let after = parallel.text_system().shaping_stats();
        assert!(after.parallel_batches > before.parallel_batches);
        assert!(after.parallel_jobs >= before.parallel_jobs + 2);
        assert_eq!(after.worker_scratch_slots, 2);
        assert_eq!(a, b);
        assert!(b.1.contains(font), "owner must retain admitted font bytes");
        for bytes in &b.1 {
            let id = bytes.iter().fold(0xcbf29ce484222325u64, |h, b| {
                (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
            });
            println!("selected font fnv64={id:016x} bytes={}", bytes.len());
        }
    }
    let replacement = Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap(),
    );
    let replacement_weak = Arc::downgrade(&replacement);
    parallel.set_text_shaping_pool(Some(replacement));
    assert!(weak.upgrade().is_none());
    serial.set_font_resource("fixture.ttf", lato.clone());
    parallel.set_font_resource("fixture.ttf", lato);
    let before = parallel.text_system().shaping_stats();
    assert_eq!(font_output(&mut serial), font_output(&mut parallel));
    let after = parallel.text_system().shaping_stats();
    assert!(after.parallel_batches > before.parallel_batches);
    assert!(after.parallel_jobs >= before.parallel_jobs + 2);
    assert_eq!(after.worker_scratch_slots, 1);
    parallel.set_text_shaping_pool(None);
    assert_eq!(
        parallel.text_system().shaping_stats().worker_scratch_slots,
        0
    );
    assert!(replacement_weak.upgrade().is_none());
    serial.set_font_resource("fixture.ttf", ahem.clone());
    parallel.set_font_resource("fixture.ttf", ahem);
    assert_eq!(font_output(&mut serial), font_output(&mut parallel));
    drop(parallel);
}

#[cfg(all(feature = "parallel-text", not(target_arch = "wasm32")))]
#[test]
fn parallel_shape_repeated_document_lifecycle() {
    use std::sync::Arc;
    for iteration in 0..8 {
        let pool = Arc::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(2)
                .build()
                .unwrap(),
        );
        let weak = Arc::downgrade(&pool);
        let mut doc = LiveryDocument::new(
            ScriptedDom::from_serialized_document(
                "<html><body><p>first unique text</p><p>second unique text</p><p>third unique text</p></body></html>",
            ),
            StyleSet::cambium(&[]),
            Device::screen(320.0, 200.0),
        );
        doc.set_text_shaping_options(genet_livery::TextShapingOptions {
            parallel_min_groups: 2,
            parallel_min_text_bytes: 0,
            ..Default::default()
        });
        doc.set_text_shaping_pool(Some(pool));
        doc.frame(320, 200).unwrap();
        let stats = doc.text_system().shaping_stats();
        assert!(
            stats.parallel_batches > 0 && stats.parallel_jobs >= 2,
            "iteration {iteration}"
        );
        assert_eq!(stats.worker_scratch_slots, 2);
        if iteration % 2 == 0 {
            doc.set_text_shaping_pool(None);
            assert_eq!(doc.text_system().shaping_stats().worker_scratch_slots, 0);
            assert!(weak.upgrade().is_none());
        }
        drop(doc);
        assert!(weak.upgrade().is_none());
    }
}
