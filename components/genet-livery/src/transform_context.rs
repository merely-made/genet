// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The supported CSS transform facts shared by box generation and paint.
//!
//! A transform on the element and a perspective for its children have
//! different paint effects, but both establish a stacking context and a
//! containing block for positioned descendants.

use livery::{
    ComputedValues,
    values::{Display, Float, Perspective, Position, Rotate, Scale, TransformStyle, Translate},
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct TransformContextFacts {
    own_transform: bool,
    perspective: bool,
    preserves_3d: bool,
}

impl TransformContextFacts {
    /// `flex_grid_item` is the box generator's blockification of a computed
    /// inline box. Replaced inline boxes are transformable without it.
    pub(crate) fn for_box(style: &ComputedValues, replaced: bool, flex_grid_item: bool) -> Self {
        let blockified = flex_grid_item
            || matches!(style.position, Position::Absolute | Position::Fixed)
            || style.float != Float::None;
        let transformable = match style.display {
            Display::None | Display::Contents => false,
            Display::Inline => replaced || blockified,
            // Replaced elements with an internal-table display are demoted
            // to inline flow by `box_tree::display_role`.
            Display::TableColumn | Display::TableColumnGroup => replaced,
            _ => true,
        };
        Self {
            own_transform: transformable
                && (!style.transform.is_none()
                    || style.rotate != Rotate::None
                    || style.scale != Scale::None
                    || style.translate != Translate::None),
            perspective: transformable && style.perspective != Perspective::None,
            preserves_3d: transformable && style.transform_style == TransformStyle::Preserve3d,
        }
    }

    /// The element's own transform needs a paint matrix and its border-box
    /// dimensions as the reference box for percentages and transform origin.
    pub(crate) fn paints_own_transform(self) -> bool {
        self.own_transform
    }

    pub(crate) fn provides_perspective(self) -> bool {
        self.perspective
    }

    pub(crate) fn preserves_3d(self) -> bool {
        self.preserves_3d
    }

    pub(crate) fn establishes_stacking_context(self) -> bool {
        self.own_transform || self.perspective || self.preserves_3d
    }

    pub(crate) fn establishes_positioned_containing_block(self) -> bool {
        self.own_transform || self.perspective || self.preserves_3d
    }
}
