// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Right-to-left matching over an [`Element`].
//!
//! The shadow-tree rules (featureless host, slot assignment, part forwarding
//! through `exportparts`) follow CSS Scoping 1 and CSS Shadow Parts 1, with
//! servo's `selectors` crate read as a reference for their edge cases.

use crate::ast::{Combinator, Nth, Selector, SelectorList, Simple};
use crate::attr::{AttrOperation, CaseSensitivity, NamespaceConstraint, ParsedCase};
use crate::{Element, PseudoClass};

impl<P: PseudoClass> SelectorList<P> {
    /// The strongest specificity among the selectors that match, if any do.
    pub fn matching_specificity<E>(&self, element: &E, shadow_host: Option<&E>) -> Option<u32>
    where
        E: Element<PseudoClass = P>,
    {
        self.0
            .iter()
            .filter(|selector| selector.matches(element, shadow_host))
            .map(Selector::specificity)
            .max()
    }

    pub fn matches<E>(&self, element: &E, shadow_host: Option<&E>) -> bool
    where
        E: Element<PseudoClass = P>,
    {
        self.0.iter().any(|selector| selector.matches(element, shadow_host))
    }
}

impl<P: PseudoClass> Selector<P> {
    /// `shadow_host` is the host of the tree scope the selector's stylesheet
    /// lives in, or `None` for a document stylesheet. `:host` compares against
    /// it, and `::slotted()` and `::part()` resolve relative to it.
    pub fn matches<E>(&self, element: &E, shadow_host: Option<&E>) -> bool
    where
        E: Element<PseudoClass = P>,
    {
        matches_from(self, 0, element, false, shadow_host)
    }
}

fn list_matches<E: Element>(
    list: &[Selector<E::PseudoClass>],
    element: &E,
    featureless: bool,
    host: Option<&E>,
) -> bool {
    list.iter().any(|selector| matches_from(selector, 0, element, featureless, host))
}

/// Match `selector.compounds[index..]` with `element` as the candidate for
/// compound `index`. `featureless` marks a shadow host reached from inside its
/// own shadow tree, which only `:host` can see.
fn matches_from<E: Element>(
    selector: &Selector<E::PseudoClass>,
    index: usize,
    element: &E,
    featureless: bool,
    host: Option<&E>,
) -> bool {
    let compound = &selector.compounds[index];
    if !compound.iter().all(|simple| matches_simple(simple, element, featureless, host)) {
        return false;
    }
    let Some(combinator) = selector.combinators.get(index) else {
        return true;
    };
    // Nothing outside a shadow tree is visible from inside it.
    if featureless {
        return false;
    }
    let next = index + 1;
    match combinator {
        Combinator::Child => ancestor_step(element)
            .is_some_and(|(parent, featureless)| matches_from(selector, next, &parent, featureless, host)),
        Combinator::Descendant => {
            let mut current = ancestor_step(element);
            while let Some((ancestor, featureless)) = current {
                if matches_from(selector, next, &ancestor, featureless, host) {
                    return true;
                }
                current = if featureless { None } else { ancestor_step(&ancestor) };
            }
            false
        },
        Combinator::NextSibling => element
            .prev_sibling_element()
            .is_some_and(|sibling| matches_from(selector, next, &sibling, false, host)),
        Combinator::LaterSibling => {
            let mut current = element.prev_sibling_element();
            while let Some(sibling) = current {
                if matches_from(selector, next, &sibling, false, host) {
                    return true;
                }
                current = sibling.prev_sibling_element();
            }
            false
        },
        Combinator::SlotAssignment => slot_in_scope(element, host)
            .is_some_and(|slot| matches_from(selector, next, &slot, false, host)),
        Combinator::Part => host_for_part(element, host)
            .is_some_and(|part_host| matches_from(selector, next, &part_host, false, host)),
    }
}

/// The parent element, or the featureless host when the parent is a shadow
/// root.
fn ancestor_step<E: Element>(element: &E) -> Option<(E, bool)> {
    if let Some(parent) = element.parent_element() {
        return Some((parent, false));
    }
    if element.parent_node_is_shadow_root() {
        return element.containing_shadow_host().map(|host| (host, true));
    }
    None
}

fn same<E: Element>(a: Option<&E>, b: Option<&E>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => a.is_same(b),
        (None, None) => true,
        _ => false,
    }
}

/// The slot, in the stylesheet's own shadow tree, that `element` ends up
/// assigned to. A slot may itself be slotted into a deeper tree.
fn slot_in_scope<E: Element>(element: &E, host: Option<&E>) -> Option<E> {
    let host = host?;
    let mut slot = element.assigned_slot()?;
    while !slot.containing_shadow_host().is_some_and(|h| h.is_same(host)) {
        slot = slot.assigned_slot()?;
    }
    Some(slot)
}

/// The host, visible from the stylesheet's scope, whose shadow tree (or a
/// tree nested in it) holds `element`.
fn host_for_part<E: Element>(element: &E, scope: Option<&E>) -> Option<E> {
    let mut current = element.containing_shadow_host()?;
    if same(Some(&current), scope) {
        return Some(current);
    }
    loop {
        let outer = current.containing_shadow_host();
        if same(outer.as_ref(), scope) {
            return Some(current);
        }
        current = outer?;
    }
}

