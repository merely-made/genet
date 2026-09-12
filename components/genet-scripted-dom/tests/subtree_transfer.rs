/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Storage-only adoption prerequisite. These are not JS adoption receipts.
use genet_scripted_dom::{
    NodeId, NodeIdentityError, Pins, ScriptedDom, ShadowRootInit, ShadowRootMode, SlotAssignmentMode,
    SubtreeTransferError,
};
use layout_dom_api::{DomMutation, LayoutDom, LayoutDomMut, LocalName, Namespace, QualName};

fn qual(name: &str) -> QualName {
    QualName::new(
        None,
        Namespace::from("http://www.w3.org/1999/xhtml"),
        LocalName::from(name),
    )
}

fn drain(dom: &mut ScriptedDom) {
    dom.drain_mutations(&mut Vec::new());
    dom.take_observed();
}

#[test]
fn transfer_preserves_identity_readback_and_destination_mutation() {
    let mut source = ScriptedDom::new();
    let mut target = ScriptedDom::new();
    let node = source.create_element(qual("p"));
    let text = source.create_text("before");
    source.append_child(node, text);
    let target_control = target.create_element(qual("aside"));
    target.append_child(target.document(), target_control);
    drain(&mut source);
    drain(&mut target);
    let ids = source
        .transfer_detached_subtree_to(&mut target, node)
        .unwrap();
    assert_eq!(ids, [node, text]);
    assert!(!source.is_live(node) && !source.is_live(text));
    assert_eq!(target.parent(text), Some(node));
    assert_eq!(target.text(text), Some("before"));
    assert_eq!(
        target.dom_children(target.document()).collect::<Vec<_>>(),
        [target_control]
    );
    target.append_child(target.document(), node);
    target.set_text_content(text, "after");
    assert_eq!(target.text(text), Some("after"));
    let mut mutations = Vec::new();
    target.drain_mutations(&mut mutations);
    assert!(matches!(mutations[0], DomMutation::Inserted { node: id, .. } if id == node));
    assert!(
        mutations
            .iter()
            .any(|m| matches!(m, DomMutation::CharacterDataChanged { node } if *node == text))
    );
}

#[test]
fn transfer_round_trip_and_explicit_pin_relocation_then_release() {
    let mut a = ScriptedDom::new();
    let mut b = ScriptedDom::new();
    let node = a.create_element(qual("p"));
    let text = a.create_text("retained");
    let captured = a.capture_node_id(node);
    a.append_child(node, text);
    drain(&mut a);
    let mut pins_a = Pins::default();
    let mut pins_b = Pins::default();
    pins_a.pin(text);
    a.collect(pins_a.iter());
    let moved = a.transfer_detached_subtree_to(&mut b, node).unwrap();
    assert_eq!(
        b.try_capture_node_id(node),
        Err(NodeIdentityError::CaptureRequiresTranslation)
    );
    for id in moved {
        if pins_a.unpin(id) {
            pins_b.pin(id);
        }
    }
    a.collect(pins_a.iter());
    b.collect(pins_b.iter());
    assert_eq!(b.text(text), Some("retained"));
    assert!(b.is_live(node));
    let moved = b.transfer_detached_subtree_to(&mut a, node).unwrap();
    assert_eq!(moved, [node, text]);
    assert_eq!(a.try_capture_node_id(node), Ok(captured));
    for id in moved {
        if pins_b.unpin(id) {
            pins_a.pin(id);
        }
    }
    a.collect(pins_a.iter());
    assert!(a.is_live(node));
    pins_a.clear();
    assert_eq!(a.collect(pins_a.iter()), 2);
    assert!(!a.is_live(node) && !b.is_live(node));
}

fn assert_refused(
    source: &mut ScriptedDom,
    target: &mut ScriptedDom,
    node: NodeId,
    error: SubtreeTransferError,
) {
    let before = (
        source.live_node_count(),
        target.live_node_count(),
        source.structure_epoch(),
        target.structure_epoch(),
        source.pending_mutations().1.len(),
        target.pending_mutations().1.len(),
    );
    assert_eq!(
        source.transfer_detached_subtree_to(target, node),
        Err(error)
    );
    assert_eq!(
        before,
        (
            source.live_node_count(),
            target.live_node_count(),
            source.structure_epoch(),
            target.structure_epoch(),
            source.pending_mutations().1.len(),
            target.pending_mutations().1.len()
        )
    );
}

