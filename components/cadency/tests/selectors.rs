// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::rc::Rc;

use cadency::{
    AttrOperation, CaseSensitivity, Element, Key, NamespaceConstraint, ParseErrorKind, PseudoClass,
    Reach, SelectorList,
};

#[derive(Clone, Debug, PartialEq)]
enum State {
    Hover,
    Checked,
}

impl PseudoClass for State {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "hover" => Some(Self::Hover),
            "checked" => Some(Self::Checked),
            _ => None,
        }
    }
}

type List = SelectorList<State>;

#[derive(Default)]
struct Node {
    name: &'static str,
    attributes: Vec<(&'static str, &'static str)>,
    /// `None` for a root, and for a child of a shadow root.
    parent: Option<usize>,
    children: Vec<usize>,
    text: bool,
    hover: bool,
    /// Set on the children of a shadow root.
    shadow_host: Option<usize>,
    /// Set on every node inside a shadow tree.
    containing_host: Option<usize>,
    slot: Option<usize>,
}

#[derive(Default)]
struct Dom(Vec<Node>);

impl Dom {
    fn add(&mut self, parent: Option<usize>, name: &'static str, attributes: &[(&'static str, &'static str)]) -> usize {
        let id = self.0.len();
        let containing_host = parent.and_then(|p| self.0[p].containing_host);
        self.0.push(Node {
            name,
            attributes: attributes.to_vec(),
            parent,
            containing_host,
            ..Node::default()
        });
        if let Some(parent) = parent {
            self.0[parent].children.push(id);
        }
        id
    }

    /// A top-level child of `host`'s shadow root.
    fn add_shadow(&mut self, host: usize, name: &'static str, attributes: &[(&'static str, &'static str)]) -> usize {
        let id = self.add(None, name, attributes);
        self.0[id].shadow_host = Some(host);
        self.0[id].containing_host = Some(host);
        id
    }
}

#[derive(Clone)]
struct El(Rc<Dom>, usize);

impl El {
    fn node(&self) -> &Node {
        &self.0.0[self.1]
    }

    fn at(&self, id: usize) -> Self {
        Self(self.0.clone(), id)
    }

    fn attribute(&self, name: &str) -> Option<&'static str> {
        self.node().attributes.iter().find(|(n, _)| *n == name).map(|(_, v)| *v)
    }

    fn sibling(&self, offset: isize) -> Option<Self> {
        let node = self.node();
        let siblings: Vec<usize> = match (node.parent, node.shadow_host) {
            (Some(parent), _) => self.0.0[parent].children.clone(),
            (None, Some(host)) => (0..self.0.0.len())
                .filter(|id| self.0.0[*id].shadow_host == Some(host))
                .collect(),
            (None, None) => return None,
        };
        let index = siblings.iter().position(|id| *id == self.1)? as isize + offset;
        Some(self.at(*siblings.get(usize::try_from(index).ok()?)?))
    }
}

impl Element for El {
    type PseudoClass = State;

    fn is_same(&self, other: &Self) -> bool {
        self.1 == other.1
    }
    fn parent_element(&self) -> Option<Self> {
        self.node().parent.map(|id| self.at(id))
    }
    fn prev_sibling_element(&self) -> Option<Self> {
        self.sibling(-1)
    }
    fn next_sibling_element(&self) -> Option<Self> {
        self.sibling(1)
    }
    fn is_root(&self) -> bool {
        self.node().parent.is_none() && self.node().shadow_host.is_none()
    }
    fn is_empty(&self) -> bool {
        self.node().children.is_empty() && !self.node().text
    }
    fn is_html_element_in_html_document(&self) -> bool {
        true
    }
    fn has_local_name(&self, name: &str) -> bool {
        self.node().name == name
    }
    fn has_namespace(&self, url: &str) -> bool {
        url == "http://www.w3.org/1999/xhtml"
    }
    fn is_same_type(&self, other: &Self) -> bool {
        self.node().name == other.node().name
    }
    fn has_id(&self, id: &str, case: CaseSensitivity) -> bool {
        self.attribute("id").is_some_and(|value| case.eq(value, id))
    }
    fn has_class(&self, class: &str, case: CaseSensitivity) -> bool {
        self.attribute("class")
            .is_some_and(|value| value.split_ascii_whitespace().any(|c| case.eq(c, class)))
    }
    fn attr_matches(&self, namespace: &NamespaceConstraint<'_>, name: &str, operation: &AttrOperation<'_>) -> bool {
        matches!(namespace, NamespaceConstraint::Any | NamespaceConstraint::Specific(""))
            && self.attribute(name).is_some_and(|value| operation.eval_str(value))
    }
    fn matches_pseudo_class(&self, state: &State) -> bool {
        match state {
            State::Hover => self.node().hover,
            State::Checked => self.attribute("checked").is_some(),
        }
    }
    fn parent_node_is_shadow_root(&self) -> bool {
        self.node().shadow_host.is_some()
    }
    fn containing_shadow_host(&self) -> Option<Self> {
        self.node().containing_host.map(|id| self.at(id))
    }
    fn assigned_slot(&self) -> Option<Self> {
        self.node().slot.map(|id| self.at(id))
    }
    fn is_slot(&self) -> bool {
        self.node().name == "slot"
    }
    fn is_part(&self, name: &str) -> bool {
        self.attribute("part").is_some_and(|v| v.split_ascii_whitespace().any(|p| p == name))
    }
    fn imported_part(&self, outer: &str) -> Option<String> {
        self.attribute("exportparts")?.split(',').find_map(|entry| {
            let (inner, exposed) = entry.trim().split_once(':').unwrap_or((entry.trim(), entry.trim()));
            (exposed.trim() == outer).then(|| inner.trim().to_string())
        })
    }
}

