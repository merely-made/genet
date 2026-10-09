// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! HTML input and textarea current-value state.

use layout_dom_api::{
    FormControlState, LayoutDom, LayoutDomMut, LocalName, QualName, SelectOptionState,
    SelectionDirection,
};

use crate::{NodeId, ScriptedDom};

/// A short-lived view of one checked radio's association before a DOM
/// mutation. This is intentionally not stored on the node or serialized.
#[derive(Clone, Copy)]
pub(super) struct RadioAssociationSnapshot {
    node: NodeId,
    form_owner: Option<NodeId>,
    connected: bool,
}

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
    /// Inputs, textareas, and selects use their current arena state.
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
                            "input" | "select" | "textarea" => {
                                self.form_control_value(id).unwrap_or_default()
                            },
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

    /// HTML-aware option text, with ASCII whitespace stripped and collapsed.
    pub fn option_text(&self, id: NodeId) -> Option<String> {
        (self.html_name(id) == Some("option")).then(|| collect_option_text(self, id))
    }

    /// An explicit value stays exact; otherwise HTML-aware text supplies it.
    pub fn option_value(&self, id: NodeId) -> Option<String> {
        if self.html_name(id) != Some("option") {
            return None;
        }
        Some(
            self.attribute(id, &markup5ever::ns!(), &LocalName::from("value"))
                .map(str::to_owned)
                .unwrap_or_else(|| collect_option_text(self, id)),
        )
    }

    pub fn form_control_value(&self, id: NodeId) -> Option<String> {
        let node = self.node(id);
        let name = node.name.as_ref()?;
        if name.ns != markup5ever::ns!(html) {
            return None;
        }
        match name.local.as_ref() {
            "textarea" => Some(api_value(&self.node(id).form_control.as_ref()?.value)),
            "select" => {
                let selected = self.select_options(id).into_iter().find(|&option| {
                    self.node(option)
                        .option_state
                        .is_some_and(|state| state.selected)
                })?;
                self.option_value(selected)
            },
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
                self.clear_user_edit_state(id);
                Ok(())
            },
            "select" => {
                let options = self.select_options(id);
                let index = options
                    .iter()
                    .position(|&option| self.option_value(option).as_deref() == Some(value))
                    .map_or(-1, |index| index.min(i32::MAX as usize) as i32);
                self.set_select_selected_index(id, index);
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
                    self.clear_user_edit_state(id);
                    Ok(())
                },
                InputValueMode::Default | InputValueMode::DefaultOn => {
                    self.set_attribute(id, attr_name("value"), value);
                    self.clear_user_edit_state(id);
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
                    self.clear_user_edit_state(id);
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
                self.reset_interaction_state(id);
                true
            },
            "textarea" => {
                let default = self.textarea_default_value(id);
                let state = self.node_mut(id).form_control.as_mut().unwrap();
                state.value = default;
                state.dirty_value = false;
                self.mutations
                    .push(crate::DomMutation::FormControlStateChanged { node: id });
                self.reset_interaction_state(id);
                true
            },
            "select" => {
                for option in self.select_options(id) {
                    let selected = self
                        .attribute(option, &markup5ever::ns!(), &LocalName::from("selected"))
                        .is_some();
                    self.set_option_selected_state(
                        option,
                        SelectOptionState {
                            selected,
                            dirty: false,
                        },
                    );
                }
                self.set_select_selectedness(id);
                self.reset_interaction_state(id);
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
        {
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
            dst.custom_validity_message.clear();
        }
        self.reset_interaction_state(target);
        true
    }

    /// HTML's option cloning steps propagate selectedness and dirtiness.
    pub fn clone_option_state(&mut self, source: NodeId, target: NodeId) -> bool {
        if !self.is_html_option(source) || !self.is_html_option(target) {
            return false;
        }
        let Some(state) = self.node(source).option_state else {
            return false;
        };
        let Some(target_state) = self.node_mut(target).option_state.as_mut() else {
            return false;
        };
        *target_state = state;
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
        if name.ns != markup5ever::ns!() {
            return;
        }
        if self.is_html_option(id) && name.local.as_ref() == "selected" {
            if !self.node(id).option_state.is_some_and(|state| state.dirty) {
                let selected = self
                    .attribute(id, &markup5ever::ns!(), &LocalName::from("selected"))
                    .is_some();
                self.set_option_selected(id, selected, false);
            }
            if let Some(select) = self.option_select_ancestor(id) {
                self.set_select_selectedness(select);
            }
        }
        if self.html_name(id) == Some("select") {
            match name.local.as_ref() {
                "size" => {
                    self.set_select_selectedness(id);
                },
                "multiple" if !self.has_input_attribute(id, "multiple") => {
                    let selected = self
                        .select_options(id)
                        .into_iter()
                        .filter(|&option| {
                            self.node(option)
                                .option_state
                                .is_some_and(|state| state.selected)
                        })
                        .collect::<Vec<_>>();
                    for option in selected.into_iter().skip(1) {
                        self.set_option_selected(option, false, false);
                    }
                    self.set_select_selectedness(id);
                },
                _ => {},
            }
        }
        if name.local.as_ref() == "disabled"
            && (self.is_html_option(id) || self.html_name(id) == Some("optgroup"))
            && let Some(select) = self.option_select_ancestor(id)
        {
            self.set_select_selectedness(select);
        }
        if !self.is_html_input(id) {
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
                let old_type = normalized_input_type(old_type.unwrap_or("text"));
                let old_mode = value_mode(&old_type);
                let new_type = self.input_type(id).to_owned();
                let new_mode = value_mode(&new_type);
                let old_api_value = match old_mode {
                    InputValueMode::Value => {
                        self.node(id).form_control.as_ref().unwrap().value.clone()
                    },
                    InputValueMode::Default => {
                        self.input_attribute(id, "value").unwrap_or("").to_owned()
                    },
                    InputValueMode::DefaultOn => {
                        self.input_attribute(id, "value").unwrap_or("on").to_owned()
                    },
                    InputValueMode::Filename => String::new(),
                };
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
                let value_unchanged =
                    self.form_control_value(id).as_deref() == Some(old_api_value.as_str());
                set_selection_support(
                    self.node_mut(id).form_control.as_mut().unwrap(),
                    &new_type,
                    true,
                );
                if new_type == "radio" && self.node(id).form_control.as_ref().unwrap().checked {
                    self.uncheck_radio_group(id);
                }
                if old_type != new_type {
                    // A genuine type-state change replaces the old editing UI.
                    // Drop any raw, unconvertible draft and its focus baseline,
                    // while retaining user-validity and custom validity.
                    self.clear_user_edit_state_preserving_provenance(id, value_unchanged);
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
        let select = if self.html_name(node) == Some("select") {
            Some(node)
        } else {
            self.option_select_ancestor(node)
        };
        if let Some(select) = select {
            self.set_select_selectedness(select);
        }
    }

    /// Apply the select-list insertion rule to selected options in an inserted
    /// subtree. Moving within one option list preserves its existing selection;
    /// joining a different list makes newly added selected options take effect.
    pub(super) fn form_control_option_subtree_inserted(
        &mut self,
        subtree: NodeId,
        former_select: Option<NodeId>,
    ) {
        let Some(select) = self.option_select_ancestor(subtree) else {
            return;
        };
        if former_select == Some(select) {
            return;
        }

        let inserted_selected_options = self
            .select_options(select)
            .into_iter()
            .filter(|&option| {
                let mut ancestor = Some(option);
                while let Some(id) = ancestor {
                    if id == subtree {
                        return true;
                    }
                    ancestor = self.parent(id);
                }
                false
            })
            .filter(|&option| {
                self.option_selected_state(option)
                    .is_some_and(|state| state.selected)
            })
            .collect::<Vec<_>>();
        for option in inserted_selected_options {
            // Selectedness changes from adding an option do not dirty it.
            self.set_option_selected(option, true, false);
        }
    }

    /// HTML's select display size. A parsed zero remains zero; only an absent
    /// or invalid size uses the select type's fallback.
    pub(super) fn select_display_size_is_one(&self, select: NodeId) -> bool {
        self.attribute(select, &markup5ever::ns!(), &LocalName::from("size"))
            .and_then(parse_html_nonnegative_integer_is_one)
            .unwrap_or_else(|| !self.has_input_attribute(select, "multiple"))
    }

    /// Normalize a single-select's selectedness and select the first enabled
    /// option only for display size 1 when no option is selected.
    pub(super) fn set_select_selectedness(&mut self, select: NodeId) -> bool {
        if self.html_name(select) != Some("select") || self.has_input_attribute(select, "multiple")
        {
            return false;
        }
        let options = self.select_options(select);
        let mut last_selected = None;
        let mut first_enabled = None;
        let mut changed = false;
        for &option in &options {
            if self
                .node(option)
                .option_state
                .is_some_and(|state| state.selected)
            {
                if let Some(previous) = last_selected {
                    self.set_option_selected(previous, false, false);
                    changed = true;
                }
                last_selected = Some(option);
            }
            if first_enabled.is_none() && !self.option_is_disabled(option) {
                first_enabled = Some(option);
            }
        }
        if last_selected.is_none() && self.select_display_size_is_one(select) {
            if let Some(option) = first_enabled {
                self.set_option_selected(option, true, false);
                changed = true;
            }
        }
        changed
    }

    pub(super) fn option_is_disabled(&self, option: NodeId) -> bool {
        if self.has_input_attribute(option, "disabled") {
            return true;
        }
        let mut ancestor = self.parent(option);
        while let Some(id) = ancestor {
            match self.html_name(id) {
                Some("select" | "hr" | "datalist" | "option") => return false,
                Some("optgroup") => return self.has_input_attribute(id, "disabled"),
                _ => ancestor = self.parent(id),
            }
        }
        false
    }

    pub(super) fn option_select_ancestor(&self, option: NodeId) -> Option<NodeId> {
        let mut ancestor = self.parent(option);
        let mut optgroup = false;
        while let Some(id) = ancestor {
            match self.html_name(id) {
                Some("select") => return Some(id),
                Some("hr" | "datalist" | "option") => return None,
                Some("optgroup") => {
                    if optgroup {
                        return None;
                    }
                    optgroup = true;
                },
                _ => {},
            }
            ancestor = self.parent(id);
        }
        None
    }

    /// Capture checked radios in the affected ordinary trees, traversing each
    /// host's shadow root as another tree. Call before topology or `id` changes.
    pub(super) fn snapshot_radio_associations(
        &self,
        seeds: &[NodeId],
    ) -> Vec<RadioAssociationSnapshot> {
        let mut stack = Vec::new();
        for seed in seeds.iter().rev().copied() {
            if self.try_index(seed).is_some() {
                stack.push(self.form_control_tree_root(seed));
            }
        }
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        while let Some(root) = stack.pop() {
            if self.try_index(root).is_none() || !seen.insert(root) {
                continue;
            }
            let mut descendants = Vec::new();
            collect_descendants(self, root, &mut descendants);
            for id in descendants {
                if self.is_html_input(id)
                    && self.input_type(id) == "radio"
                    && self
                        .node(id)
                        .form_control
                        .as_ref()
                        .is_some_and(|state| state.checked)
                {
                    out.push(RadioAssociationSnapshot {
                        node: id,
                        form_owner: self.form_control_form_owner(id),
                        connected: self.is_connected_through_shadow_hosts(id),
                    });
                }
                if let Some(shadow_root) = self.shadow_root(id) {
                    stack.push(shadow_root);
                }
            }
        }
        out
    }

    /// Reconcile only checked radios whose form owner changed or which became
    /// connected. Reverse post-tree order gives the last eligible checked radio
    /// first claim; rechecking live checkedness prevents an earlier candidate
    /// from undoing that winner after it has already been unchecked.
    pub(super) fn reconcile_radio_associations(
        &mut self,
        before: &[RadioAssociationSnapshot],
        seeds: &[NodeId],
    ) {
        let old: std::collections::HashMap<_, _> =
            before.iter().map(|item| (item.node, *item)).collect();
        let after = self.snapshot_radio_associations(seeds);
        for current in after.into_iter().rev() {
            // A remove/release may retire a snapshotted node. The post scan
            // checks roots, and this guard also protects stale candidate IDs.
            if self.try_index(current.node).is_none() {
                continue;
            }
            if !self
                .node(current.node)
                .form_control
                .as_ref()
                .is_some_and(|state| state.checked)
            {
                continue;
            }
            let triggered = match old.get(&current.node) {
                Some(previous) => {
                    previous.form_owner != current.form_owner
                        || (!previous.connected && current.connected)
                },
                None => current.form_owner.is_some() || current.connected,
            };
            if triggered {
                self.uncheck_radio_group(current.node);
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
        normalized_input_type(self.input_attribute(id, "type").unwrap_or("text"))
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

    /// The current form owner of an HTML input or textarea. Explicit `form`
    /// lookup is scoped to the control's ordinary tree; it crosses a shadow
    /// host only when deciding whether the control is connected.
    pub fn form_control_form_owner(&self, id: NodeId) -> Option<NodeId> {
        if !self.is_listed_control(id) || self.html_name(id) == Some("form") {
            return None;
        }
        let root = self.form_control_tree_root(id);
        self.input_form_owner(id, root)
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
        let root = self.form_control_tree_root(id);
        let form_owner = self.form_control_form_owner(id);
        let mut descendants = Vec::new();
        collect_descendants(self, root, &mut descendants);
        let peers: Vec<_> = descendants
            .into_iter()
            .filter(|peer| {
                *peer != id
                    && self.is_html_input(*peer)
                    && self.input_type(*peer) == "radio"
                    && self.input_attribute(*peer, "name") == Some(name.as_str())
                    && self.form_control_form_owner(*peer) == form_owner
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
        if let Some(form_id) = self.attribute(id, &markup5ever::ns!(), &LocalName::from("form"))
            && self.is_connected_through_shadow_hosts(id)
        {
            let mut nodes = Vec::new();
            collect_descendants(self, root, &mut nodes);
            let first_match = nodes.into_iter().find(|candidate| {
                self.attribute(*candidate, &markup5ever::ns!(), &LocalName::from("id"))
                    == Some(form_id)
            });
            return first_match.filter(|candidate| {
                self.element_name(*candidate)
                    .is_some_and(|q| q.ns == markup5ever::ns!(html) && q.local.as_ref() == "form")
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
    fn is_connected_through_shadow_hosts(&self, id: NodeId) -> bool {
        let mut current = id;
        loop {
            if self.kind(current) == layout_dom_api::NodeKind::Document {
                return true;
            }
            if let Some(parent) = self.parent(current) {
                current = parent;
            } else if let Some(host) = self.shadow_host_of(current) {
                current = host;
            } else {
                return false;
            }
        }
    }
    pub(super) fn form_control_tree_root(&self, id: NodeId) -> NodeId {
        let mut root = id;
        while let Some(parent) = self.parent(root) {
            root = parent;
        }
        root
    }

    pub(super) fn is_html_option(&self, id: NodeId) -> bool {
        self.element_name(id)
            .is_some_and(|q| q.ns == markup5ever::ns!(html) && q.local.as_ref() == "option")
    }

    pub(super) fn clear_user_edit_state(&mut self, id: NodeId) {
        self.clear_user_edit_state_preserving_provenance(id, false);
    }

    fn clear_user_edit_state_preserving_provenance(
        &mut self,
        id: NodeId,
        preserve_provenance: bool,
    ) {
        if let Some(state) = self.node_mut(id).form_interaction.as_mut() {
            let changed = (!preserve_provenance && state.last_change_by_user)
                || state.user_edit_pending
                || state.bad_input
                || state.draft_value.is_some()
                || state.draft_selection_start.is_some()
                || state.draft_selection_end.is_some()
                || state.draft_selection_direction != SelectionDirection::None
                || state.user_edit_initial_value.is_some();
            if !preserve_provenance {
                state.last_change_by_user = false;
            }
            state.user_edit_pending = false;
            state.user_edit_initial_value = None;
            state.bad_input = false;
            state.draft_value = None;
            state.draft_selection_start = None;
            state.draft_selection_end = None;
            state.draft_selection_direction = SelectionDirection::None;
            if changed {
                self.mutations
                    .push(crate::DomMutation::FormControlInteractionStateChanged { node: id });
            }
        }
    }
}

fn collect_option_text(dom: &ScriptedDom, root: NodeId) -> String {
    let mut stack = vec![root];
    let mut out = String::new();
    while let Some(id) = stack.pop() {
        if dom.element_name(id).is_some_and(|name| {
            (name.ns == markup5ever::ns!(html) && matches!(name.local.as_ref(), "script" | "img"))
                || (name.ns == markup5ever::ns!(svg) && name.local.as_ref() == "script")
        }) {
            continue;
        }
        if matches!(
            dom.kind(id),
            layout_dom_api::NodeKind::Text | layout_dom_api::NodeKind::CdataSection
        ) {
            out.push_str(dom.text(id).unwrap_or(""));
        } else {
            stack.extend(dom.dom_children(id).collect::<Vec<_>>().into_iter().rev());
        }
    }
    out.split(|c| matches!(c, ' ' | '\t' | '\n' | '\u{000c}' | '\r'))
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
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
    fn form_association_walk_preserves_preorder_on_a_small_stack() {
        std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(|| {
                let mut dom = ScriptedDom::new();
                let root = dom.document();
                let mut expected = vec![root];
                let mut parent = root;
                for _ in 0..4096 {
                    let child = dom.create_element(tag(local_name!("div")));
                    dom.attach_silent(parent, child);
                    expected.push(child);
                    parent = child;
                }
                let sibling = dom.create_element(tag(local_name!("input")));
                dom.attach_silent(root, sibling);
                expected.push(sibling);
                let mut actual = Vec::new();
                collect_descendants(&dom, root, &mut actual);
                assert_eq!(actual, expected);
            })
            .expect("the small-stack walker thread starts")
            .join()
            .expect("the tree walk completes without using depth-sized stack space");
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
    fn range_decimal_steps_do_not_add_binary_arithmetic_tails() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(tag(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "range");
        dom.set_attribute(input, attr(local_name!("min")), "0");
        dom.set_attribute(input, attr(local_name!("max")), "1");
        dom.set_attribute(input, attr(local_name!("step")), ".1");
        for (value, expected) in [
            (".6", "0.6"),
            (".3", "0.3"),
            ("6e-1", "0.6"),
            (".61", "0.6"),
        ] {
            dom.set_form_control_value(input, value).unwrap();
            assert_eq!(dom.form_control_value(input).as_deref(), Some(expected));
        }
        dom.set_attribute(input, attr(local_name!("min")), "-.2");
        dom.set_form_control_value(input, "-.09").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some("-0.1"));
        dom.set_attribute(input, attr(local_name!("max")), ".36");
        dom.set_form_control_value(input, "1").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some("0.3"));
        dom.set_attribute(input, attr(local_name!("min")), "0");
        dom.set_attribute(input, attr(local_name!("max")), "1");
        dom.set_attribute(input, attr(local_name!("step")), ".29");
        dom.set_form_control_value(input, ".88").unwrap();
        assert_eq!(dom.form_control_value(input).as_deref(), Some("0.87"));
        dom.set_attribute(input, attr(local_name!("min")), "1000000000000000");
        dom.set_attribute(input, attr(local_name!("max")), "1000000000000002");
        dom.set_attribute(input, attr(local_name!("step")), ".1");
        dom.set_form_control_value(input, "1000000000000000.1")
            .unwrap();
        assert_eq!(
            dom.form_control_value(input).as_deref(),
            Some("1000000000000000.1")
        );
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
    fn explicit_form_uses_first_matching_id_even_when_it_is_not_a_form() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let earlier_div = dom.create_element(tag(local_name!("div")));
        dom.set_attribute(earlier_div, attr(local_name!("id")), "duplicate");
        let later_form = dom.create_element(tag(local_name!("form")));
        dom.set_attribute(later_form, attr(local_name!("id")), "duplicate");
        let explicit = dom.create_element(tag(local_name!("input")));
        let owned = dom.create_element(tag(local_name!("input")));
        for radio in [explicit, owned] {
            dom.set_attribute(radio, attr(local_name!("type")), "radio");
            dom.set_attribute(radio, attr(local_name!("name")), "group");
        }
        dom.set_attribute(explicit, attr(local_name!("form")), "duplicate");
        dom.append_child(root, earlier_div);
        dom.append_child(root, later_form);
        dom.append_child(root, explicit);
        dom.append_child(later_form, owned);

        assert_eq!(dom.form_control_form_owner(explicit), None);
        assert_eq!(dom.form_control_form_owner(owned), Some(later_form));
        assert!(dom.set_form_control_checked(explicit, true));
        assert!(dom.set_form_control_checked(owned, true));
        assert!(dom.form_control_state(explicit).unwrap().checked);
        assert!(dom.form_control_state(owned).unwrap().checked);
    }

    #[test]
    fn disconnected_form_attribute_falls_back_to_nearest_ancestor_form() {
        let mut dom = ScriptedDom::new();
        let fragment = dom.create_fragment();
        let ancestor = dom.create_element(tag(local_name!("form")));
        dom.set_attribute(ancestor, attr(local_name!("id")), "ancestor");
        let external = dom.create_element(tag(local_name!("form")));
        dom.set_attribute(external, attr(local_name!("id")), "external");
        let detached_radio = dom.create_element(tag(local_name!("input")));
        let external_radio = dom.create_element(tag(local_name!("input")));
        let external_textarea = dom.create_element(tag(local_name!("textarea")));
        for radio in [detached_radio, external_radio] {
            dom.set_attribute(radio, attr(local_name!("type")), "radio");
            dom.set_attribute(radio, attr(local_name!("name")), "group");
        }
        dom.set_attribute(detached_radio, attr(local_name!("form")), "external");
        dom.append_child(fragment, ancestor);
        dom.append_child(ancestor, detached_radio);
        dom.append_child(fragment, external);
        dom.append_child(external, external_radio);
        dom.append_child(external, external_textarea);

        assert_eq!(dom.form_control_form_owner(detached_radio), Some(ancestor));
        assert_eq!(dom.form_control_form_owner(external_radio), Some(external));
        assert_eq!(
            dom.form_control_form_owner(external_textarea),
            Some(external)
        );
        assert!(dom.set_form_control_checked(detached_radio, true));
        assert!(dom.set_form_control_checked(external_radio, true));
        assert!(dom.form_control_state(detached_radio).unwrap().checked);
        assert!(dom.form_control_state(external_radio).unwrap().checked);
    }

    #[test]
    fn connected_shadow_radio_searches_its_tree_for_explicit_form_owner() {
        let mut dom = ScriptedDom::new();
        let document = dom.document();
        let host = dom.create_element(tag(local_name!("div")));
        let light_form = dom.create_element(tag(local_name!("form")));
        dom.set_attribute(light_form, attr(local_name!("id")), "outside");
        dom.append_child(document, host);
        dom.append_child(host, light_form);
        let shadow = dom
            .attach_shadow(host, layout_dom_api::ShadowRootInit::default())
            .unwrap();
        let shadow_radio = dom.create_element(tag(local_name!("input")));
        let light_radio = dom.create_element(tag(local_name!("input")));
        for radio in [shadow_radio, light_radio] {
            dom.set_attribute(radio, attr(local_name!("type")), "radio");
            dom.set_attribute(radio, attr(local_name!("name")), "group");
        }
        dom.set_attribute(shadow_radio, attr(local_name!("form")), "outside");
        dom.append_child(shadow, shadow_radio);
        dom.append_child(light_form, light_radio);

        assert_eq!(dom.form_control_form_owner(shadow_radio), None);
        assert_eq!(dom.form_control_form_owner(light_radio), Some(light_form));
        assert!(dom.set_form_control_checked(shadow_radio, true));
        assert!(dom.set_form_control_checked(light_radio, true));
        assert!(dom.form_control_state(shadow_radio).unwrap().checked);
        assert!(dom.form_control_state(light_radio).unwrap().checked);
    }

    #[test]
    fn radio_reconciliation_preserves_unowned_disconnected_groups_but_resolves_form_owner() {
        let mut dom = ScriptedDom::new();
        let fragment = dom.create_fragment();
        let generic = dom.create_element(tag(local_name!("div")));
        dom.append_child(fragment, generic);
        dom.set_inner_html(
            generic,
            "<input type=radio name=g checked><input type=radio name=g checked>",
        );
        let generic_radios: Vec<_> = dom
            .dom_children(generic)
            .filter(|id| dom.is_html_input(*id))
            .collect();
        assert_eq!(generic_radios.len(), 2);
        assert!(
            generic_radios
                .iter()
                .all(|id| dom.form_control_state(*id).unwrap().checked)
        );

        let form = dom.create_element(tag(local_name!("form")));
        dom.append_child(fragment, form);
        dom.set_inner_html(
            form,
            "<input type=radio name=g checked><input type=radio name=g checked>",
        );
        let form_radios: Vec<_> = dom
            .dom_children(form)
            .filter(|id| dom.is_html_input(*id))
            .collect();
        assert_eq!(form_radios.len(), 2);
        assert!(!dom.form_control_state(form_radios[0]).unwrap().checked);
        assert!(dom.form_control_state(form_radios[1]).unwrap().checked);
    }

    #[test]
    fn radio_id_target_removal_reconciles_newly_shared_owner_group() {
        let mut dom = ScriptedDom::new();
        let document = dom.document();
        let form = dom.create_element(tag(local_name!("form")));
        dom.set_attribute(form, attr(local_name!("id")), "target");
        let explicit = dom.create_element(tag(local_name!("input")));
        let unowned = dom.create_element(tag(local_name!("input")));
        for radio in [explicit, unowned] {
            dom.set_attribute(radio, attr(local_name!("type")), "radio");
            dom.set_attribute(radio, attr(local_name!("name")), "shared");
        }
        dom.set_attribute(explicit, attr(local_name!("form")), "target");
        dom.append_child(document, form);
        dom.append_child(document, explicit);
        dom.append_child(document, unowned);
        assert_eq!(dom.form_control_form_owner(explicit), Some(form));
        assert!(dom.set_form_control_checked(explicit, true));
        assert!(dom.set_form_control_checked(unowned, true));

        dom.remove_attribute(form, attr(local_name!("id")));
        assert_eq!(dom.form_control_form_owner(explicit), None);
        assert!(dom.form_control_state(explicit).unwrap().checked);
        assert!(!dom.form_control_state(unowned).unwrap().checked);
    }

    #[test]
    fn text_content_subtree_replacement_reconciles_surviving_explicit_controls() {
        let mut dom = ScriptedDom::new();
        let document = dom.document();
        let wrapper = dom.create_element(tag(local_name!("div")));
        let form = dom.create_element(tag(local_name!("form")));
        dom.set_attribute(form, attr(local_name!("id")), "target");
        let explicit = dom.create_element(tag(local_name!("input")));
        let unowned = dom.create_element(tag(local_name!("input")));
        for radio in [explicit, unowned] {
            dom.set_attribute(radio, attr(local_name!("type")), "radio");
            dom.set_attribute(radio, attr(local_name!("name")), "shared");
        }
        dom.set_attribute(explicit, attr(local_name!("form")), "target");
        dom.append_child(document, wrapper);
        dom.append_child(wrapper, form);
        dom.append_child(document, explicit);
        dom.append_child(document, unowned);
        assert!(dom.set_form_control_checked(explicit, true));
        assert!(dom.set_form_control_checked(unowned, true));
        assert_eq!(dom.form_control_form_owner(explicit), Some(form));

        dom.set_text_content(wrapper, "replacement");
        assert_eq!(dom.form_control_form_owner(explicit), None);
        assert!(dom.form_control_state(explicit).unwrap().checked);
        assert!(!dom.form_control_state(unowned).unwrap().checked);
    }

    #[test]
    fn simultaneous_owner_changes_keep_last_radio_checked_in_tree_order() {
        let mut dom = ScriptedDom::new();
        let document = dom.document();
        let wrapper = dom.create_element(tag(local_name!("div")));
        let form_a = dom.create_element(tag(local_name!("form")));
        let form_b = dom.create_element(tag(local_name!("form")));
        dom.set_attribute(form_a, attr(local_name!("id")), "a");
        dom.set_attribute(form_b, attr(local_name!("id")), "b");
        let first = dom.create_element(tag(local_name!("input")));
        let second = dom.create_element(tag(local_name!("input")));
        for (radio, form_id) in [(first, "a"), (second, "b")] {
            dom.set_attribute(radio, attr(local_name!("type")), "radio");
            dom.set_attribute(radio, attr(local_name!("name")), "shared");
            dom.set_attribute(radio, attr(local_name!("form")), form_id);
        }
        dom.append_child(document, wrapper);
        dom.append_child(wrapper, form_a);
        dom.append_child(wrapper, form_b);
        dom.append_child(document, first);
        dom.append_child(document, second);
        assert!(dom.set_form_control_checked(first, true));
        assert!(dom.set_form_control_checked(second, true));
        assert_eq!(dom.form_control_form_owner(first), Some(form_a));
        assert_eq!(dom.form_control_form_owner(second), Some(form_b));

        dom.set_text_content(wrapper, "replace both form owners");
        assert_eq!(dom.form_control_form_owner(first), None);
        assert_eq!(dom.form_control_form_owner(second), None);
        assert!(!dom.form_control_state(first).unwrap().checked);
        assert!(dom.form_control_state(second).unwrap().checked);
    }

    #[test]
    fn connecting_host_reconciles_checked_radios_inside_its_shadow_tree() {
        let mut dom = ScriptedDom::new();
        let document = dom.document();
        let host = dom.create_element(tag(local_name!("div")));
        let shadow = dom
            .attach_shadow(host, layout_dom_api::ShadowRootInit::default())
            .unwrap();
        let radios: Vec<_> = (0..2)
            .map(|_| {
                let radio = dom.create_element(tag(local_name!("input")));
                dom.set_attribute(radio, attr(local_name!("type")), "radio");
                dom.set_attribute(radio, attr(local_name!("name")), "shadow-group");
                dom.set_form_control_checked(radio, true);
                dom.append_child(shadow, radio);
                radio
            })
            .collect();
        assert!(
            radios
                .iter()
                .all(|id| dom.form_control_state(*id).unwrap().checked)
        );

        dom.append_child(document, host);
        assert!(!dom.form_control_state(radios[0]).unwrap().checked);
        assert!(dom.form_control_state(radios[1]).unwrap().checked);
    }

    #[test]
    fn release_subtree_skips_retired_radio_snapshot_ids() {
        let mut dom = ScriptedDom::new();
        let document = dom.document();
        let form = dom.create_element(tag(local_name!("form")));
        let radio = dom.create_element(tag(local_name!("input")));
        dom.set_attribute(radio, attr(local_name!("type")), "radio");
        dom.set_attribute(radio, attr(local_name!("name")), "group");
        dom.append_child(document, form);
        dom.append_child(form, radio);
        dom.set_form_control_checked(radio, true);

        LayoutDomMut::remove(&mut dom, form);
        assert!(!dom.is_live(form));
        assert!(!dom.is_live(radio));
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
fn normalized_input_type(raw: &str) -> String {
    let kind = raw.to_ascii_lowercase();
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
    let mut pending = vec![node];
    while let Some(node) = pending.pop() {
        out.push(node);
        let children: Vec<_> = dom.dom_children(node).collect();
        pending.extend(children.into_iter().rev());
    }
}
fn attr_name(local: &str) -> QualName {
    QualName::new(None, markup5ever::ns!(), LocalName::from(local))
}
fn parse_html_nonnegative_integer_is_one(raw: &str) -> Option<bool> {
    let bytes = raw.as_bytes();
    let mut start = 0;
    while bytes
        .get(start)
        .is_some_and(|byte| matches!(*byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0c))
    {
        start += 1;
    }
    let negative = match bytes.get(start) {
        Some(&b'-') => {
            start += 1;
            true
        },
        Some(&b'+') => {
            start += 1;
            false
        },
        _ => false,
    };
    let digits_start = start;
    while bytes.get(start).is_some_and(|byte| byte.is_ascii_digit()) {
        start += 1;
    }
    if start == digits_start {
        return None;
    }
    let significant = bytes[digits_start..start]
        .iter()
        .position(|&byte| byte != b'0')
        .map(|offset| &bytes[digits_start + offset..start]);
    if negative && significant.is_some() {
        return None;
    }
    Some(significant.is_some_and(|digits| digits == b"1"))
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
                let candidate = range_step_value(base, step, rounded);
                if candidate >= low && (high < low || candidate <= high) {
                    n = candidate;
                } else {
                    // If rounding points outside the bounds, use the nearest
                    // admitted step instead of retaining a step mismatch.
                    let bounded = if candidate < low {
                        range_step_value(base, step, ((low - base) / step).ceil())
                    } else {
                        range_step_value(base, step, ((high - base) / step).floor())
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

/// Form attributes describe decimal steps. Scale their canonical decimal
/// representations before interpolation so, for example, six tenths produces
/// 0.6 rather than the binary multiplication tail 0.6000000000000001.
fn range_step_value(base: f64, step: f64, index: f64) -> f64 {
    let decimal_places = |value: f64| {
        value
            .to_string()
            .split_once('.')
            .map_or(0, |(_, fraction)| fraction.len() as i32)
    };
    let scale = 10_f64.powi(decimal_places(base).max(decimal_places(step)));
    let scaled_base = (base * scale).round();
    let scaled_step = (step * scale).round();
    let scaled_offset = index * scaled_step;
    let scaled_value = scaled_base + scaled_offset;
    const MAX_EXACT_INTEGER: f64 = ((1_u64 << 53) - 1) as f64;
    if scale.is_finite()
        && [scaled_base, scaled_step, scaled_offset, scaled_value]
            .iter()
            .all(|value| value.abs() <= MAX_EXACT_INTEGER)
    {
        scaled_value / scale
    } else {
        // Scaling outside f64's exact-integer range can erase a representable
        // small step. Extreme bases and subnormal steps also take this path.
        base + index * step
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
pub(super) fn parse_html_float(value: &str) -> Option<f64> {
    (is_html_float(value))
        .then(|| value.parse().ok())
        .flatten()
        .filter(|n: &f64| n.is_finite())
}
fn valid_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|b| b.is_ascii_hexdigit())
}
pub(super) fn valid_month(s: &str) -> bool {
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
pub(super) fn valid_date(s: &str) -> bool {
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
pub(super) fn valid_week(s: &str) -> bool {
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
pub(super) fn valid_time(s: &str) -> bool {
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
