// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

const HTML: u8 = 1;
const BODY: u8 = 2;
const STRIP: u8 = 3;

fn element(
    node: u8,
    display: DisplayRole,
    positioning: PositioningScheme,
    children: Vec<BoxTreeInput<u8>>,
) -> BoxTreeInput<u8> {
    BoxTreeInput::new(
        BoxOrigin::Element(node),
        display,
        FlowAxes::HORIZONTAL_LTR,
        positioning,
        false,
        children,
    )
}

/// An absolute row holding one absolute cell with text.
fn row(index: u8) -> BoxTreeInput<u8> {
    let text = BoxTreeInput::text(
        BoxOrigin::Text(100 + index),
        FlowAxes::HORIZONTAL_LTR,
        false,
        false,
    );
    let cell = element(
        50 + index,
        DisplayRole::BLOCK_FLOW,
        PositioningScheme::Absolute,
        vec![text],
    );
    element(
        10 + index,
        DisplayRole::BLOCK_FLOW,
        PositioningScheme::Absolute,
        vec![cell],
    )
}

/// html > body (relative) > strip (absolute) > rows, plus `extra` under
/// body after the strip and `strip_extra` under the strip after the rows.
fn document(
    rows: &[u8],
    body: DisplayRole,
    extra: Vec<BoxTreeInput<u8>>,
    strip_extra: Vec<BoxTreeInput<u8>>,
) -> CssBoxTree<u8> {
    let mut strip_children = rows.iter().copied().map(row).collect::<Vec<_>>();
    strip_children.extend(strip_extra);
    let strip = element(
        STRIP,
        DisplayRole::BLOCK_FLOW,
        PositioningScheme::Absolute,
        strip_children,
    );
    let mut body_children = vec![strip];
    body_children.extend(extra);
    generate_box_tree([element(
        HTML,
        DisplayRole::BLOCK_FLOW,
        PositioningScheme::Static,
        vec![element(
            BODY,
            body,
            PositioningScheme::Relative,
            body_children,
        )],
    )])
}

fn plain(rows: &[u8]) -> CssBoxTree<u8> {
    document(rows, DisplayRole::BLOCK_FLOW, Vec::new(), Vec::new())
}

/// Equal boxes under equal ids, stored in the same order, with equal tree
/// positions and provenance maps.
fn assert_same_tree(spliced: &CssBoxTree<u8>, regenerated: &CssBoxTree<u8>) {
    assert_eq!(spliced.roots, regenerated.roots);
    assert_eq!(spliced.ids, regenerated.ids, "storage order");
    assert_eq!(spliced.boxes, regenerated.boxes);
    assert_eq!(spliced.slots, regenerated.slots);
    assert_eq!(spliced.principal_boxes, regenerated.principal_boxes);
    assert_eq!(spliced.boxes_by_node, regenerated.boxes_by_node);
}

/// Regenerate `next` and reconcile it against `previous`, as a relayout does.
fn regenerated(previous: &CssBoxTree<u8>, next: CssBoxTree<u8>) -> CssBoxTree<u8> {
    let mut next = next;
    next.reconcile_identifiers(previous);
    next
}

fn strip_of(tree: &CssBoxTree<u8>) -> BoxId {
    tree.principal_box(STRIP).unwrap()
}

#[test]
fn rows_entering_at_the_bottom_and_leaving_at_the_top_match_regeneration() {
    let previous = plain(&[1, 2, 3]);
    let mut spliced = previous.clone();
    let strip = strip_of(&spliced);
    spliced.insert_generated_child(strip, 3, row(4)).unwrap();
    spliced.insert_generated_child(strip, 4, row(5)).unwrap();
    assert!(spliced.remove_subtree(spliced.principal_box(11).unwrap()));
    assert_same_tree(&spliced, &regenerated(&previous, plain(&[2, 3, 4, 5])));
}

#[test]
fn rows_entering_at_the_top_and_leaving_at_the_bottom_match_regeneration() {
    let previous = plain(&[3, 4, 5]);
    let mut spliced = previous.clone();
    let strip = strip_of(&spliced);
    spliced.insert_generated_child(strip, 0, row(1)).unwrap();
    spliced.insert_generated_child(strip, 1, row(2)).unwrap();
    assert!(spliced.remove_subtree(spliced.principal_box(15).unwrap()));
    assert_same_tree(&spliced, &regenerated(&previous, plain(&[1, 2, 3, 4])));
}

#[test]
fn a_row_entering_between_kept_rows_is_stored_in_tree_order() {
    let previous = plain(&[1, 3]);
    let mut spliced = previous.clone();
    spliced
        .insert_generated_child(strip_of(&spliced), 1, row(2))
        .unwrap();
    assert_same_tree(&spliced, &regenerated(&previous, plain(&[1, 2, 3])));
}