#[test]
fn wrong_store_dead_document_and_attached_roots_are_refused_atomically() {
    let mut a = ScriptedDom::new();
    let mut b = ScriptedDom::new();
    let foreign = b.document();
    assert_refused(&mut a, &mut b, foreign, SubtreeTransferError::MissingNode);
    let root = a.document();
    assert_refused(&mut a, &mut b, root, SubtreeTransferError::UnsupportedNode);
    let node = a.create_element(qual("p"));
    a.append_child(root, node);
    drain(&mut a);
    assert_refused(&mut a, &mut b, node, SubtreeTransferError::AttachedRoot);
    a.remove_child(node);
    drain(&mut a);
    a.collect([]);
    assert_refused(&mut a, &mut b, node, SubtreeTransferError::MissingNode);
}

#[test]
fn queued_mutations_and_observer_records_must_be_consumed_first() {
    let mut a = ScriptedDom::new();
    let mut b = ScriptedDom::new();
    a.set_observing(true);
    let node = a.create_element(qual("p"));
    a.append_child(a.document(), node);
    a.remove_child(node);
    assert_refused(
        &mut a,
        &mut b,
        node,
        SubtreeTransferError::PendingSourceWork,
    );
    a.drain_mutations(&mut Vec::new());
    assert_refused(
        &mut a,
        &mut b,
        node,
        SubtreeTransferError::PendingSourceWork,
    );
    assert_eq!(a.take_observed().len(), 2);
    assert_eq!(
        a.transfer_detached_subtree_to(&mut b, node).unwrap(),
        [node]
    );
}

#[test]
fn a_parse_refuses_only_what_the_tree_builder_still_holds() {
    let mut a = ScriptedDom::new();
    let mut b = ScriptedDom::new();
    // A completed sibling subtree and the element the parser is inside of.
    let open = a.create_element(qual("section"));
    a.append_child(a.document(), open);
    let done = a.create_element(qual("p"));
    a.append_child(a.document(), done);
    a.remove_child(done);
    drain(&mut a);
    a.set_parsing(true);
    a.set_parser_guard(vec![open, done], Some(open), None);
    // `done` was created and never popped, but it is not an inclusive ancestor
    // of the current node, so it is not on the stack of open elements.
    assert!(a.parser_holds(open));
    assert!(!a.parser_holds(done));
    // `open` is still attached, so the read-only preflight is what names the
    // parser as the obstacle; the detached transfer would stop at its parent.
    assert_eq!(
        a.preflight_subtree_transfer_to(&b, open),
        Err(SubtreeTransferError::PendingSourceWork)
    );
    assert_eq!(a.transfer_detached_subtree_to(&mut b, done).unwrap(), [done]);
    assert!(b.is_live(done));

    // The form element pointer protects its element wherever it sits.
    let form = a.create_element(qual("form"));
    a.set_parser_guard(vec![open], Some(open), Some(form));
    assert_refused(
        &mut a,
        &mut b,
        form,
        SubtreeTransferError::PendingSourceWork,
    );
    a.set_parsing(false);
    assert!(!a.parser_holds(form));
    assert!(a.transfer_detached_subtree_to(&mut b, form).is_ok());
}

#[test]
fn an_observer_group_refuses_even_with_empty_queues() {
    let mut a = ScriptedDom::new();
    let mut b = ScriptedDom::new();
    let node = a.create_element(qual("p"));
    a.set_observing(true);
    a.set_observer_group(true);
    assert_refused(
        &mut a,
        &mut b,
        node,
        SubtreeTransferError::PendingSourceWork,
    );
    a.set_observer_group(false);
    assert!(a.transfer_detached_subtree_to(&mut b, node).is_ok());
}

