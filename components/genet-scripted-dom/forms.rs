// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! HTML input and textarea current-value state.

use layout_dom_api::{
    FormControlState, LayoutDom, LayoutDomMut, LocalName, QualName, SelectionDirection,
};

use crate::{NodeId, ScriptedDom};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormControlValueError {
    InvalidState,
}

impl ScriptedDom {
    /// Whether HTML's selection APIs apply to this control. Arena selection
    /// offsets are also retained for other native-editable value-mode inputs,
    /// but script must observe null/InvalidStateError for those types.
    pub fn form_control_selection_applies(&self, id: NodeId) -> bool {
        if self.is_html_textarea(id) {
            return true;
        }
        self.is_html_input(id)
            && matches!(
                self.input_type(id).as_str(),
                "text" | "search" | "tel" | "url" | "password"
            )
    }

    /// Form-control values under `root`, keyed by `name` and then `id`.
    /// Inputs and textareas use their current arena value; select retains its
    /// pre-existing attribute-based behavior until selectedness is implemented.
    pub fn form_values(&self, root: NodeId) -> Vec<(String, String)> {
        let mut out = Vec::new();
        self.collect_form_values(root, &mut out);
        out
    }

    fn collect_form_values(&self, id: NodeId, out: &mut Vec<(String, String)>) {
        let node = self.node(id);
        if node.kind == layout_dom_api::NodeKind::Element {
            if let Some(name) = &node.name {
                if name.ns == markup5ever::ns!(html)
                    && matches!(name.local.as_ref(), "input" | "select" | "textarea")
                {
                    let attr = |local: &str| {
                        node.attrs
                            .iter()
                            .find(|(n, _)| n.ns == markup5ever::ns!() && n.local.as_ref() == local)
                            .map(|(_, v)| v.as_str())
                    };
                    if let Some(key) = attr("name").or_else(|| attr("id")) {
                        let value = match name.local.as_ref() {
                            "input" | "textarea" => self.form_control_value(id).unwrap_or_default(),
                            _ => attr("value").unwrap_or("").to_owned(),
                        };
                        out.push((key.to_owned(), value));
                    }
                }
            }
        }
        for child in self.dom_children(id) {
            self.collect_form_values(child, out);
        }
    }

    pub fn form_control_value(&self, id: NodeId) -> Option<String> {
        let node = self.node(id);
        let name = node.name.as_ref()?;
        if name.ns != markup5ever::ns!(html) {
            return None;
        }
        match name.local.as_ref() {
            "textarea" => Some(api_value(&self.node(id).form_control.as_ref()?.value)),
            "input" => {
                let state = node.form_control.as_ref()?;
                Some(match self.input_value_mode(id) {
                    InputValueMode::Value => state.value.clone(),
                    InputValueMode::Default => self
                        .attribute(id, &markup5ever::ns!(), &LocalName::from("value"))
                        .unwrap_or("")
                        .to_owned(),
                    InputValueMode::DefaultOn => self
                        .attribute(id, &markup5ever::ns!(), &LocalName::from("value"))
                        .unwrap_or("on")
                        .to_owned(),
                    InputValueMode::Filename => String::new(),
                })
            },
            _ => None,
        }
    }

    pub fn set_form_control_value(
        &mut self,
        id: NodeId,
        value: &str,
    ) -> Result<(), FormControlValueError> {
        let Some(name) = self.element_name(id).cloned() else {
            return Err(FormControlValueError::InvalidState);
        };
        if name.ns != markup5ever::ns!(html) {
            return Err(FormControlValueError::InvalidState);
        }
        match name.local.as_ref() {
            "textarea" => {
                let state = self
                    .node_mut(id)
                    .form_control
                    .as_mut()
                    .ok_or(FormControlValueError::InvalidState)?;
                let old = api_value(&state.value);
                state.value = value.to_owned();
                state.dirty_value = true;
                let current = state.value.clone();
                if api_value(&current) != old {
                    collapse_selection(state, &current);
                }
                self.mutations
                    .push(crate::DomMutation::FormControlStateChanged { node: id });
                Ok(())
            },
            "input" => match self.input_value_mode(id) {
                InputValueMode::Filename if !value.is_empty() => {
                    Err(FormControlValueError::InvalidState)
                },
                InputValueMode::Filename => {
                    if let Some(state) = self.node_mut(id).form_control.as_mut() {
                        state.value.clear();
                    }
                    Ok(())
                },
                InputValueMode::Default | InputValueMode::DefaultOn => {
                    self.set_attribute(id, attr_name("value"), value);
                    Ok(())
                },
                InputValueMode::Value => {
                    let kind = self.input_type(id).to_owned();
                    let old = self.form_control_value(id).unwrap_or_default();
                    let value = self.sanitize_control_value(id, &kind, value);
                    let state = self
                        .node_mut(id)
                        .form_control
                        .as_mut()
                        .ok_or(FormControlValueError::InvalidState)?;
                    state.value = value.clone();
                    state.dirty_value = true;
                    if value != old && state.selection_start.is_some() {
                        collapse_selection(state, &value);
                    }
                    self.mutations
                        .push(crate::DomMutation::FormControlStateChanged { node: id });
                    Ok(())
                },
            },
            _ => Err(FormControlValueError::InvalidState),
        }
    }

