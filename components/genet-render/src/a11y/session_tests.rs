/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Receipts for the retained session: after every batch the session equals a
//! full projection of the same DOM, by id with each child order exact, and a
//! patch's change set is exactly the difference between the two projections.

use std::collections::HashMap;

use document_session_api::{DocumentA11yNode, DocumentA11yNodeId, DocumentA11yProjection};
use genet_livery::{
    Device, IncrementalStyle, InteractionStates, LiveryLayout, PseudoElement, StyleSet, layout,
    rendered_visible,
};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{DomMutation, LayoutDom, LayoutDomMut};

use super::session::Refusal;
use super::table_fixture::*;
use super::{A11ySession, A11yStyleQueries, A11yUpdate, document_a11y_projection_with_style};

pub(super) struct Host {
    pub(super) dom: ScriptedDom,
    set: StyleSet,
    style: IncrementalStyle<NodeId>,
    pub(super) session: A11ySession<NodeId>,
    pub(super) focus: Option<NodeId>,
    previous: DocumentA11yProjection,
}

/// Run `f` with the style owner's queries for the current styles.
fn with_queries<R>(
    dom: &ScriptedDom,
    style: &IncrementalStyle<NodeId>,
    f: impl FnOnce(&A11yStyleQueries<'_, NodeId>) -> R,
) -> R {
    let styles = style.styles();
    let generated = |node| {
        let text = |pseudo| styles.generated_text(node, pseudo).unwrap_or("").to_owned();
        (text(PseudoElement::Before), text(PseudoElement::After))
    };
    let rendered = |node| rendered_visible(dom, styles, node);
    f(&A11yStyleQueries {
        generated: &generated,
        rendered: &rendered,
    })
}

fn device() -> Device {
    Device::screen(WIDTH as f32, HEIGHT as f32)
}

impl Host {
    pub(super) fn new(dom: ScriptedDom) -> Self {
        let set = StyleSet::cambium(CSS);
        let mut style = IncrementalStyle::new();
        style.update(&dom, &set, &device(), &InteractionStates::default(), &[]);
        let fragments = Self::layout(&dom, &style);
        let (session, previous) = with_queries(&dom, &style, |queries| {
            (
                A11ySession::new(&dom, &fragments, None, 0, None, queries),
                document_a11y_projection_with_style(&dom, &fragments, None, 0, None, queries),
            )
        });
        Self {
            dom,
            set,
            style,
            session,
            focus: None,
            previous,
        }
    }

    fn layout(dom: &ScriptedDom, style: &IncrementalStyle<NodeId>) -> LiveryLayout<NodeId> {
        layout(dom, style.styles(), WIDTH as f32, HEIGHT as f32).expect("layout")
    }

    /// Apply `edit`, restyle and lay out, then update the session.
    pub(super) fn prepare(
        &mut self,
        edit: impl FnOnce(&mut ScriptedDom),
    ) -> (Vec<DomMutation<NodeId>>, LiveryLayout<NodeId>) {
        edit(&mut self.dom);
        let mut mutations = Vec::new();
        self.dom.drain_mutations(&mut mutations);
        self.style.update(
            &self.dom,
            &self.set,
            &device(),
            &InteractionStates::default(),
            &mutations,
        );
        let fragments = Self::layout(&self.dom, &self.style);
        (mutations, fragments)
    }

    fn frame_with(
        &mut self,
        edit: impl FnOnce(&mut ScriptedDom),
        restyled: bool,
        scroll: Option<&HashMap<NodeId, (f32, f32)>>,
    ) -> A11yUpdate {
        let (mutations, fragments) = self.prepare(edit);
        let (dom, style, session, focus) = (&self.dom, &self.style, &mut self.session, self.focus);
        let restyled = restyled.then(|| style.restyled_elements()).flatten();
        let (update, full) = with_queries(dom, style, |queries| {
            let update = session.update(
                dom, &fragments, focus, 1, scroll, queries, restyled, &mutations,
            );
            let full =
                document_a11y_projection_with_style(dom, &fragments, focus, 1, scroll, queries);
            (update, full)
        });
        assert_same(&self.session.projection(), &full);
        if let A11yUpdate::Patched { .. } = &update {
            assert_eq!(
                normalized(update.clone()),
                expected_changes(&self.previous, &full)
            );
        }
        self.previous = full;
        update
    }

    /// One frame with the session's update and a full projection timed, and
    /// the restyled set's size; the session must patch.
    pub(super) fn timed(
        &mut self,
        edit: impl FnOnce(&mut ScriptedDom),
    ) -> (std::time::Duration, std::time::Duration, usize) {
        use std::time::Instant;
        let (mutations, fragments) = self.prepare(edit);
        let (dom, style, session) = (&self.dom, &self.style, &mut self.session);
        let restyled = style.restyled_elements();
        with_queries(dom, style, |queries| {
            let start = Instant::now();
            let update = session.update(
                dom, &fragments, None, 0, None, queries, restyled, &mutations,
            );
            let patched = start.elapsed();
            assert_ne!(update, A11yUpdate::Rebuilt, "{:?}", session.refusal());
            let start = Instant::now();
            std::hint::black_box(document_a11y_projection_with_style(
                dom, &fragments, None, 0, None, queries,
            ));
            (
                patched,
                start.elapsed(),
                restyled.map_or(0, |set| set.len()),
            )
        })
    }

    /// A batch the session must patch.
    pub(super) fn admitted(&mut self, edit: impl FnOnce(&mut ScriptedDom)) -> A11yUpdate {
        let update = self.frame_with(edit, true, None);
        assert_ne!(update, A11yUpdate::Rebuilt, "{:?}", self.session.refusal());
        update
    }

    /// A batch the session must refuse, for `reason`.
    pub(super) fn refused(&mut self, reason: Refusal, edit: impl FnOnce(&mut ScriptedDom)) {
        assert_eq!(self.frame_with(edit, true, None), A11yUpdate::Rebuilt);
        assert_eq!(self.session.refusal(), Some(reason));
    }
}

fn by_id_map(
    projection: &DocumentA11yProjection,
) -> HashMap<DocumentA11yNodeId, &DocumentA11yNode> {
    projection
        .nodes()
        .iter()
        .map(|node| (node.id, node))
        .collect()
}

fn assert_same(retained: &DocumentA11yProjection, full: &DocumentA11yProjection) {
    assert_eq!(retained.root(), full.root());
    assert_eq!(retained.revision(), full.revision());
    assert_eq!(retained.nodes().len(), full.nodes().len(), "node count");
    let full = by_id_map(full);
    for node in retained.nodes() {
        assert_eq!(Some(&node), full.get(&node.id), "node {:?}", node.id);
    }
}

/// The change set a patch from `old` to `new` must report.
fn expected_changes(old: &DocumentA11yProjection, new: &DocumentA11yProjection) -> A11yUpdate {
    let (old, new) = (by_id_map(old), by_id_map(new));
    let sorted = |mut ids: Vec<DocumentA11yNodeId>| {
        ids.sort();
        ids
    };
    let (mut changed, mut moved) = (Vec::new(), Vec::new());
    for (id, node) in &new {
        let Some(before) = old.get(id) else { continue };
        let mut probe = (*node).clone();
        probe.bounds = before.bounds;
        if &probe != *before {
            changed.push(*id);
        } else if node.bounds != before.bounds {
            moved.push(*id);
        }
    }
    A11yUpdate::Patched {
        added: sorted(
            new.keys()
                .filter(|id| !old.contains_key(id))
                .copied()
                .collect(),
        ),
        changed: sorted(changed),
        removed: sorted(
            old.keys()
                .filter(|id| !new.contains_key(id))
                .copied()
                .collect(),
        ),
        moved: sorted(moved),
    }
}

/// Compare change sets as sets.
fn normalized(update: A11yUpdate) -> A11yUpdate {
    match update {
        A11yUpdate::Patched {
            mut added,
            mut changed,
            mut removed,
            mut moved,
        } => {
            for ids in [&mut added, &mut changed, &mut removed, &mut moved] {
                ids.sort();
            }
            A11yUpdate::Patched {
                added,
                changed,
                removed,
                moved,
            }
        },
        rebuilt => rebuilt,
    }
}

const FIRST: usize = 500;

fn host() -> Host {
    Host::new(table_dom(FIRST))
}

fn node_id(dom: &ScriptedDom, node: NodeId) -> DocumentA11yNodeId {
    DocumentA11yNodeId::new(dom.opaque_id(node))
}

fn cell(dom: &ScriptedDom, row: usize, cell: usize) -> NodeId {
    let row = by_id(dom, &format!("r{row}"));
    dom.dom_children(row).nth(cell).expect("cell")
}

fn parts(update: A11yUpdate) -> [Vec<DocumentA11yNodeId>; 4] {
    match normalized(update) {
        A11yUpdate::Patched {
            added,
            changed,
            removed,
            moved,
        } => [added, changed, removed, moved],
        A11yUpdate::Rebuilt => panic!("rebuilt"),
    }
}

#[test]
fn the_table_scroll_batch_patches_over_several_frames() {
    let mut host = host();
    for frame in 0..4 {
        let [added, changed, removed, moved] =
            parts(host.admitted(|dom| scroll_down(dom, FIRST + frame)));
        assert_eq!(
            (added.len(), removed.len()),
            (17, 17),
            "one row in, one out"
        );
        // Only the strip's child list changes; kept rows only move.
        assert_eq!(changed, [node_id(&host.dom, by_id(&host.dom, "strip"))]);
        assert!(moved.len() > 400, "kept rows moved with the strip");
    }
    // Scrolling back inserts before the first kept row.
    host.admitted(|dom| {
        let strip = by_id(dom, "strip");
        dom.set_attribute(
            strip,
            attr("style"),
            &format!("top: -{}px", (FIRST + 3) * ROW),
        );
        dom.remove(by_id(dom, &format!("r{}", FIRST + 3 + WINDOW)));
        let first = by_id(dom, &format!("r{}", FIRST + 4));
        insert_row(dom, FIRST + 3, Some(first));
    });
}

#[test]
fn two_rows_in_and_two_out_patch() {
    let mut host = host();
    let [added, _, removed, _] = parts(host.admitted(|dom| {
        dom.remove(by_id(dom, &format!("r{FIRST}")));
        dom.remove(by_id(dom, &format!("r{}", FIRST + 1)));
        insert_row(dom, FIRST + WINDOW, None);
        insert_row(dom, FIRST + WINDOW + 1, None);
    }));
    assert_eq!((added.len(), removed.len()), (34, 34));
}

#[test]
fn the_strip_moving_alone_only_moves_bounds() {
    let mut host = host();
    let [added, changed, removed, moved] = parts(host.admitted(|dom| {
        let strip = by_id(dom, "strip");
        dom.set_attribute(strip, attr("style"), "top: -100px");
    }));
    assert!(added.is_empty() && changed.is_empty() && removed.is_empty());
    // Every row node and the strip itself.
    assert_eq!(moved.len(), WINDOW * 17 + 1);
}

#[test]
fn a_write_that_changes_nothing_reports_nothing() {
    let mut host = host();
    let update = host.admitted(|dom| {
        let row = by_id(dom, &format!("r{}", FIRST + 2));
        dom.set_attribute(row, attr("data-row"), "other");
        dom.set_attribute(row, attr("class"), "grid-row selected");
    });
    assert_eq!(parts(update), [vec![], vec![], vec![], vec![]]);
}

#[test]
fn a_kept_write_reports_the_node_and_the_names_it_feeds() {
    let mut host = host();
    let target = cell(&host.dom, FIRST + 2, 3);
    let row = by_id(&host.dom, &format!("r{}", FIRST + 2));
    let [_, changed, _, _] = parts(host.admitted(|dom| {
        dom.set_attribute(target, attr("aria-label"), "Renamed");
    }));
    // The row takes its name from its cells' contents.
    let mut expected = vec![node_id(&host.dom, target), node_id(&host.dom, row)];
    expected.sort();
    assert_eq!(changed, expected);
    let [_, changed, _, _] = parts(host.admitted(|dom| {
        dom.set_attribute(row, attr("aria-selected"), "true");
    }));
    assert_eq!(changed, [node_id(&host.dom, row)]);
}

#[test]
fn focus_moves_as_node_changes_and_a_removed_focus_is_removed() {
    let mut host = host();
    let first = cell(&host.dom, FIRST, 0);
    let second = cell(&host.dom, FIRST + 1, 0);
    host.focus = Some(first);
    let [_, changed, _, _] = parts(host.admitted(|_| {}));
    assert_eq!(changed, [node_id(&host.dom, first)]);
    host.focus = Some(second);
    let mut both = vec![node_id(&host.dom, first), node_id(&host.dom, second)];
    both.sort();
    assert_eq!(parts(host.admitted(|_| {}))[1], both);
    // The focused cell leaves with its row.
    let leaving = node_id(&host.dom, second);
    let [_, _, removed, _] = parts(host.admitted(|dom| {
        dom.remove(by_id(dom, &format!("r{}", FIRST + 1)));
    }));
    assert!(removed.contains(&leaving));
}

#[test]
fn an_inserted_row_may_reference_inside_itself() {
    let mut host = host();
    host.admitted(|dom| {
        let row = insert_row(dom, FIRST + WINDOW, None);
        let cells: Vec<_> = dom.dom_children(row).collect();
        dom.set_attribute(cells[0], attr("id"), "local-label");
        dom.set_attribute(cells[1], attr("aria-labelledby"), "local-label");
    });
}

/// A fixture element appended to the body before the session starts.
pub(super) fn host_with(build: impl FnOnce(&mut ScriptedDom)) -> Host {
    let mut dom = table_dom(FIRST);
    build(&mut dom);
    dom.drain_mutations(&mut Vec::new());
    Host::new(dom)
}

fn body(dom: &ScriptedDom) -> NodeId {
    dom.first_tag(dom.document(), "body").expect("body")
}

pub(super) fn append_html(dom: &mut ScriptedDom, html: &str) {
    let holder = dom.create_element(layout_dom_api::QualName::new(
        None,
        layout_dom_api::Namespace::from("http://www.w3.org/1999/xhtml"),
        "div".into(),
    ));
    let parent = body(dom);
    dom.append_child(parent, holder);
    dom.set_inner_html(holder, html);
}

#[test]
fn an_insertion_renames_a_generic_with_direct_text() {
    let mut host = host_with(|dom| append_html(dom, "<div id=note>Note <span>a</span></div>"));
    let note = node_id(&host.dom, by_id(&host.dom, "note"));
    let [added, changed, _, _] = parts(host.admitted(|dom| {
        let span = dom.create_element(layout_dom_api::QualName::new(
            None,
            layout_dom_api::Namespace::from("http://www.w3.org/1999/xhtml"),
            "span".into(),
        ));
        let note = by_id(dom, "note");
        dom.append_child(note, span);
        let text = dom.create_text(" b");
        dom.append_child(span, text);
    }));
    assert_eq!(added.len(), 1);
    assert_eq!(changed, [note]);
    let name = host.session.node(note).and_then(|n| n.name.clone());
    assert_eq!(name.as_deref(), Some("Note a b"));
}

#[test]
fn refuses_without_a_restyled_set() {
    let mut host = host();
    assert_eq!(
        host.frame_with(|dom| scroll_down(dom, FIRST), false, None),
        A11yUpdate::Rebuilt
    );
    assert_eq!(host.session.refusal(), Some(Refusal::NoRestyledSet));
}

#[test]
fn refuses_when_scroll_offsets_change() {
    let mut host = host();
    let strip = by_id(&host.dom, "strip");
    let offsets = HashMap::from([(strip, (0.0, 10.0))]);
    assert_eq!(
        host.frame_with(|_| {}, true, Some(&offsets)),
        A11yUpdate::Rebuilt
    );
    assert_eq!(host.session.refusal(), Some(Refusal::ScrollOffsetsChanged));
}

#[test]
fn refuses_other_mutations() {
    let mut host = host();
    let text = dom_text(&host.dom, cell(&host.dom, FIRST, 1));
    host.refused(Refusal::UnsupportedMutation, |dom| {
        dom.set_text(text, "edited")
    });
}

fn dom_text(dom: &ScriptedDom, node: NodeId) -> NodeId {
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        if dom.kind(node) == layout_dom_api::NodeKind::Text {
            return node;
        }
        pending.extend(dom.dom_children(node));
    }
    panic!("no text")
}

