// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Livery's selector surface over `cadency`: the state vocabulary, and the
//! per-list facts the cascade and the invalidator read.

use std::fmt;

pub use cadency::{AttrOperation, CaseSensitivity, Element, NamespaceConstraint};

use crate::cascade::Specificity;

/// Dynamic element states used by the first Cambium selector lane.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StatePseudoClass {
    Hover,
    Active,
    Focus,
    FocusWithin,
    Disabled,
    Checked,
}

impl cadency::PseudoClass for StatePseudoClass {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "hover" => Self::Hover,
            "active" => Self::Active,
            "focus" => Self::Focus,
            "focus-within" => Self::FocusWithin,
            "disabled" => Self::Disabled,
            "checked" => Self::Checked,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectorParseError(String);

impl fmt::Display for SelectorParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for SelectorParseError {}

/// How far out of its own tree scope one selector in a list can reach.
/// A stylesheet in a shadow tree styles that tree, but three constructs
/// deliberately cross the boundary, and a document sheet must not reach in
/// except through `::part()`. Classified once at parse time so the cascade's
/// per-element loop is a comparison, not a selector inspection.
pub use cadency::Reach as SelectorReach;

/// A parsed selector list. Matching returns the strongest specificity among
/// the selectors in the list that matched the element.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectorList {
    selectors: cadency::SelectorList<StatePseudoClass>,
    /// Per-selector reach, aligned with `selectors.selectors()`.
    reach: Vec<SelectorReach>,
    sibling_dependency: bool,
    structural_dependency: bool,
}

impl SelectorList {
    pub fn parse(source: &str) -> Result<Self, SelectorParseError> {
        let selectors = cadency::SelectorList::parse(source)
            .map_err(|error| SelectorParseError(error.to_string()))?;
        let each = selectors.selectors();
        let sibling_dependency = each.iter().any(cadency::Selector::depends_on_siblings);
        Ok(Self {
            reach: each.iter().map(cadency::Selector::reach).collect(),
            sibling_dependency,
            // Deliberately conservative: a sibling dependency widens the
            // structural scope too, never narrows it.
            structural_dependency: sibling_dependency
                || each.iter().any(cadency::Selector::depends_on_structure),
            selectors,
        })
    }

    /// A changed element may alter the match of one of its following siblings.
    pub fn has_sibling_dependency(&self) -> bool {
        self.sibling_dependency
    }

    /// Child-list changes may alter `:empty`, positional, or sibling matching.
    pub fn has_structural_dependency(&self) -> bool {
        self.structural_dependency
    }

    pub fn matching_specificity<E>(&self, element: &E) -> Option<Specificity>
    where
        E: Element<PseudoClass = StatePseudoClass>,
    {
        self.matching_specificity_scoped(element, true, None)
    }

    /// Match with a tree-scope boundary.
    ///
    /// `same_scope` says whether this selector list's stylesheet lives in the
    /// same tree scope as `element`. When it does not, only the three
    /// boundary-crossing reaches are offered — that, and nothing about the
    /// selectors themselves, is what makes a shadow tree's styles private and
    /// keeps the document's styles out of it.
    ///
    /// `shadow_host` is the host of the *stylesheet's* scope, which is what
    /// `:host` compares against and what `::part()` climbs from.
    pub fn matching_specificity_scoped<E>(
        &self,
        element: &E,
        same_scope: bool,
        shadow_host: Option<E>,
    ) -> Option<Specificity>
    where
        E: Element<PseudoClass = StatePseudoClass>,
    {
        self.selectors
            .selectors()
            .iter()
            .zip(&self.reach)
            .filter(|(_, reach)| same_scope || **reach != SelectorReach::Inner)
            .filter(|(selector, _)| selector.matches(element, shadow_host.as_ref()))
            .map(|(selector, _)| Specificity(selector.specificity()))
            .max()
    }

    /// Whether any selector in this list can reach outside its own tree scope.
    /// A rule with none is skipped wholesale across a boundary.
    pub fn crosses_tree_scope(&self) -> bool {
        self.reach
            .iter()
            .any(|reach| *reach != SelectorReach::Inner)
    }

    /// One [`SelectorKey`] per selector in this list, in list order.
    ///
    /// A rule is a candidate for an element when *any* of its selectors' keys
    /// matches, so a caller indexes the rule under each distinct key returned.
    pub fn rightmost_keys(&self) -> Vec<SelectorKey> {
        self.selectors
            .selectors()
            .iter()
            .map(|selector| match selector.subject_key() {
                cadency::Key::Id(id) => SelectorKey::Id(id.into()),
                cadency::Key::Class(class) => SelectorKey::Class(class.into()),
                cadency::Key::LocalName(name) => SelectorKey::LocalName(name.into()),
                cadency::Key::Universal => SelectorKey::Universal,
            })
            .collect()
    }
}

/// The name a rule's rightmost compound requires of any element it can match.
///
/// This is bucketing input for a cascade's selector index, not a matching
/// decision: a key is sound only when *every* element the selector matches
/// carries it, so a compound that names nothing indexable — or one that can
/// reach across a tree scope — keys as [`SelectorKey::Universal`] and is
/// offered to every element. Attribute selectors and pseudo-classes only
/// narrow a compound, so they never change the key; they land in the bucket of
/// whatever else the compound names, else in the universal bucket.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelectorKey {
    /// The compound names `#id`. Case-sensitive, as matching is.
    Id(Box<str>),
    /// The compound names `.class`. Case-sensitive, as matching is.
    Class(Box<str>),
    /// The compound names an element type, ASCII-lowercased to match
    /// `Element::has_local_name`, which compares case-insensitively.
    LocalName(Box<str>),
    /// Nothing indexable; the rule must be offered to every element.
    Universal,
}
