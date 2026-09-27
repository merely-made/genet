// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Out-of-flow boxes against Chromium 152 in a 600x400 viewport: a fixed box
//! inside absolutely positioned ones keeps the viewport as its containing
//! block, and an out-of-flow child adds nothing to a shrink-to-fit width
//! (design_docs/2026-09-26_out_of_flow_placement_plan.md). Each case runs
//! through both the one-shot `layout` entry and the retained document.

use genet_livery::{Device, InteractionStates, LiveryDocument, StyleSet, layout, resolve_styles};
use genet_static_dom::StaticDocument;
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

type Rect = [f32; 4];

fn find(
    dom: &StaticDocument,
    node: <StaticDocument as LayoutDom>::NodeId,
    needle: &str,
) -> Option<<StaticDocument as LayoutDom>::NodeId> {
    if dom.kind(node) == NodeKind::Element
        && dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some(needle)
    {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find(dom, child, needle))
}

/// `#a`'s and `#f`'s rectangles through each entry point.
fn measure(body: &str) -> [(&'static str, [Option<Rect>; 2]); 2] {
    let html = format!("<!DOCTYPE html><html><body style='margin:0'>{body}</body></html>");
    let document = StaticDocument::parse(&html);
    let styles = resolve_styles(
        &document,
        &StyleSet::cambium(&[]),
        &Device::screen(600.0, 400.0),
        &InteractionStates::default(),
    );
    let fragments = layout(&document, &styles, 600.0, 400.0).expect("layout");
    let mut session = LiveryDocument::new(
        StaticDocument::parse(&html),
        StyleSet::cambium(&[]),
        Device::screen(600.0, 400.0),
    );
    session.frame(600, 400).expect("frame");
    let one_shot = ["a", "f"].map(|id| {
        find(&document, document.document(), id)
            .and_then(|node| fragments.get(node))
            .map(|fragment| {
                let rect = fragment.physical_rect();
                [rect.x, rect.y, rect.width, rect.height]
            })
    });
    let retained = ["a", "f"].map(|id| {
        find(session.dom(), session.dom().document(), id)
            .and_then(|node| session.fragment_rect(node))
    });
    [("layout", one_shot), ("document", retained)]
}

/// A case's markup, then Chromium's `#a` and `#f`.
type Case<'a> = (&'a str, &'a str, Option<Rect>, Option<Rect>);

fn assert_matches_chromium(cases: &[Case<'_>]) {
    let near = |got: Option<Rect>, want: Rect| {
        got.is_some_and(|got| {
            got.iter()
                .zip(want)
                .all(|(got, want)| (got - want).abs() <= 0.5)
        })
    };
    let failures = cases
        .iter()
        .flat_map(|(name, body, a, f)| {
            measure(body)
                .into_iter()
                .filter(|(_, [got_a, got_f])| {
                    a.is_some_and(|a| !near(*got_a, a)) || f.is_some_and(|f| !near(*got_f, f))
                })
                .map(|(entry, [got_a, got_f])| {
                    format!(
                        "{name} ({entry}): #a {got_a:?}, #f {got_f:?}; Chromium #a {a:?}, #f {f:?}"
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

const INSET: &str = "<div id=f style='position:fixed; left:0; top:0; right:0; bottom:0'></div>";

/// A fixed box's containing block is the viewport whatever its absolutely
/// positioned ancestors; on an axis whose insets are both auto it takes its
/// static position, which does move with them.
#[test]
fn fixed_boxes_inside_absolute_ones_match_chromium() {
    let viewport = [0.0, 0.0, 600.0, 400.0];
    let bottom = format!(
        "<div id=a style='position:absolute; left:0; right:0; bottom:0; height:20px'>{INSET}</div>"
    );
    let knot = format!("<div style='height:360px'></div>{bottom}");
    let offset = format!(
        "<div id=a style='position:absolute; left:30px; top:50px; width:100px; height:20px'>\
         {INSET}</div>"
    );
    let nested = format!(
        "<div style='position:absolute; top:40px; left:0; right:0; height:100px'>\
         <div id=a style='position:absolute; top:30px; left:0; right:0; height:20px'>\
         {INSET}</div></div>"
    );
    let in_flow = format!("<div style='height:300px'></div>{INSET}");
    let relative = format!(
        "<div style='height:300px'></div><div id=a style='position:relative; top:10px'>{INSET}</div>"
    );
    assert_matches_chromium(&[
        (
            "knot",
            &knot,
            Some([0.0, 380.0, 600.0, 20.0]),
            Some(viewport),
        ),
        (
            "absolute alone",
            &bottom,
            Some([0.0, 380.0, 600.0, 20.0]),
            Some(viewport),
        ),
        (
            "absolute offset",
            &offset,
            Some([30.0, 50.0, 100.0, 20.0]),
            Some(viewport),
        ),
        (
            "nested absolute",
            &nested,
            Some([0.0, 70.0, 600.0, 20.0]),
            Some(viewport),
        ),
        (
            "static position",
            "<div id=a style='position:absolute; left:30px; top:50px; width:100px; height:20px'>\
             <div id=f style='position:fixed; width:10px; height:10px'></div></div>",
            Some([30.0, 50.0, 100.0, 20.0]),
            Some([30.0, 50.0, 10.0, 10.0]),
        ),
        (
            "static block position",
            "<div id=a style='position:absolute; left:0; top:50px; height:20px'>\
             <div id=f style='position:fixed; left:0; right:0; height:10px'></div></div>",
            Some([0.0, 50.0, 0.0, 20.0]),
            Some([0.0, 50.0, 600.0, 10.0]),
        ),
        ("in flow", &in_flow, None, Some(viewport)),
        ("relative", &relative, None, Some(viewport)),
    ]);
}

/// An out-of-flow child adds nothing to its parent's shrink-to-fit width, so
/// a box holding only one is as narrow as an empty box.
#[test]
fn out_of_flow_children_leave_shrink_to_fit_widths_alone() {
    assert_matches_chromium(&[
        (
            "absolute around a fixed box",
            "<div id=a style='position:absolute; left:0; top:50px; height:20px'>\
             <div id=f style='position:fixed; left:0; top:0; width:50px; height:10px'></div></div>",
            Some([0.0, 50.0, 0.0, 20.0]),
            Some([0.0, 0.0, 50.0, 10.0]),
        ),
        (
            "absolute around an inset absolute box",
            "<div id=a style='position:absolute; left:0; top:50px; height:20px'>\
             <div id=f style='position:absolute; left:0; right:0; height:10px'></div></div>",
            Some([0.0, 50.0, 0.0, 20.0]),
            Some([0.0, 50.0, 0.0, 10.0]),
        ),
        (
            "absolute around a 50px absolute box",
            "<div id=a style='position:absolute; left:0; top:50px; height:20px'>\
             <div id=f style='position:absolute; left:0; width:50px; height:10px'></div></div>",
            Some([0.0, 50.0, 0.0, 20.0]),
            Some([0.0, 50.0, 50.0, 10.0]),
        ),
        (
            "inline-block around a fixed box",
            "<div id=a style='display:inline-block; height:20px'>\
             <div id=f style='position:fixed; left:0; right:0; height:10px'></div></div>",
            Some([0.0, 0.0, 0.0, 20.0]),
            Some([0.0, 0.0, 600.0, 10.0]),
        ),
        (
            "float around a 50px absolute box",
            "<div id=a style='float:left; height:20px'>\
             <div id=f style='position:absolute; left:0; width:50px; height:10px'></div></div>",
            Some([0.0, 0.0, 0.0, 20.0]),
            Some([0.0, 0.0, 50.0, 10.0]),
        ),
    ]);
}

/// A relative or sticky child is in flow and sizes as a static one would;
/// its offset moves it without widening its parent.
#[test]
fn relative_and_sticky_children_size_their_shrink_to_fit_parent() {
    assert_matches_chromium(&[
        (
            "relative",
            "<div id=a style='position:absolute; left:0; top:50px; height:20px'>\
             <div id=f style='position:relative; width:50px; height:10px'>x</div></div>",
            Some([0.0, 50.0, 50.0, 20.0]),
            Some([0.0, 50.0, 50.0, 10.0]),
        ),
        (
            "sticky",
            "<div id=a style='position:absolute; left:0; top:50px; height:20px'>\
             <div id=f style='position:sticky; top:0; width:50px; height:10px'>x</div></div>",
            Some([0.0, 50.0, 50.0, 20.0]),
            Some([0.0, 50.0, 50.0, 10.0]),
        ),
        (
            "relative offset",
            "<div id=a style='position:absolute; left:0; top:50px; height:20px'>\
             <div id=f style='position:relative; left:30px; width:50px; height:10px'>x</div></div>",
            Some([0.0, 50.0, 50.0, 20.0]),
            Some([30.0, 50.0, 50.0, 10.0]),
        ),
    ]);
}

/// An auto-width inline-block takes the shrink-to-fit width (CSS 2.1 10.3.9)
/// with its percentage padding resolved against its containing block (8.4),
/// through the one-shot entry as through the retained one, including a box
/// Buckram's intrinsic lane does not admit.
#[test]
fn percentage_padded_inline_blocks_shrink_to_fit_in_both_entries() {
    let padded = "display:inline-block; height:20px; padding-left:10%";
    assert_matches_chromium(&[
        (
            "empty",
            &format!("<div id=a style='{padded}'></div>"),
            Some([0.0, 0.0, 60.0, 20.0]),
            None,
        ),
        (
            "in a 300px block",
            &format!("<div style='width:300px'><div id=a style='{padded}'></div></div>"),
            Some([0.0, 0.0, 30.0, 20.0]),
            None,
        ),
        (
            "around a fixed box",
            &format!(
                "<div id=a style='{padded}'>\
                 <div id=f style='position:fixed; left:0; right:0; height:10px'></div></div>"
            ),
            Some([0.0, 0.0, 60.0, 20.0]),
            Some([0.0, 0.0, 600.0, 10.0]),
        ),
        (
            "relative",
            &format!("<div id=a style='{padded}; position:relative'></div>"),
            Some([0.0, 0.0, 60.0, 20.0]),
            None,
        ),
    ]);
}
