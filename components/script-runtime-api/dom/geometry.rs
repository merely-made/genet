// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The geometry seam: element boxes, client/offset/scroll metrics and scroll
//! positions out of (and back into) the host's layout.
//!
//! The runtime never links a layout engine, so a host that has one implements
//! [`GeometryHandler`], exactly as it does [`SelectionHandler`]. Nodes are
//! reflector raw ids. Everything the handler returns is in CSS pixels in
//! viewport coordinates unless a field says otherwise; the script side rounds
//! the integer-valued CSSOM View attributes itself.
//!
//! A read must see the DOM as script left it, so a host brings its layout up
//! to date inside each call. That is why these are methods, not a snapshot.

use super::*;

/// Layout metrics for one element. An element with no box answers [`None`] from
/// [`GeometryHandler::metrics`] instead.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ElementMetrics {
    pub client_top: f32,
    pub client_left: f32,
    pub client_width: f32,
    pub client_height: f32,
    pub scroll_left: f32,
    pub scroll_top: f32,
    pub scroll_width: f32,
    pub scroll_height: f32,
    pub offset_top: f32,
    pub offset_left: f32,
    pub offset_width: f32,
    pub offset_height: f32,
    /// The reflector raw id of `offsetParent`, when it has one.
    pub offset_parent: Option<u64>,
}

/// One `IntersectionObserver` observation to compute. A margin side is its
/// number and whether it is a percentage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntersectionRequest {
    pub target: u64,
    /// `None` is the implicit root: the viewport.
    pub root: Option<u64>,
    /// Top, right, bottom, left.
    pub root_margin: [(f32, bool); 4],
    pub scroll_margin: [(f32, bool); 4],
}

/// The geometry of one observation, rectangles as `[x, y, width, height]` in
/// viewport coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntersectionReport {
    pub target: [f32; 4],
    pub intersection: [f32; 4],
    pub root_bounds: Option<[f32; 4]>,
    pub is_intersecting: bool,
}

/// The host's layout seam for CSSOM View element geometry and scrolling.
pub trait GeometryHandler {
    /// How much of a target shows through the clips up to a root. `None` when
    /// the host has no layout to answer from.
    fn intersection(&self, request: &IntersectionRequest) -> Option<IntersectionReport>;

    /// Border-box rectangles `[x, y, width, height]` of the element's boxes in
    /// viewport coordinates, transforms applied. Empty when it has no box.
    fn client_rects(&self, node: u64) -> Vec<[f32; 4]>;

    /// Client, scroll and offset metrics. `None` when the element has no box.
    fn metrics(&self, node: u64) -> Option<ElementMetrics>;

    /// Set an element's scroll offset; `None` leaves that axis alone. The host
    /// clamps to the scroll range. The viewport's scroller (the root element,
    /// or `body` in quirks mode) scrolls the viewport instead.
    fn set_scroll(&self, node: u64, left: Option<f32>, top: Option<f32>);

    /// The viewport scroll offset, after any layout update.
    fn viewport_scroll(&self) -> (f32, f32);

    /// Scroll the viewport to `(x, y)`, clamped; returns the resulting offset.
    fn set_viewport_scroll(&self, x: f32, y: f32) -> (f32, f32);
}

fn handler<E: ScriptEngine>(cx: &mut E::CallCx<'_>) -> Option<Rc<dyn GeometryHandler>> {
    let host = adoption::host_for_call::<E>(cx)?;
    let handler = host.borrow().geometry.clone();
    handler
}

fn number_arg<E: ScriptEngine>(
    cx: &mut E::CallCx<'_>,
    index: usize,
) -> Result<Option<f32>, E::Error> {
    let value = cx.arg(index);
    let text = cx.value_to_string(&value)?;
    Ok(text
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|number| number.is_finite()))
}

/// `__geomRects(ref)` -> newline-separated `x,y,width,height`, empty without a box.
pub(crate) struct Rects;
impl<E: ScriptEngine> NativeFn<E> for Rects {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let value = cx.arg(0);
        let Some(node) = cx.owned_node(&value)? else {
            return cx.make_string("");
        };
        let rects = handler::<E>(cx)
            .map(|h| h.client_rects(node.raw()))
            .unwrap_or_default();
        let encoded = rects
            .iter()
            .map(|r| format!("{},{},{},{}", r[0], r[1], r[2], r[3]))
            .collect::<Vec<_>>()
            .join("\n");
        cx.make_string(&encoded)
    }
}

