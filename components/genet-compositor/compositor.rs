/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! OS presentation and master-texture capture backends.
//!
//! Implements [`netrender_device::compositor::Compositor`], the trait
//! `netrender::Renderer::render_with_compositor` invokes once per frame
//! to hand back the master texture + per-surface `LayerPresent` slice.
//!
//! ## What's here
//!
//! - [`WgpuMasterCaptureBackend`] — single-fullscreen-surface capture. Doesn't
//!   talk to the OS compositor; just stashes the master texture from
//!   the most recent `present_frame` call. The `composite_texture`
//!   path on `Paint` reads from this for the wgpu-shared-device
//!   embedder hand-off (`WebView::composite_texture()`).
//! - [`ServoCompositor`] — production wrapper that delegates to a
//!   per-platform [`OsCompositorBackend`]. Holds a
//!   [`HostWgpuContext`] (device + queue + detected backend) so
//!   backends can issue their own GPU work; per-platform
//!   synchronizers (e.g. [`crate::interop::Dx12FenceSynchronizer`])
//!   own the producer-fence / consumer-fence bookkeeping that
//!   survives the netrender → OS-compositor handoff.
//!
//! ## Direction note
//!
//! Pre-cut, the OS-handoff was webrender's job and lived inside its
//! renderer. C4 lifts the responsibility into a netrender-shaped
//! [`Compositor`] impl on the consumer side. The direction-neutral
//! interop primitives ([`InteropBackend`], [`HostWgpuContext`],
//! [`SyncMechanism`], the platform fence wrappers) live in
//! [`crate::interop`] — see
//! [`docs/2026-05-09_interop_lineage.md`](../../docs/2026-05-09_interop_lineage.md)
//! for the slint → graft → scrying → genet provenance and the
//! export-direction trait-shape reasoning.
//!
//! See [`docs/2026-05-05_genet_netrender_cut_plan.md`](../../docs/2026-05-05_genet_netrender_cut_plan.md)
//! § C4 for the cut design.

use std::collections::HashMap;

use netrender_device::compositor::{Compositor, PresentedFrame, SurfaceKey};
use rustc_hash::FxHashMap;
use wgpu::Texture;

use crate::interop::{HostWgpuContext, InteropBackend, SyncMechanism};

/// What a compositor is capable of handing to its caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationRoute {
    Display,
    Capture,
}

/// The operation that prevented a complete OS handoff for one frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationStage {
    Master,
    Declare(SurfaceKey),
    Surface(SurfaceKey),
}

/// Host-visible result of the most recent `present_frame` call. A display
/// handoff means the native commit/present call accepted the frame, not that
/// the OS has scanned it out.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PresentationOutcome {
    DisplayHandoff,
    CaptureReady,
    Failed {
        stage: PresentationStage,
        message: String,
    },
}

/// A `Compositor` impl that captures the master texture from each
/// `present_frame` call so embedder code can read it back via
/// [`WgpuMasterCaptureBackend::last_master`].
///
/// This is the **wgpu-shared-device embedder route** — the embedder
/// holds the same wgpu device as netrender, so the master texture it
/// reads here is directly samplable in its own render pass (zero
/// copy). It's the right backend when the embedder wants to integrate
/// the genet composite into its own render pipeline (e.g. for a
/// custom UI shell that draws on top), and the wrong one when the
/// embedder wants genet to present pixels directly to the OS — for
/// that, install a per-platform backend ([`crate::compositor_dxgi::WindowsDxgiBackend`]
/// on Windows; `MacosCALayerBackend` on macOS;
/// `WaylandSubsurfaceBackend` on Linux).
///
pub struct WgpuMasterCaptureBackend {
    /// Per-surface world bounds; updated on `declare_surface`.
    surfaces: FxHashMap<SurfaceKey, [f32; 4]>,
    /// Cloned handle to the most recently presented master texture.
    /// `wgpu::Texture` is `Arc`-shared internally so cloning is cheap.
    last_master: Option<Texture>,
    last_outcome: Option<PresentationOutcome>,
}

impl WgpuMasterCaptureBackend {
    pub fn new() -> Self {
        Self {
            surfaces: FxHashMap::default(),
            last_master: None,
            last_outcome: None,
        }
    }

    /// Texture handed to the most recent `present_frame` call, if any.
    /// Used by `Paint::composite_texture` for the wgpu-shared-device
    /// embedder route.
    pub fn last_master(&self) -> Option<&Texture> {
        self.last_master.as_ref()
    }
}

