// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;

fn span(wall: u64, cpu: Option<u64>) -> NativeWorkSpan {
    NativeWorkSpan {
        wall: Duration::from_nanos(wall),
        thread_cpu: cpu.map(Duration::from_nanos),
        calls: 1,
    }
}

#[test]
fn disjoint_wall_and_cpu_telescope_without_nested_quality_or_scope_wait() {
    let total = span(101, Some(47));
    let a = span(10, Some(3));
    let b = span(20, Some(5));
    let (other, rejected) = remainder(total, &[&a, &b]);
    let other = other.unwrap();
    assert!(!rejected);
    assert_eq!(other.wall, Duration::from_nanos(71));
    assert_eq!(other.thread_cpu, Some(Duration::from_nanos(39)));
    assert_eq!(a.wall + b.wall + other.wall, total.wall);
    assert_eq!(
        a.thread_cpu.unwrap() + b.thread_cpu.unwrap() + other.thread_cpu.unwrap(),
        total.thread_cpu.unwrap()
    );
}

#[test]
fn unavailable_or_invalid_cpu_and_overlapping_wall_are_not_clamped_to_zero() {
    let a = span(10, None);
    assert_eq!(
        remainder(span(20, Some(5)), &[&a]).0.unwrap().thread_cpu,
        None
    );
    assert!(!remainder(span(20, Some(5)), &[&a]).1);
    let a = span(10, Some(9));
    let (other, rejected) = remainder(span(20, Some(5)), &[&a]);
    assert!(rejected);
    assert_eq!(other.unwrap().thread_cpu, None);
    assert_eq!(a.thread_cpu, Some(Duration::from_nanos(9)));
    assert!(remainder(span(5, Some(5)), &[&a]).0.is_none());
}

fn core() -> Arc<RenderCore> {
    Arc::new(
        RenderCore::boot(netrender::NetrenderOptions {
            enable_vello: true,
            tile_cache_size: Some(32),
            optional_features: RenderCore::native_worker_diagnostic_features(),
            ..Default::default()
        })
        .expect("actual native adapter required"),
    )
}
fn collect<T>(
    pending: &mut PendingNativeFrame<T>,
) -> Result<AcceptedNativeFrame<T>, RasterizeFailure> {
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        if let Some(result) = pending.poll() {
            return result;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn scene() -> netrender::Scene {
    let mut scene = netrender::Scene::new(32, 24);
    scene.push_rect(0., 0., 32., 24., [0.25, 0.5, 0.75, 1.]);
    scene
}

#[test]
#[ignore = "actual GPU adapter inventory; execute serialized qualification"]
fn enumerates_all_native_timestamp_capabilities() {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let mut future = std::pin::pin!(instance.enumerate_adapters(wgpu::Backends::all()));
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    let deadline = Instant::now() + Duration::from_secs(5);
    let adapters = loop {
        if let std::task::Poll::Ready(adapters) = future.as_mut().poll(&mut context) {
            break adapters;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    };
    assert!(
        !adapters.is_empty(),
        "actual adapter inventory must execute nonzero"
    );
    for adapter in adapters {
        eprintln!(
            "native-adapter={:?} timestamp_query_supported={}",
            adapter.get_info(),
            adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY)
        );
    }
}

#[test]
#[ignore = "actual GPU pixels and worker diagnostics; execute serialized qualification"]
fn native_worker_attribution_preserves_output_and_reports_real_capability() {
    let core = core();
    let scene = scene();
    let (baseline, _) = core
        .try_rasterize_scaled_for(
            101,
            &scene,
            32,
            24,
            netrender::ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.,
        )
        .unwrap();
    let expected = core.read_rgba8_texture(&baseline, 32, 24).unwrap();
    core.enable_native_worker_diagnostics().unwrap();
    let frame = core.try_begin_native_frame().unwrap();
    let id = frame.batch_id();
    let (texture, _) = frame
        .rasterize(
            101,
            &scene,
            32,
            24,
            netrender::ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.,
        )
        .unwrap();
    let accepted = collect(&mut frame.seal(texture)).unwrap();
    assert_eq!(accepted.batch_id(), id);
    let data = accepted
        .worker_attribution()
        .expect("CPU diagnostics independent of GPU support");
    assert_eq!(data.batch_id, id);
    assert_eq!(data.core_generation, core.generation());
    // The comparable worker wall interval also includes diagnostic report
    // finalization after the narrower same-thread observation has ended.
    assert!(data.total.wall <= accepted.timing().worker());
    let parts = [
        data.poll,
        data.wait,
        data.scopes,
        data.quality_map,
        data.quality_lock,
        data.gpu_readback,
        data.yielded,
        data.other.unwrap(),
    ];
    assert_eq!(
        parts.iter().map(|s| s.wall).sum::<Duration>(),
        data.total.wall
    );
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    assert!(
        data.total.thread_cpu.is_some(),
        "native current-thread CPU must be observed"
    );
    assert_eq!(
        data.gpu_timestamp_enabled,
        core.device()
            .features()
            .contains(wgpu::Features::TIMESTAMP_QUERY)
    );
    if !data.gpu_timestamp_enabled {
        assert_eq!(data.gpu.status, netrender::GpuStatus::Unsupported);
        assert_eq!(data.gpu.elapsed_ns, None);
    }
    eprintln!("native-worker={data:?}");
    let texture = accepted.publish(|texture| texture).unwrap();
    assert_eq!(
        core.read_rgba8_texture(&texture, 32, 24).unwrap().rgba,
        expected.rgba
    );
    if data.gpu_timestamp_enabled {
        // Test-only following native turn observes any late diagnostic map;
        // production qualification never waits for a timing probe.
        let follow = core.try_begin_native_frame().unwrap();
        collect(&mut follow.seal(()))
            .unwrap()
            .publish(|_| ())
            .unwrap();
        let report = core.renderer.diagnostic_gpu_snapshot(id);
        assert!(
            report.recordings > 0,
            "actual scene commands must be bracketed"
        );
        assert!(matches!(
            report.status,
            netrender::GpuStatus::Available | netrender::GpuStatus::Pending
        ));
        if report.status == netrender::GpuStatus::Available {
            assert!(report.elapsed_ns.unwrap() > 0);
        } else {
            assert_eq!(report.elapsed_ns, None);
        }
        eprintln!("actual-scene-GPU-bracket={report:?}");
    }
}

#[test]
#[ignore = "actual GPU validation preservation; execute serialized qualification"]
fn diagnostic_observation_preserves_original_validation_refusal() {
    let core = core();
    for diagnostic in [false, true] {
        if diagnostic {
            core.enable_native_worker_diagnostics().unwrap();
        }
        let frame = core.try_begin_native_frame().unwrap();
        let _ = frame
            .rasterize(
                101,
                &scene(),
                32,
                24,
                netrender::ColorLoad::Clear(wgpu::Color::TRANSPARENT),
                1.,
            )
            .unwrap();
        let _ = core
            .device()
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("native diagnostic refusal control"),
                source: wgpu::ShaderSource::Wgsl("invalid shader".into()),
            });
        let error = match collect(&mut frame.seal(())) {
            Ok(_) => panic!("invalid shader must remain refused"),
            Err(error) => error,
        };
        assert!(matches!(error, RasterizeFailure::ProviderError(_)));
        assert!(core.device_failure().is_none());
    }
}

