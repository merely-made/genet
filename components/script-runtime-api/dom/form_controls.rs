// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! HTML live control state. Native entry points resolve genuine node owners;
//! script properties are projections, never the authority for current values.
use super::*;
use layout_dom_api::SelectionDirection;

pub(crate) struct FormControlGet;
impl<E: ScriptEngine> NativeFn<E> for FormControlGet {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let ref_value = cx.arg(0);
        let Some(node) = cx.owned_node(&ref_value)? else {
            return Ok(cx.make_null());
        };
        let key_value = cx.arg(1);
        let key = cx.value_to_string(&key_value)?;
        let value = node.with_dom(|dom| {
            if key == "value" {
                return dom.form_control_value(node.id());
            }
            if key == "defaultValue" {
                let mut value = String::new();
                for child in dom.dom_children(node.id()) {
                    if matches!(dom.kind(child), NodeKind::Text | NodeKind::CdataSection) {
                        value.push_str(dom.text(child).unwrap_or(""));
                    }
                }
                return Some(value);
            }
            if key.starts_with("selection") && !dom.form_control_selection_applies(node.id()) {
                return None;
            }
            let state = dom.form_control_state(node.id())?;
            match key.as_str() {
                "checked" => Some(state.checked.to_string()),
                "selectionStart" => state.selection_start.map(|v| v.to_string()),
                "selectionEnd" => state.selection_end.map(|v| v.to_string()),
                "selectionDirection" if state.selection_start.is_some() => Some(
                    match state.selection_direction {
                        SelectionDirection::None => "none",
                        SelectionDirection::Forward => "forward",
                        SelectionDirection::Backward => "backward",
                    }
                    .to_owned(),
                ),
                _ => None,
            }
        });
        match value {
            Some(v) => cx.make_string(&v),
            None => Ok(cx.make_null()),
        }
    }
}

pub(crate) struct FormControlSet;
impl<E: ScriptEngine> NativeFn<E> for FormControlSet {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let ref_value = cx.arg(0);
        let Some(node) = cx.owned_node(&ref_value)? else {
            return cx.make_string("TypeError");
        };
        let key_value = cx.arg(1);
        let value_value = cx.arg(2);
        let key = cx.value_to_string(&key_value)?;
        let value = cx.value_to_string(&value_value)?;
        let error = node.with_dom(|dom| {
            if dom.form_control_state(node.id()).is_none() {
                return Some("TypeError");
            }
            match key.as_str() {
                "value" => dom
                    .set_form_control_value(node.id(), &value)
                    .err()
                    .map(|_| "InvalidStateError"),
                "checked" => {
                    dom.set_form_control_checked(node.id(), value == "true");
                    None
                },
                _ => Some("TypeError"),
            }
        });
        match error {
            Some(e) => cx.make_string(e),
            None => Ok(cx.make_null()),
        }
    }
}

pub(crate) struct FormControlSelect;
impl<E: ScriptEngine> NativeFn<E> for FormControlSelect {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let ref_value = cx.arg(0);
        let Some(node) = cx.owned_node(&ref_value)? else {
            return cx.make_string("TypeError");
        };
        let start_value = cx.arg(1);
        let end_value = cx.arg(2);
        let direction_value = cx.arg(3);
        let start = cx
            .value_to_string(&start_value)?
            .parse::<u32>()
            .unwrap_or(0);
        let end = cx.value_to_string(&end_value)?.parse::<u32>().unwrap_or(0);
        let direction = cx.value_to_string(&direction_value)?;
        let error = node.with_dom(|dom| {
            if !dom.form_control_selection_applies(node.id()) {
                return Some("InvalidStateError");
            }
            let Some(mut state) = dom.form_control_state(node.id()) else {
                return Some("TypeError");
            };
            if state.selection_start.is_none() {
                return Some("InvalidStateError");
            }
            let length = dom
                .form_control_value(node.id())
                .unwrap_or_default()
                .encode_utf16()
                .count()
                .min(u32::MAX as usize) as u32;
            let end = end.min(length);
            state.selection_start = Some(start.min(length).min(end));
            state.selection_end = Some(end);
            state.selection_direction = match direction.as_str() {
                "backward" => SelectionDirection::Backward,
                "forward" => SelectionDirection::Forward,
                _ => SelectionDirection::None,
            };
            dom.set_form_control_state(node.id(), state);
            None
        });
        match error {
            Some(e) => cx.make_string(e),
            None => Ok(cx.make_null()),
        }
    }
}

pub(crate) struct CopyFormControlState;
impl<E: ScriptEngine> NativeFn<E> for CopyFormControlState {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let source_value = cx.arg(0);
        let copy_value = cx.arg(1);
        if let (Some(source), Some(copy)) =
            (cx.owned_node(&source_value)?, cx.owned_node(&copy_value)?)
        {
            let source_state = source.with_dom(|dom| dom.form_control_clone_state(source.id()));
            if let Some(source_state) = source_state {
                copy.with_dom(|dom| copy_state(dom, copy.id(), source_state));
            }
        }
        Ok(cx.undefined())
    }
}

pub(crate) fn copy_state(
    dom: &mut ScriptedDom,
    copy: NodeId,
    source: layout_dom_api::FormControlState,
) {
    if let Some(mut state) = dom.form_control_state(copy) {
        state.value = source.value;
        state.dirty_value = source.dirty_value;
        if dom
            .element_name(copy)
            .is_some_and(|name| name.local.as_ref() == "input")
        {
            state.checked = source.checked;
            state.dirty_checkedness = source.dirty_checkedness;
        }
        dom.set_form_control_state(copy, state);
    }
}

pub(crate) struct ResetFormControl;
impl<E: ScriptEngine> NativeFn<E> for ResetFormControl {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let ref_value = cx.arg(0);
        if let Some(node) = cx.owned_node(&ref_value)? {
            node.with_dom(|dom| dom.reset_form_control(node.id()));
        }
        Ok(cx.undefined())
    }
}
