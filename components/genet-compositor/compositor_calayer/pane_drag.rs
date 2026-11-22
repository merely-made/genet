/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Drag-scoped Core Animation activity for a pane covered by opaque contents.

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSString, ns_string};
use objc2_quartz_core::{
    CABasicAnimation, CALayer, CAMediaTiming, CAMediaTimingFunction, CATransaction,
    kCAMediaTimingFunctionLinear,
};

fn key() -> &'static NSString {
    ns_string!("genet.opaquePaneDrag")
}

/// Keeps the pane's layer participating in display refreshes during a pointer drag.
/// Drop on release or cancellation. The animation leaves the model layer untouched.
#[must_use = "dropping this guard ends drag activity"]
pub struct OpaquePaneDrag {
    layer: Retained<CALayer>,
}

impl OpaquePaneDrag {
    pub(super) fn begin(layer: &CALayer) -> Option<Self> {
        // SAFETY: the returned animation is used only to test occupancy of our own key.
        if unsafe { layer.animationForKey(key()) }.is_some() {
            return None;
        }
        let animation = CABasicAnimation::animationWithKeyPath(Some(ns_string!("backgroundColor")));
        let black = objc2_app_kit::NSColor::blackColor().CGColor();
        let white = objc2_app_kit::NSColor::whiteColor().CGColor();
        // SAFETY: backgroundColor accepts CGColor objects. The setters retain both endpoints.
        unsafe {
            animation.setFromValue(Some(&*Retained::as_ptr(&black).cast::<AnyObject>()));
            animation.setToValue(Some(&*Retained::as_ptr(&white).cast::<AnyObject>()));
            animation.setTimingFunction(Some(&CAMediaTimingFunction::functionWithName(
                kCAMediaTimingFunctionLinear,
            )));
        }
        animation.setDuration(0.811);
        animation.setAutoreverses(true);
        animation.setRepeatCount(f32::INFINITY);
        layer.addAnimation_forKey(&animation, Some(key()));
        CATransaction::flush();
        use objc2::Message;
        Some(Self {
            layer: layer.retain(),
        })
    }
}

impl Drop for OpaquePaneDrag {
    fn drop(&mut self) {
        self.layer.removeAnimationForKey(key());
        CATransaction::flush();
    }
}
