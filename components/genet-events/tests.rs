// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, 
    LayoutDomMut, LocalName, Namespace, QualName, ShadowRootInit, ShadowRootMode,
};

use crate::*;

fn el(dom: &mut ScriptedDom, name: &str) -> NodeId {
    dom.create_element(QualName::new(None, Namespace::from("http://www.w3.org/1999/xhtml"), LocalName::from(name)))
}

fn spec(token: u64, ty: &str, capture: bool) -> ListenerSpec {
    ListenerSpec {
        owner: Owner::SCRIPT,
        token,
        event_type: ty.into(),
        capture,
        once: false,
        passive: false,
    }
}

fn init(ty: &str, bubbles: bool) -> EventInit {
    EventInit {
        event_type: ty.into(),
        bubbles,
        stop_propagation: false,
        stop_immediate: false,
        canceled: false,
        activation_event: false,
    }
}

type Hook = Box<dyn FnMut(u64, &Invocation<'_>) -> InvokeResult>;

/// Records `(token, current, target, phase)` and lets a test react per token.
struct Rec {
    log: Vec<(u64, TargetKey, Option<TargetKey>, Phase)>,
    hook: Hook,
}

impl Rec {
    fn new() -> Self {
        Rec {
            log: Vec::new(),
            hook: Box::new(|_, _| InvokeResult::default()),
        }
    }
    fn tokens(&self) -> Vec<u64> {
        self.log.iter().map(|e| e.0).collect()
    }
}

impl Invoker for Rec {
    fn invoke(&mut self, call: Invocation<'_>) -> InvokeResult {
        self.log
            .push((call.listener.token, call.current_target, call.target, call.phase));
        (self.hook)(call.listener.token, &call)
    }
}

fn run(
    reg: &RefCell<Registry>,
    dom: &ScriptedDom,
    target: TargetKey,
    window: Option<u64>,
    ev: &EventInit,
    rec: &mut Rec,
) -> DispatchOutcome {
    let path = build_path(dom, target, window, &ev.event_type, true);
    dispatch(reg, path, ev, rec)
}

struct Tree {
    dom: ScriptedDom,
    outer: NodeId,
    inner: NodeId,
}

fn tree() -> Tree {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let outer = el(&mut dom, "div");
    let inner = el(&mut dom, "span");
    dom.append_child(root, outer);
    dom.append_child(outer, inner);
    Tree { dom, outer, inner }
}

#[test]
fn capture_target_bubble_order_with_window() {
    let t = tree();
    let reg = RefCell::new(Registry::new());
    let (doc, w) = (TargetKey::Node(t.dom.document()), TargetKey::Window(9));
    let (o, i) = (TargetKey::Node(t.outer), TargetKey::Node(t.inner));
    {
        let mut r = reg.borrow_mut();
        r.add(w, spec(1, "x", true));
        r.add(w, spec(2, "x", false));
        r.add(doc, spec(3, "x", false));
        r.add(o, spec(4, "x", true));
        r.add(o, spec(5, "x", false));
        r.add(i, spec(6, "x", false));
        r.add(i, spec(7, "x", true));
    }
    let mut rec = Rec::new();
    run(&reg, &t.dom, i, Some(9), &init("x", true), &mut rec);
    // Capture root-down, target capture then bubble listeners, bubble upward.
    assert_eq!(rec.tokens(), vec![1, 4, 7, 6, 5, 3, 2]);
    let phases: Vec<_> = rec.log.iter().map(|e| e.3).collect();
    assert_eq!(phases[2], Phase::AtTarget);
    assert_eq!(phases[3], Phase::AtTarget);
    assert_eq!(phases[4], Phase::Bubbling);
    // Window listeners see the last non-null target, the span.
    assert_eq!(rec.log[0].2, Some(i));
}

#[test]
fn non_bubbling_event_skips_bubble_phase_above_target() {
    let t = tree();
    let reg = RefCell::new(Registry::new());
    let (o, i) = (TargetKey::Node(t.outer), TargetKey::Node(t.inner));
    reg.borrow_mut().add(o, spec(1, "x", false));
    reg.borrow_mut().add(o, spec(2, "x", true));
    reg.borrow_mut().add(i, spec(3, "x", false));
    let mut rec = Rec::new();
    run(&reg, &t.dom, i, None, &init("x", false), &mut rec);
    assert_eq!(rec.tokens(), vec![2, 3]);
}

#[test]
fn stop_propagation_finishes_node_stop_immediate_does_not() {
    let t = tree();
    let reg = RefCell::new(Registry::new());
    let (o, i) = (TargetKey::Node(t.outer), TargetKey::Node(t.inner));
    for token in 1..=2 {
        reg.borrow_mut().add(i, spec(token, "x", false));
    }
    reg.borrow_mut().add(o, spec(3, "x", false));
    let mut rec = Rec::new();
    rec.hook = Box::new(|token, _| InvokeResult {
        stop_propagation: token == 1,
        ..Default::default()
    });
    run(&reg, &t.dom, i, None, &init("x", true), &mut rec);
    assert_eq!(rec.tokens(), vec![1, 2]);
    let mut rec = Rec::new();
    rec.hook = Box::new(|token, _| InvokeResult {
        stop_immediate: token == 1,
        ..Default::default()
    });
    run(&reg, &t.dom, i, None, &init("x", true), &mut rec);
    assert_eq!(rec.tokens(), vec![1]);
}

#[test]
fn preset_stop_flag_runs_nothing() {
    let t = tree();
    let reg = RefCell::new(Registry::new());
    let i = TargetKey::Node(t.inner);
    reg.borrow_mut().add(i, spec(1, "x", false));
    let mut ev = init("x", true);
    ev.stop_propagation = true;
    let mut rec = Rec::new();
    run(&reg, &t.dom, i, None, &ev, &mut rec);
    assert!(rec.log.is_empty());
}

#[test]
fn once_removes_before_call_and_removal_skips_pending() {
    let t = tree();
    let reg = Rc::new(RefCell::new(Registry::new()));
    let i = TargetKey::Node(t.inner);
    reg.borrow_mut().add(
        i,
        ListenerSpec {
            once: true,
            ..spec(1, "x", false)
        },
    );
    reg.borrow_mut().add(i, spec(2, "x", false));
    reg.borrow_mut().add(i, spec(3, "x", false));
    let mut rec = Rec::new();
    // Listener 2 removes listener 3 while the dispatch is in flight.
    rec.hook = Box::new({
        let reg = reg.clone();
        move |token, _| {
            if token == 2 {
                reg.borrow_mut().remove(i, Owner::SCRIPT, 3, "x", false);
            }
            InvokeResult::default()
        }
    });
    run(&reg, &t.dom, i, None, &init("x", true), &mut rec);
    assert_eq!(rec.tokens(), vec![1, 2]);
    let mut rec = Rec::new();
    run(&reg, &t.dom, i, None, &init("x", true), &mut rec);
    assert_eq!(rec.tokens(), vec![2], "the once listener and the removed one are gone");
}

#[test]
fn duplicate_listener_is_ignored_and_remove_reports() {
    let mut r = Registry::new();
    let t = TargetKey::Object(1);
    assert!(r.add(t, spec(1, "x", false)).is_some());
    assert!(r.add(t, spec(1, "x", false)).is_none());
    assert!(r.add(t, spec(1, "x", true)).is_some());
    assert!(r.has_type("x"));
    assert!(r.remove(t, Owner::SCRIPT, 1, "x", false));
    assert!(!r.remove(t, Owner::SCRIPT, 1, "x", false));
    assert!(r.token_in_use(t, Owner::SCRIPT, 1));
    r.remove(t, Owner::SCRIPT, 1, "x", true);
    assert!(!r.has_type("x"));
    assert!(!r.token_in_use(t, Owner::SCRIPT, 1));
}

fn shadow_tree(mode: ShadowRootMode) -> (ScriptedDom, NodeId, NodeId, NodeId) {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let host = el(&mut dom, "div");
    dom.append_child(root, host);
    let sr = dom
        .attach_shadow(
            host,
            ShadowRootInit {
                mode,
                ..Default::default()
            },
        )
        .unwrap();
    let inner = el(&mut dom, "span");
    dom.append_child(sr, inner);
    (dom, host, sr, inner)
}

#[test]
fn composed_event_retargets_to_host_outside_the_shadow_tree() {
    let (dom, host, sr, inner) = shadow_tree(ShadowRootMode::Open);
    let reg = RefCell::new(Registry::new());
    let (h, s, i) = (TargetKey::Node(host), TargetKey::Node(sr), TargetKey::Node(inner));
    {
        let mut r = reg.borrow_mut();
        r.add(i, spec(1, "x", false));
        r.add(s, spec(2, "x", false));
        r.add(h, spec(3, "x", false));
    }
    let mut rec = Rec::new();
    run(&reg, &dom, i, None, &init("x", true), &mut rec);
    assert_eq!(rec.tokens(), vec![1, 2, 3]);
    assert_eq!(rec.log[1].2, Some(i), "inside the tree the real target");
    assert_eq!(rec.log[2].2, Some(h), "outside, the host");
    assert_eq!(rec.log[2].3, Phase::AtTarget, "host is at-target for its own level");
}

#[test]
fn non_composed_event_stops_at_the_shadow_root() {
    let (dom, host, sr, inner) = shadow_tree(ShadowRootMode::Open);
    let reg = RefCell::new(Registry::new());
    reg.borrow_mut().add(TargetKey::Node(sr), spec(1, "x", false));
    reg.borrow_mut().add(TargetKey::Node(host), spec(2, "x", false));
    let path = build_path(&dom, TargetKey::Node(inner), None, "x", false);
    let mut rec = Rec::new();
    dispatch(&reg, path, &init("x", true), &mut rec);
    assert_eq!(rec.tokens(), vec![1]);
}

#[test]
fn composed_path_hides_closed_tree_from_outside() {
    let (dom, host, sr, inner) = shadow_tree(ShadowRootMode::Closed);
    let (h, s, i) = (TargetKey::Node(host), TargetKey::Node(sr), TargetKey::Node(inner));
    let doc = TargetKey::Node(dom.document());
    let path = build_path(&dom, i, Some(4), "x", true);
    let w = TargetKey::Window(4);
    assert_eq!(composed_path(&path.items, i), vec![i, s, h, doc, w]);
    assert_eq!(composed_path(&path.items, h), vec![h, doc, w]);
    assert_eq!(composed_path(&path.items, s), vec![i, s, h, doc, w]);
    assert!(!path.clear_targets, "the path exits through the host, whose root is the document");
    let sealed = build_path(&dom, i, Some(4), "x", false);
    assert!(sealed.clear_targets, "a path ending inside the shadow tree nulls event.target afterwards");
}

#[test]
fn assigned_node_path_goes_through_the_slot() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    let host = el(&mut dom, "div");
    dom.append_child(root, host);
    let light = el(&mut dom, "span");
    dom.append_child(host, light);
    let sr = dom.attach_shadow(host, ShadowRootInit::default()).unwrap();
    let slot = el(&mut dom, "slot");
    dom.append_child(sr, slot);
    assert_eq!(dom.assigned_slot_of(light), Some(slot));
    let path = build_path(&dom, TargetKey::Node(light), None, "x", true);
    let order: Vec<_> = path.items.iter().map(|p| p.target).collect();
    assert_eq!(
        order[..4],
        [
            TargetKey::Node(light),
            TargetKey::Node(slot),
            TargetKey::Node(sr),
            TargetKey::Node(host)
        ]
    );
}

#[test]
fn non_node_target_has_a_single_item_path() {
    let dom = ScriptedDom::new();
    let reg = RefCell::new(Registry::new());
    let t = TargetKey::Object(5);
    reg.borrow_mut().add(t, spec(1, "x", false));
    reg.borrow_mut().add(t, spec(2, "x", true));
    let mut rec = Rec::new();
    run(&reg, &dom, t, None, &init("x", true), &mut rec);
    assert_eq!(rec.tokens(), vec![2, 1]);
}

#[test]
fn document_does_not_reach_window_for_load() {
    let t = tree();
    let path = build_path(&t.dom, TargetKey::Node(t.inner), Some(1), "load", true);
    assert!(!path.items.iter().any(|p| p.target == TargetKey::Window(1)));
}
