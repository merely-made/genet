// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Attribute selectors.

/// How two strings compare.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaseSensitivity {
    CaseSensitive,
    AsciiCaseInsensitive,
}

impl CaseSensitivity {
    pub fn eq(self, a: &str, b: &str) -> bool {
        match self {
            Self::CaseSensitive => a == b,
            Self::AsciiCaseInsensitive => a.eq_ignore_ascii_case(b),
        }
    }

    fn contains(self, haystack: &str, needle: &str) -> bool {
        match self {
            Self::CaseSensitive => haystack.contains(needle),
            Self::AsciiCaseInsensitive => haystack
                .as_bytes()
                .windows(needle.len())
                .any(|window| window.eq_ignore_ascii_case(needle.as_bytes())),
        }
    }
}

/// The namespace an attribute selector asks for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NamespaceConstraint<'a> {
    /// `[*|attr]`.
    Any,
    /// `[attr]` and `[|attr]` ask for the empty namespace.
    Specific(&'a str),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttrOperator {
    Equal,
    Includes,
    DashMatch,
    Prefix,
    Substring,
    Suffix,
}

/// What an attribute selector asks of a value, resolved for one element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttrOperation<'a> {
    Exists,
    WithValue {
        operator: AttrOperator,
        case_sensitivity: CaseSensitivity,
        value: &'a str,
    },
}

impl AttrOperation<'_> {
    /// Evaluate against one attribute value of the element.
    pub fn eval_str(&self, attribute: &str) -> bool {
        let Self::WithValue {
            operator,
            case_sensitivity: case,
            value,
        } = *self
        else {
            return true;
        };
        let boundary = |at: usize| attribute.is_char_boundary(at);
        match operator {
            AttrOperator::Equal => case.eq(attribute, value),
            AttrOperator::Prefix => {
                !value.is_empty()
                    && attribute.len() >= value.len()
                    && boundary(value.len())
                    && case.eq(&attribute[..value.len()], value)
            },
            AttrOperator::Suffix => {
                let Some(start) = attribute.len().checked_sub(value.len()) else {
                    return false;
                };
                !value.is_empty() && boundary(start) && case.eq(&attribute[start..], value)
            },
            AttrOperator::Substring => !value.is_empty() && case.contains(attribute, value),
            AttrOperator::Includes => {
                !value.is_empty()
                    && attribute
                        .split([' ', '\t', '\n', '\r', '\x0C'])
                        .any(|part| case.eq(part, value))
            },
            AttrOperator::DashMatch => {
                case.eq(attribute, value)
                    || (attribute.as_bytes().get(value.len()) == Some(&b'-')
                        && boundary(value.len())
                        && case.eq(&attribute[..value.len()], value))
            },
        }
    }
}

/// Case rule as written, before the element is known.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParsedCase {
    Sensitive,
    Insensitive,
    /// One of HTML's legacy enumerated attributes: insensitive on an HTML
    /// element in an HTML document, sensitive elsewhere.
    InsensitiveInHtml,
}

/// A parsed attribute selector.
#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    /// `true` for `[*|attr]`.
    pub(crate) any_namespace: bool,
    pub(crate) name: Box<str>,
    pub(crate) name_lower: Box<str>,
    pub(crate) operation: Option<(AttrOperator, ParsedCase, Box<str>)>,
}

/// HTML's list of attributes whose values match ASCII case-insensitively.
/// <https://html.spec.whatwg.org/multipage/semantics-other.html#case-sensitivity-of-selectors>
pub(crate) const HTML_CASE_INSENSITIVE_ATTRIBUTES: &[&str] = &[
    "accept",
    "accept-charset",
    "align",
    "alink",
    "axis",
    "bgcolor",
    "charset",
    "checked",
    "clear",
    "codetype",
    "color",
    "compact",
    "declare",
    "defer",
    "dir",
    "direction",
    "disabled",
    "enctype",
    "face",
    "frame",
    "hreflang",
    "http-equiv",
    "lang",
    "language",
    "link",
    "media",
    "method",
    "multiple",
    "nohref",
    "noresize",
    "noshade",
    "nowrap",
    "readonly",
    "rel",
    "rev",
    "rules",
    "scope",
    "scrolling",
    "selected",
    "shape",
    "target",
    "text",
    "type",
    "valign",
    "valuetype",
    "vlink",
];
