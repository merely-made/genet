// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Constraint-validation facts derived from the scripted DOM's current nodes.
//!
//! Pattern matching uses the locked ECMAScript-compatible `regress` engine
//! with the HTML-required UnicodeSets mode. It does not store a second value.

use layout_dom_api::{
    FormControlInteractionState, FormControlValidity, LayoutDom, LocalName, NodeKind,
    SelectionDirection, ValidityState,
};
use markup5ever::ns;
use regress::{Flags, Regex};
use std::cmp::Ordering;

use crate::{NodeId, ScriptedDom};

const LISTED: &[&str] = &[
    "input", "textarea", "select", "button", "fieldset", "output", "object",
];
#[cfg(test)]
const INPUT_TYPES: &[&str] = &[
    "hidden",
    "text",
    "search",
    "tel",
    "url",
    "email",
    "password",
    "date",
    "month",
    "week",
    "time",
    "datetime-local",
    "number",
    "range",
    "color",
    "checkbox",
    "radio",
    "file",
    "submit",
    "image",
    "reset",
    "button",
];

impl ScriptedDom {
    pub(super) fn html_name(&self, id: NodeId) -> Option<&str> {
        let name = self.element_name(id)?;
        (name.ns == ns!(html)).then_some(name.local.as_ref())
    }

    pub fn set_custom_validity(&mut self, id: NodeId, message: &str) -> bool {
        if !self.is_validation_element(id) {
            return false;
        }
        let normalized = normalize_newlines(message);
        let legacy = self.node(id).form_control.is_some();
        let changed = if let Some(state) = self.node_mut(id).form_control.as_mut() {
            let changed = state.custom_validity_message != normalized;
            if changed {
                state.custom_validity_message = normalized;
            }
            changed
        } else if let Some(message) = self.node_mut(id).custom_validity_message.as_mut() {
            let changed = *message != normalized;
            if changed {
                *message = normalized;
            }
            changed
        } else {
            return false;
        };
        if changed {
            if legacy {
                self.mutations
                    .push(crate::DomMutation::FormControlStateChanged { node: id });
            } else {
                self.mutations
                    .push(crate::DomMutation::FormControlCustomValidityChanged { node: id });
            }
        }
        true
    }

    pub fn form_control_custom_validity_message(&self, id: NodeId) -> Option<String> {
        if !self.is_validation_element(id) {
            return None;
        }
        self.node(id)
            .form_control
            .as_ref()
            .map(|state| state.custom_validity_message.clone())
            .or_else(|| self.node(id).custom_validity_message.clone())
    }

    pub fn associated_form_controls(&self, form: NodeId) -> Vec<NodeId> {
        if self.html_name(form) != Some("form") {
            return Vec::new();
        }
        let root = self.form_control_tree_root(form);
        let mut nodes = Vec::new();
        collect_preorder(self, root, &mut nodes);
        nodes
            .into_iter()
            .filter(|&id| {
                self.is_listed_control(id) && self.form_control_form_owner(id) == Some(form)
            })
            .collect()
    }

    pub fn set_form_control_user_value(
        &mut self,
        id: NodeId,
        value: &str,
        start: Option<u32>,
        end: Option<u32>,
    ) -> bool {
        let Some(kind) = self.html_name(id).map(str::to_owned) else {
            return false;
        };
        if !matches!(kind.as_str(), "input" | "textarea") {
            return false;
        }
        let input_type = if kind == "input" {
            self.input_type_for_state(id)
        } else {
            "textarea".to_owned()
        };
        let old_control = self.node(id).form_control.clone();
        let old_value = self.form_control_value(id).unwrap_or_default();
        let old_interaction = self.node(id).form_interaction.clone().unwrap_or_default();
        let sanitized = if kind == "input" {
            self.sanitize_value_for_input(id, &input_type, value)
        } else {
            value.replace("\r\n", "\n").replace('\r', "\n")
        };
        if let Some(state) = self.node_mut(id).form_control.as_mut() {
            state.value = sanitized.clone();
            state.dirty_value = true;
            if let (Some(start), Some(end)) = (start, end) {
                state.selection_start = Some(start.min(utf16_len(&sanitized)));
                state.selection_end = Some(
                    end.min(utf16_len(&sanitized))
                        .max(start.min(utf16_len(&sanitized))),
                );
                state.selection_direction = SelectionDirection::None;
            }
        } else {
            return false;
        }
        let bad_input = kind == "input"
            && input_type_supports_bad_input(&input_type)
            && !value.is_empty()
            && sanitized.is_empty();
        let raw_draft = bad_input.then(|| value.to_owned());
        let changed_by_user = old_value != sanitized || old_interaction.draft_value != raw_draft;
        let control_changed = self.node(id).form_control != old_control;
        let editing_value = raw_draft.as_deref().unwrap_or(&sanitized);
        let interaction = self
            .node_mut(id)
            .form_interaction
            .get_or_insert_with(Default::default);
        interaction.last_change_by_user |= changed_by_user;
        interaction.user_edit_pending = interaction
            .user_edit_initial_value
            .as_deref()
            .map_or(changed_by_user, |initial| initial != editing_value);
        interaction.bad_input = bad_input;
        interaction.draft_value = raw_draft;
        if bad_input {
            let length = utf16_len(value);
            interaction.draft_selection_start = Some(start.unwrap_or(length).min(length));
            interaction.draft_selection_end = Some(
                end.unwrap_or(length)
                    .min(length)
                    .max(interaction.draft_selection_start.unwrap_or(0)),
            );
            interaction.draft_selection_direction = SelectionDirection::None;
        } else {
            interaction.draft_selection_start = None;
            interaction.draft_selection_end = None;
            interaction.draft_selection_direction = SelectionDirection::None;
        }
        let interaction_changed = *interaction != old_interaction;
        if control_changed {
            self.mutations
                .push(crate::DomMutation::FormControlStateChanged { node: id });
        }
        if interaction_changed {
            self.mutations
                .push(crate::DomMutation::FormControlInteractionStateChanged { node: id });
        }
        true
    }

    /// Update internal native-editor selection without representing a value
    /// edit or changing constraint-validation interaction history.
    pub fn set_form_control_editing_selection(
        &mut self,
        id: NodeId,
        start: Option<u32>,
        end: Option<u32>,
        direction: SelectionDirection,
    ) -> bool {
        let Some(editing) = self.form_control_editing_value(id) else {
            return false;
        };
        let length = utf16_len(&editing.value);
        let start = start.unwrap_or(length).min(length);
        let end = end.unwrap_or(length).min(length).max(start);
        if self
            .node(id)
            .form_interaction
            .as_ref()
            .is_some_and(|state| state.bad_input)
        {
            let Some(state) = self.node_mut(id).form_interaction.as_mut() else {
                return false;
            };
            let changed = state.draft_selection_start != Some(start)
                || state.draft_selection_end != Some(end)
                || state.draft_selection_direction != direction;
            state.draft_selection_start = Some(start);
            state.draft_selection_end = Some(end);
            state.draft_selection_direction = direction;
            if changed {
                self.mutations
                    .push(crate::DomMutation::FormControlInteractionStateChanged { node: id });
            }
        } else {
            let Some(state) = self.node_mut(id).form_control.as_mut() else {
                return false;
            };
            let changed = state.selection_start != Some(start)
                || state.selection_end != Some(end)
                || state.selection_direction != direction;
            state.selection_start = Some(start);
            state.selection_end = Some(end);
            state.selection_direction = direction;
            if changed {
                self.mutations
                    .push(crate::DomMutation::FormControlStateChanged { node: id });
            }
        }
        true
    }

    /// Start a native user-edit session, establishing the baseline used by
    /// the HTML user-validity transition at blur/commit time.
    pub fn begin_form_control_user_edit(&mut self, id: NodeId) -> bool {
        let baseline = if self.html_name(id) == Some("select") {
            Some(self.select_selected_index(id).to_string())
        } else {
            self.node(id).form_interaction.as_ref().map(|state| {
                state
                    .draft_value
                    .clone()
                    .unwrap_or_else(|| self.form_control_value(id).unwrap_or_default())
            })
        };
        let Some(state) = self.node_mut(id).form_interaction.as_mut() else {
            return false;
        };
        if state.user_edit_pending || state.user_edit_initial_value != baseline {
            state.user_edit_pending = false;
            state.user_edit_initial_value = baseline;
            self.mutations
                .push(crate::DomMutation::FormControlInteractionStateChanged { node: id });
        }
        true
    }

    pub fn commit_form_control_user_edit(&mut self, id: NodeId) -> bool {
        let Some(state) = self.node_mut(id).form_interaction.as_mut() else {
            return false;
        };
        let changed = state.user_edit_pending && !state.user_validity;
        if changed {
            state.user_validity = true;
        }
        let pending_changed = state.user_edit_pending;
        state.user_edit_pending = false;
        let baseline_changed = state.user_edit_initial_value.take().is_some();
        if changed || pending_changed || baseline_changed {
            self.mutations
                .push(crate::DomMutation::FormControlInteractionStateChanged { node: id });
        }
        true
    }

    /// Mark a user-driven select selection as having passed through the
    /// control's user-validity transition.
    pub fn commit_select_user_selection(&mut self, select: NodeId) -> bool {
        if self.html_name(select) != Some("select") {
            return false;
        }
        let Some(state) = self.node_mut(select).form_interaction.as_mut() else {
            return false;
        };
        let changed = state.user_edit_pending && !state.user_validity;
        if changed {
            state.user_validity = true;
        }
        let pending_changed = state.user_edit_pending;
        state.user_edit_pending = false;
        if changed || pending_changed {
            self.mutations
                .push(crate::DomMutation::FormControlInteractionStateChanged { node: select });
        }
        true
    }

