// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The receipts' fixture: a windowed table on a borrowed `ScriptedDom`, the
//! cascade run by hand on each side of a batch as a host that owns its DOM
//! does, and everything a reader of the layout can observe.

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{DomMutation, LayoutDomMut, QualName};
use paint_list_api::{DeviceIntSize, PaintList};

use super::super::scroll_batch::scroll_batches;
use super::super::*;
use crate::{
    IncrementalStyle, InteractionStates, StyleSet, emit_paint_list_with_text_system, resolve_styles,
};

pub(super) const WIDTH: f32 = 400.0;
pub(super) const HEIGHT: f32 = 300.0;
pub(super) const ROW: usize = 20;
pub(super) const WINDOW: usize = 6;

pub(super) const CSS: &str = "html, body { margin: 0; padding: 0; font: 10px monospace; } \
    #body { position: relative; overflow: hidden; width: 300px; height: 100px; \
            margin: 5px; border: 2px solid black; padding: 3px; } \
    #rows { position: absolute; left: 0; width: 300px; border: 1px solid gray; padding: 2px; } \
    .row { position: absolute; left: 0; width: 290px; height: 20px; } \
    .cell { position: absolute; top: 0; height: 20px; } \
    .c0 { left: 0; width: 90px; } .c1 { left: 90px; width: 100px; } \
    .c2 { left: 190px; width: 100px; } \
    .row:nth-child(odd) { background-color: #eee; } .focus { background-color: yellow; } .wide { width: 120px; }";

pub(super) fn ahem_text_system() -> TextSystem {
    let ahem = include_bytes!("../../../../../tests/wpt/tests/fonts/Ahem.ttf");
    let mut text = TextSystem::new();
    for generic in ["serif", "sans-serif", "monospace", "system-ui"] {
        text.register_generic_font_bytes(ahem.to_vec(), generic)
            .unwrap();
    }
    text
}

pub(super) fn attr(name: &str) -> QualName {
    QualName::new(None, Namespace::from(""), LocalName::from(name))
}

pub(super) fn row_html(index: usize) -> String {
    let top = index * ROW;
    format!(
        "<div class=row id=r{index} data-row={index} style=\"top: {top}px\">\
         <div class=\"cell c0\">row {index}</div><div class=\"cell c1\">col <b>one</b></div>\
         <div class=\"cell c2\">col two</div></div>"
    )
}

/// The table at row `first`: the strip is moved up so row `first` shows.
pub(super) fn table_html(first: usize) -> String {
    let rows = (first..first + WINDOW).map(row_html).collect::<String>();
    let top = first * ROW;
    format!(
        "<html><body><div id=body><div id=rows style=\"top: -{top}px\">{rows}</div></div>\
         </body></html>"
    )
}

pub(super) fn by_id(dom: &ScriptedDom, id: &str) -> NodeId {
    let mut pending = vec![dom.document()];
    while let Some(node) = pending.pop() {
        if dom.attribute(node, &Namespace::from(""), &LocalName::from("id")) == Some(id) {
            return node;
        }
        pending.extend(dom.dom_children(node));
    }
    panic!("fixture node {id}")
}

/// Append row `index` under the strip, built detached first as a host does.
pub(super) fn insert_row(dom: &mut ScriptedDom, index: usize, before: Option<NodeId>) -> NodeId {
    let element = |dom: &mut ScriptedDom, class: &str| {
        let node = dom.create_element(QualName::new(
            None,
            Namespace::from(""),
            LocalName::from("div"),
        ));
        dom.set_attribute(node, attr("class"), class);
        node
    };
    let row = element(dom, "row");
    dom.set_attribute(row, attr("id"), &format!("r{index}"));
    dom.set_attribute(row, attr("data-row"), &index.to_string());
    dom.set_attribute(row, attr("style"), &format!("top: {}px", index * ROW));
    for (class, label) in [
        ("cell c0", None),
        ("cell c1", Some("one")),
        ("cell c2", None),
    ] {
        let cell = element(dom, class);
        match (class, label) {
            ("cell c0", _) => {
                let text = dom.create_text(&format!("row {index}"));
                dom.append_child(cell, text);
            },
            (_, Some(label)) => {
                let text = dom.create_text("col ");
                dom.append_child(cell, text);
                let bold = dom.create_element(QualName::new(
                    None,
                    Namespace::from(""),
                    LocalName::from("b"),
                ));
                let inner = dom.create_text(label);
                dom.append_child(bold, inner);
                dom.append_child(cell, bold);
            },
            _ => {
                let text = dom.create_text("col two");
                dom.append_child(cell, text);
            },
        }
        dom.append_child(row, cell);
    }
    let strip = by_id(dom, "rows");
    dom.insert_before(strip, row, before);
    row
}

