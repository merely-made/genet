// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The CSS Writing Modes Level 4 logical-to-physical axis and side mapping.
//!
//! This small vocabulary is shared by style values and box geometry without
//! making either layer depend on the other.

#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LogicalAxis {
    Inline,
    Block,
}

impl LogicalAxis {
    pub const fn to_physical(self, writing_mode: WritingMode) -> PhysicalAxis {
        match (self, writing_mode.is_vertical()) {
            (Self::Inline, false) | (Self::Block, true) => PhysicalAxis::Horizontal,
            (Self::Inline, true) | (Self::Block, false) => PhysicalAxis::Vertical,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PhysicalAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WritingMode {
    #[default]
    HorizontalTb,
    VerticalRl,
    VerticalLr,
    SidewaysRl,
    SidewaysLr,
}

impl WritingMode {
    pub const fn is_vertical(self) -> bool {
        !matches!(self, Self::HorizontalTb)
    }
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
        let block_start = match writing_mode {
            WritingMode::HorizontalTb => PhysicalSide::Top,
            WritingMode::VerticalRl | WritingMode::SidewaysRl => PhysicalSide::Right,
            WritingMode::VerticalLr | WritingMode::SidewaysLr => PhysicalSide::Left,
        };
        let inline_start = match (writing_mode, direction) {
            (WritingMode::HorizontalTb, Direction::Ltr) => PhysicalSide::Left,
            (WritingMode::HorizontalTb, Direction::Rtl) => PhysicalSide::Right,
            (
                WritingMode::VerticalRl | WritingMode::VerticalLr | WritingMode::SidewaysRl,
                Direction::Ltr,
            ) => PhysicalSide::Top,
            (
                WritingMode::VerticalRl | WritingMode::VerticalLr | WritingMode::SidewaysRl,
                Direction::Rtl,
            ) => PhysicalSide::Bottom,
            (WritingMode::SidewaysLr, Direction::Ltr) => PhysicalSide::Bottom,
            (WritingMode::SidewaysLr, Direction::Rtl) => PhysicalSide::Top,
        };
        match self {
            Self::BlockStart => block_start,
            Self::BlockEnd => block_start.opposite(),
            Self::InlineStart => inline_start,
            Self::InlineEnd => inline_start.opposite(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PhysicalSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl PhysicalSide {
    pub const fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }
}