#[test]
#[ignore = "actual GPU shader-quality preservation; execute serialized qualification"]
fn diagnostic_observation_preserves_original_curve_quality_refusal_and_recovery() {
    let core = core();
    let generation = core.generation();
    let mut path = netrender::scene::ScenePath::new();
    path.move_to(8., 12.).line_to(24., 12.);
    let mut bad = netrender::Scene::new(32, 24);
    bad.push_shape_stroked(path, [0.25, 0.5, 0.75, 1.], 40_000_000.);
    let netrender::scene::SceneOp::Shape(shape) = &mut bad.ops[0] else {
        unreachable!()
    };
    shape.stroke.as_mut().unwrap().cap = netrender::scene::SceneStrokeCap::Round;
    for diagnostic in [false, true] {
        if diagnostic {
            core.enable_native_worker_diagnostics().unwrap();
        }
        let frame = core.try_begin_native_frame().unwrap();
        let (texture, _) = frame
            .rasterize(
                101,
                &bad,
                32,
                24,
                netrender::ColorLoad::Clear(wgpu::Color::TRANSPARENT),
                1.,
            )
            .unwrap();
        match collect(&mut frame.seal(texture)) {
            Err(RasterizeFailure::ProviderError(error)) => assert!(matches!(
                &*error,
                netrender::VelloError::CurveFlatteningLimit
            )),
            _ => panic!("diagnostics changed current shader-quality refusal"),
        }
        assert_eq!(core.generation(), generation);
        assert!(core.device_failure().is_none());
        assert_eq!(core.renderer.vello_curve_quality_status(), Some((0, 0)));
        let frame = core.try_begin_native_frame().unwrap();
        let (texture, _) = frame
            .rasterize(
                101,
                &scene(),
                32,
                24,
                netrender::ColorLoad::Clear(wgpu::Color::TRANSPARENT),
                1.,
            )
            .unwrap();
        collect(&mut frame.seal(texture))
            .unwrap()
            .publish(|_| ())
            .unwrap();
    }
}