#[test]
fn refuses_a_reference_written_on_a_kept_node() {
    let mut host = host();
    let target = cell(&host.dom, FIRST, 1);
    host.refused(Refusal::ReferenceAttributeWritten, |dom| {
        dom.set_attribute(target, attr("aria-describedby"), "strip");
    });
}

#[test]
fn refuses_an_inserted_id_a_kept_reference_names() {
    let mut host = host_with(|dom| append_html(dom, "<button aria-labelledby=later>Go</button>"));
    host.refused(Refusal::ReferenceChanged, |dom| {
        let row = insert_row(dom, FIRST + WINDOW, None);
        dom.set_attribute(row, attr("id"), "later");
    });
}

#[test]
fn refuses_an_inserted_reference_that_leaves_its_subtree() {
    let mut host = host_with(|dom| append_html(dom, "<span id=outside>Out</span>"));
    host.refused(Refusal::ReferenceLeavesSubtree, |dom| {
        let row = insert_row(dom, FIRST + WINDOW, None);
        let first = dom.dom_children(row).next().unwrap();
        dom.set_attribute(first, attr("aria-describedby"), "outside");
    });
}

#[test]
fn refuses_removing_a_reference_to_a_kept_node() {
    let mut host = host_with(|dom| {
        append_html(dom, "<span id=outside>Out</span>");
        let target = cell(dom, FIRST, 1);
        dom.set_attribute(target, attr("aria-describedby"), "outside");
    });
    host.refused(Refusal::RemovedReferenceToKeptNode, |dom| {
        dom.remove(by_id(dom, &format!("r{FIRST}")));
    });
}

