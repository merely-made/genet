// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn qualified_frame_can_be_collected_and_published_after_deadline() {
    let core = core();
    let frame = core.try_begin_native_frame().unwrap();
    let (staging, _) = draw(&frame, &scene());
    let mut pending = frame.seal(staging);
    let mut completion = pending
        .completed
        .take()
        .unwrap()
        .recv_timeout(Duration::from_secs(6))
        .unwrap();
    assert!(
        completion.result.is_ok(),
        "worker must qualify before the deadline"
    );
    completion.qualification.as_mut().unwrap().deadline = Instant::now() - Duration::from_secs(1);
    let (sender, receiver) = mpsc::channel();
    assert!(sender.send(completion).is_ok());
    pending.completed = Some(receiver);
    let mut accepted = pending.poll().expect("completed frame").unwrap();
    accepted.qualification.as_mut().unwrap().deadline = Instant::now() - Duration::from_secs(1);
    let texture = accepted.publish(|texture| texture).unwrap();
    assert!(!pixels(&core, &texture).is_blank());
    core.close_native_frames().unwrap();
}
