// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Inserting and removing one unfragmented structural subtree. These are the
//! siblings of [`FragmentTree::replace_subtree`]: the same edge checks, with
//! the indices repaired in place rather than rebuilt.

use std::collections::{HashMap, HashSet};

use super::{FragmentId, FragmentTree, StaticPositionSource};
use crate::{BoxId, FragmentationContextId};

impl FragmentTree {
    /// Copy the subtree at `source_root` of `source` into this tree as a new
    /// child of `parent`, placed after `after`'s subtree, or first when
    /// `after` is `None`. Its root takes `containing_fragment`; descendants get
    /// fresh identities and keep their static positions. Indices are repaired
    /// in place; aggregate overflow is left for [`Self::flush_overflow`].
    ///
    /// Declined, leaving the tree unchanged: an anchor that is not live or not
    /// a child of `parent`; an incoming box that already has fragments or a
    /// static position here; a fragmented box; a descendant whose parent,
    /// containing fragment or static-position source lies outside the
    /// subtree; a fragment outside the initial fragmentation context.
    pub fn insert_subtree(
        &mut self,
        parent: Option<FragmentId>,
        after: Option<FragmentId>,
        containing_fragment: Option<FragmentId>,
        source: &Self,
        source_root: FragmentId,
    ) -> Option<FragmentId> {
        if parent.is_some_and(|parent| self.get(parent).is_none())
            || containing_fragment.is_some_and(|containing| self.get(containing).is_none())
            || after.is_some_and(|after| self.get(after).is_none_or(|f| f.parent != parent))
        {
            return None;
        }
        let (incoming, incoming_boxes) = source.self_contained(source_root)?;
        if incoming_boxes.iter().any(|box_id| {
            self.by_box.contains_key(box_id) || self.static_positions.contains_key(box_id)
        }) {
            return None;
        }

        let at = match (after, parent) {
            (Some(after), _) => {
                self.descendants_of(after)
                    .iter()
                    .map(|id| self.slots[id])
                    .max()
                    .expect("a live anchor is its own descendant")
                    + 1
            },
            (None, Some(parent)) => self.slots[&parent] + 1,
            (None, None) => 0,
        };
        let identifiers = incoming
            .iter()
            .map(|id| (*id, self.allocate_fragment_id()))
            .collect::<HashMap<_, _>>();
        let imported = incoming.iter().map(|id| {
            let mut fragment = source.get(*id).expect("a live source fragment").clone();
            fragment.id = identifiers[id];
            if *id == source_root {
                fragment.parent = parent;
                fragment.containing_fragment = containing_fragment;
            } else {
                fragment.parent = fragment.parent.map(|p| identifiers[&p]);
                fragment.containing_fragment =
                    fragment.containing_fragment.map(|c| identifiers[&c]);
            }
            fragment
        });
        let tail = self.fragments.split_off(at);
        self.fragments.extend(imported);
        self.fragments.extend(tail);
        let tail = self.ids.split_off(at);
        self.ids.extend(incoming.iter().map(|id| identifiers[id]));
        self.ids.extend(tail);

        // Repair the indices in place: only slots at and after `at` moved.
        self.reslot_from(at);
        for fragment in &self.fragments[at..at + incoming.len()] {
            self.by_box
                .entry(fragment.box_id)
                .or_default()
                .push(fragment.id);
            if fragment.id != identifiers[&source_root] {
                let parent = fragment.parent.expect("a checked descendant has a parent");
                self.children.entry(parent).or_default().push(fragment.id);
            }
        }
        self.attach_child(parent, identifiers[&source_root]);

        for position in source.static_positions.values() {
            if incoming_boxes.contains(&position.box_id) {
                let mut position = *position;
                if let StaticPositionSource::Fragment(from) = position.source {
                    position.source = StaticPositionSource::Fragment(identifiers[&from]);
                }
                self.static_positions.insert(position.box_id, position);
            }
        }
        // Aggregate overflow waits for `flush_overflow`, as after a translation.
        self.overflow_dirty = true;
        #[cfg(any(debug_assertions, test))]
        self.assert_invariants();
        Some(identifiers[&source_root])
    }

