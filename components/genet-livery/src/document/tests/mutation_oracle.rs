// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use buckram::{BoxId, BoxOrigin, PhysicalRect};

const HTML: &str = "<html><body><main id=stage><div id=a>Alpha</div><div id=b>Beta</div><div id=layer></div><div id=top></div></main></body></html>";
const CSS: &str = "body { margin:0 } #stage { position:relative; width:200px }
    #a, #b { display:flow-root; width:140px; height:32px }
    #a::before { content:'>'; }
    #layer, #top { position:absolute; left:6px; top:4px; width:30px; height:12px; transform:translate(1px,1px) }
    #layer { background:red; z-index:1 } #top { background:blue; z-index:2 }";

#[derive(Clone, Copy)]
enum Edit {
    Style(&'static str, &'static str),
    Text(&'static str, &'static str),
    Reorder,
    Remove,
    Append,
}

impl Edit {
    fn apply(self, dom: &mut ScriptedDom) {
        match self {
            Self::Style(id, style) => dom.set_attribute(by_id(dom, id), attr("style"), style),
            Self::Text(id, value) => {
                let node = dom.dom_children(by_id(dom, id)).next().unwrap();
                dom.set_text(node, value);
            },
            Self::Reorder => {
                dom.insert_before(by_id(dom, "stage"), by_id(dom, "b"), Some(by_id(dom, "a")))
            },
            Self::Append => {
                let child = dom.create_element(QualName::new(
                    None,
                    Namespace::from("http://www.w3.org/1999/xhtml"),
                    LocalName::from("div"),
                ));
                dom.set_text_content(child, "Inserted");
                dom.append_child(by_id(dom, "stage"), child);
            },
            Self::Remove => dom.remove_child(by_id(dom, "top")),
        }
    }
}

fn document(dom: ScriptedDom) -> LiveryDocument<ScriptedDom> {
    let mut doc = LiveryDocument::new(dom, StyleSet::cambium(&[CSS]), Device::screen(240.0, 140.0));
    bind_ahem_generics(&mut doc);
    doc
}

// Compare box topology and fragments in tree order. Arena handles may differ
// after a retained subtree is reconciled with the previous frame.
fn geometry(layout: &crate::LiveryLayout<NodeId>) -> Vec<(usize, String, Vec<PhysicalRect>)> {
    fn visit(
        layout: &crate::LiveryLayout<NodeId>,
        id: BoxId,
        depth: usize,
        rows: &mut Vec<(usize, String, Vec<PhysicalRect>)>,
    ) {
        let b = &layout.boxes()[id];
        let role = match b.origin {
            BoxOrigin::Anonymous { kind, .. } => format!("anonymous {kind:?}"),
            BoxOrigin::Pseudo { pseudo, .. } => format!("pseudo {pseudo:?}"),
            BoxOrigin::Element(_) => "element".into(),
            BoxOrigin::Text(_) => "text".into(),
        };
        rows.push((
            depth,
            format!("{role} {:?}", b.display),
            layout
                .fragments()
                .fragments_for_box(id)
                .map(|f| f.physical_rect())
                .collect(),
        ));
        for &child in b.children() {
            visit(layout, child, depth + 1, rows);
        }
    }
    let mut rows = Vec::new();
    for &id in layout.boxes().roots() {
        visit(layout, id, 0, &mut rows);
    }
    rows
}

#[test]
fn retained_frames_match_cold_layout_after_each_mutation() {
    use Edit::*;
    let steps: &[(&str, &[Edit])] = &[
        ("initial", &[]),
        (
            "move and raise overlapping layer",
            &[Style("layer", "transform:translate(3px,2px);z-index:3")],
        ),
        (
            "lower overlapping layer",
            &[Style("layer", "transform:translate(3px,2px);z-index:0")],
        ),
        ("change width", &[Style("a", "width:110px")]),
        ("edit inline text", &[Text("b", "A changed label")]),
        (
            "edit text then repaint sibling",
            &[
                Text("b", "Replacement"),
                Style("a", "width:110px;background:green"),
            ],
        ),
        ("edit two siblings", &[Text("a", "One"), Text("b", "Two")]),
        (
            "edit text and move positioned sibling",
            &[Text("a", "Moved"), Style("top", "left:23px")],
        ),
        (
            "resize positioned sibling then edit text",
            &[Style("top", "left:23px; width:45px"), Text("b", "Resized")],
        ),
        ("append subtree", &[Append]),
        ("move existing subtree", &[Reorder]),
        ("hide subtree", &[Style("b", "display:none")]),
        ("restore subtree", &[Style("b", "display:flow-root")]),
        ("remove top layer", &[Remove]),
    ];
    let mut retained = document(ScriptedDom::from_serialized_document(HTML));
    retained.dom.drain_mutations(&mut Vec::new());
    let mut previous_paint: Option<LiveryPaintList> = None;
    for (i, &(label, edits)) in steps.iter().enumerate() {
        let before = retained.layout_generation();
        let local = retained.retained_root_relayout_generation;
        for edit in edits {
            retained.mutate_dom(|dom| edit.apply(dom));
        }
        let frame = retained.frame(240, 140).unwrap();
        assert!(frame.commands().iter().any(
            |cmd| matches!(cmd, paint_list_api::PaintCmd::DrawText(run) if !run.glyphs.is_empty())
        ));

        let mut dom = ScriptedDom::from_serialized_document(HTML);
        for (_, batch) in &steps[..=i] {
            for edit in *batch {
                edit.apply(&mut dom);
            }
        }
        dom.drain_mutations(&mut Vec::new());
        let mut cold = document(dom);
        let reference = cold.frame(240, 140).unwrap();
        let actual_geometry = geometry(&retained.layout.as_ref().unwrap().fragments);
        let expected_geometry = geometry(&cold.layout.as_ref().unwrap().fragments);
        let actual = frame.commands();
        let expected = reference.commands();
        let first = actual
            .iter()
            .zip(expected)
            .position(|(a, b)| a != b)
            .or_else(|| {
                (actual.len() != expected.len()).then_some(actual.len().min(expected.len()))
            });
        let same_geometry = actual_geometry == expected_geometry;
        let reused = before == retained.layout_generation();
        eprintln!(
            "{label}: geometry={same_geometry} first unequal paint command={first:?} geometry pass skipped={reused} local root relayout={}",
            retained.retained_root_relayout_generation > local
        );
        assert_eq!(actual_geometry, expected_geometry, "{label}: box geometry");
        assert!(
            first.is_none(),
            "{label}: first paint difference at {first:?}"
        );
        if label == "lower overlapping layer" {
            assert_ne!(
                previous_paint.as_ref().unwrap().commands(),
                actual,
                "z-index alone must change paint order"
            );
        }
        if i == 1 || i == 2 {
            assert!(reused, "{label} must exercise paint-order reuse");
        }
        if label == "edit inline text" {
            assert!(
                retained.retained_root_relayout_generation > local,
                "must exercise local text relayout"
            );
        }
        previous_paint = Some(frame);
    }
}
