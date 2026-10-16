// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Native user edits of live HTML text controls. Value state, drafts, and
//! selection remain in the DOM arena; sessions keep only transient input state.

use genet_text::{next_grapheme_boundary, previous_grapheme_boundary};
use layout_dom_api::{LayoutDom, LocalName, SelectionDirection};

use crate::{NodeId, ScriptedDom};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormEditCommand<'a> {
    Insert(&'a str),
    DeleteBackward,
    DeleteForward,
    MoveLeft,
    MoveRight,
    Home,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormEditOutcome {
    Rejected,
    Unchanged,
    SelectionChanged,
    /// A user value-write operation, even when replacement leaves the text equal.
    ValueEdited,
}

impl FormEditOutcome {
    pub fn accepted(self) -> bool {
        self != Self::Rejected
    }
}

/// Map an internal UTF-16 selection offset to the preceding UTF-8 scalar
/// boundary when it falls inside a surrogate pair.
pub fn form_edit_byte_offset_from_utf16(value: &str, offset: u32) -> usize {
    let mut units = 0u32;
    for (byte, ch) in value.char_indices() {
        let next = units + ch.len_utf16() as u32;
        if offset < next {
            return byte;
        }
        units = next;
        if offset == units {
            return byte + ch.len_utf8();
        }
    }
    value.len()
}

/// Map a UTF-8 scalar boundary in a control value to UTF-16 selection units.
pub fn form_edit_utf16_offset_from_byte(value: &str, byte_offset: usize) -> u32 {
    value
        .char_indices()
        .take_while(|(byte, _)| *byte < byte_offset)
        .map(|(_, ch)| ch.len_utf16() as u32)
        .sum()
}

fn control_attribute<'a>(dom: &'a ScriptedDom, id: NodeId, local: &str) -> Option<&'a str> {
    dom.attribute(id, &markup5ever::ns!(), &LocalName::from(local))
}

impl ScriptedDom {
    /// Apply the current native text-control eligibility policy to this live
    /// node. This gate runs for every edit, including a control focused before
    /// script changed its type, attributes, or connection.
    pub fn can_user_edit_text_control(&self, id: NodeId) -> bool {
        if !self.is_live(id) || !self.is_connected_through_shadow_hosts(id) {
            return false;
        }
        if self.html_name(id) == Some("input") {
            if matches!(
                self.input_type_for_state(id).as_str(),
                "button" | "checkbox" | "file" | "hidden" | "image" | "radio" | "reset" | "submit"
            ) {
                return false;
            }
        } else if self.html_name(id) != Some("textarea") {
            return false;
        }
        if self.form_control_is_disabled(id).unwrap_or(true)
            || control_attribute(self, id, "readonly").is_some()
        {
            return false;
        }
        for name in ["aria-disabled", "aria-readonly"] {
            if control_attribute(self, id, name).is_some_and(|value| value.trim().eq_ignore_ascii_case("true")) {
                return false;
            }
        }
        self.form_control_editing_value(id).is_some()
    }

