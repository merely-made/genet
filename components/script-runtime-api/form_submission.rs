// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Shared native decisions, request planning and private host activation.
use crate::{
    OwnerResolvedCx as _,
    dom::{adoption, reflect_pinned},
    form_data,
};
use genet_scripted_dom::{FormImplicitSubmission, NodeId};
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
pub(crate) struct ControlInfo;
impl<E: ScriptEngine> NativeFn<E> for ControlInfo {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let v = cx.arg(0);
        let Some(node) = cx.owned_node(&v)? else {
            return cx.make_string("null");
        };
        let (submit, disabled) = node.with_dom(|dom| {
            (
                dom.is_form_submit_button(node.id()),
                dom.form_control_is_disabled(node.id()).unwrap_or(false),
            )
        });
        cx.make_string(&format!("{{\"submit\":{submit},\"disabled\":{disabled}}}"))
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
}
fn options(node: &adoption::OwnedNode, submitter: Option<NodeId>) -> Options {
    let realm = node.owner_realm();
    node.with_host(|host| {
        let allowed = host
            .agent
            .upgrade()
            .is_some_and(|a| a.borrow().frames.form_submission_allowed(realm));
        let eligible = host.is_connected_node(node.id()) && allowed;
        let dom = &host.dom;
        let choose = |key: &str, override_key: &str| {
            submitter
                .and_then(|s| attr(dom, s, override_key))
                .or_else(|| attr(dom, node.id(), key))
        };
        let encoding = genet_scripted_dom::form_encoding::select_form_encoding(
            attr(dom, node.id(), "accept-charset").as_deref(),
            dom.document_encoding(dom.document())
                .map(|e| e.encoding)
                .unwrap_or(genet_scripted_dom::form_encoding::UTF_8),
        );
        let method = choose("method", "formmethod")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let enctype = choose("enctype", "formenctype")
            .unwrap_or_default()
            .to_ascii_lowercase();
        let base = host
            .base_url
            .clone()
            .unwrap_or_else(|| "about:blank".into());
        Options {
            action: choose("action", "formaction")
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
            target: choose("target", "formtarget")
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| fallback_target(dom)),
            encoding,
            no_validate: attr(dom, node.id(), "novalidate").is_some()
                || submitter.is_some_and(|s| attr(dom, s, "formnovalidate").is_some()),
            eligible,
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
        let button = cx.owned_node(&b)?.map(|s| s.id());
        let o = options(&form, button);
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
        let button = cx.owned_node(&b)?.map(|s| s.id());
        let o = options(&form, button);
        if !o.eligible || o.method == "dialog" {
            return Ok(cx.undefined());
        }
        if let Some(button) = button {
            let good = form.with_dom(|dom| {
                dom.is_form_submit_button(button)
                    && dom.form_control_form_owner(button) == Some(form.id())
            });
            if !good {
                return Ok(cx.undefined());
            }
        }
        let base = form.with_host(|h| h.document_base_url());
        let resolved = crate::fetch::resolve_against(base.as_deref(), &o.action);
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
    engine.set_function::<ControlInfo>("__formControlInfo", 1)?;
    engine.set_function::<SubmitterCheck>("__formSubmitterCheck", 2)?;
    engine.set_function::<ImplicitDecision>("__formImplicitDecision", 1)?;
    engine.set_function::<FormMeta>("__formMeta", 2)?;
    engine.set_function::<FormSubmissionPlan>("__formSubmissionPlan", 4)?;
    Ok(())
}
impl<E: ScriptEngine> crate::Runtime<E> {
    fn private_form_host(&mut self, raw: u64, op: &str, details: &str) -> Result<bool, E::Error> {
        let Some((realm, _)) = adoption::owner_host(&self.host, NodeId::from_raw(raw)) else {
            return Ok(false);
        };
        let hook = self.agent.borrow().dom_adoption.hook(realm);
        let Some(hook) = hook else { return Ok(false) };
        let Some(function) = hook.downcast_ref::<E::Value>() else {
            return Ok(false);
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
            .map(|s| s != "false")
            .unwrap_or(false);
        self.perform_microtask_checkpoint();
        Ok(result)
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
}
