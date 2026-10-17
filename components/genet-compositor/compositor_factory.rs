/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(unsafe_code)]

//! Per-platform default-compositor factory.
//!
//! Embedders typically want "give me the right OS-handoff backend
//! for this platform, given my window handle." This module provides
//! that one-stop construction:
//!
//! ```ignore
//! use paint::{HostWgpuContext, default_compositor_for_window};
//!
//! let host = HostWgpuContext::new(device, queue);
//! let compositor = default_compositor_for_window(
//!     host,
//!     display_handle,
//!     window_handle,
//! )?;
//! paint.install_compositor(compositor);
//! ```
//!
//! The factory dispatches on `cfg`:
//!
//! - `target_os = "windows"` → [`crate::WindowsDxgiBackend`] wrapped
//!   in [`crate::ServoCompositor`].
//! - `target_vendor = "apple"` → [`crate::MacosCALayerBackend`].
//! - `target_os = "linux"` → [`crate::WaylandSubsurfaceBackend`].
//! - otherwise → [`crate::WgpuMasterCaptureBackend`] (no OS handoff
//!   available; embedder reads composite back via
//!   `Paint::composite_texture`).
//!
//! Each per-platform backend's construction can fail (wrong backend
//! detected on `host`, null window handle, OS API failure). The
//! factory propagates those errors as `BoxedFactoryError`. Embedders
//! that want "platform if available, else fall back to capture"
//! should call [`default_compositor_for_window_or_capture`] instead
//! — that variant returns a typed selection retaining the native
//! construction error when capture is chosen.

use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::compositor::{PaintCompositor, PresentationRoute, WgpuMasterCaptureBackend};
use crate::interop::HostWgpuContext;

/// Type-erased factory error. Per-platform backends return their own
/// `BackendError` types; this wraps them so the cfg-dispatched
/// factory has one return shape.
pub type BoxedFactoryError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// The route selected by the factory. A capture fallback has pixels for an
/// embedder, but must never be counted as an OS display handoff.
pub enum CompositorSelection {
    Display(Box<dyn PaintCompositor>),
    CaptureFallback {
        compositor: Box<dyn PaintCompositor>,
        /// `None` means this target has no OS handoff backend. `Some` is the
        /// construction error that caused a platform backend to fall back.
        cause: Option<BoxedFactoryError>,
    },
}

impl CompositorSelection {
    pub fn compositor_mut(&mut self) -> &mut dyn PaintCompositor {
        match self {
            Self::Display(compositor) | Self::CaptureFallback { compositor, .. } => {
                compositor.as_mut()
            },
        }
    }

    pub fn into_compositor(self) -> Box<dyn PaintCompositor> {
        match self {
            Self::Display(compositor) | Self::CaptureFallback { compositor, .. } => compositor,
        }
    }
}