/// ```text
/// html
///   body
///     ul#list
///       li.a  li.b[lang=EN]  li.a(hover)  li.b  li.a
///     p(text)  p(empty)
///     x-card#card            shadow: section.frame > slot, button[part=action]
///       span.lit (slotted)             x-inner[exportparts="knob: handle"]
///                                        shadow: i[part=knob]
/// ```
struct Fixture {
    dom: Rc<Dom>,
    li: [usize; 5],
    p_text: usize,
    p_empty: usize,
    card: usize,
    lit: usize,
    section: usize,
    slot: usize,
    button: usize,
    knob: usize,
}

impl Fixture {
    fn new() -> Self {
        let mut dom = Dom::default();
        let html = dom.add(None, "html", &[]);
        let body = dom.add(Some(html), "body", &[]);
        let ul = dom.add(Some(body), "ul", &[("id", "list")]);
        let li = [
            dom.add(Some(ul), "li", &[("class", "a")]),
            dom.add(Some(ul), "li", &[("class", "b"), ("lang", "EN"), ("data-k", "x-y z")]),
            dom.add(Some(ul), "li", &[("class", "a")]),
            dom.add(Some(ul), "li", &[("class", "b")]),
            dom.add(Some(ul), "li", &[("class", "a")]),
        ];
        dom.0[li[2]].hover = true;
        let p_text = dom.add(Some(body), "p", &[]);
        dom.0[p_text].text = true;
        let p_empty = dom.add(Some(body), "p", &[]);
        let card = dom.add(Some(body), "x-card", &[("id", "card"), ("class", "wide")]);
        let lit = dom.add(Some(card), "span", &[("class", "lit")]);
        let section = dom.add_shadow(card, "section", &[("class", "frame")]);
        let slot = dom.add(Some(section), "slot", &[]);
        let button = dom.add_shadow(card, "button", &[("part", "action")]);
        let inner = dom.add_shadow(card, "x-inner", &[("exportparts", "knob: handle")]);
        let knob = dom.add_shadow(inner, "i", &[("part", "knob")]);
        dom.0[lit].slot = Some(slot);
        Self {
            dom: Rc::new(dom),
            li,
            p_text,
            p_empty,
            card,
            lit,
            section,
            slot,
            button,
            knob,
        }
    }

    fn el(&self, id: usize) -> El {
        El(self.dom.clone(), id)
    }

    fn matches(&self, selector: &str, id: usize) -> bool {
        List::parse(selector).unwrap().matches(&self.el(id), None)
    }

    fn matches_in(&self, selector: &str, id: usize, host: usize) -> bool {
        List::parse(selector).unwrap().matches(&self.el(id), Some(&self.el(host)))
    }

    /// Which of the five `li` match.
    fn li(&self, selector: &str) -> Vec<usize> {
        (0..5).filter(|i| self.matches(selector, self.li[*i])).map(|i| i + 1).collect()
    }
}

