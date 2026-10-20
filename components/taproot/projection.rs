// Copyright 2026 the taproot authors.
// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use document_session_api::{DocumentA11yProjection, DocumentA11yRole};

use super::*;

/// Match role/name selectors against the host's current accessibility projection.
///
/// A role selector's `containing` filter searches only the computed accessible
/// name. Missing and hidden projected nodes are authoritative misses, even when
/// raw DOM attributes would match. Class selectors keep [`matching_classes`]'s shallow
/// DOM text/`aria-label` behavior, and attribute filters always inspect the DOM.
/// [`text_present`] is unchanged.
///
/// Results follow DOM order. The host must supply the projection of this same
/// retained DOM, whose semantic IDs equal [`LayoutDom::opaque_id`], and pair the
/// result with current host geometry. This function neither computes names nor
/// creates semantics or geometry for nodes absent from that projection.
pub fn matching_with_projection(
    dom: &ScriptedDom,
    sel: &Selector,
    projection: &DocumentA11yProjection,
) -> Vec<NodeId> {
    let Match::Role(role) = &sel.matcher else {
        return matching_classes(dom, sel);
    };
    let nodes: BTreeMap<_, _> = projection
        .nodes()
        .iter()
        .map(|node| (node.id.get(), node))
        .collect();
    let mut pending = vec![dom.document()];
    let mut result = Vec::new();
    while let Some(node) = pending.pop() {
        if let Some(semantic) = nodes.get(&dom.opaque_id(node))
            && !semantic.state.hidden
            && role_matches(semantic.role, role)
            && sel.text.as_ref().is_none_or(|text| {
                semantic
                    .name
                    .as_ref()
                    .is_some_and(|name| name.contains(text))
            })
            && sel.attr.as_ref().is_none_or(|(name, value)| {
                attr(dom, node, name).is_some_and(|attr| attr.contains(value))
            })
        {
            result.push(node);
        }
        let children: Vec<_> = dom.dom_children(node).collect();
        pending.extend(children.into_iter().rev());
    }
    result
}

/// Prefer an exact owner-computed name among projected role matches.
///
/// This is an opt-in selection policy: [`matching_with_projection`] and the
/// legacy DOM matcher keep their substring and document-order contracts. For a
/// role selector with text, return the exact-name subset if it is nonempty;
/// otherwise retain every original match. Class selectors and roles without
/// text or with empty text are unchanged. All role, hidden-state and attribute filters are applied
/// before this preference. Duplicate exact names retain their document order.
/// The host still supplies geometry and owns click delivery.
pub fn matching_prefer_exact_with_projection(
    dom: &ScriptedDom,
    sel: &Selector,
    projection: &DocumentA11yProjection,
) -> Vec<NodeId> {
    let matched = matching_with_projection(dom, sel, projection);
    let (Match::Role(_), Some(text)) = (&sel.matcher, &sel.text) else {
        return matched;
    };
    if text.is_empty() {
        return matched;
    }
    let exact_names: BTreeMap<_, _> = projection
        .nodes()
        .iter()
        .filter(|node| node.name.as_deref() == Some(text.as_str()))
        .map(|node| (node.id.get(), ()))
        .collect();
    let exact: Vec<_> = matched
        .iter()
        .copied()
        .filter(|node| exact_names.contains_key(&dom.opaque_id(*node)))
        .collect();
    if exact.is_empty() { matched } else { exact }
}