    pub fn set_form_control_checked(&mut self, id: NodeId, checked: bool) -> bool {
        if !self.is_html_input(id) {
            return false;
        }
        let radio = checked && self.input_type(id) == "radio";
        let Some(state) = self.node_mut(id).form_control.as_mut() else {
            return false;
        };
        state.checked = checked;
        state.dirty_checkedness = true;
        if radio {
            self.uncheck_radio_group(id);
        }
        self.mutations
            .push(crate::DomMutation::FormControlStateChanged { node: id });
        true
    }

    pub fn reset_form_control(&mut self, id: NodeId) -> bool {
        let Some(name) = self.element_name(id).cloned() else {
            return false;
        };
        if name.ns != markup5ever::ns!(html) {
            return false;
        }
        match name.local.as_ref() {
            "input" => {
                let kind = self.input_type(id).to_owned();
                let default = self.input_attribute(id, "value").unwrap_or("").to_owned();
                let checked = self.has_input_attribute(id, "checked");
                let mode = value_mode(&kind);
                let current = match mode {
                    InputValueMode::Default => default.clone(),
                    InputValueMode::DefaultOn => {
                        self.input_attribute(id, "value").unwrap_or("on").to_owned()
                    },
                    InputValueMode::Filename => String::new(),
                    InputValueMode::Value => self.sanitize_control_value(id, &kind, &default),
                };
                let state = self.node_mut(id).form_control.as_mut().unwrap();
                state.value = current;
                state.dirty_value = false;
                state.checked = checked;
                state.dirty_checkedness = false;
                set_selection_support(state, &kind, false);
                let radio = kind == "radio" && checked;
                if radio {
                    self.uncheck_radio_group(id);
                }
                self.mutations
                    .push(crate::DomMutation::FormControlStateChanged { node: id });
                true
            },
            "textarea" => {
                let default = self.textarea_default_value(id);
                let state = self.node_mut(id).form_control.as_mut().unwrap();
                state.value = default;
                state.dirty_value = false;
                self.mutations
                    .push(crate::DomMutation::FormControlStateChanged { node: id });
                true
            },
            _ => false,
        }
    }

    pub fn clone_form_control_state(&mut self, source: NodeId, target: NodeId) -> bool {
        let (Some(src_name), Some(dst_name)) = (
            self.element_name(source).cloned(),
            self.element_name(target).cloned(),
        ) else {
            return false;
        };
        if src_name.ns != markup5ever::ns!(html) || src_name != dst_name {
            return false;
        }
        let Some(src) = self.node(source).form_control.clone() else {
            return false;
        };
        let Some(dst) = self.node_mut(target).form_control.as_mut() else {
            return false;
        };
        match src_name.local.as_ref() {
            "input" => {
                dst.value = src.value;
                dst.dirty_value = src.dirty_value;
                dst.checked = src.checked;
                dst.dirty_checkedness = src.dirty_checkedness;
            },
            "textarea" => {
                dst.value = src.value;
                dst.dirty_value = src.dirty_value;
            },
            _ => return false,
        }
        true
    }

    /// Raw arena state for HTML's element cloning steps. The public LayoutDom
    /// projection normalizes textarea API values; cloning needs the raw value.
    pub fn form_control_clone_state(&self, id: NodeId) -> Option<FormControlState> {
        self.node(id).form_control.clone()
    }

    pub(super) fn form_control_attribute_changed(
        &mut self,
        id: NodeId,
        name: &QualName,
        old_type: Option<&str>,
    ) {
        if name.ns != markup5ever::ns!() || !self.is_html_input(id) {
            return;
        }
        match name.local.as_ref() {
            "value" if self.input_value_mode(id) != InputValueMode::Value => {
                let value = match self.input_value_mode(id) {
                    InputValueMode::Default => {
                        self.input_attribute(id, "value").unwrap_or("").to_owned()
                    },
                    InputValueMode::DefaultOn => {
                        self.input_attribute(id, "value").unwrap_or("on").to_owned()
                    },
                    InputValueMode::Filename => String::new(),
                    InputValueMode::Value => unreachable!(),
                };
                self.node_mut(id).form_control.as_mut().unwrap().value = value;
            },
            "value" if !self.node(id).form_control.as_ref().unwrap().dirty_value => {
                let kind = self.input_type(id).to_owned();
                let value = self.input_attribute(id, "value").unwrap_or("");
                let sanitized = self.sanitize_control_value(id, &kind, value);
                self.node_mut(id).form_control.as_mut().unwrap().value = sanitized;
            },
            "checked"
                if !self
                    .node(id)
                    .form_control
                    .as_ref()
                    .unwrap()
                    .dirty_checkedness =>
            {
                let checked = self.has_input_attribute(id, "checked");
                self.node_mut(id).form_control.as_mut().unwrap().checked = checked;
                if checked && self.input_type(id) == "radio" {
                    self.uncheck_radio_group(id);
                }
            },
            "name" | "form"
                if self.input_type(id) == "radio"
                    && self.node(id).form_control.as_ref().unwrap().checked =>
            {
                self.uncheck_radio_group(id);
            },
            "type" => {
                let old_type = old_type.unwrap_or("text");
                let old_mode = value_mode(old_type);
                let new_type = self.input_type(id).to_owned();
                let new_mode = value_mode(&new_type);
                if old_mode == InputValueMode::Value
                    && matches!(
                        new_mode,
                        InputValueMode::Default | InputValueMode::DefaultOn
                    )
                {
                    let value = self.node(id).form_control.as_ref().unwrap().value.clone();
                    if !value.is_empty() {
                        self.set_attribute(id, attr_name("value"), &value);
                    }
                } else if old_mode != InputValueMode::Value && new_mode == InputValueMode::Value {
                    let value = self.input_attribute(id, "value").unwrap_or("").to_owned();
                    let sanitized = self.sanitize_control_value(id, &new_type, &value);
                    let state = self.node_mut(id).form_control.as_mut().unwrap();
                    state.value = sanitized;
                    state.dirty_value = false;
                }
                let synced_value = match new_mode {
                    InputValueMode::Value => {
                        let current = self.node(id).form_control.as_ref().unwrap().value.clone();
                        self.sanitize_control_value(id, &new_type, &current)
                    },
                    InputValueMode::Default => {
                        self.input_attribute(id, "value").unwrap_or("").to_owned()
                    },
                    InputValueMode::DefaultOn => {
                        self.input_attribute(id, "value").unwrap_or("on").to_owned()
                    },
                    InputValueMode::Filename => String::new(),
                };
                self.node_mut(id).form_control.as_mut().unwrap().value = synced_value;
                set_selection_support(
                    self.node_mut(id).form_control.as_mut().unwrap(),
                    &new_type,
                    true,
                );
                if new_type == "radio" && self.node(id).form_control.as_ref().unwrap().checked {
                    self.uncheck_radio_group(id);
                }
            },
            "min" | "max" | "step" if self.input_type(id) == "range" => {
                let value = self.node(id).form_control.as_ref().unwrap().value.clone();
                let sanitized = self.sanitize_control_value(id, "range", &value);
                self.node_mut(id).form_control.as_mut().unwrap().value = sanitized;
            },
            _ => {},
        }
    }