#[test]
fn the_enabled_grammar_parses() {
    for selector in [
        "*", "li", "LI", "*|li", "|li", "*|*", "#list", ".a.b", "li.a#x",
        "[lang]", "[lang=en]", "[lang='en' i]", "[lang=\"en\" s]", "[*|lang~=en]", "[|lang|=en]",
        "[a^=b]", "[a$=b]", "[a*=b]", "ul li", "ul > li", "li + li", "li ~ li", "ul>li",
        " ul  >  li ", "a, b , c",
        ":root", ":empty", ":scope", ":first-child", ":last-child", ":only-child",
        ":first-of-type", ":last-of-type", ":only-of-type",
        ":nth-child(2n+1)", ":nth-child( odd )", ":nth-last-child(-n+3)", ":nth-of-type(2)",
        ":nth-last-of-type(even)", ":nth-child(2 of .a, .b)", ":nth-last-child(1 of li > b)",
        ":not(.a)", ":not(.a, ul > .b)", ":is(.a, .b)", ":where(ul li)", ":is(:bogus, .a)", ":is()",
        "li:hover", "li:HOVER:checked",
        ":host", ":host(.wide)", ":host( .wide )", ":host > section",
        "::slotted(span)", "slot::slotted(.lit)", "section ::slotted(*)",
        "::part(action)", "x-card::part(action handle)", "body x-card::part(action):hover",
        "::part(action):not(:hover)",
        // Forgiving: the bad argument is dropped, leaving an empty `:is()`.
        ":is(::part(a))",
    ] {
        assert!(List::parse(selector).is_ok(), "{selector} should parse: {:?}", List::parse(selector));
    }
}

#[test]
fn everything_else_is_a_parse_error() {
    for selector in [
        "", ",", "li,", ", li", "li >", "> li", "li > > b", "li!", "#1", ".", "li .", "[", "[]",
        "[=a]", "[a=]", "[a=b x]", "[a=b i i]",
        "svg|rect", "[svg|href]", "*|", "|",
        ":has(li)", "li:has(> b)", ":bogus", ":nth-child()", ":nth-child(x)", ":nth-of-type(2 of .a)",
        ":not()", ":not(:bogus)", ":not(.a,)", ":host(ul li)", ":host-context(.x)",
        "::before", ":before", ":after", ":first-line", "::marker", "::slotted()", "::slotted(a b)",
        "::part()", "::part(a, b)", "::bogus(x)",
        "::slotted(span) b", "::slotted(span).x", "::slotted(span):hover", "::slotted(span)::part(x)",
        "::part(a) b", "::part(a) > b", "::part(a).x", "::part(a)#x", "::part(a)[x]",
        "::part(a):first-child", "::part(a):nth-child(1)", "::part(a):not(.x)", "::part(a)::part(b)",
        ":not(::slotted(a))", ":host(::part(a))",
    ] {
        assert!(List::parse(selector).is_err(), "{selector:?} should not parse");
    }
    assert_eq!(List::parse("svg|rect").unwrap_err().kind, ParseErrorKind::NamespacePrefix("svg".into()));
    assert_eq!(List::parse("li:bogus").unwrap_err().kind, ParseErrorKind::UnsupportedPseudo(":bogus".into()));
    assert_eq!(List::parse("::part(a).x").unwrap_err().kind, ParseErrorKind::Misplaced);
}

#[test]
fn specificity_is_packed_id_class_element() {
    let spec = |selector: &str| {
        let value = List::parse(selector).unwrap().selectors()[0].specificity();
        (value >> 20, value >> 10 & 1023, value & 1023)
    };
    assert_eq!(spec("*"), (0, 0, 0));
    assert_eq!(spec("li"), (0, 0, 1));
    assert_eq!(spec("ul > li.a"), (0, 1, 2));
    assert_eq!(spec("#list li[lang]:hover"), (1, 2, 1));
    assert_eq!(spec(":root:empty:first-child"), (0, 3, 0));
    assert_eq!(spec(":is(li, #list, .a)"), (1, 0, 0));
    assert_eq!(spec(":not(li, .a.b)"), (0, 2, 0));
    assert_eq!(spec(":where(#list .a)"), (0, 0, 0));
    assert_eq!(spec(":nth-child(2n)"), (0, 1, 0));
    assert_eq!(spec(":nth-child(2n of #list, li)"), (1, 1, 0));
    assert_eq!(spec(":host"), (0, 1, 0));
    assert_eq!(spec(":host(#card.wide)"), (1, 2, 0));
    assert_eq!(spec("::slotted(span.lit)"), (0, 1, 2));
    assert_eq!(spec("x-card::part(action):hover"), (0, 1, 2));
    let many = ".a".repeat(2000);
    assert_eq!(spec(&many), (0, 1023, 0));
}

#[test]
fn a_list_reports_its_strongest_match() {
    let f = Fixture::new();
    let list = List::parse("li, .a, #nope").unwrap();
    assert_eq!(list.matching_specificity(&f.el(f.li[0]), None), Some(1 << 10));
    assert_eq!(list.matching_specificity(&f.el(f.li[1]), None), Some(1));
    assert_eq!(list.matching_specificity(&f.el(f.p_text), None), None);
}