impl Default for WgpuMasterCaptureBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Compositor for WgpuMasterCaptureBackend {
    fn declare_surface(&mut self, key: SurfaceKey, world_bounds: [f32; 4]) {
        self.surfaces.insert(key, world_bounds);
    }

    fn destroy_surface(&mut self, key: SurfaceKey) {
        self.surfaces.remove(&key);
    }

    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        // Capture the master so `Paint::composite_texture` can hand
        // it back to the wgpu-shared-device embedder.
        self.last_master = Some(frame.master.clone());
        self.last_outcome = Some(PresentationOutcome::CaptureReady);
    }
}

/// Extension trait Paint stores compositors through. Adds the
/// `last_master()` accessor that [`Paint::composite_texture`] reads
/// — a method only [`WgpuMasterCaptureBackend`] meaningfully
/// implements. Default returns `None` so platform OS-handoff
/// backends ([`ServoCompositor`] / `WindowsDxgiBackend` etc.)
/// satisfy the trait without exposing a master-texture path that
/// wouldn't make sense for them (their pixels go to the OS
/// compositor, not back to the embedder's wgpu device).
pub trait PaintCompositor: Compositor + Send {
    /// The route chosen at construction, independent of any frame result.
    fn presentation_route(&self) -> PresentationRoute;

    /// Result of the most recent frame, or `None` before the first frame.
    fn presentation_outcome(&self) -> Option<&PresentationOutcome>;

    /// Consume the latest result. A host can clear this before rendering and
    /// require a fresh result afterward, even if its renderer skips the callback.
    fn take_presentation_outcome(&mut self) -> Option<PresentationOutcome>;

    /// Most recently captured master texture, if this backend
    /// captures masters. `None` for OS-handoff backends.
    fn last_master(&self) -> Option<&Texture> {
        None
    }
}

impl PaintCompositor for WgpuMasterCaptureBackend {
    fn presentation_route(&self) -> PresentationRoute {
        PresentationRoute::Capture
    }

    fn presentation_outcome(&self) -> Option<&PresentationOutcome> {
        self.last_outcome.as_ref()
    }

    fn take_presentation_outcome(&mut self) -> Option<PresentationOutcome> {
        self.last_outcome.take()
    }

    fn last_master(&self) -> Option<&Texture> {
        WgpuMasterCaptureBackend::last_master(self)
    }
}

impl<B: OsCompositorBackend> PaintCompositor for ServoCompositor<B> {
    fn presentation_route(&self) -> PresentationRoute {
        PresentationRoute::Display
    }

    fn presentation_outcome(&self) -> Option<&PresentationOutcome> {
        self.last_outcome.as_ref()
    }

    fn take_presentation_outcome(&mut self) -> Option<PresentationOutcome> {
        self.last_outcome.take()
    }
}

/// Per-platform OS-compositor backend. Implementations bridge
/// netrender's wgpu master texture into a native texture that the OS
/// compositor (DXGI Composition / CALayer / Wayland subsurface) can
/// consume.
///
/// The trait surface is parameterized over [`HostWgpuContext`] (so
/// backends can encode their own GPU copies onto the host's device+
/// queue) and [`SyncMechanism`] (so the producer→consumer fence
/// machinery is uniform across platforms; the per-platform
/// synchronizer in [`crate::interop`] drives the actual fence
/// signalling).
///
/// Per-platform impls live alongside their `OsCompositorBackend`
/// blanket: the Windows path is in
/// [`crate::compositor_dxgi::WindowsDxgiBackend`].
pub trait OsCompositorBackend: Send {
    /// Present the netrender master texture as the root visual /
    /// fullscreen surface of this backend's OS compositor handoff.
    /// Called from [`ServoCompositor::present_frame`] once per
    /// frame, before any per-`SurfaceKey` `present()` calls.
    ///
    /// A successful return means the native handoff accepted the master.
    fn present_master(&mut self, master: &Texture) -> Result<(), BoxedBackendError>;

    /// Which wgpu/native graphics backend this implementation targets.
    /// `ServoCompositor` cross-checks against the
    /// [`HostWgpuContext::backend`] at construction.
    fn interop_backend(&self) -> InteropBackend;

    /// Synchronization mechanism this backend speaks. Determines what
    /// fence/semaphore the per-frame `present` path coordinates.
    fn sync_mechanism(&self) -> SyncMechanism {
        SyncMechanism::None
    }

