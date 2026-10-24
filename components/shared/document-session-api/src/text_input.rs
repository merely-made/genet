// SPDX-License-Identifier: MPL-2.0
//! Focused native text-control observations, without document values or IDs.

/// Caret rectangle in the session's logical viewport coordinates, after page
/// zoom and scrolling. Window placement and device scale belong to the host.
/// An offscreen caret may have negative coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SessionTextInputRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// Metadata and geometry for the focused writable text control. Selection
/// offsets address its displayed editing value in UTF-16 code units. They do
/// not address the separate, UTF-8-indexed `SessionIme::Preedit` string.
///
/// Geometry comes from the current completed layout. No field text, password
/// value, document node identity or native window handle crosses this boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionTextInputState {
    pub element_kind: String,
    pub input_type: String,
    pub input_mode: String,
    pub autocomplete: String,
    pub multiline: bool,
    pub password: bool,
    pub selection_utf16: (u32, u32),
    pub caret: SessionTextInputRect,
}