#[test]
fn combinators_walk_the_tree() {
    let f = Fixture::new();
    assert_eq!(f.li("ul li"), [1, 2, 3, 4, 5]);
    assert_eq!(f.li("html li"), [1, 2, 3, 4, 5]);
    assert_eq!(f.li("body > li"), [] as [usize; 0]);
    assert_eq!(f.li("body > ul > li"), [1, 2, 3, 4, 5]);
    assert_eq!(f.li(".b + li"), [3, 5]);
    assert_eq!(f.li(".b ~ li"), [3, 4, 5]);
    assert_eq!(f.li(".a + .b + .a"), [3, 5]);
    // Backtracking: the nearest `.a` sibling is not the one after `.b`.
    assert_eq!(f.li(".b + .a ~ .a"), [5]);
    assert_eq!(f.li("p li, #list > .b"), [2, 4]);
}

#[test]
fn structural_forms_count_siblings() {
    let f = Fixture::new();
    assert_eq!(f.li(":first-child"), [1]);
    assert_eq!(f.li(":last-child"), [5]);
    assert_eq!(f.li(":only-child"), [] as [usize; 0]);
    assert_eq!(f.li(":nth-child(odd)"), [1, 3, 5]);
    assert_eq!(f.li(":nth-child(2n)"), [2, 4]);
    assert_eq!(f.li(":nth-child(-n+2)"), [1, 2]);
    assert_eq!(f.li(":nth-child(n+4)"), [4, 5]);
    assert_eq!(f.li(":nth-child(0n+3)"), [3]);
    assert_eq!(f.li(":nth-child(-2n+5)"), [1, 3, 5]);
    assert_eq!(f.li(":nth-child(0)"), [] as [usize; 0]);
    assert_eq!(f.li(":nth-last-child(2)"), [4]);
    assert_eq!(f.li(":nth-child(2 of .a)"), [3]);
    assert_eq!(f.li(":nth-child(odd of .a)"), [1, 5]);
    assert_eq!(f.li(":nth-last-child(1 of .b)"), [4]);
    assert_eq!(f.li(":nth-child(1 of .b)"), [2]);
    assert!(f.matches("p:nth-of-type(2)", f.p_empty));
    assert!(!f.matches("p:nth-child(2)", f.p_empty));
    assert!(f.matches("p:first-of-type", f.p_text));
    assert!(f.matches("p:last-of-type:empty", f.p_empty));
    assert!(!f.matches("p:empty", f.p_text));
    assert!(f.matches("ul:only-of-type", f.dom.0[f.li[0]].parent.unwrap()));
    assert!(f.matches(":root", 0) && f.matches(":scope", 0) && !f.matches(":root", f.li[0]));
}

#[test]
fn attributes_states_and_logic() {
    let f = Fixture::new();
    assert_eq!(f.li("[lang]"), [2]);
    // `lang` is on HTML's case-insensitive list; `data-k` is not.
    assert_eq!(f.li("[lang=en]"), [2]);
    assert_eq!(f.li("[lang=en s]"), [] as [usize; 0]);
    assert_eq!(f.li("[data-k='X-Y Z']"), [] as [usize; 0]);
    assert_eq!(f.li("[data-k='X-Y Z' i]"), [2]);
    assert_eq!(f.li("[data-k~=z]"), [2]);
    assert_eq!(f.li("[data-k|=x]"), [2]);
    assert_eq!(f.li("[data-k^=x-][data-k$=' z'][data-k*='y z']"), [2]);
    assert_eq!(f.li("[data-k^='']"), [] as [usize; 0]);
    assert_eq!(f.li("[*|lang]"), [2]);
    assert_eq!(f.li("LI"), [1, 2, 3, 4, 5]);
    assert_eq!(f.li("*|li"), [1, 2, 3, 4, 5]);
    assert_eq!(f.li("|li"), [] as [usize; 0]);
    assert_eq!(f.li(":hover"), [3]);
    assert_eq!(f.li(":not(.a)"), [2, 4]);
    assert_eq!(f.li(":not(.a, :nth-child(2))"), [4]);
    assert_eq!(f.li(":is(.b, :hover)"), [2, 3, 4]);
    assert_eq!(f.li(":is(:bogus, .b)"), [2, 4]);
    assert_eq!(f.li(":is()"), [] as [usize; 0]);
    assert_eq!(f.li(":where(ul > .a):not(:hover)"), [1, 5]);
}

