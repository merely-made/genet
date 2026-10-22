// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Intersection Observer geometry on a retained frame, without script: the
//! implicit and explicit roots, clipping scrollers, the containing-block chain,
//! margins and the edge-inclusive rule.

use std::collections::HashMap;

use genet_livery::{
    InteractionStates, Intersection, IntersectionFrame, LiveryLayout, MarginValue, Margins,
    StylePlane, StyleSet, ZERO_MARGINS, layout, resolve_styles, scroll_metrics,
};
use genet_static_dom::StaticDocument;
use layout_dom_api::LayoutDom;
use livery::media::Device;

type Node = <StaticDocument as LayoutDom>::NodeId;

struct Fixture {
    dom: StaticDocument,
    styles: StylePlane<Node>,
    fragments: LiveryLayout<Node>,
    scroll: HashMap<Node, (f32, f32)>,
}

impl Fixture {
    fn new(html: &str, css: &str) -> Self {
        let dom = StaticDocument::parse(html);
        let styles = resolve_styles(
            &dom,
            &StyleSet::cambium(&["html, body { margin: 0; padding: 0; }", css]),
            &Device::screen(200.0, 100.0),
            &InteractionStates::default(),
        );
        let fragments = layout(&dom, &styles, 200.0, 100.0).unwrap();
        Self {
            dom,
            styles,
            fragments,
            scroll: HashMap::new(),
        }
    }

    fn node(&self, class: &str) -> Node {
        self.dom
            .first_with_class(self.dom.document(), class)
            .unwrap()
    }

    fn intersect(
        &self,
        target: &str,
        root: Option<&str>,
        viewport_scroll: (f32, f32),
        root_margin: Margins,
    ) -> Intersection {
        IntersectionFrame {
            dom: &self.dom,
            styles: &self.styles,
            fragments: &self.fragments,
            element_scroll: &self.scroll,
            viewport: (200.0, 100.0),
            viewport_scroll,
        }
        .intersection(
            self.node(target),
            root.map(|class| self.node(class)),
            &root_margin,
            &ZERO_MARGINS,
        )
    }
}

fn px(value: f32) -> Margins {
    [MarginValue::Px(value); 4]
}

const PAGE: &str = "<html><body>\
    <div class=above></div><div class=clip><div class=inner></div></div>\
    <div class=tall></div></body></html>";
const CSS: &str = ".above { height: 100px; } \
    .clip { overflow: hidden; width: 80px; height: 40px; } \
    .inner { width: 60px; height: 100px; } \
    .tall { height: 400px; }";

#[test]
fn the_implicit_root_is_the_viewport() {
    let page = Fixture::new(PAGE, CSS);
    // `.clip` starts at y = 100, exactly below the 100px viewport: edge-adjacent.
    let touching = page.intersect("clip", None, (0.0, 0.0), ZERO_MARGINS);
    assert!(touching.is_intersecting, "touching edges intersect");
    assert_eq!(touching.intersection_rect[3], 0.0, "with zero area");
    assert_eq!(touching.root_bounds, Some([0.0, 0.0, 200.0, 100.0]));
    // Scrolled 20px, it shows 20 of its 40 rows.
    let shown = page.intersect("clip", None, (0.0, 20.0), ZERO_MARGINS);
    assert!(shown.is_intersecting);
    assert_eq!(shown.target_rect, [0.0, 80.0, 80.0, 40.0]);
    assert_eq!(shown.intersection_rect, [0.0, 80.0, 80.0, 20.0]);
}

#[test]
fn root_margin_grows_the_root_and_lifts_the_far_edge_case() {
    let page = Fixture::new(PAGE, CSS);
    let apart = page.intersect("tall", None, (0.0, 0.0), ZERO_MARGINS);
    assert!(!apart.is_intersecting, "a target 140px below is out");
    assert_eq!(apart.intersection_rect, [0.0; 4]);
    let grown = page.intersect("tall", None, (0.0, 0.0), px(150.0));
    assert!(grown.is_intersecting, "a 150px margin reaches it");
    assert_eq!(grown.root_bounds, Some([-150.0, -150.0, 500.0, 400.0]));
    let percent = page.intersect(
        "tall",
        None,
        (0.0, 0.0),
        [MarginValue::Percent(200.0); 4],
    );
    assert_eq!(percent.root_bounds, Some([-400.0, -200.0, 1000.0, 500.0]));
}

