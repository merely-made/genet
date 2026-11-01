use genet_livery::{Device, LiveryDocument, StyleSet};
use genet_scripted_dom::ScriptedDom;
use paint_list_api::PaintList;

#[test]
fn plain_resize_reuses_unbroken_text() {
    use layout_dom_api::LayoutDom;
    let html = "<html><body><p>One paragraph with enough words to wrap across several lines in a narrow viewport.</p></body></html>";
    let make = |retained: bool| {
        let mut doc = LiveryDocument::new(
            ScriptedDom::from_serialized_document(html),
            StyleSet::cambium(&[
                "html, body, p { margin: 0; padding: 0; } p { font-size: 20px; line-height: 24px; }",
            ]),
            Device::screen(640.0, 480.0),
        );
        if !retained {
            doc.set_text_shaping_options(genet_livery::TextShapingOptions {
                retained_entries: 0,
                ..Default::default()
            });
        }
        doc
    };
    let mut doc = make(true);
    let mut uncached = make(false);
    doc.frame(640, 480).unwrap();
    let first = doc.text_system().shaping_stats();
    let paragraph = |doc: &LiveryDocument<ScriptedDom>| {
        let html = doc.dom().dom_children(doc.dom().document()).next().unwrap();
        let body = doc
            .dom()
            .dom_children(html)
            .find(|n| doc.dom().dom_children(*n).next().is_some())
            .unwrap();
        doc.dom().dom_children(body).next().unwrap()
    };
    let wide_height = doc.fragment_rect(paragraph(&doc)).unwrap()[3];
    let mut heights = Vec::new();
    for width in [180, 280] {
        let actual = format!("{:?}", doc.frame(width, 480).unwrap().commands());
        let expected = format!("{:?}", uncached.frame(width, 480).unwrap().commands());
        assert_eq!(actual, expected);
        heights.push(doc.fragment_rect(paragraph(&doc)).unwrap()[3]);
        assert_eq!(
            doc.fragment_rect(paragraph(&doc)),
            uncached.fragment_rect(paragraph(&uncached))
        );
    }
    assert!(
        heights[0] > heights[1],
        "narrower width must add lines: {heights:?}"
    );
    assert!(
        heights[1] > wide_height,
        "both widths must change line placement"
    );
    let last = doc.text_system().shaping_stats();
    assert!(last.format_requests > first.format_requests);
    assert_eq!(
        last.layout_builds - last.metric_layout_builds,
        first.layout_builds - first.metric_layout_builds
    );
    assert!(last.cache_hits >= first.cache_hits + 2);
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
    let oversized = format!("{:?}", small.frame(180, 200).unwrap().commands());
    let expected = format!("{:?}", uncached.frame(180, 200).unwrap().commands());
    assert_eq!(oversized, expected);
    assert_eq!(small.text_system().shaping_stats().retained_entries, 0);
}

fn font_document(css: &str) -> LiveryDocument<ScriptedDom> {
    LiveryDocument::new(
        ScriptedDom::from_serialized_document(
            "<html><body>office fine affinity text</body></html>",
        ),
        StyleSet::cambium(&[css]),
        Device::screen(320.0, 200.0),
    )
}

fn font_output(doc: &mut LiveryDocument<ScriptedDom>) -> (String, Vec<Vec<u8>>) {
    let paint = doc.frame(320, 200).unwrap();
    (
        format!("{:?}", paint.commands()),
        paint
            .fonts()
            .iter()
            .map(|font| font.data.as_slice().to_vec())
            .collect(),
    )
}

