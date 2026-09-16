// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pointer hits on form controls through the shared paint/input placement.
//! Livery gives an unsized control no intrinsic size, so its border box has
//! zero extent; genet-wpt's testdriver aims at that box's layout centre.
//! Expected document points are computed here from layout rects and literal
//! CSS, independently of the placement inverse.

use std::collections::HashMap;

use genet_livery::{
    InteractionStates, LiveryLayout, StylePlane, StyleSet, hit_test_with_scroll, layout,
    resolve_styles,
};
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::LayoutDom;
use livery::media::Device;

type Node = <ScriptedDom as LayoutDom>::NodeId;

/// Each row of the 2026-09-16 regression table with its pre-T3 target.
const CONTROLS: [(&str, &str); 9] = [
    (
        r#"<input id="target" style="margin: 20px">"#,
        "INPUT#target",
    ),
    (
        r#"<input id="target" style="margin: 20px; display: block">"#,
        "INPUT#target",
    ),
    (
        r#"<div id="wrap" style="padding: 30px"><input id="target"></div>"#,
        "INPUT#target",
    ),
    (
        r#"<input id="target" style="margin: 20px; width: 150px; height: 40px">"#,
        "INPUT#target",
    ),
    (
        r#"<input id="target" type="checkbox" style="margin: 20px">"#,
        "INPUT#target",
    ),
    (
        r#"<input id="target" type="button" value="press" style="margin: 20px">"#,
        "INPUT#target",
    ),
    (
        r#"<textarea id="target" style="margin: 20px"></textarea>"#,
        "TEXTAREA#target",
    ),
    (
        r#"<button id="target" style="margin: 20px">press</button>"#,
        "BUTTON#target",
    ),
    (
        r#"<select id="target" style="margin: 20px"><option>one</option></select>"#,
        "OPTION",
    ),
];

struct Page {
    dom: ScriptedDom,
    styles: StylePlane<Node>,
    fragments: LiveryLayout<Node>,
    scroll: HashMap<Node, (f32, f32)>,
}

impl Page {
    fn new(body: &str) -> Self {
        let dom = ScriptedDom::from_serialized_document(&format!(
            "<!DOCTYPE html><html><head><title>controls</title></head><body>{body}</body></html>"
        ));
        let styles = resolve_styles(
            &dom,
            &StyleSet::cambium(&[]),
            &Device::screen(800.0, 600.0),
            &InteractionStates::default(),
        );
        let fragments = layout(&dom, &styles, 800.0, 600.0).unwrap();
        Self {
            dom,
            styles,
            fragments,
            scroll: HashMap::new(),
        }
    }

    fn node(&self, id: &str) -> Node {
        fn find(dom: &ScriptedDom, node: Node, id: &str) -> Option<Node> {
            if dom
                .attributes(node)
                .any(|a| a.name.local.as_ref() == "id" && a.value == id)
            {
                return Some(node);
            }
            dom.dom_children(node)
                .find_map(|child| find(dom, child, id))
        }
        find(&self.dom, self.dom.document(), id).unwrap()
    }

    fn rect(&self, id: &str) -> (f32, f32, f32, f32) {
        let fragment = self.fragments.get(self.node(id)).unwrap();
        (fragment.x, fragment.y, fragment.width, fragment.height)
    }

    /// genet-wpt's element-origin resolver: the untransformed layout centre.
    fn centre(&self, id: &str) -> (f32, f32) {
        let (x, y, width, height) = self.rect(id);
        (x + width / 2.0, y + height / 2.0)
    }

    fn hit(&self, (x, y): (f32, f32)) -> Option<String> {
        let node =
            hit_test_with_scroll(&self.dom, &self.styles, &self.fragments, &self.scroll, x, y)?;
        let name = self
            .dom
            .element_name(node)?
            .local
            .as_ref()
            .to_ascii_uppercase();
        Some(
            match self
                .dom
                .attributes(node)
                .find(|a| a.name.local.as_ref() == "id")
            {
                Some(id) => format!("{name}#{}", id.value),
                None => name,
            },
        )
    }
}

#[test]
fn each_control_is_hit_at_its_layout_centre() {
    for (control, expected) in CONTROLS {
        let page = Page::new(control);
        let centre = page.centre("target");
        assert_eq!(page.hit(centre).as_deref(), Some(expected), "{control}");
    }
}

