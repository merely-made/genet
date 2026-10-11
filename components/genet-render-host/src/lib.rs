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
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::Duration;

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
pub struct PendingRgbaReadback {
    device: wgpu::Device,
    buffer: wgpu::Buffer,
    mapped: Receiver<Result<(), wgpu::BufferAsyncError>>,
    submission: wgpu::SubmissionIndex,
    width: u32,
    height: u32,
    padded: u32,
    completed: bool,
}

impl PendingRgbaReadback {
    /// Service completed GPU work and return the captured pixels once mapped.
    /// `None` means the host should yield and poll on a later turn.
    pub fn poll(&mut self) -> Option<Result<RgbaFrame, String>> {
        if self.completed {
            return Some(Err("RGBA readback was already collected".into()));
        }
        if let Err(error) = self.device.poll(wgpu::PollType::Poll) {
            self.completed = true;
            return Some(Err(format!("RGBA readback poll failed: {error}")));
        }
        self.collect()
    }

    fn collect(&mut self) -> Option<Result<RgbaFrame, String>> {
        match self.mapped.try_recv() {
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                self.completed = true;
                return Some(Err("RGBA readback map callback disconnected".into()));
            },
            Ok(Err(error)) => {
                self.completed = true;
                return Some(Err(format!("RGBA readback map failed: {error}")));
            },
            Ok(Ok(())) => {},
        }
        self.completed = true;
        Some(self.copy_mapped_rows())
    }

    fn copy_mapped_rows(&self) -> Result<RgbaFrame, String> {
        let slice = self.buffer.slice(..);
        let data = slice
            .get_mapped_range()
            .map_err(|error| format!("RGBA readback map failed: {error}"))?;
        let unpadded = self.width as usize * 4;
        let mut rgba = Vec::with_capacity(unpadded * self.height as usize);
        for row in 0..self.height as usize {
            let start = row * self.padded as usize;
            rgba.extend_from_slice(&data[start..start + unpadded]);
        }
        drop(data);
        self.buffer.unmap();
        Ok(RgbaFrame {
            width: self.width,
            height: self.height,
            rgba,
        })
    }
}

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
        assert!(matches!(first.poll(), Some(Err(error)) if error.contains("already collected")));
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
        let renderer = netrender::create_netrender_instance(handles, options)
            .map_err(|e| format!("netrender init failed: {e:?}"))?;
        Ok(Self { renderer })
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
        let core = &self.renderer.wgpu_device.core;
        let surface = core
            .instance
            .create_surface(target)
            .map_err(|e| format!("create_surface failed: {e}"))?;
        let caps = surface.get_capabilities(&core.adapter);
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
        Ok(WindowSurface {
            surface,
            surface_config,
        })
    }

    /// The netrender renderer — call `compose_external_texture` (and friends) on
    /// it to composite rasterized layers onto a surface's backbuffer.
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    /// Retire one keyed surface's retained tile state after its content source
    /// is replaced in place. The next [`rasterize_for`](Self::rasterize_for)
    /// call rebuilds the surface from the complete current scene.
    pub fn invalidate_surface(&self, surface: u64) -> bool {
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
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(pending.submission.clone()),
                timeout: Some(Duration::from_secs(5)),
            })
            .map_err(|error| format!("RGBA readback poll failed: {error}"))?;
        pending
            .collect()
            .unwrap_or_else(|| Err("RGBA readback mapping did not complete".into()))
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
        Ok(PendingRgbaReadback {
            device: self.device().clone(),
            buffer,
            mapped,
            submission,
            width,
            height,
            padded,
            completed: false,
        })
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
        let (tex, view) = self.scene_target(w, h);
        self.renderer
            .render_vello_scaled_for(surface, scene, &view, clear, scale);
        self.log_raster_spans(w, h);
        (tex, view)
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
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.surface.configure(core.device(), &self.surface_config);
    }

    /// Acquire this target's backbuffer for the frame. Returns `None` (and
    /// reconfigures via the shared core's device) when the surface is outdated /
    /// lost or otherwise unavailable, so the caller simply skips the frame. Stays
    /// non-blocking so a slow window never stalls another on the shared loop.
    pub fn acquire(&self, core: &RenderCore) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => Some(frame),
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.surface.configure(core.device(), &self.surface_config);
                None
            },
            other => {
                eprintln!("[genet-render-host] surface acquire skipped: {other:?}");
                None
            },
        }
    }
}