#[test]
fn a_shadow_tree_sees_its_host_only_through_host() {
    let f = Fixture::new();
    assert!(f.matches_in(":host", f.card, f.card));
    assert!(f.matches_in(":host(.wide)", f.card, f.card));
    assert!(!f.matches_in(":host(.narrow)", f.card, f.card));
    assert!(!f.matches(":host", f.card));
    assert!(!f.matches_in(":host", f.section, f.card));

    assert!(f.matches_in(":host > section", f.section, f.card));
    assert!(f.matches_in(":host(.wide) slot", f.slot, f.card));
    assert!(f.matches_in(":is(:host, .nope) > .frame", f.section, f.card));
    // The host is featureless from inside: no type, id or class reaches it,
    // and nothing above it is visible.
    assert!(!f.matches_in("x-card > section", f.section, f.card));
    assert!(!f.matches_in("#card section", f.section, f.card));
    assert!(!f.matches_in("body section", f.section, f.card));
    assert!(!f.matches_in("body :host > section", f.section, f.card));
    assert!(!f.matches_in(":not(:host) > section", f.section, f.card));
    assert!(!f.matches_in("section:root", f.section, f.card));
}

#[test]
fn slotted_and_part_cross_the_boundary() {
    let f = Fixture::new();
    assert!(f.matches_in("::slotted(span)", f.lit, f.card));
    assert!(f.matches_in("slot::slotted(.lit)", f.lit, f.card));
    assert!(f.matches_in(".frame > slot::slotted(*)", f.lit, f.card));
    assert!(f.matches_in(":host(.wide) ::slotted(.lit)", f.lit, f.card));
    assert!(!f.matches_in("::slotted(b)", f.lit, f.card));
    assert!(!f.matches_in("button::slotted(span)", f.lit, f.card));
    assert!(!f.matches("::slotted(span)", f.lit));
    assert!(!f.matches_in("::slotted(*)", f.slot, f.card));

    assert!(f.matches("::part(action)", f.button));
    assert!(f.matches("body > x-card.wide::part(action)", f.button));
    assert!(!f.matches("p::part(action)", f.button));
    assert!(!f.matches("::part(other)", f.button));
    assert!(!f.matches("::part(action):hover", f.button));
    assert!(f.matches("::part(action):not(:hover)", f.button));
    assert!(!f.matches("::part(action)", f.lit));

    // `knob` sits two trees down and is exposed to the document as `handle`.
    assert!(f.matches("x-card::part(handle)", f.knob));
    assert!(!f.matches("x-card::part(knob)", f.knob));
    // From inside x-card's tree it is one boundary away, under its own name.
    assert!(f.matches_in("x-inner::part(knob)", f.knob, f.card));
}

#[test]
fn index_facts_read_off_the_parsed_form() {
    let one = |selector: &str| List::parse(selector).unwrap().selectors()[0].clone();
    assert_eq!(one("ul li.a#x").subject_key(), Key::Id("x"));
    assert_eq!(one("#list LI.a").subject_key(), Key::Class("a"));
    assert_eq!(one("#list LI").subject_key(), Key::LocalName("li"));
    assert_eq!(one("#list > *").subject_key(), Key::Universal);
    assert_eq!(one(":is(#a, .b)").subject_key(), Key::Universal);
    assert_eq!(one("[lang]:hover").subject_key(), Key::Universal);
    assert_eq!(one(".x::part(a)").subject_key(), Key::Universal);
    assert_eq!(one(":host(.x)").subject_key(), Key::Universal);

    assert_eq!(one(".a .b").reach(), Reach::Inner);
    assert_eq!(one(":host > .b").reach(), Reach::Inner);
    assert_eq!(one(":host(.a)").reach(), Reach::Host);
    assert_eq!(one("slot::slotted(a)").reach(), Reach::Slotted);
    assert_eq!(one("x::part(a):hover").reach(), Reach::Part);

    let deps = |selector: &str| {
        let s = one(selector);
        (s.depends_on_siblings(), s.depends_on_structure())
    };
    assert_eq!(deps(".card .label"), (false, false));
    assert_eq!(deps(".card + .card"), (true, false));
    assert_eq!(deps(":not(a ~ b) c"), (true, false));
    assert_eq!(deps("li:nth-child(2n+1)"), (false, true));
    assert_eq!(deps("li:nth-child(2 of .a)"), (true, true));
    assert_eq!(deps("[data-key='a+b'] .label"), (false, false));
    assert_eq!(deps(":is(p:empty) b"), (false, true));
    assert_eq!(deps("li:only-of-type"), (false, true));
}
