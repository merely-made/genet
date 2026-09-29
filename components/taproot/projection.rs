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
/// raw DOM attributes would match. Class selectors keep [`matching`]'s shallow
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
        return matching(dom, sel);
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
            matching(&dom, &Selector::role("button")),
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
        assert!(matching(&dom, &Selector::role("button").containing("[Save]")).is_empty());
    }
}
