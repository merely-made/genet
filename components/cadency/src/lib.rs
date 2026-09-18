// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! CSS selector parsing and matching over any element tree.
//!
//! Implement [`Element`] for the tree and [`PseudoClass`] for the dynamic
//! states it tracks, then [`SelectorList::parse`] and match. See the README
//! for what is and is not parsed.

mod ast;
mod attr;
mod matching;
mod parse;

use std::fmt;

pub use ast::{Combinator, Key, Nth, Reach, Selector, SelectorList, Simple};
pub use attr::{AttrOperation, AttrOperator, Attribute, CaseSensitivity, NamespaceConstraint};
pub use parse::{ParseError, ParseErrorKind};

/// The consumer's vocabulary of non-structural pseudo-classes (`:hover`,
/// `:checked`, ...). cadency hardcodes none.
pub trait PseudoClass: Clone + fmt::Debug + PartialEq {
    /// `name` is ASCII-lowercased and has no leading colon. `None` makes the
    /// selector invalid.
    fn parse(name: &str) -> Option<Self>;
}

/// No dynamic states: every non-structural pseudo-class is invalid.
impl PseudoClass for std::convert::Infallible {
    fn parse(_name: &str) -> Option<Self> {
        None
    }
}

/// An element of the tree being matched. Siblings and parents are elements
/// only; text and comments are skipped by the implementor.
pub trait Element: Sized + Clone {
    type PseudoClass: PseudoClass;

    /// Node identity, not structural equality.
    fn is_same(&self, other: &Self) -> bool;
    fn parent_element(&self) -> Option<Self>;
    fn prev_sibling_element(&self) -> Option<Self>;
    fn next_sibling_element(&self) -> Option<Self>;
    fn is_root(&self) -> bool;
    /// No element children and no non-empty text.
    fn is_empty(&self) -> bool;

    /// Selects ASCII-lowercased names and HTML's case-insensitive attributes.
    fn is_html_element_in_html_document(&self) -> bool;
    fn has_local_name(&self, name: &str) -> bool;
    fn has_namespace(&self, url: &str) -> bool;
    /// Same namespace and local name, for the `of-type` forms.
    fn is_same_type(&self, other: &Self) -> bool;
    fn has_id(&self, id: &str, case_sensitivity: CaseSensitivity) -> bool;
    fn has_class(&self, class: &str, case_sensitivity: CaseSensitivity) -> bool;
    /// Whether any attribute satisfies all three; test a value with
    /// [`AttrOperation::eval_str`].
    fn attr_matches(
        &self,
        namespace: &NamespaceConstraint<'_>,
        local_name: &str,
        operation: &AttrOperation<'_>,
    ) -> bool;
    fn matches_pseudo_class(&self, pseudo_class: &Self::PseudoClass) -> bool;

    // Shadow trees. The defaults describe a tree that has none.

    fn parent_node_is_shadow_root(&self) -> bool {
        false
    }
    fn containing_shadow_host(&self) -> Option<Self> {
        None
    }
    fn assigned_slot(&self) -> Option<Self> {
        None
    }
    fn is_slot(&self) -> bool {
        false
    }
    fn is_part(&self, _name: &str) -> bool {
        false
    }
    /// This host's `exportparts` mapping, read from the outer name a part is
    /// exposed under to the name it has inside this host's shadow tree.
    fn imported_part(&self, _outer_name: &str) -> Option<String> {
        None
    }
}
