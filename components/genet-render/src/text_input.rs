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

/// The non-collapsed selection of a live text control, for host Copy and Cut.
/// Password inputs never answer, matching the platform's refusal to copy them.
pub fn selected_text(dom: &ScriptedDom, node: NodeId) -> Option<String> {
    if !dom.is_live(node) {
        return None;
    }
    let editing = dom.form_control_editing_value(node)?;
    if dom.element_name(node)?.local.as_str() == "input"
        && dom.input_type_for_state(node) == "password"
    {
        return None;
    }
    let (start, end) = (editing.selection_start?, editing.selection_end?);
    let start = genet_scripted_dom::form_edit_byte_offset_from_utf16(&editing.value, start);
    let end = genet_scripted_dom::form_edit_byte_offset_from_utf16(&editing.value, end);
    (start < end).then(|| editing.value[start..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::selected_text;
    use genet_scripted_dom::{NodeId, ScriptedDom};
    use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, Namespace, SelectionDirection};

    fn by_id(dom: &ScriptedDom, node: NodeId, id: &str) -> Option<NodeId> {
        if dom.attribute(node, &Namespace::default(), &LocalName::from("id")) == Some(id) {
            return Some(node);
        }
        dom.dom_children(node).find_map(|child| by_id(dom, child, id))
    }

    #[test]
    fn control_selection_reads_utf16_range_and_refuses_passwords() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        dom.set_inner_html(
            root,
            "<input id=field value='a😀b'><textarea id=note>cedar</textarea>\
             <input id=secret type=password value=hunter2><p id=para>text</p>",
        );
        let field = by_id(&dom, root, "field").unwrap();
        let note = by_id(&dom, root, "note").unwrap();
        let secret = by_id(&dom, root, "secret").unwrap();
        let para = by_id(&dom, root, "para").unwrap();
        let select = |dom: &mut ScriptedDom, node, start, end| {
            assert!(dom.set_form_control_editing_selection(
                node,
                Some(start),
                Some(end),
                SelectionDirection::Forward,
            ));
        };
        select(&mut dom, field, 1, 4);
        assert_eq!(selected_text(&dom, field).as_deref(), Some("😀b"));
        select(&mut dom, note, 1, 4);
        assert_eq!(selected_text(&dom, note).as_deref(), Some("eda"));
        select(&mut dom, note, 2, 2);
        assert_eq!(selected_text(&dom, note), None, "a collapsed range copies nothing");
        select(&mut dom, secret, 0, 7);
        assert_eq!(selected_text(&dom, secret), None, "password values never leave");
        assert_eq!(selected_text(&dom, para), None, "not a text control");
    }
}
