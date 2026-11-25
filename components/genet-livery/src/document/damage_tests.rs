// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::frame::normalize_damage_roots;
use super::*;
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDomMut, QualName};

const SHEET: &str =
    "html, body { margin: 0; } #stage { position: relative; width: 200px; height: 200px; }
    .fc { position: absolute; left: 0; top: 0; width: 40px; height: 40px; }";

fn attr(name: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(name))
}

fn by_id(dom: &ScriptedDom, id: &str) -> NodeId {
    find_id(dom, dom.document(), id).expect("fixture node")
}

/// A nested chain of formatting contexts (outer > inner > leaf), sibling
/// contexts, and plain blocks that fall back to their nearest context.
fn fixture() -> LiveryDocument<ScriptedDom> {
    let mut dom = ScriptedDom::from_serialized_document(
        "<html><body><div id=stage>\
         <div id=outer class=fc><div id=inner class=fc><div id=leaf class=fc></div></div><p id=plain>x</p></div>\
         <div id=s1 class=fc></div><div id=s2 class=fc><b id=s2kid>y</b></div><div id=s3 class=fc></div>\
         </div></body></html>",
    );
    dom.drain_mutations(&mut Vec::new());
    let mut document = LiveryDocument::new(
        dom,
        StyleSet::cambium(&[SHEET]),
        Device::screen(200.0, 200.0),
    );
    document.frame(200, 200).expect("initial frame");
    document
}

/// The pre-batch algorithm: insert one candidate at a time, scanning every
/// kept root. Kept as the reference the linear pass must reproduce.
fn insert_one_at_a_time(dom: &ScriptedDom, roots: &mut Vec<NodeId>, candidate: NodeId) {
    let is_ancestor = |ancestor: NodeId, node: NodeId| {
        std::iter::successors(Some(node), |id| dom.parent(*id)).any(|id| id == ancestor)
    };
    if roots.iter().any(|root| is_ancestor(*root, candidate)) {
        return;
    }
    roots.retain(|root| !is_ancestor(candidate, *root));
    roots.push(candidate);
}

fn reference(dom: &ScriptedDom, candidates: &[NodeId]) -> Vec<NodeId> {
    let mut roots = Vec::new();
    for candidate in candidates {
        insert_one_at_a_time(dom, &mut roots, *candidate);
    }
    roots
}

fn permutations(items: &[NodeId]) -> Vec<Vec<NodeId>> {
    if items.len() <= 1 {
        return vec![items.to_vec()];
    }
    let mut all = Vec::new();
    for index in 0..items.len() {
        let mut rest = items.to_vec();
        let head = rest.remove(index);
        for mut tail in permutations(&rest) {
            tail.insert(0, head);
            all.push(tail);
        }
    }
    all
}

#[test]
fn normalized_roots_match_one_at_a_time_insertion_in_every_order() {
    let document = fixture();
    let dom = document.dom();
    let names = ["outer", "inner", "leaf", "s1", "s2", "plain"];
    let nodes: Vec<_> = names.iter().map(|name| by_id(dom, name)).collect();
    let mut checked = 0;
    for order in permutations(&nodes) {
        // Repeats and the document root exercise dedup and total absorption.
        for extra in [None, Some(order[0]), Some(dom.document())] {
            let candidates: Vec<_> = order.iter().copied().chain(extra).collect();
            let mut actual = candidates.clone();
            normalize_damage_roots(dom, &mut actual);
            assert_eq!(actual, reference(dom, &candidates), "{candidates:?}");
            checked += 1;
        }
    }
    assert_eq!(checked, 720 * 3);
}

#[test]
fn batch_damage_keeps_ancestor_and_drops_descendant_roots() {
    let mut document = fixture();
    let leaf = by_id(document.dom(), "leaf");
    let inner = by_id(document.dom(), "inner");
    let s2 = by_id(document.dom(), "s2");
    let s2kid = by_id(document.dom(), "s2kid");
    // The descendant is touched first, its formatting-context ancestor later,
    // then a sibling and the sibling's plain child, which share one context.
    document.mutate_dom(|dom| {
        dom.set_attribute(leaf, attr("style"), "left: 1px");
        dom.set_attribute(inner, attr("style"), "left: 2px");
        dom.set_attribute(s2kid, attr("style"), "color: red");
        dom.set_attribute(s2, attr("style"), "left: 3px");
    });
    let damage = document.last_layout_damage().expect("damage");
    assert!(!damage.full_document);
    assert_eq!(damage.roots, vec![inner, s2]);
}

#[test]
fn pending_batches_merge_and_removal_damages_the_former_parent() {
    let mut document = fixture();
    let (leaf, inner, s1, s2, s3) = {
        let dom = document.dom();
        (
            by_id(dom, "leaf"),
            by_id(dom, "inner"),
            by_id(dom, "s1"),
            by_id(dom, "s2"),
            by_id(dom, "s3"),
        )
    };
    document.mutate_dom(|dom| {
        dom.set_attribute(leaf, attr("style"), "left: 1px");
        dom.set_attribute(s1, attr("style"), "left: 1px");
        dom.set_attribute(s3, attr("style"), "left: 1px");
    });
    assert_eq!(
        document.last_layout_damage().unwrap().roots,
        vec![leaf, s1, s3]
    );

    // No frame between the batches. Removing s1 damages its former parent,
    // the stage, which absorbs the rest. A detached node still counts as live
    // to the DOM, so its carried-over root stays, as it did before.
    let stage = by_id(document.dom(), "stage");
    document.mutate_dom(|dom| {
        dom.remove_child(s1);
        dom.set_attribute(inner, attr("style"), "left: 2px");
        dom.set_attribute(s2, attr("style"), "left: 2px");
    });
    assert_eq!(
        document.last_layout_damage().unwrap().roots,
        vec![s1, stage]
    );
}

/// One container with `count` absolutely positioned children whose inline
/// style all change in a single batch: N distinct formatting roots.
fn sibling_batch_millis(count: usize) -> u128 {
    let mut html = String::from("<html><body><div id=stage>");
    for index in 0..count {
        html.push_str(&format!("<div class=fc id=n{index}></div>"));
    }
    html.push_str("</div></body></html>");
    let mut dom = ScriptedDom::from_serialized_document(&html);
    dom.drain_mutations(&mut Vec::new());
    let mut document = LiveryDocument::new(
        dom,
        StyleSet::cambium(&[SHEET]),
        Device::screen(200.0, 200.0),
    );
    document.frame(200, 200).expect("initial frame");
    let children: Vec<_> = document
        .dom()
        .flat_children(by_id(document.dom(), "stage"))
        .collect();
    assert_eq!(children.len(), count);
    let start = std::time::Instant::now();
    document.mutate_dom(|dom| {
        for (index, child) in children.iter().enumerate() {
            dom.set_attribute(
                *child,
                attr("style"),
                &format!("transform: translate({index}px, 0px)"),
            );
        }
    });
    let elapsed = start.elapsed().as_millis();
    assert_eq!(document.last_layout_damage().unwrap().roots.len(), count);
    elapsed
}

/// Run with `--ignored --nocapture`; prints `mutate_dom` time per size. The
/// 50k case spends minutes building its document, so it needs
/// `GENET_PROBE_50K=1`.
#[test]
#[ignore = "scaling probe"]
fn sibling_style_batch_scaling_probe() {
    let mut sizes = vec![2_000, 10_000];
    if std::env::var_os("GENET_PROBE_50K").is_some() {
        sizes.push(50_000);
    }
    for count in sizes {
        println!("mutate_dom N={count}: {} ms", sibling_batch_millis(count));
    }
}
