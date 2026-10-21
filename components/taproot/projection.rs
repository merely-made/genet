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
/// Results follow flattened DOM order. The host must supply the projection of this same
/// retained DOM, whose semantic IDs equal [`LayoutDom::opaque_id`], and pair the
/// result with current host geometry. This function neither computes names nor
/// creates semantics or geometry for nodes absent from that projection.
pub fn matching_with_projection(
    dom: &ScriptedDom,
    sel: &Selector,
    projection: &DocumentA11yProjection,
) -> Vec<NodeId> {
    matching_with_lookup(dom, sel, &DocumentLookup::new(dom, projection))
        .into_iter()
        .filter_map(|target| match target {
            SemanticTarget::Dom(node) => Some(node),
            _ => None,
        })
        .collect()
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
    matching_prefer_exact_with_lookup(dom, sel, &DocumentLookup::new(dom, projection))
        .into_iter()
        .filter_map(|target| match target {
            SemanticTarget::Dom(node) => Some(node),
            _ => None,
        })
        .collect()
}

// This is a spelling adapter for the neutral role vocabulary, not DOM role
// inference. In particular Unknown must not resurrect an unprojected ARIA role.
pub(super) fn role_token(role: DocumentA11yRole) -> Option<&'static str> {
    use DocumentA11yRole::*;
    Some(match role {
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
        Cell => "cell",
        Image => "img",
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
        Unknown => return None,
    })
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
        assert!(
            matching_estimated_fixture(&dom, &Selector::role("button").containing("[Save]"))
                .is_empty()
        );
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
        assert_eq!(
            matching_estimated_fixture(&dom, &selector),
            [workbench, detail]
        );
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
        let empty_name =
            DocumentA11yProjection::new(5, projection.support().clone(), projection.root(), nodes);
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