/// Construct the platform-default OS-handoff compositor.
///
/// Returns `Err` if the per-platform backend's construction fails
/// (wrong wgpu backend, null handle, OS API error, etc.). On
/// platforms with no OS-handoff backend, returns
/// [`WgpuMasterCaptureBackend`] (never errors — that's a fallback
/// in its own right).
///
/// `_display` is required by Wayland (needs `wl_display`) and
/// ignored on Windows / macOS — pass it from the embedder
/// regardless to keep call sites portable.
pub fn default_compositor_for_window(
    host: HostWgpuContext,
    _display: RawDisplayHandle,
    window: RawWindowHandle,
) -> Result<Box<dyn PaintCompositor>, BoxedFactoryError> {
    #[cfg(target_os = "windows")]
    {
        let RawWindowHandle::Win32(win32) = window else {
            return Err(format!(
                "default_compositor_for_window (windows): expected Win32 \
                 RawWindowHandle, got {window:?}"
            )
            .into());
        };
        let hwnd = windows::Win32::Foundation::HWND(win32.hwnd.get() as *mut _);
        let backend = crate::WindowsDxgiBackend::new(&host, hwnd)
            .map_err(|e| Box::new(e) as BoxedFactoryError)?;
        let compositor = crate::ServoCompositor::new(host, backend);
        Ok(Box::new(compositor))
    }

    // NSView and UIView are NOT CALayers — they *have* a backing
    // CALayer accessible via the `layer` property. The previous
    // version of this code cast the view pointer to CALayer*
    // directly, which type-puns NSView as CALayer (different
    // Objective-C classes) and would crash the moment
    // `MacosCALayerBackend::new` called CALayer methods on it.
    //
    // The branches below extract the actual CALayer per platform:
    //   - macOS: AppKit. `setWantsLayer:YES` makes the contract
    //     explicit even though winit defaults layer-backed views
    //     since Big Sur.
    //   - iOS / tvOS / visionOS: UIKit. UIViews are always
    //     layer-backed; `[ui_view layer]` returns directly.
    //
    // The local `Retained<CALayer>` is dropped at the end of this
    // function — `MacosCALayerBackend::new` retains its own copy
    // internally, so the embedder reference (from the raw window
    // handle) is independent of the backend's.
    #[cfg(target_os = "macos")]
    {
        use objc2::rc::Retained;
        use objc2_app_kit::NSView;
        let RawWindowHandle::AppKit(appkit) = window else {
            return Err(format!(
                "default_compositor_for_window (macos): expected AppKit \
                 RawWindowHandle, got {window:?}"
            )
            .into());
        };
        // SAFETY: raw-window-handle hands us a NonNull NSView
        // pointer; the embedder owns the NSView lifetime.
        let ns_view: Retained<NSView> = unsafe {
            Retained::retain(appkit.ns_view.as_ptr() as *mut NSView)
                .ok_or_else(|| -> BoxedFactoryError { "failed to retain NSView".into() })?
        };
        ns_view.setWantsLayer(true);
        let layer = ns_view.layer().ok_or_else(|| -> BoxedFactoryError {
            "NSView.layer returned nil after setWantsLayer; view is not layer-backed".into()
        })?;
        let layer_ptr = Retained::as_ptr(&layer) as *mut std::ffi::c_void;
        // SAFETY: layer_ptr is the live `Retained<CALayer>` we just
        // got from `ns_view.layer()`; backend retains its own
        // reference. AppKit guarantees `[ns_view layer]` returns
        // a CALayer (or subclass), so the cast is type-correct.
        let backend = unsafe { crate::MacosCALayerBackend::new(&host, layer_ptr) }
            .map_err(|e| Box::new(e) as BoxedFactoryError)?;
        let compositor = crate::ServoCompositor::new(host, backend);
        // `layer` and `ns_view` drop here; backend holds its own
        // CALayer reference.
        drop(layer);
        drop(ns_view);
        return Ok(Box::new(compositor));
    }

    #[cfg(any(target_os = "ios", target_os = "tvos", target_os = "visionos"))]
    {
        use objc2::msg_send;
        use objc2::runtime::AnyObject;
        let RawWindowHandle::UiKit(uikit) = window else {
            return Err(format!(
                "default_compositor_for_window (uikit): expected UiKit \
                 RawWindowHandle, got {window:?}"
            )
            .into());
        };
        // UIView is always layer-backed; `[ui_view layer]` returns
        // a `CALayer*`. We use raw `msg_send!` rather than adding
        // an `objc2-ui-kit` dep just for the one method call.
        // SAFETY: raw-window-handle hands us a NonNull UIView
        // pointer; the embedder owns the UIView lifetime.
        let layer_ptr: *mut std::ffi::c_void = unsafe {
            let ui_view = uikit.ui_view.as_ptr() as *mut AnyObject;
            let layer: *mut AnyObject = msg_send![ui_view, layer];
            if layer.is_null() {
                return Err("UIView.layer returned nil".into());
            }
            layer as *mut std::ffi::c_void
        };
        // SAFETY: layer_ptr is a `CALayer*` returned by UIKit. The
        // UIView (held by the embedder) retains its layer, so the
        // pointer remains valid as long as the embedder keeps the
        // view alive — same lifetime contract as the macOS path.
        let backend = unsafe { crate::MacosCALayerBackend::new(&host, layer_ptr) }
            .map_err(|e| Box::new(e) as BoxedFactoryError)?;
        let compositor = crate::ServoCompositor::new(host, backend);
        return Ok(Box::new(compositor));
    }

    #[cfg(target_os = "linux")]
    {
        let RawWindowHandle::Wayland(wl_window) = window else {
            return Err(format!(
                "default_compositor_for_window (linux): expected Wayland \
                 RawWindowHandle, got {window:?}"
            )
            .into());
        };
        let RawDisplayHandle::Wayland(wl_display) = _display else {
            return Err(format!(
                "default_compositor_for_window (linux): expected Wayland \
                 RawDisplayHandle, got {_display:?}"
            )
            .into());
        };
        // SAFETY: pointers are non-null; embedder owns the wl_display +
        // wl_surface lifetimes.
        let backend = unsafe {
            crate::WaylandSubsurfaceBackend::new(
                &host,
                wl_display.display.as_ptr() as *mut _,
                wl_window.surface.as_ptr() as *mut _,
            )
        }
        .map_err(|e| Box::new(e) as BoxedFactoryError)?;
        let compositor = crate::ServoCompositor::new(host, backend);
        return Ok(Box::new(compositor));
    }

    #[cfg(not(any(target_os = "windows", target_vendor = "apple", target_os = "linux")))]
    {
        let _ = (host, window);
        Ok(Box::new(WgpuMasterCaptureBackend::new()))
    }
}

