// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The script binding for `genet-events`: the natives behind
//! `addEventListener`, `removeEventListener`, `dispatchEvent` and
//! `composedPath`. The listener registry and the dispatch algorithm live in
//! Rust; a JS callback stays on its target's wrapper (so a detached node and its
//! listeners are collected together) and is reached through the realm's private
//! `evHook` that the dispatching realm hands in. A listener's token is the
//! callback's agent-wide id.

use std::cell::RefCell;
use std::rc::Rc;

use genet_events::{
    build_path, dispatch, EventInit, EventPath, Invocation, InvokeResult, Invoker, ListenerSpec,
    Owner, PathItem, Registry, TargetKey,
};
use genet_scripted_dom::NodeId;
use script_engine_api::{CallCx, NativeFn, ScriptEngine};

use crate::dom::adoption::{current_host, resolve_owner};

pub(crate) fn registry<C: CallCx + ?Sized>(cx: &C) -> Option<Rc<RefCell<Registry>>> {
    let agent = current_host(cx)?.borrow().agent.upgrade()?;
    let events = agent.borrow().events.clone();
    Some(events)
}

fn key_string(key: TargetKey) -> String {
    match key {
        TargetKey::Node(id) => format!("n{}", id.raw()),
        TargetKey::Window(id) => format!("w{id}"),
        TargetKey::Object(id) => format!("o{id}"),
    }
}

fn parse_key_string(text: &str) -> Option<TargetKey> {
    let (kind, digits) = text.split_at(text.chars().next()?.len_utf8());
    let n: u64 = digits.parse().ok()?;
    match kind {
        "n" => Some(TargetKey::Node(NodeId::from_raw(n))),
        "w" => Some(TargetKey::Window(n)),
        "o" => Some(TargetKey::Object(n)),
        _ => None,
    }
}

/// A target argument: a node reflector, or a `w<id>` / `o<id>` key string.
fn target_arg<C: CallCx + ?Sized>(cx: &mut C, index: usize) -> Result<Option<TargetKey>, C::Error> {
    let value = cx.arg(index);
    if let Some(raw) = cx.reflector_data(&value) {
        return Ok(Some(TargetKey::Node(NodeId::from_raw(raw))));
    }
    let text = cx.value_to_string(&value)?;
    Ok(parse_key_string(&text))
}

fn string_arg<C: CallCx + ?Sized>(cx: &mut C, index: usize) -> Result<String, C::Error> {
    let value = cx.arg(index);
    cx.value_to_string(&value)
}

fn token_arg<C: CallCx + ?Sized>(cx: &mut C, index: usize) -> Result<u64, C::Error> {
    Ok(string_arg(cx, index)?.parse().unwrap_or(0))
}

/// `__evAdd(target, type, callbackId, flags)` → `"1"` if a new listener was
/// recorded, `"0"` for a duplicate. Flags: `c` capture, `o` once, `p` passive.
pub(crate) struct EvAdd;
impl<E: ScriptEngine> NativeFn<E> for EvAdd {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let Some(target) = target_arg(cx, 0)? else {
            return cx.make_string("0");
        };
        let event_type = string_arg(cx, 1)?;
        let token = token_arg(cx, 2)?;
        let flags = string_arg(cx, 3)?;
        let owner = Owner::SCRIPT;
        let added = registry(cx).is_some_and(|registry| {
            registry
                .borrow_mut()
                .add(
                    target,
                    ListenerSpec {
                        owner,
                        token,
                        event_type,
                        capture: flags.contains('c'),
                        once: flags.contains('o'),
                        passive: flags.contains('p'),
                    },
                )
                .is_some()
        });
        cx.make_string(if added { "1" } else { "0" })
    }
}

/// `__evRemove(target, type, callbackId, capture)` → `"0"` nothing removed,
/// `"1"` removed, `"2"` removed and no listener of that callback remains on the
/// target (the wrapper may drop its reference).
pub(crate) struct EvRemove;
impl<E: ScriptEngine> NativeFn<E> for EvRemove {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let Some(target) = target_arg(cx, 0)? else {
            return cx.make_string("0");
        };
        let event_type = string_arg(cx, 1)?;
        let token = token_arg(cx, 2)?;
        let capture = string_arg(cx, 3)? == "1";
        let owner = Owner::SCRIPT;
        let result = registry(cx).map_or("0", |registry| {
            let mut registry = registry.borrow_mut();
            if !registry.remove(target, owner, token, &event_type, capture) {
                "0"
            } else if registry.token_in_use(target, owner, token) {
                "1"
            } else {
                "2"
            }
        });
        cx.make_string(result)
    }
}

