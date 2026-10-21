/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The genet present core, independent of what it presents onto.
//!
//! Booting wgpu + a netrender [`Renderer`], configuring a surface, rasterizing
//! a [`Scene`] into an offscreen texture, and acquiring a backbuffer are the
//! same mechanics whether the target is a desktop window or a browser canvas.
//! They live here so one implementation serves both; each host keeps its own
//! scene composition and input routing.
//!
//! [`RenderCore::create_surface`] takes anything wgpu accepts as a surface
//! target, so a winit `Arc<Window>` and an `HtmlCanvasElement` both work
//! without this crate depending on either. `genet-winit-host` adds the winit
//! event loop, wheel translation, and the AccessKit bridge on top.
//!
//! Per-frame shape a host follows:
//!
//! ```text
//! let (_tex, view) = core.rasterize(&scene, w, h, clear);   // one per layer
//! let Some(frame)  = surface.acquire(&core) else { return }; // skip if outdated
//! let target = frame.texture.create_view(&Default::default());
//! core.renderer().compose_external_texture(&view, &target, surface.format(), w, h, placement);
//! core.queue().present(frame);
//! ```

use netrender::{ColorLoad, NetrenderOptions, Renderer, Scene};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, Weak};
#[cfg(test)]
use std::time::Duration;

#[cfg(target_arch = "wasm32")]
mod browser_frame;
#[cfg(target_arch = "wasm32")]
pub use browser_frame::{AcceptedBrowserFrame, BrowserFrameCommands, BrowserFrameStartError, BrowserFrameSubmission, PendingBrowserFrame, RendererAccess};

mod readback;
mod recovery;
pub use recovery::{
    AsyncReplacementFactory, CoreGeneration, DeviceLossMonitor, DeviceLossNotice,
    PendingReplacement, RecoveryStatus, RecoverySubscription, ReplacementCompletionError,
    ReplacementExpiry, ReplacementFactory, ReplacementFuture, SharedRenderCore,
};

/// A rendering refusal on the actual host device. Provider errors do not
/// imply device loss; only the installed monitor can authorize recovery.
#[derive(Clone, Debug)]
pub enum RasterizeFailure {
    DeviceLost(DeviceLossNotice),
    ProviderError(Arc<netrender::VelloError>),
}
impl std::fmt::Display for RasterizeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeviceLost(loss) => write!(f, "rasterization refused: DeviceLost({loss:?})"),
            Self::ProviderError(error) => write!(f, "rasterization refused: {error}"),
        }
    }
}
impl std::error::Error for RasterizeFailure {}

#[cfg(not(target_arch = "wasm32"))]
struct ScopedRasterResult {
    result: Result<(wgpu::Texture, wgpu::TextureView), RasterizeFailure>,
    #[cfg(test)]
    captured_scope_error: Option<Arc<netrender::VelloError>>,
    #[cfg(test)]
    scope_pending: bool,
}

pub enum SurfaceAcquire {
    Frame(wgpu::SurfaceTexture),
    SurfaceLost,
    Unavailable { reason: String },
    DeviceLost(DeviceLossNotice),
}

#[derive(Clone, Copy)]
struct CoreBootRecipe {
    apply_limit_buckets: bool,
    tile_cache_size: Option<u32>,
    enable_vello: bool,
    enable_tile_dirty_overlay: bool,
    tile_dirty_overlay_window_frames: u32,
    backends: Option<wgpu::Backends>,
    optional_features: wgpu::Features,
}
impl CoreBootRecipe {
    fn from_options(o: &NetrenderOptions) -> Self {
        Self {
            apply_limit_buckets: o.apply_limit_buckets,
            tile_cache_size: o.tile_cache_size,
            enable_vello: o.enable_vello,
            enable_tile_dirty_overlay: o.enable_tile_dirty_overlay,
            tile_dirty_overlay_window_frames: o.tile_dirty_overlay_window_frames,
            backends: o.backends,
            optional_features: o.optional_features,
        }
    }
    fn options(self) -> NetrenderOptions {
        NetrenderOptions {
            apply_limit_buckets: self.apply_limit_buckets,
            tile_cache_size: self.tile_cache_size,
            enable_vello: self.enable_vello,
            enable_tile_dirty_overlay: self.enable_tile_dirty_overlay,
            tile_dirty_overlay_window_frames: self.tile_dirty_overlay_window_frames,
            backends: self.backends,
            optional_features: self.optional_features,
        }
    }
}

#[derive(Default)]
struct CaptureMasterCompositor {
    master: Option<wgpu::Texture>,
}

impl netrender::Compositor for CaptureMasterCompositor {
    fn declare_surface(&mut self, _key: netrender::SurfaceKey, _world_bounds: [f32; 4]) {}
    fn destroy_surface(&mut self, _key: netrender::SurfaceKey) {}

    fn present_frame(&mut self, frame: netrender::PresentedFrame<'_>) {
        self.master = Some(frame.master.clone());
    }
}

/// One tightly packed RGBA8 frame read back from the shared render device.
///
/// The host decides whether to encode, digest, or compare the bytes. Keeping the
/// staging-buffer mechanics here lets native windows, browser canvases, and
/// product receipt runners use the same device-level operation.
pub struct RgbaFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// An owned RGBA copy whose mapping is completed on later host turns.
///
/// Polling never waits for the GPU, so a shared render device can keep servicing
/// native window events and presentations while this capture is outstanding.
/// The copy is queued when this value is created; later draws cannot replace
/// the pixels it returns. A host owns its scheduling and deadline.
pub use readback::{PendingRgbaReadback, ReadbackFailure};

impl RgbaFrame {
    /// A stable, order-sensitive FNV-1a digest of the frame bytes.
    pub fn digest(&self) -> u64 {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in &self.rgba {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash
    }

    /// Whether every pixel is fully transparent.
    ///
    /// Opaque black is valid content, so it must not be treated as a failed
    /// receipt merely because every RGB channel is zero.
    pub fn is_blank(&self) -> bool {
        self.rgba.chunks_exact(4).all(|pixel| pixel[3] == 0)
    }
}

#[cfg(test)]
mod rgba_frame_tests {
    use super::RgbaFrame;

    #[test]
    fn transparent_frames_are_blank_but_opaque_black_is_content() {
        let transparent = RgbaFrame {
            width: 1,
            height: 1,
            rgba: vec![12, 34, 56, 0],
        };
        assert!(transparent.is_blank());

        let black = RgbaFrame {
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 255],
        };
        assert!(!black.is_blank());
        assert_ne!(black.digest(), transparent.digest());
    }
}

