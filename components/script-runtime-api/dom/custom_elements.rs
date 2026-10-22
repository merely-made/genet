// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Custom element sinks: the element's state on the arena node, the reaction
//! recording switch, and the drain of the arena's reaction candidates.
//!
//! The registry, the custom element reactions stack and callback invocation live
//! in the JS bootstrap. The arena records which custom elements were connected,
//! disconnected or had an attribute changed, at the mutation itself, so every
//! caller produces the same facts (G5).

use super::mutation_observer::{escape, old_field};
use super::*;
use genet_scripted_dom::{CustomElementState, ReactionCandidate};

/// Encode drained candidates, one per line, fields tab-separated:
///
/// - `C` node — connected
/// - `D` node — disconnected
/// - `J` node oldDocument newDocument — adopted
/// - `A` node localName namespace hasOld old hasNew new — attribute changed
pub(super) fn encode(candidates: &[ReactionCandidate], dom: &ScriptedDom) -> String {
    let mut out = String::new();
    for candidate in candidates {
        let node = candidate.node();
        if !dom.is_live(node) {
            continue;
        }
        let line = match candidate {
            ReactionCandidate::Connected(_) => format!("C\t{}", node.raw()),
            ReactionCandidate::Disconnected(_) => format!("D\t{}", node.raw()),
            ReactionCandidate::Adopted {
                old_document,
                new_document,
                ..
            } => format!(
                "J\t{}\t{}\t{}",
                node.raw(),
                old_document.raw(),
                new_document.raw()
            ),
            ReactionCandidate::AttributeChanged {
                name,
                old_value,
                new_value,
                ..
            } => format!(
                "A\t{}\t{}\t{}\t{}\t{}",
                node.raw(),
                escape(&name.local),
                escape(&name.ns),
                old_field(old_value.as_ref()),
                old_field(new_value.as_ref()),
            ),
        };
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&line);
    }
    out
}

/// `__ceState(element)` → the element's custom element state.
pub(crate) struct State;
impl<E: ScriptEngine> NativeFn<E> for State {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let node = cx.arg(0);
        let state = match cx.owned_node(&node)? {
            Some(raw) => raw.with_dom(|dom| dom.custom_element_state(raw.id())),
            None => CustomElementState::default(),
        };
        cx.make_string(state.as_str())
    }
}

/// `__ceSetState(element, state)` → `"1"` when the state was set.
pub(crate) struct SetState;
impl<E: ScriptEngine> NativeFn<E> for SetState {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let node = cx.arg(0);
        let state_v = cx.arg(1);
        let text = cx.value_to_string(&state_v)?;
        let Some(state) = CustomElementState::parse(&text) else {
            return Err(cx.error("unknown custom element state"));
        };
        let set = match cx.owned_node(&node)? {
            Some(raw) => raw.with_dom(|dom| dom.set_custom_element_state(raw.id(), state)),
            None => false,
        };
        cx.make_string(if set { "1" } else { "0" })
    }
}

/// `__ceRecording(flag)` — this realm's arena records reaction candidates.
/// Used where no agent coordinates several realms; the agent path is the
/// `ceActive` dispatch.
pub(crate) struct SetRecording;
impl<E: ScriptEngine> NativeFn<E> for SetRecording {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let arg = cx.arg(0);
        let on = cx.value_to_string(&arg)? == "1";
        with_dom::<E, _>(cx, |dom| dom.set_reaction_recording(on));
        Ok(cx.undefined())
    }
}

/// `__ceTake()` — drain this realm's arena candidates. Empty when none.
pub(crate) struct Take;
impl<E: ScriptEngine> NativeFn<E> for Take {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let encoded = with_dom::<E, _>(cx, |dom| {
            let candidates = dom.take_reaction_candidates();
            encode(&candidates, dom)
        })
        .unwrap_or_default();
        cx.make_string(&encoded)
    }
}

/// `__ceAdopted(root, oldDocument, newDocument)` — record `adopted` for the
/// custom elements in `root`'s shadow-including inclusive descendants. The
/// adoption step calls it because the owning document is not yet an arena
/// relation (the plan's F4 ruling).
pub(crate) struct Adopted;
impl<E: ScriptEngine> NativeFn<E> for Adopted {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let root = cx.arg(0);
        let old_v = cx.arg(1);
        let new_v = cx.arg(2);
        let (Some(root), Some(old_doc), Some(new_doc)) = (
            cx.owned_node(&root)?,
            cx.owned_node(&old_v)?,
            cx.owned_node(&new_v)?,
        ) else {
            return Ok(cx.undefined());
        };
        let (old_id, new_id) = (old_doc.id(), new_doc.id());
        root.with_dom(|dom| dom.record_adopted(root.id(), old_id, new_id));
        Ok(cx.undefined())
    }
}

/// `__ceReactionMembers()` → `Interface.member` pairs the vendored IDL marks
/// `[CEReactions]`, comma-separated. The bootstrap wraps each one it defines.
pub(crate) struct ReactionMembers;
impl<E: ScriptEngine> NativeFn<E> for ReactionMembers {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let text = super::html_interfaces_generated::CE_REACTIONS
            .iter()
            .map(|(iface, member)| format!("{iface}.{member}"))
            .collect::<Vec<_>>()
            .join(",");
        cx.make_string(&text)
    }
}

pub(crate) fn install<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    engine.set_function::<State>("__ceState", 1)?;
    engine.set_function::<SetState>("__ceSetState", 2)?;
    engine.set_function::<SetRecording>("__ceRecording", 1)?;
    engine.set_function::<Take>("__ceTake", 0)?;
    engine.set_function::<Adopted>("__ceAdopted", 3)?;
    engine.set_function::<ReactionMembers>("__ceReactionMembers", 0)?;
    Ok(())
}
