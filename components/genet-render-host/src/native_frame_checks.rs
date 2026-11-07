// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Qualification {
    pub(super) fn check_live(&self) -> Result<Arc<RenderCore>, RasterizeFailure> {
        if let Some(loss) = &self
            .admission
            .lock()
            .expect("native admission poisoned")
            .loss
        {
            return Err(RasterizeFailure::DeviceLost(loss.clone()));
        }
        let core = self
            .core
            .upgrade()
            .ok_or_else(|| unavailable("native frame Core retired"))?;
        if let Some(loss) = core.device_failure() {
            return Err(RasterizeFailure::DeviceLost(loss));
        }
        Ok(core)
    }

    pub(super) fn check(&self) -> Result<Arc<RenderCore>, RasterizeFailure> {
        let core = self.check_live()?;
        if Instant::now() >= self.deadline {
            let now = Instant::now();
            return Err(RenderCore::native_raster_unavailable(format!(
                "native frame qualification exceeded five seconds (admission={:?}, seal={:?}, worker={:?}, last scopes/backlog={}, {})",
                now.duration_since(self.admitted),
                self.sealed.map(|at| now.duration_since(at)),
                self.worker_started.map(|at| now.duration_since(at)),
                self.last_observation
                    .map_or_else(|| "unknown".to_string(), |value| format!("{value:?}")),
                self.progress.describe(now),
            )));
        }
        Ok(core)
    }
}
