// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::NativeThreadCpuStamp;
pub use netrender::DiagnosticWorkSpan as NativeWorkSpan;
use std::sync::atomic::Ordering;

/// Worker-thread measurements for this exact batch. Nested fields are named
/// separately: quality_total contains poll/map/lock; scope_wait contains wait.
/// `total` ends after the GPU metadata snapshot, before diagnostic report
/// finalization, completion construction, channel delivery and wake. The
/// broader NativeFrameTiming worker interval retains its original endpoint.
#[derive(Clone, Copy, Debug)]
pub struct NativeWorkerAttribution {
    pub batch_id: u64,
    pub core_generation: u64,
    pub total: NativeWorkSpan,
    pub poll: NativeWorkSpan,
    pub wait: NativeWorkSpan,
    pub scopes: NativeWorkSpan,
    pub quality_map: NativeWorkSpan,
    pub quality_lock: NativeWorkSpan,
    pub gpu_readback: NativeWorkSpan,
    pub yielded: NativeWorkSpan,
    pub other: Option<NativeWorkSpan>,
    pub quality_total: NativeWorkSpan,
    pub scope_wait: NativeWorkSpan,
    pub quality_polls: u64,
    pub mapped_buffers: u64,
    pub mapped_bytes: u64,
    pub map_callbacks: u64,
    pub map_ready_wall: Option<Duration>,
    pub cpu_partition_rejected: bool,
    pub gpu_timestamp_enabled: bool,
    pub gpu_timestamp_supported: bool,
    pub gpu: netrender::GpuReport,
}

pub(super) struct Capture {
    pub(super) data: NativeWorkerAttribution,
    start: Option<(Instant, NativeThreadCpuStamp)>,
}

fn add(into: &mut NativeWorkSpan, sample: NativeWorkSpan) {
    if sample.calls == 0 {
        return;
    }
    into.wall += sample.wall;
    into.thread_cpu = if into.calls == 0 {
        sample.thread_cpu
    } else {
        into.thread_cpu.zip(sample.thread_cpu).map(|(a, b)| a + b)
    };
    into.calls += sample.calls;
}

fn remainder(total: NativeWorkSpan, spans: &[&NativeWorkSpan]) -> (Option<NativeWorkSpan>, bool) {
    let wall_sum: Duration = spans.iter().map(|s| s.wall).sum();
    let cpu_sum = spans
        .iter()
        .filter(|s| s.calls > 0)
        .try_fold(Duration::ZERO, |sum, s| s.thread_cpu.map(|cpu| sum + cpu));
    let cpu = total
        .thread_cpu
        .zip(cpu_sum)
        .and_then(|(total, parts)| total.checked_sub(parts));
    let rejected = total.thread_cpu.is_some() && cpu_sum.is_some() && cpu.is_none();
    (
        total.wall.checked_sub(wall_sum).map(|wall| NativeWorkSpan {
            wall,
            thread_cpu: cpu,
            calls: 1,
        }),
        rejected,
    )
}

fn measure<T>(enabled: bool, call: impl FnOnce() -> T) -> (T, NativeWorkSpan) {
    if !enabled {
        return (call(), NativeWorkSpan::default());
    }
    let wall = Instant::now();
    let cpu = NativeThreadCpuStamp::now();
    let result = call();
    let thread_cpu = NativeThreadCpuStamp::now().elapsed_since(cpu);
    (
        result,
        NativeWorkSpan {
            wall: wall.elapsed(),
            thread_cpu,
            calls: 1,
        },
    )
}

impl Capture {
    pub(super) fn new(id: u64, generation: u64, enabled: bool, supported: bool) -> Self {
        Self {
            start: None,
            data: NativeWorkerAttribution {
                batch_id: id,
                core_generation: generation,
                total: NativeWorkSpan::default(),
                poll: NativeWorkSpan::default(),
                wait: NativeWorkSpan::default(),
                scopes: NativeWorkSpan::default(),
                quality_map: NativeWorkSpan::default(),
                quality_lock: NativeWorkSpan::default(),
                gpu_readback: NativeWorkSpan::default(),
                yielded: NativeWorkSpan::default(),
                other: None,
                quality_total: NativeWorkSpan::default(),
                scope_wait: NativeWorkSpan::default(),
                quality_polls: 0,
                mapped_buffers: 0,
                mapped_bytes: 0,
                map_callbacks: 0,
                map_ready_wall: None,
                cpu_partition_rejected: false,
                gpu_timestamp_enabled: enabled,
                gpu_timestamp_supported: supported,
                gpu: netrender::GpuReport::disabled(id),
            },
        }
    }
    pub(super) fn start(&mut self, at: Instant) {
        self.start = Some((at, NativeThreadCpuStamp::now()));
    }
    fn finish(mut self, gpu: netrender::GpuReport) -> NativeWorkerAttribution {
        let cpu_end = NativeThreadCpuStamp::now();
        let finished = Instant::now();
        if let Some((wall, cpu)) = self.start {
            self.data.total = NativeWorkSpan {
                wall: finished.duration_since(wall),
                thread_cpu: cpu_end.elapsed_since(cpu),
                calls: 1,
            };
        }
        self.data.gpu = gpu;
        let spans = [
            &self.data.poll,
            &self.data.wait,
            &self.data.scopes,
            &self.data.quality_map,
            &self.data.quality_lock,
            &self.data.gpu_readback,
            &self.data.yielded,
        ];
        (self.data.other, self.data.cpu_partition_rejected) = remainder(self.data.total, &spans);
        self.data
    }
}

#[cfg(test)]
mod tests;

