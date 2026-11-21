/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Neutral roles from ARIA `role` tokens and native HTML elements.

use document_session_api::DocumentA11yRole;
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

/// ARIA role tokens the neutral role enum models, in both directions.
pub(super) const ARIA_ROLES: &[(&str, DocumentA11yRole)] = &[
    ("document", DocumentA11yRole::Document),
    ("article", DocumentA11yRole::Article),
    ("region", DocumentA11yRole::Region),
    ("group", DocumentA11yRole::Group),
    ("navigation", DocumentA11yRole::Navigation),
    ("main", DocumentA11yRole::Main),
    ("heading", DocumentA11yRole::Heading { level: 0 }),
    ("paragraph", DocumentA11yRole::Paragraph),
    ("link", DocumentA11yRole::Link),
    ("button", DocumentA11yRole::Button),
    ("textbox", DocumentA11yRole::TextField),
    ("checkbox", DocumentA11yRole::CheckBox),
    ("radio", DocumentA11yRole::RadioButton),
    ("radiogroup", DocumentA11yRole::RadioGroup),
    ("switch", DocumentA11yRole::Switch),
    ("combobox", DocumentA11yRole::ComboBox),
    ("listbox", DocumentA11yRole::ListBox),
    ("option", DocumentA11yRole::ListBoxOption),
    ("list", DocumentA11yRole::List),
    ("listitem", DocumentA11yRole::ListItem),
    ("table", DocumentA11yRole::Table),
    ("row", DocumentA11yRole::Row),
    ("cell", DocumentA11yRole::Cell),
    ("gridcell", DocumentA11yRole::Cell),
    ("image", DocumentA11yRole::Image),
    ("img", DocumentA11yRole::Image),
    ("form", DocumentA11yRole::Form),
    ("dialog", DocumentA11yRole::Dialog),
    ("alert", DocumentA11yRole::Alert),
    ("menu", DocumentA11yRole::Menu),
    ("menubar", DocumentA11yRole::MenuBar),
    ("menuitem", DocumentA11yRole::MenuItem),
    ("menuitemcheckbox", DocumentA11yRole::MenuItemCheckBox),
    ("menuitemradio", DocumentA11yRole::MenuItemRadio),
    ("tablist", DocumentA11yRole::TabList),
    ("tab", DocumentA11yRole::Tab),
    ("tabpanel", DocumentA11yRole::TabPanel),
    ("tree", DocumentA11yRole::Tree),
    ("treeitem", DocumentA11yRole::TreeItem),
    ("slider", DocumentA11yRole::Slider),
    ("spinbutton", DocumentA11yRole::SpinButton),
    ("status", DocumentA11yRole::Status),
    ("log", DocumentA11yRole::Log),
    ("note", DocumentA11yRole::Note),
    ("separator", DocumentA11yRole::Splitter),
    ("toolbar", DocumentA11yRole::Toolbar),
    ("progressbar", DocumentA11yRole::ProgressIndicator),
];

/// The ARIA token for a neutral role, when the enum models one.
pub(super) fn aria_token(role: DocumentA11yRole) -> Option<&'static str> {
    let kind = std::mem::discriminant(&role);
    ARIA_ROLES
        .iter()
        .find(|(_, known)| std::mem::discriminant(known) == kind)
        .map(|(token, _)| *token)
}

pub(super) fn document_role<D: LayoutDom>(dom: &D, node: D::NodeId) -> DocumentA11yRole {
    if let Some(role) = dom.attribute(node, &Namespace::default(), &LocalName::from("role")) {
        let role = role.trim().to_ascii_lowercase();
        if let Some((_, known)) = ARIA_ROLES.iter().find(|(token, _)| *token == role) {
            return *known;
        }
    }
    if dom.element_name(node).is_some_and(|name| {
        name.ns.as_str() == "http://www.w3.org/1999/xhtml"
            && name.local.as_str().eq_ignore_ascii_case("input")
    }) {
        let input_type = dom
            .attribute(node, &Namespace::default(), &LocalName::from("type"))
            .map(str::to_ascii_lowercase)
            .unwrap_or_else(|| "text".to_owned());
        match input_type.as_str() {
            "checkbox" => return DocumentA11yRole::CheckBox,
            "radio" => return DocumentA11yRole::RadioButton,
            _ => {},
        }
    }
    match dom.kind(node) {
        NodeKind::Document => DocumentA11yRole::Window,
        NodeKind::Element => match dom.element_name(node).map(|name| name.local.as_str()) {
            Some("html") => DocumentA11yRole::Document,
            Some("article") => DocumentA11yRole::Article,
            Some("nav") => DocumentA11yRole::Navigation,
            Some("main") => DocumentA11yRole::Main,
            Some("form") => DocumentA11yRole::Form,
            Some("dialog") => DocumentA11yRole::Dialog,
            Some("h1") => DocumentA11yRole::Heading { level: 1 },
            Some("h2") => DocumentA11yRole::Heading { level: 2 },
            Some("h3") => DocumentA11yRole::Heading { level: 3 },
            Some("h4") => DocumentA11yRole::Heading { level: 4 },
            Some("h5") => DocumentA11yRole::Heading { level: 5 },
            Some("h6") => DocumentA11yRole::Heading { level: 6 },
            Some("p") => DocumentA11yRole::Paragraph,
            Some("a")
                if dom
                    .attribute(node, &Namespace::default(), &LocalName::from("href"))
                    .is_some() =>
            {
                DocumentA11yRole::Link
            },
            Some("button") => DocumentA11yRole::Button,
            Some("input" | "textarea") => DocumentA11yRole::TextField,
            Some("label") => DocumentA11yRole::Label,
            Some("ul" | "ol") => DocumentA11yRole::List,
            Some("li") => DocumentA11yRole::ListItem,
            Some("table") => DocumentA11yRole::Table,
            Some("tr") => DocumentA11yRole::Row,
            Some("td" | "th") => DocumentA11yRole::Cell,
            Some("img") => DocumentA11yRole::Image,
            _ => DocumentA11yRole::Unknown,
        },
        _ => DocumentA11yRole::Unknown,
    }
}