    pub(super) fn form_control_children_changed(&mut self, node: NodeId) {
        let textarea = if self.is_html_textarea(node) {
            Some(node)
        } else if matches!(
            self.kind(node),
            layout_dom_api::NodeKind::Text | layout_dom_api::NodeKind::CdataSection
        ) {
            self.parent(node)
                .filter(|parent| self.is_html_textarea(*parent))
        } else {
            None
        };
        if let Some(id) = textarea {
            if !self.node(id).form_control.as_ref().unwrap().dirty_value {
                let value = self.textarea_default_value(id);
                self.node_mut(id).form_control.as_mut().unwrap().value = value;
            }
        }
    }

    pub(super) fn radio_group_membership_changed(&mut self, subtree: NodeId) {
        let mut descendants = Vec::new();
        collect_descendants(self, subtree, &mut descendants);
        for id in descendants {
            if self.is_html_input(id)
                && self.input_type(id) == "radio"
                && self
                    .node(id)
                    .form_control
                    .as_ref()
                    .is_some_and(|state| state.checked)
            {
                self.uncheck_radio_group(id);
            }
        }
    }

    fn is_html_input(&self, id: NodeId) -> bool {
        self.element_name(id)
            .is_some_and(|q| q.ns == markup5ever::ns!(html) && q.local.as_ref() == "input")
    }
    fn is_html_textarea(&self, id: NodeId) -> bool {
        self.element_name(id)
            .is_some_and(|q| q.ns == markup5ever::ns!(html) && q.local.as_ref() == "textarea")
    }
    fn input_type(&self, id: NodeId) -> String {
        let kind = self
            .input_attribute(id, "type")
            .unwrap_or("text")
            .to_ascii_lowercase();
        if matches!(
            kind.as_str(),
            "hidden"
                | "text"
                | "search"
                | "tel"
                | "url"
                | "email"
                | "password"
                | "date"
                | "month"
                | "week"
                | "time"
                | "datetime-local"
                | "number"
                | "range"
                | "color"
                | "checkbox"
                | "radio"
                | "file"
                | "submit"
                | "image"
                | "reset"
                | "button"
        ) {
            kind
        } else {
            "text".to_owned()
        }
    }
    fn input_attribute(&self, id: NodeId, local: &str) -> Option<&str> {
        self.attribute(id, &markup5ever::ns!(), &LocalName::from(local))
    }
    fn has_input_attribute(&self, id: NodeId, local: &str) -> bool {
        self.input_attribute(id, local).is_some()
    }
    fn input_value_mode(&self, id: NodeId) -> InputValueMode {
        value_mode(&self.input_type(id))
    }
    fn sanitize_control_value(&self, id: NodeId, kind: &str, value: &str) -> String {
        sanitize_input_value_with_options(
            kind,
            value,
            self.input_attribute(id, "min"),
            self.input_attribute(id, "max"),
            self.input_attribute(id, "step"),
            self.has_input_attribute(id, "multiple"),
            self.input_attribute(id, "value"),
        )
    }

    fn textarea_default_value(&self, id: NodeId) -> String {
        let mut out = String::new();
        // Only direct Text children contribute to a textarea's value.
        for child in self.dom_children(id) {
            if matches!(
                self.kind(child),
                layout_dom_api::NodeKind::Text | layout_dom_api::NodeKind::CdataSection
            ) {
                out.push_str(self.text(child).unwrap_or(""));
            }
        }
        out
    }

