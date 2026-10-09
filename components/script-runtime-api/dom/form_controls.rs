// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! HTML live control state. Native entry points resolve genuine node owners;
//! script properties are projections, never the authority for current values.
use super::*;

fn built_in_validation_message(
    flags: layout_dom_api::ValidityState,
    catalog: &crate::ValidationMessageCatalog,
) -> String {
    let kind = if flags.value_missing {
        crate::ValidationMessageKind::ValueMissing
    } else if flags.type_mismatch {
        crate::ValidationMessageKind::TypeMismatch
    } else if flags.pattern_mismatch {
        crate::ValidationMessageKind::PatternMismatch
    } else if flags.too_long {
        crate::ValidationMessageKind::TooLong
    } else if flags.too_short {
        crate::ValidationMessageKind::TooShort
    } else if flags.range_underflow {
        crate::ValidationMessageKind::RangeUnderflow
    } else if flags.range_overflow {
        crate::ValidationMessageKind::RangeOverflow
    } else if flags.step_mismatch {
        crate::ValidationMessageKind::StepMismatch
    } else if flags.bad_input {
        crate::ValidationMessageKind::BadInput
    } else {
        return String::new();
    };
    catalog.message(kind).to_owned()
}
use layout_dom_api::SelectionDirection;

pub(crate) struct FormControlOwner;
impl<E: ScriptEngine> NativeFn<E> for FormControlOwner {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let ref_value = cx.arg(0);
        let Some(node) = cx.owned_node(&ref_value)? else {
            return Ok(cx.make_null());
        };
        let owner = node.with_dom(|dom| dom.form_control_form_owner(node.id()));
        match owner {
            Some(owner) => reflect_pinned::<E>(cx, owner.raw() as u64),
            None => Ok(cx.make_null()),
        }
    }
}

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
            if key == "kind" {
                return dom
                    .element_name(node.id())
                    .filter(|name| name.ns.as_ref() == XHTML_NS)
                    .map(|name| name.local.as_ref().to_owned());
            }
            if key == "selectedIndex" {
                return Some(dom.select_selected_index(node.id()).to_string());
            }
            if key == "selected" {
                return dom
                    .option_selected_state(node.id())
                    .map(|state| state.selected.to_string());
            }
            if key == "value" {
                return dom.form_control_value(node.id());
            }
            if key == "optionValue" {
                return dom.option_value(node.id());
            }
            if key == "optionText" {
                return dom.option_text(node.id());
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
        let error = node.with_dom(|dom| match key.as_str() {
            "value" => dom
                .set_form_control_value(node.id(), &value)
                .err()
                .map(|_| "InvalidStateError"),
            "checked" => {
                if dom.form_control_state(node.id()).is_none() {
                    return Some("TypeError");
                }
                dom.set_form_control_checked(node.id(), value == "true");
                None
            },
            "selectedIndex" => {
                dom.set_select_selected_index(node.id(), value.parse().unwrap_or(-1));
                None
            },
            "selected" => {
                if !dom.set_option_selected_by_script(node.id(), value == "true") {
                    return Some("TypeError");
                }
                None
            },
            _ => Some("TypeError"),
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
            let option_state = source.with_dom(|dom| dom.option_selected_state(source.id()));
            if let Some(option_state) = option_state {
                copy.with_dom(|dom| dom.set_option_selected_state(copy.id(), option_state));
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

/// One live flag, read from the same native view used by CSS selectors.
pub(crate) struct FormControlValidityGet;
impl<E: ScriptEngine> NativeFn<E> for FormControlValidityGet {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let ref_value = cx.arg(0);
        let Some(node) = cx.owned_node(&ref_value)? else {
            return Ok(cx.make_null());
        };
        let key_value = cx.arg(1);
        let key = cx.value_to_string(&key_value)?;
        if key == "validationMessage" {
            let catalog = node
                .host()
                .borrow()
                .agent
                .upgrade()
                .map(|agent| agent.borrow().validation_message_catalog.clone())
                .unwrap_or_default();
            let message = node.with_dom(|dom| {
                let validity = dom.form_control_validity(node.id())?;
                if !validity.will_validate || validity.flags.valid() {
                    return Some(String::new());
                }
                if validity.flags.custom_error {
                    return Some(
                        dom.form_control_custom_validity_message(node.id())
                            .unwrap_or_default(),
                    );
                }
                Some(built_in_validation_message(validity.flags, &catalog))
            });
            return match message {
                Some(message) => cx.make_string(&message),
                None => Ok(cx.make_null()),
            };
        }
        let value = node.with_dom(|dom| {
            let validity = dom.form_control_validity(node.id())?;
            let flags = validity.flags;
            Some(match key.as_str() {
                "willValidate" => validity.will_validate,
                "valueMissing" => flags.value_missing,
                "typeMismatch" => flags.type_mismatch,
                "patternMismatch" => flags.pattern_mismatch,
                "tooLong" => flags.too_long,
                "tooShort" => flags.too_short,
                "rangeUnderflow" => flags.range_underflow,
                "rangeOverflow" => flags.range_overflow,
                "stepMismatch" => flags.step_mismatch,
                "badInput" => flags.bad_input,
                "customError" => flags.custom_error,
                "valid" => flags.valid(),
                _ => return None,
            })
        });
        match value {
            Some(value) => cx.make_string(if value { "true" } else { "false" }),
            None => Ok(cx.make_null()),
        }
    }
}

/// Bootstrap-private sink for native `reportValidity()` feedback. It is
/// captured and deleted before authored script runs; only uncanceled trusted
/// invalid dispatches reach it.
pub(crate) struct QueueValidationReport;
impl<E: ScriptEngine> NativeFn<E> for QueueValidationReport {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let ref_value = cx.arg(0);
        let Some(node) = cx.owned_node(&ref_value)? else {
            return Ok(cx.make_null());
        };
        let message_value = cx.arg(1);
        let message = cx.value_to_string(&message_value)?;
        // Allocate ordering before borrowing the owner host's queue. The agent
        // and host are distinct RefCells, but never hold both borrows at once.
        let agent = { node.host().borrow().agent.upgrade() };
        let sequence = agent.map_or(0, |agent| {
            let mut agent = agent.borrow_mut();
            let sequence = agent.next_validation_report_sequence;
            agent.next_validation_report_sequence = sequence.saturating_add(1);
            sequence
        });
        node.with_host(|host| {
            host.pending_validation_reports
                .push_back(crate::PendingValidationReport {
                    sequence,
                    node: node.id(),
                    message,
                });
        });
        Ok(cx.make_null())
    }
}

pub(crate) struct SetCustomValidity;
impl<E: ScriptEngine> NativeFn<E> for SetCustomValidity {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let ref_value = cx.arg(0);
        let Some(node) = cx.owned_node(&ref_value)? else {
            return cx.make_string("TypeError");
        };
        let message_value = cx.arg(1);
        let message = cx.value_to_string(&message_value)?;
        if node.with_dom(|dom| dom.set_custom_validity(node.id(), &message)) {
            Ok(cx.make_null())
        } else {
            cx.make_string("TypeError")
        }
    }
}

/// Form association and option collections use native tree order and owners.
/// The count/item protocol follows the existing DOM collection sinks.
pub(crate) struct FormControlListCount;
impl<E: ScriptEngine> NativeFn<E> for FormControlListCount {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let ref_value = cx.arg(0);
        let Some(node) = cx.owned_node(&ref_value)? else {
            return Ok(cx.make_null());
        };
        let kind_value = cx.arg(1);
        let kind = cx.value_to_string(&kind_value)?;
        let count = node.with_dom(|dom| match kind.as_str() {
            "form" => dom.associated_form_controls(node.id()).len(),
            "select" => dom.select_options(node.id()).len(),
            _ => 0,
        });
        cx.make_string(&count.to_string())
    }
}

pub(crate) struct FormControlListItem;
impl<E: ScriptEngine> NativeFn<E> for FormControlListItem {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let ref_value = cx.arg(0);
        let Some(node) = cx.owned_node(&ref_value)? else {
            return Ok(cx.make_null());
        };
        let kind_value = cx.arg(1);
        let kind = cx.value_to_string(&kind_value)?;
        let index_value = cx.arg(2);
        let index = cx
            .value_to_string(&index_value)?
            .parse::<usize>()
            .unwrap_or(usize::MAX);
        let item = node.with_dom(|dom| {
            let controls = match kind.as_str() {
                "form" => dom.associated_form_controls(node.id()),
                "select" => dom.select_options(node.id()),
                _ => return None,
            };
            controls.get(index).copied()
        });
        match item {
            Some(item) => reflect_pinned::<E>(cx, item.raw() as u64),
            None => Ok(cx.make_null()),
        }
    }
}
