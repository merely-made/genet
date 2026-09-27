use genet_livery::{Device, LiveryDocument, StyleSet};
use genet_static_dom::StaticDocument;
use layout_dom_api::LayoutDom;
use paint_list_api::{PaintCmd, PaintList};

fn document(body: &str, css: &str) -> LiveryDocument<StaticDocument> {
    LiveryDocument::new(
        StaticDocument::parse(&format!("<!doctype html><html><body>{body}</body></html>")),
        StyleSet::cambium(&[
            "html,body { margin:0 } .target { font-size:20px; display:inline; }",
            css,
        ]),
        Device::screen(320.0, 200.0),
    )
}

fn glyphs(doc: &mut LiveryDocument<StaticDocument>) -> usize {
    let paint = doc.frame(320, 200).unwrap();
    paint
        .commands()
        .iter()
        .map(|cmd| match cmd {
            PaintCmd::DrawText(run) => run.glyphs.len(),
            _ => 0,
        })
        .sum()
}

fn painted_glyphs(doc: &mut LiveryDocument<StaticDocument>) -> Vec<(u32, f32, f32)> {
    doc.frame(320, 200)
        .unwrap()
        .commands()
        .iter()
        .flat_map(|cmd| match cmd {
            PaintCmd::DrawText(run) => run
                .glyphs
                .iter()
                .map(|glyph| (glyph.index, glyph.point.x, glyph.point.y))
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        })
        .collect()
}

#[test]
fn generated_text_paints_like_literal_text_and_does_not_change_dom() {
    let mut generated = document(
        "<span class='target' data-label='right'>middle</span>",
        ".target::before { content:'left '; } .target:after { content:' ' attr(data-label); }",
    );
    let mut literal = document("<span class='target'>left middle right</span>", "");
    let count = glyphs(&mut generated);
    assert!(count > 0);
    assert_eq!(count, glyphs(&mut literal));
    assert_eq!(painted_glyphs(&mut generated), painted_glyphs(&mut literal));
    let id = generated
        .dom()
        .first_with_class(generated.dom().document(), "target")
        .unwrap();
    let children: Vec<_> = generated.dom().dom_children(id).collect();
    assert_eq!(children.len(), 1);
    assert_eq!(generated.dom().text(children[0]), Some("middle"));
    assert_eq!(
        generated.generated_text(id),
        ("left ".into(), " right".into())
    );
    assert_eq!(
        glyphs(&mut generated),
        count,
        "retained frame preserves text"
    );
    let range = genet_livery::TextRange {
        anchor_node: children[0],
        anchor_offset: 0,
        focus_node: children[0],
        focus_offset: 6,
    };
    assert_eq!(generated.selection_for_range(range).unwrap().text, "middle");
    let fake = genet_livery::TextRange {
        anchor_node: id,
        anchor_offset: 0,
        focus_node: id,
        focus_offset: 4,
    };
    assert!(
        generated.selection_for_range(fake).is_none(),
        "generated glyphs are not DOM byte offsets"
    );
}

#[test]
fn pseudo_cascade_does_not_leak_to_owner_and_none_suppresses() {
    let mut doc = document(
        "<span class='target'>middle</span>",
        ".target::before { content:'wrong'; color:red } .target:before { content:'prefix'; } .target::after { content:'suffix' } .target::after { content:none }",
    );
    glyphs(&mut doc);
    let id = doc
        .dom()
        .first_with_class(doc.dom().document(), "target")
        .unwrap();
    assert_eq!(doc.generated_text(id), ("prefix".into(), String::new()));
    assert_eq!(doc.computed_style(id, "content").as_deref(), Some("normal"));
    assert_ne!(
        doc.computed_style(id, "color").as_deref(),
        Some("rgb(255, 0, 0)")
    );
}

#[test]
fn attr_and_rule_mutations_refresh_generated_text() {
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::{LayoutDomMut, LocalName, Namespace, NodeKind, QualName};
    let dom = ScriptedDom::from_serialized_document(
        "<html><body><span data-label='one'>body</span></body></html>",
    );
    fn find(
        dom: &ScriptedDom,
        node: <ScriptedDom as LayoutDom>::NodeId,
    ) -> Option<<ScriptedDom as LayoutDom>::NodeId> {
        if dom.kind(node) == NodeKind::Element
            && dom
                .attribute(node, &Namespace::default(), &LocalName::from("data-label"))
                .is_some()
        {
            return Some(node);
        }
        dom.dom_children(node).find_map(|child| find(dom, child))
    }
    let id = find(&dom, dom.document()).unwrap();
    let mut doc = LiveryDocument::new(
        dom,
        StyleSet::cambium(&["span::before { content:attr(DATA-LABEL) }"]),
        Device::screen(320.0, 200.0),
    );
    doc.frame(320, 200).unwrap();
    assert_eq!(doc.generated_text(id).0, "one");
    doc.mutate_dom(|dom| {
        dom.set_attribute(
            id,
            QualName::new(None, Namespace::default(), LocalName::from("data-label")),
            "two",
        )
    });
    doc.frame(320, 200).unwrap();
    assert_eq!(doc.generated_text(id).0, "two");
    doc.mutate_dom(|dom| {
        dom.remove_attribute(
            id,
            QualName::new(None, Namespace::default(), LocalName::from("data-label")),
        )
    });
    doc.frame(320, 200).unwrap();
    assert_eq!(doc.generated_text(id).0, "");
}

#[test]
fn contents_owner_keeps_text_but_suppressed_owner_does_not() {
    let mut contents = document(
        "<span class='target'>body</span>",
        ".target {display:contents} .target::before {content:'prefix'}",
    );
    glyphs(&mut contents);
    let id = contents
        .dom()
        .first_with_class(contents.dom().document(), "target")
        .unwrap();
    assert_eq!(contents.generated_text(id).0, "prefix");
    let mut hidden = document(
        "<div style='display:none'><span class='target'>body</span></div>",
        ".target::before {content:'prefix'}",
    );
    glyphs(&mut hidden);
    let id = hidden
        .dom()
        .first_with_class(hidden.dom().document(), "target")
        .unwrap();
    assert_eq!(hidden.generated_text(id), (String::new(), String::new()));
}

#[test]
fn pseudo_only_text_and_wrapping_match_literal_references() {
    for (body, reference, css) in [
        (
            "<span class='target'></span>",
            "<span class='target'>only text</span>",
            ".target::before {content:'only text'}",
        ),
        (
            "<span class='target'>middle </span>",
            "<span class='target'>left middle right end</span>",
            ".target::before {content:var(--prefix)} .target::after {content:'right end'}",
        ),
    ] {
        let base = ".target { display:block; width:65px; --prefix:'left '; }";
        let mut generated = document(body, &format!("{base} {css}"));
        let mut literal = document(reference, base);
        let glyphs = painted_glyphs(&mut generated);
        assert!(!glyphs.is_empty());
        assert_eq!(glyphs, painted_glyphs(&mut literal));
    }
}

#[test]
fn hidden_generated_text_keeps_layout_but_is_not_named_or_visible() {
    let mut doc = document(
        "<span class='target'>body</span>",
        ".target::before {content:'hidden';visibility:hidden}",
    );
    let paint = doc.frame(320, 200).unwrap();
    assert!(paint.commands().iter().any(
        |cmd| matches!(cmd, PaintCmd::DrawText(run) if run.color.a == 0.0 && !run.glyphs.is_empty())
    ));
    let id = doc
        .dom()
        .first_with_class(doc.dom().document(), "target")
        .unwrap();
    assert_eq!(doc.generated_text(id).0, "");
}
