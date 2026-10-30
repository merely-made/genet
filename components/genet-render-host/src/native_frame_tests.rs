// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{ColorLoad, NetrenderOptions, RgbaFrame, Scene};
use std::sync::atomic::{AtomicUsize, Ordering};

fn core() -> Arc<RenderCore> {
    Arc::new(
        RenderCore::boot(NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(32),
            ..Default::default()
        })
        .expect("actual native adapter required"),
    )
}
fn scene() -> Scene {
    let mut scene = Scene::new(32, 24);
    scene.push_rect(0., 0., 32., 24., [0.25, 0.5, 0.75, 1.]);
    scene
}
fn draw(frame: &NativeFrameSubmission, scene: &Scene) -> (wgpu::Texture, wgpu::TextureView) {
    frame
        .rasterize(
            101,
            scene,
            32,
            24,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.,
        )
        .unwrap()
}
fn collect<T>(
    pending: &mut PendingNativeFrame<T>,
) -> Result<AcceptedNativeFrame<T>, RasterizeFailure> {
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        if let Some(result) = pending.poll() {
            return result;
        }
        assert!(Instant::now() < deadline, "native batch did not complete");
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn pixels(core: &RenderCore, texture: &wgpu::Texture) -> RgbaFrame {
    let mut pending = core.start_rgba8_readback(texture, 32, 24).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(result) = pending.poll() {
            return result.unwrap();
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn actual_loss(core: &RenderCore) {
    core.device().destroy();
    let deadline = Instant::now() + Duration::from_secs(5);
    while core.device_failure().is_none() {
        assert!(
            Instant::now() < deadline,
            "actual device loss not delivered"
        );
        let _ = core.device().poll(wgpu::PollType::Poll);
        std::thread::yield_now();
    }
}
fn begin_after_retirement(core: &Arc<RenderCore>) -> NativeFrameSubmission {
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        match core.try_begin_native_frame() {
            Ok(frame) => return frame,
            Err(NativeFrameStartError::Busy) => {},
            Err(NativeFrameStartError::Refused(error)) => panic!("retirement failed: {error}"),
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_stages_before_quality_completion_and_publishes_exact_pixels() {
    let core = core();
    let frame = core.try_begin_native_frame().unwrap();
    let (texture, _) = draw(&frame, &scene());
    assert!(
        core.renderer.vello_curve_quality_status().unwrap().0 > 0,
        "submission drained current quality counters on the submitting thread"
    );
    let mut pending = frame.seal(texture);
    assert!(matches!(
        core.try_begin_native_frame(),
        Err(NativeFrameStartError::Busy)
    ));
    let texture = collect(&mut pending)
        .unwrap()
        .publish(|texture| texture)
        .unwrap();
    assert!(pending.poll().is_none());
    assert_eq!(core.renderer.vello_curve_quality_status(), Some((0, 0)));
    let image = pixels(&core, &texture);
    assert!(!image.is_blank());
    assert!(
        image
            .rgba
            .chunks_exact(4)
            .any(|p| p[2] > p[1] && p[1] > p[0] && p[3] == 255)
    );
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_current_shader_refusal_is_not_published_and_healthy_core_recovers() {
    let core = core();
    let generation = core.generation();
    let mut path = netrender::scene::ScenePath::new();
    path.move_to(8., 12.).line_to(24., 12.);
    let mut bad = Scene::new(32, 24);
    bad.push_shape_stroked(path, [0.25, 0.5, 0.75, 1.], 40_000_000.);
    let netrender::scene::SceneOp::Shape(shape) = &mut bad.ops[0] else {
        unreachable!()
    };
    shape.stroke.as_mut().unwrap().cap = netrender::scene::SceneStrokeCap::Round;
    let frame = core.try_begin_native_frame().unwrap();
    let staging = draw(&frame, &bad);
    let mut pending = frame.seal(staging);
    match collect(&mut pending) {
        Err(RasterizeFailure::ProviderError(error)) => assert!(matches!(
            &*error,
            netrender::VelloError::CurveFlatteningLimit
        )),
        _ => panic!("bad current shader frame escaped qualification"),
    }
    assert!(core.device_failure().is_none());
    assert_eq!(core.generation(), generation);
    assert_eq!(core.renderer.vello_curve_quality_status(), Some((0, 0)));
    let frame = core.try_begin_native_frame().unwrap();
    let (texture, _) = draw(&frame, &scene());
    let mut pending = frame.seal(texture);
    let texture = collect(&mut pending)
        .unwrap()
        .publish(|value| value)
        .unwrap();
    assert!(!pixels(&core, &texture).is_blank());
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_unpolled_loss_releases_admission_and_refuses_collection() {
    let core = core();
    let frame = core.try_begin_native_frame().unwrap();
    let staging = draw(&frame, &scene());
    let mut pending = frame.seal(staging);
    actual_loss(&core);
    let deadline = Instant::now() + Duration::from_secs(5);
    while core.loss_notice().is_none() {
        assert!(
            Instant::now() < deadline,
            "unpolled pending frame held recovery admission"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(matches!(
        pending.poll(),
        Some(Err(RasterizeFailure::DeviceLost(_)))
    ));
    assert!(pending.poll().is_none());
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_loss_after_acceptance_refuses_final_publication() {
    let core = core();
    let frame = core.try_begin_native_frame().unwrap();
    let staging = draw(&frame, &scene());
    let mut pending = frame.seal(staging);
    let accepted = collect(&mut pending).unwrap();
    actual_loss(&core);
    let published = AtomicUsize::new(0);
    assert!(matches!(
        accepted.publish(|_| published.fetch_add(1, Ordering::SeqCst)),
        Err(RasterizeFailure::DeviceLost(_))
    ));
    assert_eq!(published.load(Ordering::SeqCst), 0);
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_cancel_drops_staging_and_retires_before_next_batch() {
    struct Payload(Arc<AtomicUsize>);
    impl Drop for Payload {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let core = core();
    let frame = core.try_begin_native_frame().unwrap();
    let staging = draw(&frame, &scene());
    let drops = Arc::new(AtomicUsize::new(0));
    drop(frame.seal((staging, Payload(drops.clone()))));
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let next = begin_after_retirement(&core);
    let staging = draw(&next, &scene());
    let mut pending = next.seal((staging, 42));
    assert_eq!(
        collect(&mut pending)
            .unwrap()
            .publish(|(_, value)| value)
            .unwrap(),
        42
    );
    assert_eq!(core.renderer.vello_curve_quality_status(), Some((0, 0)));
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_pending_does_not_retain_core() {
    let core = core();
    let weak = Arc::downgrade(&core);
    let frame = core.try_begin_native_frame().unwrap();
    let staging = draw(&frame, &scene());
    let mut pending = frame.seal(staging);
    drop(core);
    let deadline = Instant::now() + Duration::from_secs(1);
    while weak.upgrade().is_some() {
        assert!(
            Instant::now() < deadline,
            "completion worker retained retired Core"
        );
        std::thread::yield_now();
    }
    assert!(matches!(
        pending.poll(),
        Some(Err(RasterizeFailure::ProviderError(_)))
    ));
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_scopes_capture_real_validation_error_and_allow_next_valid_batch() {
    let core = core();
    let frame = core.try_begin_native_frame().unwrap();
    let _invalid = core.device().create_buffer(&wgpu::BufferDescriptor {
        label: Some("native-batch-invalid-usage"),
        size: 16,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let mut pending = frame.seal(());
    match collect(&mut pending) {
        Err(RasterizeFailure::ProviderError(error)) => assert!(matches!(
            &*error,
            netrender::VelloError::WgpuErrorFromScope(wgpu::Error::Validation { .. })
        )),
        _ => panic!("actual validation error escaped native frame scopes"),
    }
    let frame = core.try_begin_native_frame().unwrap();
    let staging = draw(&frame, &scene());
    collect(&mut frame.seal(staging))
        .unwrap()
        .publish(|_| ())
        .unwrap();
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_expiry_refuses_and_actual_loss_wins_over_deadline() {
    let first = core();
    let mut frame = first.try_begin_native_frame().unwrap();
    frame.qualification.as_mut().unwrap().deadline = Instant::now() - Duration::from_secs(1);
    actual_loss(&first);
    assert!(matches!(
        collect(&mut frame.seal(())),
        Err(RasterizeFailure::DeviceLost(_))
    ));
    let second = core();
    let mut frame = second.try_begin_native_frame().unwrap();
    frame.qualification.as_mut().unwrap().deadline = Instant::now() - Duration::from_secs(1);
    assert!(matches!(
        collect(&mut frame.seal(())),
        Err(RasterizeFailure::ProviderError(_))
    ));
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_excludes_other_submitters_and_legacy_checked_rasters() {
    let core = core();
    let frame = core.try_begin_native_frame().unwrap();
    assert!(
        core.try_rasterize_scaled_for(
            1,
            &scene(),
            32,
            24,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.
        )
        .is_err()
    );
    let other = core.clone();
    assert!(
        std::thread::spawn(move || matches!(
            other.try_begin_native_frame(),
            Err(NativeFrameStartError::Busy)
        ))
        .join()
        .unwrap()
    );
    drop(frame);
    let next = begin_after_retirement(&core);
    let staging = draw(&next, &scene());
    collect(&mut next.seal(staging))
        .unwrap()
        .publish(|_| ())
        .unwrap();
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_completion_wakes_from_worker_and_collection_is_channel_only() {
    let core = core();
    let frame = core.try_begin_native_frame().unwrap();
    let staging = draw(&frame, &scene());
    let ui = std::thread::current().id();
    let (sender, receiver) = mpsc::channel();
    let mut pending = frame.seal_with_wake(
        staging,
        Arc::new(move || {
            let _ = sender.send(std::thread::current().id());
        }),
    );
    let completed_on = receiver.recv_timeout(Duration::from_secs(6)).unwrap();
    assert_ne!(completed_on, ui);
    pending
        .poll()
        .expect("wake preceded completion delivery")
        .unwrap()
        .publish(|_| ())
        .unwrap();
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_loss_after_worker_completion_refuses_collection() {
    let core = core();
    let frame = core.try_begin_native_frame().unwrap();
    let staging = draw(&frame, &scene());
    let (sender, receiver) = mpsc::channel();
    let mut pending = frame.seal_with_wake(
        staging,
        Arc::new(move || {
            let _ = sender.send(());
        }),
    );
    receiver.recv_timeout(Duration::from_secs(6)).unwrap();
    actual_loss(&core);
    assert!(
        matches!(pending.poll(), Some(Err(RasterizeFailure::DeviceLost(_)))),
        "ready completion hid a later actual loss"
    );
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_loss_waits_for_active_submission_to_retire() {
    let core = core();
    let frame = core.try_begin_native_frame().unwrap();
    let _staging = draw(&frame, &scene());
    actual_loss(&core);
    assert!(
        core.loss_notice().is_none(),
        "loss escaped an active submitting interval"
    );
    assert!(matches!(
        frame.rasterize(
            2,
            &scene(),
            32,
            24,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.
        ),
        Err(RasterizeFailure::DeviceLost(_))
    ));
    drop(frame);
    let deadline = Instant::now() + Duration::from_secs(5);
    while core.loss_notice().is_none() {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
}

#[test]
#[ignore = "actual GPU; execute the serialized native frame gate"]
fn native_batch_ignored_raster_error_still_refuses_the_whole_batch() {
    let core = core();
    let frame = core.try_begin_native_frame().unwrap();
    let mut bad = Scene::new(32, 24);
    let mut path = netrender::scene::ScenePath::new();
    path.move_to(0., 0.)
        .cubic_to(0., 1e30, 1e30, 1e30, 31., 0.)
        .line_to(0., 0.)
        .close();
    bad.push_shape_filled(path, [1., 0., 0., 1.]);
    assert!(matches!(
        frame.rasterize(
            1,
            &bad,
            32,
            24,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.
        ),
        Err(RasterizeFailure::ProviderError(_))
    ));
    let staging = draw(&frame, &scene());
    match collect(&mut frame.seal(staging)) {
        Err(RasterizeFailure::ProviderError(error)) => assert!(matches!(
            &*error,
            netrender::VelloError::CurveEstimateLimit { .. }
        )),
        _ => panic!("ignored raster refusal allowed a partially valid batch to publish"),
    }
}