#[test]
fn a_shadow_tree_and_its_nested_hosts_travel_with_the_host() {
    let mut a = ScriptedDom::new();
    let mut b = ScriptedDom::new();
    let host = a.create_element(qual("div"));
    let root = a
        .attach_shadow(
            host,
            ShadowRootInit {
                mode: ShadowRootMode::Closed,
                ..ShadowRootInit::default()
            },
        )
        .unwrap();
    let slot = a.create_element(qual("slot"));
    a.append_child(root, slot);
    let inner_host = a.create_element(qual("span"));
    a.append_child(root, inner_host);
    let inner_root = a
        .attach_shadow(inner_host, ShadowRootInit::default())
        .unwrap();
    let light = a.create_element(qual("p"));
    a.append_child(host, light);
    drain(&mut a);

    let members = a.transfer_detached_subtree_to(&mut b, host).unwrap();
    for id in [host, root, slot, inner_host, inner_root, light] {
        assert!(members.contains(&id), "{id:?} did not travel");
        assert!(b.is_live(id) && !a.is_live(id));
    }
    // The shadow root's side maps came with it, closed stays closed, and the
    // source keeps none of the bookkeeping.
    assert_eq!(b.shadow_root_of(host), Some(root));
    assert_eq!(b.shadow_host_of(root), Some(host));
    assert_eq!(b.shadow_root_of(inner_host), Some(inner_root));
    assert_eq!(
        b.shadow_init(root).map(|init| init.mode),
        Some(ShadowRootMode::Closed)
    );
    assert_eq!(a.shadow_root_count(), 0);
    assert_eq!(b.shadow_root_count(), 2);
    assert_eq!(b.assigned_nodes_of(slot), vec![light]);
    assert_eq!(b.assigned_slot_of(light), Some(slot));
    assert_eq!(b.tree_root(inner_root), Some(host));
}

#[test]
fn a_shadow_root_is_never_the_thing_adopted_and_a_split_host_is_refused() {
    let mut a = ScriptedDom::new();
    let mut b = ScriptedDom::new();
    let host = a.create_element(qual("div"));
    let root = a.attach_shadow(host, ShadowRootInit::default()).unwrap();
    let inside = a.create_element(qual("p"));
    a.append_child(root, inside);
    drain(&mut a);
    // A shadow root has no parent and is not an adoptable node of its own.
    assert_refused(&mut a, &mut b, root, SubtreeTransferError::UnsupportedNode);
    // Neither is a node that would leave its containing shadow tree behind:
    // `inside` is attached, so the runtime removes it first; here the storage
    // boundary sees an attached root.
    assert_refused(&mut a, &mut b, inside, SubtreeTransferError::AttachedRoot);
    assert_eq!(a.shadow_root_of(host), Some(root));
}

#[test]
fn template_contents_are_rehomed_to_the_destination_inert_document() {
    let mut a = ScriptedDom::new();
    let mut b = ScriptedDom::new();
    let template = a.create_element(qual("template"));
    let contents = a.ensure_template_contents(template);
    let nested = a.create_element(qual("template"));
    a.append_child(contents, nested);
    let nested_contents = a.ensure_template_contents(nested);
    let source_owner = a.template_owner_document_if_any().unwrap();
    // Something else already owns an inert document in the destination, so the
    // re-homing must land on *that* one rather than minting a second.
    let resident = b.create_element(qual("template"));
    let resident_contents = b.ensure_template_contents(resident);
    let destination_owner = b.template_owner_document_if_any().unwrap();
    drain(&mut a);

    let members = a.transfer_detached_subtree_to(&mut b, template).unwrap();
    for id in [template, contents, nested, nested_contents] {
        assert!(members.contains(&id));
    }
    assert!(!members.contains(&source_owner));
    assert_eq!(b.template_contents_of(template), Some(contents));
    assert_eq!(b.template_contents_of(nested), Some(nested_contents));
    assert!(b.is_in_template_contents(nested));
    assert_eq!(b.template_owner_document_if_any(), Some(destination_owner));
    assert_eq!(b.template_contents_of(resident), Some(resident_contents));
    assert_eq!(a.template_contents_of(template), None);
    // A contents fragment whose template stays behind is still refused.
    let orphan = b.create_element(qual("template"));
    let orphan_contents = b.ensure_template_contents(orphan);
    drain(&mut b);
    let mut c = ScriptedDom::new();
    assert_eq!(
        b.preflight_subtree_transfer_to(&c, orphan_contents),
        Err(SubtreeTransferError::AssociatedState)
    );
    assert!(b.transfer_detached_subtree_to(&mut c, orphan).is_ok());
}

#[test]
fn transfer_churn_keeps_live_store_bounded_and_never_reuses_ids() {
    let mut a = ScriptedDom::new();
    let mut b = ScriptedDom::new();
    let mut retired = Vec::new();
    for _ in 0..128 {
        let node = a.create_element(qual("p"));
        assert!(!retired.contains(&node));
        a.transfer_detached_subtree_to(&mut b, node).unwrap();
        assert_eq!(b.collect([]), 1);
        retired.push(node);
        assert_eq!((a.live_node_count(), b.live_node_count()), (1, 1));
    }
    assert!(retired.iter().all(|id| !a.is_live(*id) && !b.is_live(*id)));
}

