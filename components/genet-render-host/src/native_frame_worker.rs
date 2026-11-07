// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Job {
    pub(super) fn run(mut self) {
        self.qualification.worker_started = Some(Instant::now());
        self.timing.start_worker();
        self.qualification
            .start_attribution(self.timing.worker_started_at());
        let result = self.observe();
        self.qualification.progress.worker_finished = Some(Instant::now());
        self.complete(result);
    }

    fn observe(&mut self) -> Result<(), RasterizeFailure> {
        let core = self.qualification.check()?;
        // This interval actually touches GPU resources. Loss can cancel the
        // idle/publication lease, but must await this interval's retirement.
        let operation = core
            .loss
            .begin_operation()
            .map_err(RasterizeFailure::DeviceLost)?;
        let device = core.device().clone();
        drop(core);
        // Native scopes are thread-local, including download unmapping. UI
        // submission scopes cannot cover these completion-time operations.
        let mut scopes = [
            wgpu::ErrorFilter::OutOfMemory,
            wgpu::ErrorFilter::Internal,
            wgpu::ErrorFilter::Validation,
        ]
        .into_iter()
        .map(|filter| device.push_error_scope(filter))
        .collect::<Vec<_>>();
        let result = self.qualification.wait();
        self.qualification.progress.wait_returned = Some(Instant::now());
        let started = Instant::now();
        while let Some(scope) = scopes.pop() {
            self.qualification.scopes.push(Box::pin(scope.pop()));
        }
        self.qualification.progress.worker_scope_pop = Some(started.elapsed());
        // Scope futures on native wgpu-core are observed without issuing
        // another GPU operation after their worker-local scopes have closed.
        let started = Instant::now();
        self.qualification.observe_scopes();
        self.qualification.progress.worker_scope_observe = Some(started.elapsed());
        let result = if let Err(error) = self.qualification.check() {
            Err(error)
        } else if !self.qualification.scopes.is_empty() {
            Err(unavailable("native worker scope outcome unavailable"))
        } else if let Some(error) = self.qualification.first_error.take() {
            Err(error)
        } else {
            result
        };
        drop(device);
        drop(operation);
        result
    }

    pub(super) fn refuse(self, reason: &str) {
        self.complete(Err(unavailable(reason)));
    }

    fn complete(mut self, result: Result<(), RasterizeFailure>) {
        let reusable =
            self.qualification.drained() || matches!(&result, Err(RasterizeFailure::DeviceLost(_)));
        let attribution = self.qualification.finish_attribution();
        let batch_id = self.qualification.batch_id;
        let completion = Completion {
            qualification: Some(self.qualification),
            result,
            reusable,
            batch_id,
            attribution,
            // Preserve the original worker-ready endpoint after diagnostic
            // finalization, rather than moving it to the CPU sample boundary.
            timing: self.timing.ready_at(Instant::now()),
        };
        if let Some(sender) = self.completed {
            let _ = sender.send(completion);
        } else {
            drop(completion);
        }
        // Optional host code runs outside every owner lock and GPU scope.
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (self.wake)()));
    }
}
