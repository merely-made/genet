// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The capture / at-target / bubble walk (DOM "dispatch", "invoke", "inner
//! invoke").

use std::cell::RefCell;
use std::rc::Rc;

use crate::path::EventPath;
use crate::registry::{Listener, Registry, TargetKey};

/// `Event.eventPhase` for the listener being run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Capturing = 1,
    AtTarget = 2,
    Bubbling = 3,
}

/// The event as dispatch sees it.
#[derive(Clone, Debug)]
pub struct EventInit {
    pub event_type: String,
    pub bubbles: bool,
    /// Stop flags that were already set when dispatch began (DOM keeps them
    /// across `cancelBubble = true` before `dispatchEvent`).
    pub stop_propagation: bool,
    pub stop_immediate: bool,
    pub canceled: bool,
    /// Whether the event can start an activation behaviour (a trusted-or-not
    /// `click` mouse event); only then is [`Invoker::is_activation_target`] asked.
    pub activation_event: bool,
}

/// One listener call, as the owner's [`Invoker`] receives it.
pub struct Invocation<'a> {
    pub listener: &'a Listener,
    /// The dispatch id for [`Registry::composed_path`].
    pub dispatch: u64,
    pub current_target: TargetKey,
    /// What `event.target` reads while this listener runs.
    pub target: Option<TargetKey>,
    pub phase: Phase,
    /// `preventDefault` is ignored while this is set.
    pub in_passive: bool,
}

/// The flags an owner reports back after a listener ran.
#[derive(Clone, Copy, Debug, Default)]
pub struct InvokeResult {
    pub stop_propagation: bool,
    pub stop_immediate: bool,
    pub canceled: bool,
}

pub trait Invoker {
    /// Run one listener. An owner reports its own listener failures.
    fn invoke(&mut self, call: Invocation<'_>) -> InvokeResult;

    /// Whether `target` has an activation behaviour (asked in path order for
    /// activation events only).
    fn is_activation_target(&mut self, _target: &TargetKey) -> bool {
        false
    }

    /// Run `target`'s activation behaviour after an uncanceled dispatch.
    fn activate(&mut self, _target: &TargetKey) {}
}

#[derive(Clone, Debug)]
pub struct DispatchOutcome {
    pub canceled: bool,
    /// The target the post-dispatch cleanup should expose as `event.target`
    /// (`None` when the spec's clearTargets rule nulls it).
    pub clear_targets: bool,
    pub activation: Option<TargetKey>,
}

/// Walk `path` for `event`, running each matching listener through `invoker`.
pub fn dispatch(
    registry: &RefCell<Registry>,
    path: EventPath,
    event: &EventInit,
    invoker: &mut dyn Invoker,
) -> DispatchOutcome {
    let items = Rc::new(path.items);
    let mut stop = event.stop_propagation || event.stop_immediate;
    let mut stop_immediate = event.stop_immediate;
    let mut canceled = event.canceled;
    let mut activation = None;
    if event.activation_event {
        for (index, item) in items.iter().enumerate() {
            if index > 0 && !event.bubbles {
                break;
            }
            if matches!(item.target, TargetKey::Node(_)) && invoker.is_activation_target(&item.target) {
                activation = Some(item.target);
                break;
            }
        }
    }
    let dispatch_id = registry.borrow_mut().begin_dispatch(items.clone());
    let count = items.len();
    for pass in [Pass::Capture, Pass::Bubble] {
        for step in 0..count {
            let index = match pass {
                Pass::Capture => count - 1 - step,
                Pass::Bubble => step,
            };
            let item = items[index];
            let phase = if item.adjusted.is_some() {
                Phase::AtTarget
            } else {
                if pass == Pass::Bubble && !event.bubbles {
                    continue;
                }
                match pass {
                    Pass::Capture => Phase::Capturing,
                    Pass::Bubble => Phase::Bubbling,
                }
            };
            if stop {
                continue;
            }
            let target = items[..=index].iter().rev().find_map(|i| i.adjusted);
            registry.borrow_mut().set_current(dispatch_id, Some(item.target));
            let listeners = registry.borrow().snapshot(item.target, &event.event_type);
            for listener in listeners {
                if listener.is_removed() {
                    continue;
                }
                if listener.capture != (pass == Pass::Capture) {
                    continue;
                }
                if listener.once {
                    registry.borrow_mut().remove_listener(item.target, &listener);
                }
                let result = invoker.invoke(Invocation {
                    listener: &listener,
                    dispatch: dispatch_id,
                    current_target: item.target,
                    target,
                    phase,
                    in_passive: listener.passive,
                });
                stop |= result.stop_propagation || result.stop_immediate;
                stop_immediate |= result.stop_immediate;
                canceled |= result.canceled;
                if stop_immediate {
                    break;
                }
            }
        }
    }
    registry.borrow_mut().end_dispatch(dispatch_id);
    if !canceled {
        if let Some(target) = activation {
            invoker.activate(&target);
        }
    }
    DispatchOutcome {
        canceled,
        clear_targets: path.clear_targets,
        activation,
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Pass {
    Capture,
    Bubble,
}
