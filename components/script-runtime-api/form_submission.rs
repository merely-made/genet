// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Shared native decisions, request planning and private host activation.
use crate::{
    OwnerResolvedCx as _,
    dom::{adoption, reflect_pinned},
    form_data,
};
use genet_scripted_dom::{FormEditCommand, FormEditOutcome, FormImplicitSubmission, NodeId};
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use script_engine_api::{CallCx, NativeFn, ScriptEngine};

/// Physical key modifiers supplied by a native input host.
#[derive(Clone, Copy, Debug, Default)]
pub struct FormKeyModifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub meta: bool,
    pub shift: bool,
}
/// The only script-facing user edit callback. The shared DOM owns value,
/// selection, grapheme movement, draft state, and control eligibility.
pub(crate) struct HostFormEdit;
impl<E: ScriptEngine> NativeFn<E> for HostFormEdit {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let value = cx.arg(0);
        let Some(node) = cx.owned_node(&value)? else { return cx.make_string("rejected") };
        let operation = cx.arg(1);
        let operation = cx.value_to_string(&operation)?;
        let text = cx.arg(2);
        let text = cx.value_to_string(&text)?;
        let command = match operation.as_str() {
            "insert" => FormEditCommand::Insert(&text),
            "backward" => FormEditCommand::DeleteBackward,
            "forward" => FormEditCommand::DeleteForward,
            "left" => FormEditCommand::MoveLeft,
            "right" => FormEditCommand::MoveRight,
            "home" => FormEditCommand::Home,
            "end" => FormEditCommand::End,
            _ => return cx.make_string("rejected"),
        };
        let result = node.with_dom(|dom| {
            if !dom.can_user_edit_text_control(node.id()) {
                return FormEditOutcome::Rejected;
            }
            if dom.form_control_interaction_state(node.id())
                .is_some_and(|state| state.user_edit_initial_value.is_none())
            {
                dom.begin_form_control_user_edit(node.id());
            }
            dom.apply_form_control_user_edit(node.id(), command)
        });
        cx.make_string(match result {
            FormEditOutcome::Rejected => "rejected",
            FormEditOutcome::Unchanged => "unchanged",
            FormEditOutcome::SelectionChanged => "selection",
            FormEditOutcome::ValueEdited => "edited",
        })
    }
}
pub(crate) struct ControlInfo;
impl<E: ScriptEngine> NativeFn<E> for ControlInfo {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let v = cx.arg(0);
        let Some(node) = cx.owned_node(&v)? else {
            return cx.make_string("null");
        };
        let (submit, disabled, implicit) = node.with_dom(|dom| {
            (
                dom.is_form_submit_button(node.id()),
                dom.form_control_is_disabled(node.id()).unwrap_or(false),
                dom.form_control_implicit_submit_eligible(node.id()),
            )
        });
        cx.make_string(&format!(
            "{{\"submit\":{submit},\"disabled\":{disabled},\"implicit\":{implicit}}}"
        ))
    }
}
pub(crate) struct SubmitterCheck;
impl<E: ScriptEngine> NativeFn<E> for SubmitterCheck {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let f = cx.arg(0);
        let b = cx.arg(1);
        let Some(form) = cx.owned_node(&f)? else {
            return cx.make_string("TypeError");
        };
        let valid = form.with_dom(|dom| {
            dom.element_name(form.id()).is_some_and(|name| {
                name.local.as_ref() == "form" && name.ns.as_ref() == "http://www.w3.org/1999/xhtml"
            })
        });
        if !valid {
            return cx.make_string("TypeError");
        }
        let Some(button) = cx.owned_node(&b)? else {
            return cx.make_string("");
        };
        let (submit, owner) = button.with_dom(|dom| {
            (
                dom.is_form_submit_button(button.id()),
                dom.form_control_form_owner(button.id()),
            )
        });
        cx.make_string(if !submit {
            "TypeError"
        } else if owner != Some(form.id()) {
            "NotFoundError"
        } else {
            ""
        })
    }
}
pub(crate) struct ImplicitDecision;
impl<E: ScriptEngine> NativeFn<E> for ImplicitDecision {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let f = cx.arg(0);
        let Some(form) = cx.owned_node(&f)? else {
            return cx.make_string("blocked");
        };
        let decision = form.with_dom(|dom| dom.implicit_form_submission(form.id()));
        match decision {
            Ok(FormImplicitSubmission::SubmitButton(id)) => reflect_pinned::<E>(cx, id.raw()),
            Ok(FormImplicitSubmission::SubmitForm) => cx.make_string("form"),
            _ => cx.make_string("blocked"),
        }
    }
}
pub(crate) struct SubmissionUserValidity;
impl<E: ScriptEngine> NativeFn<E> for SubmissionUserValidity {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let value = cx.arg(0);
        if let Some(form) = cx.owned_node(&value)? {
            form.with_dom(|dom| dom.mark_form_submission_user_validity(form.id()));
        }
        Ok(cx.undefined())
    }
}
fn attr(dom: &genet_scripted_dom::ScriptedDom, id: NodeId, key: &str) -> Option<String> {
    dom.attribute(id, &Namespace::from(""), &LocalName::from(key))
        .map(str::to_owned)
}
fn fallback_target(dom: &genet_scripted_dom::ScriptedDom) -> String {
    let mut stack = vec![dom.document()];
    while let Some(node) = stack.pop() {
        if dom.element_name(node).is_some_and(|name| {
            name.local.as_ref() == "base" && name.ns.as_ref() == "http://www.w3.org/1999/xhtml"
        }) {
            if let Some(target) = attr(dom, node, "target") {
                return if target.contains(['\n', '\t', '<']) {
                    "_blank".into()
                } else {
                    target
                };
            }
        }
        let children: Vec<_> = dom.dom_children(node).collect();
        stack.extend(children.into_iter().rev());
    }
    String::new()
}
struct Options {
    action: String,
    method: String,
    enctype: String,
    target: String,
    encoding: &'static genet_scripted_dom::form_encoding::Encoding,
    no_validate: bool,
    eligible: bool,
    referrer: String,
    policy: String,
    resolution_base: Option<String>,
}
fn options(node: &adoption::OwnedNode, submitter: Option<&adoption::OwnedNode>) -> Options {
    // Listener mutations may change the captured submitter's type or owner.
    // Read its current attributes in its actual owning arena, before borrowing
    // the original form's host. That form still owns the entries and encoding.
    let (action, method, enctype, target, no_validate, resolution_base) =
        submitter.unwrap_or(node).with_host(|host| {
            let resolution_base = host.document_base_url();
            let dom = &host.dom;
            let owner = match submitter {
                Some(button) => dom.form_control_form_owner(button.id()),
                None => Some(node.id()),
            };
            let button = submitter.filter(|button| dom.is_form_submit_button(button.id()));
            let choose = |key: &str, override_key: &str| {
                button
                    .and_then(|button| attr(dom, button.id(), override_key))
                    .or_else(|| owner.and_then(|owner| attr(dom, owner, key)))
            };
            (
                choose("action", "formaction"),
                choose("method", "formmethod")
                    .unwrap_or_default()
                    .to_ascii_lowercase(),
                choose("enctype", "formenctype")
                    .unwrap_or_default()
                    .to_ascii_lowercase(),
                choose("target", "formtarget")
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| fallback_target(dom)),
                owner.is_some_and(|owner| attr(dom, owner, "novalidate").is_some())
                    || button
                        .is_some_and(|button| attr(dom, button.id(), "formnovalidate").is_some()),
                resolution_base,
            )
        });
    let realm = node.owner_realm();
    node.with_host(|host| {
        let allowed = host
            .agent
            .upgrade()
            .is_some_and(|a| a.borrow().frames.form_submission_allowed(realm));
        let eligible = host.is_connected_node(node.id()) && allowed;
        let dom = &host.dom;
        let encoding = genet_scripted_dom::form_encoding::select_form_encoding(
            attr(dom, node.id(), "accept-charset").as_deref(),
            dom.document_encoding(dom.document())
                .map(|e| e.encoding)
                .unwrap_or(genet_scripted_dom::form_encoding::UTF_8),
        );
        let base = host
            .base_url
            .clone()
            .unwrap_or_else(|| "about:blank".into());
        Options {
            action: action
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| base.clone()),
            method: if method == "post" {
                "POST".into()
            } else if method == "dialog" {
                "dialog".into()
            } else {
                "GET".into()
            },
            enctype: if matches!(enctype.as_str(), "text/plain" | "multipart/form-data") {
                enctype
            } else {
                "application/x-www-form-urlencoded".into()
            },
            target,
            encoding,
            no_validate,
            eligible,
            resolution_base,
            referrer: base,
            policy: if attr(dom, node.id(), "rel").is_some_and(|s| {
                s.split_ascii_whitespace()
                    .any(|s| s.eq_ignore_ascii_case("noreferrer"))
            }) {
                "no-referrer".into()
            } else {
                String::new()
            },
        }
    })
}
pub(crate) struct FormMeta;
impl<E: ScriptEngine> NativeFn<E> for FormMeta {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let f = cx.arg(0);
        let b = cx.arg(1);
        let form = cx
            .owned_node(&f)?
            .ok_or_else(|| form_data::type_error::<E>(cx, "Invalid form"))?;
        let button = cx.owned_node(&b)?;
        let o = options(&form, button.as_ref());
        cx.make_string(&format!(
            "{{\"encoding\":{},\"noValidate\":{},\"eligible\":{}}}",
            crate::js_str(o.encoding.name()),
            o.no_validate,
            o.eligible
        ))
    }
}
pub(crate) struct FormSubmissionPlan;
impl<E: ScriptEngine> NativeFn<E> for FormSubmissionPlan {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let f = cx.arg(0);
        let b = cx.arg(1);
        let fd = cx.arg(2);
        let selected_encoding = cx.arg(3);
        let entries = form_data::entries::<E>(cx, fd)?;
        let selected_encoding = cx.value_to_string(&selected_encoding)?;
        let encoding =
            genet_scripted_dom::form_encoding::Encoding::for_label(selected_encoding.as_bytes())
                .ok_or_else(|| form_data::type_error::<E>(cx, "Invalid selected form encoding"))?
                .output_encoding();
        let form = cx
            .owned_node(&f)?
            .ok_or_else(|| form_data::type_error::<E>(cx, "Invalid form"))?;
        let button = cx.owned_node(&b)?;
        let o = options(&form, button.as_ref());
        if !o.eligible || o.method == "dialog" {
            return Ok(cx.undefined());
        }
        let resolved = crate::fetch::resolve_against(o.resolution_base.as_deref(), &o.action);
        let Ok(mut url) = url::Url::parse(&resolved) else {
            return Ok(cx.undefined());
        };
        let (body, headers) = if o.method == "GET" {
            let bytes = genet_scripted_dom::form_encoding::encode_urlencoded(&entries, encoding);
            url.set_query(Some(std::str::from_utf8(&bytes).expect("ASCII URL pairs")));
            (None, Vec::new())
        } else {
            let (bytes, content_type) = form_data::encoded(&entries, encoding, &o.enctype);
            (Some(bytes), vec![("content-type".into(), content_type)])
        };
        let from = form.owner_realm();
        let Some(target) = crate::frames::navigation_target::<E>(cx, from, &o.target) else {
            return Ok(cx.undefined());
        };
        let request = crate::FetchRequest {
            method: o.method,
            url: url.to_string(),
            headers,
            body,
            cache: "default".into(),
            redirect: "follow".into(),
            mode: "navigate".into(),
            referrer: o.referrer,
            referrer_policy: o.policy,
            credentials: "include".into(),
            integrity: String::new(),
        };
        crate::frames::navigate_form_request::<E>(cx, from, target, request, false, form.id())
    }
}
pub(crate) fn install<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    engine.set_function::<HostFormEdit>("__hostFormEdit", 3)?;
    engine.set_function::<ControlInfo>("__formControlInfo", 1)?;
    engine.set_function::<SubmitterCheck>("__formSubmitterCheck", 2)?;
    engine.set_function::<ImplicitDecision>("__formImplicitDecision", 1)?;
    engine.set_function::<FormMeta>("__formMeta", 2)?;
    engine.set_function::<SubmissionUserValidity>("__formSubmissionUserValidity", 1)?;
    engine.set_function::<FormSubmissionPlan>("__formSubmissionPlan", 4)?;
    Ok(())
}
impl<E: ScriptEngine> crate::Runtime<E> {
    fn private_form_host_result(&mut self, raw: u64, op: &str, details: &str, checkpoint: bool) -> Result<Option<String>, E::Error> {
        let Some((realm, _)) = adoption::owner_host(&self.host, NodeId::from_raw(raw)) else {
            return Ok(None);
        };
        let hook = self.agent.borrow().dom_adoption.hook(realm);
        let Some(hook) = hook else { return Ok(None) };
        let Some(function) = hook.downcast_ref::<E::Value>() else {
            return Ok(None);
        };
        let op = self
            .engine
            .eval_in_realm(realm, &format!("({})", crate::js_str(op)))
            .map_err(|e| self.engine_error(&e.to_string()))?;
        let id = self
            .engine
            .eval_in_realm(realm, &format!("('{raw}')"))
            .map_err(|e| self.engine_error(&e.to_string()))?;
        let details = self
            .engine
            .eval_in_realm(realm, &format!("({})", crate::js_str(details)))
            .map_err(|e| self.engine_error(&e.to_string()))?;
        let this = self
            .engine
            .eval_in_realm(realm, "void 0")
            .map_err(|e| self.engine_error(&e.to_string()))?;
        let result = self
            .engine
            .call_function(function, &this, &[op, id, details])
            .map_err(|e| self.engine_error(&e.to_string()))?;
        let result = self
            .engine
            .value_to_string(&result)
            .ok();
        if checkpoint { self.perform_microtask_checkpoint(); }
        Ok(result)
    }
    fn private_form_host(&mut self, raw: u64, op: &str, details: &str) -> Result<bool, E::Error> {
        Ok(self.private_form_host_result(raw, op, details, true)?.is_some_and(|value| value != "false"))
    }
    /// A host input turn finishes its defaults before this checkpoint.
    pub fn finish_host_input(&mut self) { self.perform_microtask_checkpoint(); }
    pub fn dispatch_host_focus_step(&mut self, raw: u64, focused: bool) -> Result<bool, E::Error> {
        Ok(self.private_form_host_result(raw, "formFocusRaw", if focused { "true" } else { "false" }, false)?.is_some_and(|value| value != "false"))
    }
    pub fn dispatch_host_text_edit(&mut self, raw: u64, operation: &str, text: &str, input_type: &str) -> Result<String, E::Error> {
        let details = format!("{{\"operation\":{},\"text\":{},\"inputType\":{}}}", crate::js_str(operation), crate::js_str(text), crate::js_str(input_type));
        Ok(self.private_form_host_result(raw, "hostTextRaw", &details, false)?.unwrap_or_else(|| "rejected".into()))
    }
    pub fn dispatch_host_change(&mut self, raw: u64) -> Result<bool, E::Error> {
        Ok(self.private_form_host_result(raw, "hostChangeRaw", "", false)?.is_some_and(|value| value != "false"))
    }
    pub fn dispatch_host_composition(&mut self, raw: u64, phase: &str, text: &str) -> Result<bool, E::Error> {
        let details = format!("{{\"phase\":{},\"text\":{}}}", crate::js_str(phase), crate::js_str(text));
        Ok(self.private_form_host_result(raw, "hostCompositionRaw", &details, false)?.is_some_and(|value| value != "false"))
    }
    /// Set the represented realm's private host focus projection.
    pub fn dispatch_form_focus(
        &mut self,
        raw_node_id: u64,
        focused: bool,
    ) -> Result<bool, E::Error> {
        if focused {
            let Some((_, host)) = adoption::owner_host(&self.host, NodeId::from_raw(raw_node_id))
            else {
                return Ok(false);
            };
            if !host
                .borrow_mut()
                .is_connected_node(NodeId::from_raw(raw_node_id))
            {
                return Ok(false);
            }
        }
        self.private_form_host(
            raw_node_id,
            "formFocusRaw",
            if focused { "true" } else { "false" },
        )
    }
    /// Dispatch a trusted pointer click, then its cancellation/default action,
    /// before the host microtask checkpoint. The position is relative to an image.
    pub fn dispatch_form_activation(
        &mut self,
        raw_node_id: u64,
        image_position: Option<(u32, u32)>,
    ) -> Result<bool, E::Error> {
        let (x, y) = image_position.unwrap_or((0, 0));
        self.private_form_host(raw_node_id, "formActivateRaw", &format!("{x},{y}"))
    }
    /// Dispatch a trusted keydown and, for unmodified Enter, use the shared
    /// native implicit-submission decision before the microtask checkpoint.
    pub fn dispatch_form_keydown(
        &mut self,
        raw_node_id: u64,
        key: &str,
        modifiers: FormKeyModifiers,
    ) -> Result<bool, E::Error> {
        let text = format!(
            "{{\"key\":{},\"alt\":{},\"ctrl\":{},\"meta\":{},\"shift\":{}}}",
            crate::js_str(key),
            modifiers.alt,
            modifiers.ctrl,
            modifiers.meta,
            modifiers.shift
        );
        self.private_form_host(raw_node_id, "formKeyRaw", &text)
    }
    /// Dispatch a host keyboard event without checkpointing; the embedding
    /// document still has to apply Tab and text defaults for this same turn.
    pub fn dispatch_host_key_step(&mut self, raw: u64, key: &str, down: bool, repeat: bool, modifiers: FormKeyModifiers) -> Result<bool, E::Error> {
        let details = format!(
            "{{\"key\":{},\"down\":{},\"repeat\":{},\"alt\":{},\"ctrl\":{},\"meta\":{},\"shift\":{}}}",
            crate::js_str(key), down, repeat, modifiers.alt, modifiers.ctrl, modifiers.meta, modifiers.shift
        );
        Ok(self.private_form_host_result(raw, "hostKeyRaw", &details, false)?.is_some_and(|value| value != "false"))
    }
}