    /// Allocate (or reallocate) the destination texture for `key`,
    /// and perform any OS-side bookkeeping needed to register it
    /// with the OS compositor (creating a CALayer /
    /// `IDCompositionVisual` / `wl_subsurface` / etc.).
    ///
    /// Returns the destination wgpu texture the
    /// [`ServoCompositor`] wrapper will encode the per-frame
    /// `copy_texture_to_texture(master[rect] -> dest)` blit into.
    /// Lifetime of the texture is owned by the wrapper (stored in
    /// the per-`SurfaceKey` destination map); `destroy` is the
    /// signal to release any per-key OS resources the backend
    /// allocated alongside.
    ///
    /// The default implementation creates a plain wgpu texture
    /// sized to `width x height` of `format`, with no OS-side
    /// registration. This is enough for the
    /// [`WgpuMasterCaptureBackend`] embedder route and is the right
    /// starting point for any backend that has not yet wired its
    /// per-`SurfaceKey` OS handoff. Per-platform backends that
    /// need IOSurface-backed allocation (macOS), shared-handle
    /// textures (Windows), dmabuf import (Linux Wayland), or
    /// per-surface OS resources (CALayer / `IDCompositionVisual` /
    /// `wl_subsurface`) override this method.
    fn declare(
        &mut self,
        _key: SurfaceKey,
        host: &HostWgpuContext,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<wgpu::Texture, BoxedBackendError> {
        Ok(host.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("OsCompositorBackend default destination"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        }))
    }

    /// Drop a previously-declared surface. After this, the OS
    /// compositor no longer references the surface.
    ///
    /// Default is a no-op for backends that don't allocate
    /// per-surface OS resources alongside the wgpu destination
    /// texture.
    fn destroy(&mut self, _key: SurfaceKey) {}

    /// Hand the surface's native texture to the OS compositor with
    /// the given world transform / clip / opacity. This corresponds
    /// to one entry in netrender's `present_frame` `layers` slice.
    ///
    /// A successful return means the native handoff accepted this surface.
    fn present(
        &mut self,
        key: SurfaceKey,
        transform: [f32; 6],
        clip: Option<[f32; 4]>,
        opacity: f32,
    ) -> Result<(), BoxedBackendError>;
}

/// Boxed error returned by [`OsCompositorBackend::declare`].
/// Backends that fail allocation (out-of-memory, OS handle
/// exhaustion, IOSurface/CALayer creation failure) box their
/// concrete error into this shape so the trait surface stays free
/// of platform-specific error types. `ServoCompositor` records the
/// first failure for the host and skips a layer whose declaration fails.
pub type BoxedBackendError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Production compositor wrapper. Holds an [`OsCompositorBackend`], a
/// [`HostWgpuContext`] for GPU encode access, and a per-`SurfaceKey`
/// destination-texture pool. `present_frame` blits from the master
/// into the destination textures, then hands native handles to the
/// backend.
pub struct ServoCompositor<B: OsCompositorBackend> {
    host: HostWgpuContext,
    destinations: HashMap<SurfaceKey, Texture>,
    backend: B,
    last_outcome: Option<PresentationOutcome>,
}

impl<B: OsCompositorBackend> ServoCompositor<B> {
    /// Construct a compositor over the given host wgpu context and
    /// backend. Panics if the backend's
    /// [`OsCompositorBackend::interop_backend`] doesn't match
    /// `host.backend` — these must agree at construction time so the
    /// per-platform GPU encode paths work.
    pub fn new(host: HostWgpuContext, backend: B) -> Self {
        assert_eq!(
            host.backend,
            backend.interop_backend(),
            "ServoCompositor: HostWgpuContext backend ({:?}) does not match OsCompositorBackend ({:?})",
            host.backend,
            backend.interop_backend(),
        );
        Self {
            host,
            destinations: HashMap::new(),
            backend,
            last_outcome: None,
        }
    }

    /// Reference to the underlying backend (for tests and debug
    /// inspection).
    pub fn backend(&self) -> &B {
        &self.backend
    }

    fn record_failure(&mut self, stage: PresentationStage, error: BoxedBackendError) {
        log::warn!("[ServoCompositor] {stage:?} failed: {error}");
        if !matches!(
            self.last_outcome.as_ref(),
            Some(PresentationOutcome::Failed { .. })
        ) {
            self.last_outcome = Some(PresentationOutcome::Failed {
                stage,
                message: error.to_string(),
            });
        }
    }
}

impl<B: OsCompositorBackend> Compositor for ServoCompositor<B> {
    fn declare_surface(&mut self, _key: SurfaceKey, _world_bounds: [f32; 4]) {
        // Backend texture allocation lands with the per-platform impl.
        // Deferred: per-backend `declare` is invoked from
        // `present_frame` once a destination texture exists for the
        // surface.
    }

