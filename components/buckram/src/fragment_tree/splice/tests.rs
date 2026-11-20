// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::{
    BoxId, BoxOrigin, ContainingBlock, CssBox, CssBoxTree, DisplayRole, FlowAxes, Fragment,
    FragmentId, FragmentTree, LogicalRect, PhysicalRect, PositioningScheme, StaticPosition,
    StaticPositionSource,
};

/// `count` unrelated box identities; fragments do not consult the box tree.
fn box_ids(count: u8) -> Vec<BoxId> {
    let mut boxes = CssBoxTree::default();
    (0..count)
        .map(|node| {
            boxes.push(
                CssBox::new(
                    BoxOrigin::Element(node),
                    DisplayRole::BLOCK_FLOW,
                    FlowAxes::HORIZONTAL_LTR,
                    PositioningScheme::Absolute,
                    false,
                    None,
                    ContainingBlock::Initial,
                ),
                None,
                true,
            )
        })
        .collect()
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> PhysicalRect {
    PhysicalRect {
        x,
        y,
        width,
        height,
    }
}

fn push(
    tree: &mut FragmentTree,
    box_id: BoxId,
    area: PhysicalRect,
    parent: Option<FragmentId>,
) -> FragmentId {
    tree.push(
        Fragment::from_horizontal_physical(box_id, area),
        parent,
        parent,
    )
}

fn static_at(box_id: BoxId, source: StaticPositionSource) -> StaticPosition {
    StaticPosition {
        box_id,
        source,
        containing_block: ContainingBlock::Initial,
        logical_rect: LogicalRect::default(),
        containing_block_area: None,
    }
}

/// A strip holding two rows, each with one cell.
struct Strip {
    tree: FragmentTree,
    strip: FragmentId,
    rows: [FragmentId; 2],
    boxes: Vec<BoxId>,
}

fn strip_tree() -> Strip {
    let boxes = box_ids(12);
    let mut tree = FragmentTree::default();
    let strip = push(&mut tree, boxes[0], rect(0.0, 0.0, 100.0, 0.0), None);
    let first = push(
        &mut tree,
        boxes[1],
        rect(0.0, 0.0, 100.0, 10.0),
        Some(strip),
    );
    push(&mut tree, boxes[2], rect(0.0, 0.0, 50.0, 10.0), Some(first));
    let second = push(
        &mut tree,
        boxes[3],
        rect(0.0, 10.0, 100.0, 10.0),
        Some(strip),
    );
    push(
        &mut tree,
        boxes[4],
        rect(0.0, 10.0, 50.0, 10.0),
        Some(second),
    );
    tree.record_static_position(static_at(boxes[1], StaticPositionSource::Fragment(strip)));
    tree.recompute_overflow();
    Strip {
        tree,
        strip,
        rows: [first, second],
        boxes,
    }
}

/// A detached row (box `row`) with one cell (box `cell`) at `y`.
fn row(boxes: &[BoxId], row: usize, cell: usize, y: f32) -> (FragmentTree, FragmentId) {
    let mut source = FragmentTree::default();
    let root = push(&mut source, boxes[row], rect(0.0, y, 100.0, 10.0), None);
    push(
        &mut source,
        boxes[cell],
        rect(0.0, y, 50.0, 10.0),
        Some(root),
    );
    source.record_static_position(static_at(boxes[cell], StaticPositionSource::Fragment(root)));
    (source, root)
}

/// The in-place index repair must leave exactly what a whole rebuild would,
/// and deferred overflow must flush to a whole recompute's result.
fn assert_indices_match_rebuild(tree: &mut FragmentTree) {
    let mut rebuilt = tree.clone();
    rebuilt.rebuild_indices();
    assert_eq!(tree.slots, rebuilt.slots, "slots");
    assert_eq!(tree.by_box, rebuilt.by_box, "fragments by box");
    assert_eq!(tree.roots, rebuilt.roots, "roots");
    assert_eq!(tree.children, rebuilt.children, "children in slot order");
    assert!(tree.overflow_is_stale(), "overflow waits for the flush");
    tree.flush_overflow();
    rebuilt.recompute_overflow();
    let overflow = |tree: &FragmentTree| {
        tree.fragments
            .iter()
            .map(|fragment| (fragment.id(), fragment.overflow))
            .collect::<Vec<_>>()
    };
    assert_eq!(overflow(tree), overflow(&rebuilt));
}

fn order(tree: &FragmentTree) -> Vec<BoxId> {
    tree.fragments.iter().map(Fragment::box_id).collect()
}

#[test]
fn insert_places_the_subtree_after_its_anchor_and_extends_overflow() {
    let Strip {
        mut tree,
        strip,
        rows,
        boxes,
    } = strip_tree();
    let (source, root) = row(&boxes, 5, 6, 20.0);
    assert!(source.is_self_contained_subtree(root));
    let inserted = tree
        .insert_subtree(Some(strip), Some(rows[0]), Some(strip), &source, root)
        .expect("a self-contained row inserts");
    assert_indices_match_rebuild(&mut tree);

    assert_eq!(
        order(&tree),
        [0, 1, 2, 5, 6, 3, 4].map(|index| boxes[index]),
        "the row follows the anchor's whole subtree, before the next sibling",
    );
    let fragment = tree.get(inserted).unwrap();
    assert_eq!(fragment.parent(), Some(strip));
    assert_eq!(fragment.containing_fragment(), Some(strip));
    assert!(rows.iter().all(|row| *row != inserted));
    let cell = tree.fragment_ids_for_box(boxes[6])[0];
    assert_eq!(tree.get(cell).unwrap().parent(), Some(inserted));
    assert_eq!(
        tree.static_position_for_box(boxes[6]).unwrap().source,
        StaticPositionSource::Fragment(inserted),
        "an imported static position names the imported fragment",
    );
    assert_eq!(tree.get(strip).unwrap().overflow.block_size, 30.0);
}

#[test]
fn insert_without_an_anchor_becomes_the_first_child() {
    let Strip {
        mut tree,
        strip,
        boxes,
        ..
    } = strip_tree();
    let (source, root) = row(&boxes, 5, 6, -10.0);
    tree.insert_subtree(Some(strip), None, Some(strip), &source, root)
        .expect("a first child inserts");
    assert_indices_match_rebuild(&mut tree);
    assert_eq!(
        order(&tree),
        [0, 5, 6, 1, 2, 3, 4].map(|index| boxes[index])
    );
    let overflow = tree.get(strip).unwrap().overflow;
    assert_eq!((overflow.block_start, overflow.block_size), (-10.0, 30.0));
}

#[test]
fn insert_declines_crossing_dependencies_and_leaves_the_tree_unchanged() {
    let Strip {
        mut tree,
        strip,
        rows,
        boxes,
    } = strip_tree();
    let before = order(&tree);

    // The anchor must be a child of the parent.
    let (source, root) = row(&boxes, 5, 6, 20.0);
    assert!(
        tree.insert_subtree(Some(rows[0]), Some(rows[1]), None, &source, root)
            .is_none()
    );
    // An incoming box may not already have fragments here.
    let (source, root) = row(&boxes, 1, 6, 20.0);
    assert!(
        tree.insert_subtree(Some(strip), None, None, &source, root)
            .is_none()
    );
    // A descendant's containing fragment must be inside the subtree.
    let mut source = FragmentTree::default();
    let outside = push(&mut source, boxes[7], rect(0.0, 0.0, 1.0, 1.0), None);
    let root = push(&mut source, boxes[8], rect(0.0, 0.0, 1.0, 1.0), None);
    source.push(
        Fragment::from_horizontal_physical(boxes[9], rect(0.0, 0.0, 1.0, 1.0)),
        Some(root),
        Some(outside),
    );
    assert!(!source.is_self_contained_subtree(root));
    assert!(
        tree.insert_subtree(Some(strip), None, None, &source, root)
            .is_none()
    );
    // A static position must not be sourced from outside the subtree.
    let (mut source, root) = row(&boxes, 10, 11, 20.0);
    let outside = push(&mut source, boxes[7], rect(0.0, 0.0, 1.0, 1.0), None);
    source.static_positions.insert(
        boxes[11],
        static_at(boxes[11], StaticPositionSource::Fragment(outside)),
    );
    assert!(!source.is_self_contained_subtree(root));
    assert!(
        tree.insert_subtree(Some(strip), None, None, &source, root)
            .is_none()
    );
    // Nor may a box outside it be sourced from inside.
    let (mut source, root) = row(&boxes, 10, 11, 20.0);
    push(&mut source, boxes[7], rect(0.0, 0.0, 1.0, 1.0), None);
    source.record_static_position(static_at(boxes[7], StaticPositionSource::Fragment(root)));
    assert!(
        tree.insert_subtree(Some(strip), None, None, &source, root)
            .is_none()
    );
    assert_eq!(order(&tree), before);
}

#[test]
fn remove_retires_the_subtree_its_static_positions_and_its_overflow() {
    let Strip {
        mut tree,
        strip,
        rows,
        boxes,
    } = strip_tree();
    tree.record_static_position(static_at(boxes[4], StaticPositionSource::Fragment(rows[1])));
    assert!(tree.can_remove_subtree(rows[1]));
    assert!(tree.remove_subtree(rows[1]));
    assert_indices_match_rebuild(&mut tree);
    assert_eq!(order(&tree), [0, 1, 2].map(|index| boxes[index]));
    assert!(tree.get(rows[1]).is_none());
    assert!(tree.static_position_for_box(boxes[4]).is_none());
    assert!(tree.static_position_for_box(boxes[1]).is_some());
    assert_eq!(tree.get(strip).unwrap().overflow.block_size, 10.0);
}

#[test]
fn remove_declines_a_subtree_named_from_outside() {
    let mut tree = strip_tree().tree;
    assert!(
        !tree.remove_subtree(super::super::FragmentId(999)),
        "a dead root is declined"
    );

    // A static position outside the subtree is sourced from inside it.
    let Strip {
        mut tree,
        rows,
        boxes,
        ..
    } = strip_tree();
    tree.record_static_position(static_at(boxes[3], StaticPositionSource::Fragment(rows[0])));
    let before = order(&tree);
    assert!(!tree.can_remove_subtree(rows[0]));
    assert!(!tree.remove_subtree(rows[0]));
    assert_eq!(order(&tree), before);

    // A fragment outside the subtree uses it as its containing block.
    let Strip {
        mut tree,
        strip,
        rows,
        boxes,
    } = strip_tree();
    tree.push(
        Fragment::from_horizontal_physical(boxes[5], rect(0.0, 0.0, 1.0, 1.0)),
        Some(strip),
        Some(rows[1]),
    );
    let before = order(&tree);
    assert!(!tree.can_remove_subtree(rows[1]));
    assert!(!tree.remove_subtree(rows[1]));
    assert_eq!(order(&tree), before);

    // A box that keeps a fragment outside the subtree.
    let Strip {
        mut tree,
        strip,
        rows,
        boxes,
    } = strip_tree();
    push(&mut tree, boxes[2], rect(0.0, 0.0, 1.0, 1.0), Some(strip));
    let before = order(&tree);
    assert!(!tree.can_remove_subtree(rows[0]));
    assert!(!tree.remove_subtree(rows[0]));
    assert_eq!(order(&tree), before);
}

#[test]
fn remove_then_insert_round_trips_order_and_geometry() {
    let Strip {
        mut tree,
        strip,
        rows,
        boxes,
    } = strip_tree();
    let geometry = |tree: &FragmentTree| {
        tree.fragments
            .iter()
            .map(|fragment| (fragment.box_id(), fragment.physical_rect()))
            .collect::<Vec<_>>()
    };
    let before = geometry(&tree);
    let (source, root) = row(&boxes, 3, 4, 10.0);
    assert!(tree.remove_subtree(rows[1]));
    tree.insert_subtree(Some(strip), Some(rows[0]), Some(strip), &source, root)
        .expect("the removed row inserts again");
    assert_indices_match_rebuild(&mut tree);
    assert_eq!(geometry(&tree), before);
}

/// Removing a subtree from the middle shifts every later slot; inserting
/// a root-level subtree goes through the root list.
#[test]
fn middle_and_root_level_splices_repair_indices_like_a_rebuild() {
    let Strip {
        mut tree,
        rows,
        boxes,
        ..
    } = strip_tree();
    assert!(tree.remove_subtree(rows[0]));
    assert_indices_match_rebuild(&mut tree);
    let (source, root) = row(&boxes, 5, 6, 40.0);
    let inserted = tree
        .insert_subtree(None, None, None, &source, root)
        .expect("a root-level subtree inserts");
    assert_indices_match_rebuild(&mut tree);
    assert_eq!(tree.roots()[0], inserted);
    let (source, root) = row(&boxes, 7, 8, 60.0);
    tree.insert_subtree(None, Some(inserted), None, &source, root)
        .expect("a second root-level subtree inserts");
    assert_indices_match_rebuild(&mut tree);
}