    /// Apply one host text edit against current DOM value and selection.
    pub fn apply_form_control_user_edit(
        &mut self,
        id: NodeId,
        command: FormEditCommand<'_>,
    ) -> FormEditOutcome {
        if !self.can_user_edit_text_control(id) {
            return FormEditOutcome::Rejected;
        }
        let Some(state) = self.form_control_editing_value(id) else {
            return FormEditOutcome::Rejected;
        };
        let mut value = state.value;
        let start = form_edit_byte_offset_from_utf16(&value, state.selection_start.unwrap_or(0));
        let end = form_edit_byte_offset_from_utf16(
            &value,
            state.selection_end.unwrap_or(state.selection_start.unwrap_or(0)),
        );
        let near = start.min(end);
        let far = start.max(end);

        if matches!(command, FormEditCommand::MoveLeft | FormEditCommand::MoveRight | FormEditCommand::Home | FormEditCommand::End) {
            let caret = match command {
                FormEditCommand::MoveLeft if near != far => near,
                FormEditCommand::MoveRight if near != far => far,
                FormEditCommand::MoveLeft => previous_grapheme_boundary(&value, start).unwrap_or(start),
                FormEditCommand::MoveRight => next_grapheme_boundary(&value, start).unwrap_or(start),
                FormEditCommand::Home => 0,
                FormEditCommand::End => value.len(),
                _ => unreachable!(),
            };
            let offset = form_edit_utf16_offset_from_byte(&value, caret);
            if state.selection_start == Some(offset)
                && state.selection_end == Some(offset)
                && state.selection_direction == SelectionDirection::None
            {
                return FormEditOutcome::Unchanged;
            }
            return if self.set_form_control_editing_selection(
                id, Some(offset), Some(offset), SelectionDirection::None,
            ) {
                FormEditOutcome::SelectionChanged
            } else {
                FormEditOutcome::Rejected
            };
        }

        let (from, to, insert) = match command {
            FormEditCommand::Insert(text) => (near, far, text),
            FormEditCommand::DeleteBackward if near != far => (near, far, ""),
            FormEditCommand::DeleteForward if near != far => (near, far, ""),
            FormEditCommand::DeleteBackward => {
                (previous_grapheme_boundary(&value, start).unwrap_or(start), start, "")
            },
            FormEditCommand::DeleteForward => {
                (start, next_grapheme_boundary(&value, start).unwrap_or(start), "")
            },
            _ => unreachable!(),
        };
        if from == to && insert.is_empty() {
            return FormEditOutcome::Unchanged;
        }
        value.replace_range(from..to, insert);
        let caret = form_edit_utf16_offset_from_byte(&value, from + insert.len());
        if self.set_form_control_user_value(id, &value, Some(caret), Some(caret)) {
            FormEditOutcome::ValueEdited
        } else {
            FormEditOutcome::Rejected
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{FormEditCommand, FormEditOutcome, NodeId, ScriptedDom};
    use layout_dom_api::{LayoutDom, LayoutDomMut, SelectionDirection};
    use markup5ever::{local_name, ns, QualName};

    fn control(dom: &mut ScriptedDom, tag: markup5ever::LocalName) -> NodeId {
        let node = dom.create_element(QualName::new(None, ns!(html), tag));
        dom.append_child(dom.document(), node);
        node
    }

    fn attr(name: markup5ever::LocalName) -> QualName {
        QualName::new(None, ns!(), name)
    }

    #[test]
    fn backspace_and_delete_remove_whole_graphemes() {
        let mut dom = ScriptedDom::new();
        let input = control(&mut dom, local_name!("input"));
        assert!(dom.set_form_control_user_value(input, "a\u{301}😀z", Some(4), Some(4)));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::DeleteBackward), FormEditOutcome::ValueEdited);
        let editing = dom.form_control_editing_value(input).unwrap();
        assert_eq!(editing.value, "a\u{301}z");
        assert_eq!(editing.selection_start, Some(2));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::DeleteBackward), FormEditOutcome::ValueEdited);
        assert_eq!(dom.form_control_editing_value(input).unwrap().value, "z");
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::DeleteForward), FormEditOutcome::ValueEdited);
        assert_eq!(dom.form_control_editing_value(input).unwrap().value, "");
    }

    #[test]
    fn astral_caret_moves_in_utf16_units() {
        let mut dom = ScriptedDom::new();
        let input = control(&mut dom, local_name!("input"));
        dom.set_form_control_user_value(input, "A😀B", Some(1), Some(1));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::MoveRight), FormEditOutcome::SelectionChanged);
        assert_eq!(dom.form_control_editing_value(input).unwrap().selection_start, Some(3));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::MoveLeft), FormEditOutcome::SelectionChanged);
        assert_eq!(dom.form_control_editing_value(input).unwrap().selection_start, Some(1));
    }

    #[test]
    fn insert_at_half_surrogate_offset_snaps_to_scalar_start() {
        let mut dom = ScriptedDom::new();
        let input = control(&mut dom, local_name!("input"));
        dom.set_form_control_user_value(input, "A😀B", Some(2), Some(2));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("x")), FormEditOutcome::ValueEdited);
        let editing = dom.form_control_editing_value(input).unwrap();
        assert_eq!(editing.value, "Ax😀B");
        assert_eq!((editing.selection_start, editing.selection_end), (Some(2), Some(2)));
    }

    #[test]
    fn selection_replacement_and_arrows_collapse_to_near_endpoint() {
        let mut dom = ScriptedDom::new();
        let textarea = control(&mut dom, local_name!("textarea"));
        dom.set_form_control_user_value(textarea, "abcd", Some(1), Some(3));
        assert_eq!(dom.apply_form_control_user_edit(textarea, FormEditCommand::MoveRight), FormEditOutcome::SelectionChanged);
        let editing = dom.form_control_editing_value(textarea).unwrap();
        assert_eq!((editing.selection_start, editing.selection_end), (Some(3), Some(3)));
        dom.set_form_control_editing_selection(textarea, Some(1), Some(3), SelectionDirection::None);
        assert_eq!(dom.apply_form_control_user_edit(textarea, FormEditCommand::MoveLeft), FormEditOutcome::SelectionChanged);
        let editing = dom.form_control_editing_value(textarea).unwrap();
        assert_eq!((editing.selection_start, editing.selection_end), (Some(1), Some(1)));
        dom.set_form_control_editing_selection(textarea, Some(1), Some(3), SelectionDirection::None);
        assert_eq!(dom.apply_form_control_user_edit(textarea, FormEditCommand::Insert("X")), FormEditOutcome::ValueEdited);
        let editing = dom.form_control_editing_value(textarea).unwrap();
        assert_eq!(editing.value, "aXd");
        assert_eq!((editing.selection_start, editing.selection_end), (Some(2), Some(2)));
    }

    #[test]
    fn number_bad_input_draft_is_the_editing_value() {
        let mut dom = ScriptedDom::new();
        let input = control(&mut dom, local_name!("input"));
        dom.set_attribute(input, attr(local_name!("type")), "number");
        dom.set_form_control_user_value(input, "1e", Some(2), Some(2));
        assert_eq!(dom.form_control_value(input).as_deref(), Some(""));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("2")), FormEditOutcome::ValueEdited);
        assert_eq!(dom.form_control_editing_value(input).unwrap().value, "1e2");
        assert_eq!(dom.form_control_value(input).as_deref(), Some("1e2"));
    }

    #[test]
    fn number_draft_survives_navigation_and_valid_invalid_transitions() {
        let mut dom = ScriptedDom::new();
        let input = control(&mut dom, local_name!("input"));
        dom.set_attribute(input, attr(local_name!("type")), "number");
        dom.set_form_control_user_value(input, "1e", Some(2), Some(2));
        assert_eq!(dom.form_control_value(input).as_deref(), Some(""));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::MoveLeft), FormEditOutcome::SelectionChanged);
        assert_eq!(dom.form_control_editing_value(input).unwrap().selection_start, Some(1));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::MoveRight), FormEditOutcome::SelectionChanged);
        assert_eq!(dom.form_control_editing_value(input).unwrap().value, "1e");
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("2")), FormEditOutcome::ValueEdited);
        assert_eq!(dom.form_control_value(input).as_deref(), Some("1e2"));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::DeleteBackward), FormEditOutcome::ValueEdited);
        let editing = dom.form_control_editing_value(input).unwrap();
        assert_eq!(editing.value, "1e");
        assert_eq!(editing.selection_start, Some(2));
        assert_eq!(dom.form_control_value(input).as_deref(), Some(""));
        assert!(dom.form_control_validity(input).unwrap().flags.bad_input);
    }

    #[test]
    fn email_keeps_internal_selection_without_public_selection_idl() {
        let mut dom = ScriptedDom::new();
        let input = control(&mut dom, local_name!("input"));
        dom.set_attribute(input, attr(local_name!("type")), "email");
        dom.set_form_control_user_value(input, "ab@cd.test", Some(2), Some(2));
        assert!(!dom.form_control_selection_applies(input));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::MoveRight), FormEditOutcome::SelectionChanged);
        assert_eq!(dom.form_control_editing_value(input).unwrap().selection_start, Some(3));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("X")), FormEditOutcome::ValueEdited);
        let editing = dom.form_control_editing_value(input).unwrap();
        assert_eq!(editing.value, "ab@Xcd.test");
        assert_eq!((editing.selection_start, editing.selection_end), (Some(4), Some(4)));
        assert!(!dom.form_control_selection_applies(input));
    }

    #[test]
    fn stale_or_ineligible_controls_reject_edits() {
        let mut dom = ScriptedDom::new();
        let input = control(&mut dom, local_name!("input"));
        assert!(dom.can_user_edit_text_control(input));
        dom.set_attribute(input, attr(local_name!("readonly")), "");
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("x")), FormEditOutcome::Rejected);
        dom.remove_attribute(input, attr(local_name!("readonly")));
        dom.set_attribute(input, attr(local_name!("aria-readonly")), "true");
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("x")), FormEditOutcome::Rejected);
        dom.remove_attribute(input, attr(local_name!("aria-readonly")));
        dom.set_attribute(input, attr(local_name!("disabled")), "");
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("x")), FormEditOutcome::Rejected);
        dom.remove_attribute(input, attr(local_name!("disabled")));
        dom.set_attribute(input, attr(local_name!("aria-disabled")), "true");
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("x")), FormEditOutcome::Rejected);
        dom.remove_attribute(input, attr(local_name!("aria-disabled")));
        dom.remove_child(input);
        assert!(dom.is_live(input));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("x")), FormEditOutcome::Rejected);
        dom.collect([]);
        assert!(!dom.is_live(input));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("x")), FormEditOutcome::Rejected);
    }

    #[test]
    fn attached_shadow_control_is_editable_but_orphaned_host_is_not() {
        let mut dom = ScriptedDom::new();
        let host = control(&mut dom, local_name!("div"));
        let shadow = dom.attach_shadow(host, layout_dom_api::ShadowRootInit::default()).unwrap();
        let input = dom.create_element(QualName::new(None, ns!(html), local_name!("input")));
        dom.append_child(shadow, input);
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("x")), FormEditOutcome::ValueEdited);
        dom.remove_child(host);
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("y")), FormEditOutcome::Rejected);
    }

    #[test]
    fn adopted_control_keeps_edit_state_only_in_new_owner() {
        let mut source = ScriptedDom::new();
        let input = control(&mut source, local_name!("input"));
        source.set_form_control_user_value(input, "abc", Some(2), Some(2));
        source.remove_child(input);
        source.drain_mutations(&mut Vec::new());
        source.take_observed();

        let mut destination = ScriptedDom::new();
        assert_eq!(source.transfer_detached_subtree_to(&mut destination, input).unwrap(), vec![input]);
        assert_eq!(source.apply_form_control_user_edit(input, FormEditCommand::Insert("x")), FormEditOutcome::Rejected);
        assert_eq!(destination.apply_form_control_user_edit(input, FormEditCommand::Insert("x")), FormEditOutcome::Rejected);
        destination.append_child(destination.document(), input);
        let editing = destination.form_control_editing_value(input).unwrap();
        assert_eq!(editing.value, "abc");
        assert_eq!((editing.selection_start, editing.selection_end), (Some(2), Some(2)));
        assert_eq!(destination.apply_form_control_user_edit(input, FormEditCommand::Insert("X")), FormEditOutcome::ValueEdited);
        assert_eq!(destination.form_control_editing_value(input).unwrap().value, "abXc");
        assert_eq!(source.apply_form_control_user_edit(input, FormEditCommand::DeleteBackward), FormEditOutcome::Rejected);
    }

    #[test]
    fn non_text_input_kinds_are_ineligible() {
        let mut dom = ScriptedDom::new();
        let input = control(&mut dom, local_name!("input"));
        for kind in ["button", "checkbox", "file", "hidden", "image", "radio", "reset", "submit"] {
            dom.set_attribute(input, attr(local_name!("type")), kind);
            assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("x")), FormEditOutcome::Rejected, "{kind}");
        }
    }

    #[test]
    fn boundary_commands_are_accepted_without_a_write() {
        let mut dom = ScriptedDom::new();
        let input = control(&mut dom, local_name!("input"));
        dom.set_form_control_user_value(input, "a", Some(0), Some(0));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::DeleteBackward), FormEditOutcome::Unchanged);
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::MoveLeft), FormEditOutcome::Unchanged);
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Home), FormEditOutcome::Unchanged);
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::End), FormEditOutcome::SelectionChanged);
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::DeleteForward), FormEditOutcome::Unchanged);
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::MoveRight), FormEditOutcome::Unchanged);
        assert!(FormEditOutcome::Unchanged.accepted());
        assert!(!FormEditOutcome::Rejected.accepted());
    }

    #[test]
    fn replacing_with_the_same_text_counts_as_a_value_edit() {
        let mut dom = ScriptedDom::new();
        let input = control(&mut dom, local_name!("input"));
        dom.set_form_control_user_value(input, "abc", Some(1), Some(2));
        assert_eq!(dom.apply_form_control_user_edit(input, FormEditCommand::Insert("b")), FormEditOutcome::ValueEdited);
        assert_eq!(dom.form_control_editing_value(input).unwrap().value, "abc");
    }
}
