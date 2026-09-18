// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The parsed form, and the facts a cascade index reads off it.

use crate::attr::Attribute;

/// What joins a compound to the one on its left.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Combinator {
    Descendant,
    Child,
    NextSibling,
    LaterSibling,
    /// From a `::slotted()` subject to the slot it is assigned to.
    SlotAssignment,
    /// From a `::part()` subject to the shadow host exposing it.
    Part,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NthKind {
    Child,
    LastChild,
    OfType,
    LastOfType,
}

impl NthKind {
    pub(crate) fn of_type(self) -> bool {
        matches!(self, Self::OfType | Self::LastOfType)
    }

    pub(crate) fn counts_from_end(self) -> bool {
        matches!(self, Self::LastChild | Self::LastOfType)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Nth<P> {
    pub(crate) kind: NthKind,
    pub(crate) a: i32,
    pub(crate) b: i32,
    /// `of S`. Empty when absent.
    pub(crate) of: Vec<Selector<P>>,
}

/// One simple selector.
#[derive(Clone, Debug, PartialEq)]
pub enum Simple<P> {
    /// `*`. Matches everything; kept so an explicit universal is a compound.
    Universal,
    /// `|E`: the element must be in no namespace.
    NoNamespace,
    LocalName {
        name: Box<str>,
        lower: Box<str>,
    },
    Id(Box<str>),
    Class(Box<str>),
    Attribute(Attribute),
    Root,
    Empty,
    Scope,
    Nth(Nth<P>),
    /// `:only-child` / `:only-of-type`.
    Only {
        of_type: bool,
    },
    Not(Vec<Selector<P>>),
    Is(Vec<Selector<P>>),
    Where(Vec<Selector<P>>),
    Host(Option<Box<Selector<P>>>),
    Slotted(Box<Selector<P>>),
    Part(Vec<Box<str>>),
    State(P),
}

/// How far out of its own tree scope a selector can reach.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reach {
    /// Matches only inside the selector's own tree scope.
    Inner,
    /// `:host` / `:host(...)` in the subject compound.
    Host,
    /// `::slotted(...)`.
    Slotted,
    /// `::part(...)`.
    Part,
}

/// The name a selector's subject compound requires of any element it matches.
/// Sound as an index key only because every matching element carries it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key<'a> {
    Id(&'a str),
    Class(&'a str),
    /// ASCII-lowercased.
    LocalName(&'a str),
    Universal,
}

/// One complex selector. Compounds are stored subject first.
#[derive(Clone, Debug, PartialEq)]
pub struct Selector<P> {
    pub(crate) compounds: Vec<Vec<Simple<P>>>,
    /// `combinators[i]` joins `compounds[i]` to `compounds[i + 1]`.
    pub(crate) combinators: Vec<Combinator>,
    specificity: u32,
}

const MAX_10BIT: u32 = (1 << 10) - 1;

#[derive(Clone, Copy, Default)]
struct Counts {
    id: u32,
    class: u32,
    element: u32,
}

impl Counts {
    fn pack(self) -> u32 {
        self.id.min(MAX_10BIT) << 20 | self.class.min(MAX_10BIT) << 10 | self.element.min(MAX_10BIT)
    }

    fn unpack(value: u32) -> Self {
        Self {
            id: value >> 20 & MAX_10BIT,
            class: value >> 10 & MAX_10BIT,
            element: value & MAX_10BIT,
        }
    }

    fn add(&mut self, other: Self) {
        self.id += other.id;
        self.class += other.class;
        self.element += other.element;
    }
}

fn max_specificity<P>(list: &[Selector<P>]) -> Counts {
    Counts::unpack(list.iter().map(|s| s.specificity).max().unwrap_or(0))
}

impl<P> Selector<P> {
    pub(crate) fn new(compounds: Vec<Vec<Simple<P>>>, combinators: Vec<Combinator>) -> Self {
        let mut counts = Counts::default();
        for simple in compounds.iter().flatten() {
            match simple {
                Simple::Universal | Simple::NoNamespace | Simple::Where(_) => {},
                Simple::LocalName { .. } | Simple::Part(_) => counts.element += 1,
                Simple::Id(_) => counts.id += 1,
                Simple::Class(_)
                | Simple::Attribute(_)
                | Simple::Root
                | Simple::Empty
                | Simple::Scope
                | Simple::Only { .. }
                | Simple::State(_) => counts.class += 1,
                Simple::Nth(nth) => {
                    counts.class += 1;
                    counts.add(max_specificity(&nth.of));
                },
                Simple::Not(list) | Simple::Is(list) => counts.add(max_specificity(list)),
                Simple::Host(inner) => {
                    counts.class += 1;
                    if let Some(inner) = inner {
                        counts.add(Counts::unpack(inner.specificity));
                    }
                },
                Simple::Slotted(inner) => {
                    counts.element += 1;
                    counts.add(Counts::unpack(inner.specificity));
                },
            }
        }
        Self {
            compounds,
            combinators,
            specificity: counts.pack(),
        }
    }

    /// Packed `id << 20 | class << 10 | element`, each capped at 10 bits.
    pub fn specificity(&self) -> u32 {
        self.specificity
    }

    /// The subject compound.
    pub fn subject(&self) -> &[Simple<P>] {
        &self.compounds[0]
    }

    pub fn reach(&self) -> Reach {
        let subject = self.subject();
        if subject.iter().any(|s| matches!(s, Simple::Slotted(_))) {
            Reach::Slotted
        } else if subject.iter().any(|s| matches!(s, Simple::Part(_))) {
            Reach::Part
        } else if subject.iter().any(|s| matches!(s, Simple::Host(_))) {
            Reach::Host
        } else {
            Reach::Inner
        }
    }

    /// The most selective name the subject compound requires. Arguments of
    /// `:is()`, `:where()` and `:not()` are not descended into: a key from one
    /// branch would be unsound for the others. A selector that reaches across
    /// a tree scope keys as universal, since the boundary picks its subject.
    pub fn subject_key(&self) -> Key<'_> {
        if self.reach() != Reach::Inner {
            return Key::Universal;
        }
        let find = |pick: fn(&Simple<P>) -> Option<&str>| self.subject().iter().find_map(pick);
        if let Some(id) = find(|s| match s {
            Simple::Id(id) => Some(id),
            _ => None,
        }) {
            return Key::Id(id);
        }
        if let Some(class) = find(|s| match s {
            Simple::Class(class) => Some(class),
            _ => None,
        }) {
            return Key::Class(class);
        }
        if let Some(lower) = find(|s| match s {
            Simple::LocalName { lower, .. } => Some(lower),
            _ => None,
        }) {
            return Key::LocalName(lower);
        }
        Key::Universal
    }

    fn any_simple(&self, test: &impl Fn(&Simple<P>) -> bool) -> bool {
        self.compounds.iter().flatten().any(|simple| {
            test(simple)
                || match simple {
                    Simple::Not(list) | Simple::Is(list) | Simple::Where(list) => {
                        list.iter().any(|s| s.any_simple(test))
                    },
                    Simple::Nth(nth) => nth.of.iter().any(|s| s.any_simple(test)),
                    Simple::Host(Some(inner)) | Simple::Slotted(inner) => inner.any_simple(test),
                    _ => false,
                }
        })
    }

    fn any_combinator(&self, test: &impl Fn(Combinator) -> bool) -> bool {
        self.combinators.iter().any(|c| test(*c))
            || self.compounds.iter().flatten().any(|simple| match simple {
                Simple::Not(list) | Simple::Is(list) | Simple::Where(list) => {
                    list.iter().any(|s| s.any_combinator(test))
                },
                Simple::Nth(nth) => nth.of.iter().any(|s| s.any_combinator(test)),
                _ => false,
            })
    }

    /// A change to one element may alter the match of a sibling: a sibling
    /// combinator anywhere, or an `of S` filter that renumbers siblings.
    pub fn depends_on_siblings(&self) -> bool {
        self.any_combinator(&|c| matches!(c, Combinator::NextSibling | Combinator::LaterSibling))
            || self.any_simple(&|s| matches!(s, Simple::Nth(nth) if !nth.of.is_empty()))
    }

    /// A child-list change may alter the match: `:empty` or a positional form.
    pub fn depends_on_structure(&self) -> bool {
        self.any_simple(&|s| matches!(s, Simple::Empty | Simple::Nth(_) | Simple::Only { .. }))
    }
}

/// A comma-separated selector list.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectorList<P>(pub(crate) Vec<Selector<P>>);

impl<P> SelectorList<P> {
    pub fn selectors(&self) -> &[Selector<P>] {
        &self.0
    }
}
