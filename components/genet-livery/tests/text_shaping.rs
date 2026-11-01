use genet_livery::{Device, LiveryDocument, StyleSet};
use genet_scripted_dom::ScriptedDom;
use paint_list_api::PaintList;

#[test]
fn plain_resize_reuses_unbroken_text() {
    let dom = ScriptedDom::from_serialized_document(
        "<html><body><p>One paragraph with enough words to wrap across several lines in a narrow viewport.</p></body></html>",
    );
    let mut doc = LiveryDocument::new(dom, StyleSet::cambium(&[]), Device::screen(640.0, 480.0));
    doc.frame(640, 480).unwrap();
    let first = doc.text_system().shaping_stats();
    doc.frame(180, 480).unwrap();
    doc.frame(280, 480).unwrap();
    let last = doc.text_system().shaping_stats();
    assert!(last.format_requests > first.format_requests);
    assert_eq!(
        last.layout_builds - last.metric_layout_builds,
        first.layout_builds - first.metric_layout_builds
    );
    assert!(last.cache_hits > first.cache_hits);
}

#[test]
fn retention_limits_bound_churn() {
    use genet_livery::TextShapingOptions;
    let html = "<html><body><p>first paragraph</p><p>second paragraph</p><p>third paragraph</p></body></html>";
    let make = |entries, bytes| {
        let mut doc = LiveryDocument::new(
            ScriptedDom::from_serialized_document(html),
            StyleSet::cambium(&[]),
            Device::screen(320.0, 200.0),
        );
        doc.set_text_shaping_options(TextShapingOptions {
            retained_entries: entries,
            retained_text_bytes: bytes,
            ..Default::default()
        });
        doc
    };
    let mut uncached = make(0, 0);
    let mut small = make(1, 20);
    for width in [320, 180, 240, 320] {
        let a = format!("{:?}", uncached.frame(width, 200).unwrap().commands());
        let b = format!("{:?}", small.frame(width, 200).unwrap().commands());
        assert_eq!(a, b);
        let stats = small.text_system().shaping_stats();
        assert!(stats.retained_entries <= 1 && stats.retained_text_bytes <= 20);
        assert_eq!(uncached.text_system().shaping_stats().retained_entries, 0);
    }
    small.set_text_shaping_options(TextShapingOptions {
        retained_entries: 4,
        retained_text_bytes: 2,
        ..Default::default()
    });
    small.frame(180, 200).unwrap();
    assert_eq!(small.text_system().shaping_stats().retained_entries, 0);
}

#[test]
fn font_generation_invalidates_shapes() {
    let bytes = include_bytes!("../../../tests/wpt/tests/fonts/Ahem.ttf").to_vec();
    let mut doc = LiveryDocument::new(
        ScriptedDom::from_serialized_document("<html><body>text retained here</body></html>"),
        StyleSet::cambium(&[]),
        Device::screen(320.0, 200.0),
    );
    doc.frame(320, 200).unwrap();
    assert!(doc.text_system().shaping_stats().retained_entries > 0);
    doc.set_font_resource("fixture", bytes.clone());
    assert_eq!(doc.text_system().shaping_stats().retained_entries, 0);
    doc.frame(280, 200).unwrap();
    assert!(doc.text_system().shaping_stats().retained_entries > 0);
    assert!(
        doc.replace_generic_font_resources([("sans-serif".into(), vec![0])])
            .is_err()
    );
    assert!(doc.text_system().shaping_stats().retained_entries > 0);
    doc.replace_generic_font_resources([("sans-serif".into(), bytes)])
        .unwrap();
    assert_eq!(doc.text_system().shaping_stats().retained_entries, 0);
    doc.frame(320, 200).unwrap();
    doc.replace_font_resources([]);
    assert_eq!(doc.text_system().shaping_stats().retained_entries, 0);
}

