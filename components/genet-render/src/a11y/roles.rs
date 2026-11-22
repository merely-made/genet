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
    ("grid", DocumentA11yRole::Grid),
    ("treegrid", DocumentA11yRole::TreeGrid),
    ("columnheader", DocumentA11yRole::ColumnHeader),
    ("rowheader", DocumentA11yRole::RowHeader),
    ("gridcell", DocumentA11yRole::GridCell),
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

/// Whether the neutral role enum models ARIA role `token`.
pub(super) fn modelled(token: &str) -> bool {
    ARIA_ROLES.iter().any(|(known, _)| *known == token)
}

/// Every non-abstract role defined in WAI-ARIA 1.2 (Recommendation, section
/// 5.4). Section 5.3's categories omit `code`; 5.4 defines it.
pub const ARIA_1_2_ROLES: &[&str] = &[
    "alert",
    "alertdialog",
    "application",
    "article",
    "banner",
    "blockquote",
    "button",
    "caption",
    "cell",
    "checkbox",
    "code",
    "columnheader",
    "combobox",
    "complementary",
    "contentinfo",
    "definition",
    "deletion",
    "dialog",
    "directory",
    "document",
    "emphasis",
    "feed",
    "figure",
    "form",
    "generic",
    "grid",
    "gridcell",
    "group",
    "heading",
    "img",
    "insertion",
    "link",
    "list",
    "listbox",
    "listitem",
    "log",
    "main",
    "marquee",
    "math",
    "menu",
    "menubar",
    "menuitem",
    "menuitemcheckbox",
    "menuitemradio",
    "meter",
    "navigation",
    "none",
    "note",
    "option",
    "paragraph",
    "presentation",
    "progressbar",
    "radio",
    "radiogroup",
    "region",
    "row",
    "rowgroup",
    "rowheader",
    "scrollbar",
    "search",
    "searchbox",
    "separator",
    "slider",
    "spinbutton",
    "status",
    "strong",
    "subscript",
    "superscript",
    "switch",
    "tab",
    "table",
    "tablist",
    "tabpanel",
    "term",
    "textbox",
    "time",
    "timer",
    "toolbar",
    "tooltip",
    "tree",
    "treegrid",
    "treeitem",
];

/// The role a space-separated `role` value names: its first token that is a
/// WAI-ARIA 1.2 role (or one the enum models, such as `image`), lowercased.
/// The winner may be a role Genet does not model; `None` means no real role.
pub fn first_role_token(value: &str) -> Option<String> {
    value
        .split_ascii_whitespace()
        .map(str::to_ascii_lowercase)
        .find(|token| ARIA_1_2_ROLES.contains(&token.as_str()) || modelled(token))
}

/// The winning role token of `node`'s `role` attribute (see
/// [`first_role_token`]).
pub(super) fn explicit_role<D: LayoutDom>(dom: &D, node: D::NodeId) -> Option<String> {
    first_role_token(dom.attribute(node, &Namespace::default(), &LocalName::from("role"))?)
}

/// An explicit role the enum does not model (e.g. `presentation`) still wins
/// over later tokens; like a lone unmodelled role it leaves the native role.
pub(super) fn document_role<D: LayoutDom>(dom: &D, node: D::NodeId) -> DocumentA11yRole {
    if let Some(role) = explicit_role(dom, node)
        && let Some((_, known)) = ARIA_ROLES.iter().find(|(token, _)| *token == role)
    {
        return *known;
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
            Some("td" | "th") => super::grid::native_cell_role(dom, node),
            Some("img") => DocumentA11yRole::Image,
            _ => DocumentA11yRole::Unknown,
        },
        _ => DocumentA11yRole::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use document_session_api::DocumentA11yRole::{self, *};
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::{LayoutDom, LayoutDomMut};

    use super::document_role;
    use crate::a11y::table_fixture::by_id;

    fn role_of(html: &str) -> DocumentA11yRole {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        dom.set_inner_html(root, html);
        document_role(&dom, by_id(&dom, "t"))
    }

    #[test]
    fn the_first_recognised_role_token_wins() {
        assert_eq!(role_of("<div id=t role='grid foo'></div>"), Grid);
        assert_eq!(role_of("<div id=t role='foo grid'></div>"), Grid);
        assert_eq!(role_of("<div id=t role='row grid'></div>"), Row);
        assert_eq!(role_of("<div id=t role=' \t GRID \n row '></div>"), Grid);
        assert_eq!(role_of("<div id=t role='TreeGrid'></div>"), TreeGrid);
    }

    #[test]
    fn no_recognised_token_falls_back_to_the_native_role() {
        assert_eq!(role_of("<div id=t role=foo></div>"), Unknown);
        assert_eq!(role_of("<button id=t role='foo bar'>b</button>"), Button);
        assert_eq!(role_of("<button id=t role='  '>b</button>"), Button);
        assert_eq!(role_of("<h2 id=t role=''>h</h2>"), Heading { level: 2 });
    }

    /// The first real ARIA role wins even when Genet does not model it, and
    /// then the element keeps its native role, as a lone such role does.
    #[test]
    fn an_unmodelled_real_role_wins_over_later_tokens() {
        assert_eq!(role_of("<div id=t role=presentation></div>"), Unknown);
        assert_eq!(
            role_of("<div id=t role='presentation button'></div>"),
            Unknown
        );
        assert_eq!(role_of("<div id=t role='none grid'></div>"), Unknown);
        assert_eq!(role_of("<div id=t role='foo button'></div>"), Button);
        assert_eq!(role_of("<div id=t role=marquee></div>"), Unknown);
        assert_eq!(role_of("<div id=t role='marquee grid'></div>"), Unknown);
        assert_eq!(
            role_of("<h3 id=t role='presentation button'>h</h3>"),
            Heading { level: 3 }
        );
        assert_eq!(
            role_of("<button id=t role='Marquee grid'>b</button>"),
            Button
        );
    }

    #[test]
    fn the_role_list_has_every_aria_1_2_role_once() {
        let mut roles = super::ARIA_1_2_ROLES.to_vec();
        roles.sort_unstable();
        roles.dedup();
        assert_eq!(roles.len(), 82);
        assert!(!roles.contains(&"image"), "image is ARIA 1.3");
        assert_eq!(
            super::first_role_token(" FOO\tImage "),
            Some("image".into())
        );
    }
}
