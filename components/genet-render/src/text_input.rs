// SPDX-License-Identifier: MPL-2.0
//! Shared metadata projection for the static and scripted native editors.

pub use document_session_api::{SessionTextInputRect, SessionTextInputState};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace};

/// The caller obtains `caret` from its current shaped layout, in viewport
/// coordinates. Editing values are read only to bound UTF-16 selection offsets;
/// they are never copied into the observation.
pub fn text_input_state(
    dom: &ScriptedDom,
    node: NodeId,
    caret: [f32; 4],
) -> Option<SessionTextInputState> {
    if !dom.is_live(node)
        || !dom.can_user_edit_text_control(node)
        || !caret.into_iter().all(f32::is_finite)
        || caret[2] <= 0.0
        || caret[3] <= 0.0
    {
        return None;
    }
    let kind = dom.element_name(node)?.local.to_string();
    let editing = dom.form_control_editing_value(node)?;
    let len = editing.value.encode_utf16().count().min(u32::MAX as usize) as u32;
    let start = editing.selection_start.unwrap_or(len).min(len);
    let end = editing.selection_end.unwrap_or(start).min(len);
    let attr = |name: &str| {
        dom.attribute(node, &Namespace::default(), &LocalName::from(name))
            .unwrap_or("")
            .to_string()
    };
    let input_type = if kind == "input" {
        dom.input_type_for_state(node)
    } else {
        String::new()
    };
    Some(SessionTextInputState {
        multiline: kind == "textarea",
        password: input_type == "password",
        element_kind: kind,
        input_type,
        input_mode: attr("inputmode"),
        autocomplete: attr("autocomplete"),
        selection_utf16: (start, end),
        caret: SessionTextInputRect {
            x: caret[0] as f64,
            y: caret[1] as f64,
            width: caret[2] as f64,
            height: caret[3] as f64,
        },
    })
}