    fn destroy_surface(&mut self, key: SurfaceKey) {
        self.destinations.remove(&key);
        self.backend.destroy(key);
    }

    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        // Step 1 — present the master through the OS compositor.
        self.last_outcome = Some(PresentationOutcome::DisplayHandoff);
        if let Err(error) = self.backend.present_master(frame.master) {
            self.record_failure(PresentationStage::Master, error);
            return;
        }

        // Step 2 — per-`SurfaceKey` blit + present. For each
        // `LayerPresent`:
        //  1. ensure `self.destinations[key]` is sized to the layer's
        //     `source_rect_in_master` (allocate or reallocate as
        //     needed; backend.declare on (re)alloc, backend.destroy
        //     before reallocation so any per-key OS resources get
        //     reclaimed).
        //  2. if `dirty`, encode `copy_texture_to_texture(master[rect]
        //     → destination)` on a wgpu encoder built off
        //     `frame.handles.device`.
        //  3. hand the destination's native handle to the OS
        //     compositor via `backend.present(key, transform, clip,
        //     opacity)`.
        // Submit the (single) encoder once after all layers are
        // recorded.
        if frame.layers.is_empty() {
            return;
        }

        let mut encoder =
            frame
                .handles
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("ServoCompositor::present_frame layer blits"),
                });
        let mut recorded_any = false;
        let mut presents = Vec::with_capacity(frame.layers.len());

        for layer in frame.layers {
            let [x0, y0, x1, y1] = layer.source_rect_in_master;
            let w = x1.saturating_sub(x0);
            let h = y1.saturating_sub(y0);
            if w == 0 || h == 0 {
                continue;
            }

            let needs_alloc = match self.destinations.get(&layer.key) {
                Some(t) => {
                    let s = t.size();
                    s.width != w || s.height != h
                },
                None => true,
            };
            if needs_alloc {
                if self.destinations.contains_key(&layer.key) {
                    self.backend.destroy(layer.key);
                    self.destinations.remove(&layer.key);
                }
                // Backend owns allocation now: it can choose plain
                // wgpu (the default impl), IOSurface-backed (Mac),
                // shared-handle (Windows), dmabuf (Linux), etc., and
                // do its OS-side bookkeeping (CALayer /
                // IDCompositionVisual / wl_subsurface) in the same
                // call. On declare failure, log and skip the layer
                // — the master path still presented; only this
                // particular declared surface is missed for the
                // frame.
                match self
                    .backend
                    .declare(layer.key, &self.host, w, h, frame.master.format())
                {
                    Ok(dest) => {
                        self.destinations.insert(layer.key, dest);
                    },
                    Err(err) => {
                        self.record_failure(PresentationStage::Declare(layer.key), err);
                        continue;
                    },
                }
            }

            let dest = match self.destinations.get(&layer.key) {
                Some(d) => d,
                None => continue, // unreachable in practice — we just inserted
            };

            // OR target-side reallocation into the content-side
            // `dirty` signal per path-(b′) §4: netrender's `dirty`
            // tracks *content* changes (tile_intersection, newly_
            // declared, bounds_changed, absent_last_frame); the
            // consumer is responsible for adding *target-side*
            // signals, of which "the destination texture was just
            // (re)allocated and is uninitialized" is the canonical
            // one. Without this OR-in, a `declare` failure on
            // frame N followed by success on a clean (non-dirty)
            // frame N+1 presents an uninitialized destination.
            if layer.dirty || needs_alloc {
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: frame.master,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x: x0, y: y0, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: dest,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: w,
                        height: h,
                        depth_or_array_layers: 1,
                    },
                );
                recorded_any = true;
            }

            presents.push((layer.key, layer.world_transform, layer.clip, layer.opacity));
        }

        if recorded_any {
            frame.handles.queue.submit([encoder.finish()]);
        }
        // Otherwise the encoder is dropped without a submit — fine,
        // wgpu just discards the empty command buffer.

        for (key, world_transform, clip, opacity) in presents {
            if let Err(error) = self.backend.present(key, world_transform, clip, opacity) {
                self.record_failure(PresentationStage::Surface(key), error);
            }
        }

        let _ = &self.host;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netrender_device::WgpuHandles;
    use netrender_device::compositor::LayerPresent;

    struct RejectingBackend {
        backend: InteropBackend,
        reject_master: bool,
        reject_declare: bool,
        reject_surface: bool,
        declare_calls: usize,
        surface_calls: usize,
    }

    impl OsCompositorBackend for RejectingBackend {
        fn interop_backend(&self) -> InteropBackend {
            self.backend
        }

        fn present_master(&mut self, _: &Texture) -> Result<(), BoxedBackendError> {
            if self.reject_master {
                Err("master rejected".into())
            } else {
                Ok(())
            }
        }

        fn declare(
            &mut self,
            _: SurfaceKey,
            host: &HostWgpuContext,
            width: u32,
            height: u32,
            format: wgpu::TextureFormat,
        ) -> Result<Texture, BoxedBackendError> {
            self.declare_calls += 1;
            if self.reject_declare {
                return Err("allocation rejected".into());
            }
            Ok(host.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("H10 mock destination"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            }))
        }

        fn present(
            &mut self,
            _: SurfaceKey,
            _: [f32; 6],
            _: Option<[f32; 4]>,
            _: f32,
        ) -> Result<(), BoxedBackendError> {
            self.surface_calls += 1;
            if self.reject_surface {
                Err("surface rejected".into())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn native_errors_and_recovery_are_visible_per_frame() {
        let handles = pollster::block_on(async {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                flags: wgpu::InstanceFlags::default(),
                memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
                backend_options: wgpu::BackendOptions::default(),
                display: None,
            });
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::LowPower,
                    force_fallback_adapter: true,
                    compatible_surface: None,
                    apply_limit_buckets: false,
                })
                .await
                .expect("fallback adapter");
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("H10 outcome test"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    ..Default::default()
                })
                .await
                .expect("device");
            WgpuHandles {
                instance,
                adapter,
                device,
                queue,
            }
        });
        let host = HostWgpuContext::new(handles.device.clone(), handles.queue.clone());
        let mut compositor = ServoCompositor::new(
            host.clone(),
            RejectingBackend {
                backend: host.backend,
                reject_master: true,
                reject_declare: false,
                reject_surface: false,
                declare_calls: 0,
                surface_calls: 0,
            },
        );
        let master = handles.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("H10 mock master"),
            size: wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let empty = PresentedFrame {
            master: &master,
            handles: &handles,
            layers: &[],
        };
        compositor.present_frame(empty);
        assert_eq!(
            compositor.take_presentation_outcome(),
            Some(PresentationOutcome::Failed {
                stage: PresentationStage::Master,
                message: "master rejected".into(),
            })
        );
        assert_eq!(compositor.take_presentation_outcome(), None);

        let key = SurfaceKey(7);
        let layer = LayerPresent {
            key,
            source_rect_in_master: [0, 0, 4, 4],
            world_transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            clip: None,
            opacity: 1.0,
            dirty: true,
        };
        compositor.present_frame(PresentedFrame {
            master: &master,
            handles: &handles,
            layers: std::slice::from_ref(&layer),
        });
        assert_eq!(
            compositor.take_presentation_outcome(),
            Some(PresentationOutcome::Failed {
                stage: PresentationStage::Master,
                message: "master rejected".into(),
            })
        );
        assert_eq!(compositor.backend.declare_calls, 0);
        assert_eq!(compositor.backend.surface_calls, 0);
        assert!(compositor.destinations.is_empty());

        compositor.backend.reject_master = false;
        compositor.backend.reject_declare = true;
        compositor.present_frame(PresentedFrame {
            master: &master,
            handles: &handles,
            layers: std::slice::from_ref(&layer),
        });
        assert_eq!(
            compositor.take_presentation_outcome(),
            Some(PresentationOutcome::Failed {
                stage: PresentationStage::Declare(key),
                message: "allocation rejected".into(),
            })
        );

        compositor.backend.reject_declare = false;
        compositor.backend.reject_surface = true;
        compositor.present_frame(PresentedFrame {
            master: &master,
            handles: &handles,
            layers: std::slice::from_ref(&layer),
        });
        assert_eq!(
            compositor.take_presentation_outcome(),
            Some(PresentationOutcome::Failed {
                stage: PresentationStage::Surface(key),
                message: "surface rejected".into(),
            })
        );

        compositor.backend.reject_surface = false;
        compositor.present_frame(PresentedFrame {
            master: &master,
            handles: &handles,
            layers: std::slice::from_ref(&layer),
        });
        assert_eq!(
            compositor.take_presentation_outcome(),
            Some(PresentationOutcome::DisplayHandoff)
        );

        let mut capture = WgpuMasterCaptureBackend::new();
        capture.present_frame(PresentedFrame {
            master: &master,
            handles: &handles,
            layers: &[],
        });
        assert_eq!(capture.presentation_route(), PresentationRoute::Capture);
        assert_eq!(
            capture.take_presentation_outcome(),
            Some(PresentationOutcome::CaptureReady)
        );
    }
}
