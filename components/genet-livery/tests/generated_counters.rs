use genet_livery::{Device, LiveryDocument, StyleSet, TextRange};
use genet_scripted_dom::ScriptedDom;
use genet_static_dom::StaticDocument;
use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace, NodeKind, QualName};
use paint_list_api::{PaintCmd, PaintList};

fn find<D: LayoutDom>(dom: &D, node: D::NodeId, id: &str) -> D::NodeId {
    fn search<D: LayoutDom>(dom: &D, node: D::NodeId, id: &str) -> Option<D::NodeId> {
        if dom.attribute(node, &Namespace::default(), &LocalName::from("id")) == Some(id) {
            return Some(node);
        }
        dom.dom_children(node)
            .find_map(|child| search(dom, child, id))
    }
    search(dom, node, id).expect("fixture node")
}

fn document(body: &str, css: &str) -> LiveryDocument<StaticDocument> {
    LiveryDocument::new(
        StaticDocument::parse(&format!("<html><body>{body}</body></html>")),
        StyleSet::cambium(&["html,body {margin:0} span {display:inline}", css]),
        Device::screen(400.0, 300.0),
    )
}

fn painted<D: LayoutDom>(doc: &mut LiveryDocument<D>) -> Vec<(u32, f32, f32)> {
    doc.frame(400, 300)
        .unwrap()
        .commands()
        .iter()
        .flat_map(|cmd| match cmd {
            PaintCmd::DrawText(run) => run
                .glyphs
                .iter()
                .map(|glyph| (glyph.index, glyph.point.x, glyph.point.y))
                .collect(),
            _ => Vec::new(),
        })
        .collect()
}

fn names(doc: &mut LiveryDocument<StaticDocument>, ids: &[&str]) -> Vec<(String, String)> {
    doc.frame(400, 300).unwrap();
    ids.iter()
        .map(|id| doc.generated_text(find(doc.dom(), doc.dom().document(), id)))
        .collect()
}

#[test]
fn reset_increment_set_defaults_duplicates_and_negative_values() {
    let mut doc = document(
        "<span id=a>A</span><span id=b>B</span><span id=c>C</span>",
        "body {counter-reset: n 99 n -2} span {counter-increment:n n 2} #b {counter-set:n 7 n -5} span::before {content:counter(n) ':'}",
    );
    assert_eq!(
        names(&mut doc, &["a", "b", "c"]),
        vec![
            ("1:".into(), "".into()),
            ("-5:".into(), "".into()),
            ("-2:".into(), "".into())
        ]
    );
    let node = find(doc.dom(), doc.dom().document(), "a");
    assert_eq!(
        doc.computed_style(node, "counter-increment").as_deref(),
        Some("n 1 n 2")
    );
}

#[test]
fn nested_counters_restore_outer_scope_and_sibling_resets_replace() {
    let mut nested = document(
        "<div class=scope><span id=a></span><div><div class=scope><span id=b></span><span id=c></span></div></div><span id=d></span></div><div class=scope><span id=e></span></div>",
        ".scope {counter-reset:n} span {counter-increment:n} span::before {content:counters(n, '.')} ",
    );
    assert_eq!(
        names(&mut nested, &["a", "b", "c", "d", "e"])
            .into_iter()
            .map(|v| v.0)
            .collect::<Vec<_>>(),
        ["1", "1.1", "1.2", "2", "1"]
    );
    let mut siblings = document(
        "<span id=a></span><span id=b></span><span id=c></span>",
        "#a {counter-reset:n 3} #b {counter-reset:n 8} #c {counter-increment:n} span::before {content:counters(n,'.')}",
    );
    assert_eq!(
        names(&mut siblings, &["a", "b", "c"])
            .into_iter()
            .map(|v| v.0)
            .collect::<Vec<_>>(),
        ["3", "8", "9"]
    );
}

#[test]
fn same_named_parent_and_sibling_instances_keep_their_creator_identity() {
    let mut doc = document(
        "<span id=a></span><span id=b></span><span id=c></span>",
        "body {counter-reset:n 10} #a {counter-reset:n 3} #b {counter-increment:n} #c {counter-reset:n 8} span::before {content:counters(n,'.')}",
    );
    assert_eq!(
        names(&mut doc, &["a", "b", "c"])
            .into_iter()
            .map(|pair| pair.0)
            .collect::<Vec<_>>(),
        ["10.3", "10.4", "10.8"]
    );
}

#[test]
fn content_instantiates_a_missing_counter_only_in_its_scope() {
    let mut doc = document(
        "<span id=a><em id=b></em></span><span id=c></span>",
        "#a::before {content:counter(n)} #b {counter-increment:n 4} #a::after {content:counter(n)} #c::before {content:counter(n)}",
    );
    assert_eq!(
        names(&mut doc, &["a", "c"]),
        vec![("0".into(), "4".into()), ("0".into(), "".into())]
    );
}