impl NativeFrames {
    pub(super) fn begin_attribution(&self, core: &RenderCore, id: u64) -> Option<Capture> {
        if !self.diagnostic.load(Ordering::Acquire) {
            return None;
        }
        core.renderer.begin_diagnostic_batch(id);
        Some(Capture::new(
            id,
            core.generation(),
            core.device()
                .features()
                .contains(wgpu::Features::TIMESTAMP_QUERY),
            core.renderer
                .wgpu_device
                .core
                .adapter
                .features()
                .contains(wgpu::Features::TIMESTAMP_QUERY),
        ))
    }
    pub(super) fn next_batch_id(&self) -> u64 {
        self.batch_counter
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_add(1))
            .expect("native batch ID exhausted")
            + 1
    }
}

impl RenderCore {
    pub fn native_worker_diagnostic_features() -> wgpu::Features {
        wgpu::Features::TIMESTAMP_QUERY
    }
    /// Configure only an idle Core; a shared device cannot gain new features.
    pub fn enable_native_worker_diagnostics(&self) -> Result<(), RasterizeFailure> {
        let _gate = self.native_frames.checked()?;
        let gpu = self
            .device()
            .features()
            .contains(Self::native_worker_diagnostic_features());
        if !self.renderer.enable_native_diagnostics(gpu) {
            return Err(unavailable("Classic diagnostics unavailable"));
        }
        self.native_frames.diagnostic.store(true, Ordering::Release);
        Ok(())
    }
}

impl NativeFrameSubmission {
    pub fn batch_id(&self) -> u64 {
        self.batch_id
    }
}
impl<T> AcceptedNativeFrame<T> {
    pub fn batch_id(&self) -> u64 {
        self.batch_id
    }
    pub fn worker_attribution(&self) -> Option<NativeWorkerAttribution> {
        self.attribution
    }
}

impl Qualification {
    pub(super) fn start_attribution(&mut self, at: Instant) {
        if let Some(capture) = &mut self.attribution {
            capture.start(at);
        }
    }
    pub(super) fn finish_attribution(&mut self) -> Option<NativeWorkerAttribution> {
        let capture = self.attribution.take()?;
        let gpu = self.core.upgrade().map_or_else(
            || netrender::GpuReport::disabled(self.batch_id),
            |core| core.renderer.diagnostic_gpu_snapshot(self.batch_id),
        );
        Some(capture.finish(gpu))
    }
    pub(super) fn poll_device(
        &mut self,
        core: &RenderCore,
    ) -> Result<wgpu::PollStatus, wgpu::PollError> {
        let started = Instant::now();
        let (result, span) = measure(self.attribution.is_some(), || {
            core.device().poll(wgpu::PollType::Poll)
        });
        self.progress.last_device_poll = Some(started.elapsed());
        if let Some(c) = &mut self.attribution {
            add(&mut c.data.poll, span);
        }
        result
    }
    pub(super) fn poll_quality(
        &mut self,
        core: &RenderCore,
    ) -> Result<Option<(usize, u64)>, netrender::VelloError> {
        let started = Instant::now();
        if self.attribution.is_none() {
            let result = core.renderer.poll_vello_curve_quality();
            self.progress.last_quality_poll = Some(started.elapsed());
            return result;
        }
        let ((result, observation), total) =
            measure(true, || core.renderer.poll_vello_curve_quality_observed());
        self.progress.last_quality_poll = Some(started.elapsed());
        let c = self.attribution.as_mut().unwrap();
        add(&mut c.data.quality_total, total);
        add(&mut c.data.quality_lock, observation.lock);
        add(&mut c.data.poll, observation.provider.driver_poll);
        add(&mut c.data.quality_map, observation.provider.map);
        add(&mut c.data.gpu_readback, observation.gpu_readback);
        c.data.quality_polls += 1;
        c.data.mapped_buffers += observation.provider.mapped_buffers;
        c.data.mapped_bytes += observation.provider.mapped_bytes;
        c.data.map_callbacks += observation.provider.map_callbacks;
        if let Some(wall) = observation.provider.map_ready_wall {
            *c.data.map_ready_wall.get_or_insert(Duration::ZERO) += wall;
        }
        result
    }
    pub(super) fn wait_device(&mut self, device: &wgpu::Device) {
        let pending_scopes = !self.scopes.is_empty();
        let started = Instant::now();
        let (result, span) = measure(self.attribution.is_some(), || {
            device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_millis(5)),
            })
        });
        self.progress.last_wait = Some((started.elapsed(), format!("{result:?}")));
        if let Some(c) = &mut self.attribution {
            add(&mut c.data.wait, span);
            if pending_scopes {
                add(&mut c.data.scope_wait, span);
            }
        }
    }
    pub(super) fn yield_worker(&mut self) {
        let (_, span) = measure(self.attribution.is_some(), std::thread::yield_now);
        if let Some(c) = &mut self.attribution {
            add(&mut c.data.yielded, span);
        }
    }
    pub(super) fn scope_stamp(&self) -> Option<(Instant, NativeThreadCpuStamp)> {
        self.attribution
            .as_ref()
            .map(|_| (Instant::now(), NativeThreadCpuStamp::now()))
    }
    pub(super) fn record_scopes(&mut self, stamp: Option<(Instant, NativeThreadCpuStamp)>) {
        if let Some((wall, cpu)) = stamp {
            let thread_cpu = NativeThreadCpuStamp::now().elapsed_since(cpu);
            if let Some(c) = &mut self.attribution {
                add(
                    &mut c.data.scopes,
                    NativeWorkSpan {
                        wall: wall.elapsed(),
                        thread_cpu,
                        calls: 1,
                    },
                );
            }
        }
    }
}
