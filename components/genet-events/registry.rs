// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Listener storage and the bookkeeping of dispatches in flight.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use genet_scripted_dom::NodeId;

use crate::path::PathItem;

/// An event target. Nodes are keyed by their agent-wide id; windows and other
/// script `EventTarget`s by an id the binding allocates.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TargetKey {
    Node(NodeId),
    Window(u64),
    Object(u64),
}

/// Who runs a listener. `Owner::SCRIPT` is the JS binding; native owners
/// (a toolkit runner) take other values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Owner(pub u32);

impl Owner {
    pub const SCRIPT: Owner = Owner(0);
}

/// What `addEventListener` records.
#[derive(Clone, Debug)]
pub struct ListenerSpec {
    pub owner: Owner,
    /// Owner-defined handle. Two listeners on one target with the same owner,
    /// token, type and capture are the same listener (DOM duplicate rule).
    pub token: u64,
    pub event_type: String,
    pub capture: bool,
    pub once: bool,
    pub passive: bool,
}

/// A registered listener. Shared with in-flight dispatches, which consult
/// [`Listener::is_removed`] before running it.
#[derive(Debug)]
pub struct Listener {
    pub id: u64,
    pub owner: Owner,
    pub token: u64,
    pub event_type: Rc<str>,
    pub capture: bool,
    pub once: bool,
    pub passive: bool,
    removed: Cell<bool>,
}

impl Listener {
    pub fn is_removed(&self) -> bool {
        self.removed.get()
    }
}

struct ActiveDispatch {
    path: Rc<Vec<PathItem>>,
    current: Option<TargetKey>,
}

#[derive(Default)]
pub struct Registry {
    lists: HashMap<TargetKey, Vec<Rc<Listener>>>,
    type_counts: HashMap<Rc<str>, usize>,
    next_listener: u64,
    next_dispatch: u64,
    active: HashMap<u64, ActiveDispatch>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a listener unless an identical one exists. `None` for a duplicate.
    pub fn add(&mut self, target: TargetKey, spec: ListenerSpec) -> Option<Rc<Listener>> {
        let list = self.lists.entry(target).or_default();
        let duplicate = list.iter().any(|l| {
            l.owner == spec.owner
                && l.token == spec.token
                && l.capture == spec.capture
                && *l.event_type == *spec.event_type
        });
        if duplicate {
            return None;
        }
        self.next_listener += 1;
        let event_type = self
            .type_counts
            .get_key_value(spec.event_type.as_str())
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| Rc::from(spec.event_type.as_str()));
        let listener = Rc::new(Listener {
            id: self.next_listener,
            owner: spec.owner,
            token: spec.token,
            event_type: event_type.clone(),
            capture: spec.capture,
            once: spec.once,
            passive: spec.passive,
            removed: Cell::new(false),
        });
        list.push(listener.clone());
        *self.type_counts.entry(event_type).or_default() += 1;
        Some(listener)
    }

    /// Remove the matching listener, flagging it so in-flight dispatches skip it.
    pub fn remove(
        &mut self,
        target: TargetKey,
        owner: Owner,
        token: u64,
        event_type: &str,
        capture: bool,
    ) -> bool {
        let Some(list) = self.lists.get(&target) else {
            return false;
        };
        let Some(found) = list.iter().find(|l| {
            l.owner == owner && l.token == token && l.capture == capture && *l.event_type == *event_type
        }) else {
            return false;
        };
        let found = found.clone();
        self.remove_listener(target, &found);
        true
    }

    /// Remove one listener by identity (the `once` path).
    pub fn remove_listener(&mut self, target: TargetKey, listener: &Rc<Listener>) {
        if listener.removed.replace(true) {
            return;
        }
        if let Some(list) = self.lists.get_mut(&target) {
            list.retain(|l| !Rc::ptr_eq(l, listener));
            if list.is_empty() {
                self.lists.remove(&target);
            }
        }
        self.uncount(&listener.event_type);
    }

    fn uncount(&mut self, event_type: &Rc<str>) {
        if let Some(n) = self.type_counts.get_mut(event_type) {
            *n -= 1;
            if *n == 0 {
                self.type_counts.remove(event_type);
            }
        }
    }

    /// Whether `target` still has a listener from `owner` carrying `token`.
    pub fn token_in_use(&self, target: TargetKey, owner: Owner, token: u64) -> bool {
        self.lists
            .get(&target)
            .is_some_and(|list| list.iter().any(|l| l.owner == owner && l.token == token))
    }

    pub fn has_listeners(&self, target: TargetKey) -> bool {
        self.lists.contains_key(&target)
    }

    /// Whether any listener of `event_type` exists anywhere: the fast path that
    /// lets a hot event skip path construction when nobody is listening.
    pub fn has_type(&self, event_type: &str) -> bool {
        self.type_counts.contains_key(event_type)
    }

    /// Listeners of `event_type` on `target`, in registration order.
    pub fn snapshot(&self, target: TargetKey, event_type: &str) -> Vec<Rc<Listener>> {
        self.lists
            .get(&target)
            .map(|list| {
                list.iter()
                    .filter(|l| *l.event_type == *event_type)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether any of `targets` carries a non-passive listener of `event_type`.
    pub fn has_non_passive(&self, targets: &[TargetKey], event_type: &str) -> bool {
        targets.iter().any(|t| {
            self.lists.get(t).is_some_and(|list| {
                list.iter()
                    .any(|l| *l.event_type == *event_type && !l.passive)
            })
        })
    }

    /// Whether `target` has any listener, of any type.
    pub fn any_listener_on(&self, target: TargetKey) -> bool {
        self.has_listeners(target)
    }

    /// Drop every listener on `target`; returns the tokens that were held.
    pub fn drop_target(&mut self, target: TargetKey) -> Vec<(Owner, u64)> {
        let Some(list) = self.lists.remove(&target) else {
            return Vec::new();
        };
        let mut tokens = Vec::with_capacity(list.len());
        for l in list {
            l.removed.set(true);
            self.uncount(&l.event_type);
            tokens.push((l.owner, l.token));
        }
        tokens
    }

    /// Drop the listeners of every node target `live` rejects.
    pub fn retain_nodes(&mut self, live: impl Fn(NodeId) -> bool) -> usize {
        let dead: Vec<TargetKey> = self
            .lists
            .keys()
            .filter(|k| matches!(k, TargetKey::Node(id) if !live(*id)))
            .copied()
            .collect();
        let n = dead.len();
        for key in dead {
            self.drop_target(key);
        }
        n
    }

    pub fn target_count(&self) -> usize {
        self.lists.len()
    }

    pub(crate) fn begin_dispatch(&mut self, path: Rc<Vec<PathItem>>) -> u64 {
        self.next_dispatch += 1;
        self.active.insert(
            self.next_dispatch,
            ActiveDispatch {
                path,
                current: None,
            },
        );
        self.next_dispatch
    }

    pub(crate) fn set_current(&mut self, dispatch: u64, current: Option<TargetKey>) {
        if let Some(active) = self.active.get_mut(&dispatch) {
            active.current = current;
        }
    }

    pub(crate) fn end_dispatch(&mut self, dispatch: u64) {
        self.active.remove(&dispatch);
    }

    /// `Event.composedPath()` for a dispatch in flight: empty once it has ended.
    pub fn composed_path(&self, dispatch: u64) -> Vec<TargetKey> {
        match self.active.get(&dispatch) {
            Some(ActiveDispatch {
                path,
                current: Some(current),
            }) => crate::path::composed_path(path, *current),
            _ => Vec::new(),
        }
    }
}
