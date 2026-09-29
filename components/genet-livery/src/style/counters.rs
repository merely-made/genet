// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! CSS Lists 3 counter sets and values, resolved after the ordinary cascade.
//! The pass touches generated strings only; source nodes/offsets stay intact.

use std::hash::Hash;

use buckram::PseudoElement;
use layout_dom_api::{LayoutDom, NodeKind};
use livery::{ComputedValues, values::Display};

use super::{StylePlane, generated_content_attribute};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Key<Id> {
    Element(Id),
    Pseudo(Id, PseudoElement),
}

#[derive(Clone)]
struct Counter<Id> {
    name: String,
    creator: Key<Id>,
    parent: Option<Key<Id>>,
    value: i32,
}

fn instantiate<Id: Copy + Eq>(
    set: &mut Vec<Counter<Id>>,
    key: Key<Id>,
    parent: Option<Key<Id>>,
    name: &str,
    value: i32,
) {
    if let Some(index) = set.iter().rposition(|counter| counter.name == name)
        && (set[index].creator == key || set[index].parent == parent)
    {
        set.remove(index);
    }
    set.push(Counter {
        name: name.to_owned(),
        creator: key,
        parent,
        value,
    });
}

fn innermost<Id: Copy + Eq>(
    set: &mut Vec<Counter<Id>>,
    key: Key<Id>,
    parent: Option<Key<Id>>,
    name: &str,
) -> usize {
    if let Some(index) = set.iter().rposition(|counter| counter.name == name) {
        return index;
    }
    instantiate(set, key, parent, name, 0);
    set.len() - 1
}

fn apply<Id: Copy + Eq>(
    style: &ComputedValues,
    set: &mut Vec<Counter<Id>>,
    key: Key<Id>,
    parent: Option<Key<Id>>,
) {
    for operation in &style.counter_reset.0 {
        instantiate(set, key, parent, &operation.name, operation.value);
    }
    for operation in &style.counter_increment.0 {
        let index = innermost(set, key, parent, &operation.name);
        set[index].value = set[index].value.saturating_add(operation.value);
    }
    for operation in &style.counter_set.0 {
        let index = innermost(set, key, parent, &operation.name);
        set[index].value = operation.value;
    }
}

pub(crate) fn resolve<D: LayoutDom>(dom: &D, plane: &mut StylePlane<D::NodeId>) {
    // Counter properties have no observable output until content uses them.
    // Keep documents without such output off the tree-walk path.
    if !plane
        .generated
        .values()
        .any(|(style, _)| style.content.has_counters())
    {
        return;
    }
    let mut resolver = Resolver {
        dom,
        plane,
        previous: Vec::new(),
    };
    let mut sibling = None;
    for child in dom.flat_children(dom.document()) {
        if let Some(set) = resolver.element(child, None, &[], sibling.as_deref()) {
            sibling = Some(set);
        }
    }
}

struct Resolver<'a, D: LayoutDom> {
    dom: &'a D,
    plane: &'a mut StylePlane<D::NodeId>,
    previous: Vec<Counter<D::NodeId>>,
}

impl<D: LayoutDom> Resolver<'_, D>
where
    D::NodeId: Copy + Eq + Hash,
{
    /// CSS Lists editor's draft, 2026-09-29, section 4.4.1: identities come
    /// from the preceding sibling, or the parent when no sibling exists.
    /// Retain only matching name+creator instances in the previous tree-order
    /// value source. The 2020 TR's parent-name-first rule is superseded here.
    fn inherit(
        &self,
        parent: &[Counter<D::NodeId>],
        sibling: Option<&[Counter<D::NodeId>]>,
    ) -> Vec<Counter<D::NodeId>> {
        let source = sibling.unwrap_or(parent);
        self.previous
            .iter()
            .filter(|value| {
                source
                    .iter()
                    .any(|counter| counter.name == value.name && counter.creator == value.creator)
            })
            .cloned()
            .collect()
    }

    fn element(
        &mut self,
        node: D::NodeId,
        parent_key: Option<Key<D::NodeId>>,
        parent: &[Counter<D::NodeId>],
        sibling: Option<&[Counter<D::NodeId>]>,
    ) -> Option<Vec<Counter<D::NodeId>>> {
        crate::with_recursion_stack(|| self.element_inner(node, parent_key, parent, sibling))
    }

    fn element_inner(
        &mut self,
        node: D::NodeId,
        parent_key: Option<Key<D::NodeId>>,
        parent: &[Counter<D::NodeId>],
        sibling: Option<&[Counter<D::NodeId>]>,
    ) -> Option<Vec<Counter<D::NodeId>>> {
        if self.dom.kind(node) != NodeKind::Element {
            return None;
        }
        let style = self.plane.values.get(&node)?;
        if style.display == Display::None {
            return None;
        }
        let key = Key::Element(node);
        let mut set = self.inherit(parent, sibling);
        if style.display != Display::Contents {
            apply(style, &mut set, key, parent_key);
        }
        self.previous = set.clone();
        // Replaced-owner descendants have no admitted generated box here.
        if !crate::box_tree::is_replaced_element(self.dom, node) {
            let mut child_sibling = None;
            if let Some(before) =
                self.pseudo(node, PseudoElement::Before, &set, child_sibling.as_deref())
            {
                child_sibling = Some(before);
            }
            for child in self.dom.flat_children(node) {
                if let Some(child_set) =
                    self.element(child, Some(key), &set, child_sibling.as_deref())
                {
                    child_sibling = Some(child_set);
                }
            }
            self.pseudo(node, PseudoElement::After, &set, child_sibling.as_deref());
        }
        Some(set)
    }

    fn pseudo(
        &mut self,
        owner: D::NodeId,
        pseudo: PseudoElement,
        parent: &[Counter<D::NodeId>],
        sibling: Option<&[Counter<D::NodeId>]>,
    ) -> Option<Vec<Counter<D::NodeId>>> {
        let key = Key::Pseudo(owner, pseudo);
        let parent_key = Some(Key::Element(owner));
        let mut set = self.inherit(parent, sibling);
        let (style, text) = self.plane.generated.get_mut(&(owner, pseudo))?;
        apply(style, &mut set, key, parent_key);
        *text = style
            .content
            .resolve_with_counters(
                |name| generated_content_attribute(self.dom, owner, name),
                |name, separator| {
                    let index = innermost(&mut set, key, parent_key, name);
                    if let Some(separator) = separator {
                        set.iter()
                            .filter(|counter| counter.name == name)
                            .map(|counter| counter.value.to_string())
                            .collect::<Vec<_>>()
                            .join(separator)
                    } else {
                        set[index].value.to_string()
                    }
                },
            )
            .expect("admitted content list");
        self.previous = set.clone();
        Some(set)
    }
}