/// `__geomMetrics(ref)` -> thirteen comma-separated numbers in the field order
/// of [`ElementMetrics`] (`offsetParent` last, `-1` for none), empty without a box.
pub(crate) struct Metrics;
impl<E: ScriptEngine> NativeFn<E> for Metrics {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let value = cx.arg(0);
        let Some(node) = cx.owned_node(&value)? else {
            return cx.make_string("");
        };
        let Some(m) = handler::<E>(cx).and_then(|h| h.metrics(node.raw())) else {
            return cx.make_string("");
        };
        let parent = m.offset_parent.map_or(-1.0, |id| id as f64);
        let encoded = format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{}",
            m.client_top,
            m.client_left,
            m.client_width,
            m.client_height,
            m.scroll_left,
            m.scroll_top,
            m.scroll_width,
            m.scroll_height,
            m.offset_top,
            m.offset_left,
            m.offset_width,
            m.offset_height,
            parent
        );
        cx.make_string(&encoded)
    }
}

/// `__geomScroll(ref, left, top)` — a non-numeric argument leaves its axis alone.
pub(crate) struct SetScroll;
impl<E: ScriptEngine> NativeFn<E> for SetScroll {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let value = cx.arg(0);
        if let Some(node) = cx.owned_node(&value)? {
            let left = number_arg::<E>(cx, 1)?;
            let top = number_arg::<E>(cx, 2)?;
            if let Some(handler) = handler::<E>(cx) {
                handler.set_scroll(node.raw(), left, top);
            }
        }
        Ok(cx.undefined())
    }
}

/// `__geomOffsetParent(ref)` -> the `offsetParent` node's reference, or `undefined`.
pub(crate) struct OffsetParent;
impl<E: ScriptEngine> NativeFn<E> for OffsetParent {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let value = cx.arg(0);
        let Some(node) = cx.owned_node(&value)? else {
            return Ok(cx.undefined());
        };
        let parent = handler::<E>(cx)
            .and_then(|h| h.metrics(node.raw()))
            .and_then(|m| m.offset_parent);
        match parent {
            Some(raw) => reflect_pinned::<E>(cx, raw),
            None => Ok(cx.undefined()),
        }
    }
}

fn margin_arg<E: ScriptEngine>(
    cx: &mut E::CallCx<'_>,
    index: usize,
) -> Result<[(f32, bool); 4], E::Error> {
    let value = cx.arg(index);
    let text = cx.value_to_string(&value)?;
    let numbers: Vec<f32> = text
        .split(',')
        .map(|part| part.trim().parse::<f32>().unwrap_or(0.0))
        .collect();
    let mut sides = [(0.0, false); 4];
    for (side, pair) in sides.iter_mut().zip(numbers.chunks(2)) {
        *side = (pair[0], pair.get(1).is_some_and(|flag| *flag != 0.0));
    }
    Ok(sides)
}

/// `__geomIntersection(targetRef, rootRef, rootMargin, scrollMargin)` ->
/// `target;intersection;rootBounds;isIntersecting`, each rectangle
/// `x,y,width,height` (the last two possibly empty), or empty without layout.
/// An empty `rootRef` is the implicit root.
pub(crate) struct Intersection;
impl<E: ScriptEngine> NativeFn<E> for Intersection {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let target_value = cx.arg(0);
        let root_value = cx.arg(1);
        let Some(target) = cx.owned_node(&target_value)? else {
            return cx.make_string("");
        };
        let root_text = cx.value_to_string(&root_value)?;
        let root = if root_text.is_empty() {
            None
        } else {
            match cx.owned_node(&root_value)? {
                Some(root) => Some(root.raw()),
                None => return cx.make_string(""),
            }
        };
        let request = IntersectionRequest {
            target: target.raw(),
            root,
            root_margin: margin_arg::<E>(cx, 2)?,
            scroll_margin: margin_arg::<E>(cx, 3)?,
        };
        let Some(report) = handler::<E>(cx).and_then(|h| h.intersection(&request)) else {
            return cx.make_string("");
        };
        let rect = |r: [f32; 4]| format!("{},{},{},{}", r[0], r[1], r[2], r[3]);
        let encoded = format!(
            "{};{};{};{}",
            rect(report.target),
            rect(report.intersection),
            report.root_bounds.map(rect).unwrap_or_default(),
            u8::from(report.is_intersecting)
        );
        cx.make_string(&encoded)
    }
}