/// `__evHasListeners(target)` → `"1"` if the target carries any listener.
pub(crate) struct EvHasListeners;
impl<E: ScriptEngine> NativeFn<E> for EvHasListeners {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let found = match (target_arg(cx, 0)?, registry(cx)) {
            (Some(target), Some(registry)) => registry.borrow().has_listeners(target),
            _ => false,
        };
        cx.make_string(if found { "1" } else { "0" })
    }
}

/// `__evNewId()` → a fresh decimal id for a window or `EventTarget` object.
pub(crate) struct EvNewId;
impl<E: ScriptEngine> NativeFn<E> for EvNewId {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id = current_host(cx)
            .and_then(|host| host.borrow().agent.upgrade())
            .map_or(0, |agent| {
                let mut agent = agent.borrow_mut();
                agent.next_event_target += 1;
                agent.next_event_target
            });
        cx.make_string(&id.to_string())
    }
}

/// `__evDropObject(id)`: the object died; forget its listeners.
pub(crate) struct EvDropObject;
impl<E: ScriptEngine> NativeFn<E> for EvDropObject {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        if let (Some(registry), Ok(id)) = (registry(cx), string_arg(cx, 0)) {
            if let Ok(id) = id.parse() {
                registry.borrow_mut().drop_target(TargetKey::Object(id));
            }
        }
        Ok(cx.undefined())
    }
}

/// `__evComposedPath(dispatchId)` → comma-separated target keys, outermost last.
pub(crate) struct EvComposedPath;
impl<E: ScriptEngine> NativeFn<E> for EvComposedPath {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id = token_arg(cx, 0)?;
        let keys = registry(cx)
            .map(|registry| registry.borrow().composed_path(id))
            .unwrap_or_default();
        let text = keys.into_iter().map(key_string).collect::<Vec<_>>().join(",");
        cx.make_string(&text)
    }
}

/// Build the path for `target`, resolving a node's owning arena first.
fn path_for<C: CallCx + ?Sized>(
    cx: &mut C,
    target: TargetKey,
    window: Option<u64>,
    event_type: &str,
    composed: bool,
) -> Result<Option<EventPath>, C::Error> {
    let TargetKey::Node(id) = target else {
        return Ok(Some(EventPath {
            items: vec![PathItem {
                target,
                adjusted: Some(target),
                root_of_closed_tree: false,
                slot_in_closed_tree: false,
            }],
            clear_targets: false,
        }));
    };
    let owned = resolve_owner(cx, id.raw())?;
    Ok(owned.with_dom(|dom| {
        use layout_dom_api::LayoutDom;
        dom.is_live(id)
            .then(|| build_path(dom, target, window, event_type, composed))
    }))
}

fn window_arg<C: CallCx + ?Sized>(cx: &mut C, index: usize) -> Result<Option<u64>, C::Error> {
    Ok(string_arg(cx, index)?.parse().ok())
}

/// `__evHasNonPassive(target, type, windowKey)` → `"1"` if any listener of
/// `type` on the target's path is non-passive (the UA touch/wheel rule).
pub(crate) struct EvHasNonPassive;
impl<E: ScriptEngine> NativeFn<E> for EvHasNonPassive {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let (Some(target), Some(registry)) = (target_arg(cx, 0)?, registry(cx)) else {
            return cx.make_string("0");
        };
        let event_type = string_arg(cx, 1)?;
        let window = window_arg(cx, 2)?;
        let Some(path) = path_for(cx, target, window, &event_type, true)? else {
            return cx.make_string("0");
        };
        let keys: Vec<TargetKey> = path.items.iter().map(|item| item.target).collect();
        let found = registry.borrow().has_non_passive(&keys, &event_type);
        cx.make_string(if found { "1" } else { "0" })
    }
}

/// Runs listeners and activation behaviour through the realm hooks.
struct ScriptInvoker<'a, 'cx, E: ScriptEngine + 'cx> {
    cx: &'a mut E::CallCx<'cx>,
    /// The dispatching realm's own `__evHook`, passed in by its binding: a
    /// wrapper whose realm was discarded still runs its listeners through the
    /// code it was bound with.
    hook: E::Value,
    failure: Option<E::Error>,
}