#[test]
fn entering_into_an_empty_parent_and_leaving_it_empty_match_regeneration() {
    let previous = plain(&[]);
    let mut spliced = previous.clone();
    spliced
        .insert_generated_child(strip_of(&spliced), 0, row(1))
        .unwrap();
    let entered = regenerated(&previous, plain(&[1]));
    assert_same_tree(&spliced, &entered);
    assert!(spliced.remove_subtree(spliced.principal_box(11).unwrap()));
    assert_same_tree(&spliced, &regenerated(&entered, plain(&[])));
}

#[test]
fn the_entering_row_takes_its_containing_blocks_from_its_ancestors() {
    let mut tree = plain(&[]);
    let strip = strip_of(&tree);
    let root = tree.insert_generated_child(strip, 0, row(1)).unwrap();
    assert_eq!(tree[root].containing_block, ContainingBlock::Box(strip));
    let cell = tree[root].children()[0];
    assert_eq!(tree[cell].containing_block, ContainingBlock::Box(root));
}

#[test]
fn removing_an_unknown_box_changes_nothing() {
    let mut tree = plain(&[1]);
    let before = tree.clone();
    assert!(!tree.remove_subtree(BoxId(999)));
    assert_same_tree(&tree, &before);
}

fn assert_refused(
    mut tree: CssBoxTree<u8>,
    parent: BoxId,
    index: usize,
    input: BoxTreeInput<u8>,
    reason: ChildSpliceRefusal,
) {
    let before = tree.clone();
    assert_eq!(
        tree.insert_generated_child(parent, index, input),
        Err(reason)
    );
    assert_same_tree(&tree, &before);
}

#[test]
fn an_inline_parent_is_refused() {
    let tree = plain(&[1]);
    let cell = tree[tree.principal_box(11).unwrap()].children()[0];
    let text = tree[cell].children()[0];
    assert_refused(tree, text, 0, row(2), ChildSpliceRefusal::Parent);
}

#[test]
fn an_index_past_the_children_is_refused() {
    let tree = plain(&[1]);
    let strip = strip_of(&tree);
    assert_refused(tree, strip, 2, row(2), ChildSpliceRefusal::Index);
}

#[test]
fn an_in_flow_sibling_is_refused() {
    let static_child = element(
        60,
        DisplayRole::BLOCK_FLOW,
        PositioningScheme::Static,
        Vec::new(),
    );
    let tree = document(
        &[1],
        DisplayRole::BLOCK_FLOW,
        Vec::new(),
        vec![static_child],
    );
    let strip = strip_of(&tree);
    assert_refused(tree, strip, 1, row(2), ChildSpliceRefusal::InFlowSibling);
}

#[test]
fn an_in_flow_child_is_refused() {
    let tree = plain(&[1]);
    let strip = strip_of(&tree);
    let in_flow = element(
        12,
        DisplayRole::BLOCK_FLOW,
        PositioningScheme::Static,
        Vec::new(),
    );
    assert_refused(
        tree,
        strip,
        1,
        in_flow,
        ChildSpliceRefusal::NotOneOutOfFlowBox,
    );
}

#[test]
fn a_child_generating_no_box_is_refused() {
    let tree = plain(&[1]);
    let strip = strip_of(&tree);
    let mut hidden = row(2);
    hidden.display.generation = BoxGeneration::None;
    assert_refused(
        tree,
        strip,
        1,
        hidden,
        ChildSpliceRefusal::NotOneOutOfFlowBox,
    );
}

#[test]
fn an_inline_ancestor_is_refused() {
    let tree = document(&[1], DisplayRole::INLINE_FLOW, Vec::new(), Vec::new());
    let strip = strip_of(&tree);
    assert_refused(tree, strip, 1, row(2), ChildSpliceRefusal::AncestorContext);
}

#[test]
fn an_ancestor_among_table_parts_is_refused() {
    let table_row = element(
        70,
        DisplayRole {
            generation: BoxGeneration::Normal,
            outside: None,
            inside: None,
            list_item: false,
            internal_table: Some(InternalTableRole::Row),
        },
        PositioningScheme::Static,
        Vec::new(),
    );
    let tree = document(&[1], DisplayRole::BLOCK_FLOW, vec![table_row], Vec::new());
    let strip = strip_of(&tree);
    assert_refused(tree, strip, 1, row(2), ChildSpliceRefusal::AncestorContext);
}

#[test]
fn a_node_that_already_has_boxes_is_refused() {
    let tree = plain(&[1]);
    let strip = strip_of(&tree);
    assert_refused(tree, strip, 1, row(1), ChildSpliceRefusal::NodeHasBoxes);
}