// This is a spelling adapter for the neutral role vocabulary, not DOM role
// inference. In particular Unknown must not resurrect an unprojected ARIA role.
fn role_matches(role: DocumentA11yRole, requested: &str) -> bool {
    use DocumentA11yRole::*;
    let token = match role {
        Window => "window",
        Document => "document",
        Article => "article",
        Region => "region",
        Group => "group",
        Navigation => "navigation",
        Main => "main",
        Heading { .. } => "heading",
        Paragraph => "paragraph",
        StaticText => "text",
        Link => "link",
        Button => "button",
        TextField => "textbox",
        CheckBox => "checkbox",
        RadioButton => "radio",
        RadioGroup => "radiogroup",
        Switch => "switch",
        ComboBox => "combobox",
        List => "list",
        ListItem => "listitem",
        ListBox => "listbox",
        ListBoxOption => "option",
        Table => "table",
        Row => "row",
        Cell => return matches!(requested, "cell" | "gridcell"),
        Image => return matches!(requested, "img" | "image"),
        Form => "form",
        Dialog => "dialog",
        Alert => "alert",
        Menu => "menu",
        MenuBar => "menubar",
        MenuItem => "menuitem",
        MenuItemCheckBox => "menuitemcheckbox",
        MenuItemRadio => "menuitemradio",
        TabList => "tablist",
        Tab => "tab",
        TabPanel => "tabpanel",
        Tree => "tree",
        TreeItem => "treeitem",
        Slider => "slider",
        SpinButton => "spinbutton",
        Splitter => "separator",
        Toolbar => "toolbar",
        ProgressIndicator => "progressbar",
        Label => "label",
        Status => "status",
        Log => "log",
        Note => "note",
        Unknown => return false,
    };
    requested == token
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout_dom_api::{LayoutDomMut, QualName};

    fn element(
        dom: &mut ScriptedDom,
        parent: NodeId,
        tag: &str,
        attrs: &[(&str, &str)],
        text: &str,
    ) -> NodeId {
        let qual = |name: &str| QualName::new(None, Namespace::from(""), LocalName::from(name));
        let node = dom.create_element(qual(tag));
        for (name, value) in attrs {
            dom.set_attribute(node, qual(name), value);
        }
        dom.append_child(parent, node);
        if !text.is_empty() {
            let child = dom.create_text(text);
            dom.append_child(node, child);
        }
        node
    }

    fn project(dom: &ScriptedDom) -> DocumentA11yProjection {
        let styles = resolve_styles(
            dom,
            &StyleSet::cambium(&[""]),
            &Device::screen(600.0, 600.0),
            &InteractionStates::default(),
        );
        let fragments = layout(dom, &styles, 600.0, 600.0).unwrap();
        genet_render::document_a11y_projection(dom, &fragments, None, 1)
    }

    #[test]
    fn role_names_follow_referenced_native_and_nested_owner_labels() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        element(
            &mut dom,
            root,
            "span",
            &[("id", "label")],
            "Referenced name",
        );
        let referenced = element(
            &mut dom,
            root,
            "button",
            &[
                ("aria-labelledby", "label"),
                ("aria-label", "Raw label"),
                ("class", "control"),
            ],
            "Raw text",
        );
        element(&mut dom, root, "label", &[("for", "field")], "Native name");
        let native = element(
            &mut dom,
            root,
            "input",
            &[("id", "field"), ("data-key", "native")],
            "",
        );
        let nested = element(&mut dom, root, "button", &[("class", "nested")], "");
        element(&mut dom, nested, "span", &[], "Nested name");
        let wrapping = element(&mut dom, root, "label", &[], "Wrapping name");
        let wrapped = element(&mut dom, wrapping, "input", &[], "");
        let projection = project(&dom);
        for (role, name, expected) in [
            ("button", "Referenced name", referenced),
            ("textbox", "Native name", native),
            ("button", "Nested name", nested),
            ("textbox", "Wrapping name", wrapped),
        ] {
            assert_eq!(
                matching_with_projection(&dom, &Selector::role(role).containing(name), &projection),
                [expected]
            );
        }
        for raw in ["Raw label", "Raw text"] {
            assert!(
                matching_with_projection(
                    &dom,
                    &Selector::role("button").containing(raw),
                    &projection
                )
                .is_empty()
            );
            assert_eq!(
                matching_with_projection(
                    &dom,
                    &Selector::class("control").containing(raw),
                    &projection
                ),
                [referenced]
            );
        }
        assert!(
            matching_with_projection(
                &dom,
                &Selector::class("nested").containing("Nested name"),
                &projection
            )
            .is_empty()
        );
        assert!(text_present(
            &[ProbeSurface {
                name: "test",
                dom: &dom,
                rect: [0.0, 0.0, 600.0, 600.0],
                sheet: ""
            }],
            "Nested name"
        ));
        assert_eq!(
            matching_with_projection(
                &dom,
                &Selector::role("textbox").with_attr("data-key", "native"),
                &projection
            ),
            [native]
        );
        assert!(
            matching_with_projection(
                &dom,
                &Selector::role("textbox")
                    .containing("Native name")
                    .with_attr("data-key", "missing"),
                &projection
            )
            .is_empty()
        );
    }

    #[test]
    fn role_override_native_heading_link_and_aliases_follow_projection() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let checkbox = element(&mut dom, root, "button", &[("role", "checkbox")], "Toggle");
        let heading = element(&mut dom, root, "h2", &[], "Section");
        let link = element(&mut dom, root, "a", &[("href", "#target")], "Go");
        let image = element(&mut dom, root, "img", &[("alt", "Picture")], "");
        let projection = project(&dom);
        for (role, expected) in [
            ("checkbox", checkbox),
            ("heading", heading),
            ("link", link),
            ("image", image),
            ("img", image),
        ] {
            assert_eq!(
                matching_with_projection(&dom, &Selector::role(role), &projection),
                [expected]
            );
        }
        assert!(matching_with_projection(&dom, &Selector::role("button"), &projection).is_empty());
        assert_eq!(
            matching_estimated_fixture(&dom, &Selector::role("button")),
            [checkbox],
            "legacy DOM policy remains explicit"
        );
    }

    #[test]
    fn missing_hidden_and_unknown_semantics_never_fall_back_to_dom() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let visible = element(&mut dom, root, "button", &[("class", "raw")], "Visible");
        element(
            &mut dom,
            root,
            "button",
            &[("aria-hidden", "true")],
            "Hidden",
        );
        element(
            &mut dom,
            root,
            "button",
            &[("style", "display:none")],
            "Not laid out",
        );
        element(
            &mut dom,
            root,
            "div",
            &[("role", "unrecognized")],
            "Unknown",
        );
        let nameless = element(&mut dom, root, "input", &[("value", "Secret value")], "");
        let projection = project(&dom);
        assert_eq!(
            matching_with_projection(&dom, &Selector::role("button"), &projection),
            [visible]
        );
        assert!(
            matching_with_projection(&dom, &Selector::role("unrecognized"), &projection).is_empty()
        );
        assert_eq!(
            matching_with_projection(&dom, &Selector::role("textbox"), &projection),
            [nameless]
        );
        assert!(
            matching_with_projection(
                &dom,
                &Selector::role("textbox").containing("Secret value"),
                &projection
            )
            .is_empty()
        );
        let empty =
            DocumentA11yProjection::new(2, projection.support().clone(), projection.root(), vec![]);
        assert!(matching_with_projection(&dom, &Selector::role("button"), &empty).is_empty());
        assert_eq!(
            matching_with_projection(&dom, &Selector::class("raw").containing("Visible"), &empty),
            [visible]
        );
        let mut nodes = projection.nodes().to_vec();
        nodes
            .iter_mut()
            .find(|n| n.id.get() == dom.opaque_id(visible))
            .unwrap()
            .state
            .hidden = true;
        let hidden =
            DocumentA11yProjection::new(3, projection.support().clone(), projection.root(), nodes);
        assert!(matching_with_projection(&dom, &Selector::role("button"), &hidden).is_empty());
    }

    #[test]
    fn generated_names_are_supplied_by_the_owner_without_dom_reconstruction() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let button = element(&mut dom, root, "button", &[], "Save");
        let styles = resolve_styles(
            &dom,
            &StyleSet::cambium(&[""]),
            &Device::screen(600.0, 600.0),
            &InteractionStates::default(),
        );
        let fragments = layout(&dom, &styles, 600.0, 600.0).unwrap();
        let projection = genet_render::document_a11y_projection_with_generated_text(
            &dom,
            &fragments,
            None,
            4,
            None,
            &|node| {
                if node == button {
                    ("[".into(), "]".into())
                } else {
                    (String::new(), String::new())
                }
            },
        );
        assert_eq!(
            matching_with_projection(
                &dom,
                &Selector::role("button").containing("[Save]"),
                &projection
            ),
            [button]
        );
        assert!(matching_estimated_fixture(&dom, &Selector::role("button").containing("[Save]")).is_empty());
    }

    #[test]
    fn exact_projected_role_name_wins_over_an_earlier_prefix_collision() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let workbench = element(&mut dom, root, "button", &[], "Open selected in workbench");
        let detail = element(&mut dom, root, "button", &[], "Open selected");
        let projection = project(&dom);
        let selector = Selector::role("button").containing("Open selected");
        assert_eq!(
            matching_with_projection(&dom, &selector, &projection),
            [workbench, detail]
        );
        assert_eq!(
            matching_prefer_exact_with_projection(&dom, &selector, &projection),
            [detail]
        );
        assert_eq!(matching_estimated_fixture(&dom, &selector), [workbench, detail]);
    }

    #[test]
    fn exact_name_preference_preserves_partial_class_unlabelled_and_duplicate_order() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let first = element(
            &mut dom,
            root,
            "button",
            &[("class", "open")],
            "Open selected",
        );
        let second = element(
            &mut dom,
            root,
            "button",
            &[("class", "open")],
            "Open selected",
        );
        let projection = project(&dom);
        for selector in [
            Selector::role("button").containing("Open selected"),
            Selector::role("button").containing("Open"),
            Selector::role("button"),
            Selector::class("open").containing("Open"),
        ] {
            assert_eq!(
                matching_prefer_exact_with_projection(&dom, &selector, &projection),
                [first, second]
            );
        }
        assert!(
            matching_prefer_exact_with_projection(
                &dom,
                &Selector::role("button").containing("Missing"),
                &projection
            )
            .is_empty()
        );
        // An empty filter retains the legacy vector even if an owner supplies
        // an empty computed name for one of the controls.
        let mut nodes = projection.nodes().to_vec();
        nodes
            .iter_mut()
            .find(|node| node.id.get() == dom.opaque_id(first))
            .unwrap()
            .name = Some(String::new());
        let empty_name = DocumentA11yProjection::new(
            5,
            projection.support().clone(),
            projection.root(),
            nodes,
        );
        assert_eq!(
            matching_prefer_exact_with_projection(
                &dom,
                &Selector::role("button").containing(""),
                &empty_name,
            ),
            [first, second]
        );
    }

    #[test]
    fn exact_name_preference_uses_owner_names_and_keeps_attribute_filters() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let prefix = element(
            &mut dom,
            root,
            "button",
            &[("data-key", "prefix")],
            "Open selected in workbench",
        );
        element(
            &mut dom,
            root,
            "span",
            &[("id", "detail-name")],
            "Open selected",
        );
        let exact = element(
            &mut dom,
            root,
            "button",
            &[
                ("aria-labelledby", "detail-name"),
                ("aria-label", "Wrong raw name"),
                ("data-key", "exact"),
            ],
            "Wrong raw text",
        );
        let projection = project(&dom);
        let selector = Selector::role("button").containing("Open selected");
        assert_eq!(
            matching_prefer_exact_with_projection(&dom, &selector, &projection),
            [exact]
        );
        assert_eq!(
            matching_prefer_exact_with_projection(
                &dom,
                &selector.clone().with_attr("data-key", "prefix"),
                &projection
            ),
            [prefix]
        );
        assert!(
            matching_prefer_exact_with_projection(
                &dom,
                &Selector::role("button").containing("Wrong raw name"),
                &projection
            )
            .is_empty()
        );
    }

    #[test]
    fn exact_name_preference_never_resurrects_missing_or_hidden_owner_nodes() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let prefix = element(&mut dom, root, "button", &[], "Open selected in workbench");
        let exact = element(&mut dom, root, "button", &[], "Open selected");
        let projection = project(&dom);
        let selector = Selector::role("button").containing("Open selected");
        let empty =
            DocumentA11yProjection::new(2, projection.support().clone(), projection.root(), vec![]);
        assert!(matching_prefer_exact_with_projection(&dom, &selector, &empty).is_empty());
        let mut nodes = projection.nodes().to_vec();
        nodes
            .iter_mut()
            .find(|node| node.id.get() == dom.opaque_id(exact))
            .unwrap()
            .state
            .hidden = true;
        let hidden =
            DocumentA11yProjection::new(3, projection.support().clone(), projection.root(), nodes);
        assert_eq!(
            matching_prefer_exact_with_projection(&dom, &selector, &hidden),
            [prefix]
        );
    }

    #[test]
    fn a_held_partial_target_stops_matching_when_an_exact_target_appears() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let held = element(&mut dom, root, "button", &[], "Open selected in workbench");
        let selector = Selector::role("button").containing("Open selected");
        let before = project(&dom);
        assert!(matching_prefer_exact_with_projection(&dom, &selector, &before).contains(&held));
        let exact = element(&mut dom, root, "button", &[], "Open selected");
        let after = project(&dom);
        assert!(matching_with_projection(&dom, &selector, &after).contains(&held));
        let preferred = matching_prefer_exact_with_projection(&dom, &selector, &after);
        assert_eq!(preferred, [exact]);
        assert!(!preferred.contains(&held));
    }

    #[test]
    fn exact_name_preference_does_not_cross_requested_roles() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        element(&mut dom, root, "button", &[], "Open selected");
        let field = element(
            &mut dom,
            root,
            "input",
            &[("aria-label", "Open selected notes")],
            "",
        );
        let projection = project(&dom);
        assert_eq!(
            matching_prefer_exact_with_projection(
                &dom,
                &Selector::role("textbox").containing("Open selected"),
                &projection
            ),
            [field]
        );
    }

}