impl<'cx, E: ScriptEngine + 'cx> ScriptInvoker<'_, 'cx, E> {
    fn call(&mut self, args: Vec<E::Value>) -> Option<String> {
        let this = self.cx.undefined();
        match E::call_from_call(self.cx, &self.hook, &this, &args) {
            Ok(value) => self.cx.value_to_string(&value).ok(),
            Err(error) => {
                self.failure.get_or_insert(error);
                None
            },
        }
    }
    fn string(&mut self, text: &str) -> Option<E::Value> {
        self.cx.make_string(text).ok()
    }
}

impl<'cx, E: ScriptEngine + 'cx> Invoker for ScriptInvoker<'_, 'cx, E> {
    fn invoke(&mut self, call: Invocation<'_>) -> InvokeResult {
        let flags = format!(
            "{}{}",
            call.phase as u8,
            if call.in_passive { "p" } else { "" }
        );
        let target = call.target.map(key_string).unwrap_or_default();
        let (Some(op), Some(current), Some(token), Some(target), Some(flags), Some(id)) = (
            self.string("invoke"),
            self.string(&key_string(call.current_target)),
            self.string(&call.listener.token.to_string()),
            self.string(&target),
            self.string(&flags),
            self.string(&call.dispatch.to_string()),
        ) else {
            return InvokeResult::default();
        };
        let args = vec![op, current, token, self.cx.arg(1), target, flags, id, self.cx.arg(2)];
        let bits = self
            .call(args)
            .and_then(|text| text.parse::<u8>().ok())
            .unwrap_or(0);
        InvokeResult {
            stop_propagation: bits & 1 != 0,
            stop_immediate: bits & 2 != 0,
            canceled: bits & 4 != 0,
        }
    }

    fn is_activation_target(&mut self, target: &TargetKey) -> bool {
        let (Some(op), Some(key)) = (self.string("activationTarget"), self.string(&key_string(*target))) else {
            return false;
        };
        self.call(vec![op, key]).as_deref() == Some("1")
    }

    fn activate(&mut self, target: &TargetKey) {
        let (Some(op), Some(key)) = (self.string("activate"), self.string(&key_string(*target))) else {
            return;
        };
        let event = self.cx.arg(1);
        self.call(vec![op, key, event]);
    }
}

/// `__evDispatch(target, event, targetObject, windowKey, type, flags, hook)` runs the
/// DOM dispatch algorithm and returns `canceled` and `clearTargets` as two
/// digits. Flags: `b` bubbles, `c` composed, `s`/`i` stop flags already set,
/// `x` canceled already, `a` activation event.
pub(crate) struct EvDispatch;
impl<E: ScriptEngine> NativeFn<E> for EvDispatch {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let Some(target) = target_arg(cx, 0)? else {
            return cx.make_string("00");
        };
        let window = window_arg(cx, 3)?;
        let event_type = string_arg(cx, 4)?;
        let flags = string_arg(cx, 5)?;
        let has = |c: char| flags.contains(c);
        let event = EventInit {
            event_type: event_type.clone(),
            bubbles: has('b'),
            stop_propagation: has('s'),
            stop_immediate: has('i'),
            canceled: has('x'),
            activation_event: has('a'),
        };
        let Some(registry) = registry(cx) else {
            return cx.make_string("00");
        };
        // Nobody listens to this type and nothing can activate: no path needed.
        if !event.activation_event && !registry.borrow().has_type(&event_type) {
            return cx.make_string(if event.canceled { "10" } else { "00" });
        }
        let Some(path) = path_for(cx, target, window, &event_type, has('c'))? else {
            return cx.make_string("00");
        };
        let hook = cx.arg(6);
        let mut invoker = ScriptInvoker::<E> {
            cx,
            hook,
            failure: None,
        };
        let outcome = dispatch(&registry, path, &event, &mut invoker);
        if let Some(error) = invoker.failure {
            return Err(error);
        }
        cx.make_string(&format!(
            "{}{}",
            u8::from(outcome.canceled),
            u8::from(outcome.clear_targets)
        ))
    }
}

pub(crate) fn install<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    engine.set_function::<EvAdd>("__evAdd", 4)?;
    engine.set_function::<EvRemove>("__evRemove", 4)?;
    engine.set_function::<EvDispatch>("__evDispatch", 7)?;
    engine.set_function::<EvComposedPath>("__evComposedPath", 1)?;
    engine.set_function::<EvHasNonPassive>("__evHasNonPassive", 3)?;
    engine.set_function::<EvHasListeners>("__evHasListeners", 1)?;
    engine.set_function::<EvNewId>("__evNewId", 0)?;
    engine.set_function::<EvDropObject>("__evDropObject", 1)?;
    Ok(())
}
