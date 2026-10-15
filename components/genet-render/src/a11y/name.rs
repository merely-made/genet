/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Bounded DOM text alternatives, following AccName 1.2 (2026-09-23).
//! CSS rendering counts as hidden only when the style owner supplies it
//! (`with_rendered`). Name from content follows the flat tree (AccName's
//! "rendered child nodes"); ID references resolve only within the referring
//! node's own tree. Embedded control values remain open.

use std::collections::{HashMap, HashSet};

use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

pub(super) struct Names<'a, D: LayoutDom> {
    dom: &'a D,
    /// IDs per node tree, keyed by the tree's root (the document or a shadow
    /// root). A reference never crosses a shadow boundary.
    ids: HashMap<D::NodeId, HashMap<String, D::NodeId>>,
    /// Every `<label>` in tree order within its own tree, with that tree's root.
    labels: Vec<(D::NodeId, D::NodeId)>,
    generated: Option<&'a dyn Fn(D::NodeId) -> (String, String)>,
    rendered: Option<&'a dyn Fn(D::NodeId) -> bool>,
}

fn attr<'a, D: LayoutDom>(dom: &'a D, node: D::NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::default(), &LocalName::from(name))
}

/// Name of a button input with no `value`, `alt` or `title`. English only:
/// localizing these strings is unfinished (Mark, 2026-10-15).
fn default_button_name(kind: &str) -> &'static str {
    match kind {
        "submit" => "Submit",
        "reset" => "Reset",
        "image" => "Submit Query",
        _ => "",
    }
}

fn flat(text: &str) -> String {
    text.split_ascii_whitespace().collect::<Vec<_>>().join(" ")
}

impl<'a, D: LayoutDom> Names<'a, D> {
    pub(super) fn new(dom: &'a D) -> Self {
        let mut names = Self {
            dom,
            ids: HashMap::new(),
            labels: Vec::new(),
            generated: None,
            rendered: None,
        };
        // Each tree is walked in its own DOM order; a host's shadow root starts
        // a new tree, walked after the host's light-DOM subtree. Each tree is
        // still visited contiguously, so tree order holds within every tree.
        let mut pending = vec![(dom.document(), dom.document())];
        while let Some((node, tree)) = pending.pop() {
            if let Some(id) = attr(dom, node, "id") {
                names
                    .ids
                    .entry(tree)
                    .or_default()
                    .entry(id.to_owned())
                    .or_insert(node);
            }
            if names.tag(node) == Some("label") {
                names.labels.push((node, tree));
            }
            if let Some(root) = dom
                .has_shadow_trees()
                .then(|| dom.shadow_root(node))
                .flatten()
            {
                pending.push((root, root));
            }
            let children: Vec<_> = dom.dom_children(node).collect();
            pending.extend(children.into_iter().rev().map(|child| (child, tree)));
        }
        names
    }

    pub(super) fn with_generated(
        mut self,
        generated: &'a dyn Fn(D::NodeId) -> (String, String),
    ) -> Self {
        self.generated = Some(generated);
        self
    }

    /// CSS rendering from the style owner: a node it reports not rendered
    /// visible is hidden content, as `hidden` and `aria-hidden` are.
    pub(super) fn with_rendered(mut self, rendered: &'a dyn Fn(D::NodeId) -> bool) -> Self {
        self.rendered = Some(rendered);
        self
    }

    fn tag(&self, node: D::NodeId) -> Option<&str> {
        self.dom.element_name(node).map(|name| name.local.as_ref())
    }

