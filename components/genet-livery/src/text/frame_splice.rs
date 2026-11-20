// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Merging one subtree's text into a retained frame in place: the replaced
//! sources leave, the fresh ones arrive, and nothing else is copied. The
//! result paints and selects as `TextFrame::replace_subtree_from` does.

use super::*;

impl<Id> TextFrame<Id>
where
    Id: Copy + Eq + Hash,
{
    /// Drop every source in `replaced` and, with `fresh = Some((frame,
    /// root))`, add that frame's sources, which must all be in `replaced` and
    /// under `root`. They join the text order before the first kept source
    /// that follows `root` in document order.
    pub(crate) fn splice_subtree_from<D>(
        &mut self,
        fresh: Option<(&Self, Id)>,
        replaced: &HashSet<Id>,
        dom: &D,
    ) where
        D: LayoutDom<NodeId = Id>,
    {
        // Paint-time state, which a merged frame also starts without.
        self.painted_decorations.clear();
        self.deferred_decorations.clear();
        self.held_groups.clear();

        let mut emptied = HashSet::new();
        for source in replaced {
            if let Some(group) = self.source_groups.remove(source) {
                emptied.insert(group);
            }
            self.prepared_sources.remove(source);
            self.prepared_controls.remove(source);
            self.line_bounds.remove(source);
            self.inline_fragments.remove(source);
            self.inline_line_keys.remove(source);
            self.cluster_sources.remove(source);
            self.text_values.remove(source);
            self.password_offsets.remove(source);
            self.text_groups.remove(source);
        }
        // A group shared with a kept source keeps all its commands, as a
        // merged frame's copy of that group does.
        if !emptied.is_empty() {
            for group in self.source_groups.values() {
                emptied.remove(group);
            }
            for group in emptied {
                self.prepared_groups[group].clear();
            }
        }
        self.text_clusters
            .retain(|cluster| !replaced.contains(&cluster.source));
        self.text_order.retain(|source| !replaced.contains(source));

        let group_base = self.next_text_group;
        if let Some((fresh, root)) = fresh {
            self.append_fresh(fresh, replaced, group_base);
            let at = self.order_position_after(dom, root);
            let mut entering = Vec::new();
            let mut pending = vec![root];
            while let Some(node) = pending.pop() {
                if self.text_values.contains_key(&node) && replaced.contains(&node) {
                    entering.push(node);
                }
                let children = dom.dom_children(node).collect::<Vec<_>>();
                pending.extend(children.into_iter().rev());
            }
            self.text_order.splice(at..at, entering);
            self.next_text_group = group_base.saturating_add(fresh.next_text_group);
        }
        self.rebuild_used_fonts();
    }

    fn append_fresh(&mut self, fresh: &Self, replaced: &HashSet<Id>, group_base: usize) {
        let mut sources_by_group: Vec<Vec<Id>> = vec![Vec::new(); fresh.prepared_groups.len()];
        for (source, group) in &fresh.source_groups {
            if replaced.contains(source) {
                sources_by_group[*group].push(*source);
            }
        }
        for (commands, sources) in fresh.prepared_groups.iter().zip(sources_by_group) {
            if sources.is_empty() {
                continue;
            }
            let group = self.prepared_groups.len();
            self.prepared_groups.push(commands.clone());
            for source in sources {
                self.source_groups.insert(source, group);
                self.prepared_sources.insert(source);
                if fresh.prepared_controls.contains(&source) {
                    self.prepared_controls.insert(source);
                }
            }
        }
        let fresh_only = |source: &Id| replaced.contains(source);
        self.line_bounds.extend(
            fresh
                .line_bounds
                .iter()
                .filter(|(source, _)| fresh_only(source))
                .map(|(source, bounds)| (*source, *bounds)),
        );
        self.inline_fragments.extend(
            fresh
                .inline_fragments
                .iter()
                .filter(|(source, _)| fresh_only(source))
                .map(|(source, fragments)| (*source, fragments.clone())),
        );
        self.inline_line_keys.extend(
            fresh
                .inline_line_keys
                .iter()
                .filter(|(source, _)| fresh_only(source))
                .map(|(source, lines)| (*source, lines.clone())),
        );
        for cluster in &fresh.text_clusters {
            if fresh_only(&cluster.source) {
                self.cluster_sources.insert(cluster.source);
                self.text_clusters.push(cluster.clone());
            }
        }
        for (source, value) in &fresh.text_values {
            if !fresh_only(source) {
                continue;
            }
            self.text_values.insert(*source, value.clone());
            if let Some(offsets) = fresh.password_offsets.get(source) {
                self.password_offsets.insert(*source, offsets.clone());
            }
            let group = fresh.text_groups.get(source).copied().unwrap_or_default();
            self.text_groups
                .insert(*source, group.saturating_add(group_base));
        }
    }

    /// The text-order index of the first kept source after `root`'s subtree
    /// in document order, or the end.
    fn order_position_after<D>(&self, dom: &D, root: Id) -> usize
    where
        D: LayoutDom<NodeId = Id>,
    {
        let mut node = root;
        while let Some(parent) = dom.parent(node) {
            let mut following = dom.dom_children(parent).skip_while(|child| *child != node);
            following.next();
            for sibling in following {
                let mut pending = vec![sibling];
                while let Some(candidate) = pending.pop() {
                    if self.text_values.contains_key(&candidate)
                        && let Some(at) = self
                            .text_order
                            .iter()
                            .position(|source| *source == candidate)
                    {
                        return at;
                    }
                    let children = dom.dom_children(candidate).collect::<Vec<_>>();
                    pending.extend(children.into_iter().rev());
                }
            }
            node = parent;
        }
        self.text_order.len()
    }
}