    pub fn select_options(&self, select: NodeId) -> Vec<NodeId> {
        if self.html_name(select) != Some("select") {
            return Vec::new();
        }
        let mut options = Vec::new();
        let mut stack = self
            .dom_children(select)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|id| (id, false))
            .collect::<Vec<_>>();
        while let Some((id, inside_optgroup)) = stack.pop() {
            let name = self.html_name(id);
            if name == Some("option") {
                options.push(id);
                continue;
            }
            if matches!(name, Some("select" | "hr" | "datalist"))
                || (name == Some("optgroup") && inside_optgroup)
            {
                continue;
            }
            let child_inside_optgroup = inside_optgroup || name == Some("optgroup");
            let children = self.dom_children(id).collect::<Vec<_>>();
            stack.extend(
                children
                    .into_iter()
                    .rev()
                    .map(|child| (child, child_inside_optgroup)),
            );
        }
        options
    }

    pub fn select_selected_index(&self, select: NodeId) -> i32 {
        self.select_options(select)
            .iter()
            .position(|&id| {
                self.option_selected_state(id)
                    .is_some_and(|state| state.selected)
            })
            .map_or(-1, |index| index.min(i32::MAX as usize) as i32)
    }

    pub fn set_select_selected_index(&mut self, select: NodeId, index: i32) -> bool {
        if self.html_name(select) != Some("select") {
            return false;
        }
        self.apply_select_selected_index(select, index, true);
        self.clear_user_edit_state(select);
        true
    }

    fn apply_select_selected_index(&mut self, select: NodeId, index: i32, dirty: bool) {
        let options = self.select_options(select);
        for (i, option) in options.into_iter().enumerate() {
            let chosen = index >= 0 && i == index as usize;
            self.set_option_selected(option, chosen, dirty && chosen);
        }
    }

    pub fn set_select_user_selected_index(&mut self, select: NodeId, index: i32) -> bool {
        if self.html_name(select) != Some("select") {
            return false;
        }
        let before = self.select_selected_index(select);
        self.apply_select_selected_index(select, index, true);
        if before != self.select_selected_index(select) {
            let current = self.select_selected_index(select).to_string();
            let state = self
                .node_mut(select)
                .form_interaction
                .get_or_insert_with(Default::default);
            state.last_change_by_user = true;
            state.user_edit_pending = state
                .user_edit_initial_value
                .as_deref()
                .map_or(true, |initial| initial != current);
            self.mutations
                .push(crate::DomMutation::FormControlInteractionStateChanged { node: select });
        }
        true
    }

    pub fn set_option_selected(&mut self, option: NodeId, selected: bool, dirty: bool) -> bool {
        if self.html_name(option) != Some("option") {
            return false;
        }
        if selected {
            if let Some(select) = self.option_select_ancestor(option)
                && !self.has_attr(select, "multiple")
            {
                let peers = self
                    .select_options(select)
                    .into_iter()
                    .filter(|&peer| peer != option)
                    .collect::<Vec<_>>();
                for peer in peers {
                    self.set_option_selected(peer, false, false);
                }
            }
        }
        let Some(state) = self.node_mut(option).option_state.as_mut() else {
            return false;
        };
        let changed = state.selected != selected || (dirty && !state.dirty);
        state.selected = selected;
        if dirty {
            state.dirty = true;
        }
        if changed {
            self.mutations
                .push(crate::DomMutation::OptionStateChanged { node: option });
        }
        true
    }

    /// Apply the option `selected` IDL setter: update selectedness and
    /// dirtiness, then ask the owning select to reset its selectedness.
    /// SelectedIndex/value setters use the low-level helper directly because
    /// their explicit `-1` / unmatched-value state must remain unselected.
    pub fn set_option_selected_by_script(&mut self, option: NodeId, selected: bool) -> bool {
        if !self.is_html_option(option) || !self.set_option_selected(option, selected, true) {
            return false;
        }
        if let Some(select) = self.option_select_ancestor(option) {
            self.set_select_selectedness(select);
        }
        true
    }

    pub(super) fn is_listed_control(&self, id: NodeId) -> bool {
        self.html_name(id)
            .is_some_and(|name| LISTED.contains(&name))
    }

    fn is_validation_element(&self, id: NodeId) -> bool {
        matches!(
            self.html_name(id),
            Some("input" | "textarea" | "select" | "button" | "fieldset" | "output" | "object")
        )
    }

    pub(super) fn compute_form_control_validity(&self, id: NodeId) -> Option<FormControlValidity> {
        let name = self.html_name(id)?;
        if !self.is_validation_element(id) {
            return None;
        }
        let interaction = self.node(id).form_interaction.clone().unwrap_or_default();
        let custom_error = self
            .node(id)
            .form_control
            .as_ref()
            .map(|state| !state.custom_validity_message.is_empty())
            .or_else(|| {
                self.node(id)
                    .custom_validity_message
                    .as_ref()
                    .map(|message| !message.is_empty())
            })
            .unwrap_or(false);
        let value = self.control_value_for_validity(id);
        let mut flags = ValidityState {
            custom_error,
            bad_input: interaction.bad_input,
            ..ValidityState::default()
        };
        let kind = (name == "input").then(|| self.input_type_for_state(id));
        let required_applies = match name {
            "input" => kind.as_deref().is_some_and(input_type_supports_required),
            "textarea" | "select" => true,
            _ => false,
        };
        let required_uses_mutability = name == "textarea"
            || name == "input" && kind.as_deref().is_some_and(input_type_supports_readonly);
        let mutable_for_required = !required_uses_mutability
            || (!self.is_disabled(id)
                && !(name == "textarea" && self.has_attr(id, "readonly"))
                && !(name == "input"
                    && kind.as_deref().is_some_and(input_type_supports_readonly)
                    && self.has_attr(id, "readonly")));
        if required_applies && mutable_for_required && self.has_attr(id, "required") {
            flags.value_missing = match (name, kind.as_deref()) {
                ("input", Some("checkbox")) => !self
                    .node(id)
                    .form_control
                    .as_ref()
                    .is_some_and(|s| s.checked),
                ("input", Some("radio")) => self.radio_group_required_missing(id),
                ("select", _) => self.select_value_missing(id),
                _ => value.is_empty(),
            };
        }
        if name == "input" && kind.as_deref() == Some("radio") {
            flags.value_missing |= self.radio_group_required_missing(id);
        }
        if let Some(kind) = kind.as_deref() {
            if kind == "email" && !value.is_empty() {
                let multiple = self.has_attr(id, "multiple");
                flags.type_mismatch = if multiple {
                    value.split(',').any(|token| {
                        let token = token.trim_matches(|c: char| c.is_ascii_whitespace());
                        !valid_email(token)
                    })
                } else {
                    !valid_email(&value)
                };
            } else if kind == "url" && !value.is_empty() {
                flags.type_mismatch = !valid_url(&value);
            }
            if input_type_supports_pattern(kind)
                && !value.is_empty()
                && let Some(pattern) = self.attr(id, "pattern")
                && pattern_mismatch(
                    pattern,
                    &value,
                    kind == "email" && self.has_attr(id, "multiple"),
                )
            {
                flags.pattern_mismatch = true;
            }
            if let Some(numeric) = self.numeric_value(id, kind, &value) {
                if !is_calendar_kind(kind) {
                    let min = self
                        .attr(id, "min")
                        .and_then(|bound| self.numeric_value(id, kind, bound));
                    let max = self
                        .attr(id, "max")
                        .and_then(|bound| self.numeric_value(id, kind, bound));
                    if kind == "time" && min.zip(max).is_some_and(|(min, max)| min > max) {
                        let (min, max) = min.zip(max).unwrap();
                        let in_reversed_gap = numeric > max && numeric < min;
                        flags.range_underflow = in_reversed_gap;
                        flags.range_overflow = in_reversed_gap;
                    } else {
                        flags.range_underflow = min.is_some_and(|min| numeric < min);
                        flags.range_overflow = max.is_some_and(|max| numeric > max);
                    }
                }
                flags.step_mismatch = self.step_mismatch(id, kind, numeric);
            }
            if is_calendar_kind(kind) {
                flags.range_underflow = self
                    .attr(id, "min")
                    .and_then(|min| compare_calendar_values(kind, &value, min))
                    .is_some_and(|order| order == Ordering::Less);
                flags.range_overflow = self
                    .attr(id, "max")
                    .and_then(|max| compare_calendar_values(kind, &value, max))
                    .is_some_and(|order| order == Ordering::Greater);
            }
        }
        if name == "textarea"
            || name == "input" && kind.as_deref().is_some_and(input_type_supports_length)
        {
            let dirty_value = self
                .node(id)
                .form_control
                .as_ref()
                .is_some_and(|state| state.dirty_value);
            if dirty_value && self.last_user_change(id) {
                let length = utf16_len(&value);
                flags.too_long = self
                    .attr(id, "maxlength")
                    .and_then(parse_nonnegative)
                    .is_some_and(|n| length > n);
                flags.too_short = self
                    .attr(id, "minlength")
                    .and_then(parse_nonnegative)
                    .is_some_and(|n| length < n && !value.is_empty());
            }
        }
        let will_validate = self.will_validate(id, name, kind.as_deref());
        Some(FormControlValidity {
            flags,
            will_validate,
            user_validity: interaction.user_validity,
        })
    }

    fn will_validate(&self, id: NodeId, name: &str, kind: Option<&str>) -> bool {
        if name == "fieldset" || name == "output" {
            return false;
        }
        if name == "object" {
            return false;
        }
        if name == "input" && kind.is_some_and(|t| matches!(t, "hidden" | "button" | "reset")) {
            return false;
        }
        if self.is_disabled(id) || self.has_datalist_ancestor(id) {
            return false;
        }
        if name == "button" {
            let button_type = self.attr(id, "type").unwrap_or("").to_ascii_lowercase();
            return match button_type.as_str() {
                "submit" => true,
                "reset" | "button" => false,
                // Missing and invalid values map to Auto. Auto is a submit
                // button only outside command and select contexts.
                _ => {
                    !self.has_attr(id, "command")
                        && !self.has_attr(id, "commandfor")
                        && self
                            .parent(id)
                            .is_none_or(|parent| self.html_name(parent) != Some("select"))
                },
            };
        }
        if matches!(name, "input" | "textarea") && self.has_attr(id, "readonly") {
            return false;
        }
        true
    }

    fn is_disabled(&self, id: NodeId) -> bool {
        if self.has_attr(id, "disabled") {
            return true;
        }
        let mut ancestor = self.parent(id);
        while let Some(fieldset) = ancestor {
            if self.html_name(fieldset) == Some("fieldset") && self.has_attr(fieldset, "disabled") {
                let first_legend = self
                    .dom_children(fieldset)
                    .find(|&child| self.html_name(child) == Some("legend"));
                if !first_legend.is_some_and(|legend| self.is_descendant_or_self(id, legend)) {
                    return true;
                }
            }
            ancestor = self.parent(fieldset);
        }
        false
    }

    fn has_datalist_ancestor(&self, id: NodeId) -> bool {
        let mut ancestor = self.parent(id);
        while let Some(node) = ancestor {
            if self.html_name(node) == Some("datalist") {
                return true;
            }
            ancestor = self.parent(node);
        }
        false
    }

    fn is_descendant_or_self(&self, mut node: NodeId, ancestor: NodeId) -> bool {
        loop {
            if node == ancestor {
                return true;
            }
            let Some(parent) = self.parent(node) else {
                return false;
            };
            node = parent;
        }
    }

    fn has_attr(&self, id: NodeId, local: &str) -> bool {
        self.attr(id, local).is_some()
    }
    fn attr(&self, id: NodeId, local: &str) -> Option<&str> {
        self.attribute(id, &markup5ever::ns!(), &LocalName::from(local))
    }

    fn control_value_for_validity(&self, id: NodeId) -> String {
        match self.html_name(id) {
            Some("input" | "textarea" | "select") => {
                self.form_control_value(id).unwrap_or_default()
            },
            Some("output") => text_content(self, id),
            _ => String::new(),
        }
    }

    fn select_value_missing(&self, select: NodeId) -> bool {
        let options = self.select_options(select);
        let selected: Vec<_> = options
            .iter()
            .copied()
            .filter(|&o| self.node(o).option_state.is_some_and(|s| s.selected))
            .collect();
        if selected.is_empty() {
            return true;
        }
        if self.has_attr(select, "multiple") || !self.select_display_size_is_one(select) {
            return false;
        }
        let first_option = options.first().copied();
        let placeholder_is_eligible = first_option.is_some_and(|option| {
            let mut ancestor = self.parent(option);
            while let Some(id) = ancestor {
                if id == select {
                    return true;
                }
                if self.html_name(id) == Some("optgroup") {
                    return false;
                }
                ancestor = self.parent(id);
            }
            false
        });
        selected.len() == 1
            && Some(selected[0]) == first_option
            && placeholder_is_eligible
            && self.control_value_for_validity(select).is_empty()
    }

    fn radio_group_required_missing(&self, id: NodeId) -> bool {
        let Some(name) = self.attr(id, "name") else {
            return false;
        };
        if name.is_empty() {
            return false;
        }
        let root = self.form_control_tree_root(id);
        let mut nodes = Vec::new();
        collect_preorder(self, root, &mut nodes);
        let group = nodes
            .into_iter()
            .filter(|&other| {
                self.html_name(other) == Some("input")
                    && self.input_type_for_state(other) == "radio"
                    && self.attr(other, "name") == Some(name)
                    && self.form_control_form_owner(other) == self.form_control_form_owner(id)
            })
            .collect::<Vec<_>>();
        group.iter().any(|&other| self.has_attr(other, "required"))
            && !group.into_iter().any(|other| {
                self.node(other)
                    .form_control
                    .as_ref()
                    .is_some_and(|s| s.checked)
            })
    }

    fn last_user_change(&self, id: NodeId) -> bool {
        self.node(id)
            .form_interaction
            .as_ref()
            .is_some_and(|s| s.last_change_by_user)
    }

    fn sanitize_value_for_input(&self, id: NodeId, kind: &str, value: &str) -> String {
        crate::forms::sanitize_value_for_state(
            kind,
            value,
            self.attr(id, "min"),
            self.attr(id, "max"),
            self.attr(id, "step"),
            self.has_attr(id, "multiple"),
            self.attr(id, "value"),
        )
    }

    fn numeric_value(&self, id: NodeId, kind: &str, value: &str) -> Option<f64> {
        // Constraint range ordering for date/month/week/datetime-local is
        // handled from validated components below, not by this Number-sized
        // conversion. Step checks currently retain a finite f64 conversion;
        // loss of precision or overflow for enormous calendar years remains
        // a compliance limitation, not a bound on HTML's valid year strings.
        match kind {
            "number" | "range" => parse_html_float(value),
            "date" => date_days(value),
            "month" => parse_month(value),
            // Normalize to an integer week count anchored at 1970-W01;
            // HTML's default week step is one week, not one day.
            "week" => week_monday_days(value).map(|d| (d + 3.0) / 7.0),
            "time" => parse_time_millis(value),
            "datetime-local" => parse_datetime_millis(value),
            _ => {
                let _ = id;
                None
            },
        }
    }

    fn step_mismatch(&self, id: NodeId, kind: &str, value: f64) -> bool {
        if self
            .attr(id, "step")
            .is_some_and(|s| s.eq_ignore_ascii_case("any"))
        {
            return false;
        }
        let default_step = match kind {
            "date" => 1.0,
            "week" => 1.0,
            "time" | "datetime-local" => 60.0,
            _ => 1.0,
        };
        let step = self
            .attr(id, "step")
            .and_then(parse_html_float)
            .filter(|step| *step > 0.0)
            .unwrap_or(default_step)
            * if matches!(kind, "time" | "datetime-local") {
                1000.0
            } else {
                1.0
            };
        let base = self
            .attr(id, "min")
            .and_then(|v| self.numeric_value(id, kind, v))
            .or_else(|| {
                self.attr(id, "value")
                    .and_then(|v| self.numeric_value(id, kind, v))
            })
            .unwrap_or(0.0);
        let quotient = (value - base) / step;
        // Allow only the floating-point error of this computation. A fixed
        // absolute tolerance would accept small, genuinely nonzero remainders
        // such as value=1e-10 with step=1.
        let distance = (quotient - quotient.round()).abs();
        distance > f64::EPSILON * quotient.abs() * 4.0
    }

    pub(super) fn reset_interaction_state(&mut self, id: NodeId) {
        if let Some(state) = self.node_mut(id).form_interaction.as_mut() {
            let new = FormControlInteractionState::default();
            if *state != new {
                *state = new;
                self.mutations
                    .push(crate::DomMutation::FormControlInteractionStateChanged { node: id });
            }
        }
    }
}