#[test]
fn detached_manual_slot_request_is_still_associated_state() {
    let mut a = ScriptedDom::new();
    let mut b = ScriptedDom::new();
    let host = a.create_element(qual("div"));
    let root = a
        .attach_shadow(
            host,
            ShadowRootInit {
                slot_assignment: SlotAssignmentMode::Manual,
                ..ShadowRootInit::default()
            },
        )
        .unwrap();
    let slot = a.create_element(qual("slot"));
    a.append_child(root, slot);
    let node = a.create_element(qual("p"));
    a.set_manual_assignment(slot, vec![node]);
    assert_eq!(a.assigned_slot_of(node), None);
    drain(&mut a);
    assert_refused(&mut a, &mut b, node, SubtreeTransferError::AssociatedState);
    assert!(a.is_live(node));
}

#[test]
fn preflight_then_removal_preserves_layout_queue_until_its_consumer_drains() {
    let mut source = ScriptedDom::new();
    let mut target = ScriptedDom::new();
    let control = source.create_element(qual("aside"));
    source.append_child(source.document(), control);
    drain(&mut source); // Give the retained layout queue a nonzero base.
    source.set_observing(true);
    let node = source.create_element(qual("p"));
    let text = source.create_text("retained");
    source.append_child(node, text);
    source.append_child(source.document(), node);
    let snapshot = format!("{:?}", source.pending_mutations());
    let epochs = (source.structure_epoch(), target.structure_epoch());
    assert_eq!(
        source.preflight_subtree_transfer_to(&target, node).unwrap(),
        [node, text]
    );
    assert_eq!(format!("{:?}", source.pending_mutations()), snapshot);
    assert_eq!(epochs, (source.structure_epoch(), target.structure_epoch()));
    assert_eq!(source.parent(node), Some(source.document()));
    assert_eq!(
        source.transfer_detached_subtree_preserving_mutations_to(&mut target, node),
        Err(SubtreeTransferError::AttachedRoot)
    );
    assert_eq!(format!("{:?}", source.pending_mutations()), snapshot);

    source.remove_child(node);
    let snapshot = format!("{:?}", source.pending_mutations());
    assert_eq!(
        source.transfer_detached_subtree_preserving_mutations_to(&mut target, node),
        Err(SubtreeTransferError::PendingSourceWork)
    );
    assert_eq!(format!("{:?}", source.pending_mutations()), snapshot);
    assert!(source.is_live(node) && !target.is_live(node));
    assert!(!source.take_observed().is_empty());
    let base = source.pending_mutations().0;
    let count = source.pending_mutations().1.len();
    assert!(base > 0 && count > 0);
    assert_eq!(
        source
            .transfer_detached_subtree_preserving_mutations_to(&mut target, node)
            .unwrap(),
        [node, text]
    );
    assert_eq!(format!("{:?}", source.pending_mutations()), snapshot);
    assert!(!source.is_live(node) && target.is_live(node));
    assert_eq!(target.text(text), Some("retained"));
    let mut layout = Vec::new();
    source.drain_mutations(&mut layout);
    assert_eq!(layout.len(), count);
    assert_eq!(source.pending_mutations().0, base + count as u64);
    assert!(
        layout
            .iter()
            .any(|entry| matches!(entry, DomMutation::Removed { node: id, .. } if *id == node))
    );
    assert_eq!(
        source.dom_children(source.document()).collect::<Vec<_>>(),
        [control]
    );
}

#[test]
fn unsupported_preflight_does_not_detach_or_consume_pending_work() {
    let mut source = ScriptedDom::new();
    let target = ScriptedDom::new();
    source.set_observing(true);
    let host = source.create_element(qual("div"));
    source.append_child(source.document(), host);
    let root = source
        .attach_shadow(host, ShadowRootInit::default())
        .unwrap();
    let queue = format!("{:?}", source.pending_mutations());
    let epoch = source.structure_epoch();
    let count = source.live_node_count();
    // A shadow root travels with its host; it is never adopted on its own.
    assert_eq!(
        source.preflight_subtree_transfer_to(&target, root),
        Err(SubtreeTransferError::UnsupportedNode)
    );
    assert_eq!(source.parent(host), Some(source.document()));
    assert_eq!(format!("{:?}", source.pending_mutations()), queue);
    assert_eq!(source.structure_epoch(), epoch);
    assert_eq!(source.live_node_count(), count);
    assert!(!source.take_observed().is_empty());
    assert_eq!(target.live_node_count(), 1);
}
