// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use accesskit::Role;

/// Shared role spelling for semantic selectors and ARIA lowering.
/// Unsupported roles have no invented token.
pub fn accesskit_role_token(role: Role) -> Option<&'static str> {
    Some(match role {
        Role::Window => "window",
        Role::TextRun => "text",
        Role::Label => "label",
        Role::Document => "document",
        Role::Article => "article",
        Role::Region => "region",
        Role::Group => "group",
        Role::Navigation => "navigation",
        Role::Main => "main",
        Role::Heading => "heading",
        Role::Paragraph => "paragraph",
        Role::Link => "link",
        Role::Button => "button",
        Role::TextInput | Role::MultilineTextInput => "textbox",
        Role::CheckBox => "checkbox",
        Role::RadioButton => "radio",
        Role::RadioGroup => "radiogroup",
        Role::Switch => "switch",
        Role::ComboBox => "combobox",
        Role::List => "list",
        Role::ListItem => "listitem",
        Role::ListBox => "listbox",
        Role::ListBoxOption => "option",
        Role::Table => "table",
        Role::Row => "row",
        Role::Cell => "cell",
        Role::Image | Role::Canvas | Role::ColorWell => "img",
        Role::Form => "form",
        Role::Dialog => "dialog",
        Role::Alert => "alert",
        Role::Menu => "menu",
        Role::MenuBar => "menubar",
        Role::MenuItem => "menuitem",
        Role::MenuItemCheckBox => "menuitemcheckbox",
        Role::MenuItemRadio => "menuitemradio",
        Role::TabList => "tablist",
        Role::Tab => "tab",
        Role::TabPanel => "tabpanel",
        Role::Tree => "tree",
        Role::TreeItem => "treeitem",
        Role::Slider => "slider",
        Role::SpinButton => "spinbutton",
        Role::Splitter => "separator",
        Role::Toolbar => "toolbar",
        Role::ProgressIndicator => "progressbar",
        Role::Meter => "meter",
        Role::Status => "status",
        Role::Log => "log",
        Role::Note => "note",
        Role::GraphicsDocument => "graphics-document",
        Role::GraphicsObject => "graphics-object",
        Role::GraphicsSymbol => "graphics-symbol",
        _ => return None,
    })
}

/// ARIA spelling for the supported platform roles. Structural window/text/label
/// nodes retain their ordinary browser element semantics.
pub fn accesskit_aria_role(role: Role) -> Option<&'static str> {
    match role {
        Role::Window | Role::TextRun | Role::Label => None,
        _ => accesskit_role_token(role),
    }
}