/// Whether `element` is exposed under every name in `names` to the
/// stylesheet's scope, translating through each intervening `exportparts`.
fn matches_part<E: Element>(element: &E, names: &[Box<str>], scope: Option<&E>) -> bool {
    let Some(mut current) = element.containing_shadow_host() else {
        return false;
    };
    let mut hosts = Vec::new();
    if !same(Some(&current), scope) {
        loop {
            let outer = current.containing_shadow_host();
            if same(outer.as_ref(), scope) {
                break;
            }
            let Some(outer) = outer else {
                return false;
            };
            hosts.push(current);
            current = outer;
        }
    }
    names.iter().all(|name| {
        let mut name = name.to_string();
        for host in hosts.iter().rev() {
            match host.imported_part(&name) {
                Some(inner) => name = inner,
                None => return false,
            }
        }
        element.is_part(&name)
    })
}

/// `:host` alone, the only compound that can name a featureless host.
fn host_only<P>(selector: &Selector<P>) -> bool {
    selector.combinators.is_empty()
        && selector.compounds[0].iter().all(|simple| matches!(simple, Simple::Host(_)))
}

fn matches_simple<E: Element>(
    simple: &Simple<E::PseudoClass>,
    element: &E,
    featureless: bool,
    host: Option<&E>,
) -> bool {
    if featureless {
        return match simple {
            Simple::Host(_) => matches_host(simple, element, host),
            Simple::Is(list) | Simple::Where(list) => list
                .iter()
                .any(|s| s.combinators.is_empty() && matches_from(s, 0, element, true, host)),
            Simple::Not(list) => {
                list.iter().all(host_only) && !list_matches(list, element, true, host)
            },
            _ => false,
        };
    }
    match simple {
        Simple::Universal => true,
        Simple::NoNamespace => element.has_namespace(""),
        Simple::LocalName { name, lower } => {
            element.has_local_name(if element.is_html_element_in_html_document() {
                lower
            } else {
                name
            })
        },
        Simple::Id(id) => element.has_id(id, CaseSensitivity::CaseSensitive),
        Simple::Class(class) => element.has_class(class, CaseSensitivity::CaseSensitive),
        Simple::Attribute(attribute) => {
            let html = element.is_html_element_in_html_document();
            let namespace = if attribute.any_namespace {
                NamespaceConstraint::Any
            } else {
                NamespaceConstraint::Specific("")
            };
            let operation = match &attribute.operation {
                None => AttrOperation::Exists,
                Some((operator, case, value)) => AttrOperation::WithValue {
                    operator: *operator,
                    case_sensitivity: match case {
                        ParsedCase::Insensitive => CaseSensitivity::AsciiCaseInsensitive,
                        ParsedCase::InsensitiveInHtml if html => {
                            CaseSensitivity::AsciiCaseInsensitive
                        },
                        _ => CaseSensitivity::CaseSensitive,
                    },
                    value,
                },
            };
            let name = if html { &attribute.name_lower } else { &attribute.name };
            element.attr_matches(&namespace, name, &operation)
        },
        Simple::Root => element.is_root(),
        Simple::Empty => element.is_empty(),
        // No scoping root is ever supplied, so `:scope` is `:root`.
        Simple::Scope => element.is_root(),
        Simple::Nth(nth) => matches_nth(nth, element, host),
        Simple::Only { of_type } => {
            sibling_index(element, *of_type, false, &[], host) == 1
                && sibling_index(element, *of_type, true, &[], host) == 1
        },
        Simple::Not(list) => !list_matches(list, element, false, host),
        Simple::Is(list) | Simple::Where(list) => list_matches(list, element, false, host),
        Simple::Host(_) => matches_host(simple, element, host),
        // Slots are never slotted content themselves.
        Simple::Slotted(inner) => !element.is_slot() && matches_from(inner, 0, element, false, host),
        Simple::Part(names) => matches_part(element, names, host),
        Simple::State(state) => element.matches_pseudo_class(state),
    }
}

fn matches_host<E: Element>(
    simple: &Simple<E::PseudoClass>,
    element: &E,
    host: Option<&E>,
) -> bool {
    let Simple::Host(inner) = simple else {
        return false;
    };
    host.is_some_and(|host| host.is_same(element))
        && inner
            .as_ref()
            .is_none_or(|inner| matches_from(inner, 0, element, false, host))
}

/// 1-based position among the siblings that count.
fn sibling_index<E: Element>(
    element: &E,
    of_type: bool,
    from_end: bool,
    of: &[Selector<E::PseudoClass>],
    host: Option<&E>,
) -> i32 {
    let step = |e: &E| if from_end { e.next_sibling_element() } else { e.prev_sibling_element() };
    let mut index = 1;
    let mut current = step(element);
    while let Some(sibling) = current {
        let counts = (!of_type || sibling.is_same_type(element))
            && (of.is_empty() || list_matches(of, &sibling, false, host));
        index += i32::from(counts);
        current = step(&sibling);
    }
    index
}

fn matches_nth<E: Element>(nth: &Nth<E::PseudoClass>, element: &E, host: Option<&E>) -> bool {
    if !nth.of.is_empty() && !list_matches(&nth.of, element, false, host) {
        return false;
    }
    let index = sibling_index(element, nth.kind.of_type(), nth.kind.counts_from_end(), &nth.of, host);
    // index = a*n + b for some n >= 0.
    let offset = i64::from(index) - i64::from(nth.b);
    match i64::from(nth.a) {
        0 => offset == 0,
        a => offset % a == 0 && offset / a >= 0,
    }
}
