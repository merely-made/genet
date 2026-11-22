/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::NSView;
use objc2_core_foundation::{CGAffineTransform, CGPoint, CGRect, CGSize};
use objc2_io_surface::IOSurfaceRef;
use objc2_quartz_core::{CALayer, CATransaction};
use raw_window_handle::{RawWindowHandle, WindowHandle};

use super::OpaquePaneDrag;

/// A producer-owned, opaque IOSurface displayed directly over an AppKit content view.
/// All geometry is in logical points measured from the top left of the content view.
pub struct MacosImportedPane {
    layer: Retained<CALayer>,
    parent: Retained<CALayer>,
}

impl MacosImportedPane {
    /// Attach on the window's main thread. The producer must finish writing before
    /// handing off the surface and keep its published pixels stable until replaced.
    /// The contents must completely cover the pane with opaque pixels.
    pub fn attach(
        window: WindowHandle<'_>,
        surface: &IOSurfaceRef,
        rect: [f64; 4],
    ) -> Result<Self, &'static str> {
        let RawWindowHandle::AppKit(handle) = window.as_raw() else {
            return Err("imported panes require an AppKit window");
        };
        let _main = MainThreadMarker::new().ok_or("attach imported panes on the main thread")?;
        // SAFETY: the borrowed AppKit handle guarantees a live NSView for this call.
        let view = unsafe { &*handle.ns_view.cast::<NSView>().as_ptr() };
        let parent = view
            .layer()
            .ok_or("the host content view is not layer-backed")?;
        let layer = CALayer::new();
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        layer.setName(Some(objc2_foundation::ns_string!("Genet imported pane")));
        layer.setContentsScale(parent.contentsScale());
        layer.setOpaque(true);
        let pane = Self { layer, parent };
        pane.set_surface(surface);
        pane.place(rect);
        pane.parent.addSublayer(&pane.layer);
        CATransaction::commit();
        Ok(pane)
    }

    /// Replace only the producer image. An in-progress drag stays attached to this layer.
    pub fn set_surface(&self, surface: &IOSurfaceRef) {
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        // SAFETY: CALayer accepts and retains IOSurface contents.
        unsafe {
            self.layer
                .setContents(Some(&*(surface as *const IOSurfaceRef).cast::<AnyObject>()))
        };
        CATransaction::commit();
    }

    /// Hidden tabs retain the producer image for reactivation.
    pub fn set_visible(&self, visible: bool) {
        if self.layer.isHidden() == visible {
            CATransaction::begin();
            CATransaction::setDisableActions(true);
            self.layer.setHidden(!visible);
            CATransaction::commit();
        }
    }

    pub fn place(&self, [x, y, w, h]: [f64; 4]) {
        let bounds = self.parent.bounds();
        let y = if self.parent.isGeometryFlipped() {
            y
        } else {
            bounds.size.height - y - h
        };
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        self.layer.setAffineTransform(CGAffineTransform {
            a: 1.,
            b: 0.,
            c: 0.,
            d: 1.,
            tx: 0.,
            ty: 0.,
        });
        self.layer.setFrame(CGRect {
            origin: CGPoint {
                x: bounds.origin.x + x,
                y: bounds.origin.y + y,
            },
            size: CGSize {
                width: w,
                height: h,
            },
        });
        CATransaction::commit();
    }

    /// Publish a pointer placement before returning to the event loop.
    pub fn translate_drag(&self, dx: f64, dy: f64) {
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        self.layer.setAffineTransform(CGAffineTransform {
            a: 1.,
            b: 0.,
            c: 0.,
            d: 1.,
            tx: dx,
            ty: if self.parent.isGeometryFlipped() {
                dy
            } else {
                -dy
            },
        });
        CATransaction::commit();
        CATransaction::flush();
    }

    /// Begin at pointer-down and drop the returned guard on release or cancellation.
    pub fn begin_drag(&self) -> Option<OpaquePaneDrag> {
        OpaquePaneDrag::begin(&self.layer)
    }
}

impl Drop for MacosImportedPane {
    fn drop(&mut self) {
        self.layer.removeFromSuperlayer();
    }
}