    /// The lowercased `type` of an `input` that is a push button or image button.
    fn button_input(&self, node: D::NodeId) -> Option<&'static str> {
        if self.tag(node) != Some("input") {
            return None;
        }
        let kind = attr(self.dom, node, "type")?;
        ["button", "submit", "reset", "image"]
            .into_iter()
            .find(|k| kind.eq_ignore_ascii_case(k))
    }

    fn labelable(&self, node: D::NodeId) -> bool {
        match self.tag(node) {
            Some("input") => {
                !attr(self.dom, node, "type").is_some_and(|v| v.eq_ignore_ascii_case("hidden"))
            },
            Some("button" | "meter" | "output" | "progress" | "select" | "textarea") => true,
            _ => false,
        }
    }

    fn first_control(&self, node: D::NodeId) -> Option<D::NodeId> {
        for child in self.dom.dom_children(node) {
            if self.labelable(child) {
                return Some(child);
            }
            if let Some(control) = self.first_control(child) {
                return Some(control);
            }
        }
        None
    }

    /// The root of `node`'s own tree, the scope its ID references resolve in.
    fn tree(&self, node: D::NodeId) -> D::NodeId {
        if self.dom.has_shadow_trees() {
            self.dom.node_tree_root(node)
        } else {
            self.dom.document()
        }
    }

    fn element_by_id(&self, tree: D::NodeId, id: &str) -> Option<D::NodeId> {
        self.ids.get(&tree)?.get(id).copied()
    }

    fn references(&self, node: D::NodeId, attribute: &str) -> Vec<D::NodeId> {
        let mut seen = HashSet::new();
        let tree = self.tree(node);
        attr(self.dom, node, attribute)
            .unwrap_or("")
            .split_ascii_whitespace()
            .filter_map(|id| self.element_by_id(tree, id))
            .filter(|id| seen.insert(*id))
            .collect()
    }

    pub(super) fn name(&self, node: D::NodeId) -> Option<String> {
        self.compute(node, false, false, false, &mut HashSet::new())
            .map(|s| flat(&s))
    }

    pub(super) fn description(&self, node: D::NodeId) -> Option<String> {
        let references = self.references(node, "aria-describedby");
        if !references.is_empty() {
            return Some(
                references
                    .into_iter()
                    .map(|target| {
                        self.compute(
                            target,
                            true,
                            false,
                            self.hidden(target),
                            &mut HashSet::new(),
                        )
                        .unwrap_or_default()
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            )
            .map(|s| flat(&s));
        }
        if let Some(description) = attr(self.dom, node, "aria-description") {
            return Some(flat(description));
        }
        // Title is available for description only when a higher-priority name
        // source was used. Suppress title during that independent calculation.
        if self
            .compute(node, false, true, false, &mut HashSet::new())
            .is_some()
        {
            return attr(self.dom, node, "title").map(flat);
        }
        None
    }

    /// Whether the style owner reports `node` rendered and visible; true when
    /// no owner was supplied, which keeps the DOM-only entry points as they were.
    pub(super) fn rendered(&self, node: D::NodeId) -> bool {
        self.rendered.is_none_or(|rendered| rendered(node))
    }

    fn hidden(&self, node: D::NodeId) -> bool {
        attr(self.dom, node, "hidden").is_some()
            || attr(self.dom, node, "aria-hidden").is_some_and(|s| s.eq_ignore_ascii_case("true"))
            || (self.dom.kind(node) == NodeKind::Element && !self.rendered(node))
    }

    fn compute(
        &self,
        node: D::NodeId,
        referenced: bool,
        suppress_title: bool,
        include_hidden: bool,
        visited: &mut HashSet<D::NodeId>,
    ) -> Option<String> {
        // A directly referenced hidden subtree may contribute. In the normal
        // content traversal, hidden DOM subtrees cannot name their ancestors.
        if !include_hidden && self.hidden(node) {
            return None;
        }
        if !visited.insert(node) {
            return None;
        }
        let result = self.compute_inner(node, referenced, suppress_title, include_hidden, visited);
        visited.remove(&node);
        result
    }

    fn compute_inner(
        &self,
        node: D::NodeId,
        referenced: bool,
        suppress_title: bool,
        include_hidden: bool,
        visited: &mut HashSet<D::NodeId>,
    ) -> Option<String> {
        if self.dom.kind(node) == NodeKind::Text {
            return self.dom.text(node).map(str::to_owned);
        }
        if !referenced {
            let references = self.references(node, "aria-labelledby");
            if !references.is_empty() {
                return Some(
                    references
                        .into_iter()
                        .map(|target| {
                            // AccName does not recursively follow labelledby while
                            // already traversing that relation. Self references still
                            // contribute the node's aria-label/native/content source.
                            if target == node {
                                self.compute_inner(
                                    target,
                                    true,
                                    suppress_title,
                                    self.hidden(target),
                                    visited,
                                )
                            } else {
                                self.compute(target, true, false, self.hidden(target), visited)
                            }
                            .unwrap_or_default()
                        })
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
        if let Some(label) = attr(self.dom, node, "aria-label").filter(|s| !flat(s).is_empty()) {
            return Some(label.to_owned());
        }
        if self.labelable(node) {
            let labels: Vec<_> = self
                .labels
                .iter()
                .filter(|(label, tree)| {
                    if let Some(target) = attr(self.dom, *label, "for") {
                        self.element_by_id(*tree, target) == Some(node)
                    } else {
                        self.first_control(*label) == Some(node)
                    }
                })
                .map(|(label, _)| *label)
                .collect();
            if !labels.is_empty() {
                return Some(
                    labels
                        .into_iter()
                        .map(|label| {
                            self.compute(label, true, false, self.hidden(label), visited)
                                .unwrap_or_default()
                        })
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
        if self.tag(node) == Some("img") {
            if let Some(alt) = attr(self.dom, node, "alt") {
                return Some(alt.to_owned());
            }
        }
        if let Some(kind) = self.button_input(node) {
            let source = |name: &str| attr(self.dom, node, name).filter(|s| !flat(s).is_empty());
            let native = if kind == "image" { source("alt") } else { None };
            if let Some(text) = native.or_else(|| source("value")) {
                return Some(text.to_owned());
            }
            if let Some(title) = source("title") {
                return (!suppress_title).then(|| title.to_owned());
            }
            return Some(default_button_name(kind).to_owned());
        }
        // Native text fields get names from labels, never their editable value.
        let content = if matches!(self.tag(node), Some("input" | "textarea" | "select"))
            || super::document_role(self.dom, node) == document_session_api::DocumentA11yRole::TextField
            || self.dom.kind(node) == NodeKind::Document
            || (!referenced
                && matches!(
                    self.tag(node),
                    Some("html" | "body" | "main" | "article" | "nav" | "form")
                )) {
            String::new()
        } else {
            let (before, after) = self.generated.map(|f| f(node)).unwrap_or_default();
            // AccName 2F's rendered child nodes: a host's shadow root, a
            // slot's assigned nodes, otherwise the node's own children.
            let children = self
                .dom
                .flat_children(node)
                .map(|child| {
                    self.compute(child, referenced, false, include_hidden, visited)
                        .unwrap_or_default()
                })
                .collect::<String>();
            // The provider supplies admitted rendered inline text. Block
            // pseudo-elements require separate spacing rules and are not part
            // of this provider's contract.
            format!("{before}{children}{after}")
        };
        if !flat(&content).is_empty() {
            return Some(content);
        }
        if !suppress_title {
            return attr(self.dom, node, "title").map(str::to_owned);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::LayoutDomMut;

    fn alternatives(html: &str) -> (Option<String>, Option<String>) {
        let mut dom = ScriptedDom::new();
        dom.set_inner_html(dom.document(), html);
        let names = Names::new(&dom);
        let node = names
            .element_by_id(dom.document(), "target")
            .expect("target");
        (names.name(node), names.description(node))
    }

    #[test]
    fn ordered_references_outrank_aria_label_and_skip_duplicates_and_missing_ids() {
        assert_eq!(alternatives("<span id=a>First</span><span id=b>Second</span><button id=target aria-labelledby='b missing a b' aria-label=Fallback>Contents</button>").0.as_deref(), Some("Second First"));
    }

    #[test]
    fn duplicate_ids_use_first_in_tree_order_and_preserve_nonbreaking_spaces() {
        assert_eq!(alternatives("<span id=label>A&nbsp;B</span><span id=label>Wrong</span><input id=target aria-labelledby=label>").0.as_deref(), Some("A\u{a0}B"));
    }

    #[test]
    fn valid_empty_reference_blocks_fallback_but_invalid_reference_does_not() {
        assert_eq!(alternatives("<span id=empty></span><button id=target aria-labelledby=empty aria-label=Fallback></button>").0.as_deref(), Some(""));
        assert_eq!(
            alternatives("<button id=target aria-labelledby=missing aria-label=Fallback></button>")
                .0
                .as_deref(),
            Some("Fallback")
        );
    }

    #[test]
    fn self_reference_and_reference_cycles_terminate_without_chaining() {
        assert_eq!(alternatives("<button id=target aria-labelledby='target other' aria-label=Save></button><span id=other aria-labelledby=target>document</span>").0.as_deref(), Some("Save document"));
        assert_eq!(alternatives("<button id=target aria-labelledby=other>One</button><span id=other aria-labelledby=target>Two</span>").0.as_deref(), Some("Two"));
    }

    #[test]
    fn labels_use_tree_order_and_exclude_the_named_control_value() {
        assert_eq!(alternatives("<label for=target>First <b>name</b></label><input id=target value=wrong><label for=target>Second</label>").0.as_deref(), Some("First name Second"));
        assert_eq!(
            alternatives("<label>Message<textarea id=target>Wrong value</textarea></label>")
                .0
                .as_deref(),
            Some("Message")
        );
    }

    #[test]
    fn explicit_for_prevents_accidental_wrapping_association() {
        assert_eq!(
            alternatives("<label for=elsewhere>Wrong<input id=target></label><input id=elsewhere>")
                .0,
            None
        );
        assert_eq!(
            alternatives("<label>First<input id=first><input id=target></label>").0,
            None
        );
    }

    #[test]
    fn image_alternatives_preserve_empty_alt_and_nested_button_text() {
        assert_eq!(
            alternatives("<img id=target alt='' title=Wrong>")
                .0
                .as_deref(),
            Some("")
        );
        assert_eq!(
            alternatives("<button id=target><img alt=Save> <span>document</span></button>")
                .0
                .as_deref(),
            Some("Save document")
        );
    }

    #[test]
    fn descriptions_obey_precedence_and_do_not_duplicate_title_names() {
        assert_eq!(alternatives("<span id=help>Use <b>carefully</b></span><input id=target aria-describedby=help aria-description=Wrong title=Tooltip>").1.as_deref(), Some("Use carefully"));
        assert_eq!(alternatives("<span id=help></span><input id=target aria-describedby=help aria-description=Wrong>").1.as_deref(), Some(""));
        assert_eq!(
            alternatives("<input id=target aria-description='' title=Tooltip>")
                .1
                .as_deref(),
            Some("")
        );
        assert_eq!(
            alternatives("<input id=target title=Tooltip>"),
            (Some("Tooltip".into()), None)
        );
        assert_eq!(
            alternatives("<input id=target aria-label=Name title=Tooltip>"),
            (Some("Name".into()), Some("Tooltip".into()))
        );
    }

    #[test]
    fn hidden_content_is_excluded_unless_directly_referenced() {
        assert_eq!(alternatives("<span id=visible>Label<span hidden>Wrong</span></span><button id=target aria-labelledby=visible></button>").0.as_deref(), Some("Label"));
        assert_eq!(
            alternatives("<button id=target>Visible<span hidden>Wrong</span></button>")
                .0
                .as_deref(),
            Some("Visible")
        );
        assert_eq!(alternatives("<span id=hidden hidden>Hidden label</span><button id=target aria-labelledby=hidden></button>").0.as_deref(), Some("Hidden label"));
    }

    #[test]
    fn button_inputs_are_named_by_value_alt_title_then_a_default() {
        let name = |html: &str| alternatives(html).0;
        // Control: a native button already names itself from content.
        assert_eq!(name("<button id=target>Save</button>").as_deref(), Some("Save"));
        for kind in ["button", "submit", "reset"] {
            let got = name(&format!("<input id=target type={kind} value='{kind} label'>"));
            assert_eq!(got, Some(format!("{kind} label")));
        }
        assert_eq!(name("<input id=target type=IMAGE alt='image label' value=Wrong>").as_deref(), Some("image label"));
        // Fallbacks: image alt, then value, then title, then the default.
        assert_eq!(name("<input id=target type=image alt='' value=Value title=Tip>").as_deref(), Some("Value"));
        assert_eq!(name("<input id=target type=image title=Tip>").as_deref(), Some("Tip"));
        assert_eq!(name("<input id=target type=image>").as_deref(), Some("Submit Query"));
        assert_eq!(name("<input id=target type=submit>").as_deref(), Some("Submit"));
        assert_eq!(name("<input id=target type=reset value=''>").as_deref(), Some("Reset"));
        assert_eq!(name("<input id=target type=reset title=Tip>").as_deref(), Some("Tip"));
        // A plain button input has no default.
        assert_eq!(name("<input id=target type=button>").as_deref(), Some(""));
    }

    #[test]
    fn button_input_names_yield_to_aria_and_labels_and_title_stays_a_description() {
        let both = |html| alternatives(html);
        assert_eq!(both("<input id=target type=submit value=V aria-label=Aria>").0.as_deref(), Some("Aria"));
        assert_eq!(both("<span id=l>Ref</span><input id=target type=submit value=V aria-labelledby=l aria-label=Aria>").0.as_deref(), Some("Ref"));
        assert_eq!(both("<label for=target>Label</label><input id=target type=submit value=V>").0.as_deref(), Some("Label"));
        let (name, description) = both("<input id=target type=submit value=V title=Tip>");
        assert_eq!((name.as_deref(), description.as_deref()), (Some("V"), Some("Tip")));
        // Title that names the control is not repeated as its description.
        assert_eq!(both("<input id=target type=reset title=Tip>").1, None);
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn deliberately_wrong_button_input_expectation_fails() {
        assert_eq!(alternatives("<input id=target type=submit value=Real>").0.as_deref(), Some("Wrong"));
    }
}