#[cfg(test)]
mod pending_readback_tests {
    use super::*;
    use std::time::Instant;

    fn write(core: &RenderCore, texture: &wgpu::Texture, width: u32, height: u32, color: [u8; 4]) {
        core.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &color.repeat((width * height) as usize),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    fn collect(pending: &mut PendingRgbaReadback) -> RgbaFrame {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let before = Instant::now();
            let result = pending.poll();
            assert!(
                before.elapsed() < Duration::from_millis(250),
                "a readback poll blocked"
            );
            if let Some(result) = result {
                return result.expect("capture maps successfully");
            }
            assert!(
                Instant::now() < deadline,
                "GPU readback exceeded five seconds"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn pending_capture_keeps_original_rows_while_shared_device_draws_again() {
        // This is an actual adapter test. A boot failure must fail the gate,
        // rather than turn absent GPU evidence into a successful test result.
        let core = RenderCore::boot(NetrenderOptions {
            tile_cache_size: Some(4),
            ..Default::default()
        })
        .expect("GPU adapter is required for the pending readback gate");
        let adapter = core.renderer.wgpu_device.core.adapter.get_info();
        eprintln!(
            "pending readback adapter: {} ({:?}, {:?})",
            adapter.name, adapter.backend, adapter.device_type
        );
        let (width, height) = (67, 3); // Both row padding and multiple rows.
        let texture = core.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("pending RGBA readback regression"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        write(&core, &texture, width, height, [18, 52, 86, 255]);
        let mut first = core.start_rgba8_readback(&texture, width, height).unwrap();
        write(&core, &texture, width, height, [32, 48, 32, 255]);
        let mut second = core.start_rgba8_readback(&texture, width, height).unwrap();
        // Reversing collection order must not alias the first capture's pixels.
        let second_frame = collect(&mut second);
        let first_frame = collect(&mut first);
        assert_eq!(
            first_frame.rgba,
            [18, 52, 86, 255].repeat((width * height) as usize)
        );
        assert_eq!(
            second_frame.rgba,
            [32, 48, 32, 255].repeat((width * height) as usize)
        );
        assert!(!first_frame.is_blank());
        assert_ne!(first_frame.digest(), second_frame.digest());
        assert!(matches!(
            first.poll(),
            Some(Err(ReadbackFailure::AlreadyCollected))
        ));
        // A cancelled receiver must not make a later map callback panic.
        let cancelled = core.start_rgba8_readback(&texture, width, height).unwrap();
        drop(cancelled);
        let frame = core.read_rgba8_texture(&texture, width, height).unwrap();
        assert_eq!(frame.rgba, second_frame.rgba);
        assert!(
            core.start_rgba8_readback(&texture, u32::MAX, height)
                .is_err()
        );
    }
}

#[cfg(test)]
mod ordered_external_texture_tests {
    use super::*;

    const DIM: u32 = 64;

    #[test]
    fn composition_preserves_external_texture_scene_boundary() {
        let core = match RenderCore::boot(NetrenderOptions {
            tile_cache_size: Some(4),
            enable_vello: true,
            ..Default::default()
        }) {
            Ok(core) => core,
            Err(error) => {
                eprintln!("skipping GPU composition receipt: {error}");
                return;
            },
        };
        let source = core.device().create_texture(&wgpu::TextureDescriptor {
            label: Some("ordered external composition source"),
            size: wgpu::Extent3d {
                width: 16,
                height: 16,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        core.queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &source,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &vec![0_u8, 255, 0, 255].repeat(16 * 16),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(16 * 4),
                rows_per_image: Some(16),
            },
            wgpu::Extent3d {
                width: 16,
                height: 16,
                depth_or_array_layers: 1,
            },
        );
        let source_view = source.create_view(&wgpu::TextureViewDescriptor::default());
        let mut scene = Scene::new(DIM, DIM);
        scene.push_rect(0.0, 0.0, DIM as f32, DIM as f32, [1.0, 0.0, 0.0, 1.0]);
        scene.push_rect(24.0, 24.0, 40.0, 40.0, [0.0, 0.0, 1.0, 1.0]);
        let composites = [netrender::ExternalTextureComposite::new(
            &source_view,
            netrender::ExternalTexturePlacement::new([16.0, 16.0, 48.0, 48.0]),
        )
        .with_scene_op_boundary(1)];
        let (texture, _) = core.rasterize_scaled_with_external_textures(
            &scene,
            DIM,
            DIM,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.0,
            &composites,
        );
        let rgba = core
            .read_rgba8_texture(&texture, DIM, DIM)
            .expect("read ordered external composition");
        let pixel = |x: u32, y: u32| {
            let i = ((y * DIM + x) * 4) as usize;
            [
                rgba.rgba[i],
                rgba.rgba[i + 1],
                rgba.rgba[i + 2],
                rgba.rgba[i + 3],
            ]
        };
        assert_eq!(pixel(4, 4), [255, 0, 0, 255]);
        assert_eq!(pixel(20, 20), [0, 255, 0, 255]);
        assert_eq!(pixel(32, 32), [0, 0, 255, 255]);
    }
}

#[cfg(test)]
mod boot_feature_tests {
    use super::*;

    #[test]
    fn optional_features_reach_the_device_when_the_adapter_offers_them() {
        let wanted = wgpu::Features::TIMESTAMP_QUERY;
        let core = match RenderCore::boot(NetrenderOptions {
            optional_features: wanted,
            ..Default::default()
        }) {
            Ok(core) => core,
            Err(error) => {
                eprintln!("skipping GPU feature receipt: {error}");
                return;
            },
        };
        let offered = core
            .renderer()
            .wgpu_device
            .core
            .adapter
            .features()
            .contains(wanted);
        eprintln!("the adapter offers TIMESTAMP_QUERY: {offered}");
        assert_eq!(core.device().features().contains(wanted), offered);
    }
}

/// The shared present core: one wgpu device + netrender [`Renderer`], booted once
/// and shared across **every** surface. Per-target [`WindowSurface`]s are created
/// from it via [`create_surface`](Self::create_surface), so N surfaces present
/// through one device — a node texture rasterized once can be sampled into any
/// surface's swapchain without re-rendering. (Multi-window: one device, N surfaces.)
pub struct RenderCore {
    renderer: Renderer,
    loss: Arc<DeviceLossMonitor>,
    recovery_owner: Mutex<Weak<SharedRenderCore>>,
    recipe: CoreBootRecipe,
    #[cfg(target_arch = "wasm32")]
    browser_frame_gate: Arc<browser_frame::FrameGate>,
}

impl RenderCore {
    /// Boot wgpu + a netrender [`Renderer`] (native blocking). The device is shared;
    /// create per-target surfaces with [`create_surface`](Self::create_surface). On
    /// wasm the WebGPU device request is async, so use [`boot_async`](Self::boot_async).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn boot(options: NetrenderOptions) -> Result<Self, String> {
        // `options.backends` lets a host force a backend (e.g. D3D12 for same-API
        // system-WebView import); `None` honors `WGPU_BACKEND`, else all available.
        // Limit bucketing and optional features are host calls that ride on
        // TenantNeeds, so boot through the shared path even with no tenant:
        // the plain `boot`/`boot_with` pair takes TenantNeeds::default() and
        // would drop whatever the host decided.
        let needs = netrender::TenantNeeds {
            apply_limit_buckets: options.apply_limit_buckets,
            optional_features: options.optional_features,
            ..Default::default()
        };
        let handles = netrender::boot_shared(
            options
                .backends
                // None means honour WGPU_BACKEND, then everything available.
                // Backends::all() alone would silently drop the env override
                // that the plain boot path applies.
                .or_else(wgpu::Backends::from_env)
                .unwrap_or_else(wgpu::Backends::all),
            None,
            &needs,
        )
        .map_err(|e| format!("netrender wgpu boot failed: {e}"))?;
        Self::from_handles(handles, options)
    }

    /// Async boot: awaits netrender's `boot_async`. The only boot path on wasm
    /// (WebGPU device acquisition is asynchronous); works on every target.
    pub async fn boot_async(options: NetrenderOptions) -> Result<Self, String> {
        let needs = netrender::TenantNeeds {
            apply_limit_buckets: options.apply_limit_buckets,
            optional_features: options.optional_features,
            ..Default::default()
        };
        let handles = netrender::boot_async_shared(
            options
                .backends
                // None means honour WGPU_BACKEND, then everything available.
                // Backends::all() alone would silently drop the env override
                // that the plain boot path applies.
                .or_else(wgpu::Backends::from_env)
                .unwrap_or_else(wgpu::Backends::all),
            None,
            &needs,
        )
        .await
        .map_err(|e| format!("netrender wgpu boot failed: {e}"))?;
        Self::from_handles(handles, options)
    }

    fn from_handles(
        handles: netrender::WgpuHandles,
        options: NetrenderOptions,
    ) -> Result<Self, String> {
        let recipe = CoreBootRecipe::from_options(&options);
        let renderer = netrender::create_netrender_instance(handles, options)
            .map_err(|e| format!("netrender init failed: {e:?}"))?;
        let generation = recovery::next_generation();
        let loss = Arc::new(DeviceLossMonitor::new(generation));
        let callback = loss.clone();
        renderer
            .wgpu_device
            .core
            .device
            .set_device_lost_callback(move |reason, message| {
                callback.notify_loss(DeviceLossNotice {
                    generation,
                    reason,
                    message,
                });
            });
        Ok(Self {
            renderer,
            loss,
            recovery_owner: Mutex::new(Weak::new()),
            recipe,
            #[cfg(target_arch = "wasm32")]
            browser_frame_gate: Arc::new(browser_frame::FrameGate::default()),
        })
    }

    pub fn generation(&self) -> CoreGeneration {
        self.loss.generation
    }
    pub fn loss_notice(&self) -> Option<DeviceLossNotice> {
        self.loss.notice()
    }
    pub fn device_failure(&self) -> Option<DeviceLossNotice> {
        self.loss.pending_notice()
    }
    pub fn shared_recovery(self: &Arc<Self>) -> Arc<SharedRenderCore> {
        let recipe = self.recipe;
        #[cfg(not(target_arch = "wasm32"))]
        {
            SharedRenderCore::with_factory(
                self.clone(),
                Arc::new(move || Self::boot(recipe.options()).map(Arc::new)),
            )
        }
        #[cfg(target_arch = "wasm32")]
        {
            SharedRenderCore::with_async_factory(
                self.clone(),
                Arc::new(move || {
                    Box::pin(async move { Self::boot_async(recipe.options()).await.map(Arc::new) })
                }),
            )
        }
    }

    /// Create + configure a swapchain surface for `target` at `(width, height)`,
    /// sharing this core's device. Prefers a non-sRGB format, else the first
    /// advertised. The surface is created from the core's retained wgpu instance,
    /// so every surface draws through the one device.
    ///
    /// `target` is anything wgpu turns into a surface target: a winit
    /// `Arc<Window>`, or `wgpu::SurfaceTarget::Canvas(HtmlCanvasElement)` in a
    /// browser. That is the whole of what used to tie this to a windowing library.
    pub fn create_surface(
        &self,
        target: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Result<WindowSurface, String> {
        if let Some(loss) = self.device_failure() {
            return Err(format!("surface creation refused: DeviceLost({loss:?})"));
        }
        self.create_surface_with_transparency(target, width, height, false)
    }

    /// Create and configure a presentation surface, requiring composited alpha
    /// when `transparent` is true.
    ///
    /// Ordinary browser canvases and native windows stay on the historical
    /// opaque path. An app-drawn native frame with transparent shadow margins
    /// opts in explicitly, so a backend that cannot composite alpha fails at
    /// surface creation instead of presenting black margins as a false shadow.
    pub fn create_surface_with_transparency(
        &self,
        target: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
        transparent: bool,
    ) -> Result<WindowSurface, String> {
        if let Some(loss) = self.device_failure() {
            return Err(format!("surface creation refused: DeviceLost({loss:?})"));
        }
        let core = &self.renderer.wgpu_device.core;
        let surface = core
            .instance
            .create_surface(target)
            .map_err(|e| format!("create_surface failed: {e}"))?;
        let caps = surface.get_capabilities(&core.adapter);
        if caps.formats.is_empty() || caps.alpha_modes.is_empty() {
            return Err("adapter offers no compatible surface format or alpha mode".into());
        }
        // Prefer the NON-srgb surface format. The vello rasterizer writes
        // display-referred (sRGB-encoded) bytes into the Rgba8Unorm layer
        // textures and the compose pass samples them raw, so an sRGB
        // backbuffer re-encodes and washes everything out (verified against
        // the genet_web_smoke browser receipt, whose canvas surface is
        // non-srgb and renders the same scene correctly; woodshed-genet on
        // the old srgb pick showed the double-encode wash, 2026-07-05).
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let alpha_mode = if transparent {
            [
                wgpu::CompositeAlphaMode::PreMultiplied,
                wgpu::CompositeAlphaMode::PostMultiplied,
                wgpu::CompositeAlphaMode::Inherit,
            ]
            .into_iter()
            .find(|candidate| caps.alpha_modes.contains(candidate))
            .ok_or_else(|| {
                format!(
                    "surface has no composited-alpha mode (reported {:?})",
                    caps.alpha_modes
                )
            })?
        } else {
            caps.alpha_modes[0]
        };
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            // wgpu 30 made surface color space explicit. `Auto` keeps the
            // pre-30 behavior of letting the platform pick.
            color_space: wgpu::SurfaceColorSpace::Auto,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&core.device, &surface_config);
        if let Some(loss) = self.device_failure() {
            return Err(format!(
                "surface configuration refused: DeviceLost({loss:?})"
            ));
        }
        Ok(WindowSurface {
            surface,
            surface_config,
        })
    }

    /// The netrender renderer — call `compose_external_texture` (and friends) on
    /// it to composite rasterized layers onto a surface's backbuffer.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    /// Browser compatibility getter; checked callers use try_renderer.
    /// The access guard holds ownership and refuses another frame's Renderer.
    #[cfg(target_arch = "wasm32")]
    pub fn renderer(&self) -> RendererAccess<'_> {
        self.try_renderer().expect("checked renderer access refused")
    }

    /// Retire one keyed surface's retained tile state after its content source
    /// is replaced in place. The next [`rasterize_for`](Self::rasterize_for)
    /// call rebuilds the surface from the complete current scene.
    pub fn invalidate_surface(&self, surface: u64) -> bool {
        #[cfg(target_arch = "wasm32")]
        let _renderer_access = self.try_renderer().expect("checked renderer access refused");
        self.renderer.invalidate_surface_tiles(surface)
    }

    /// The shared wgpu device backing the renderer.
    pub fn device(&self) -> &wgpu::Device {
        &self.renderer.wgpu_device.core.device
    }

    /// The shared wgpu queue (e.g. for external-texture import).
    pub fn queue(&self) -> &wgpu::Queue {
        &self.renderer.wgpu_device.core.queue
    }

    /// Read a `COPY_SRC` RGBA8 texture into tightly packed host memory.
    ///
    /// Swapchain textures are intentionally not assumed to be copyable. A host
    /// that needs a receipt composes its final frame into an owned RGBA8 target,
    /// calls this method, then presents that same target.
    pub fn read_rgba8_texture(
        &self,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<RgbaFrame, String> {
        let mut pending = self.start_rgba8_readback(texture, width, height)?;
        // Compatibility for offscreen/CLI callers. Event-loop hosts should use
        // start_rgba8_readback and poll it on later turns instead of waiting.
        pending
            .wait_for_completion()
            .map_err(|error| error.to_string())
    }

    /// Queue an RGBA8 copy and mapping without waiting for GPU completion.
    /// Native event-loop hosts must use this path: blocking after presentation
    /// can prevent the platform from servicing other windows on the same device.
    pub fn start_rgba8_readback(
        &self,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
    ) -> Result<PendingRgbaReadback, String> {
        let _operation = self
            .loss
            .begin_operation()
            .map_err(|loss| format!("readback refused: DeviceLost({loss:?})"))?;
        let width = width.max(1);
        let height = height.max(1);
        let unpadded = width
            .checked_mul(4)
            .ok_or("RGBA readback row size overflow")?;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = unpadded
            .div_ceil(align)
            .checked_mul(align)
            .ok_or("RGBA readback aligned row size overflow")?;
        let buffer = self.device().create_buffer(&wgpu::BufferDescriptor {
            label: Some("genet render host RGBA readback"),
            size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("genet render host RGBA readback"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let submission = self.queue().submit(Some(encoder.finish()));
        let (send, mapped) = mpsc::channel();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                // Cancellation drops the receiver; a late callback is harmless.
                let _ = send.send(result);
            });
        Ok(PendingRgbaReadback::new(
            self.device().clone(),
            buffer,
            mapped,
            submission,
            width,
            height,
            padded,
            &self.loss,
        ))
    }

    /// Rasterize `scene` into a fresh `(w, h)` `Rgba8Unorm` texture, cleared to
    /// `clear`. Returns the texture with its view; keep the texture alive until
    /// the composite pass has sampled the view. Device-only, so any surface's frame
    /// can composite the result.
    pub fn rasterize(
        &self,
        scene: &Scene,
        w: u32,
        h: u32,
        clear: ColorLoad,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        self.rasterize_scaled(scene, w, h, clear, 1.0)
    }

    /// Like [`rasterize`](Self::rasterize) but with a per-SURFACE tile cache:
    /// `surface` names one retained surface (a shell partition, a canvas, one
    /// content card) with any stable host-chosen id. A host that rasterizes
    /// several surfaces through one core MUST key them — through the unkeyed
    /// entry each render diffs its scene against whichever surface rendered
    /// last, so every tile is dirty on every call (P4, shell paint plan
    /// 2026-07-03: 234/234 tiles rebuilt per settled frame).
    pub fn rasterize_for(
        &self,
        surface: u64,
        scene: &Scene,
        w: u32,
        h: u32,
        clear: ColorLoad,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        self.rasterize_scaled_for(surface, scene, w, h, clear, 1.0)
    }

    /// [`rasterize_scaled`](Self::rasterize_scaled) with a per-surface tile
    /// cache — see [`rasterize_for`](Self::rasterize_for).
    pub fn rasterize_scaled_for(
        &self,
        surface: u64,
        scene: &Scene,
        w: u32,
        h: u32,
        clear: ColorLoad,
        scale: f32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        #[cfg(target_arch = "wasm32")]
        let _renderer_access = self.try_renderer().expect("checked renderer access refused");
        let (tex, view) = self.scene_target(w, h);
        self.renderer
            .render_vello_scaled_for(surface, scene, &view, clear, scale);
        self.log_raster_spans(w, h);
        (tex, view)
    }

    /// Checked host drawing keeps provider refusals distinct from an actual
    /// device loss. Creation/submission participates in loss admission, and
    /// the monitor is checked again before any output is returned.
    pub fn try_rasterize_scaled_for(
        &self,
        surface: u64,
        scene: &Scene,
        w: u32,
        h: u32,
        clear: ColorLoad,
        scale: f32,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), RasterizeFailure> {
        self.rasterize_scaled_for_admitted(surface, scene, w, h, clear, scale, || {})
    }

    // Hooks are private per-call seams. Production passes no-ops; no mutable
    // global injection state exists.
    #[allow(clippy::too_many_arguments)]
    fn rasterize_scaled_for_admitted(
        &self,
        surface: u64,
        scene: &Scene,
        w: u32,
        h: u32,
        clear: ColorLoad,
        scale: f32,
        after_admission: impl FnOnce(),
    ) -> Result<(wgpu::Texture, wgpu::TextureView), RasterizeFailure> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.rasterize_scaled_for_scoped(
                surface,
                scene,
                w,
                h,
                clear,
                scale,
                after_admission,
                || {},
            )
            .result
        }
        #[cfg(target_arch = "wasm32")]
        {
            // WebGPU scope results and current-frame quality completion are
            // asynchronous. This existing synchronous API does not qualify
            // either; an async host seam remains open. Do not block JS or
            // drop a promise and claim native completion guarantees here.
            let _renderer_access = self.try_renderer()?;
            let _operation = self
                .loss
                .begin_operation()
                .map_err(RasterizeFailure::DeviceLost)?;
            let (texture, view) = self.scene_target(w, h);
            after_admission();
            if let Some(loss) = self.device_failure() {
                return Err(RasterizeFailure::DeviceLost(loss));
            }
            let rendered = self
                .renderer
                .render_vello_scaled_for_checked(surface, scene, &view, clear, scale);
            if let Some(loss) = self.device_failure() {
                return Err(RasterizeFailure::DeviceLost(loss));
            }
            rendered.map_err(|error| RasterizeFailure::ProviderError(Arc::new(error)))?;
            self.log_raster_spans(w, h);
            Ok((texture, view))
        }
    }

    /// Native target creation and checked raster submission are covered by
    /// device error scopes. This does not scope host composition, presentation,
    /// producer upload/staging, or browser asynchronous validation.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::too_many_arguments)]
    fn rasterize_scaled_for_scoped(
        &self,
        surface: u64,
        scene: &Scene,
        w: u32,
        h: u32,
        clear: ColorLoad,
        scale: f32,
        after_admission: impl FnOnce(),
        before_renderer: impl FnOnce(),
    ) -> ScopedRasterResult {
        let _operation = match self.loss.begin_operation() {
            Ok(operation) => operation,
            Err(loss) => {
                return ScopedRasterResult {
                    result: Err(RasterizeFailure::DeviceLost(loss)),
                    #[cfg(test)]
                    captured_scope_error: None,
                    #[cfg(test)]
                    scope_pending: false,
                };
            },
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        // wgpu 30 scopes are thread-local and must pop in reverse order.
        let out_of_memory = self
            .device()
            .push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let internal = self.device().push_error_scope(wgpu::ErrorFilter::Internal);
        let validation = self
            .device()
            .push_error_scope(wgpu::ErrorFilter::Validation);
        let rendered = (|| {
            let (texture, view) = self.scene_target(w, h);
            after_admission();
            if let Some(loss) = self.device_failure() {
                return Err(RasterizeFailure::DeviceLost(loss));
            }
            // Regression destroys a device after the final check, then lets
            // the actual renderer touch its buffers inside these real scopes.
            before_renderer();
            self.renderer
                .render_vello_scaled_for_checked(surface, scene, &view, clear, scale)
                .map_err(|error| RasterizeFailure::ProviderError(Arc::new(error)))?;
            self.qualify_native_raster(deadline)?;
            Ok((texture, view))
        })();
        let mut scoped_error = None;
        let mut scope_pending = false;
        for scope in [validation, internal, out_of_memory] {
            use std::future::Future as _;
            let mut future = Box::pin(scope.pop());
            let mut context = std::task::Context::from_waker(std::task::Waker::noop());
            // Native wgpu-core currently returns ready(scope.error). A future
            // backend that returns Pending is a typed refusal, never success.
            match future.as_mut().poll(&mut context) {
                std::task::Poll::Ready(Some(error)) => {
                    if scoped_error.is_none() {
                        scoped_error =
                            Some(Arc::new(netrender::VelloError::WgpuErrorFromScope(error)));
                    }
                },
                std::task::Poll::Ready(None) => {},
                std::task::Poll::Pending => scope_pending = true,
            }
        }
        let result = if let Some(loss) = self.device_failure() {
            Err(RasterizeFailure::DeviceLost(loss))
        } else if let Some(error) = &scoped_error {
            Err(RasterizeFailure::ProviderError(error.clone()))
        } else if scope_pending {
            Err(RasterizeFailure::ProviderError(Arc::new(
                netrender::VelloError::WgpuErrorFromScope(wgpu::Error::Internal {
                    source: Box::new(std::io::Error::other(
                        "native GPU error scope did not resolve synchronously",
                    )),
                    description: "native GPU error scope outcome is unavailable".into(),
                }),
            )))
        } else if rendered.is_ok() && std::time::Instant::now() >= deadline {
            Err(Self::native_raster_unavailable(
                "native Classic quality completion exceeded five seconds",
            ))
        } else {
            rendered
        };
        if result.is_ok() {
            self.log_raster_spans(w, h);
        }
        ScopedRasterResult {
            result,
            #[cfg(test)]
            captured_scope_error: scoped_error,
            #[cfg(test)]
            scope_pending,
        }
    }

    // An API-error-free submission is not yet a qualified Classic frame.
    // Observe its shader counters while scopes/admission still protect the
    // request. Native queue waits are finite; WebGPU must use an async seam.
    #[cfg(not(target_arch = "wasm32"))]
    fn native_raster_unavailable(description: impl Into<String>) -> RasterizeFailure {
        let description = description.into();
        RasterizeFailure::ProviderError(Arc::new(netrender::VelloError::WgpuErrorFromScope(
            wgpu::Error::Internal {
                source: Box::new(std::io::Error::other(description.clone())),
                description,
            },
        )))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn qualify_native_raster(&self, deadline: std::time::Instant) -> Result<(), RasterizeFailure> {
        loop {
            if let Some(loss) = self.device_failure() {
                return Err(RasterizeFailure::DeviceLost(loss));
            }
            if std::time::Instant::now() >= deadline {
                return Err(Self::native_raster_unavailable(
                    "native Classic quality completion exceeded five seconds",
                ));
            }
            match self.renderer.poll_vello_curve_quality() {
                Ok(Some((0, 0))) => {
                    if std::time::Instant::now() >= deadline {
                        return Err(Self::native_raster_unavailable(
                            "native Classic quality completion exceeded five seconds",
                        ));
                    }
                    return Ok(());
                },
                Ok(Some(_)) => {},
                Ok(None) => {
                    return Err(Self::native_raster_unavailable(
                        "native Classic quality observer is unavailable",
                    ));
                },
                Err(error) => return Err(RasterizeFailure::ProviderError(Arc::new(error))),
            }
            let remaining = deadline
                .checked_duration_since(std::time::Instant::now())
                .filter(|remaining| !remaining.is_zero())
                .ok_or_else(|| {
                    Self::native_raster_unavailable(
                        "native Classic quality completion exceeded five seconds",
                    )
                })?;
            if let Err(error) = self.device().poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(remaining),
            }) {
                if let Some(loss) = self.device_failure() {
                    return Err(RasterizeFailure::DeviceLost(loss));
                }
                return Err(Self::native_raster_unavailable(format!(
                    "native Classic quality completion failed: {error}"
                )));
            }
            // A queue completion may precede delivery of a mapping callback.
            // Yield before observing again; the shared deadline bounds retries.
            std::thread::yield_now();
        }
    }

    /// Like [`rasterize`](Self::rasterize) but for a scene laid out in **logical**
    /// (DIP) coordinates: the texture is the physical `(w, h)` and `scale` is the
    /// device-pixel-ratio, so the logical scene is rasterized crisply at physical
    /// resolution (the scene's viewport must be `(w/scale, h/scale)`). The
    /// device-pixel-ratio path for HiDPI content tiles. (Auto-DPI D2.)
    pub fn rasterize_scaled(
        &self,
        scene: &Scene,
        w: u32,
        h: u32,
        clear: ColorLoad,
        scale: f32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        #[cfg(target_arch = "wasm32")]
        let _renderer_access = self.try_renderer().expect("checked renderer access refused");
        let (tex, view) = self.scene_target(w, h);
        self.renderer
            .render_vello_scaled(scene, &view, clear, scale);
        self.log_raster_spans(w, h);
        (tex, view)
    }

    /// Rasterize a logical-coordinate scene with caller-owned same-device
    /// textures inserted at their emitted scene-operation boundaries.
    ///
    /// The current NetRender contract does not reconstruct ancestor transforms,
    /// clips, or opacity groups around an external producer. It is exact for
    /// plain canvases among opaque scene operations; richer stacking requires a
    /// renderer contract extension.
    pub fn rasterize_scaled_with_external_textures(
        &self,
        scene: &Scene,
        w: u32,
        h: u32,
        clear: ColorLoad,
        _scale: f32,
        external_textures: &[netrender::ExternalTextureComposite<'_>],
    ) -> (wgpu::Texture, wgpu::TextureView) {
        #[cfg(target_arch = "wasm32")]
        let _renderer_access = self.try_renderer().expect("checked renderer access refused");
        let mut compositor = CaptureMasterCompositor::default();
        let base_color = match clear {
            ColorLoad::Clear(color) => netrender::peniko::Color::new([
                color.r as f32,
                color.g as f32,
                color.b as f32,
                color.a as f32,
            ]),
            ColorLoad::Load => netrender::peniko::Color::new([0.0, 0.0, 0.0, 0.0]),
        };
        self.renderer.render_with_compositor_and_external_textures(
            scene,
            wgpu::TextureFormat::Rgba8Unorm,
            &mut compositor,
            base_color,
            external_textures,
        );
        let master = compositor
            .master
            .expect("NetRender ordered rasterization must present a master texture");
        let source = master.create_view(&wgpu::TextureViewDescriptor::default());
        let (texture, view) = self.scene_target(w, h);
        self.renderer.compose_external_texture(
            &source,
            &view,
            wgpu::TextureFormat::Rgba8Unorm,
            w,
            h,
            netrender::ExternalTexturePlacement::new([0.0, 0.0, w as f32, h as f32]),
        );
        self.log_raster_spans(w, h);
        (texture, view)
    }

    fn scene_target(&self, w: u32, h: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let device = self.device();
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("genet-render-host scene"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });
        let view = tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("genet-render-host scene view"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            ..Default::default()
        });
        (tex, view)
    }

    /// P3 raster attribution (shell paint plan 2026-07-03): surface the
    /// rasterizer's per-phase spans + dirty-tile count per render call, so a
    /// frame's raster cost decomposes without a debugger. Debug level; an idle
    /// app renders no frames, so there is no idle log cost.
    fn log_raster_spans(&self, w: u32, h: u32) {
        if tracing::enabled!(target: "genet_render_host::raster", tracing::Level::DEBUG) {
            if let Some(t) = self.renderer.last_frame_timings() {
                let us = |n: &str| t.span(n).map(|d| d.as_micros() as u64).unwrap_or(0);
                tracing::debug!(
                    target: "genet_render_host::raster",
                    w,
                    h,
                    total_us = t.total.as_micros() as u64,
                    tile_invalidate_us = us("tile_invalidate"),
                    dirty_tile_rebuild_us = us("dirty_tile_rebuild"),
                    master_compose_us = us("master_compose"),
                    vello_render_us = us("vello_render"),
                    dirty_tiles = self.renderer.vello_last_dirty_count().unwrap_or(0),
                    "raster spans"
                );
            }
        }
    }
}

/// One target's swapchain surface + its configuration, created from a shared
/// [`RenderCore`]. Per-target; the device behind it is the core's, so the methods
/// that touch the device ([`resize`](Self::resize) / [`acquire`](Self::acquire))
/// take the `&RenderCore` back. (One device, N surfaces.)
pub struct WindowSurface {
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
}

impl WindowSurface {
    /// The surface's texture format (pass to `compose_external_texture`).
    pub fn format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    /// Reconfigure the surface for a new size (clamped to ≥ 1), via the shared
    /// core's device.
    pub fn resize(&mut self, core: &RenderCore, width: u32, height: u32) {
        if core.device_failure().is_some() {
            return;
        }
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(core.device(), &self.surface_config);
    }

    /// Acquire this target's backbuffer for the frame. Returns `None` (and
    /// reconfigures via the shared core's device) when the surface is outdated /
    /// lost or otherwise unavailable, so the caller simply skips the frame. Stays
    /// non-blocking so a slow window never stalls another on the shared loop.
    pub fn acquire(&self, core: &RenderCore) -> Option<wgpu::SurfaceTexture> {
        match self.acquire_status(core) {
            SurfaceAcquire::Frame(frame) => Some(frame),
            _ => None,
        }
    }
    pub fn acquire_status(&self, core: &RenderCore) -> SurfaceAcquire {
        if let Some(loss) = core.device_failure() {
            return SurfaceAcquire::DeviceLost(loss);
        }
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => SurfaceAcquire::Frame(frame),
            wgpu::CurrentSurfaceTexture::Outdated => {
                if let Some(loss) = core.device_failure() {
                    return SurfaceAcquire::DeviceLost(loss);
                }
                self.surface.configure(core.device(), &self.surface_config);
                SurfaceAcquire::Unavailable {
                    reason: "surface outdated; reconfigured same device".into(),
                }
            },
            wgpu::CurrentSurfaceTexture::Lost => {
                if let Some(loss) = core.device_failure() {
                    return SurfaceAcquire::DeviceLost(loss);
                }
                SurfaceAcquire::SurfaceLost
            },
            other => {
                if let Some(loss) = core.device_failure() {
                    return SurfaceAcquire::DeviceLost(loss);
                }
                SurfaceAcquire::Unavailable {
                    reason: format!("surface acquire skipped: {other:?}"),
                }
            },
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod admitted_raster_tests {
    use super::*;
    #[test]
    #[ignore = "actual device destruction; coordinator executes serialized GPU gate"]
    fn actual_device_destroy_after_raster_admission_is_typed_and_does_not_panic() {
        let core = RenderCore::boot(NetrenderOptions {
            tile_cache_size: Some(8),
            enable_vello: true,
            ..Default::default()
        })
        .expect("an actual adapter is required for this ignored gate");
        let scene = Scene::new(32, 24);
        let result = core.rasterize_scaled_for_admitted(
            0x6164_6d69_7373,
            &scene,
            32,
            24,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.0,
            || {
                core.device().destroy();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                while core.device_failure().is_none() {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "actual loss callback did not arrive within five seconds"
                    );
                    let _ = core.device().poll(wgpu::PollType::Poll);
                    std::thread::yield_now();
                }
                assert!(
                    core.loss_notice().is_none(),
                    "the admitted raster still prevents replacement readiness"
                );
            },
        );
        match result {
            Err(RasterizeFailure::DeviceLost(loss)) => {
                assert_eq!(loss, core.loss_notice().expect("admission has now drained"));
                assert_eq!(loss.reason, wgpu::DeviceLostReason::Destroyed);
            },
            _ => panic!("mid-admission destruction must refuse the frame without unwinding"),
        }
        assert!(matches!(
            core.try_rasterize_scaled_for(
                0x6164_6d69_7373,
                &scene,
                32,
                24,
                ColorLoad::Clear(wgpu::Color::TRANSPARENT),
                1.0,
            ),
            Err(RasterizeFailure::DeviceLost(_))
        ));
    }
    #[test]
    #[ignore = "actual healthy-device curve refusal; coordinator executes serialized GPU gate"]
    fn finite_huge_curve_returns_provider_error_and_valid_raster_recovers_on_same_core() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::time::Instant;
        let core = Arc::new(
            RenderCore::boot(NetrenderOptions {
                tile_cache_size: Some(8),
                enable_vello: true,
                ..Default::default()
            })
            .expect("an actual adapter is required for this ignored gate"),
        );
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let owner = SharedRenderCore::with_factory(
            core.clone(),
            Arc::new(move || {
                counted.fetch_add(1, Ordering::SeqCst);
                Err("healthy provider refusal must never request replacement".into())
            }),
        );
        let generation = core.generation();
        // All GPU/readback work after boot shares one finite deadline. Large
        // coordinates remain a four-command CPU path and a 32x24 target;
        // the actual estimate must refuse before allocating curve scratch.
        let deadline = Instant::now() + Duration::from_secs(5);
        fn pixels(core: &RenderCore, texture: &wgpu::Texture, deadline: Instant) -> RgbaFrame {
            let mut pending = core.start_rgba8_readback(texture, 32, 24).unwrap();
            loop {
                assert!(
                    Instant::now() < deadline,
                    "healthy-refusal gate exceeded five seconds"
                );
                if let Some(frame) = pending.poll() {
                    return frame.unwrap();
                }
                std::thread::yield_now();
            }
        }
        let mut valid = Scene::new(32, 24);
        valid.push_rect(0.0, 0.0, 32.0, 24.0, [0.25, 0.5, 0.75, 1.0]);
        let (baseline_texture, _) = core
            .try_rasterize_scaled_for(
                0x6865_616c_7468,
                &valid,
                32,
                24,
                ColorLoad::Clear(wgpu::Color::TRANSPARENT),
                1.0,
            )
            .unwrap();
        let baseline = pixels(&core, &baseline_texture, deadline);
        assert!(!baseline.is_blank());
        while core.renderer().poll_vello_curve_quality().unwrap() != Some((0, 0)) {
            assert!(
                Instant::now() < deadline,
                "baseline quality observation exceeded five seconds"
            );
            std::thread::yield_now();
        }
        let mut huge = Scene::new(32, 24);
        let mut path = netrender::scene::ScenePath::new();
        path.move_to(0.0, 0.0)
            .cubic_to(0.0, 1e30, 1e30, 1e30, 31.0, 0.0)
            .line_to(0.0, 0.0)
            .close();
        huge.push_shape_filled(path, [1.0, 0.0, 0.0, 1.0]);
        match core.try_rasterize_scaled_for(
            0x6865_616c_7468,
            &huge,
            32,
            24,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.0,
        ) {
            Err(RasterizeFailure::ProviderError(error)) => {
                assert!(
                    matches!(&*error, netrender::VelloError::CurveEstimateLimit { lines, limit }
                    if *lines as usize > *limit)
                );
            },
            _ => panic!("the real finite curve estimate must return its typed provider refusal"),
        }
        assert_eq!(
            core.renderer().vello_curve_quality_status(),
            Some((0, 0)),
            "estimate refusal must not submit a huge frame or allocate its quality counters"
        );
        assert!(core.device_failure().is_none());
        assert_eq!(core.generation(), generation);
        assert!(Arc::ptr_eq(&owner.current().unwrap(), &core));
        assert!(
            matches!(owner.status(), RecoveryStatus::Ready { generation: actual } if actual == generation)
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let (after_texture, _) = core
            .try_rasterize_scaled_for(
                0x6865_616c_7468,
                &valid,
                32,
                24,
                ColorLoad::Clear(wgpu::Color::TRANSPARENT),
                1.0,
            )
            .unwrap();
        assert_eq!(pixels(&core, &after_texture, deadline).rgba, baseline.rgba);

        // Also verify the shared-cache checked API's actual take/put store.
        // A propagated error must restore tile scenes before releasing locks.
        let (shared_texture, shared_view) = core.scene_target(32, 24);
        let shared_error = core
            .renderer()
            .render_vello_scaled_checked(
                &huge,
                &shared_view,
                ColorLoad::Clear(wgpu::Color::TRANSPARENT),
                1.0,
            )
            .unwrap_err();
        assert!(matches!(
            shared_error,
            netrender::VelloError::CurveEstimateLimit { .. }
        ));
        assert!(
            core.renderer().vello_cached_tile_count().unwrap() > 0,
            "the shared tile-scene store must be restored even on provider refusal"
        );
        core.renderer()
            .render_vello_scaled_checked(
                &valid,
                &shared_view,
                ColorLoad::Clear(wgpu::Color::TRANSPARENT),
                1.0,
            )
            .unwrap();
        assert_eq!(pixels(&core, &shared_texture, deadline).rgba, baseline.rgba);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(core.device_failure().is_none());
    }
    #[test]
    #[ignore = "actual destruction after final native precheck; coordinator executes serialized GPU gate"]
    fn native_scope_captures_actual_renderer_errors_after_final_precheck_loss() {
        let core = RenderCore::boot(NetrenderOptions {
            tile_cache_size: Some(8),
            enable_vello: true,
            ..Default::default()
        })
        .expect("an actual adapter is required for this ignored gate");
        let mut scene = Scene::new(32, 24);
        scene.push_rect(0.0, 0.0, 32.0, 24.0, [0.25, 0.5, 0.75, 1.0]);
        let outcome = core.rasterize_scaled_for_scoped(
            0x7363_6f70_6564,
            &scene,
            32,
            24,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.0,
            || {},
            || {
                core.device().destroy();
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                while core.device_failure().is_none() {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "actual loss callback did not arrive within five seconds"
                    );
                    let _ = core.device().poll(wgpu::PollType::Poll);
                    std::thread::yield_now();
                }
                assert!(
                    core.loss_notice().is_none(),
                    "raster admission still owns the generation"
                );
            },
        );
        assert!(
            !outcome.scope_pending,
            "native scope results must be observed synchronously"
        );
        assert!(
            matches!(
                outcome.captured_scope_error.as_deref(),
                Some(netrender::VelloError::WgpuErrorFromScope(_))
            ),
            "the actual invalid renderer buffer calls must be captured by native wgpu scopes"
        );
        match outcome.result {
            Err(RasterizeFailure::DeviceLost(loss)) => {
                assert_eq!(
                    loss,
                    core.loss_notice()
                        .expect("scoped submission admission has drained")
                );
                assert_eq!(loss.reason, wgpu::DeviceLostReason::Destroyed);
            },
            _ => panic!("native scoped loss must return a typed refusal, never an output"),
        }
        assert!(matches!(
            core.try_rasterize_scaled_for(
                0x7363_6f70_6564,
                &scene,
                32,
                24,
                ColorLoad::Clear(wgpu::Color::TRANSPARENT),
                1.0,
            ),
            Err(RasterizeFailure::DeviceLost(_))
        ));
    }
    #[test]
    #[ignore = "actual current-frame shader refusal; coordinator executes serialized GPU gate"]
    fn round_cap_shader_quality_refuses_current_frame_before_checked_raster_success() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let core = Arc::new(
            RenderCore::boot(NetrenderOptions {
                tile_cache_size: Some(32),
                enable_vello: true,
                ..Default::default()
            })
            .expect("an actual adapter is required for this ignored gate"),
        );
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let owner = SharedRenderCore::with_factory(
            core.clone(),
            Arc::new(move || {
                counted.fetch_add(1, Ordering::SeqCst);
                Err("shader quality refusal must not replace a healthy device".into())
            }),
        );
        let generation = core.generation();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut path = netrender::scene::ScenePath::new();
        path.move_to(8.0, 12.0).line_to(24.0, 12.0);
        let mut scene = Scene::new(32, 24);
        scene.push_shape_stroked(path, [0.25, 0.5, 0.75, 1.0], 40_000_000.0);
        let netrender::scene::SceneOp::Shape(shape) = &mut scene.ops[0] else {
            unreachable!()
        };
        shape.stroke.as_mut().unwrap().cap = netrender::scene::SceneStrokeCap::Round;
        // Its conservative estimate is admitted, but the current shader cap
        // requires more than 65,536 arc segments at 1/256 device-pixel error.
        // A preflight CurveEstimateLimit is not this regression's success.
        match core.try_rasterize_scaled_for(
            0x726f_756e_6463,
            &scene,
            32,
            24,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
            1.0,
        ) {
            Err(RasterizeFailure::ProviderError(error)) => {
                assert!(
                    matches!(&*error, netrender::VelloError::CurveFlatteningLimit),
                    "the current submitted frame's actual shader refusal must be observed: {error:?}"
                );
            },
            _ => panic!(
                "the current frame must be refused before the checked API can return a texture"
            ),
        }
        assert!(
            std::time::Instant::now() < deadline,
            "shader-quality gate exceeded five seconds"
        );
        assert_eq!(
            core.renderer().vello_curve_quality_status(),
            Some((0, 0)),
            "the single rejected frame's counter buffer must be retired"
        );
        assert!(core.device_failure().is_none());
        assert_eq!(core.generation(), generation);
        assert!(Arc::ptr_eq(&owner.current().unwrap(), &core));
        assert!(
            matches!(owner.status(), RecoveryStatus::Ready { generation: actual } if actual == generation)
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let mut valid = Scene::new(32, 24);
        valid.push_rect(0.0, 0.0, 32.0, 24.0, [0.25, 0.5, 0.75, 1.0]);
        let (texture, _) = core
            .try_rasterize_scaled_for(
                0x726f_756e_6463,
                &valid,
                32,
                24,
                ColorLoad::Clear(wgpu::Color::TRANSPARENT),
                1.0,
            )
            .unwrap();
        assert_eq!(
            core.renderer().vello_curve_quality_status(),
            Some((0, 0)),
            "a successful native checked raster must already have qualified its own counters"
        );
        let mut pixels = core.start_rgba8_readback(&texture, 32, 24).unwrap();
        loop {
            assert!(
                std::time::Instant::now() < deadline,
                "shader-quality/readback gate exceeded five seconds"
            );
            if let Some(frame) = pixels.poll() {
                assert!(!frame.unwrap().is_blank());
                break;
            }
            std::thread::yield_now();
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