pub(super) fn scroll_strip(dom: &mut ScriptedDom, first: usize) {
    let strip = by_id(dom, "rows");
    dom.set_attribute(strip, attr("style"), &format!("top: -{}px", first * ROW));
}

pub(super) struct Session {
    pub(super) dom: ScriptedDom,
    pub(super) styles_set: StyleSet,
    pub(super) styles: StylePlane<NodeId>,
    pub(super) layout: LiveryLayout<NodeId>,
    pub(super) text: TextSystem,
    /// The host's style session: the cascade and its restyled set.
    pub(super) session: IncrementalStyle<NodeId>,
}

impl Session {
    pub(super) fn new(html: &str) -> Self {
        let mut dom = ScriptedDom::from_serialized_document(html);
        dom.drain_mutations(&mut Vec::new());
        let styles_set = StyleSet::cambium(&[CSS]);
        let mut text = ahem_text_system();
        let (styles, layout) = fresh_layout(&dom, &styles_set, &mut text);
        let mut session = IncrementalStyle::new();
        session.update(
            &dom,
            &styles_set,
            &Device::screen(WIDTH, HEIGHT),
            &InteractionStates::default(),
            &[],
        );
        Self {
            dom,
            styles_set,
            styles,
            layout,
            text,
            session,
        }
    }

    /// Bring the style session up to date with the DOM's mutations.
    pub(super) fn restyle(&mut self) -> Vec<DomMutation<NodeId>> {
        let mut mutations = Vec::new();
        self.dom.drain_mutations(&mut mutations);
        self.session.update(
            &self.dom,
            &self.styles_set,
            &Device::screen(WIDTH, HEIGHT),
            &InteractionStates::default(),
            &mutations,
        );
        mutations
    }

    /// Run one batch through the borrowed retained route.
    pub(super) fn batch(&mut self, edit: impl FnOnce(&mut ScriptedDom)) -> bool {
        edit(&mut self.dom);
        let mutations = self.restyle();
        self.layout
            .relayout_borrowed_dom(
                &self.dom,
                &self.styles_set,
                self.session.styles(),
                &mut self.styles,
                self.session.restyled_elements(),
                ViewportSizes::uniform(WIDTH, HEIGHT),
                &mutations,
                &mut self.text,
                &HashMap::<String, Vec<u8>>::new(),
            )
            .expect("retained relayout")
    }

    /// Everything a reader of the layout can observe, keyed by DOM node
    /// rather than by identities only a retained layout keeps.
    pub(super) fn observed(&mut self, needles: &[String]) -> Observed {
        observe(
            &self.dom,
            &self.styles,
            &self.layout,
            &mut self.text,
            needles,
        )
    }

    pub(super) fn fresh(&mut self) -> Observed {
        let (styles, layout) = fresh_layout(&self.dom, &self.styles_set, &mut self.text);
        assert!(
            styles == self.styles,
            "the adopted plane is the one a fresh layout resolves"
        );
        let needles = needles(&self.dom);
        observe(&self.dom, &styles, &layout, &mut self.text, &needles)
    }

    /// A batch that must take the route and match a fresh layout.
    pub(super) fn admitted(&mut self, edit: impl FnOnce(&mut ScriptedDom)) {
        let before = scroll_batches();
        assert!(self.batch(edit), "the batch is admitted");
        assert_eq!(scroll_batches(), before + 1, "the scroll-batch route ran");
        let retained = self.observed(&needles(&self.dom));
        let fresh = self.fresh();
        retained.assert_eq(&fresh);
    }

    /// A batch that must be refused with the published layout untouched.
    pub(super) fn refused(&mut self, edit: impl FnOnce(&mut ScriptedDom)) {
        let needles = needles(&self.dom);
        let before = self.observed(&needles);
        let batches = scroll_batches();
        assert!(!self.batch(edit), "the batch is refused");
        assert_eq!(scroll_batches(), batches);
        // The DOM moved on, so only what the layout itself holds is compared.
        self.observed(&needles).assert_layout_eq(&before);
    }
}

pub(super) fn fresh_layout(
    dom: &ScriptedDom,
    styles_set: &StyleSet,
    text: &mut TextSystem,
) -> (StylePlane<NodeId>, LiveryLayout<NodeId>) {
    let cascade = resolve_styles(
        dom,
        styles_set,
        &Device::screen(WIDTH, HEIGHT),
        &InteractionStates::default(),
    );
    layout_with_text_system(
        dom,
        &cascade,
        WIDTH,
        HEIGHT,
        ViewportSizes::uniform(WIDTH, HEIGHT),
        text,
        &HashMap::<String, Vec<u8>>::new(),
    )
    .expect("fresh layout")
}