#[test]
fn a_zero_extent_box_holds_only_its_own_point_or_line() {
    let page = Page::new(CONTROLS[0].0);
    assert_eq!(page.rect("target").2, 0.0, "unsized input has no width");
    assert_eq!(page.rect("target").3, 0.0, "unsized input has no height");
    let (x, y) = page.centre("target");
    for point in [(x + 1.0, y), (x - 1.0, y), (x, y + 1.0), (x, y - 1.0)] {
        assert_ne!(
            page.hit(point).as_deref(),
            Some("INPUT#target"),
            "{point:?}"
        );
    }

    let page = Page::new(CONTROLS[1].0);
    let (left, top, width, height) = page.rect("target");
    assert!(
        width > 2.0 && height == 0.0,
        "block input is a horizontal line"
    );
    for point in [
        (left, top),
        (left + width / 2.0, top),
        (left + width - 1.0, top),
    ] {
        assert_eq!(
            page.hit(point).as_deref(),
            Some("INPUT#target"),
            "{point:?}"
        );
    }
    for point in [
        (left + width, top),
        (left + 1.0, top + 1.0),
        (left + 1.0, top - 1.0),
    ] {
        assert_ne!(
            page.hit(point).as_deref(),
            Some("INPUT#target"),
            "{point:?}"
        );
    }

    // A box with extent keeps half-open ownership of its far edges.
    let page = Page::new(CONTROLS[3].0);
    let (left, top, width, height) = page.rect("target");
    assert_eq!(page.hit((left, top)).as_deref(), Some("INPUT#target"));
    assert_ne!(
        page.hit((left + width, top + 1.0)).as_deref(),
        Some("INPUT#target")
    );
    assert_ne!(
        page.hit((left + 1.0, top + height)).as_deref(),
        Some("INPUT#target")
    );
}

#[test]
fn transformed_controls_are_hit_where_paint_places_them() {
    for (control, expected) in CONTROLS {
        // Exact in f32: translation, a quarter turn written as a matrix, scale 2.
        let page = Page::new(&format!(
            r#"<div id="frame" style="width: 250px; transform-origin: 0 0; transform: translate(200px, 30px) matrix(0, 1, -1, 0, 0, 0) scale(2)">{control}</div>"#
        ));
        let (ox, oy, _, _) = page.rect("frame");
        let (px, py) = page.centre("target");
        // origin + translate + quarter turn of (2 * offset): (x, y) -> (-y, x).
        let painted = (ox + 200.0 - 2.0 * (py - oy), oy + 30.0 + 2.0 * (px - ox));
        assert_eq!(page.hit(painted).as_deref(), Some(expected), "{control}");
        assert_ne!(
            page.hit((px, py)).as_deref(),
            Some(expected),
            "untransformed layout centre of {control}"
        );
    }
}

#[test]
fn scrolled_controls_are_hit_where_paint_places_them_and_clip_outside() {
    for (control, expected) in CONTROLS {
        // Nothing follows the control: an empty block's margins collapse
        // through, and a following block's top edge would own its line.
        let mut page = Page::new(&format!(
            r#"<div id="header" style="height: 150px"></div><div id="scroller" style="overflow: auto; width: 250px; height: 120px"><div style="height: 80px"></div>{control}</div>"#
        ));
        let scroller = page.node("scroller");
        let (_, top, _, height) = page.rect("scroller");
        let (px, py) = page.centre("target");

        page.scroll.insert(scroller, (0.0, 50.0));
        let visible = (px, py - 50.0);
        assert!(
            visible.1 > top && visible.1 < top + height,
            "{control} is scrolled into view"
        );
        assert_eq!(page.hit(visible).as_deref(), Some(expected), "{control}");
        assert_ne!(
            page.hit((px, py)).as_deref(),
            Some(expected),
            "unscrolled layout centre of {control}"
        );

        // Scrolled above the scrollport: the point is inside the document but
        // outside the scroller's clip, so the control must not be hit.
        let offset = py - top + 20.0;
        page.scroll.insert(scroller, (0.0, offset));
        let clipped = (px, py - offset);
        assert!(clipped.1 < top && clipped.1 > 0.0, "{control} is clipped");
        assert_ne!(
            page.hit(clipped).as_deref(),
            Some(expected),
            "{control} clipped"
        );
    }
}
