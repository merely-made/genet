// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The cascade's selector index, tested at the only level that matters: the
//! computed values it produces.
//!
//! The index buckets rules by what their rightmost compound requires of a
//! candidate element, so the cascade offers each element a handful of rules
//! instead of the whole style set. Every case here is a way that bucketing
//! could quietly change an answer — a rule reached through the wrong key, a
//! rule not reached at all, or rules reached in an order that is not source
//! order — and asserts that it does not.

use genet_livery::{Device, InteractionStates, StyleSet, resolve_styles};
use genet_static_dom::StaticDocument;
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

type Id = <StaticDocument as LayoutDom>::NodeId;

fn find(dom: &StaticDocument, id: Id, needle: &str) -> Option<Id> {
    if dom.kind(id) == NodeKind::Element
        && dom.attribute(id, &Namespace::from(""), &LocalName::from("id")) == Some(needle)
    {
        return Some(id);
    }
    dom.dom_children(id)
        .find_map(|child| find(dom, child, needle))
}

fn by_id(dom: &StaticDocument, needle: &str) -> Id {
    find(dom, dom.document(), needle).unwrap_or_else(|| panic!("no element with id {needle}"))
}

/// Resolve `page` against `css` and read one element's computed property.
fn computed(page: &str, css: &str, element: &str, property: &str) -> String {
    let dom = StaticDocument::parse(page);
    let styles = resolve_styles(
        &dom,
        &StyleSet::cambium(&[css]),
        &Device::screen(800.0, 600.0),
        &InteractionStates::default(),
    );
    styles
        .computed_style(by_id(&dom, element), property)
        .unwrap_or_else(|| panic!("no computed {property} for {element}"))
}

const PAGE: &str = "<!doctype html><html><body>\
     <section class='b' id='outer'>\
       <div class='a' id='target' data-k='v'>text</div>\
     </section>\
     </body></html>";

/// Two rules of *equal* specificity that land in different buckets — one in
/// the class bucket, one in the local-name bucket. The cascade's tie-break is
/// source order, so whichever is written second must win. If the index handed
/// its buckets over one after another instead of merging them back into source
/// order, exactly one of these two directions would come out wrong.
#[test]
fn equal_specificity_rules_in_different_buckets_keep_source_order() {
    assert_eq!(
        computed(
            PAGE,
            "div.a { color: rgb(255, 0, 0); } .b div { color: rgb(0, 0, 255); }",
            "target",
            "color"
        ),
        "rgb(0, 0, 255)",
        "the later rule wins at equal specificity, whichever bucket it sits in"
    );
    assert_eq!(
        computed(
            PAGE,
            ".b div { color: rgb(0, 0, 255); } div.a { color: rgb(255, 0, 0); }",
            "target",
            "color"
        ),
        "rgb(255, 0, 0)",
        "reversing the source order must reverse the winner"
    );
}

/// A compound that names nothing indexable has to be offered to every element.
/// An attribute selector, a structural pseudo-class and a bare universal are
/// the three shapes of that.
#[test]
fn compounds_naming_nothing_indexable_still_reach_every_element() {
    assert_eq!(
        computed(
            PAGE,
            "[data-k] { color: rgb(0, 128, 0); }",
            "target",
            "color"
        ),
        "rgb(0, 128, 0)",
        "an attribute-only compound belongs in the universal bucket"
    );
    assert_eq!(
        computed(PAGE, "* { color: rgb(0, 128, 0); }", "target", "color"),
        "rgb(0, 128, 0)",
        "the universal selector must reach every element"
    );
    assert_eq!(
        computed(
            PAGE,
            ":first-child { color: rgb(0, 128, 0); }",
            "target",
            "color"
        ),
        "rgb(0, 128, 0)",
        "a bare pseudo-class compound belongs in the universal bucket"
    );
}

/// An attribute or pseudo-class beside a name narrows the compound, so the
/// rule is bucketed on the name and still has to match — and still has to
/// *not* match when the narrowing part fails.
#[test]
fn a_narrowed_compound_is_bucketed_on_the_name_it_carries() {
    assert_eq!(
        computed(
            PAGE,
            "div[data-k='v'] { color: rgb(0, 128, 0); }",
            "target",
            "color"
        ),
        "rgb(0, 128, 0)",
        "the name bucket must still offer a rule whose compound also tests an attribute"
    );
    assert_eq!(
        computed(
            PAGE,
            "div[data-k='other'] { color: rgb(0, 128, 0); }",
            "target",
            "color"
        ),
        "rgb(0, 0, 0)",
        "being offered is not matching: the attribute test still has to fail the rule"
    );
}

/// `Element::has_local_name` compares ASCII-case-insensitively, so the index
/// has to key both sides the same way or an uppercase type selector would
/// silently stop matching.
#[test]
fn an_uppercase_type_selector_still_matches_a_lowercase_element() {
    assert_eq!(
        computed(PAGE, "DIV { color: rgb(0, 128, 0); }", "target", "color"),
        "rgb(0, 128, 0)",
        "a type selector's case must not decide which bucket it is found in"
    );
}

/// The key is the *rightmost* compound. A descendant combinator's left-hand
/// side must not put the rule in a bucket the subject cannot reach, and must
/// not offer it to an element that only carries the ancestor's name.
#[test]
fn the_key_is_the_rightmost_compound_not_the_leftmost() {
    assert_eq!(
        computed(PAGE, ".b .a { color: rgb(0, 128, 0); }", "target", "color"),
        "rgb(0, 128, 0)",
        "the subject is reached through its own class, not the ancestor's"
    );
    assert_eq!(
        computed(PAGE, ".b .a { color: rgb(0, 128, 0); }", "outer", "color"),
        "rgb(0, 0, 0)",
        "the ancestor carries `.b`, which is not this rule's key, and must not match"
    );
}

/// A selector list spreads one rule across several buckets. The rule has to be
/// reachable through each of them, and reaching it twice must not change the
/// answer — the merge deduplicates before matching.
#[test]
fn a_rule_is_reachable_through_every_key_in_its_selector_list() {
    assert_eq!(
        computed(
            PAGE,
            ".a, #nothing { color: rgb(0, 128, 0); }",
            "target",
            "color"
        ),
        "rgb(0, 128, 0)",
        "the class key reaches a rule whose list also names an id"
    );
    assert_eq!(
        computed(
            PAGE,
            "#target, .a, div { color: rgb(0, 128, 0); }",
            "target",
            "color"
        ),
        "rgb(0, 128, 0)",
        "an element hitting all three of a rule's buckets matches it once, not thrice"
    );
}

/// The id bucket is case-sensitive, as `NoQuirks` id matching is, and a
/// mismatched id must not be offered through it.
#[test]
fn the_id_bucket_follows_case_sensitive_id_matching() {
    assert_eq!(
        computed(
            PAGE,
            "#target { color: rgb(0, 128, 0); }",
            "target",
            "color"
        ),
        "rgb(0, 128, 0)",
        "an exact id must reach its element"
    );
    assert_eq!(
        computed(
            PAGE,
            "#TARGET { color: rgb(0, 128, 0); }",
            "target",
            "color"
        ),
        "rgb(0, 0, 0)",
        "id matching is case-sensitive, and the bucket must not widen it"
    );
}

/// The UA sheet is in the same flattened set as the author sheets, so it goes
/// through the same buckets. A `div` still has to get its UA `display: block`.
#[test]
fn the_ua_sheet_is_bucketed_with_the_rest_and_still_applies() {
    assert_eq!(
        computed(PAGE, "", "target", "display"),
        "block",
        "the UA sheet's type rules must survive bucketing"
    );
}
