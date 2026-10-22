// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! HTML directionality from native current values, ordinary ancestry, shadow
//! hosts and slot assignment. Unicode first-strong data lives in its own table.

use crate::forms_directionality_data::strong_direction;
use crate::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, NodeKind};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormDirection {
    Ltr,
    Rtl,
}
impl FormDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
}
pub fn text_directionality(value: &str) -> Option<FormDirection> {
    value.chars().find_map(strong_direction)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DirState {
    Undefined,
    Ltr,
    Rtl,
    Auto,
}

// A contained slot can defer to its shadow host's directionality. Keeping that
// as a dependency allows iterative resolution instead of recursive ancestry.
enum AutoSource {
    Known(Option<FormDirection>),
    Host(NodeId),
}

impl ScriptedDom {
    pub fn element_directionality(&self, id: NodeId) -> FormDirection {
        let mut cursor = id;
        let mut seen = HashSet::new();
        while self.is_live(cursor) && seen.insert(cursor) {
            match self.direction_state(cursor) {
                DirState::Ltr => return FormDirection::Ltr,
                DirState::Rtl => return FormDirection::Rtl,
                state => {
                    if state == DirState::Auto || self.html_name(cursor) == Some("bdi") {
                        match self.auto_directionality_source(cursor) {
                            AutoSource::Known(value) => return value.unwrap_or(FormDirection::Ltr),
                            AutoSource::Host(host) => {
                                cursor = host;
                                continue;
                            },
                        }
                    }
                    if self.html_name(cursor) == Some("input")
                        && self.input_type_for_state(cursor) == "tel"
                    {
                        return FormDirection::Ltr;
                    }
                    match self.parent_directionality_element(cursor) {
                        Some(parent) => cursor = parent,
                        None => return FormDirection::Ltr,
                    }
                },
            }
        }
        // Native DOM construction prevents ancestry cycles. The guard also
        // bounds unusual host/slot dependency graphs without Rust recursion.
        FormDirection::Ltr
    }

    pub fn auto_directionality(&self, id: NodeId) -> Option<FormDirection> {
        match self.auto_directionality_source(id) {
            AutoSource::Known(direction) => direction,
            AutoSource::Host(host) => Some(self.element_directionality(host)),
        }
    }

    pub fn is_auto_directionality_control(&self, id: NodeId) -> bool {
        if !self.is_live(id) {
            return false;
        }
        match self.html_name(id) {
            Some("textarea") => true,
            Some("input") => matches!(
                self.input_type_for_state(id).as_str(),
                "hidden"
                    | "text"
                    | "search"
                    | "tel"
                    | "url"
                    | "email"
                    | "password"
                    | "submit"
                    | "reset"
                    | "button"
            ),
            _ => false,
        }
    }

    fn direction_state(&self, id: NodeId) -> DirState {
        // dir's enumerated states only apply in the HTML namespace. Whitespace
        // is significant: " ltr " is Undefined rather than Ltr.
        if self.html_name(id).is_none() {
            return DirState::Undefined;
        }
        match self.attribute(id, &markup5ever::ns!(), &LocalName::from("dir")) {
            Some(value) if value.eq_ignore_ascii_case("ltr") => DirState::Ltr,
            Some(value) if value.eq_ignore_ascii_case("rtl") => DirState::Rtl,
            Some(value) if value.eq_ignore_ascii_case("auto") => DirState::Auto,
            _ => DirState::Undefined,
        }
    }

    fn parent_directionality_element(&self, id: NodeId) -> Option<NodeId> {
        let parent = self.parent(id)?;
        match self.kind(parent) {
            NodeKind::Element => Some(parent),
            NodeKind::ShadowRoot => self.shadow_host_of(parent),
            _ => None,
        }
    }

    fn auto_directionality_source(&self, id: NodeId) -> AutoSource {
        if !self.is_live(id) {
            return AutoSource::Known(None);
        }
        if self.is_auto_directionality_control(id) {
            let value = self.form_control_value(id).unwrap_or_default();
            return AutoSource::Known(if value.is_empty() {
                None
            } else {
                Some(text_directionality(&value).unwrap_or(FormDirection::Ltr))
            });
        }
        if self.html_name(id) == Some("slot") && self.containing_shadow_root_of(id).is_some() {
            let assigned = self.assigned_nodes_of(id);
            if !assigned.is_empty() {
                for child in assigned {
                    let direction = match self.kind(child) {
                        NodeKind::Text | NodeKind::CdataSection => {
                            AutoSource::Known(text_directionality(self.text(child).unwrap_or("")))
                        },
                        NodeKind::Element => self.contained_text_directionality(child, true),
                        _ => AutoSource::Known(None),
                    };
                    if !matches!(direction, AutoSource::Known(None)) {
                        return direction;
                    }
                }
                return AutoSource::Known(None);
            }
        }
        self.contained_text_directionality(id, false)
    }

    fn directionality_boundary(&self, id: NodeId) -> bool {
        matches!(
            self.html_name(id),
            Some("bdi" | "script" | "style" | "textarea")
        ) || self.direction_state(id) != DirState::Undefined
    }

    fn contained_text_directionality(&self, root: NodeId, can_exclude_root: bool) -> AutoSource {
        if can_exclude_root && self.directionality_boundary(root) {
            return AutoSource::Known(None);
        }
        let mut stack = self.dom_children(root).collect::<Vec<_>>();
        stack.reverse();
        while let Some(descendant) = stack.pop() {
            match self.kind(descendant) {
                NodeKind::Element => {
                    if self.directionality_boundary(descendant) {
                        continue;
                    }
                    if self.html_name(descendant) == Some("slot")
                        && let Some(shadow) = self.containing_shadow_root_of(descendant)
                        && let Some(host) = self.shadow_host_of(shadow)
                    {
                        return AutoSource::Host(host);
                    }
                },
                NodeKind::Text | NodeKind::CdataSection => {
                    if let Some(direction) =
                        text_directionality(self.text(descendant).unwrap_or(""))
                    {
                        return AutoSource::Known(Some(direction));
                    }
                },
                _ => {},
            }
            let children = self.dom_children(descendant).collect::<Vec<_>>();
            stack.extend(children.into_iter().rev());
        }
        AutoSource::Known(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout_dom_api::{LayoutDomMut, LocalName, QualName};
    use markup5ever::ns;
    fn element(dom: &mut ScriptedDom, name: &str) -> NodeId {
        dom.create_element(QualName::new(None, ns!(html), LocalName::from(name)))
    }
    fn attr(dom: &mut ScriptedDom, id: NodeId, name: &str, value: &str) {
        dom.set_attribute(id, QualName::new(None, ns!(), LocalName::from(name)), value);
    }
    fn text(dom: &mut ScriptedDom, parent: NodeId, value: &str) {
        let id = dom.create_text(value);
        dom.append_child(parent, id);
    }

    #[test]
    fn phase_c_directionality_first_strong_and_unicode17_defaults() {
        assert_eq!(
            text_directionality("123😀\u{0301}عA"),
            Some(FormDirection::Rtl)
        );
        assert_eq!(
            text_directionality("123😀\u{0301}Aع"),
            Some(FormDirection::Ltr)
        );
        assert_eq!(text_directionality("\u{1E900}"), Some(FormDirection::Rtl));
        assert_eq!(text_directionality("\u{1EE00}"), Some(FormDirection::Rtl));
        assert_eq!(text_directionality("\u{0590}"), Some(FormDirection::Rtl));
        assert_eq!(text_directionality("123😀\u{0301}"), None);
        assert_eq!(text_directionality(""), None);
    }
    #[test]
    fn phase_c_directionality_current_control_neutrals_empty_and_tel() {
        let mut dom = ScriptedDom::new();
        let parent = element(&mut dom, "div");
        attr(&mut dom, parent, "dir", "rtl");
        let i = element(&mut dom, "input");
        attr(&mut dom, i, "value", "Latin");
        attr(&mut dom, i, "dir", "auto");
        dom.append_child(parent, i);
        dom.set_form_control_value(i, "ع").unwrap();
        assert_eq!(dom.element_directionality(i), FormDirection::Rtl);
        dom.set_form_control_value(i, "123😀").unwrap();
        assert_eq!(dom.element_directionality(i), FormDirection::Ltr);
        dom.set_form_control_value(i, "").unwrap();
        assert_eq!(dom.element_directionality(i), FormDirection::Ltr);
        dom.remove_attribute(i, QualName::new(None, ns!(), LocalName::from("dir")));
        attr(&mut dom, i, "type", "tel");
        assert_eq!(dom.element_directionality(i), FormDirection::Ltr);
        attr(&mut dom, i, "dir", "auto");
        dom.set_form_control_value(i, "ع").unwrap();
        assert_eq!(dom.element_directionality(i), FormDirection::Rtl);
    }
    #[test]
    fn phase_c_directionality_contained_text_boundaries_and_invalid_dir() {
        let mut dom = ScriptedDom::new();
        let parent = element(&mut dom, "div");
        attr(&mut dom, parent, "dir", "auto");
        for tag in ["bdi", "script", "style", "textarea"] {
            let excluded = element(&mut dom, tag);
            dom.append_child(parent, excluded);
            text(&mut dom, excluded, "Latin");
        }
        let valid = element(&mut dom, "span");
        attr(&mut dom, valid, "dir", "ltr");
        dom.append_child(parent, valid);
        text(&mut dom, valid, "Latin");
        let invalid = element(&mut dom, "span");
        attr(&mut dom, invalid, "dir", " ltr ");
        dom.append_child(parent, invalid);
        text(&mut dom, invalid, "ع");
        assert_eq!(dom.element_directionality(parent), FormDirection::Rtl);
        assert_eq!(dom.element_directionality(invalid), FormDirection::Rtl);
    }
    #[test]
    fn phase_c_directionality_shadow_host_and_assigned_slot_text() {
        let mut dom = ScriptedDom::new();
        let host = element(&mut dom, "div");
        attr(&mut dom, host, "dir", "rtl");
        let shadow = dom
            .attach_shadow(host, crate::ShadowRootInit::default())
            .unwrap();
        let child = element(&mut dom, "span");
        dom.append_child(shadow, child);
        assert_eq!(dom.element_directionality(child), FormDirection::Rtl);
        let slot = element(&mut dom, "slot");
        attr(&mut dom, slot, "dir", "auto");
        dom.append_child(shadow, slot);
        text(&mut dom, host, "ع");
        assert_eq!(dom.element_directionality(slot), FormDirection::Rtl);
    }
}