#[test]
fn refuses_a_change_inside_a_referenced_node() {
    let mut host = host_with(|dom| append_html(dom, "<button aria-describedby=strip>Go</button>"));
    host.refused(Refusal::ChangeInsideReferencedNode, |dom| {
        insert_row(dom, FIRST + WINDOW, None);
    });
}

#[test]
fn refuses_a_change_inside_a_label() {
    let mut host = host_with(|dom| {
        append_html(
            dom,
            "<label id=lab>Name <span id=inner>x</span></label><input>",
        )
    });
    host.refused(Refusal::ChangeInsideLabel, |dom| {
        let inner = by_id(dom, "inner");
        dom.set_attribute(inner, attr("aria-label"), "y");
    });
}

#[test]
fn refuses_a_change_inside_aria_hidden() {
    let mut host =
        host_with(|dom| append_html(dom, "<div aria-hidden=true><span id=inner>x</span></div>"));
    host.refused(Refusal::ChangeInsideAriaHidden, |dom| {
        let inner = by_id(dom, "inner");
        dom.set_attribute(inner, attr("class"), "y");
    });
}

#[test]
fn refuses_an_insertion_under_a_shadow_host() {
    let mut host = host_with(|dom| {
        append_html(
            dom,
            "<div id=host><template shadowrootmode=open><slot></slot></template></div>",
        );
        let root = dom.document();
        assert_eq!(dom.realize_declarative_shadow_roots(root).len(), 1);
    });
    host.refused(Refusal::ShadowTree, |dom| {
        let host = by_id(dom, "host");
        let span = dom.create_element(layout_dom_api::QualName::new(
            None,
            layout_dom_api::Namespace::from("http://www.w3.org/1999/xhtml"),
            "span".into(),
        ));
        dom.append_child(host, span);
    });
}

#[test]
fn refuses_a_kept_node_that_stops_rendering() {
    let mut host = host();
    let target = cell(&host.dom, FIRST, 1);
    host.refused(Refusal::ProjectionChanged, |dom| {
        dom.set_attribute(target, attr("style"), "left: 95px; visibility: hidden");
    });
}

#[test]
fn refuses_a_kept_node_inserted_again() {
    let mut host = host();
    host.refused(Refusal::Reinserted, |dom| {
        let strip = by_id(dom, "strip");
        let row = by_id(dom, &format!("r{FIRST}"));
        dom.insert_before(strip, row, None);
    });
}

#[test]
fn a_rebuild_leaves_the_session_patchable() {
    let mut host = host();
    host.refused(Refusal::UnsupportedMutation, |dom| {
        let text = dom_text(dom, cell(dom, FIRST, 1));
        dom.set_text(text, "edited");
    });
    host.admitted(|dom| scroll_down(dom, FIRST));
}