#[test]
fn previous_descendant_values_and_before_after_order_are_used() {
    let mut doc = document(
        "<div id=a><span id=b></span></div><span id=c></span>",
        "body {counter-reset:n} #a {counter-increment:n} #b {counter-increment:n} #a::before {content:counter(n);counter-increment:n} #a::after {content:counter(n);counter-increment:n} span::before {content:counter(n)}",
    );
    assert_eq!(
        names(&mut doc, &["a", "b", "c"]),
        vec![
            ("2".into(), "4".into()),
            ("3".into(), "".into()),
            ("4".into(), "".into())
        ]
    );
}

#[test]
fn no_box_suppression_missing_counter_and_saturation() {
    let mut doc = document(
        "<span id=a style='display:none'></span><div style='display:none'><span id=b></span></div><span id=c></span><span id=d></span>",
        "body {counter-reset:n 2147483647} span {counter-increment:n} #a {counter-reset:n 99} #c::before {content:counter(n) '/' counter(missing)} #c::after {content:none;counter-set:n 3} #d::before {content:counter(n)}",
    );
    assert_eq!(
        names(&mut doc, &["a", "b", "c", "d"]),
        vec![
            ("".into(), "".into()),
            ("".into(), "".into()),
            ("2147483647/0".into(), "".into()),
            ("2147483647".into(), "".into())
        ]
    );
    let mut invisible = document(
        "<span id=a></span><span id=b></span>",
        "body {counter-reset:n} #a::before {visibility:hidden;content:'';counter-increment:n} #b::before {content:counter(n)}",
    );
    assert_eq!(names(&mut invisible, &["b"])[0].0, "1");
}

#[test]
fn counters_paint_like_literal_text_and_preserve_dom_selection() {
    let mut generated = document(
        "<span id=a>First</span> <span id=b>Second</span>",
        "body {counter-reset:n} span::before {counter-increment:n;content:counter(n) '. '}",
    );
    let mut literal = document("<span>1. First</span> <span>2. Second</span>", "");
    assert_eq!(painted(&mut generated), painted(&mut literal));
    let owner = find(generated.dom(), generated.dom().document(), "a");
    let children: Vec<_> = generated.dom().dom_children(owner).collect();
    assert_eq!(children.len(), 1);
    assert_eq!(generated.dom().text(children[0]), Some("First"));
    assert_eq!(
        generated
            .selection_for_range(TextRange {
                anchor_node: children[0],
                anchor_offset: 0,
                focus_node: children[0],
                focus_offset: 5
            })
            .unwrap()
            .text,
        "First"
    );
}

#[test]
fn partial_restyle_refreshes_later_counter_users_and_matches_fresh() {
    let body =
        "<html><body><div><span id=a>A</span></div><div><span id=b>B</span></div></body></html>";
    let css = "body {counter-reset:n} div {display:block;height:40px} span {counter-increment:n} span::before {content:counter(n)}";
    let dom = ScriptedDom::from_serialized_document(body);
    let first = find(&dom, dom.document(), "a");
    let second = find(&dom, dom.document(), "b");
    let mut retained = LiveryDocument::new(
        dom,
        StyleSet::cambium(&["html,body {margin:0} span {display:inline}", css]),
        Device::screen(400.0, 300.0),
    );
    retained.frame(400, 300).unwrap();
    assert_eq!(retained.generated_text(second).0, "2");
    retained.mutate_dom(|dom| {
        dom.set_attribute(
            first,
            QualName::new(None, Namespace::default(), LocalName::from("style")),
            "counter-increment:n 15",
        )
    });
    retained.frame(400, 300).unwrap();
    assert_eq!(retained.generated_text(second).0, "16");
    let mut fresh = document(
        "<div><span id=a style='counter-increment:n 15'>A</span></div><div><span id=b>B</span></div>",
        css,
    );
    assert_eq!(
        retained.generated_text(second),
        names(&mut fresh, &["b"])[0]
    );
    assert_eq!(painted(&mut retained), painted(&mut fresh));
    // Removing the earlier box must also remove its contribution.
    retained.mutate_dom(|dom| {
        dom.set_attribute(
            first,
            QualName::new(None, Namespace::default(), LocalName::from("style")),
            "display:none;counter-increment:n 15",
        )
    });
    retained.frame(400, 300).unwrap();
    assert_eq!(retained.generated_text(second).0, "1");
    assert!(
        retained
            .dom()
            .dom_children(first)
            .all(|node| retained.dom().kind(node) == NodeKind::Text)
    );
    retained.mutate_dom(|dom| {
        dom.remove_attribute(
            first,
            QualName::new(None, Namespace::default(), LocalName::from("style")),
        )
    });
    retained.frame(400, 300).unwrap();
    assert_eq!(retained.generated_text(second).0, "2");
    retained.mutate_dom(|dom| dom.remove(first));
    retained.frame(400, 300).unwrap();
    assert_eq!(retained.generated_text(second).0, "1");
    let mut removed_fresh = document("<div></div><div><span id=b>B</span></div>", css);
    assert_eq!(painted(&mut retained), painted(&mut removed_fresh));
}
