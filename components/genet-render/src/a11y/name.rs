/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Bounded DOM text alternatives, following AccName 1.2 (2026-09-23).
//! CSS rendering counts as hidden only when the style owner supplies it
//! (`with_rendered`). Flat-tree traversal and embedded control values remain open.

use std::collections::{HashMap, HashSet};

use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

pub(super) struct Names<'a, D: LayoutDom> {
    dom: &'a D,
    ids: HashMap<String, D::NodeId>,
    labels: Vec<D::NodeId>,
    generated: Option<&'a dyn Fn(D::NodeId) -> (String, String)>,
    rendered: Option<&'a dyn Fn(D::NodeId) -> bool>,
}

fn attr<'a, D: LayoutDom>(dom: &'a D, node: D::NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::default(), &LocalName::from(name))
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
        let mut pending = vec![dom.document()];
        while let Some(node) = pending.pop() {
            if let Some(id) = attr(dom, node, "id") {
                names.ids.entry(id.to_owned()).or_insert(node);
            }
            if names.tag(node) == Some("label") {
                names.labels.push(node);
            }
            let children: Vec<_> = dom.dom_children(node).collect();
            pending.extend(children.into_iter().rev());
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

    fn references(&self, node: D::NodeId, attribute: &str) -> Vec<D::NodeId> {
        let mut seen = HashSet::new();
        attr(self.dom, node, attribute)
            .unwrap_or("")
            .split_ascii_whitespace()
            .filter_map(|id| self.ids.get(id).copied())
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
                .copied()
                .filter(|label| {
                    if let Some(target) = attr(self.dom, *label, "for") {
                        self.ids.get(target) == Some(&node)
                    } else {
                        self.first_control(*label) == Some(node)
                    }
                })
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
            let children = self
                .dom
                .dom_children(node)
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
        let node = names.ids["target"];
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
}