    fn uncheck_radio_group(&mut self, id: NodeId) {
        let name = self.input_attribute(id, "name").unwrap_or("").to_owned();
        if name.is_empty() {
            return;
        }
        let root = self.tree_root(id);
        let form_owner = self.input_form_owner(id, root);
        let mut descendants = Vec::new();
        collect_descendants(self, root, &mut descendants);
        let peers: Vec<_> = descendants
            .into_iter()
            .filter(|peer| {
                *peer != id
                    && self.is_html_input(*peer)
                    && self.input_type(*peer) == "radio"
                    && self.input_attribute(*peer, "name") == Some(name.as_str())
                    && self.input_form_owner(*peer, root) == form_owner
            })
            .collect();
        for peer in peers {
            let changed = self
                .node(peer)
                .form_control
                .as_ref()
                .is_some_and(|state| state.checked);
            if changed {
                self.node_mut(peer).form_control.as_mut().unwrap().checked = false;
                self.mutations
                    .push(crate::DomMutation::FormControlStateChanged { node: peer });
            }
        }
    }
    fn input_form_owner(&self, id: NodeId, root: NodeId) -> Option<NodeId> {
        if let Some(form_id) = self.input_attribute(id, "form") {
            let mut nodes = Vec::new();
            collect_descendants(self, root, &mut nodes);
            return nodes.into_iter().find(|candidate| {
                self.element_name(*candidate).is_some_and(|q| {
                    q.ns == markup5ever::ns!(html)
                        && q.local.as_ref() == "form"
                        && self.attribute(*candidate, &markup5ever::ns!(), &LocalName::from("id"))
                            == Some(form_id)
                })
            });
        }
        let mut parent = self.parent(id);
        while let Some(candidate) = parent {
            if self
                .element_name(candidate)
                .is_some_and(|q| q.ns == markup5ever::ns!(html) && q.local.as_ref() == "form")
            {
                return Some(candidate);
            }
            parent = self.parent(candidate);
        }
        None
    }
    fn tree_root(&self, id: NodeId) -> NodeId {
        let mut root = id;
        while let Some(parent) = self.parent(root) {
            root = parent;
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use html5ever::{local_name, ns};

    fn tag(local: markup5ever::LocalName) -> QualName {
        QualName::new(None, ns!(html), local)
    }
    fn attr(local: markup5ever::LocalName) -> QualName {
        QualName::new(None, ns!(), local)
    }

    #[test]
    fn input_current_value_is_separate_from_attribute_and_reset_restores_it() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(tag(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("value")), "default");
        assert_eq!(dom.form_control_value(input).as_deref(), Some("default"));
        dom.set_form_control_value(input, "live\nvalue").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some("livevalue"));
        dom.set_attribute(input, attr(local_name!("value")), "new-default");
        assert_eq!(dom.form_control_value(input).as_deref(), Some("livevalue"));
        assert!(dom.reset_form_control(input));
        assert_eq!(
            dom.form_control_value(input).as_deref(),
            Some("new-default")
        );
    }