    /// Remove the fragment `root` and every structural descendant, with their
    /// static positions, repairing the indices in place and leaving aggregate
    /// overflow for [`Self::flush_overflow`]. Declined, leaving the tree
    /// unchanged, when `root` is
    /// not live, a removed box keeps a fragment outside the subtree, or a
    /// fragment or static position outside it still names a fragment inside.
    pub fn remove_subtree(&mut self, root: FragmentId) -> bool {
        let Some((retired, retired_boxes)) = self.removable(root) else {
            return false;
        };
        let first = retired
            .iter()
            .map(|id| self.slots[id])
            .min()
            .expect("the root is retired");
        let parent = self.get(root).and_then(|fragment| fragment.parent);
        self.detach_child(parent, root);
        self.fragments
            .retain(|fragment| !retired.contains(&fragment.id));
        self.ids.retain(|id| !retired.contains(id));
        for id in &retired {
            self.slots.remove(id);
            self.children.remove(id);
        }
        for box_id in &retired_boxes {
            self.by_box.remove(box_id);
        }
        self.reslot_from(first);
        self.static_positions
            .retain(|box_id, _| !retired_boxes.contains(box_id));
        self.overflow_dirty = true;
        #[cfg(any(debug_assertions, test))]
        self.assert_invariants();
        true
    }
}

impl FragmentTree {
    /// Whether [`Self::remove_subtree`] would accept `root`, without removing
    /// it, so a caller can check every step of a batch before changing any.
    pub fn can_remove_subtree(&self, root: FragmentId) -> bool {
        self.removable(root).is_some()
    }

    /// Whether the subtree at `root` depends on nothing outside it, the
    /// source-side half of what [`Self::insert_subtree`] checks.
    pub fn is_self_contained_subtree(&self, root: FragmentId) -> bool {
        self.self_contained(root).is_some()
    }

    /// The fragments and boxes `remove_subtree(root)` would retire.
    fn removable(&self, root: FragmentId) -> Option<(HashSet<FragmentId>, HashSet<BoxId>)> {
        self.get(root)?;
        let retired = self
            .descendants_of(root)
            .into_iter()
            .collect::<HashSet<_>>();
        let retired_boxes = retired
            .iter()
            .map(|id| self.get(*id).expect("a live retired fragment").box_id)
            .collect::<HashSet<BoxId>>();
        let crosses_edge = retired_boxes
            .iter()
            .any(|box_id| self.by_box[box_id].iter().any(|id| !retired.contains(id)))
            || self.fragments.iter().any(|fragment| {
                !retired.contains(&fragment.id)
                    && fragment
                        .containing_fragment
                        .is_some_and(|c| retired.contains(&c))
            })
            || self.static_positions.values().any(|position| {
                !retired_boxes.contains(&position.box_id)
                    && matches!(position.source, StaticPositionSource::Fragment(from) if retired.contains(&from))
            });
        (!crosses_edge).then_some((retired, retired_boxes))
    }

    /// The subtree at `root` in slot order, and its boxes, when it is
    /// unfragmented, in the initial context, and self-contained.
    fn self_contained(&self, root: FragmentId) -> Option<(Vec<FragmentId>, HashSet<BoxId>)> {
        self.get(root)?;
        let incoming = self.subtree_ids(root);
        let incoming_set = incoming.iter().copied().collect::<HashSet<_>>();
        let incoming_boxes = incoming
            .iter()
            .map(|id| self.get(*id).expect("a live fragment").box_id)
            .collect::<HashSet<_>>();
        let contained = incoming_boxes.iter().all(|box_id| {
            self.by_box[box_id]
                .iter()
                .all(|id| incoming_set.contains(id))
        }) && incoming.iter().all(|id| {
            let fragment = self.get(*id).expect("a live fragment");
            fragment.fragmentation_context == FragmentationContextId::INITIAL
                && fragment.fragmentainer.is_none()
                && (*id == root
                    || (fragment.parent.is_some_and(|p| incoming_set.contains(&p))
                        && fragment
                            .containing_fragment
                            .is_none_or(|c| incoming_set.contains(&c))))
        }) && static_sources_stay_inside(self, &incoming_boxes, &incoming_set);
        contained.then_some((incoming, incoming_boxes))
    }

    /// Point every id from `start` on at its current slot.
    fn reslot_from(&mut self, start: usize) {
        for (slot, id) in self.ids.iter().enumerate().skip(start) {
            self.slots.insert(*id, slot);
        }
    }
}

/// Whether every static position touching the incoming subtree is wholly
/// inside it: an incoming box's source, and any box sourced from inside.
fn static_sources_stay_inside(
    source: &FragmentTree,
    incoming_boxes: &HashSet<BoxId>,
    incoming: &HashSet<FragmentId>,
) -> bool {
    source.static_positions.values().all(|position| {
        let StaticPositionSource::Fragment(from) = position.source else {
            return true;
        };
        incoming_boxes.contains(&position.box_id) == incoming.contains(&from)
    })
}

#[cfg(test)]
mod tests;