#[test]
fn a_clipping_ancestor_clips_the_target() {
    let page = Fixture::new(PAGE, CSS);
    // `.inner` is 60x100 inside the 80x40 clip, which starts at y = 100.
    let inner = page.intersect("inner", None, (0.0, 90.0), ZERO_MARGINS);
    assert_eq!(inner.target_rect, [0.0, 10.0, 60.0, 100.0]);
    // The clip shows rows 10..50 of it, the viewport rows 10..100: clip wins.
    assert_eq!(inner.intersection_rect, [0.0, 10.0, 60.0, 40.0]);
}

#[test]
fn an_explicit_scrolling_root_clips_to_its_padding_box_and_follows_its_scroll() {
    let page = Fixture::new(PAGE, CSS);
    let mut scrolled = Fixture::new(PAGE, CSS);
    let clip = scrolled.node("clip");
    scrolled.scroll.insert(clip, (0.0, 70.0));
    let rest = page.intersect("inner", Some("clip"), (0.0, 90.0), ZERO_MARGINS);
    assert_eq!(rest.root_bounds, Some([0.0, 10.0, 80.0, 40.0]));
    assert_eq!(rest.intersection_rect, [0.0, 10.0, 60.0, 40.0]);
    // Scrolling the root 70px moves the 100px child up until 30px remain.
    let moved = scrolled.intersect("inner", Some("clip"), (0.0, 90.0), ZERO_MARGINS);
    assert_eq!(moved.target_rect, [0.0, -60.0, 60.0, 100.0]);
    assert_eq!(moved.intersection_rect, [0.0, 10.0, 60.0, 30.0]);
    assert!(moved.is_intersecting);
    // A target outside the root's subtree never intersects it.
    let outside = page.intersect("above", Some("clip"), (0.0, 90.0), ZERO_MARGINS);
    assert!(!outside.is_intersecting);
}

#[test]
fn an_absolute_child_skips_clips_outside_its_containing_block() {
    let page = Fixture::new(
        "<html><body><div class=clip><div class=abs></div></div></body></html>",
        ".clip { overflow: hidden; width: 50px; height: 50px; } \
         .abs { position: absolute; left: 0; top: 0; width: 100px; height: 100px; }",
    );
    // The clip is not positioned, so it does not contain the absolute box.
    let out = page.intersect("abs", None, (0.0, 0.0), ZERO_MARGINS);
    assert_eq!(out.intersection_rect, [0.0, 0.0, 100.0, 100.0]);
    let out = page.intersect("abs", Some("clip"), (0.0, 0.0), ZERO_MARGINS);
    assert!(
        !out.is_intersecting,
        "a root off the containing-block chain is never reached"
    );
}

#[test]
fn display_none_and_detached_targets_do_not_intersect() {
    let page = Fixture::new(
        "<html><body><div class=gone></div></body></html>",
        ".gone { display: none; width: 10px; height: 10px; }",
    );
    let out = page.intersect("gone", None, (0.0, 0.0), ZERO_MARGINS);
    assert!(!out.is_intersecting);
    assert_eq!(out.target_rect, [0.0; 4]);
}

#[test]
fn scroll_metrics_measure_the_padding_box_and_overflow() {
    let page = Fixture::new(PAGE, CSS);
    let metrics = scroll_metrics(
        &page.dom,
        &page.styles,
        &page.fragments,
        page.node("clip"),
    )
    .unwrap();
    assert_eq!(metrics.client, (80.0, 40.0));
    assert_eq!(metrics.scroll, (80.0, 100.0));
    assert_eq!(metrics.max_offset(), (0.0, 60.0));
}
