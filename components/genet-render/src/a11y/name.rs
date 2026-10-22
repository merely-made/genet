/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Bounded DOM text alternatives, following AccName 1.2 (2026-09-23).
//! CSS rendering counts as hidden only when the style owner supplies it
//! (`with_rendered`). Name from content follows the flat tree (AccName's
//! "rendered child nodes"); ID references resolve only within the referring
//! node's own tree. Embedded controls contribute current values during labels.
//! A node that is neither referenced nor part of a content traversal takes
//! no name from content when its role bars it (see [`Names::content_barred`]).

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
        self.dom.element_name(node).map(|name| name.local.as_str())
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

    /// Whether `node`'s role forbids name from content, with whether it takes
    /// the first heading instead. `alertdialog` is read from the `role`
    /// attribute here: the neutral role enum does not model it.
    fn content_barred(&self, node: D::NodeId) -> Option<bool> {
        use document_session_api::DocumentA11yRole as Role;
        if attr(self.dom, node, "role")
            .is_some_and(|r| r.trim().eq_ignore_ascii_case("alertdialog"))
        {
            return Some(true);
        }
        match super::document_role(self.dom, node) {
            Role::Dialog | Role::Article => Some(true),
            Role::Group | Role::Table => Some(false),
            _ => None,
        }
    }

    /// The first heading below `node` in flat-tree document order (depth
    /// first, any depth), skipping hidden subtrees.
    fn first_heading(&self, node: D::NodeId) -> Option<D::NodeId> {
        for child in self.dom.flat_children(node) {
            if self.dom.kind(child) != NodeKind::Element || self.hidden(child) {
                continue;
            }
            if matches!(
                super::document_role(self.dom, child),
                document_session_api::DocumentA11yRole::Heading { .. }
            ) {
                return Some(child);
            }
            if let Some(heading) = self.first_heading(child) {
                return Some(heading);
            }
        }
        None
    }

    /// A table's first `caption` child.
    fn caption(&self, table: D::NodeId) -> Option<D::NodeId> {
        self.dom
            .dom_children(table)
            .find(|child| self.tag(*child) == Some("caption"))
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
        self.compute(node, false, false, false, false, &mut HashSet::new())
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
                            false,
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
            .compute(node, false, true, false, false, &mut HashSet::new())
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

    /// Chosen options belong to this control, not to a nested independent
    /// control. Native selectedness comes from the DOM owner when available.
    fn chosen_options(
        &self,
        node: D::NodeId,
        native: bool,
        combobox: bool,
        out: &mut Vec<D::NodeId>,
    ) -> bool {
        use document_session_api::DocumentA11yRole as Role;
        let mut has_options = false;
        for child in self.dom.flat_children(node) {
            if native {
                if self.tag(child) == Some("option") {
                    has_options = true;
                    let selected = self.dom.option_selected_state(child).map_or_else(
                        || attr(self.dom, child, "selected").is_some(),
                        |state| state.selected,
                    );
                    if selected {
                        out.push(child);
                    }
                } else if self.tag(child) == Some("optgroup") {
                    has_options |= self.chosen_options(child, true, false, out);
                }
            } else {
                match super::document_role(self.dom, child) {
                    Role::ListBoxOption => {
                        has_options = true;
                        if attr(self.dom, child, "aria-selected")
                            .is_some_and(|v| v.eq_ignore_ascii_case("true"))
                        {
                            out.push(child);
                        }
                    },
                    Role::ComboBox => {},
                    Role::ListBox if !combobox => {},
                    _ => has_options |= self.chosen_options(child, false, combobox, out),
                }
            }
        }
        has_options
    }

    fn embedded_value(
        &self,
        node: D::NodeId,
        include_hidden: bool,
        visited: &mut HashSet<D::NodeId>,
    ) -> Option<String> {
        use document_session_api::DocumentA11yRole as Role;
        let native = self
            .dom
            .element_name(node)
            .is_some_and(|name| name.ns.as_str() == "http://www.w3.org/1999/xhtml");
        let mut role = super::document_role(self.dom, node);
        // Native select/range roles are not yet represented by document_role.
        // Respect effective overrides, not empty or unrecognized role text.
        if native {
            match self.tag(node) {
                Some("select") if role == Role::Unknown => role = Role::ListBox,
                Some("input")
                    if role == Role::TextField
                        && !attr(self.dom, node, "role")
                            .is_some_and(|v| v.trim().eq_ignore_ascii_case("textbox"))
                        && attr(self.dom, node, "type").is_some_and(|v| {
                            v.eq_ignore_ascii_case("range") || v.eq_ignore_ascii_case("number")
                        }) =>
                {
                    role = Role::Slider
                },
                _ => {},
            }
        }
        match role {
            Role::TextField | Role::ComboBox => {
                if let Some(value) = super::text_control_value(self.dom, node) {
                    return Some(value);
                }
                if role == Role::TextField {
                    // Other native input types are not textboxes, despite the
                    // coarse neutral role. An ARIA textbox uses its text value.
                    return (self.tag(node) != Some("input"))
                        .then(|| super::descendant_text(self.dom, node));
                }
            },
            Role::Slider | Role::SpinButton => {
                return attr(self.dom, node, "aria-valuetext")
                    .or_else(|| attr(self.dom, node, "aria-valuenow"))
                    .map(str::to_owned)
                    .or_else(|| self.dom.form_control_state(node).map(|state| state.value))
                    .or_else(|| attr(self.dom, node, "value").map(str::to_owned))
                    .or_else(|| Some(String::new()));
            },
            Role::ListBox => {},
            _ => return None,
        }
        let native_select = native && self.tag(node) == Some("select");
        let mut options = Vec::new();
        let has_options =
            self.chosen_options(node, native_select, role == Role::ComboBox, &mut options);
        if !native_select && !has_options && role == Role::ComboBox {
            return Some(super::descendant_text(self.dom, node));
        }
        Some(
            options
                .into_iter()
                .map(|option| {
                    self.compute(option, false, false, include_hidden, true, visited)
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
                .join(" "),
        )
    }

    fn compute(
        &self,
        node: D::NodeId,
        referenced: bool,
        suppress_title: bool,
        include_hidden: bool,
        descendant: bool,
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
        let result = self.compute_inner(
            node,
            referenced,
            suppress_title,
            include_hidden,
            descendant,
            referenced || descendant,
            visited,
        );
        visited.remove(&node);
        result
    }

    fn compute_inner(
        &self,
        node: D::NodeId,
        referenced: bool,
        suppress_title: bool,
        include_hidden: bool,
        descendant: bool,
        embedded: bool,
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
                                    false,
                                    embedded,
                                    visited,
                                )
                            } else {
                                self.compute(
                                    target,
                                    true,
                                    false,
                                    self.hidden(target),
                                    false,
                                    visited,
                                )
                            }
                            .unwrap_or_default()
                        })
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
        if embedded {
            if let Some(value) = self.embedded_value(node, include_hidden, visited) {
                return Some(value);
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
                            self.compute(label, true, false, self.hidden(label), false, visited)
                                .unwrap_or_default()
                        })
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
        // The role-gated sources apply to the node whose name is being asked
        // for. A node reached through a reference or inside a content
        // traversal contributes its text whatever its role (AccName 2F).
        let root = !referenced && !descendant;
        let barred = root.then(|| self.content_barred(node)).flatten();
        if root && self.tag(node) == Some("table") {
            if let Some(caption) = self.caption(node) {
                let text = self
                    .compute(caption, true, false, false, false, visited)
                    .unwrap_or_default();
                if !flat(&text).is_empty() {
                    return Some(text);
                }
            }
        }
        if barred == Some(true) {
            if let Some(heading) = self.first_heading(node) {
                let text = self
                    .compute(heading, false, false, false, false, visited)
                    .unwrap_or_default();
                if !flat(&text).is_empty() {
                    return Some(text);
                }
            }
        }
        if self.tag(node) == Some("img") {
            if let Some(alt) = attr(self.dom, node, "alt") {
                return Some(alt.to_owned());
            }
        }
        if self.tag(node) == Some("option") {
            if let Some(label) = attr(self.dom, node, "label").filter(|s| !flat(s).is_empty()) {
                return Some(label.to_owned());
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
        // An abbreviation's `title` is its name; its content is the fallback.
        if root && self.tag(node) == Some("abbr") {
            if let Some(title) = attr(self.dom, node, "title").filter(|s| !flat(s).is_empty()) {
                return (!suppress_title).then(|| title.to_owned());
            }
        }
        // Native text fields get names from labels, never their editable value.
        let content = if barred.is_some()
            || matches!(self.tag(node), Some("input" | "textarea" | "select"))
            || super::document_role(self.dom, node)
                == document_session_api::DocumentA11yRole::TextField
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
                    self.compute(child, referenced, false, include_hidden, true, visited)
                        .unwrap_or_default()
                })
                .collect::<String>();
            // The provider supplies admitted rendered inline text. Block
            // pseudo-elements require separate spacing rules and are not part
            // of this provider's contract.
            format!("{before}{children}{after}")
        };
        // A whitespace-only descendant may separate its parent's words.
        // Keep that raw content through recursion; the public name and
        // description entry points flatten it. Root tooltip fallback stays
        // available when the complete content is only whitespace.
        if (descendant && !content.is_empty()) || !flat(&content).is_empty() {
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
    fn embedded_text_controls_contribute_values_before_their_own_labels() {
        for control in [
            "<input value=3 aria-label='number of times'>",
            "<textarea aria-label='number of times'>3</textarea>",
            "<span role=textbox contenteditable aria-label='number of times'>3</span>",
            "<input role=combobox value=3 aria-label='number of times'>",
            "<span role=combobox aria-label='number of times'>3</span>",
        ] {
            assert_eq!(
                alternatives(&format!(
                    "<label><input id=target type=checkbox>Flash {control} times</label>"
                ))
                .0
                .as_deref(),
                Some("Flash 3 times"),
                "{control}"
            );
        }
    }

    #[test]
    fn embedded_ranges_use_value_text_then_value_now_then_native_value() {
        for (control, expected) in [
            (
                "<span role=slider aria-label=Count aria-valuetext=three aria-valuenow=3>wrong</span>",
                "Flash three times",
            ),
            (
                "<span role=spinbutton aria-label=Count aria-valuenow=0>wrong</span>",
                "Flash 0 times",
            ),
            (
                "<span role=slider aria-label=Count aria-valuetext='' aria-valuenow=3>wrong</span>",
                "Flash times",
            ),
            (
                "<input type=range min=1 max=5 value=3 aria-label=Count>",
                "Flash 3 times",
            ),
            (
                "<input type=number value=3 aria-label=Count>",
                "Flash 3 times",
            ),
        ] {
            assert_eq!(
                alternatives(&format!(
                    "<label><input id=target type=checkbox>Flash {control} times</label>"
                ))
                .0
                .as_deref(),
                Some(expected),
                "{control}"
            );
        }
    }

    #[test]
    fn embedded_choices_use_selected_option_names_in_tree_order() {
        for control in [
            "<select aria-label=Count><option>wrong</option><optgroup label=Group><option selected value=stored label=three>wrong</option></optgroup></select>",
            "<div role=listbox aria-label=Count><span role=option>wrong</span><span role=option aria-selected=true aria-label=three>wrong</span></div>",
            "<div role=combobox aria-label=Count><div role=listbox><span role=option aria-selected=true>three</span></div></div>",
        ] {
            assert_eq!(
                alternatives(&format!(
                    "<label><input id=target type=checkbox>Flash {control} times</label>"
                ))
                .0
                .as_deref(),
                Some("Flash three times"),
                "{control}"
            );
        }
        assert_eq!(alternatives("<label><input id=target type=checkbox>Choose <select multiple aria-label=Options><option selected>red</option><option>wrong</option><option selected>blue</option></select> now</label>").0.as_deref(), Some("Choose red blue now"));
    }

    #[test]
    fn embedded_values_follow_live_state_without_becoming_the_controls_own_names() {
        let mut dom = ScriptedDom::new();
        dom.set_inner_html(dom.document(), "<label><input id=target type=checkbox>Flash <input id=value value=old aria-label=Count> times</label><button id=other aria-labelledby=value></button>");
        let (target, value, other) = {
            let names = Names::new(&dom);
            (
                names.element_by_id(dom.document(), "target").unwrap(),
                names.element_by_id(dom.document(), "value").unwrap(),
                names.element_by_id(dom.document(), "other").unwrap(),
            )
        };
        dom.set_form_control_value(value, "7").unwrap();
        let names = Names::new(&dom);
        assert_eq!(names.name(target).as_deref(), Some("Flash 7 times"));
        assert_eq!(names.name(other).as_deref(), Some("7"));
        assert_eq!(names.name(value).as_deref(), Some("Count"));
        dom.set_form_control_value(value, "").unwrap();
        assert_eq!(
            Names::new(&dom).name(target).as_deref(),
            Some("Flash times")
        );
    }

    #[test]
    fn embedded_select_uses_current_selection_instead_of_authored_selectedness() {
        let mut dom = ScriptedDom::new();
        dom.set_inner_html(dom.document(), "<label><input id=target type=checkbox>Choose <select id=choice aria-label=Options><option selected value=old>red</option><option value=new>blue</option></select> now</label>");
        let names = Names::new(&dom);
        let target = names.element_by_id(dom.document(), "target").unwrap();
        let choice = names.element_by_id(dom.document(), "choice").unwrap();
        assert_eq!(names.name(target).as_deref(), Some("Choose red now"));
        dom.set_form_control_value(choice, "new").unwrap();
        assert_eq!(
            Names::new(&dom).name(target).as_deref(),
            Some("Choose blue now")
        );
    }

    #[test]
    fn embedded_self_reference_keeps_value_eligibility_but_root_does_not() {
        assert_eq!(alternatives("<button id=target>Flash <input id=n aria-labelledby=n aria-label=Count value=3> times</button>").0.as_deref(), Some("Flash 3 times"));
        assert_eq!(
            alternatives("<input id=target aria-labelledby=target aria-label=Count value=3>")
                .0
                .as_deref(),
            Some("Count")
        );
    }

    #[test]
    fn embedded_native_values_survive_ineffective_role_attributes() {
        for role in ["", "unknown"] {
            for control in [
                format!("<input type=range role='{role}' value=3 aria-label=Count>"),
                format!(
                    "<select role='{role}' aria-label=Count><option selected>3</option></select>"
                ),
            ] {
                assert_eq!(
                    alternatives(&format!(
                        "<label><input id=target type=checkbox>Flash {control} times</label>"
                    ))
                    .0
                    .as_deref(),
                    Some("Flash 3 times")
                );
            }
        }
        assert_eq!(alternatives("<label><input id=target type=checkbox>Flash <select role=button aria-label=Count><option selected>3</option></select> times</label>").0.as_deref(), Some("Flash Count times"));
    }

    #[test]
    fn embedded_combobox_without_a_choice_does_not_read_all_options() {
        let aria = alternatives("<label><input id=target type=checkbox>Flash <div role=combobox aria-label=Count><div role=listbox><span role=option>wrong</span></div></div> times</label>").0;
        let mut dom = ScriptedDom::new();
        dom.set_inner_html(dom.document(), "<label><input id=target type=checkbox>Flash <select id=choice role=combobox aria-label=Count><option value=old>wrong</option></select> times</label>");
        let names = Names::new(&dom);
        let target = names.element_by_id(dom.document(), "target").unwrap();
        let choice = names.element_by_id(dom.document(), "choice").unwrap();
        dom.set_form_control_value(choice, "missing").unwrap();
        let native = Names::new(&dom).name(target);
        assert_eq!(
            [aria, native],
            [
                Some("Flash times".to_owned()),
                Some("Flash times".to_owned())
            ]
        );
    }

    #[test]
    fn nested_whitespace_separates_content_words() {
        for (element, word) in [
            ("span role=button", "button"),
            ("div role=heading", "heading"),
            ("button", "button"),
            ("h3", "heading"),
            ("a href='#'", "link"),
        ] {
            let tag = element.split_ascii_whitespace().next().unwrap();
            for separator in [" ", "\t", "\r\n"] {
                let html = format!(
                    "<{element} id=target>{word}<span><span><span><span><span><span><span>\
                     {separator}</span></span></span></span></span></span></span>label</{tag}>"
                );
                assert_eq!(
                    alternatives(&html).0,
                    Some(format!("{word} label")),
                    "{element}, separator {separator:?}"
                );
            }
        }
    }

    #[test]
    fn nested_whitespace_separates_referenced_labels_and_descriptions() {
        assert_eq!(
            alternatives(
                "<label for=target>Save<span><span> </span></span>file</label><input id=target>"
            )
            .0
            .as_deref(),
            Some("Save file")
        );
        assert_eq!(
            alternatives("<span id=label hidden>Save<span><span> </span></span>file</span><button id=target aria-labelledby=label></button>")
                .0.as_deref(),
            Some("Save file")
        );
        assert_eq!(
            alternatives("<span id=help>Use<span><span> </span></span>carefully</span><input id=target aria-describedby=help>")
                .1.as_deref(),
            Some("Use carefully")
        );
    }

    #[test]
    fn content_spacing_preserves_empty_hidden_and_nonbreaking_separators() {
        for html in [
            "<button id=target>button<span></span>label</button>",
            "<button id=target>button<!-- no separator -->label</button>",
            "<button id=target>button<span hidden><span> </span></span>label</button>",
        ] {
            assert_eq!(alternatives(html).0.as_deref(), Some("buttonlabel"));
        }
        assert_eq!(
            alternatives(
                "<button id=target>button<span><span>&nbsp;&nbsp;</span></span>label</button>"
            )
            .0
            .as_deref(),
            Some("button\u{a0}\u{a0}label")
        );
        assert_eq!(
            alternatives("<button id=target title=Tooltip><span> </span></button>")
                .0
                .as_deref(),
            Some("Tooltip")
        );
        assert_eq!(
            alternatives(
                "<button id=target>button<span title=Wrong><span> </span></span>label</button>"
            )
            .0
            .as_deref(),
            Some("button label")
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
    fn roles_that_forbid_content_take_no_name_from_it() {
        let name = |html: &str| alternatives(html).0;
        // Controls: roles that allow name from content still take it, and a
        // barred role inside a content traversal still contributes its text.
        assert_eq!(name("<button id=target>Save <b>it</b></button>").as_deref(), Some("Save it"));
        assert_eq!(name("<button id=target>Go <div role=group>there</div></button>").as_deref(), Some("Go there"));
        assert_eq!(name("<div role=group id=target>Text</div>").as_deref(), None);
        assert_eq!(name("<div role=group><div role=group><h3>No</h3></div><h2>Nor</h2>Content</div><div role=group id=target><div role=group>Not this</div></div>").as_deref(), None);
        assert_eq!(name("<div role=group id=target title=Tip>Text</div>").as_deref(), Some("Tip"));
        assert_eq!(name("<div role=group id=target aria-label=Own>Text</div>").as_deref(), Some("Own"));
        // A misspelled labelledby is no labelledby.
        assert_eq!(name("<h2 id=h>first</h2><div role=group id=target aria-labeledby=h><p>inside</p></div>").as_deref(), None);
        // Referenced, the barred node's content does count (AccName 2F).
        assert_eq!(name("<div role=group id=g>Referred text</div><button id=target aria-labelledby=g></button>").as_deref(), Some("Referred text"));
        // Layout-independent roles that stay text carriers are unchanged.
        assert_eq!(name("<p id=target>Para</p>").as_deref(), Some("Para"));
        assert_eq!(name("<div id=target>Plain</div>").as_deref(), Some("Plain"));
        assert_eq!(name("<div role=alert id=target>Warn</div>").as_deref(), Some("Warn"));
    }

    #[test]
    fn dialogs_and_articles_are_named_by_their_first_heading_depth_first() {
        let name = |html: &str| alternatives(html).0;
        for role in ["dialog", "alertdialog", "article"] {
            let html = format!("<div role={role} id=target><h2>{role} heading</h2><p>More</p></div>");
            assert_eq!(name(&html), Some(format!("{role} heading")), "{role}");
        }
        // Depth first in document order: the h3 inside the group precedes the h2.
        assert_eq!(name("<div role=article id=target><div role=group><h3>deep first</h3></div><h2>later</h2></div>").as_deref(), Some("deep first"));
        // Hidden headings are skipped; role=heading counts.
        assert_eq!(name("<div role=dialog id=target><h2 hidden>No</h2><div role=heading>Yes</div></div>").as_deref(), Some("Yes"));
        // Native article and dialog take the heading too.
        assert_eq!(name("<article id=target><h2>Native article</h2><p>text</p></article>").as_deref(), Some("Native article"));
        assert_eq!(name("<dialog open id=target><h2>Native dialog</h2><p>text</p></dialog>").as_deref(), Some("Native dialog"));
        // Order: aria-label first; no heading falls back to title, then to nothing.
        assert_eq!(name("<div role=dialog id=target aria-label=Own><h2>H</h2></div>").as_deref(), Some("Own"));
        assert_eq!(name("<div role=dialog id=target title=Tip><p>no heading</p></div>").as_deref(), Some("Tip"));
        assert_eq!(name("<div role=dialog id=target><p>no heading</p></div>").as_deref(), None);
        // A group never takes a heading.
        assert_eq!(name("<div role=group id=target><h2>H</h2></div>").as_deref(), None);
        // A heading name that names the dialog is not repeated as a description.
        assert_eq!(alternatives("<div role=dialog id=target title=Tip><h2>H</h2></div>"), (Some("H".into()), Some("Tip".into())));
    }

    #[test]
    fn tables_take_their_caption_never_their_cells_and_abbr_prefers_its_title() {
        let name = |html: &str| alternatives(html).0;
        assert_eq!(name("<table id=target><caption>Caption label</caption><tr><th>a</th><td>1</td></tr></table>").as_deref(), Some("Caption label"));
        assert_eq!(name("<table id=target><tr><th>a</th><td>1</td></tr></table>").as_deref(), None);
        assert_eq!(name("<table id=target title=Tip><caption></caption><tr><td>1</td></tr></table>").as_deref(), Some("Tip"));
        assert_eq!(name("<table id=target aria-label=Own><caption>Cap</caption><tr><td>1</td></tr></table>").as_deref(), Some("Own"));
        assert_eq!(name("<div role=table id=target><div role=row><div role=cell>cell</div></div></div>").as_deref(), None);
        // Control: a cell is still named from its content.
        assert_eq!(name("<table><tr><td id=target>cell</td></tr></table>").as_deref(), Some("cell"));
        assert_eq!(name("<abbr id=target title='Web Platform Tests'>WPT</abbr>").as_deref(), Some("Web Platform Tests"));
        assert_eq!(name("<abbr id=target>WPT</abbr>").as_deref(), Some("WPT"));
        assert_eq!(name("<abbr id=target title='' >WPT</abbr>").as_deref(), Some("WPT"));
        assert_eq!(name("<abbr id=target aria-label=Own title=Tip>WPT</abbr>").as_deref(), Some("Own"));
        // Inside a content traversal the abbr contributes its content.
        assert_eq!(name("<button id=target><abbr title='Long form'>WPT</abbr> run</button>").as_deref(), Some("WPT run"));
        // The title that names an abbr is not repeated as its description.
        assert_eq!(alternatives("<abbr id=target title=Tip>WPT</abbr>").1, None);
        // Other generics are unchanged: content first, title only as fallback.
        assert_eq!(name("<span id=target title=Tip>Text</span>").as_deref(), Some("Text"));
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn deliberately_wrong_group_expectation_fails() {
        assert_eq!(alternatives("<div role=group id=target>Text</div>").0.as_deref(), Some("Text"));
    }

    #[test]
    #[should_panic(expected = "assertion `left == right` failed")]
    fn deliberately_wrong_button_input_expectation_fails() {
        assert_eq!(alternatives("<input id=target type=submit value=Real>").0.as_deref(), Some("Wrong"));
    }
}