    #[test]
    fn value_state_mutation_invalidates_layout_without_becoming_an_attribute_record() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(tag(local_name!("input")));
        dom.set_observing(true);
        dom.set_form_control_value(input, "typed").unwrap();
        assert!(matches!(
            dom.pending_mutations().1.last(),
            Some(crate::DomMutation::FormControlStateChanged { node }) if *node == input
        ));
        assert!(dom.take_observed().is_empty());
    }

    #[test]
    fn native_editor_selection_is_retained_for_value_types_without_selection_idl() {
        let mut dom = ScriptedDom::new();
        let email = dom.create_element(tag(local_name!("input")));
        dom.set_attribute(email, attr(local_name!("type")), "email");
        dom.set_form_control_value(email, "person@example.test")
            .unwrap();
        let mut state = dom.form_control_clone_state(email).unwrap();
        state.selection_start = Some(6);
        state.selection_end = Some(6);
        assert!(dom.set_form_control_state(email, state));
        assert_eq!(
            dom.form_control_state(email).unwrap().selection_start,
            Some(6)
        );
        assert!(!dom.form_control_selection_applies(email));

        let number = dom.create_element(tag(local_name!("input")));
        dom.set_attribute(number, attr(local_name!("type")), "number");
        dom.set_form_control_value(number, "12345").unwrap();
        let mut state = dom.form_control_clone_state(number).unwrap();
        state.selection_start = Some(3);
        state.selection_end = Some(3);
        assert!(dom.set_form_control_state(number, state));
        assert_eq!(
            dom.form_control_state(number).unwrap().selection_start,
            Some(3)
        );
        assert!(!dom.form_control_selection_applies(number));

        let text = dom.create_element(tag(local_name!("input")));
        assert!(dom.form_control_selection_applies(text));
    }

    #[test]
    fn textarea_children_track_default_until_dirty_and_api_value_normalizes_newlines() {
        let mut dom = ScriptedDom::new();
        let ta = dom.create_element(tag(local_name!("textarea")));
        let text = dom.create_text("first\r\nline");
        dom.append_child(ta, text);
        assert_eq!(dom.form_control_value(ta).as_deref(), Some("first\nline"));
        dom.set_form_control_value(ta, "edited").unwrap();
        dom.set_text(text, "changed");
        assert_eq!(dom.form_control_value(ta).as_deref(), Some("edited"));
        assert!(dom.reset_form_control(ta));
        assert_eq!(dom.form_control_value(ta).as_deref(), Some("changed"));
        assert_eq!(dom.form_control_clone_state(ta).unwrap().value, "changed");
    }

    #[test]
    fn textarea_value_uses_direct_text_children_only() {
        let mut dom = ScriptedDom::new();
        let ta = dom.create_element(tag(local_name!("textarea")));
        let direct = dom.create_text("foo");
        let span = dom.create_element(tag(local_name!("span")));
        let nested = dom.create_text("baz");
        dom.append_child(ta, direct);
        dom.append_child(ta, span);
        dom.append_child(span, nested);
        assert_eq!(dom.form_control_value(ta).as_deref(), Some("foo"));
        dom.set_text(nested, "qux");
        assert_eq!(dom.form_control_value(ta).as_deref(), Some("foo"));
        dom.set_text(direct, "bar");
        assert_eq!(dom.form_control_value(ta).as_deref(), Some("bar"));
    }

    #[test]
    fn checkedness_tracks_default_until_setter_marks_it_dirty() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(tag(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "checkbox");
        dom.set_attribute(input, attr(local_name!("checked")), "");
        assert!(dom.form_control_state(input).unwrap().checked);
        assert!(dom.set_form_control_checked(input, false));
        dom.remove_attribute(input, attr(local_name!("checked")));
        assert!(!dom.form_control_state(input).unwrap().checked);
        dom.set_attribute(input, attr(local_name!("checked")), "");
        assert!(!dom.form_control_state(input).unwrap().checked);
        dom.reset_form_control(input);
        assert!(dom.form_control_state(input).unwrap().checked);
    }

    #[test]
    fn value_modes_and_sanitization_follow_input_type() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(tag(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "checkbox");
        assert_eq!(dom.form_control_value(input).as_deref(), Some("on"));
        dom.set_form_control_value(input, "yes").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some("yes"));
        dom.set_attribute(input, attr(local_name!("type")), "file");
        assert_eq!(dom.form_control_value(input).as_deref(), Some(""));
        assert_eq!(
            dom.set_form_control_value(input, "secret"),
            Err(FormControlValueError::InvalidState)
        );
        dom.set_form_control_value(input, "").unwrap();
        dom.set_attribute(input, attr(local_name!("type")), "number");
        dom.set_form_control_value(input, "not-a-number").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some(""));
    }

    #[test]
    fn typed_sanitizers_reject_bad_numbers_dates_and_weeks_and_normalize_local_time() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(tag(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "number");
        for value in ["1.", "1e999", "+1"] {
            dom.set_form_control_value(input, value).unwrap();
            assert_eq!(dom.form_control_value(input).as_deref(), Some(""));
        }
        dom.set_attribute(input, attr(local_name!("type")), "date");
        dom.set_form_control_value(input, "2024-02-30").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some(""));
        dom.set_form_control_value(input, "12345-02-28").unwrap();
        assert_eq!(
            dom.form_control_value(input).as_deref(),
            Some("12345-02-28")
        );
        dom.set_form_control_value(input, "12345678901234567890-02-28")
            .unwrap();
        assert_eq!(
            dom.form_control_value(input).as_deref(),
            Some("12345678901234567890-02-28")
        );
        dom.set_attribute(input, attr(local_name!("type")), "week");
        dom.set_form_control_value(input, "2015-W53").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some("2015-W53"));
        dom.set_form_control_value(input, "2014-W53").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some(""));
        dom.set_form_control_value(input, "12345678901234567890-W52")
            .unwrap();
        assert_eq!(
            dom.form_control_value(input).as_deref(),
            Some("12345678901234567890-W52")
        );
        dom.set_attribute(input, attr(local_name!("type")), "datetime-local");
        dom.set_form_control_value(input, "2024-02-29 12:30:00.5000")
            .unwrap();
        assert_eq!(
            dom.form_control_value(input).as_deref(),
            Some("2024-02-29T12:30:00.5")
        );
    }

    #[test]
    fn range_sanitization_uses_default_midpoint_and_tracks_changed_bounds() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(tag(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "range");
        assert_eq!(dom.form_control_value(input).as_deref(), Some("50"));
        dom.set_attribute(input, attr(local_name!("min")), "10");
        dom.set_attribute(input, attr(local_name!("max")), "20");
        assert_eq!(dom.form_control_value(input).as_deref(), Some("20"));
        dom.set_form_control_value(input, "bad").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some("15"));
        dom.set_attribute(input, attr(local_name!("step")), "2");
        dom.set_form_control_value(input, "16").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some("16"));
        dom.set_attribute(input, attr(local_name!("max")), "14");
        assert_eq!(dom.form_control_value(input).as_deref(), Some("14"));
        dom.set_attribute(input, attr(local_name!("min")), "20");
        assert_eq!(dom.form_control_value(input).as_deref(), Some("20"));
        dom.set_attribute(input, attr(local_name!("min")), "0");
        dom.set_attribute(input, attr(local_name!("max")), "5");
        dom.set_attribute(input, attr(local_name!("step")), "3");
        dom.set_form_control_value(input, "5").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some("3"));
    }

    #[test]
    fn radio_group_uses_form_owner_and_name() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let form_a = dom.create_element(tag(local_name!("form")));
        let form_b = dom.create_element(tag(local_name!("form")));
        dom.set_attribute(form_a, attr(local_name!("id")), "a");
        dom.set_attribute(form_b, attr(local_name!("id")), "b");
        let make_radio = |dom: &mut ScriptedDom, form: &str| {
            let input = dom.create_element(tag(local_name!("input")));
            dom.set_attribute(input, attr(local_name!("type")), "radio");
            dom.set_attribute(input, attr(local_name!("name")), "group");
            dom.set_attribute(input, attr(local_name!("form")), form);
            input
        };
        let first = make_radio(&mut dom, "a");
        let second = make_radio(&mut dom, "a");
        let other_form = make_radio(&mut dom, "b");
        dom.append_child(root, form_a);
        dom.append_child(root, form_b);
        dom.append_child(root, first);
        dom.append_child(root, second);
        dom.append_child(root, other_form);
        assert!(dom.set_form_control_checked(first, true));
        assert!(dom.set_form_control_checked(other_form, true));
        assert!(dom.form_control_state(first).unwrap().checked);
        assert!(dom.form_control_state(other_form).unwrap().checked);
        assert!(dom.set_form_control_checked(second, true));
        assert!(!dom.form_control_state(first).unwrap().checked);
        assert!(dom.form_control_state(second).unwrap().checked);
        assert!(dom.form_control_state(other_form).unwrap().checked);
    }

    #[test]
    fn clone_copies_only_the_html_cloning_step_state() {
        let mut dom = ScriptedDom::new();
        let source = dom.create_element(tag(local_name!("input")));
        let target = dom.create_element(tag(local_name!("input")));
        dom.set_form_control_value(source, "current").unwrap();
        let mut state = dom.form_control_clone_state(source).unwrap();
        state.checked = true;
        state.dirty_checkedness = true;
        state.selection_start = Some(3);
        state.selection_end = Some(4);
        state.custom_validity_message = "copy?".into();
        assert!(dom.set_form_control_state(source, state));

        assert!(dom.clone_form_control_state(source, target));
        let copied = dom.form_control_clone_state(target).unwrap();
        assert_eq!(copied.value, "current");
        assert!(copied.dirty_value);
        assert!(copied.checked);
        assert!(copied.dirty_checkedness);
        assert_eq!(copied.selection_start, Some(0));
        assert_eq!(copied.selection_end, Some(0));
        assert!(copied.custom_validity_message.is_empty());
    }

    #[test]
    fn form_values_reads_current_input_and_textarea_values() {
        let mut dom = ScriptedDom::new();
        let form = dom.create_element(tag(local_name!("form")));
        let input = dom.create_element(tag(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("name")), "q");
        dom.set_attribute(input, attr(local_name!("value")), "default");
        dom.append_child(form, input);
        dom.set_form_control_value(input, "current").unwrap();
        assert_eq!(
            dom.form_values(form),
            vec![("q".to_owned(), "current".to_owned())]
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InputValueModeForState {
    Value,
    Default,
    DefaultOn,
    Filename,
}
type InputValueMode = InputValueModeForState;

fn value_mode(kind: &str) -> InputValueMode {
    match kind.to_ascii_lowercase().as_str() {
        "checkbox" | "radio" => InputValueMode::DefaultOn,
        "button" | "reset" | "submit" | "image" => InputValueMode::Default,
        "file" => InputValueMode::Filename,
        _ => InputValueMode::Value,
    }
}
pub(super) fn value_mode_for_state(kind: &str) -> InputValueModeForState {
    value_mode(kind)
}
pub(super) fn sanitize_value_for_state(
    kind: &str,
    value: &str,
    min: Option<&str>,
    max: Option<&str>,
    step: Option<&str>,
    multiple: bool,
    step_base: Option<&str>,
) -> String {
    sanitize_input_value_with_options(kind, value, min, max, step, multiple, step_base)
}
pub(super) fn clamp_selection_for_state(state: &mut FormControlState, kind: &str) {
    if !tracks_internal_selection(kind) {
        state.selection_start = None;
        state.selection_end = None;
        state.selection_direction = SelectionDirection::None;
        return;
    }
    let length = utf16_len(&api_value(&state.value));
    let start = state.selection_start.unwrap_or(0).min(length);
    let end = state.selection_end.unwrap_or(start).min(length).max(start);
    state.selection_start = Some(start);
    state.selection_end = Some(end);
}
impl ScriptedDom {
    pub(super) fn input_type_for_state(&self, id: NodeId) -> String {
        self.input_type(id)
    }
}
fn collect_descendants(dom: &ScriptedDom, node: NodeId, out: &mut Vec<NodeId>) {
    out.push(node);
    for child in dom.dom_children(node) {
        collect_descendants(dom, child, out);
    }
}
fn attr_name(local: &str) -> QualName {
    QualName::new(None, markup5ever::ns!(), LocalName::from(local))
}
fn api_value(raw: &str) -> String {
    raw.replace("\r\n", "\n").replace('\r', "\n")
}
pub(super) fn api_value_for_state(raw: &str) -> String {
    api_value(raw)
}
fn utf16_len(value: &str) -> u32 {
    value.encode_utf16().count().min(u32::MAX as usize) as u32
}
fn collapse_selection(state: &mut FormControlState, value: &str) {
    let end = utf16_len(&api_value(value));
    state.selection_start = Some(end);
    state.selection_end = Some(end);
    state.selection_direction = SelectionDirection::None;
}
fn set_selection_support(state: &mut FormControlState, kind: &str, initialize: bool) {
    if tracks_internal_selection(kind) {
        if initialize && state.selection_start.is_none() {
            state.selection_start = Some(0);
            state.selection_end = Some(0);
        }
    } else {
        state.selection_start = None;
        state.selection_end = None;
        state.selection_direction = SelectionDirection::None;
    }
}

fn tracks_internal_selection(kind: &str) -> bool {
    kind == "textarea"
        || matches!(
            kind,
            "text"
                | "search"
                | "tel"
                | "url"
                | "password"
                | "email"
                | "date"
                | "month"
                | "week"
                | "time"
                | "datetime-local"
                | "number"
                | "range"
                | "color"
        )
}

fn sanitize_input_value_with_options(
    kind: &str,
    value: &str,
    min: Option<&str>,
    max: Option<&str>,
    step: Option<&str>,
    multiple: bool,
    step_base: Option<&str>,
) -> String {
    match kind {
        "text" | "search" | "tel" | "password" => value.replace(['\r', '\n'], ""),
        "url" => value
            .replace(['\r', '\n'], "")
            .trim_matches(|c: char| c.is_ascii_whitespace())
            .to_owned(),
        "email" => {
            let value = value.replace(['\r', '\n'], "");
            if multiple {
                value
                    .split(',')
                    .map(|token| token.trim_matches(|c: char| c.is_ascii_whitespace()))
                    .collect::<Vec<_>>()
                    .join(",")
            } else {
                value
                    .trim_matches(|c: char| c.is_ascii_whitespace())
                    .to_owned()
            }
        },
        "hidden" | "submit" | "reset" | "button" | "image" | "checkbox" | "radio" | "file" => {
            value.to_owned()
        },
        "number" => {
            if parse_html_float(value).is_some() {
                value.to_owned()
            } else {
                String::new()
            }
        },
        "range" => {
            let low = min.and_then(parse_html_float).unwrap_or(0.0);
            let high = max.and_then(parse_html_float).unwrap_or(100.0);
            let midpoint = match (
                min.and_then(parse_html_float),
                max.and_then(parse_html_float),
            ) {
                (Some(min), Some(max)) if min < max => min / 2.0 + max / 2.0,
                (Some(min), Some(max)) if max < min => min,
                (Some(min), _) => min / 2.0 + high / 2.0,
                (_, Some(max)) => low / 2.0 + max / 2.0,
                _ => 50.0,
            };
            let mut n = parse_html_float(value).unwrap_or(midpoint);
            if n < low {
                n = low;
            }
            if high >= low && n > high {
                n = high;
            }
            let allowed_step = if step.is_some_and(|s| s.eq_ignore_ascii_case("any")) {
                None
            } else {
                Some(
                    step.and_then(parse_html_float)
                        .filter(|s| *s > 0.0)
                        .unwrap_or(1.0),
                )
            };
            if let Some(step) = allowed_step {
                let base = min
                    .and_then(parse_html_float)
                    .or_else(|| step_base.and_then(parse_html_float))
                    .unwrap_or(0.0);
                let offset = (n - base) / step;
                let lower = offset.floor();
                let fraction = offset - lower;
                let rounded = if fraction >= 0.5 { lower + 1.0 } else { lower };
                let candidate = base + rounded * step;
                if candidate >= low && (high < low || candidate <= high) {
                    n = candidate;
                } else {
                    // If rounding points outside the bounds, use the nearest
                    // admitted step instead of retaining a step mismatch.
                    let bounded = if candidate < low {
                        base + ((low - base) / step).ceil() * step
                    } else {
                        base + ((high - base) / step).floor() * step
                    };
                    if bounded.is_finite() && bounded >= low && (high < low || bounded <= high) {
                        n = bounded;
                    }
                }
            }
            n.to_string()
        },
        "color" => {
            if valid_color(value) {
                value.to_ascii_lowercase()
            } else {
                "#000000".to_owned()
            }
        },
        "date" => {
            if valid_date(value) {
                value.to_owned()
            } else {
                String::new()
            }
        },
        "month" => {
            if valid_month(value) {
                value.to_owned()
            } else {
                String::new()
            }
        },
        "week" => {
            if valid_week(value) {
                value.to_owned()
            } else {
                String::new()
            }
        },
        "time" => {
            if valid_time(value) {
                value.to_owned()
            } else {
                String::new()
            }
        },
        "datetime-local" => normalize_local_datetime(value).unwrap_or_default(),
        _ => value.to_owned(),
    }
}

fn is_html_float(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut i = 0;
    if bytes.get(i) == Some(&b'-') {
        i += 1;
    }
    let before = digits(bytes, &mut i);
    let has_dot = bytes.get(i) == Some(&b'.');
    let after = if has_dot {
        i += 1;
        digits(bytes, &mut i)
    } else {
        false
    };
    if has_dot && !after {
        return false;
    }
    if !before && !after {
        return false;
    }
    if bytes.get(i).is_some_and(|b| *b == b'e' || *b == b'E') {
        i += 1;
        if bytes.get(i).is_some_and(|b| *b == b'+' || *b == b'-') {
            i += 1;
        }
        if !digits(bytes, &mut i) {
            return false;
        }
    }
    i == bytes.len()
}
fn digits(bytes: &[u8], i: &mut usize) -> bool {
    let start = *i;
    while bytes.get(*i).is_some_and(u8::is_ascii_digit) {
        *i += 1;
    }
    *i > start
}
fn parse_html_float(value: &str) -> Option<f64> {
    (is_html_float(value))
        .then(|| value.parse().ok())
        .flatten()
        .filter(|n: &f64| n.is_finite())
}
fn valid_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|b| b.is_ascii_hexdigit())
}
fn valid_month(s: &str) -> bool {
    let Some((year, month)) = s.split_once('-') else {
        return false;
    };
    year.len() >= 4
        && year.bytes().all(|b| b.is_ascii_digit())
        && year.trim_start_matches('0') != ""
        && month.len() == 2
        && month.bytes().all(|b| b.is_ascii_digit())
        && (1..=12).contains(&month.parse::<u8>().unwrap_or(0))
}
fn valid_date(s: &str) -> bool {
    let mut parts = s.split('-');
    let Some(year_text) = parts.next() else {
        return false;
    };
    let Some(month_text) = parts.next() else {
        return false;
    };
    let Some(day_text) = parts.next() else {
        return false;
    };
    if parts.next().is_some()
        || year_text.len() < 4
        || !year_text.bytes().all(|b| b.is_ascii_digit())
        || year_text.trim_start_matches('0').is_empty()
        || month_text.len() != 2
        || !month_text.bytes().all(|b| b.is_ascii_digit())
        || day_text.len() != 2
        || !day_text.bytes().all(|b| b.is_ascii_digit())
    {
        return false;
    }
    let year_mod_400 = decimal_modulo(year_text, 400);
    let month = month_text.parse::<u32>().unwrap_or(0);
    let day = day_text.parse::<u32>().unwrap_or(0);
    if !has_nonzero_decimal(year_text) || !(1..=12).contains(&month) {
        return false;
    }
    let leap = year_mod_400 % 4 == 0 && year_mod_400 % 100 != 0 || year_mod_400 == 0;
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    day > 0 && day <= days[(month - 1) as usize]
}
fn valid_week(s: &str) -> bool {
    let Some((year_text, week_text)) = s.split_once("-W") else {
        return false;
    };
    if year_text.len() < 4
        || !year_text.bytes().all(|b| b.is_ascii_digit())
        || year_text.trim_start_matches('0').is_empty()
        || week_text.len() != 2
        || !week_text.bytes().all(|b| b.is_ascii_digit())
    {
        return false;
    }
    let week = week_text.parse::<u8>().unwrap_or(0);
    has_nonzero_decimal(year_text)
        && week > 0
        && week <= if year_has_53_weeks(year_text) { 53 } else { 52 }
}
fn year_has_53_weeks(year: &str) -> bool {
    // Week-year shape repeats every 400 Gregorian years; parsing the whole
    // year into a machine integer would reject otherwise-valid long years.
    let year_mod_400 = decimal_modulo(year, 400);
    let prior = if year_mod_400 == 0 {
        399
    } else {
        year_mod_400 - 1
    };
    let weekday = (365 * prior + prior / 4 - prior / 100 + prior / 400) % 7;
    let leap = year_mod_400 % 4 == 0 && year_mod_400 % 100 != 0 || year_mod_400 == 0;
    weekday == 3 || (weekday == 2 && leap) // Monday=0; Thursday or leap-year Wednesday.
}
fn decimal_modulo(value: &str, modulus: u32) -> u32 {
    value.bytes().fold(0, |remainder, digit| {
        (remainder * 10 + u32::from(digit - b'0')) % modulus
    })
}
fn has_nonzero_decimal(value: &str) -> bool {
    value.bytes().any(|digit| digit != b'0')
}
fn valid_time(s: &str) -> bool {
    let (clock, fraction) = s.split_once('.').unwrap_or((s, ""));
    if s.contains('.')
        && (clock.matches(':').count() != 2
            || fraction.is_empty()
            || !fraction.bytes().all(|b| b.is_ascii_digit()))
    {
        return false;
    }
    let p: Vec<_> = clock.split(':').collect();
    if !(2..=3).contains(&p.len())
        || p.iter()
            .any(|x| x.len() != 2 || !x.bytes().all(|b| b.is_ascii_digit()))
    {
        return false;
    }
    let h = p[0].parse::<u8>().unwrap_or(255);
    let m = p[1].parse::<u8>().unwrap_or(255);
    let sec = if p.len() == 3 {
        p[2].parse::<u8>().unwrap_or(255)
    } else {
        0
    };
    h < 24 && m < 60 && sec < 60
}
fn normalize_local_datetime(s: &str) -> Option<String> {
    let (date, time) = s.split_once('T').or_else(|| s.split_once(' '))?;
    if !valid_date(date) || !valid_time(time) {
        return None;
    }
    let (clock, fraction) = time.split_once('.').unwrap_or((time, ""));
    let mut parts = clock.split(':');
    let hour = parts.next()?;
    let minute = parts.next()?;
    let second = parts.next();
    let normalized_time = match second {
        None => format!("{hour}:{minute}"),
        Some(second) => {
            let fraction = fraction.trim_end_matches('0');
            if second == "00" && fraction.is_empty() {
                format!("{hour}:{minute}")
            } else if fraction.is_empty() {
                format!("{hour}:{minute}:{second}")
            } else {
                format!("{hour}:{minute}:{second}.{fraction}")
            }
        },
    };
    Some(format!("{date}T{normalized_time}"))
}
