// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native file authority and successful-control entry snapshots. Script event
//! dispatch and reentrancy guards belong to callers, outside arena borrows.

use layout_dom_api::{LayoutDom, LocalName};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{NodeId, ScriptedDom};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FormFileId(u64);

static NEXT_FILE_ID: AtomicU64 = AtomicU64::new(1);

impl FormFileId {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// Versioned binding interchange, validated against checked native allocation.
/// This token is data, never proof of a VM File brand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormFileBindingIdentity {
    id: FormFileId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormFileBindingIdentityError {
    InvalidToken,
}
impl FormFileBindingIdentity {
    pub fn encode(self) -> String {
        format!("genet-form-file-v1:{:016x}", self.id.0)
    }
    pub fn decode(token: &str) -> Result<Self, FormFileBindingIdentityError> {
        let digits = token
            .strip_prefix("genet-form-file-v1:")
            .ok_or(FormFileBindingIdentityError::InvalidToken)?;
        if digits.len() != 16
            || !digits
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(FormFileBindingIdentityError::InvalidToken);
        }
        let value = u64::from_str_radix(digits, 16)
            .map_err(|_| FormFileBindingIdentityError::InvalidToken)?;
        if value == 0 || value >= NEXT_FILE_ID.load(Ordering::Relaxed) {
            return Err(FormFileBindingIdentityError::InvalidToken);
        }
        Ok(Self {
            id: FormFileId(value),
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormImplicitSubmission {
    Blocked,
    SubmitForm,
    SubmitButton(NodeId),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormFile {
    pub id: FormFileId,
    pub name: String,
    pub mime_type: String,
    pub bytes: Arc<[u8]>,
}

impl FormFile {
    pub fn binding_identity(&self) -> FormFileBindingIdentity {
        FormFileBindingIdentity { id: self.id }
    }
    pub fn from_binding_identity(
        identity: FormFileBindingIdentity,
        name: String,
        mime_type: String,
        bytes: Arc<[u8]>,
    ) -> Self {
        let mime_type = if mime_type.bytes().all(|b| (0x20..=0x7e).contains(&b)) {
            mime_type.to_ascii_lowercase()
        } else {
            String::new()
        };
        Self {
            id: identity.id,
            name,
            mime_type,
            bytes,
        }
    }

    pub fn new(name: String, mime_type: String, bytes: Arc<[u8]>) -> Result<Self, FormEntryError> {
        let id = NEXT_FILE_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .map_err(|_| FormEntryError::IdentityExhausted)?;
        // Blob/File type metadata is printable ASCII, lowercased. Keeping this
        // invariant also prevents native callers from introducing MIME headers.
        let mime_type = if mime_type.bytes().all(|b| (0x20..=0x7e).contains(&b)) {
            mime_type.to_ascii_lowercase()
        } else {
            String::new()
        };
        Ok(Self {
            id: FormFileId(id),
            name,
            mime_type,
            bytes,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormEntryValue {
    Text(String),
    File {
        file: FormFile,
        source_input: Option<NodeId>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormEntry {
    pub name: String,
    pub value: FormEntryValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormEntryError {
    InvalidForm,
    InvalidSubmitter,
    IdentityExhausted,
}

impl ScriptedDom {
    pub fn form_control_is_disabled(&self, node: NodeId) -> Option<bool> {
        (self.is_live(node)
            && matches!(
                self.html_name(node),
                Some(
                    "input" | "button" | "textarea" | "select" | "fieldset" | "option" | "optgroup"
                )
            ))
        .then(|| self.is_disabled(node))
    }
    pub fn implicit_form_submission(
        &self,
        form: NodeId,
    ) -> Result<FormImplicitSubmission, FormEntryError> {
        if !self.is_live(form) || self.html_name(form) != Some("form") {
            return Err(FormEntryError::InvalidForm);
        }
        let fields = self.associated_form_controls(form);
        if let Some(button) = fields
            .iter()
            .copied()
            .find(|id| self.is_form_submit_button(*id))
        {
            return Ok(if self.is_disabled(button) {
                FormImplicitSubmission::Blocked
            } else {
                FormImplicitSubmission::SubmitButton(button)
            });
        }
        let mut blockers = 0;
        for field in fields {
            if self.html_name(field) == Some("input")
                && matches!(
                    self.input_type_for_state(field).as_str(),
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
            {
                blockers += 1;
                if blockers > 1 {
                    return Ok(FormImplicitSubmission::Blocked);
                }
            }
        }
        Ok(FormImplicitSubmission::SubmitForm)
    }

    /// None identifies a non-file input (including retired or foreign nodes).
    pub fn selected_files(&self, input: NodeId) -> Option<Vec<FormFile>> {
        (self.is_live(input)
            && self.html_name(input) == Some("input")
            && self.input_type_for_state(input) == "file")
            .then(|| self.node(input).selected_files.clone())
    }

    /// Replace atomically after the bridge has validated private File brands.
    /// Records retain their native ids; no VM values enter the arena.
    pub fn set_selected_files(&mut self, input: NodeId, files: Vec<FormFile>) -> bool {
        if !self.is_live(input)
            || self.html_name(input) != Some("input")
            || self.input_type_for_state(input) != "file"
        {
            return false;
        }
        self.node_mut(input).selected_files = files;
        self.clear_user_edit_state(input);
        self.mutations
            .push(crate::DomMutation::FormControlStateChanged { node: input });
        true
    }

    /// Snapshot successful native controls without firing authored callbacks.
    /// The caller holds the form's construction flag through formdata dispatch.
    pub fn form_submission_entries(
        &self,
        form: NodeId,
        submitter: Option<NodeId>,
        encoding_name: &str,
        image_position: (u32, u32),
    ) -> Result<Vec<FormEntry>, FormEntryError> {
        if !self.is_live(form) || self.html_name(form) != Some("form") {
            return Err(FormEntryError::InvalidForm);
        }
        if let Some(button) = submitter
            && (!self.is_live(button)
                || !self.is_form_submit_button(button)
                || self.form_control_form_owner(button) != Some(form))
        {
            return Err(FormEntryError::InvalidSubmitter);
        }
        let mut entries = Vec::new();
        for field in self.associated_form_controls(form) {
            let Some(element) = self.html_name(field) else {
                continue;
            };
            if !matches!(element, "input" | "textarea" | "select" | "button")
                || self.is_disabled(field)
                || self.has_datalist_ancestor(field)
            {
                continue;
            }
            let kind = if element == "input" {
                self.input_type_for_state(field)
            } else {
                String::new()
            };
            let button = element == "button"
                || element == "input"
                    && matches!(kind.as_str(), "submit" | "reset" | "button" | "image");
            if button && submitter != Some(field) {
                continue;
            }
            if matches!(kind.as_str(), "checkbox" | "radio")
                && !self
                    .node(field)
                    .form_control
                    .as_ref()
                    .is_some_and(|state| state.checked)
            {
                continue;
            }
            let name = self.form_submission_attribute(field, "name").unwrap_or("");
            if kind == "image" {
                let prefix = if name.is_empty() {
                    String::new()
                } else {
                    format!("{name}.")
                };
                entries.push(text_entry(
                    format!("{prefix}x"),
                    image_position.0.to_string(),
                ));
                entries.push(text_entry(
                    format!("{prefix}y"),
                    image_position.1.to_string(),
                ));
                continue;
            }
            if name.is_empty() {
                continue;
            }
            match element {
                "select" => {
                    for option in self.select_options(field) {
                        if self
                            .node(option)
                            .option_state
                            .is_some_and(|state| state.selected)
                            && !self.option_is_disabled(option)
                        {
                            entries.push(text_entry(
                                name.to_owned(),
                                self.option_value(option).unwrap_or_default(),
                            ));
                        }
                    }
                },
                "input" if kind == "file" => {
                    let files = &self.node(field).selected_files;
                    if files.is_empty() {
                        let file = FormFile::new(
                            String::new(),
                            "application/octet-stream".into(),
                            Arc::from([]),
                        )?;
                        entries.push(FormEntry {
                            name: name.into(),
                            value: FormEntryValue::File {
                                file,
                                source_input: None,
                            },
                        });
                    } else {
                        entries.extend(files.iter().cloned().map(|file| FormEntry {
                            name: name.into(),
                            value: FormEntryValue::File {
                                file,
                                source_input: Some(field),
                            },
                        }));
                    }
                },
                "input" if kind == "hidden" && name.eq_ignore_ascii_case("_charset_") => {
                    entries.push(text_entry(name.into(), encoding_name.into()));
                },
                "button" => {
                    entries.push(text_entry(
                        name.into(),
                        self.form_submission_attribute(field, "value")
                            .unwrap_or("")
                            .into(),
                    ));
                },
                _ => entries.push(text_entry(
                    name.into(),
                    self.form_control_value(field).unwrap_or_default(),
                )),
            }
            if self.is_auto_directionality_control(field)
                && let Some(dirname) = self
                    .form_submission_attribute(field, "dirname")
                    .filter(|name| !name.is_empty())
            {
                entries.push(text_entry(
                    dirname.into(),
                    self.element_directionality(field).as_str().into(),
                ));
            }
        }
        Ok(entries)
    }

    fn form_submission_attribute(&self, field: NodeId, name: &str) -> Option<&str> {
        self.attribute(field, &markup5ever::ns!(), &LocalName::from(name))
    }

    /// Shared HTML submit-button classification for entry construction,
    /// validation and script activation/requestSubmit. Auto button state is
    /// context-sensitive; explicit type=submit takes precedence.
    pub fn is_form_submit_button(&self, field: NodeId) -> bool {
        if !self.is_live(field) {
            return false;
        }
        match self.html_name(field) {
            Some("input") => matches!(
                self.input_type_for_state(field).as_str(),
                "submit" | "image"
            ),
            Some("button") => {
                let kind = self.form_submission_attribute(field, "type").unwrap_or("");
                if kind.eq_ignore_ascii_case("submit") {
                    return true;
                }
                if kind.eq_ignore_ascii_case("button") || kind.eq_ignore_ascii_case("reset") {
                    return false;
                }
                // Missing and invalid type values are Auto. Attribute presence,
                // even an empty value, prevents Auto from being a submit button.
                self.form_submission_attribute(field, "command").is_none()
                    && self
                        .form_submission_attribute(field, "commandfor")
                        .is_none()
                    && !self
                        .parent(field)
                        .is_some_and(|parent| self.html_name(parent) == Some("select"))
            },
            _ => false,
        }
    }
}

fn text_entry(name: String, value: String) -> FormEntry {
    FormEntry {
        name,
        value: FormEntryValue::Text(value),
    }
}

#[cfg(test)]
pub(crate) fn fixture_file(id: u64, name: &str, mime: &str, bytes: &[u8]) -> FormFile {
    FormFile {
        id: FormFileId(id),
        name: name.into(),
        mime_type: mime.into(),
        bytes: Arc::from(bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout_dom_api::{LayoutDom, LayoutDomMut, LocalName, QualName};
    use markup5ever::ns;

    fn element(dom: &mut ScriptedDom, name: &str) -> NodeId {
        dom.create_element(QualName::new(None, ns!(html), LocalName::from(name)))
    }
    fn attr(dom: &mut ScriptedDom, id: NodeId, name: &str, value: &str) {
        dom.set_attribute(id, QualName::new(None, ns!(), LocalName::from(name)), value);
    }
    fn input(dom: &mut ScriptedDom, parent: NodeId, name: &str, kind: &str, value: &str) -> NodeId {
        let id = element(dom, "input");
        attr(dom, id, "name", name);
        attr(dom, id, "type", kind);
        attr(dom, id, "value", value);
        dom.append_child(parent, id);
        id
    }
    fn form(dom: &mut ScriptedDom) -> NodeId {
        let id = element(dom, "form");
        attr(dom, id, "id", "f");
        let document = dom.document();
        dom.append_child(document, id);
        id
    }
    fn entries(
        dom: &ScriptedDom,
        form: NodeId,
        submitter: Option<NodeId>,
        encoding: &str,
        position: (u32, u32),
    ) -> Vec<(String, String)> {
        dom.form_submission_entries(form, submitter, encoding, position)
            .unwrap()
            .into_iter()
            .map(|entry| {
                let FormEntryValue::Text(value) = entry.value else {
                    panic!("text fixture unexpectedly produced File")
                };
                (entry.name, value)
            })
            .collect()
    }
    fn pairs(values: &[(&str, &str)]) -> Vec<(String, String)> {
        values
            .iter()
            .map(|(n, v)| ((*n).into(), (*v).into()))
            .collect()
    }

    #[test]
    fn phase_c_native_external_association_order_current_values_and_names() {
        let mut dom = ScriptedDom::new();
        let document = dom.document();
        let before = input(&mut dom, document, "a", "text", "before");
        attr(&mut dom, before, "form", "f");
        let f = form(&mut dom);
        let current = input(&mut dom, f, "a", "text", "default");
        dom.set_form_control_value(current, "current").unwrap();
        let after = input(&mut dom, document, "a", "text", "after");
        attr(&mut dom, after, "form", "f");
        let foreign = input(&mut dom, f, "foreign", "text", "no");
        attr(&mut dom, foreign, "form", "other");
        let id_only = element(&mut dom, "input");
        attr(&mut dom, id_only, "id", "not-a-name");
        dom.append_child(f, id_only);
        assert_eq!(
            entries(&dom, f, None, "UTF-8", (0, 0)),
            pairs(&[("a", "before"), ("a", "current"), ("a", "after")])
        );
    }

    #[test]
    fn phase_c_native_duplicate_id_first_match_and_detached_owner() {
        let mut dom = ScriptedDom::new();
        let document = dom.document();
        let blocker = element(&mut dom, "div");
        attr(&mut dom, blocker, "id", "f");
        dom.append_child(document, blocker);
        let f = form(&mut dom);
        let external = input(&mut dom, document, "external", "text", "no");
        attr(&mut dom, external, "form", "f");
        let foreign = input(&mut dom, f, "foreign", "text", "no");
        attr(&mut dom, foreign, "form", "f");
        input(&mut dom, f, "own", "text", "yes");
        assert_eq!(
            entries(&dom, f, None, "UTF-8", (0, 0)),
            pairs(&[("own", "yes")])
        );
        let detached = element(&mut dom, "form");
        input(&mut dom, detached, "detached", "text", "current");
        assert_eq!(
            entries(&dom, detached, None, "UTF-8", (0, 0)),
            pairs(&[("detached", "current")])
        );
    }

    #[test]
    fn phase_c_native_disabled_first_legend_and_datalist() {
        let mut dom = ScriptedDom::new();
        let f = form(&mut dom);
        let fs = element(&mut dom, "fieldset");
        attr(&mut dom, fs, "disabled", "");
        dom.append_child(f, fs);
        let first = element(&mut dom, "legend");
        dom.append_child(fs, first);
        input(&mut dom, first, "keep", "text", "yes");
        let second = element(&mut dom, "legend");
        dom.append_child(fs, second);
        input(&mut dom, second, "second", "text", "no");
        input(&mut dom, fs, "disabled", "text", "no");
        let dl = element(&mut dom, "datalist");
        dom.append_child(f, dl);
        input(&mut dom, dl, "datalist", "text", "no");
        assert_eq!(
            entries(&dom, f, None, "UTF-8", (0, 0)),
            pairs(&[("keep", "yes")])
        );
    }

    #[test]
    fn phase_c_native_checked_controls_and_submitter() {
        let mut dom = ScriptedDom::new();
        let f = form(&mut dom);
        input(&mut dom, f, "unchecked", "checkbox", "no");
        input(&mut dom, f, "radio", "radio", "no");
        let checked = input(&mut dom, f, "checked", "checkbox", "");
        dom.remove_attribute(
            checked,
            QualName::new(None, ns!(), LocalName::from("value")),
        );
        dom.set_form_control_checked(checked, true);
        let submit = input(&mut dom, f, "submit", "submit", "go");
        input(&mut dom, f, "other", "submit", "no");
        input(&mut dom, f, "reset", "reset", "no");
        input(&mut dom, f, "button", "button", "no");
        assert_eq!(
            entries(&dom, f, Some(submit), "UTF-8", (0, 0)),
            pairs(&[("checked", "on"), ("submit", "go")])
        );
    }

    #[test]
    fn phase_c_native_image_coordinates_named_and_unnamed() {
        let mut dom = ScriptedDom::new();
        let f = form(&mut dom);
        let image = input(&mut dom, f, "pic", "image", "ignored");
        assert_eq!(
            entries(&dom, f, Some(image), "UTF-8", (7, 9)),
            pairs(&[("pic.x", "7"), ("pic.y", "9")])
        );
        attr(&mut dom, image, "name", "");
        assert_eq!(
            entries(&dom, f, Some(image), "UTF-8", (7, 9)),
            pairs(&[("x", "7"), ("y", "9")])
        );
    }

    #[test]
    fn phase_c_native_multiple_select_excludes_selected_disabled_options() {
        let mut dom = ScriptedDom::new();
        let f = form(&mut dom);
        let s = element(&mut dom, "select");
        attr(&mut dom, s, "name", "s");
        attr(&mut dom, s, "multiple", "");
        dom.append_child(f, s);
        for (value, disabled) in [("one", false), ("two", true), ("three", false)] {
            let o = element(&mut dom, "option");
            attr(&mut dom, o, "value", value);
            attr(&mut dom, o, "selected", "");
            if disabled {
                attr(&mut dom, o, "disabled", "");
            }
            dom.append_child(s, o);
        }
        let group = element(&mut dom, "optgroup");
        attr(&mut dom, group, "disabled", "");
        dom.append_child(s, group);
        let o = element(&mut dom, "option");
        attr(&mut dom, o, "value", "four");
        attr(&mut dom, o, "selected", "");
        dom.append_child(group, o);
        assert_eq!(
            entries(&dom, f, None, "UTF-8", (0, 0)),
            pairs(&[("s", "one"), ("s", "three")])
        );
    }

    #[test]
    fn phase_c_native_charset_is_hidden_case_insensitive_and_supplied() {
        let mut dom = ScriptedDom::new();
        let f = form(&mut dom);
        input(&mut dom, f, "_ChArSeT_", "hidden", "default");
        input(&mut dom, f, "_charset_", "text", "current");
        assert_eq!(
            entries(&dom, f, None, "windows-1252", (0, 0)),
            pairs(&[("_ChArSeT_", "windows-1252"), ("_charset_", "current")])
        );
    }

    #[test]
    fn phase_c_native_dirname_reads_current_auto_and_hidden_values() {
        let mut dom = ScriptedDom::new();
        let f = form(&mut dom);
        let i = input(&mut dom, f, "q", "text", "Latin");
        attr(&mut dom, i, "dir", "auto");
        attr(&mut dom, i, "dirname", "d");
        dom.set_form_control_value(i, "العربية").unwrap();
        let hidden = input(&mut dom, f, "h", "hidden", "ع");
        attr(&mut dom, hidden, "dir", "auto");
        attr(&mut dom, hidden, "dirname", "hd");
        assert_eq!(
            entries(&dom, f, None, "UTF-8", (0, 0)),
            pairs(&[("q", "العربية"), ("d", "rtl"), ("h", "ع"), ("hd", "rtl")])
        );
    }

    #[test]
    fn phase_c_native_empty_file_is_fresh_binary_record() {
        let mut dom = ScriptedDom::new();
        let f = form(&mut dom);
        input(&mut dom, f, "upload", "file", "");
        let a = dom
            .form_submission_entries(f, None, "UTF-8", (0, 0))
            .expect("native entry capability is required");
        let b = dom
            .form_submission_entries(f, None, "UTF-8", (0, 0))
            .unwrap();
        assert_eq!(a.len(), 1);
        assert_eq!(a[0].name, "upload");
        let FormEntryValue::File { file, source_input } = &a[0].value else {
            panic!("empty file must not be text")
        };
        assert_eq!(file.name, "");
        assert_eq!(file.mime_type, "application/octet-stream");
        assert_eq!(&*file.bytes, &[]);
        assert_eq!(*source_input, None);
        let FormEntryValue::File { file: second, .. } = &b[0].value else {
            panic!("empty file must not be text")
        };
        assert_ne!(file.id, second.id);
    }

    #[test]
    fn phase_c_native_selected_files_reset_clone_adoption_and_snapshot() {
        let mut dom = ScriptedDom::new();
        let f = form(&mut dom);
        let i = input(&mut dom, f, "upload", "file", "");
        attr(&mut dom, i, "multiple", "");
        let a = fixture_file(1, "same.bin", "application/octet-stream", &[0, 128, 255]);
        let b = fixture_file(2, "same.bin", "application/octet-stream", &[9]);
        assert!(
            dom.set_selected_files(i, vec![a.clone(), b.clone()]),
            "native selected-file replacement capability is required"
        );
        assert_eq!(dom.selected_files(i), Some(vec![a.clone(), b.clone()]));
        let snapshot = dom
            .form_submission_entries(f, None, "UTF-8", (0, 0))
            .unwrap();
        assert_eq!(
            snapshot,
            vec![
                FormEntry {
                    name: "upload".into(),
                    value: FormEntryValue::File {
                        file: a.clone(),
                        source_input: Some(i)
                    }
                },
                FormEntry {
                    name: "upload".into(),
                    value: FormEntryValue::File {
                        file: b.clone(),
                        source_input: Some(i)
                    }
                }
            ]
        );
        let cloned = element(&mut dom, "input");
        attr(&mut dom, cloned, "type", "file");
        assert!(dom.clone_form_control_state(i, cloned));
        assert_eq!(dom.selected_files(cloned), Some(vec![]));
        let detached = input(&mut dom, f, "move", "file", "");
        assert!(dom.set_selected_files(detached, vec![a.clone()]));
        dom.remove_child(detached);
        dom.drain_mutations(&mut Vec::new());
        dom.take_observed();
        let mut destination = ScriptedDom::new();
        dom.transfer_detached_subtree_to(&mut destination, detached)
            .unwrap();
        assert_eq!(destination.selected_files(detached), Some(vec![a.clone()]));
        assert!(dom.reset_form_control(i));
        assert_eq!(dom.selected_files(i), Some(vec![]));
        let FormEntryValue::File { file, .. } = &snapshot[0].value else {
            panic!("snapshot lost file")
        };
        assert_eq!(&*file.bytes, &[0, 128, 255]);
    }

    #[test]
    fn phase_c_native_file_value_clear_and_nonempty_rejection_are_atomic() {
        let mut dom = ScriptedDom::new();
        let f = form(&mut dom);
        let i = input(&mut dom, f, "upload", "file", "");
        let a = fixture_file(1, "a.bin", "application/octet-stream", &[0, 128, 255]);
        assert!(
            dom.set_selected_files(i, vec![a.clone()]),
            "native selected files unavailable"
        );
        assert_eq!(
            dom.form_control_value(i).as_deref(),
            Some("C:\\fakepath\\a.bin")
        );
        assert_eq!(
            dom.set_form_control_value(i, "forbidden"),
            Err(crate::FormControlValueError::InvalidState)
        );
        assert_eq!(dom.selected_files(i), Some(vec![a]));
        dom.set_form_control_value(i, "").unwrap();
        assert_eq!(dom.selected_files(i), Some(vec![]));
    }

    #[test]
    fn phase_c_native_file_identity_allocation_and_required_state() {
        let a = FormFile::new(
            "a.bin".into(),
            "application/octet-stream".into(),
            Arc::from([0u8, 128, 255]),
        )
        .expect("native file identity allocation capability is required");
        let b = FormFile::new(
            "a.bin".into(),
            "application/octet-stream".into(),
            Arc::from([0u8, 128, 255]),
        )
        .unwrap();
        assert_ne!(a.id, b.id);
        assert_ne!(a.id.as_u64(), 0);
        let mut dom = ScriptedDom::new();
        let f = form(&mut dom);
        let i = input(&mut dom, f, "upload", "file", "");
        attr(&mut dom, i, "required", "");
        assert!(dom.form_control_validity(i).unwrap().flags.value_missing);
        assert!(dom.set_selected_files(i, vec![a]));
        assert!(!dom.form_control_validity(i).unwrap().flags.value_missing);
        attr(&mut dom, i, "type", "text");
        attr(&mut dom, i, "type", "file");
        assert_eq!(dom.selected_files(i), Some(vec![]));
    }

    // Independent regression fixtures for current HTML button Auto semantics.
    // They exercise the actual entry API, not a proposed classifier scaffold.
    fn auto_button_fixture(
        kind: Option<&str>,
        command_attribute: Option<&str>,
        select_parent: bool,
    ) -> (ScriptedDom, NodeId, NodeId) {
        let mut dom = ScriptedDom::new();
        let f = form(&mut dom);
        let parent = if select_parent {
            let select = element(&mut dom, "select");
            dom.append_child(f, select);
            select
        } else {
            f
        };
        let button = element(&mut dom, "button");
        attr(&mut dom, button, "name", "action");
        attr(&mut dom, button, "value", "open");
        if let Some(kind) = kind {
            attr(&mut dom, button, "type", kind);
        }
        if let Some(name) = command_attribute {
            attr(&mut dom, button, name, "");
        }
        dom.append_child(parent, button);
        assert_eq!(dom.form_control_form_owner(button), Some(f));
        (dom, f, button)
    }

    #[test]
    fn phase_c_native_button_auto_missing_type_command_is_invalid_submitter() {
        let (dom, f, button) = auto_button_fixture(None, Some("command"), false);
        assert_eq!(
            dom.form_submission_entries(f, Some(button), "UTF-8", (0, 0)),
            Err(FormEntryError::InvalidSubmitter)
        );
    }

    #[test]
    fn phase_c_native_button_auto_invalid_type_command_is_invalid_submitter() {
        for kind in ["bogus", " submit "] {
            let (dom, f, button) = auto_button_fixture(Some(kind), Some("command"), false);
            assert_eq!(
                dom.form_submission_entries(f, Some(button), "UTF-8", (0, 0)),
                Err(FormEntryError::InvalidSubmitter)
            );
        }
    }

    #[test]
    fn phase_c_native_button_auto_missing_type_commandfor_is_invalid_submitter() {
        let (dom, f, button) = auto_button_fixture(None, Some("commandfor"), false);
        assert_eq!(
            dom.form_submission_entries(f, Some(button), "UTF-8", (0, 0)),
            Err(FormEntryError::InvalidSubmitter)
        );
    }

    #[test]
    fn phase_c_native_button_auto_invalid_type_commandfor_is_invalid_submitter() {
        for kind in ["bogus", " submit "] {
            let (dom, f, button) = auto_button_fixture(Some(kind), Some("commandfor"), false);
            assert_eq!(
                dom.form_submission_entries(f, Some(button), "UTF-8", (0, 0)),
                Err(FormEntryError::InvalidSubmitter)
            );
        }
    }

    #[test]
    fn phase_c_native_button_auto_select_parent_is_invalid_submitter() {
        for kind in [None, Some(""), Some("bogus")] {
            let (dom, f, button) = auto_button_fixture(kind, None, true);
            assert_eq!(
                dom.form_submission_entries(f, Some(button), "UTF-8", (0, 0)),
                Err(FormEntryError::InvalidSubmitter)
            );
        }
    }

    #[test]
    fn phase_c_native_button_auto_ordinary_default_and_invalid_types_submit_control() {
        for kind in [None, Some(""), Some("bogus"), Some(" submit ")] {
            let (dom, f, button) = auto_button_fixture(kind, None, false);
            assert_eq!(
                dom.form_submission_entries(f, Some(button), "UTF-8", (0, 0)),
                Ok(vec![FormEntry {
                    name: "action".into(),
                    value: FormEntryValue::Text("open".into())
                }])
            );
        }
    }

    #[test]
    fn phase_c_native_button_auto_explicit_submit_commands_and_select_control() {
        for select_parent in [false, true] {
            for attribute in [Some("command"), Some("commandfor"), None] {
                let (mut dom, f, button) =
                    auto_button_fixture(Some("SuBmIt"), attribute, select_parent);
                if attribute.is_none() {
                    attr(&mut dom, button, "command", "");
                    attr(&mut dom, button, "commandfor", "p");
                }
                assert_eq!(
                    dom.form_submission_entries(f, Some(button), "UTF-8", (0, 0)),
                    Ok(vec![FormEntry {
                        name: "action".into(),
                        value: FormEntryValue::Text("open".into())
                    }])
                );
            }
        }
    }

    #[test]
    fn phase_c_native_button_auto_shared_predicate_guards_foreign_retired_nonbuttons() {
        let mut dom = ScriptedDom::new();
        let div = element(&mut dom, "div");
        assert!(!dom.is_form_submit_button(div));
        let live = element(&mut dom, "button");
        assert!(dom.is_form_submit_button(live));
        let retired = element(&mut dom, "button");
        LayoutDomMut::remove(&mut dom, retired);
        assert!(!dom.is_live(retired));
        assert!(!dom.is_form_submit_button(retired));
        let mut foreign = ScriptedDom::new();
        let other = element(&mut foreign, "button");
        assert!(foreign.is_form_submit_button(other));
        assert!(!dom.is_live(other));
        assert!(!dom.is_form_submit_button(other));
    }
}

#[cfg(test)]
mod phase_c_task4_fixtures {
    use super::*;
    use layout_dom_api::{LayoutDomMut, QualName};
    use markup5ever::ns;
    #[derive(Debug, PartialEq, Eq)]
    enum Implicit {
        Blocked,
        SubmitForm,
        SubmitButton(NodeId),
    }
    // RED-only missing-capability adapter. No second implicit algorithm.
    fn implicit_adapter(dom: &ScriptedDom, form: NodeId) -> Result<Implicit, FormEntryError> {
        dom.implicit_form_submission(form).map(|decision| match decision {
            FormImplicitSubmission::Blocked => Implicit::Blocked,
            FormImplicitSubmission::SubmitForm => Implicit::SubmitForm,
            FormImplicitSubmission::SubmitButton(id) => Implicit::SubmitButton(id),
        })
    }
    // RED-only missing projection: absence is None, never evidence of disabled.
    fn disabled_adapter(dom: &ScriptedDom, node: NodeId) -> Option<bool> {
        dom.form_control_is_disabled(node)
    }
    // RED identity wire exposes current as_u64 projection, not an assumed codec.
    fn identity_encode_adapter(file: &FormFile) -> String {
        file.binding_identity().encode()
    }
    // Missing codec is distinct from authoritative InvalidToken rejection.
    fn identity_decode_adapter(token: &str) -> Result<FormFileId, &'static str> {
        FormFileBindingIdentity::decode(token).map(|identity| identity.id).map_err(|_| "InvalidToken")
    }
    fn element(dom: &mut ScriptedDom, name: &str) -> NodeId {
        dom.create_element(QualName::new(None, ns!(html), LocalName::from(name)))
    }
    fn attr(dom: &mut ScriptedDom, node: NodeId, name: &str, value: &str) {
        dom.set_attribute(
            node,
            QualName::new(None, ns!(), LocalName::from(name)),
            value,
        );
    }
    fn fixture() -> (ScriptedDom, NodeId) {
        let mut dom = ScriptedDom::new();
        let f = element(&mut dom, "form");
        attr(&mut dom, f, "id", "f");
        let doc = dom.document();
        dom.append_child(doc, f);
        (dom, f)
    }
    fn input(dom: &mut ScriptedDom, parent: NodeId, kind: &str) -> NodeId {
        let i = element(dom, "input");
        attr(dom, i, "type", kind);
        dom.append_child(parent, i);
        i
    }
    #[test]
    fn phase_c_implicit_no_button_zero_or_one_blocker() {
        let (mut dom, f) = fixture();
        assert_eq!(
            implicit_adapter(&dom, f),
            Ok(Implicit::SubmitForm),
            "missing capability: shared implicit decision"
        );
        input(&mut dom, f, "text");
        assert_eq!(implicit_adapter(&dom, f), Ok(Implicit::SubmitForm));
    }
    #[test]
    fn phase_c_implicit_two_blockers() {
        let (mut dom, f) = fixture();
        input(&mut dom, f, "text");
        input(&mut dom, f, "search");
        assert_eq!(implicit_adapter(&dom, f), Ok(Implicit::Blocked));
    }
    #[test]
    fn phase_c_implicit_invalid_type_normalizes_text() {
        let (mut dom, f) = fixture();
        input(&mut dom, f, "bogus");
        input(&mut dom, f, "text");
        assert_eq!(implicit_adapter(&dom, f), Ok(Implicit::Blocked));
    }
    #[test]
    fn phase_c_implicit_literal_blocker_type_matrix() {
        for kind in [
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
        ] {
            let (mut dom, f) = fixture();
            input(&mut dom, f, "text");
            input(&mut dom, f, kind);
            assert_eq!(
                implicit_adapter(&dom, f),
                Ok(Implicit::Blocked),
                "literal blocker {kind}"
            );
        }
        for kind in [
            "hidden", "checkbox", "radio", "range", "color", "file", "button", "reset",
        ] {
            let (mut dom, f) = fixture();
            input(&mut dom, f, "text");
            input(&mut dom, f, kind);
            assert_eq!(
                implicit_adapter(&dom, f),
                Ok(Implicit::SubmitForm),
                "literal nonblocker {kind}"
            );
        }
    }
    #[test]
    fn phase_c_implicit_image_default_native_predicate() {
        let (mut dom, f) = fixture();
        let image = input(&mut dom, f, "image");
        let later = element(&mut dom, "button");
        dom.append_child(f, later);
        assert!(dom.is_form_submit_button(image));
        assert_eq!(implicit_adapter(&dom, f), Ok(Implicit::SubmitButton(image)));
    }
    #[test]
    fn phase_c_implicit_first_disabled_default_blocks_later_enabled() {
        let (mut dom, f) = fixture();
        let first = element(&mut dom, "button");
        attr(&mut dom, first, "disabled", "");
        dom.append_child(f, first);
        let second = element(&mut dom, "button");
        dom.append_child(f, second);
        assert!(dom.is_form_submit_button(first) && dom.is_form_submit_button(second));
        assert_eq!(implicit_adapter(&dom, f), Ok(Implicit::Blocked));
    }
    #[test]
    fn phase_c_implicit_external_association_tree_order() {
        let (mut dom, f) = fixture();
        let before = element(&mut dom, "button");
        attr(&mut dom, before, "form", "f");
        let doc = dom.document();
        dom.insert_before(doc, before, Some(f));
        let inside = element(&mut dom, "button");
        dom.append_child(f, inside);
        assert_eq!(dom.form_control_form_owner(before), Some(f));
        assert_eq!(
            implicit_adapter(&dom, f),
            Ok(Implicit::SubmitButton(before))
        );
    }
    #[test]
    fn phase_c_implicit_auto_command_ignored_explicit_submit_selected() {
        let (mut dom, f) = fixture();
        let auto = element(&mut dom, "button");
        attr(&mut dom, auto, "command", "");
        dom.append_child(f, auto);
        let submit = element(&mut dom, "button");
        attr(&mut dom, submit, "type", "SuBmIt");
        attr(&mut dom, submit, "commandfor", "");
        dom.append_child(f, submit);
        assert!(!dom.is_form_submit_button(auto) && dom.is_form_submit_button(submit));
        assert_eq!(
            implicit_adapter(&dom, f),
            Ok(Implicit::SubmitButton(submit))
        );
    }
    #[test]
    fn phase_c_implicit_guards_invalid_dead_foreign_form() {
        let (mut dom, f) = fixture();
        let div = element(&mut dom, "div");
        let (other, foreign) = fixture();
        assert_eq!(
            implicit_adapter(&dom, div),
            Err(FormEntryError::InvalidForm)
        );
        assert!(other.is_live(foreign));
        assert_eq!(
            implicit_adapter(&dom, foreign),
            Err(FormEntryError::InvalidForm)
        );
        LayoutDomMut::remove(&mut dom, f);
        assert!(!dom.is_live(f));
        assert_eq!(implicit_adapter(&dom, f), Err(FormEntryError::InvalidForm));
    }
    #[test]
    fn phase_c_disabled_projection_native_fieldset_legend_and_guard() {
        let (mut dom, f) = fixture();
        let fieldset = element(&mut dom, "fieldset");
        attr(&mut dom, fieldset, "disabled", "");
        dom.append_child(f, fieldset);
        let legend = element(&mut dom, "legend");
        dom.append_child(fieldset, legend);
        let exempt = input(&mut dom, legend, "text");
        let disabled = input(&mut dom, fieldset, "text");
        assert_eq!(
            disabled_adapter(&dom, exempt),
            Some(false),
            "missing capability: public live disabled projection"
        );
        assert_eq!(disabled_adapter(&dom, disabled), Some(true));
        attr(&mut dom, exempt, "disabled", "");
        assert_eq!(disabled_adapter(&dom, exempt), Some(true));
        assert_eq!(disabled_adapter(&dom, f), None);
        assert_eq!(disabled_adapter(&dom, dom.document()), None);
        LayoutDomMut::remove(&mut dom, disabled);
        assert_eq!(disabled_adapter(&dom, disabled), None);
        let (mut other, _) = fixture();
        let document = other.document();
        let foreign = input(&mut other, document, "text");
        assert_eq!(disabled_adapter(&dom, foreign), None);
    }
    #[test]
    fn phase_c_file_identity_versioned_roundtrip_and_distinct_allocations() {
        let bytes: Arc<[u8]> = Arc::from([0, 128, 255]);
        let a = FormFile::new(
            "same.bin".into(),
            "APPLICATION/OCTET-STREAM".into(),
            bytes.clone(),
        )
        .unwrap();
        let b = FormFile::new("same.bin".into(), "application/octet-stream".into(), bytes).unwrap();
        assert_ne!(a.id, b.id, "native allocation control");
        let wire = identity_encode_adapter(&a);
        assert!(
            wire.starts_with("genet-form-file-v1:"),
            "missing capability: versioned opaque identity codec"
        );
        assert_eq!(identity_decode_adapter(&wire), Ok(a.id));
        assert_eq!(&*a.bytes, &[0, 128, 255]);
    }
    #[test]
    fn phase_c_file_identity_invalid_tokens_are_not_absence_rejections() {
        for token in [
            "",
            "1",
            "genet-form-file-v2:0000000000000001",
            "genet-form-file-v1:0000000000000000",
            "genet-form-file-v1:ffffffffffffffff",
            "genet-form-file-v1:00000000000000GG",
            "genet-form-file-v1:0000000000000001extra",
        ] {
            assert_eq!(
                identity_decode_adapter(token),
                Err("InvalidToken"),
                "invalid token {token:?}, missing codec is not brand authority"
            );
        }
    }
}