fn collect_preorder(dom: &ScriptedDom, root: NodeId, out: &mut Vec<NodeId>) {
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        out.push(id);
        stack.extend(dom.dom_children(id).collect::<Vec<_>>().into_iter().rev());
    }
}

fn text_content(dom: &ScriptedDom, root: NodeId) -> String {
    let mut nodes = Vec::new();
    collect_preorder(dom, root, &mut nodes);
    nodes
        .into_iter()
        .filter(|&id| matches!(dom.kind(id), NodeKind::Text | NodeKind::CdataSection))
        .filter_map(|id| dom.text(id))
        .collect()
}

fn input_type_supports_pattern(kind: &str) -> bool {
    matches!(
        kind,
        "text" | "search" | "tel" | "url" | "email" | "password"
    )
}

fn pattern_mismatch(pattern: &str, value: &str, multiple_email: bool) -> bool {
    // HTML first validates the attribute as a standalone v-mode expression.
    // Invalid expressions are omitted from constraint validation.
    let flags = Flags {
        // v mode implies Unicode mode. regress keeps these flags independent.
        unicode: true,
        unicode_sets: true,
        ..Flags::default()
    };
    if Regex::with_flags(pattern, flags).is_err() {
        return false;
    }
    let anchored = format!("^(?:{pattern})$");
    let Ok(regex) = Regex::with_flags(&anchored, flags) else {
        return false;
    };
    let matches = |candidate: &str| {
        let utf16 = candidate.encode_utf16().collect::<Vec<_>>();
        regex.find_from_utf16(&utf16, 0).next().is_some()
    };
    if multiple_email {
        value
            .split(',')
            .map(|item| item.trim_matches(|c: char| c.is_ascii_whitespace()))
            .filter(|item| !item.is_empty())
            .any(|item| !matches(item))
    } else {
        !matches(value)
    }
}