/// Select an OS handoff when available, otherwise expose a typed capture
/// fallback with its construction error. The caller can still use the
/// captured texture, but cannot mistake it for visible presentation.
pub fn default_compositor_for_window_or_capture(
    host: HostWgpuContext,
    display: RawDisplayHandle,
    window: RawWindowHandle,
) -> CompositorSelection {
    select_compositor(default_compositor_for_window(host, display, window))
}

fn select_compositor(
    result: Result<Box<dyn PaintCompositor>, BoxedFactoryError>,
) -> CompositorSelection {
    match result {
        Ok(compositor) => match compositor.presentation_route() {
            PresentationRoute::Display => CompositorSelection::Display(compositor),
            PresentationRoute::Capture => CompositorSelection::CaptureFallback {
                compositor,
                cause: None,
            },
        },
        Err(err) => {
            log::warn!(
                "[paint] default_compositor_for_window failed; falling back to \
                 WgpuMasterCaptureBackend: {err}"
            );
            CompositorSelection::CaptureFallback {
                compositor: Box::new(WgpuMasterCaptureBackend::new()),
                cause: Some(err),
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netrender_device::compositor::{Compositor, PresentedFrame, SurfaceKey};

    struct DisplayProbe;

    impl Compositor for DisplayProbe {
        fn declare_surface(&mut self, _: SurfaceKey, _: [f32; 4]) {}
        fn destroy_surface(&mut self, _: SurfaceKey) {}
        fn present_frame(&mut self, _: PresentedFrame<'_>) {}
    }

    impl PaintCompositor for DisplayProbe {
        fn presentation_route(&self) -> PresentationRoute {
            PresentationRoute::Display
        }

        fn presentation_outcome(&self) -> Option<&crate::PresentationOutcome> {
            None
        }

        fn take_presentation_outcome(&mut self) -> Option<crate::PresentationOutcome> {
            None
        }
    }

    #[test]
    fn successful_os_factory_stays_on_display_route() {
        let selected = select_compositor(Ok(Box::new(DisplayProbe)));
        let CompositorSelection::Display(compositor) = selected else {
            panic!("OS compositor was reported as capture fallback");
        };
        assert_eq!(compositor.presentation_route(), PresentationRoute::Display);
    }

    #[test]
    fn capture_only_factory_success_is_not_a_display_route() {
        let selected = select_compositor(Ok(Box::new(WgpuMasterCaptureBackend::new())));
        let CompositorSelection::CaptureFallback { compositor, cause } = selected else {
            panic!("capture backend was reported as display handoff");
        };
        assert!(cause.is_none());
        assert_eq!(compositor.presentation_route(), PresentationRoute::Capture);
        assert!(compositor.presentation_outcome().is_none());
    }

    #[test]
    fn failed_os_construction_keeps_its_reason_with_capture_fallback() {
        let selected = select_compositor(Err("native window rejected".into()));
        let CompositorSelection::CaptureFallback { compositor, cause } = selected else {
            panic!("failed construction was reported as display handoff");
        };
        assert_eq!(cause.unwrap().to_string(), "native window rejected");
        assert_eq!(compositor.presentation_route(), PresentationRoute::Capture);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn unsupported_window_handle_reaches_factory_capture_fallback() {
        use raw_window_handle::{WebDisplayHandle, WebWindowHandle};

        let host = pollster::block_on(async {
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
                    label: Some("H10 factory fallback test"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    ..Default::default()
                })
                .await
                .expect("device");
            HostWgpuContext::new(device, queue)
        });
        let selected = default_compositor_for_window_or_capture(
            host,
            RawDisplayHandle::Web(WebDisplayHandle::new()),
            RawWindowHandle::Web(WebWindowHandle::new(1)),
        );
        let CompositorSelection::CaptureFallback { compositor, cause } = selected else {
            panic!("unsupported window was reported as display handoff");
        };
        assert_eq!(compositor.presentation_route(), PresentationRoute::Capture);
        assert!(
            cause
                .expect("factory rejection reason")
                .to_string()
                .contains("expected Wayland RawWindowHandle")
        );
    }
}
