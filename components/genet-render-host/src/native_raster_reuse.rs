// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native admission for a caller's already qualified retained raster.

use crate::{RasterizeFailure, RenderCore};

impl RenderCore {
    /// Admit reuse of an already qualified native raster on this Core/device.
    ///
    /// The caller must supply an exact, complete content/resource witness for
    /// the retained output. This method validates only native-frame and loss
    /// admission: it does not validate a picture or submit, qualify, poll or
    /// rasterize new GPU work. `reuse` should only clone the witnessed output.
    /// A current native batch refuses reuse without calling `reuse`.
    pub fn try_reuse_native_raster<T>(
        &self,
        reuse: impl FnOnce() -> T,
    ) -> Result<T, RasterizeFailure> {
        #[cfg(feature = "without-classic-vello")]
        self.unavailable();
        if let Some(loss) = self.device_failure() {
            return Err(RasterizeFailure::DeviceLost(loss));
        }
        let _native_admission = self.native_frames.checked().map_err(|error| {
            self.device_failure()
                .map(RasterizeFailure::DeviceLost)
                .unwrap_or(error)
        })?;
        let _operation = self
            .loss
            .begin_operation()
            .map_err(RasterizeFailure::DeviceLost)?;
        let output = reuse();
        if let Some(loss) = self.device_failure() {
            return Err(RasterizeFailure::DeviceLost(loss));
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NativeFrameStartError;
    use netrender::NetrenderOptions;
    use std::{
        cell::Cell,
        sync::Arc,
        time::{Duration, Instant},
    };

    fn core() -> Arc<RenderCore> {
        Arc::new(
            RenderCore::boot(NetrenderOptions {
                enable_vello: true,
                tile_cache_size: Some(8),
                ..Default::default()
            })
            .expect("actual native adapter required"),
        )
    }

    #[test]
    #[ignore = "actual GPU; execute the serialized native frame gate"]
    fn reuse_holds_exclusive_admission_and_returns_the_cloned_value_without_raster() {
        let core = core();
        let before = core.renderer.vello_curve_quality_status();
        let called = Cell::new(0);
        let output = core
            .try_reuse_native_raster(|| {
                called.set(called.get() + 1);
                assert!(core.native_frame_busy());
                assert!(matches!(
                    core.try_begin_native_frame(),
                    Err(NativeFrameStartError::Busy)
                ));
                assert!(matches!(
                    core.try_reuse_native_raster(|| panic!("nested clone must be refused")),
                    Err(RasterizeFailure::ProviderError(_))
                ));
                71
            })
            .unwrap();
        assert_eq!(output, 71);
        assert_eq!(called.get(), 1);
        assert!(!core.native_frame_busy());
        assert_eq!(core.renderer.vello_curve_quality_status(), before);
        assert_eq!(core.try_reuse_native_raster(|| 72).unwrap(), 72);
    }

    #[test]
    #[ignore = "actual GPU; execute the serialized native frame gate"]
    fn submitting_and_pending_native_batches_refuse_reuse_before_clone() {
        let core = core();
        let called = Cell::new(0);
        let frame = core.try_begin_native_frame().unwrap();
        assert!(matches!(
            core.try_reuse_native_raster(|| called.set(called.get() + 1)),
            Err(RasterizeFailure::ProviderError(_))
        ));
        let pending = frame.seal(71);
        assert!(matches!(
            core.try_reuse_native_raster(|| called.set(called.get() + 1)),
            Err(RasterizeFailure::ProviderError(_))
        ));
        assert_eq!(called.get(), 0);
        drop(pending);
    }

    #[test]
    #[ignore = "actual device destruction; execute the serialized native frame gate"]
    fn loss_during_reuse_discards_the_value_and_later_reuse_never_calls_clone() {
        let core = core();
        let result = core.try_reuse_native_raster(|| {
            core.device().destroy();
            let deadline = Instant::now() + Duration::from_secs(5);
            while core.device_failure().is_none() {
                assert!(
                    Instant::now() < deadline,
                    "actual device loss was not delivered"
                );
                let _ = core.device().poll(wgpu::PollType::Poll);
                std::thread::yield_now();
            }
            assert!(
                core.loss_notice().is_none(),
                "reuse still holds loss admission"
            );
            71
        });
        let loss = match result {
            Err(RasterizeFailure::DeviceLost(loss)) => loss,
            _ => panic!("a lost device must never return the retained value"),
        };
        assert_eq!(loss.reason, wgpu::DeviceLostReason::Destroyed);
        assert_eq!(Some(loss.clone()), core.loss_notice());
        let called = Cell::new(false);
        assert!(matches!(
            core.try_reuse_native_raster(|| called.set(true)),
            Err(RasterizeFailure::DeviceLost(later)) if later == loss
        ));
        assert!(!called.get());
    }
}