fn input_type_supports_required(kind: &str) -> bool {
    !matches!(
        kind,
        "hidden" | "range" | "color" | "submit" | "reset" | "button" | "image"
    )
}
fn input_type_supports_readonly(kind: &str) -> bool {
    matches!(
        kind,
        "text"
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
    )
}
fn input_type_supports_length(kind: &str) -> bool {
    matches!(
        kind,
        "text" | "search" | "url" | "tel" | "email" | "password"
    )
}
fn input_type_supports_bad_input(kind: &str) -> bool {
    matches!(
        kind,
        "date" | "month" | "week" | "time" | "datetime-local" | "number"
    )
}
fn parse_nonnegative(raw: &str) -> Option<u32> {
    let raw = raw.trim_start_matches([' ', '\t', '\n', '\r', '\u{000c}']);
    let value = raw.parse::<u32>().ok()?;
    Some(value)
}
fn parse_html_float(value: &str) -> Option<f64> {
    crate::forms::parse_html_float(value)
}
fn utf16_len(value: &str) -> u32 {
    value.encode_utf16().count().min(u32::MAX as usize) as u32
}
fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n")
}
fn valid_email(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    if local.is_empty() || value.matches('@').count() != 1 {
        return false;
    }
    if !local
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b".!#$%&'*+/=?^_`{|}~-".contains(&byte))
    {
        return false;
    }
    !domain.is_empty()
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
                && label.as_bytes()[0].is_ascii_alphanumeric()
                && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
        })
}
fn valid_url(value: &str) -> bool {
    url::Url::parse(value).is_ok()
}
fn is_calendar_kind(kind: &str) -> bool {
    matches!(kind, "date" | "month" | "week" | "datetime-local")
}
fn compare_calendar_values(kind: &str, value: &str, bound: &str) -> Option<Ordering> {
    match kind {
        "date" => compare_dates(value, bound),
        "month" => {
            if !crate::forms::valid_month(value) || !crate::forms::valid_month(bound) {
                return None;
            }
            let (value_year, value_month) = value.split_once('-')?;
            let (bound_year, bound_month) = bound.split_once('-')?;
            Some(compare_years(value_year, bound_year).then_with(|| value_month.cmp(bound_month)))
        },
        "week" => {
            if !crate::forms::valid_week(value) || !crate::forms::valid_week(bound) {
                return None;
            }
            let (value_year, value_week) = value.split_once("-W")?;
            let (bound_year, bound_week) = bound.split_once("-W")?;
            Some(compare_years(value_year, bound_year).then_with(|| value_week.cmp(bound_week)))
        },
        "datetime-local" => {
            let (value_date, value_time) = datetime_parts(value)?;
            let (bound_date, bound_time) = datetime_parts(bound)?;
            if !crate::forms::valid_time(value_time) || !crate::forms::valid_time(bound_time) {
                return None;
            }
            let date_order = compare_dates(value_date, bound_date)?;
            let time_order = compare_times(value_time, bound_time)?;
            Some(date_order.then(time_order))
        },
        _ => None,
    }
}
fn compare_dates(value: &str, bound: &str) -> Option<Ordering> {
    if !crate::forms::valid_date(value) || !crate::forms::valid_date(bound) {
        return None;
    }
    let (value_year, value_month, value_day) = date_parts(value)?;
    let (bound_year, bound_month, bound_day) = date_parts(bound)?;
    Some(
        compare_years(value_year, bound_year)
            .then_with(|| value_month.cmp(&bound_month))
            .then_with(|| value_day.cmp(&bound_day)),
    )
}
fn date_parts(value: &str) -> Option<(&str, u32, u32)> {
    let (year, remainder) = value.split_once('-')?;
    let (month, day) = remainder.split_once('-')?;
    Some((year, month.parse().ok()?, day.parse().ok()?))
}
fn compare_years(value: &str, bound: &str) -> Ordering {
    let value = value.trim_start_matches('0');
    let bound = bound.trim_start_matches('0');
    value.len().cmp(&bound.len()).then_with(|| value.cmp(bound))
}
fn compare_times(value: &str, bound: &str) -> Option<Ordering> {
    if !crate::forms::valid_time(value) || !crate::forms::valid_time(bound) {
        return None;
    }
    let (value_hour, value_minute, value_second, value_fraction) = time_parts(value)?;
    let (bound_hour, bound_minute, bound_second, bound_fraction) = time_parts(bound)?;
    Some(
        value_hour
            .cmp(&bound_hour)
            .then_with(|| value_minute.cmp(&bound_minute))
            .then_with(|| value_second.cmp(&bound_second))
            .then_with(|| compare_fractions(value_fraction, bound_fraction)),
    )
}
fn time_parts(value: &str) -> Option<(u32, u32, u32, &str)> {
    let (clock, fraction) = value.split_once('.').unwrap_or((value, ""));
    let mut parts = clock.split(':');
    let hour = parts.next()?.parse().ok()?;
    let minute = parts.next()?.parse().ok()?;
    let second = match parts.next() {
        Some(second) => second.parse().ok()?,
        None => 0,
    };
    Some((hour, minute, second, fraction))
}
fn compare_fractions(value: &str, bound: &str) -> Ordering {
    let length = value.len().max(bound.len());
    for i in 0..length {
        let left = value.as_bytes().get(i).copied().unwrap_or(b'0');
        let right = bound.as_bytes().get(i).copied().unwrap_or(b'0');
        match left.cmp(&right) {
            Ordering::Equal => {},
            other => return other,
        }
    }
    Ordering::Equal
}
fn parse_month(s: &str) -> Option<f64> {
    if !crate::forms::valid_month(s) {
        return None;
    }
    let (year, month) = s.split_once('-')?;
    let year = year.parse::<f64>().ok()?;
    let month = month.parse::<u32>().ok()?;
    let value = (year - 1970.0) * 12.0 + f64::from(month) - 1.0;
    value.is_finite().then_some(value)
}
fn date_days(s: &str) -> Option<f64> {
    if !crate::forms::valid_date(s) {
        return None;
    }
    let mut parts = s.split('-');
    let year = parts.next()?.parse::<f64>().ok()?;
    let month = parts.next()?.parse::<u32>().ok()?;
    let day = parts.next()?.parse::<u32>().ok()?;
    let days = days_from_civil(year, month, day);
    days.is_finite().then_some(days)
}
fn week_monday_days(s: &str) -> Option<f64> {
    if !crate::forms::valid_week(s) {
        return None;
    }
    let (year, week) = s.split_once("-W")?;
    let year = year.parse::<f64>().ok()?;
    let week = week.parse::<u32>().ok()?;
    let jan4 = days_from_civil(year, 1, 4);
    let monday = jan4 - (jan4 + 3.0).rem_euclid(7.0);
    let days = monday + f64::from(week - 1) * 7.0;
    days.is_finite().then_some(days)
}
fn parse_time_millis(s: &str) -> Option<f64> {
    if !crate::forms::valid_time(s) {
        return None;
    }
    let (clock, fraction) = s.split_once('.').unwrap_or((s, ""));
    let mut parts = clock.split(':');
    let hour = parts.next()?.parse::<u32>().ok()?;
    let minute = parts.next()?.parse::<u32>().ok()?;
    let second = match parts.next() {
        Some(second) => second.parse::<u32>().ok()?,
        None => 0,
    };
    let fraction = if fraction.is_empty() {
        0.0
    } else {
        format!("0.{fraction}").parse::<f64>().ok()?
    };
    let millis = (f64::from((hour * 60 + minute) * 60 + second) + fraction) * 1000.0;
    millis.is_finite().then_some(millis)
}
fn datetime_parts(s: &str) -> Option<(&str, &str)> {
    s.split_once('T').or_else(|| s.split_once(' '))
}
fn parse_datetime_millis(s: &str) -> Option<f64> {
    let (date, time) = datetime_parts(s)?;
    let millis = date_days(date)? * 86_400_000.0 + parse_time_millis(time)?;
    millis.is_finite().then_some(millis)
}
fn days_from_civil(year: f64, month: u32, day: u32) -> f64 {
    let y = year - if month <= 2 { 1.0 } else { 0.0 };
    let era = (y / 400.0).floor();
    let yoe = y - era * 400.0;
    let mp = f64::from(month) + if month > 2 { -3.0 } else { 9.0 };
    let doy = ((153.0 * mp + 2.0) / 5.0).floor() + f64::from(day) - 1.0;
    let doe = yoe * 365.0 + (yoe / 4.0).floor() - (yoe / 100.0).floor() + doy;
    era * 146_097.0 + doe - 719_468.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout_dom_api::{
        FormControlInteractionState, FormControlState, LayoutDom, LayoutDomMut, SelectOptionState,
        ValidityState,
    };
    use markup5ever::{QualName, local_name, ns};

    fn html(local: markup5ever::LocalName) -> QualName {
        QualName::new(None, ns!(html), local)
    }

    fn attr(local: markup5ever::LocalName) -> QualName {
        QualName::new(None, ns!(), local)
    }

    #[test]
    fn validation_review_whitespace_option_is_a_required_placeholder() {
        let mut dom = ScriptedDom::new();
        let select = dom.create_element(html(local_name!("select")));
        dom.set_attribute(select, attr(local_name!("required")), "");
        let option = dom.create_element(html(local_name!("option")));
        dom.set_text_content(option, " \t\n\u{000c}\r ");
        dom.append_child(select, option);
        assert!(
            dom.form_control_validity(select)
                .unwrap()
                .flags
                .value_missing
        );
        assert_eq!(dom.form_control_value(select).as_deref(), Some(""));
        dom.set_text_content(option, "\u{00a0}");
        assert!(
            !dom.form_control_validity(select)
                .unwrap()
                .flags
                .value_missing
        );
    }

    #[test]
    fn validation_review_option_fallback_value_matches_the_native_setter() {
        let mut dom = ScriptedDom::new();
        let select = dom.create_element(html(local_name!("select")));
        let option = dom.create_element(html(local_name!("option")));
        dom.set_text_content(option, " child \n oak \t ");
        dom.append_child(select, option);
        assert_eq!(dom.form_control_value(select).as_deref(), Some("child oak"));
        dom.set_select_selected_index(select, -1);
        dom.set_form_control_value(select, "child oak").unwrap();
        assert_eq!(dom.select_selected_index(select), 0);
        dom.set_attribute(option, attr(local_name!("value")), " child ");
        assert_eq!(dom.form_control_value(select).as_deref(), Some(" child "));
        dom.set_form_control_value(select, "child").unwrap();
        assert_eq!(dom.select_selected_index(select), -1);
        dom.set_form_control_value(select, " child ").unwrap();
        assert_eq!(dom.select_selected_index(select), 0);
    }

    #[test]
    fn validation_review_datetime_bounds_accept_a_space_separator() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(html(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "datetime-local");
        dom.set_attribute(input, attr(local_name!("min")), "2024-01-01 12:00");
        dom.set_attribute(input, attr(local_name!("max")), "2024-01-01 13:00");
        dom.set_form_control_value(input, "2024-01-01T11:00")
            .unwrap();
        assert!(
            dom.form_control_validity(input)
                .unwrap()
                .flags
                .range_underflow
        );
        dom.set_form_control_value(input, "2024-01-01T14:00")
            .unwrap();
        assert!(
            dom.form_control_validity(input)
                .unwrap()
                .flags
                .range_overflow
        );
    }

    #[test]
    fn validation_review_datetime_step_base_accepts_a_space_separator() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(html(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "datetime-local");
        dom.set_attribute(input, attr(local_name!("min")), "2024-01-01 12:00:30");
        dom.set_attribute(input, attr(local_name!("step")), "60");
        dom.set_form_control_value(input, "2024-01-01T12:01:30")
            .unwrap();
        assert!(
            !dom.form_control_validity(input)
                .unwrap()
                .flags
                .step_mismatch
        );
        dom.remove_attribute(input, attr(local_name!("min")));
        dom.set_attribute(input, attr(local_name!("value")), "2024-01-01 12:00:30");
        assert!(
            !dom.form_control_validity(input)
                .unwrap()
                .flags
                .step_mismatch
        );
        dom.set_form_control_value(input, "2024-01-01T12:01:31")
            .unwrap();
        assert!(
            dom.form_control_validity(input)
                .unwrap()
                .flags
                .step_mismatch
        );
    }

    #[test]
    fn user_bad_number_edit_is_distinct_from_script_value_assignment() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(html(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "number");
        assert!(dom.begin_form_control_user_edit(input));
        assert!(dom.set_form_control_user_value(input, "1e", Some(2), Some(2)));
        assert!(dom.form_control_validity(input).unwrap().flags.bad_input);
        assert_eq!(
            dom.form_control_interaction_state(input)
                .unwrap()
                .draft_value
                .as_deref(),
            Some("1e")
        );
        let editing = dom.form_control_editing_value(input).unwrap();
        assert_eq!(editing.value, "1e");
        assert_eq!(editing.selection_start, Some(2));
        assert!(dom.set_form_control_editing_selection(
            input,
            Some(1),
            Some(1),
            SelectionDirection::Backward,
        ));
        let editing = dom.form_control_editing_value(input).unwrap();
        assert_eq!(editing.selection_direction, SelectionDirection::Backward);
        assert!(
            dom.form_control_interaction_state(input)
                .unwrap()
                .user_edit_pending
        );
        assert!(dom.commit_form_control_user_edit(input));
        assert!(dom.form_control_validity(input).unwrap().user_validity);
        assert!(dom.form_control_validity(input).unwrap().flags.bad_input);
        assert_eq!(
            dom.form_control_interaction_state(input)
                .unwrap()
                .draft_value
                .as_deref(),
            Some("1e")
        );

        dom.set_form_control_value(input, "2").unwrap();
        let validity = dom.form_control_validity(input).unwrap();
        assert!(!validity.flags.bad_input);
        assert!(validity.user_validity);
        assert!(
            !dom.form_control_interaction_state(input)
                .unwrap()
                .last_change_by_user
        );
    }

    #[test]
    fn restoring_focus_value_before_commit_does_not_flip_user_validity() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(html(local_name!("input")));
        assert!(dom.begin_form_control_user_edit(input));
        assert!(dom.set_form_control_user_value(input, "draft", None, None));
        assert!(dom.set_form_control_user_value(input, "", None, None));
        assert!(dom.commit_form_control_user_edit(input));
        let validity = dom.form_control_validity(input).unwrap();
        assert!(!validity.user_validity);
        assert!(
            dom.form_control_interaction_state(input)
                .unwrap()
                .last_change_by_user
        );
    }

    #[test]
    fn input_type_change_drops_number_draft_but_preserves_validity_metadata() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(html(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "number");
        dom.begin_form_control_user_edit(input);
        dom.set_form_control_user_value(input, "2", Some(1), Some(1));
        dom.commit_form_control_user_edit(input);
        dom.begin_form_control_user_edit(input);
        dom.set_form_control_user_value(input, "1e", Some(2), Some(2));
        dom.set_custom_validity(input, "Keep this custom message");
        assert!(dom.form_control_validity(input).unwrap().flags.bad_input);

        dom.set_attribute(input, attr(local_name!("type")), "text");
        let validity = dom.form_control_validity(input).unwrap();
        assert!(!validity.flags.bad_input);
        assert!(validity.user_validity);
        assert!(validity.flags.custom_error);
        assert_eq!(
            dom.form_control_custom_validity_message(input).as_deref(),
            Some("Keep this custom message")
        );
        assert_eq!(dom.form_control_editing_value(input).unwrap().value, "");

        dom.set_attribute(input, attr(local_name!("type")), "number");
        assert!(!dom.form_control_validity(input).unwrap().flags.bad_input);
        assert_eq!(dom.form_control_editing_value(input).unwrap().value, "");
    }

    #[test]
    fn type_changes_preserve_user_provenance_only_when_api_value_is_unchanged() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(html(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "number");
        dom.set_attribute(input, attr(local_name!("maxlength")), "2");
        dom.begin_form_control_user_edit(input);
        dom.set_form_control_user_value(input, "123", Some(3), Some(3));
        dom.commit_form_control_user_edit(input);
        assert!(!dom.form_control_validity(input).unwrap().flags.too_long);

        dom.set_attribute(input, attr(local_name!("type")), "text");
        assert_eq!(dom.form_control_value(input).as_deref(), Some("123"));
        assert!(dom.form_control_validity(input).unwrap().flags.too_long);

        dom.set_attribute(input, attr(local_name!("type")), "search");
        assert!(dom.form_control_validity(input).unwrap().flags.too_long);

        dom.set_attribute(input, attr(local_name!("type")), "number");
        dom.set_attribute(input, attr(local_name!("type")), "text");
        dom.begin_form_control_user_edit(input);
        dom.set_form_control_user_value(input, "not-a-number", Some(12), Some(12));
        dom.commit_form_control_user_edit(input);
        assert!(dom.form_control_validity(input).unwrap().flags.too_long);
        dom.set_attribute(input, attr(local_name!("type")), "number");
        assert_eq!(dom.form_control_value(input).as_deref(), Some(""));
        assert!(
            !dom.form_control_interaction_state(input)
                .unwrap()
                .last_change_by_user
        );
        assert!(!dom.form_control_validity(input).unwrap().flags.bad_input);
    }

    #[test]
    fn date_time_controls_keep_invalid_drafts_until_user_or_script_replaces_them() {
        let cases = [
            ("date", "2026-10-", "2026-10-07"),
            ("month", "2026-", "2026-10"),
            ("week", "2026-W", "2026-W41"),
            ("time", "12:", "12:30"),
            ("datetime-local", "2026-10-07T", "2026-10-07T12:30"),
        ];
        let mut dom = ScriptedDom::new();
        for (kind, partial, valid) in cases {
            let input = dom.create_element(html(local_name!("input")));
            dom.set_attribute(input, attr(local_name!("type")), kind);
            dom.begin_form_control_user_edit(input);
            let partial_end = partial.encode_utf16().count() as u32;
            dom.set_form_control_user_value(input, partial, Some(partial_end), Some(partial_end));
            assert!(
                dom.form_control_validity(input).unwrap().flags.bad_input,
                "{kind} partial user input should be bad input"
            );
            assert_eq!(
                dom.form_control_editing_value(input).unwrap().value,
                partial,
                "{kind} partial draft should remain editable"
            );

            let valid_end = valid.encode_utf16().count() as u32;
            dom.set_form_control_user_value(input, valid, Some(valid_end), Some(valid_end));
            assert!(!dom.form_control_validity(input).unwrap().flags.bad_input);
            assert_eq!(dom.form_control_editing_value(input).unwrap().value, valid);

            dom.set_form_control_user_value(input, partial, Some(partial_end), Some(partial_end));
            dom.set_form_control_value(input, valid).unwrap();
            let interaction = dom.form_control_interaction_state(input).unwrap();
            assert!(!interaction.bad_input);
            assert!(interaction.draft_value.is_none());
            assert_eq!(dom.form_control_editing_value(input).unwrap().value, valid);
        }
    }

    #[test]
    fn length_flags_require_both_dirty_value_and_user_provenance() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(html(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "text");
        dom.set_attribute(input, attr(local_name!("maxlength")), "2");
        dom.set_attribute(input, attr(local_name!("minlength")), "5");
        dom.set_form_control_state(
            input,
            FormControlState {
                value: "long".to_owned(),
                dirty_value: false,
                ..FormControlState::default()
            },
        );
        dom.set_form_control_interaction_state(
            input,
            FormControlInteractionState {
                last_change_by_user: true,
                ..FormControlInteractionState::default()
            },
        );
        let flags = dom.form_control_validity(input).unwrap().flags;
        assert!(!flags.too_long);
        assert!(!flags.too_short);
    }

    #[test]
    fn form_association_and_disabled_fieldset_legend_exception_are_native() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let form = dom.create_element(html(local_name!("form")));
        dom.set_attribute(form, attr(local_name!("id")), "f");
        dom.attach_silent(root, form);
        let fieldset = dom.create_element(html(local_name!("fieldset")));
        dom.set_attribute(fieldset, attr(local_name!("disabled")), "");
        dom.attach_silent(form, fieldset);
        let legend = dom.create_element(html(local_name!("legend")));
        dom.attach_silent(fieldset, legend);
        let exempt = dom.create_element(html(local_name!("input")));
        dom.attach_silent(legend, exempt);
        let barred = dom.create_element(html(local_name!("input")));
        dom.attach_silent(fieldset, barred);

        assert_eq!(dom.form_control_form_owner(exempt), Some(form));
        assert_eq!(
            dom.associated_form_controls(form),
            vec![fieldset, exempt, barred]
        );
        assert!(dom.form_control_validity(exempt).unwrap().will_validate);
        assert!(!dom.form_control_validity(barred).unwrap().will_validate);
    }

    #[test]
    fn decimal_step_and_date_range_are_derived_from_current_value() {
        let mut dom = ScriptedDom::new();
        let number = dom.create_element(html(local_name!("input")));
        dom.set_attribute(number, attr(local_name!("type")), "number");
        dom.set_attribute(number, attr(local_name!("step")), "0.1");
        dom.set_form_control_value(number, "0.3").unwrap();
        assert!(
            !dom.form_control_validity(number)
                .unwrap()
                .flags
                .step_mismatch
        );
        dom.set_form_control_value(number, "0.35").unwrap();
        assert!(
            dom.form_control_validity(number)
                .unwrap()
                .flags
                .step_mismatch
        );
        dom.set_attribute(number, attr(local_name!("step")), "1");
        dom.set_form_control_value(number, "1e-10").unwrap();
        assert!(
            dom.form_control_validity(number)
                .unwrap()
                .flags
                .step_mismatch
        );
        dom.set_form_control_value(number, "0").unwrap();
        assert!(
            !dom.form_control_validity(number)
                .unwrap()
                .flags
                .step_mismatch
        );

        let date = dom.create_element(html(local_name!("input")));
        dom.set_attribute(date, attr(local_name!("type")), "date");
        dom.set_attribute(date, attr(local_name!("min")), "2026-10-01");
        dom.set_form_control_value(date, "2026-09-30").unwrap();
        assert!(
            dom.form_control_validity(date)
                .unwrap()
                .flags
                .range_underflow
        );
        assert!(
            !ValidityState {
                range_underflow: true,
                ..ValidityState::default()
            }
            .valid()
        );
    }

    #[test]
    fn single_select_keeps_selectedness_separate_from_attributes() {
        let mut dom = ScriptedDom::new();
        let select = dom.create_element(html(local_name!("select")));
        let first = dom.create_element(html(local_name!("option")));
        let second = dom.create_element(html(local_name!("option")));
        dom.append_child(select, first);
        dom.append_child(select, second);
        assert!(dom.option_selected_state(first).unwrap().selected);
        assert!(!dom.option_selected_state(second).unwrap().selected);
        assert!(dom.set_option_selected(second, true, true));
        assert!(!dom.option_selected_state(first).unwrap().selected);
        assert!(dom.option_selected_state(second).unwrap().selected);
        assert!(dom.option_selected_state(second).unwrap().dirty);
    }

    #[test]
    fn select_reset_clears_dirty_selectedness_for_later_default_attribute_changes() {
        let mut dom = ScriptedDom::new();
        let select = dom.create_element(html(local_name!("select")));
        let first = dom.create_element(html(local_name!("option")));
        let second = dom.create_element(html(local_name!("option")));
        dom.append_child(select, first);
        dom.append_child(select, second);
        assert!(dom.set_option_selected(second, true, true));
        assert!(!dom.option_selected_state(first).unwrap().selected);
        assert!(dom.reset_form_control(select));
        assert!(dom.option_selected_state(first).unwrap().selected);
        assert!(!dom.option_selected_state(second).unwrap().dirty);
        dom.set_attribute(second, attr(local_name!("selected")), "");
        assert!(dom.option_selected_state(second).unwrap().selected);
        assert!(!dom.option_selected_state(first).unwrap().selected);
    }

    #[test]
    fn will_validate_and_value_missing_follow_applicable_states() {
        let mut dom = ScriptedDom::new();
        let submit = dom.create_element(html(local_name!("input")));
        dom.set_attribute(submit, attr(local_name!("type")), "submit");
        assert!(dom.form_control_validity(submit).unwrap().will_validate);
        let image = dom.create_element(html(local_name!("input")));
        dom.set_attribute(image, attr(local_name!("type")), "image");
        assert!(dom.form_control_validity(image).unwrap().will_validate);
        let reset = dom.create_element(html(local_name!("input")));
        dom.set_attribute(reset, attr(local_name!("type")), "reset");
        assert!(!dom.form_control_validity(reset).unwrap().will_validate);

        let object = dom.create_element(html(local_name!("object")));
        assert!(!dom.form_control_validity(object).unwrap().will_validate);
        assert!(dom.set_custom_validity(object, "custom"));
        assert!(
            dom.form_control_validity(object)
                .unwrap()
                .flags
                .custom_error
        );
        let button = dom.create_element(html(local_name!("button")));
        assert!(dom.form_control_validity(button).unwrap().will_validate);
        dom.set_attribute(button, attr(local_name!("type")), "reset");
        assert!(!dom.form_control_validity(button).unwrap().will_validate);

        let command_button = dom.create_element(html(local_name!("button")));
        dom.set_attribute(command_button, attr(local_name!("command")), "show-modal");
        assert!(
            !dom.form_control_validity(command_button)
                .unwrap()
                .will_validate
        );
        let commandfor_button = dom.create_element(html(local_name!("button")));
        dom.set_attribute(commandfor_button, attr(local_name!("commandfor")), "dialog");
        assert!(
            !dom.form_control_validity(commandfor_button)
                .unwrap()
                .will_validate
        );
        let select = dom.create_element(html(local_name!("select")));
        let auto_button = dom.create_element(html(local_name!("button")));
        dom.append_child(select, auto_button);
        assert!(
            !dom.form_control_validity(auto_button)
                .unwrap()
                .will_validate
        );

        let explicit_submit = dom.create_element(html(local_name!("button")));
        dom.set_attribute(explicit_submit, attr(local_name!("type")), "submit");
        dom.set_attribute(explicit_submit, attr(local_name!("command")), "show-modal");
        dom.append_child(select, explicit_submit);
        assert!(
            dom.form_control_validity(explicit_submit)
                .unwrap()
                .will_validate
        );

        let invalid_command_button = dom.create_element(html(local_name!("button")));
        dom.set_attribute(
            invalid_command_button,
            attr(local_name!("type")),
            "not-a-type",
        );
        dom.set_attribute(
            invalid_command_button,
            attr(local_name!("command")),
            "show-modal",
        );
        assert!(
            !dom.form_control_validity(invalid_command_button)
                .unwrap()
                .will_validate
        );
        let invalid_default_button = dom.create_element(html(local_name!("button")));
        dom.set_attribute(
            invalid_default_button,
            attr(local_name!("type")),
            "not-a-type",
        );
        assert!(
            dom.form_control_validity(invalid_default_button)
                .unwrap()
                .will_validate
        );

        let color = dom.create_element(html(local_name!("input")));
        dom.set_attribute(color, attr(local_name!("type")), "color");
        dom.set_attribute(color, attr(local_name!("readonly")), "");
        assert!(!dom.form_control_validity(color).unwrap().will_validate);

        let datalist = dom.create_element(html(local_name!("datalist")));
        let datalist_button = dom.create_element(html(local_name!("button")));
        dom.append_child(datalist, datalist_button);
        assert!(
            !dom.form_control_validity(datalist_button)
                .unwrap()
                .will_validate
        );

        let text = dom.create_element(html(local_name!("input")));
        dom.set_attribute(text, attr(local_name!("required")), "");
        dom.set_attribute(text, attr(local_name!("disabled")), "");
        assert!(!dom.form_control_validity(text).unwrap().flags.value_missing);
    }

    #[test]
    fn radio_group_requiredness_and_textarea_length_are_shared() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let first = dom.create_element(html(local_name!("input")));
        let second = dom.create_element(html(local_name!("input")));
        for radio in [first, second] {
            dom.set_attribute(radio, attr(local_name!("type")), "radio");
            dom.set_attribute(radio, attr(local_name!("name")), "choice");
            dom.attach_silent(root, radio);
        }
        dom.set_attribute(first, attr(local_name!("required")), "");
        assert!(
            dom.form_control_validity(second)
                .unwrap()
                .flags
                .value_missing
        );

        let textarea = dom.create_element(html(local_name!("textarea")));
        dom.set_attribute(textarea, attr(local_name!("maxlength")), "2");
        dom.begin_form_control_user_edit(textarea);
        dom.set_form_control_user_value(textarea, "long", None, None);
        assert!(dom.form_control_validity(textarea).unwrap().flags.too_long);
    }

    #[test]
    fn disabled_non_value_mode_required_controls_retain_value_missing() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        let checkbox = dom.create_element(html(local_name!("input")));
        dom.set_attribute(checkbox, attr(local_name!("type")), "checkbox");
        dom.set_attribute(checkbox, attr(local_name!("required")), "");
        dom.set_attribute(checkbox, attr(local_name!("disabled")), "");
        assert!(
            dom.form_control_validity(checkbox)
                .unwrap()
                .flags
                .value_missing
        );

        let file = dom.create_element(html(local_name!("input")));
        dom.set_attribute(file, attr(local_name!("type")), "file");
        dom.set_attribute(file, attr(local_name!("required")), "");
        dom.set_attribute(file, attr(local_name!("disabled")), "");
        assert!(dom.form_control_validity(file).unwrap().flags.value_missing);

        let select = dom.create_element(html(local_name!("select")));
        dom.set_attribute(select, attr(local_name!("required")), "");
        dom.set_attribute(select, attr(local_name!("disabled")), "");
        let placeholder = dom.create_element(html(local_name!("option")));
        dom.append_child(select, placeholder);
        assert!(
            dom.form_control_validity(select)
                .unwrap()
                .flags
                .value_missing
        );

        let required_radio = dom.create_element(html(local_name!("input")));
        let peer_radio = dom.create_element(html(local_name!("input")));
        for radio in [required_radio, peer_radio] {
            dom.set_attribute(radio, attr(local_name!("type")), "radio");
            dom.set_attribute(radio, attr(local_name!("name")), "group");
            dom.attach_silent(root, radio);
        }
        dom.set_attribute(required_radio, attr(local_name!("required")), "");
        dom.set_attribute(required_radio, attr(local_name!("disabled")), "");
        assert!(
            dom.form_control_validity(required_radio)
                .unwrap()
                .flags
                .value_missing
        );
        assert!(
            dom.form_control_validity(peer_radio)
                .unwrap()
                .flags
                .value_missing
        );

        let nameless = dom.create_element(html(local_name!("input")));
        dom.set_attribute(nameless, attr(local_name!("type")), "radio");
        dom.set_attribute(nameless, attr(local_name!("required")), "");
        assert!(
            !dom.form_control_validity(nameless)
                .unwrap()
                .flags
                .value_missing
        );
    }

    #[test]
    fn email_and_time_constraints_use_html_syntax() {
        let mut dom = ScriptedDom::new();
        let email = dom.create_element(html(local_name!("input")));
        dom.set_attribute(email, attr(local_name!("type")), "email");
        dom.set_form_control_value(email, ".a..b@localhost")
            .unwrap();
        assert!(
            !dom.form_control_validity(email)
                .unwrap()
                .flags
                .type_mismatch
        );

        let time = dom.create_element(html(local_name!("input")));
        dom.set_attribute(time, attr(local_name!("type")), "time");
        dom.set_attribute(time, attr(local_name!("min")), "21:00");
        dom.set_attribute(time, attr(local_name!("max")), "06:00");
        dom.set_form_control_value(time, "12:00").unwrap();
        let flags = dom.form_control_validity(time).unwrap().flags;
        assert!(flags.range_underflow && flags.range_overflow);
        dom.set_form_control_value(time, "00:00").unwrap();
        let flags = dom.form_control_validity(time).unwrap().flags;
        assert!(!flags.range_underflow && !flags.range_overflow);
    }

    #[test]
    fn pattern_nested_negated_unicode_sets_keep_the_complement() {
        for (pattern, matching, mismatching) in [
            ("[[a-z]&&[^aeiou]]+", "bcdf", "abc"),
            ("[[^aeiou]&&[a-z]]+", "bcdf", "abc"),
            (r"[[\u{1F600}-\u{1F601}]&&[^\u{1F601}]]", "😀", "😁"),
            ("[[a-z]--[^aeiou]]+", "aeiou", "bcdf"),
        ] {
            assert!(
                !pattern_mismatch(pattern, matching, false),
                "{pattern}: {matching}"
            );
            assert!(
                pattern_mismatch(pattern, mismatching, false),
                "{pattern}: {mismatching}"
            );
        }
        assert!(
            !pattern_mismatch("[a-z&&[^aeiou]]+", "abc", false),
            "invalid v syntax must be omitted"
        );
    }

    #[test]
    fn pattern_uses_unicode_sets_and_full_value_rules() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(html(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "text");
        dom.set_attribute(input, attr(local_name!("pattern")), ".");
        dom.set_form_control_value(input, "😀").unwrap();
        assert!(
            !dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );

        dom.set_attribute(input, attr(local_name!("pattern")), r"\u{1F600}");
        assert!(
            !dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );
        dom.set_form_control_value(input, "😃").unwrap();
        assert!(
            dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );

        dom.set_attribute(input, attr(local_name!("pattern")), "[\\p{ASCII}&&[A-Z]]+");
        dom.set_form_control_value(input, "ABC").unwrap();
        assert!(
            !dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );
        dom.set_form_control_value(input, "AbC").unwrap();
        assert!(
            dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );

        dom.set_attribute(input, attr(local_name!("pattern")), "[\\p{ASCII}--[A-Z]]+");
        dom.set_form_control_value(input, "abc").unwrap();
        assert!(
            !dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );
        dom.set_form_control_value(input, "ABC").unwrap();
        assert!(
            dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );

        dom.set_attribute(input, attr(local_name!("pattern")), r"[\q{a|bc}]");
        dom.set_form_control_value(input, "bc").unwrap();
        assert!(
            !dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );
        dom.set_form_control_value(input, "b").unwrap();
        assert!(
            dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );

        dom.set_attribute(input, attr(local_name!("pattern")), "(a+)+");
        dom.set_form_control_value(input, "aaaaaaaaaaaab").unwrap();
        assert!(
            dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );

        dom.set_attribute(input, attr(local_name!("pattern")), "[A-Z]+");
        dom.set_form_control_value(input, "").unwrap();
        assert!(
            !dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );
        dom.set_form_control_value(input, "ABCx").unwrap();
        assert!(
            dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );

        // These are valid legacy-u character classes but invalid v classes;
        // HTML ignores the invalid compiled pattern instead of flagging value.
        for invalid_v_pattern in ["[(]", "[[]", "[&&]"] {
            dom.set_attribute(input, attr(local_name!("pattern")), invalid_v_pattern);
            assert!(
                !dom.form_control_validity(input)
                    .unwrap()
                    .flags
                    .pattern_mismatch,
                "invalid v pattern {invalid_v_pattern:?} should be ignored"
            );
        }

        dom.set_attribute(input, attr(local_name!("type")), "date");
        dom.set_form_control_value(input, "2026-10-08").unwrap();
        assert!(
            !dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );
    }

    #[test]
    fn multiple_email_pattern_matches_each_nonempty_item() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(html(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "email");
        dom.set_attribute(input, attr(local_name!("multiple")), "");
        dom.set_attribute(input, attr(local_name!("pattern")), "[a-z]@[a-z]");
        dom.set_form_control_value(input, "a@b, c@d").unwrap();
        assert!(
            !dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );
        dom.set_form_control_value(input, "a@b, C@d").unwrap();
        assert!(
            dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );
        dom.set_form_control_value(input, "").unwrap();
        assert!(
            !dom.form_control_validity(input)
                .unwrap()
                .flags
                .pattern_mismatch
        );
    }

    #[test]
    fn url_type_mismatch_uses_the_locked_whatwg_url_parser() {
        let mut dom = ScriptedDom::new();
        let input = dom.create_element(html(local_name!("input")));
        dom.set_attribute(input, attr(local_name!("type")), "url");
        for (value, expected) in [
            ("http://", true),
            ("/relative/path", true),
            ("https://example.test/path", false),
            ("mailto:user@example.test", false),
        ] {
            dom.set_form_control_value(input, value).unwrap();
            assert_eq!(
                dom.form_control_validity(input)
                    .unwrap()
                    .flags
                    .type_mismatch,
                expected
            );
        }
    }

    #[test]
    fn required_select_placeholder_allows_wrappers_but_not_optgroup() {
        let mut dom = ScriptedDom::new();
        let select = dom.create_element(html(local_name!("select")));
        dom.set_attribute(select, attr(local_name!("required")), "");
        let group = dom.create_element(html(local_name!("optgroup")));
        let grouped_empty = dom.create_element(html(local_name!("option")));
        dom.append_child(group, grouped_empty);
        dom.append_child(select, group);
        assert!(
            !dom.form_control_validity(select)
                .unwrap()
                .flags
                .value_missing
        );

        let wrapped_select = dom.create_element(html(local_name!("select")));
        dom.set_attribute(wrapped_select, attr(local_name!("required")), "");
        let wrapper = dom.create_element(html(local_name!("div")));
        let wrapped_empty = dom.create_element(html(local_name!("option")));
        dom.append_child(wrapper, wrapped_empty);
        dom.append_child(wrapped_select, wrapper);
        dom.set_option_selected(wrapped_empty, true, true);
        assert!(
            dom.form_control_validity(wrapped_select)
                .unwrap()
                .flags
                .value_missing
        );
    }

    #[test]
    fn required_select_size_above_one_does_not_get_selection_fallback() {
        let mut dom = ScriptedDom::new();
        let select = dom.create_element(html(local_name!("select")));
        dom.set_attribute(select, attr(local_name!("required")), "");
        dom.set_attribute(select, attr(local_name!("size")), "2");
        let option = dom.create_element(html(local_name!("option")));
        dom.append_child(select, option);
        assert_eq!(dom.select_selected_index(select), -1);
        assert!(
            dom.form_control_validity(select)
                .unwrap()
                .flags
                .value_missing
        );

        let zero_size = dom.create_element(html(local_name!("select")));
        dom.set_attribute(zero_size, attr(local_name!("size")), "0");
        let zero_size_option = dom.create_element(html(local_name!("option")));
        dom.append_child(zero_size, zero_size_option);
        assert_eq!(dom.select_selected_index(zero_size), -1);

        let parsed_prefix = dom.create_element(html(local_name!("select")));
        dom.set_attribute(parsed_prefix, attr(local_name!("size")), " \t+2ignored");
        let prefix_option = dom.create_element(html(local_name!("option")));
        dom.append_child(parsed_prefix, prefix_option);
        assert_eq!(dom.select_selected_index(parsed_prefix), -1);

        let invalid_size = dom.create_element(html(local_name!("select")));
        dom.set_attribute(invalid_size, attr(local_name!("size")), "-1");
        let invalid_size_option = dom.create_element(html(local_name!("option")));
        dom.append_child(invalid_size, invalid_size_option);
        assert_eq!(dom.select_selected_index(invalid_size), 0);
    }

    #[test]
    fn select_fallback_skips_disabled_options_and_disabled_optgroups() {
        let mut dom = ScriptedDom::new();
        let select = dom.create_element(html(local_name!("select")));
        let disabled_option = dom.create_element(html(local_name!("option")));
        dom.set_attribute(disabled_option, attr(local_name!("disabled")), "");
        dom.append_child(select, disabled_option);
        assert_eq!(dom.select_selected_index(select), -1);

        let group = dom.create_element(html(local_name!("optgroup")));
        dom.set_attribute(group, attr(local_name!("disabled")), "");
        let grouped = dom.create_element(html(local_name!("option")));
        dom.append_child(group, grouped);
        dom.append_child(select, group);
        assert_eq!(dom.select_selected_index(select), -1);

        let enabled = dom.create_element(html(local_name!("option")));
        dom.append_child(select, enabled);
        assert_eq!(dom.select_selected_index(select), 2);
        assert!(dom.option_selected_state(enabled).unwrap().selected);

        dom.set_select_selected_index(select, -1);
        dom.remove_attribute(group, attr(local_name!("disabled")));
        assert_eq!(dom.select_selected_index(select), 1);
        dom.set_select_selected_index(select, -1);
        dom.set_attribute(grouped, attr(local_name!("disabled")), "");
        assert_eq!(dom.select_selected_index(select), 2);
        dom.set_select_selected_index(select, -1);
        dom.set_attribute(enabled, attr(local_name!("disabled")), "");
        assert_eq!(dom.select_selected_index(select), -1);
    }

    #[test]
    fn select_reset_keeps_last_default_and_clears_option_dirtiness() {
        let mut dom = ScriptedDom::new();
        let select = dom.create_element(html(local_name!("select")));
        let options = (0..3)
            .map(|_| dom.create_element(html(local_name!("option"))))
            .collect::<Vec<_>>();
        for &option in &options {
            dom.set_attribute(option, attr(local_name!("selected")), "");
            dom.append_child(select, option);
        }
        assert_eq!(dom.select_selected_index(select), 2);
        assert!(dom.set_option_selected(options[0], true, true));
        for &option in &options {
            dom.set_option_selected_state(
                option,
                SelectOptionState {
                    selected: true,
                    dirty: false,
                },
            );
        }
        assert!(dom.reset_form_control(select));
        assert_eq!(dom.select_selected_index(select), 2);
        for (index, &option) in options.iter().enumerate() {
            let state = dom.option_selected_state(option).unwrap();
            assert_eq!(state.selected, index == 2);
            assert!(!state.dirty);
            assert!(
                dom.attribute(option, &ns!(), &markup5ever::LocalName::from("selected"))
                    .is_some()
            );
        }
    }

    #[test]
    fn removing_multiple_keeps_first_selected_option() {
        let mut dom = ScriptedDom::new();
        let select = dom.create_element(html(local_name!("select")));
        dom.set_attribute(select, attr(local_name!("multiple")), "");
        let first = dom.create_element(html(local_name!("option")));
        let second = dom.create_element(html(local_name!("option")));
        let third = dom.create_element(html(local_name!("option")));
        for option in [first, second, third] {
            dom.append_child(select, option);
        }
        dom.set_option_selected(first, true, true);
        dom.set_option_selected(second, true, true);
        dom.set_option_selected(third, true, true);
        dom.remove_attribute(select, attr(local_name!("multiple")));
        assert!(dom.option_selected_state(first).unwrap().selected);
        assert!(!dom.option_selected_state(second).unwrap().selected);
        assert!(!dom.option_selected_state(third).unwrap().selected);
    }

    #[test]
    fn option_selected_script_setter_asks_select_to_reset() {
        let mut dom = ScriptedDom::new();
        let single = dom.create_element(html(local_name!("select")));
        let first = dom.create_element(html(local_name!("option")));
        let second = dom.create_element(html(local_name!("option")));
        dom.append_child(single, first);
        dom.append_child(single, second);
        assert!(dom.option_selected_state(first).unwrap().selected);
        assert!(dom.set_option_selected_by_script(first, false));
        assert!(dom.option_selected_state(first).unwrap().selected);
        assert!(dom.option_selected_state(first).unwrap().dirty);

        let listbox = dom.create_element(html(local_name!("select")));
        dom.set_attribute(listbox, attr(local_name!("size")), "2");
        let listbox_option = dom.create_element(html(local_name!("option")));
        dom.append_child(listbox, listbox_option);
        assert!(dom.set_option_selected_by_script(listbox_option, true));
        assert!(dom.set_option_selected_by_script(listbox_option, false));
        assert_eq!(dom.select_selected_index(listbox), -1);

        let multiple = dom.create_element(html(local_name!("select")));
        dom.set_attribute(multiple, attr(local_name!("multiple")), "");
        let multiple_option = dom.create_element(html(local_name!("option")));
        dom.append_child(multiple, multiple_option);
        assert!(dom.set_option_selected_by_script(multiple_option, true));
        assert!(dom.set_option_selected_by_script(multiple_option, false));
        assert_eq!(dom.select_selected_index(multiple), -1);

        assert!(dom.set_select_selected_index(single, -1));
        assert_eq!(dom.select_selected_index(single), -1);
    }

    #[test]
    fn selected_options_joining_a_new_list_win_even_when_inserted_before_peers() {
        let mut dom = ScriptedDom::new();
        let select = dom.create_element(html(local_name!("select")));
        let peer = dom.create_element(html(local_name!("option")));
        dom.append_child(select, peer);
        dom.set_option_selected(peer, true, false);

        let inserted = dom.create_element(html(local_name!("option")));
        dom.set_option_selected(inserted, true, false);
        dom.insert_before(select, inserted, Some(peer));
        assert_eq!(dom.select_selected_index(select), 0);
        assert!(dom.option_selected_state(inserted).unwrap().selected);
        assert!(!dom.option_selected_state(peer).unwrap().selected);
        assert!(!dom.option_selected_state(peer).unwrap().dirty);

        let other_select = dom.create_element(html(local_name!("select")));
        let other_peer = dom.create_element(html(local_name!("option")));
        dom.append_child(other_select, other_peer);
        dom.set_option_selected(other_peer, true, false);
        let wrapper = dom.create_element(html(local_name!("div")));
        let subtree_option = dom.create_element(html(local_name!("option")));
        let subtree_last = dom.create_element(html(local_name!("option")));
        dom.set_option_selected(subtree_option, true, false);
        dom.set_option_selected(subtree_last, true, false);
        dom.append_child(wrapper, subtree_option);
        dom.append_child(wrapper, subtree_last);
        dom.append_child(other_select, wrapper);
        assert_eq!(dom.select_selected_index(other_select), 2);
        assert!(!dom.option_selected_state(subtree_option).unwrap().selected);
        assert!(dom.option_selected_state(subtree_last).unwrap().selected);
        assert!(!dom.option_selected_state(other_peer).unwrap().selected);
        assert!(!dom.option_selected_state(subtree_last).unwrap().dirty);

        // Removing a selected option from its former list restores that list's
        // fallback before the option joins the destination list.
        let source = dom.create_element(html(local_name!("select")));
        let source_fallback = dom.create_element(html(local_name!("option")));
        let moving = dom.create_element(html(local_name!("option")));
        dom.append_child(source, source_fallback);
        dom.append_child(source, moving);
        dom.set_option_selected(moving, true, false);
        dom.append_child(other_select, moving);
        assert_eq!(dom.select_selected_index(source), 0);
        assert!(dom.option_selected_state(source_fallback).unwrap().selected);
        assert!(dom.option_selected_state(moving).unwrap().selected);

        // A move within one option list changes order but keeps the same
        // selectedness; it does not treat the option as newly added.
        let same_list_select = dom.create_element(html(local_name!("select")));
        let a = dom.create_element(html(local_name!("option")));
        let b = dom.create_element(html(local_name!("option")));
        dom.append_child(same_list_select, a);
        dom.append_child(same_list_select, b);
        dom.set_option_selected(b, true, false);
        dom.move_before(same_list_select, b, Some(a));
        assert_eq!(dom.select_selected_index(same_list_select), 0);
        assert!(dom.option_selected_state(b).unwrap().selected);
        assert!(!dom.option_selected_state(a).unwrap().selected);

        let insert_select = dom.create_element(html(local_name!("select")));
        let insert_first = dom.create_element(html(local_name!("option")));
        let insert_selected = dom.create_element(html(local_name!("option")));
        dom.append_child(insert_select, insert_first);
        dom.append_child(insert_select, insert_selected);
        dom.set_option_selected(insert_selected, true, false);
        dom.insert_before(insert_select, insert_selected, Some(insert_first));
        assert_eq!(dom.select_selected_index(insert_select), 0);
        assert!(dom.option_selected_state(insert_selected).unwrap().selected);
        assert!(!dom.option_selected_state(insert_first).unwrap().selected);

        let move_select = dom.create_element(html(local_name!("select")));
        let move_first = dom.create_element(html(local_name!("option")));
        let move_selected = dom.create_element(html(local_name!("option")));
        dom.append_child(move_select, move_first);
        dom.append_child(move_select, move_selected);
        dom.set_option_selected(move_selected, true, false);
        dom.move_before(move_select, move_selected, Some(move_first));
        assert_eq!(dom.select_selected_index(move_select), 0);
        assert!(dom.option_selected_state(move_selected).unwrap().selected);
        assert!(!dom.option_selected_state(move_first).unwrap().selected);

        let append_select = dom.create_element(html(local_name!("select")));
        let append_selected = dom.create_element(html(local_name!("option")));
        let append_other = dom.create_element(html(local_name!("option")));
        dom.append_child(append_select, append_selected);
        dom.append_child(append_select, append_other);
        dom.append_child(append_select, append_selected);
        assert_eq!(dom.select_selected_index(append_select), 1);
        assert!(dom.option_selected_state(append_selected).unwrap().selected);
        assert!(!dom.option_selected_state(append_other).unwrap().selected);
    }

    #[test]
    fn select_option_list_respects_tree_order_and_subtree_barriers() {
        let mut dom = ScriptedDom::new();
        let select = dom.create_element(html(local_name!("select")));
        let wrapper = dom.create_element(html(local_name!("div")));
        let wrapped = dom.create_element(html(local_name!("option")));
        dom.append_child(wrapper, wrapped);
        dom.append_child(select, wrapper);

        let group = dom.create_element(html(local_name!("optgroup")));
        let grouped = dom.create_element(html(local_name!("option")));
        dom.append_child(group, grouped);
        let nested_group = dom.create_element(html(local_name!("optgroup")));
        let nested_group_option = dom.create_element(html(local_name!("option")));
        dom.append_child(nested_group, nested_group_option);
        dom.append_child(group, nested_group);
        dom.append_child(select, group);

        let datalist = dom.create_element(html(local_name!("datalist")));
        let datalist_option = dom.create_element(html(local_name!("option")));
        dom.append_child(datalist, datalist_option);
        dom.append_child(select, datalist);
        let hr = dom.create_element(html(local_name!("hr")));
        let hr_option = dom.create_element(html(local_name!("option")));
        dom.append_child(hr, hr_option);
        dom.append_child(select, hr);
        let option = dom.create_element(html(local_name!("option")));
        let nested_option = dom.create_element(html(local_name!("option")));
        dom.append_child(option, nested_option);
        dom.append_child(select, option);
        let nested_select = dom.create_element(html(local_name!("select")));
        let nested_select_option = dom.create_element(html(local_name!("option")));
        dom.append_child(nested_select, nested_select_option);
        dom.append_child(select, nested_select);

        assert_eq!(dom.select_options(select), vec![wrapped, grouped, option]);
    }

    #[test]
    fn oversized_valid_date_is_preserved_when_f64_step_conversion_is_unavailable() {
        let mut dom = ScriptedDom::new();
        let date = dom.create_element(html(local_name!("input")));
        dom.set_attribute(date, attr(local_name!("type")), "date");
        let huge = format!("{}-01-01", "9".repeat(400));
        dom.set_form_control_value(date, &huge).unwrap();
        let validity = dom.form_control_validity(date).unwrap();
        // Calendar range flags compare exact strings, but step mismatch still
        // uses the native Number-sized conversion and is unavailable here.
        assert!(!validity.flags.step_mismatch);
        assert_eq!(dom.form_control_value(date).as_deref(), Some(huge.as_str()));
    }

    #[test]
    fn calendar_range_ordering_survives_f64_year_precision_and_overflow() {
        let mut dom = ScriptedDom::new();
        let date = dom.create_element(html(local_name!("input")));
        dom.set_attribute(date, attr(local_name!("type")), "date");
        dom.set_attribute(date, attr(local_name!("max")), "9007199254740992-01-01");
        dom.set_form_control_value(date, "9007199254740993-01-01")
            .unwrap();
        assert!(
            dom.form_control_validity(date)
                .unwrap()
                .flags
                .range_overflow
        );

        let month = dom.create_element(html(local_name!("input")));
        dom.set_attribute(month, attr(local_name!("type")), "month");
        dom.set_attribute(month, attr(local_name!("max")), "9007199254740992-01");
        dom.set_form_control_value(month, "9007199254740993-01")
            .unwrap();
        assert!(
            dom.form_control_validity(month)
                .unwrap()
                .flags
                .range_overflow
        );

        let week = dom.create_element(html(local_name!("input")));
        dom.set_attribute(week, attr(local_name!("type")), "week");
        dom.set_attribute(week, attr(local_name!("max")), "9007199254740992-W01");
        dom.set_form_control_value(week, "9007199254740993-W01")
            .unwrap();
        assert!(
            dom.form_control_validity(week)
                .unwrap()
                .flags
                .range_overflow
        );

        let datetime = dom.create_element(html(local_name!("input")));
        dom.set_attribute(datetime, attr(local_name!("type")), "datetime-local");
        dom.set_attribute(
            datetime,
            attr(local_name!("max")),
            "9007199254740992-01-01T00:00",
        );
        dom.set_form_control_value(datetime, "9007199254740993-01-01T00:00")
            .unwrap();
        assert!(
            dom.form_control_validity(datetime)
                .unwrap()
                .flags
                .range_overflow
        );

        let huge_max = format!("{}-01-01", "9".repeat(400));
        let huge_value = format!("1{}-01-01", "0".repeat(400));
        dom.set_attribute(date, attr(local_name!("max")), &huge_max);
        dom.set_form_control_value(date, &huge_value).unwrap();
        assert!(
            dom.form_control_validity(date)
                .unwrap()
                .flags
                .range_overflow
        );
        assert_eq!(
            dom.form_control_value(date).as_deref(),
            Some(huge_value.as_str())
        );
    }
}
