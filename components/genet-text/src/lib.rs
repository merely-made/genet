// SPDX-License-Identifier: MPL-2.0
//! Default UAX #29 boundaries over an unchanged UTF-8 string.
//!
//! This crate supplies boundary mechanics, not locale dictionaries, search
//! normalization, DOM traversal, or index policy. Consumers choose their own
//! tailoring outside this default profile. Upgrade the pinned engine and its
//! official conformance corpus together.
use std::ops::Range;
use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

/// Unicode data version of this implementation; persist with boundary caches.
pub const UNICODE_VERSION: (u64, u64, u64) = unicode_segmentation::UNICODE_VERSION;
/// Identifies default extended grapheme, word, and sentence rules (no tailoring).
pub const PROFILE: &str = "uax29-default-unicode-17.0.0";

/// All offsets address the same unchanged string, never a normalized copy.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Offset {
    pub utf8_bytes: usize,
    pub unicode_scalars: usize,
    pub utf16_units: usize,
}

/// Half-open range in each explicitly named coordinate system.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Segment<'a> {
    pub text: &'a str,
    pub start: Offset,
    pub end: Offset,
}

impl Segment<'_> {
    pub fn byte_range(&self) -> Range<usize> {
        self.start.utf8_bytes..self.end.utf8_bytes
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryKind {
    ExtendedGrapheme,
    /// Includes punctuation and whitespace. Word filtering is separate.
    Word,
    /// Includes punctuation and trailing whitespace.
    Sentence,
}

/// Streaming segments with offsets in bytes, Unicode scalar values, and UTF-16.
/// Each scalar is counted once; offsets do not require repeated prefix scans.
pub fn segments(text: &str, kind: BoundaryKind) -> impl Iterator<Item = Segment<'_>> {
    enum Parts<'a> {
        Grapheme(unicode_segmentation::Graphemes<'a>),
        Word(unicode_segmentation::UWordBounds<'a>),
        Sentence(unicode_segmentation::USentenceBounds<'a>),
    }
    let mut parts = match kind {
        BoundaryKind::ExtendedGrapheme => Parts::Grapheme(text.graphemes(true)),
        BoundaryKind::Word => Parts::Word(text.split_word_bounds()),
        BoundaryKind::Sentence => Parts::Sentence(text.split_sentence_bounds()),
    };
    let mut offset = Offset::default();
    std::iter::from_fn(move || {
        let part = match &mut parts {
            Parts::Grapheme(p) => p.next(),
            Parts::Word(p) => p.next(),
            Parts::Sentence(p) => p.next(),
        }?;
        let start = offset;
        offset.utf8_bytes += part.len();
        for c in part.chars() {
            offset.unicode_scalars += 1;
            offset.utf16_units += c.len_utf16();
        }
        Some(Segment {
            text: part,
            start,
            end: offset,
        })
    })
}

/// UAX #29 words containing alphabetic or numeric characters. Search-specific
/// punctuation splitting, normalization and stemming belong to the caller.
pub fn words(text: &str) -> impl Iterator<Item = &str> {
    text.unicode_words()
}

/// Adjacent extended-grapheme boundary in the complete string, in UTF-8 bytes.
/// Invalid byte positions return None. A scalar boundary within a grapheme is
/// moved to the preceding/following grapheme edge. Endpoints stay at endpoints.
/// Full context matters for regional indicators and combining sequences.
pub fn previous_grapheme_boundary(text: &str, byte: usize) -> Option<usize> {
    if !text.is_char_boundary(byte) {
        return None;
    }
    GraphemeCursor::new(byte, text.len(), true)
        .prev_boundary(text, 0)
        .ok()
        .map(|boundary| boundary.unwrap_or(0))
}

pub fn next_grapheme_boundary(text: &str, byte: usize) -> Option<usize> {
    if !text.is_char_boundary(byte) {
        return None;
    }
    GraphemeCursor::new(byte, text.len(), true)
        .next_boundary(text, 0)
        .ok()
        .map(|boundary| boundary.unwrap_or(text.len()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn offsets_distinguish_bytes_scalars_and_utf16() {
        let parts: Vec<_> = segments("a\u{301}😀z", BoundaryKind::ExtendedGrapheme).collect();
        assert_eq!(
            parts[0].end,
            Offset {
                utf8_bytes: 3,
                unicode_scalars: 2,
                utf16_units: 2
            }
        );
        assert_eq!(
            parts[1].end,
            Offset {
                utf8_bytes: 7,
                unicode_scalars: 3,
                utf16_units: 4
            }
        );
        assert_eq!(parts[2].byte_range(), 7..8);
        assert_eq!(UNICODE_VERSION, (17, 0, 0));
    }
    #[test]
    fn caret_preserves_context_and_rejects_invalid_bytes() {
        let text = "🇺🇸🇨🇦a\u{301}";
        assert_eq!(next_grapheme_boundary(text, 4), Some(8));
        assert_eq!(previous_grapheme_boundary(text, 4), Some(0));
        assert_eq!(next_grapheme_boundary(text, 8), Some(16));
        assert_eq!(next_grapheme_boundary(text, 1), None);
        assert_eq!(previous_grapheme_boundary(text, text.len() + 1), None);
        assert_eq!(next_grapheme_boundary("", 0), Some(0));
        assert_eq!(previous_grapheme_boundary(text, 0), Some(0));
        assert_eq!(next_grapheme_boundary(text, text.len()), Some(text.len()));
    }
}
