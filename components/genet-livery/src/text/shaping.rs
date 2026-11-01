// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::prepared::ShapeInput;
use super::*;

pub(super) struct ShapeWorker {
    pub font_context: FontContext,
    pub layout_context: LayoutContext<Brush>,
}
pub(super) struct UnbrokenText {
    pub layout: parley::Layout<Brush>,
}
impl ShapeWorker {
    pub(super) fn new() -> Self {
        Self {
            font_context: FontContext::new(),
            layout_context: LayoutContext::new(),
        }
    }
    pub(super) fn shape(&mut self, input: &ShapeInput) -> UnbrokenText {
        let mut builder = self.layout_context.ranged_builder(
            &mut self.font_context,
            &input.text,
            f32::from_bits(input.scale),
            input.quantize,
        );
        input.apply(&mut builder);
        UnbrokenText {
            layout: builder.build(&input.text),
        }
    }
}
