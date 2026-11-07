// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::sync::atomic::AtomicUsize;

fn busy() -> Arc<NativeFrames> {
    let frames = Arc::new(NativeFrames::default());
    frames.state.lock().unwrap().phase = State::Pending;
    frames
}
fn count(calls: &Arc<AtomicUsize>) -> Wake {
    let calls = calls.clone();
    Arc::new(move || {
        calls.fetch_add(1, Ordering::SeqCst);
    })
}

#[test]
fn idle_and_terminal_unavailability_notify_immediately_outside_the_lock() {
    for terminal in [false, true] {
        let frames = Arc::new(NativeFrames::default());
        if terminal {
            frames.finish(false);
        }
        let outside = Arc::new(AtomicBool::new(false));
        let observed = outside.clone();
        let weak = Arc::downgrade(&frames);
        let waiter = frames.notify_available(Arc::new(move || {
            observed.store(
                weak.upgrade().unwrap().state.try_lock().is_ok(),
                Ordering::SeqCst,
            );
        }));
        assert!(waiter.ready());
        assert!(outside.load(Ordering::SeqCst));
        assert!(frames.state.lock().unwrap().waiters.is_empty());
    }
}

#[test]
fn finish_marks_all_waiters_before_notifying_each_once() {
    let frames = busy();
    let calls = Arc::new(AtomicUsize::new(0));
    let signals: Arc<Mutex<Vec<Weak<Signal>>>> = Arc::new(Mutex::new(Vec::new()));
    let all_ready = Arc::new(AtomicBool::new(true));
    let make = || {
        let (calls, signals, ready) = (calls.clone(), signals.clone(), all_ready.clone());
        frames.notify_available(Arc::new(move || {
            let every = signals
                .lock()
                .unwrap()
                .iter()
                .filter_map(Weak::upgrade)
                .all(|signal| signal.ready.load(Ordering::Acquire));
            ready.fetch_and(every, Ordering::SeqCst);
            calls.fetch_add(1, Ordering::SeqCst);
        }))
    };
    let first = make();
    signals.lock().unwrap().push(Arc::downgrade(&first.signal));
    let second = make();
    signals.lock().unwrap().push(Arc::downgrade(&second.signal));
    assert!(!first.ready() && !second.ready());
    frames.finish(true);
    assert!(first.ready() && second.ready());
    assert!(all_ready.load(Ordering::SeqCst));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    frames.finish(false);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn dropping_a_busy_waiter_unregisters_and_never_retains_the_gate() {
    let frames = busy();
    let owner = Arc::downgrade(&frames.state);
    let calls = Arc::new(AtomicUsize::new(0));
    let waiter = frames.notify_available(count(&calls));
    assert_eq!(frames.state.lock().unwrap().waiters.len(), 1);
    drop(waiter);
    assert!(frames.state.lock().unwrap().waiters.is_empty());
    frames.finish(true);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let retained = frames.notify_available(count(&calls));
    drop(frames);
    assert!(owner.upgrade().is_none());
    assert!(retained.ready());
}

#[test]
fn retirement_before_registration_and_legacy_retirement_both_signal() {
    let frames = busy();
    let calls = Arc::new(AtomicUsize::new(0));
    frames.finish(true); // Busy was observed before registration could lock.
    let after_retirement = frames.notify_available(count(&calls));
    assert!(after_retirement.ready());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let legacy = frames.checked().unwrap();
    let while_legacy = frames.notify_available(count(&calls));
    assert!(!while_legacy.ready());
    drop(legacy);
    assert!(while_legacy.ready());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn notification_can_reenter_registration_and_one_panicking_wake_does_not_starve_others() {
    let frames = busy();
    let panicking = frames.notify_available(Arc::new(|| panic!("notification isolation control")));
    let calls = Arc::new(AtomicUsize::new(0));
    let weak = Arc::downgrade(&frames);
    let notified = calls.clone();
    let waiter = frames.notify_available(Arc::new(move || {
        let frames = weak.upgrade().unwrap();
        // Immediate reentrant notification would deadlock if finish retained
        // its state lock while invoking the first notification.
        let nested = frames.notify_available(count(&notified));
        assert!(nested.ready());
    }));
    frames.finish(true);
    assert!(panicking.ready() && waiter.ready());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn concurrent_retirement_and_registration_do_not_lose_the_edge() {
    for _ in 0..32 {
        let frames = busy();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let calls = Arc::new(AtomicUsize::new(0));
        let retiring = frames.clone();
        let go = barrier.clone();
        let thread = std::thread::spawn(move || {
            go.wait();
            retiring.finish(true);
        });
        barrier.wait();
        let waiter = frames.notify_available(count(&calls));
        thread.join().unwrap();
        assert!(waiter.ready());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}

#[test]
fn registrations_are_send_sync_and_ids_do_not_recur_after_drop() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<NativeFrameWaiter>();
    let frames = busy();
    let first = frames.notify_available(Arc::new(|| {}));
    let old = first.id.unwrap();
    drop(first);
    let second = frames.notify_available(Arc::new(|| {}));
    assert!(second.id.unwrap() > old);
}

#[test]
fn shutdown_notifies_pending_waiters_outside_the_gate_and_stays_closed() {
    let frames = busy();
    let calls = Arc::new(AtomicUsize::new(0));
    let outside = Arc::new(AtomicBool::new(false));
    let (observed, notified, weak) = (outside.clone(), calls.clone(), Arc::downgrade(&frames));
    let waiter = frames.notify_available(Arc::new(move || {
        observed.store(
            weak.upgrade().unwrap().state.try_lock().is_ok(),
            Ordering::SeqCst,
        );
        notified.fetch_add(1, Ordering::SeqCst);
    }));
    assert!(!waiter.ready());
    frames.close().unwrap();
    assert!(waiter.ready());
    assert!(outside.load(Ordering::SeqCst));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    frames.finish(true);
    assert!(matches!(frames.state.lock().unwrap().phase, State::Closed));
    assert!(
        frames.checked().is_err(),
        "a late completion cannot reopen shutdown admission"
    );
    let after = frames.notify_available(count(&calls));
    assert!(
        after.ready(),
        "terminal closure never registers an unwakeable waiter"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn closing_publication_wakes_waiters_after_retiring_the_publisher() {
    let frames = busy();
    let publication = frames.begin_publication().unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let waiter = frames.notify_available(count(&calls));
    frames.state.lock().unwrap().phase = State::Publishing(std::thread::current().id(), true);
    assert!(!waiter.ready());
    drop(publication);
    assert!(waiter.ready());
    assert!(matches!(frames.state.lock().unwrap().phase, State::Closed));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    frames.finish(false);
    assert!(matches!(frames.state.lock().unwrap().phase, State::Closed));
}

#[test]
#[ignore = "actual GPU; serialized Core availability/admission integration"]
fn accepted_drop_and_checked_legacy_retirement_wake_actual_busy_gate_waiters() {
    let core = Arc::new(
        RenderCore::boot(crate::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(16),
            ..Default::default()
        })
        .expect("actual native adapter required"),
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let frame = core.try_begin_native_frame().unwrap();
    let waiter = core.notify_native_frame_available(count(&calls));
    assert!(!waiter.ready());
    let mut pending = frame.seal(());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(6);
    let accepted = loop {
        if let Some(accepted) = pending.poll() {
            break accepted.unwrap();
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(1));
    };
    assert!(
        !waiter.ready(),
        "acceptance still owns the publication lease"
    );
    drop(accepted);
    assert!(waiter.ready());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let legacy = core
        .try_reuse_native_raster(|| {
            let waiter = core.notify_native_frame_available(count(&calls));
            assert!(!waiter.ready());
            waiter
        })
        .unwrap();
    assert!(legacy.ready());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert!(!core.native_frame_busy());
    let retired = Arc::downgrade(&core);
    drop(core);
    assert!(
        retired.upgrade().is_none(),
        "live waiter tokens cannot retain the Core"
    );
}