#[cfg(all(test, feature = "accesskit"))]
mod lookup_tests {
    use super::*;
    use layout_dom_api::{LayoutDomMut, QualName, ShadowRootInit};
    fn element(dom: &mut ScriptedDom, parent: NodeId, tag: &str, role: &str) -> NodeId {
        let q = |s: &str| QualName::new(None, Namespace::default(), LocalName::from(s));
        let node = dom.create_element(q(tag));
        dom.set_attribute(node, q("role"), role);
        dom.set_attribute(node, q("aria-label"), "Owned name");
        dom.append_child(parent, node);
        node
    }
    fn projection(dom: &ScriptedDom) -> DocumentA11yProjection {
        let styles = resolve_styles(
            dom,
            &StyleSet::cambium(&["* {display:block}"]),
            &Device::screen(600.0, 600.0),
            &InteractionStates::default(),
        );
        let fragments = layout(dom, &styles, 600.0, 600.0).unwrap();
        genet_render::document_a11y_projection(dom, &fragments, None, 1)
    }
    #[test]
    fn lookup_reads_shadow_and_assigned_nodes_in_flat_order() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let host = element(&mut dom, root, "section", "group");
        let decoy = element(&mut dom, host, "button", "button");
        let shadow = dom.attach_shadow_unchecked(host, ShadowRootInit::default(), false);
        let inside = element(&mut dom, shadow, "button", "button");
        let slot = element(&mut dom, shadow, "slot", "group");
        let fallback = element(&mut dom, slot, "button", "button");
        let q = |s: &str| QualName::new(None, Namespace::default(), LocalName::from(s));
        dom.set_attribute(decoy, q("slot"), "missing");
        let assigned = element(&mut dom, host, "button", "button");
        let p = projection(&dom);
        assert_eq!(
            matching_with_projection(&dom, &Selector::role("button"), &p),
            [inside, assigned]
        );
        assert!(!matching_with_projection(&dom, &Selector::role("button"), &p).contains(&fallback));
        dom.remove(assigned);
        let p = projection(&dom);
        assert_eq!(
            matching_with_projection(&dom, &Selector::role("button"), &p),
            [inside, fallback]
        );
    }
    #[test]
    fn composed_lookup_preserves_dom_roles_and_class_attribute_contracts() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        for role in [
            "button",
            "textbox",
            "checkbox",
            "slider",
            "menu",
            "menubar",
            "cell",
            "image",
            "tree",
            "status",
            "group",
            "spinbutton",
            "paragraph",
            "note",
        ] {
            element(&mut dom, root, "div", role);
        }
        element(&mut dom, root, "label", "label");
        let p = projection(&dom);
        let owner = DocumentLookup::new(&dom, &p);
        let targets: BTreeMap<_, _> = owner
            .nodes()
            .into_iter()
            .filter_map(|n| match n.target {
                SemanticTarget::Dom(node) => Some((dom.opaque_id(node), node)),
                _ => None,
            })
            .collect();
        let tree = genet_render::accesskit_tree_from_projection(p.clone());
        let composed = AccessKitLookup::new(&tree, |id| targets.get(&id).copied());
        for role in [
            "window",
            "button",
            "textbox",
            "checkbox",
            "slider",
            "menu",
            "menubar",
            "cell",
            "gridcell",
            "image",
            "img",
            "tree",
            "status",
            "group",
            "spinbutton",
            "paragraph",
            "note",
            "label",
            "text",
            "unrecognized",
        ] {
            let sel = Selector::role(role).containing("Owned");
            assert_eq!(
                matching_with_lookup(&dom, &sel, &owner),
                matching_with_lookup(&dom, &sel, &composed),
                "{role}"
            );
        }
    }
    #[test]
    fn generated_actions_keep_identity_exact_names_and_hidden_subtrees() {
        use accesskit::{Node, NodeId as Id, Role};
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let native = element(&mut dom, root, "button", "button");
        let q = |s: &str| QualName::new(None, Namespace::default(), LocalName::from(s));
        dom.set_attribute(native, q("aria-label"), "Pin selected in collection");
        dom.set_attribute(native, q("data-key"), "native");
        let p = projection(&dom);
        let mut tree = genet_render::accesskit_tree_from_projection(p);
        let root_id = tree.tree.as_ref().unwrap().root;
        tree.nodes
            .iter_mut()
            .find(|(id, _)| *id == root_id)
            .unwrap()
            .1
            .push_child(Id(999_001));
        let mut action = Node::new(Role::Button);
        action.set_label("Pin selected");
        tree.nodes.push((Id(999_001), action));
        let lookup =
            AccessKitLookup::new(&tree, |id| (id == dom.opaque_id(native)).then_some(native));
        let sel = Selector::role("button").containing("Pin selected");
        assert_eq!(
            matching_with_lookup(&dom, &sel, &lookup),
            [
                SemanticTarget::Dom(native),
                SemanticTarget::Contributed(999_001)
            ]
        );
        assert_eq!(
            matching_prefer_exact_with_lookup(&dom, &sel, &lookup),
            [SemanticTarget::Contributed(999_001)]
        );
        assert_eq!(
            matching_prefer_exact_with_lookup(
                &dom,
                &sel.clone().with_attr("data-key", "native"),
                &lookup
            ),
            [SemanticTarget::Dom(native)]
        );
        tree.nodes
            .iter_mut()
            .find(|(id, _)| *id == root_id)
            .unwrap()
            .1
            .set_hidden();
        let lookup =
            AccessKitLookup::new(&tree, |id| (id == dom.opaque_id(native)).then_some(native));
        assert!(matching_with_lookup(&dom, &sel, &lookup).is_empty());
        // Explicit classes remain DOM selectors even with an unavailable tree.
        dom.set_attribute(native, q("class"), "control");
        let empty = accesskit::TreeUpdate {
            nodes: vec![],
            tree: None,
            ..tree
        };
        let lookup = AccessKitLookup::new(&empty, |_| None);
        assert_eq!(
            matching_with_lookup(&dom, &Selector::class("control"), &lookup),
            [SemanticTarget::Dom(native)]
        );
    }
}