#[test]
fn text_reuse_rebinds_current_sources() {
    use genet_scripted_dom::NodeId;
    use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace, QualName};
    fn find(dom: &ScriptedDom, node: NodeId, id: &str) -> Option<NodeId> {
        if dom.attribute(node, &Namespace::default(), &LocalName::from("id")) == Some(id) {
            return Some(node);
        }
        dom.dom_children(node)
            .find_map(|child| find(dom, child, id))
    }
    let initial = "<html><body id='body'><p id='first'>identical text</p><p id='second'>identical text</p></body></html>";
    let final_html = "<html><body id='body'><p id='second'>identical text</p><p id='new'>identical text</p></body></html>";
    let styles = || {
        StyleSet::cambium(&["html, body, p { margin: 0; padding: 0; } p { line-height: 24px; }"])
    };
    let mut retained = LiveryDocument::new(
        ScriptedDom::from_serialized_document(initial),
        styles(),
        Device::screen(320.0, 200.0),
    );
    retained.frame(320, 200).unwrap();
    let first = find(retained.dom(), retained.dom().document(), "first").unwrap();
    let first_text = retained.dom().dom_children(first).next().unwrap();
    retained.mutate_dom(|dom| {
        let body = find(dom, dom.document(), "body").unwrap();
        let first = find(dom, dom.document(), "first").unwrap();
        let second = find(dom, dom.document(), "second").unwrap();
        dom.insert_before(body, second, Some(first));
        dom.remove_child(first);
        let new = dom.create_element(QualName::new(
            None,
            Namespace::from(""),
            LocalName::from("p"),
        ));
        let text = dom.create_text("identical text");
        dom.append_child(new, text);
        dom.append_child(body, new);
    });
    let paint = format!("{:?}", retained.frame(320, 200).unwrap().commands());
    let mut fresh = LiveryDocument::new(
        ScriptedDom::from_serialized_document(final_html),
        styles(),
        Device::screen(320.0, 200.0),
    );
    let expected = format!("{:?}", fresh.frame(320, 200).unwrap().commands());
    assert_eq!(paint, expected);
    assert!(retained.caret_rect(first_text, 0).is_none());
    for (y, index) in [(8.0, 0), (32.0, 1)] {
        let (node, offset) = retained.text_position_at_point(8.0, y).unwrap();
        let (_, fresh_offset) = fresh.text_position_at_point(8.0, y).unwrap();
        assert_ne!(node, first_text);
        assert_eq!(offset, fresh_offset);
        let body = find(retained.dom(), retained.dom().document(), "body").unwrap();
        let owner = retained.dom().dom_children(body).nth(index).unwrap();
        assert_eq!(node, retained.dom().dom_children(owner).next().unwrap());
        let fresh_body = find(fresh.dom(), fresh.dom().document(), "body").unwrap();
        let fresh_owner = fresh.dom().dom_children(fresh_body).nth(index).unwrap();
        let fresh_text = fresh.dom().dom_children(fresh_owner).next().unwrap();
        let range = |source| genet_livery::TextRange {
            anchor_node: source,
            anchor_offset: 0,
            focus_node: source,
            focus_offset: 9,
        };
        let selection = retained.selection_for_range(range(node)).unwrap();
        let fresh_selection = fresh.selection_for_range(range(fresh_text)).unwrap();
        assert_eq!(selection.text, fresh_selection.text);
        assert_eq!(selection.rects, fresh_selection.rects);
        assert_eq!(selection.source_nodes, vec![node]);
        assert_eq!(retained.hit_test(8.0, y), Some(owner));
        assert_eq!(fresh.hit_test(8.0, y), Some(fresh_owner));
    }
    assert!(retained.text_system().shaping_stats().cache_hits > 0);
}

#[test]
fn one_paragraph_edit_preserves_other_shapes() {
    use layout_dom_api::LayoutDom;
    let html = "<html><body><p>first paragraph to edit</p><p>second paragraph remains unchanged</p><p>third paragraph remains unchanged</p></body></html>";
    let mut doc = LiveryDocument::new(
        ScriptedDom::from_serialized_document(html),
        StyleSet::cambium(&[]),
        Device::screen(320.0, 200.0),
    );
    doc.frame(320, 200).unwrap();
    let before = doc.text_system().shaping_stats();
    doc.mutate_dom(|dom| {
        let html = dom.dom_children(dom.document()).next().unwrap();
        let body = dom
            .dom_children(html)
            .find(|node| dom.dom_children(*node).count() == 3)
            .unwrap();
        let first = dom.dom_children(body).next().unwrap();
        dom.set_text_content(first, "edited paragraph");
    });
    doc.frame(300, 200).unwrap();
    let after = doc.text_system().shaping_stats();
    assert_eq!(
        (after.layout_builds - after.metric_layout_builds)
            - (before.layout_builds - before.metric_layout_builds),
        1
    );
    assert!(after.cache_hits > before.cache_hits);
}
