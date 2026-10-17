// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Opt-in, thread-local receipts for Parley's Fontique queries.

#[cfg(test)]
use std::cell::Cell;
use std::{cell::RefCell, marker::PhantomData, rc::Rc, string::String, thread_local, vec::Vec};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontDiagnosticCandidate {
    pub family: String,
    pub index: u32,
    pub status: FontDiagnosticCandidateStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FontDiagnosticCandidateStatus {
    NoCharmap,
    Complete,
    Keep,
    Discard,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontDiagnosticEvent {
    /// Source characters in the queried grapheme cluster.
    pub cluster: String,
    pub fallback_script: [u8; 4],
    pub candidates: Vec<FontDiagnosticCandidate>,
    pub selected_candidate: Option<usize>,
}

#[derive(Default)]
struct State {
    enabled: bool,
    events: Vec<FontDiagnosticEvent>,
}

thread_local! {
    static STATE: RefCell<State> = const { RefCell::new(State { enabled: false, events: Vec::new() }) };
}

#[cfg(test)]
thread_local! {
    static EVENT_CONSTRUCTIONS: Cell<usize> = const { Cell::new(0) };
}

/// Enables recording on this thread until [`take`](Self::take) or drop.
///
/// Starting a capture clears stale events. Dropping an unconsumed capture
/// disables recording and discards its events, including during unwinding.
#[must_use = "capture diagnostics with take() or let the scope drop"]
pub struct FontDiagnosticCapture {
    active: bool,
    // Capture teardown must run on the thread whose thread-local state it owns.
    _not_send_or_sync: PhantomData<Rc<()>>,
}

/// Starts one scoped capture of Fontique queries on the current thread.
pub fn begin_font_diagnostic_capture() -> FontDiagnosticCapture {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        assert!(!state.enabled, "font diagnostic capture is already active");
        state.events.clear();
        state.enabled = true;
    });
    FontDiagnosticCapture {
        active: true,
        _not_send_or_sync: PhantomData,
    }
}

impl FontDiagnosticCapture {
    /// Stops recording and returns this scope's events.
    pub fn take(mut self) -> Vec<FontDiagnosticEvent> {
        self.active = false;
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            state.enabled = false;
            core::mem::take(&mut state.events)
        })
    }
}

impl Drop for FontDiagnosticCapture {
    fn drop(&mut self) {
        if self.active {
            STATE.with(|state| {
                let mut state = state.borrow_mut();
                state.enabled = false;
                state.events.clear();
            });
        }
    }
}

pub(crate) fn construct_if_capturing<T>(construct: impl FnOnce() -> T) -> Option<T> {
    STATE.with(|state| state.borrow().enabled).then(construct)
}

#[cfg(test)]
pub(crate) fn note_event_construction() {
    EVENT_CONSTRUCTIONS.with(|count| count.set(count.get() + 1));
}

pub(crate) fn record(event: FontDiagnosticEvent) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if state.enabled {
            state.events.push(event);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{borrow::Cow, sync::Arc};

    use crate::{FontContext, FontFamily, LayoutContext, StyleProperty};
    use fontique::{Blob, FontInfoOverride};

    fn shape_control() -> f32 {
        let mut font_context = FontContext {
            collection: fontique::Collection::new(fontique::CollectionOptions {
                shared: false,
                system_fonts: false,
            }),
            source_cache: fontique::SourceCache::default(),
        };
        font_context.collection.register_fonts(
            Blob::new(Arc::new(
                include_bytes!("../../../../tests/wpt/tests/fonts/Lato-Medium.ttf").to_vec(),
            )),
            Some(FontInfoOverride {
                family_name: Some("face"),
                ..Default::default()
            }),
        );
        let mut layout_context: LayoutContext<[u8; 4]> = LayoutContext::new();
        let mut builder = layout_context.ranged_builder(&mut font_context, "a", 1.0, false);
        builder.push_default(StyleProperty::FontSize(32.0));
        builder.push_default(StyleProperty::FontFamily(FontFamily::Source(
            Cow::Borrowed("face"),
        )));
        let mut layout = builder.build("a");
        layout.break_all_lines(None);
        let advance = layout
            .lines()
            .next()
            .expect("one text line")
            .metrics()
            .advance;
        advance
    }

    #[test]
    fn shaping_skips_payload_without_capture_and_records_it_with_capture() {
        EVENT_CONSTRUCTIONS.with(|count| count.set(0));
        let inactive_advance = shape_control();
        assert_eq!(EVENT_CONSTRUCTIONS.with(Cell::get), 0);

        let capture = begin_font_diagnostic_capture();
        let active_advance = shape_control();
        let events = capture.take();
        assert!(EVENT_CONSTRUCTIONS.with(Cell::get) > 0);
        assert_eq!(inactive_advance, active_advance);
        let event = events
            .iter()
            .find(|event| event.cluster == "a")
            .expect("authored control reaches the font query");
        assert_eq!(event.fallback_script, *b"Latn");
        assert_eq!(event.selected_candidate, Some(0));
        assert_eq!(
            event.candidates[0].status,
            FontDiagnosticCandidateStatus::Complete
        );
    }

    #[test]
    fn active_capture_records_event_and_rejects_nested_capture() {
        let capture = begin_font_diagnostic_capture();
        assert!(std::panic::catch_unwind(begin_font_diagnostic_capture).is_err());

        let event = construct_if_capturing(|| FontDiagnosticEvent {
            cluster: String::from("a"),
            fallback_script: *b"Latn",
            candidates: std::vec![FontDiagnosticCandidate {
                family: String::from("Test"),
                index: 3,
                status: FontDiagnosticCandidateStatus::Complete,
            }],
            selected_candidate: Some(0),
        })
        .expect("capture remains active after rejected nesting");
        record(event.clone());

        assert_eq!(capture.take(), std::vec![event]);
        assert!(construct_if_capturing(|| ()).is_none());
    }
}
