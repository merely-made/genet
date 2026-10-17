/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Flow-relative geometry used by generated logical-property mappings.
//!
//! This is a bounded adaptation of `genet-stylo` 0.19.1's
//! `logical_geometry.rs`. Livery keeps its own value enums and retains only
//! the axis/side projection required by its generated property catalog.

use super::{Direction, WritingMode};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LogicalAxis {
    Inline,
    Block,
}

impl LogicalAxis {
    pub const fn to_physical(self, writing_mode: WritingMode) -> PhysicalAxis {
        let axis = match self {
            Self::Inline => css_flow::LogicalAxis::Inline,
            Self::Block => css_flow::LogicalAxis::Block,
        };
        match axis.to_physical(flow_writing_mode(writing_mode)) {
            css_flow::PhysicalAxis::Horizontal => PhysicalAxis::Horizontal,
            css_flow::PhysicalAxis::Vertical => PhysicalAxis::Vertical,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PhysicalAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LogicalSide {
    BlockStart,
    BlockEnd,
    InlineStart,
    InlineEnd,
}

impl LogicalSide {
    pub const fn to_physical(
        self,
        writing_mode: WritingMode,
        direction: Direction,
    ) -> PhysicalSide {
        let side = match self {
            Self::BlockStart => css_flow::LogicalSide::BlockStart,
            Self::BlockEnd => css_flow::LogicalSide::BlockEnd,
            Self::InlineStart => css_flow::LogicalSide::InlineStart,
            Self::InlineEnd => css_flow::LogicalSide::InlineEnd,
        };
        match side.to_physical(flow_writing_mode(writing_mode), flow_direction(direction)) {
            css_flow::PhysicalSide::Top => PhysicalSide::Top,
            css_flow::PhysicalSide::Right => PhysicalSide::Right,
            css_flow::PhysicalSide::Bottom => PhysicalSide::Bottom,
            css_flow::PhysicalSide::Left => PhysicalSide::Left,
        }
    }
}

const fn flow_writing_mode(writing_mode: WritingMode) -> css_flow::WritingMode {
    match writing_mode {
        WritingMode::HorizontalTb => css_flow::WritingMode::HorizontalTb,
        WritingMode::VerticalRl => css_flow::WritingMode::VerticalRl,
        WritingMode::VerticalLr => css_flow::WritingMode::VerticalLr,
        WritingMode::SidewaysRl => css_flow::WritingMode::SidewaysRl,
        WritingMode::SidewaysLr => css_flow::WritingMode::SidewaysLr,
    }
}

const fn flow_direction(direction: Direction) -> css_flow::Direction {
    match direction {
        Direction::Ltr => css_flow::Direction::Ltr,
        Direction::Rtl => css_flow::Direction::Rtl,
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PhysicalSide {
    Top,
    Right,
    Bottom,
    Left,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logical_sides_and_axes_follow_all_writing_modes_and_directions() {
        let cases = [
            (
                WritingMode::HorizontalTb,
                Direction::Ltr,
                PhysicalSide::Top,
                PhysicalSide::Left,
            ),
            (
                WritingMode::HorizontalTb,
                Direction::Rtl,
                PhysicalSide::Top,
                PhysicalSide::Right,
            ),
            (
                WritingMode::VerticalRl,
                Direction::Ltr,
                PhysicalSide::Right,
                PhysicalSide::Top,
            ),
            (
                WritingMode::VerticalRl,
                Direction::Rtl,
                PhysicalSide::Right,
                PhysicalSide::Bottom,
            ),
            (
                WritingMode::VerticalLr,
                Direction::Ltr,
                PhysicalSide::Left,
                PhysicalSide::Top,
            ),
            (
                WritingMode::VerticalLr,
                Direction::Rtl,
                PhysicalSide::Left,
                PhysicalSide::Bottom,
            ),
            (
                WritingMode::SidewaysRl,
                Direction::Ltr,
                PhysicalSide::Right,
                PhysicalSide::Top,
            ),
            (
                WritingMode::SidewaysRl,
                Direction::Rtl,
                PhysicalSide::Right,
                PhysicalSide::Bottom,
            ),
            (
                WritingMode::SidewaysLr,
                Direction::Ltr,
                PhysicalSide::Left,
                PhysicalSide::Bottom,
            ),
            (
                WritingMode::SidewaysLr,
                Direction::Rtl,
                PhysicalSide::Left,
                PhysicalSide::Top,
            ),
        ];
        for (writing_mode, direction, block_start, inline_start) in cases {
            assert_eq!(
                LogicalSide::InlineStart.to_physical(writing_mode, direction),
                inline_start
            );
            assert_eq!(
                LogicalSide::BlockStart.to_physical(writing_mode, direction),
                block_start
            );
            let opposite = |side| match side {
                PhysicalSide::Top => PhysicalSide::Bottom,
                PhysicalSide::Right => PhysicalSide::Left,
                PhysicalSide::Bottom => PhysicalSide::Top,
                PhysicalSide::Left => PhysicalSide::Right,
            };
            assert_eq!(
                LogicalSide::InlineEnd.to_physical(writing_mode, direction),
                opposite(inline_start)
            );
            assert_eq!(
                LogicalSide::BlockEnd.to_physical(writing_mode, direction),
                opposite(block_start)
            );
            let inline_axis = if writing_mode.is_vertical() {
                PhysicalAxis::Vertical
            } else {
                PhysicalAxis::Horizontal
            };
            assert_eq!(LogicalAxis::Inline.to_physical(writing_mode), inline_axis);
            assert_eq!(
                LogicalAxis::Block.to_physical(writing_mode),
                match inline_axis {
                    PhysicalAxis::Horizontal => PhysicalAxis::Vertical,
                    PhysicalAxis::Vertical => PhysicalAxis::Horizontal,
                }
            );
        }
    }
}