pub(super) struct Observed {
    paint: String,
    fragments: Vec<String>,
    static_positions: Vec<String>,
    metrics: String,
    text: Vec<String>,
    hits: Vec<Option<NodeId>>,
}

impl Observed {
    /// What the layout holds; paint and hit testing also walk the DOM.
    pub(super) fn assert_layout_eq(&self, other: &Self) {
        assert_eq!(
            self.fragments, other.fragments,
            "fragment order and geometry"
        );
        assert_eq!(self.static_positions, other.static_positions);
        assert_eq!(self.metrics, other.metrics, "scroll metrics and extent");
        assert_eq!(self.text, other.text, "text-find rects");
    }

    pub(super) fn assert_eq(&self, other: &Self) {
        self.assert_layout_eq(other);
        assert_eq!(self.hits, other.hits, "hit-test targets");
        assert_eq!(self.paint, other.paint, "paint commands");
    }
}

/// The text a receipt looks for: fixed words and each current row's label.
pub(super) fn needles(dom: &ScriptedDom) -> Vec<String> {
    let strip = by_id(dom, "rows");
    let rows = dom
        .dom_children(strip)
        .filter_map(|row| dom.attribute(row, &Namespace::from(""), &LocalName::from("data-row")))
        .map(|index| format!("row {index}"));
    ["row", "col two", "one"]
        .iter()
        .map(|needle| needle.to_string())
        .chain(rows)
        .collect()
}

pub(super) fn observe(
    dom: &ScriptedDom,
    styles: &StylePlane<NodeId>,
    layout: &LiveryLayout<NodeId>,
    text: &mut TextSystem,
    needles: &[String],
) -> Observed {
    let boxes = layout.boxes();
    let fragments = layout.fragments();
    let origin = |box_id: BoxId| format!("{:?}", boxes[box_id].origin);
    // Box-tree order is generation order, the same for both layouts of one
    // DOM; fragment order itself shows in paint and hit-test order.
    let fragment_lines = boxes
        .iter()
        .flat_map(|(box_id, _)| fragments.fragments_for_box(box_id))
        .map(|fragment| {
            let name = |id: Option<FragmentId>| {
                id.and_then(|id| fragments.get(id))
                    .map(|f| origin(f.box_id()))
            };
            format!(
                "{} {:?} overflow {:?} parent {:?} containing {:?} baselines {:?}",
                origin(fragment.box_id()),
                fragment.physical_rect(),
                fragment.overflow,
                name(fragment.parent()),
                name(fragment.containing_fragment()),
                fragment.baselines,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        fragment_lines.len(),
        fragments.len(),
        "every fragment has a box"
    );
    let mut static_positions = boxes
        .iter()
        .filter_map(|(box_id, _)| {
            let position = fragments.static_position_for_box(box_id)?;
            let source = match position.source {
                StaticPositionSource::Fragment(source) => {
                    fragments.get(source).map(|f| origin(f.box_id()))
                },
                StaticPositionSource::InitialContainingBlock => None,
            };
            Some(format!(
                "{} {:?} from {:?}",
                origin(box_id),
                position.logical_rect,
                source
            ))
        })
        .collect::<Vec<_>>();
    static_positions.sort();
    let body = by_id(dom, "body");
    let strip = by_id(dom, "rows");
    let metrics = format!(
        "{:?} {:?}",
        crate::scroll_metrics(dom, styles, layout, body),
        crate::scroll_metrics(dom, styles, layout, strip)
    );
    let text_rects = needles
        .iter()
        .map(|needle| {
            let range = layout.text_range_for_text(&needle);
            let selection = range.and_then(|range| layout.text_selection(range));
            format!("{needle}: {:?}", selection.map(|s| (s.rects, s.text)))
        })
        .collect();
    let hits = [(20.0, 15.0), (120.0, 30.0), (230.0, 60.0), (100.0, 95.0)]
        .iter()
        .map(|(x, y)| hit_test(dom, styles, layout, *x, *y))
        .collect();
    let paint = emit_paint_list_with_text_system(
        dom,
        styles,
        layout,
        DeviceIntSize::new(WIDTH as i32, HEIGHT as i32),
        1,
        text,
    );
    Observed {
        paint: format!("{:?}", paint.commands()),
        fragments: fragment_lines,
        static_positions,
        metrics,
        text: text_rects,
        hits,
    }
}