#[test]
fn font_generation_invalidates_shapes() {
    let ahem = include_bytes!("../../../tests/wpt/tests/fonts/Ahem.ttf").to_vec();
    let lato = include_bytes!("../../../tests/wpt/tests/fonts/Lato-Medium-Liga.ttf").to_vec();
    assert_ne!(ahem, lato);
    let css = "@font-face { font-family: Fixture; src: url('fixture.ttf'); } body { font: 20px Fixture; }";
    let mut doc = font_document(css);
    doc.set_font_resource("fixture.ttf", ahem.clone());
    let original = font_output(&mut doc);
    assert!(original.1.contains(&ahem));
    // Populate and hit the cache before replacing the very same ledger key.
    doc.frame(300, 200).unwrap();
    assert!(doc.text_system().shaping_stats().cache_hits > 0);
    doc.set_font_resource("fixture.ttf", lato.clone());
    assert_eq!(doc.text_system().shaping_stats().retained_entries, 0);
    let replacement = font_output(&mut doc);
    let mut fresh = font_document(css);
    fresh.set_font_resource("fixture.ttf", lato.clone());
    assert_eq!(replacement, font_output(&mut fresh));
    assert!(replacement.1.contains(&lato));
    assert_ne!(original.0, replacement.0);
    doc.replace_font_resources([]);
    assert_eq!(doc.text_system().shaping_stats().retained_entries, 0);
    let removed = font_output(&mut doc);
    assert_eq!(removed, font_output(&mut font_document(css)));
    assert!(!removed.1.contains(&lato));

    let generic_css = "body { font: 20px sans-serif; }";
    let mut generic = font_document(generic_css);
    generic
        .replace_generic_font_resources([("sans-serif".into(), ahem.clone())])
        .unwrap();
    let ahem_output = font_output(&mut generic);
    generic.frame(300, 200).unwrap();
    let before = generic.text_system().shaping_stats();
    assert!(
        generic
            .replace_generic_font_resources([("sans-serif".into(), vec![0])])
            .is_err()
    );
    assert_eq!(generic.text_system().shaping_stats(), before);
    let after_invalid = font_output(&mut generic);
    let mut fresh_ahem = font_document(generic_css);
    fresh_ahem
        .replace_generic_font_resources([("sans-serif".into(), ahem)])
        .unwrap();
    assert_eq!(after_invalid, font_output(&mut fresh_ahem));
    assert_eq!(ahem_output, after_invalid);
    assert!(generic.text_system().shaping_stats().cache_hits > before.cache_hits);
    generic
        .replace_generic_font_resources([("sans-serif".into(), lato.clone())])
        .unwrap();
    let generic_lato = font_output(&mut generic);
    let mut fresh_lato = font_document(generic_css);
    fresh_lato
        .replace_generic_font_resources([("sans-serif".into(), lato)])
        .unwrap();
    assert_eq!(generic_lato, font_output(&mut fresh_lato));
    assert_ne!(ahem_output.0, generic_lato.0);
}

#[test]
fn face_feature_changes_match_fresh_shaping() {
    let font = include_bytes!("../../../tests/wpt/tests/fonts/Lato-Medium-Liga.ttf").to_vec();
    let face_on = "@font-face { font-family: Fixture; src: url('fixture.ttf'); font-feature-settings: 'liga' 1; }";
    let face_off = "@font-face { font-family: Fixture; src: url('fixture.ttf'); font-feature-settings: 'liga' 0; }";
    let body = "body { font: 20px Fixture; }";
    let mut doc = font_document(&format!("{face_on} {body}"));
    doc.set_font_resource("fixture.ttf", font.clone());
    let on = font_output(&mut doc);
    doc.frame(300, 200).unwrap();
    assert!(doc.text_system().shaping_stats().cache_hits > 0);
    doc.delete_author_rule(0, 0).unwrap();
    doc.insert_author_rule(0, face_off, 0).unwrap();
    let off = font_output(&mut doc);
    let mut fresh = font_document(&format!("{face_off} {body}"));
    fresh.set_font_resource("fixture.ttf", font);
    assert_eq!(off, font_output(&mut fresh));
    assert_ne!(on.0, off.0, "fixture must expose the face feature change");
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
    let before_hits = retained.text_system().shaping_stats().cache_hits;
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
    assert!(retained.text_system().shaping_stats().cache_hits > before_hits);
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
