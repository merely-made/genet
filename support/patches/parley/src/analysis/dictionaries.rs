// Copyright 2026 the Genet Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Segmenter dictionaries a host supplies at runtime (feature `complex-scripts`).
//!
//! ICU's line segmenter needs dictionaries to break Thai, Lao, Khmer and
//! Burmese, and its word segmenter those plus one for Chinese and Japanese.
//! Compiled in they are megabytes, so they travel as blobs a host loads when
//! text asks for them; until then those scripts break by the plain rules.
//!
//! A blob is [`MAGIC`] followed by entries of
//! `[group: u8][name length: u8][name][trie length in u16s: u32 LE][trie: u16 LE…]`,
//! the group being [`SOUTHEAST_ASIAN`] or [`CHINESE_JAPANESE`] and the name
//! ICU's dictionary attribute (`thaidict`, `cjdict`, …).

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

use icu_provider::buf::BufferMarker;
use icu_provider::prelude::*;
use icu_segmenter::provider::{
    SegmenterDictionaryAutoV1, SegmenterDictionaryExtendedV1, UCharDictionaryBreakData,
};
use zerovec::ZeroVec;

/// The first bytes of every dictionary blob.
pub const MAGIC: &[u8; 8] = b"GSEGDIC1";
/// The group byte of a Thai, Lao, Khmer or Burmese dictionary.
pub const SOUTHEAST_ASIAN: u8 = 0;
/// The group byte of the Chinese and Japanese dictionary.
pub const CHINESE_JAPANESE: u8 = 1;

/// Which dictionary groups text wants, or a context holds.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SegmenterDictionaries {
    /// Thai, Lao, Khmer and Burmese: line and word breaks.
    pub southeast_asian: bool,
    /// Chinese and Japanese: word breaks only (line breaks need no dictionary).
    pub chinese_japanese: bool,
}

impl SegmenterDictionaries {
    /// Whether neither group is set.
    pub fn is_empty(self) -> bool {
        !self.southeast_asian && !self.chinese_japanese
    }

    /// The groups in `self` that `held` lacks.
    pub fn missing_from(self, held: Self) -> Self {
        Self {
            southeast_asian: self.southeast_asian && !held.southeast_asian,
            chinese_japanese: self.chinese_japanese && !held.chinese_japanese,
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            southeast_asian: self.southeast_asian || other.southeast_asian,
            chinese_japanese: self.chinese_japanese || other.chinese_japanese,
        }
    }

    /// The groups `text` would break better with.
    pub fn wanted_by(text: &str) -> Self {
        let mut wanted = Self::default();
        if text.is_ascii() {
            return wanted;
        }
        for c in text.chars() {
            match u32::from(c) {
                // Thai, Lao; Myanmar and its extensions; Khmer and its symbols.
                0x0E00..=0x0EFF
                | 0x1000..=0x109F
                | 0xA9E0..=0xA9FF
                | 0xAA60..=0xAA7F
                | 0x1780..=0x17FF
                | 0x19E0..=0x19FF => wanted.southeast_asian = true,
                // Hiragana, Katakana, CJK ideographs and their extensions,
                // halfwidth Katakana.
                0x3040..=0x30FF
                | 0x31F0..=0x31FF
                | 0x3400..=0x4DBF
                | 0x4E00..=0x9FFF
                | 0xF900..=0xFAFF
                | 0xFF66..=0xFF9F
                | 0x20000..=0x3FFFF => wanted.chinese_japanese = true,
                _ => {},
            }
            if wanted.southeast_asian && wanted.chinese_japanese {
                break;
            }
        }
        wanted
    }
}

/// Why a dictionary blob was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DictionaryBlobError {
    /// The blob does not start with [`MAGIC`].
    NotADictionaryBlob,
    /// An entry runs past the end of the blob.
    Truncated,
    /// An entry names a group other than the two known ones.
    UnknownGroup(u8),
    /// An entry's name is not UTF-8.
    BadName,
}

impl fmt::Display for DictionaryBlobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotADictionaryBlob => write!(f, "not a segmenter dictionary blob"),
            Self::Truncated => write!(f, "the segmenter dictionary blob is truncated"),
            Self::UnknownGroup(group) => write!(f, "unknown segmenter dictionary group {group}"),
            Self::BadName => write!(f, "a segmenter dictionary name is not UTF-8"),
        }
    }
}

impl core::error::Error for DictionaryBlobError {}

/// Write a blob from `(group, name, trie)` entries.
pub fn encode_blob<'a>(entries: impl IntoIterator<Item = (u8, &'a str, &'a [u16])>) -> Vec<u8> {
    let mut blob = MAGIC.to_vec();
    for (group, name, trie) in entries {
        blob.push(group);
        blob.push(u8::try_from(name.len()).expect("dictionary names are short"));
        blob.extend_from_slice(name.as_bytes());
        let len = u32::try_from(trie.len()).expect("a dictionary fits in u32 units");
        blob.extend_from_slice(&len.to_le_bytes());
        for unit in trie {
            blob.extend_from_slice(&unit.to_le_bytes());
        }
    }
    blob
}

struct Entry {
    group: u8,
    name: String,
    trie: Range<usize>,
}

/// A checked dictionary blob. Cloning shares the bytes, so a host parses a
/// blob once and hands it to every [`LayoutContext`](crate::LayoutContext) it makes.
#[derive(Clone)]
pub struct DictionaryBlob {
    bytes: DataPayload<BufferMarker>,
    entries: Arc<[Entry]>,
    dictionaries: SegmenterDictionaries,
}

impl DictionaryBlob {
    /// Check `bytes` as a dictionary blob.
    pub fn new(bytes: Vec<u8>) -> Result<Self, DictionaryBlobError> {
        let entries = parse(&bytes)?;
        let mut dictionaries = SegmenterDictionaries::default();
        for entry in &entries {
            match entry.group {
                SOUTHEAST_ASIAN => dictionaries.southeast_asian = true,
                _ => dictionaries.chinese_japanese = true,
            }
        }
        Ok(Self {
            bytes: DataPayload::from_owned_buffer(bytes.into_boxed_slice()),
            entries: entries.into(),
            dictionaries,
        })
    }

    /// The groups this blob carries.
    pub fn dictionaries(&self) -> SegmenterDictionaries {
        self.dictionaries
    }
}

impl fmt::Debug for DictionaryBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DictionaryBlob")
            .field("dictionaries", &self.dictionaries)
            .field("entries", &self.entries.len())
            .finish_non_exhaustive()
    }
}

/// Every blob loaded so far.
///
/// ICU's word segmenter replaces its Chinese and Japanese dictionary on each
/// load, so every load hands it everything held rather than the newest blob.
#[derive(Default)]
pub(crate) struct DictionaryProvider {
    blobs: Vec<DictionaryBlob>,
    held: SegmenterDictionaries,
}

impl DictionaryProvider {
    pub(crate) fn held(&self) -> SegmenterDictionaries {
        self.held
    }

    /// Keep `blob`, unless this provider already holds it.
    pub(crate) fn add(&mut self, blob: &DictionaryBlob) -> bool {
        if self
            .blobs
            .iter()
            .any(|held| Arc::ptr_eq(&held.entries, &blob.entries))
        {
            return false;
        }
        self.blobs.push(blob.clone());
        self.held = self.held.union(blob.dictionaries);
        true
    }

    fn load<M>(&self, group: u8, req: DataRequest<'_>) -> Result<DataResponse<M>, DataError>
    where
        M: DataMarker<DataStruct = UCharDictionaryBreakData<'static>>,
    {
        let wanted = req.id.marker_attributes.as_str();
        // The newest blob wins when two carry the same dictionary.
        let (blob, entry) = self
            .blobs
            .iter()
            .rev()
            .find_map(|blob| {
                let entry = blob
                    .entries
                    .iter()
                    .find(|entry| entry.group == group && entry.name == wanted)?;
                Some((blob, entry))
            })
            .ok_or_else(|| DataErrorKind::IdentifierNotFound.with_req(M::INFO, req))?;
        let range = entry.trie.clone();
        let payload = blob.bytes.clone().map_project(move |bytes, _| {
            UCharDictionaryBreakData {
                // Lengths were checked when the blob was made.
                trie_data: ZeroVec::parse_bytes(&bytes[range]).expect("checked u16 trie"),
            }
        });
        Ok(DataResponse {
            metadata: DataResponseMetadata::default(),
            payload,
        })
    }
}

impl DataProvider<SegmenterDictionaryExtendedV1> for DictionaryProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<SegmenterDictionaryExtendedV1>, DataError> {
        self.load(SOUTHEAST_ASIAN, req)
    }
}

impl DataProvider<SegmenterDictionaryAutoV1> for DictionaryProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<SegmenterDictionaryAutoV1>, DataError> {
        self.load(CHINESE_JAPANESE, req)
    }
}

fn parse(blob: &[u8]) -> Result<Vec<Entry>, DictionaryBlobError> {
    if !blob.starts_with(MAGIC) {
        return Err(DictionaryBlobError::NotADictionaryBlob);
    }
    let mut at = MAGIC.len();
    let mut entries = Vec::new();
    let take = |at: &mut usize, n: usize| -> Result<Range<usize>, DictionaryBlobError> {
        let end = at.checked_add(n).filter(|end| *end <= blob.len());
        let end = end.ok_or(DictionaryBlobError::Truncated)?;
        let range = *at..end;
        *at = end;
        Ok(range)
    };
    while at < blob.len() {
        let group = blob[take(&mut at, 1)?.start];
        if group != SOUTHEAST_ASIAN && group != CHINESE_JAPANESE {
            return Err(DictionaryBlobError::UnknownGroup(group));
        }
        let name_len = usize::from(blob[take(&mut at, 1)?.start]);
        let name = core::str::from_utf8(&blob[take(&mut at, name_len)?])
            .map_err(|_| DictionaryBlobError::BadName)?
            .into();
        let len = &blob[take(&mut at, 4)?];
        let units = u32::from_le_bytes([len[0], len[1], len[2], len[3]]) as usize;
        let trie = take(
            &mut at,
            units.checked_mul(2).ok_or(DictionaryBlobError::Truncated)?,
        )?;
        entries.push(Entry { group, name, trie });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use icu_segmenter::options::{LineBreakOptions, WordBreakInvariantOptions};
    use icu_segmenter::provider::Baked;
    use icu_segmenter::{LineSegmenter, WordSegmenter};

    /// The baked dictionaries, as a host's blobs would carry them.
    pub(crate) fn baked_blobs() -> (Vec<u8>, Vec<u8>) {
        fn trie<M>(name: &str) -> Vec<u16>
        where
            M: DataMarker<DataStruct = UCharDictionaryBreakData<'static>>,
            Baked: DataProvider<M>,
        {
            let attributes = DataMarkerAttributes::try_from_str(name).unwrap();
            let response: DataResponse<M> = Baked
                .load(DataRequest {
                    id: DataIdentifierBorrowed::for_marker_attributes(attributes),
                    ..Default::default()
                })
                .unwrap();
            response.payload.get().trie_data.to_vec()
        }
        let sea: Vec<_> = ["burmesedict", "khmerdict", "laodict", "thaidict"]
            .map(|name| (name, trie::<SegmenterDictionaryExtendedV1>(name)))
            .into();
        let cj = trie::<SegmenterDictionaryAutoV1>("cjdict");
        (
            encode_blob(sea.iter().map(|(n, t)| (SOUTHEAST_ASIAN, *n, t.as_slice()))),
            encode_blob([(CHINESE_JAPANESE, "cjdict", cj.as_slice())]),
        )
    }

    const SAMPLES: [&str; 6] = [
        "ภาษาไทยเป็นภาษาที่มีระดับเสียงของคำแน่นอน",
        "ພາສາລາວເປັນພາສາທາງການຂອງປະເທດລາວ",
        "ភាសាខ្មែរជាភាសាផ្លូវការរបស់ប្រទេសកម្ពុជា",
        "မြန်မာဘာသာစကားသည်မြန်မာနိုင်ငံ၏ရုံးသုံးဘာသာစကားဖြစ်သည်",
        "中华人民共和国是一个位于东亚的国家",
        "日本語は日本の公用語として使われています",
    ];

    #[test]
    fn loaded_blobs_break_as_the_baked_dictionaries_do() {
        let (sea, cj) = baked_blobs();
        let sea = DictionaryBlob::new(sea).unwrap();
        assert_eq!(
            sea.dictionaries(),
            SegmenterDictionaries {
                southeast_asian: true,
                chinese_japanese: false
            }
        );
        let mut provider = DictionaryProvider::default();
        assert!(provider.add(&sea));
        assert!(
            !provider.add(&sea.clone()),
            "a blob already held is not added again"
        );
        assert!(provider.add(&DictionaryBlob::new(cj).unwrap()));

        let mut line = LineSegmenter::new_for_non_complex_scripts(LineBreakOptions::default())
            .static_to_owned();
        line.load_dictionary_unstable(&provider).unwrap();
        let mut word =
            WordSegmenter::new_for_non_complex_scripts(WordBreakInvariantOptions::default())
                .static_to_owned();
        word.load_dictionary_unstable(&provider).unwrap();
        let baked_line = LineSegmenter::new_dictionary(LineBreakOptions::default());
        let baked_word = WordSegmenter::new_dictionary(WordBreakInvariantOptions::default());
        let plain_word =
            WordSegmenter::new_for_non_complex_scripts(WordBreakInvariantOptions::default());
        for text in SAMPLES {
            let loaded: Vec<_> = line.as_borrowed().segment_str(text).collect();
            assert_eq!(
                loaded,
                baked_line.segment_str(text).collect::<Vec<_>>(),
                "{text}"
            );
            let loaded: Vec<_> = word.as_borrowed().segment_str(text).collect();
            assert_eq!(
                loaded,
                baked_word.segment_str(text).collect::<Vec<_>>(),
                "{text}"
            );
            assert_ne!(
                loaded,
                plain_word.segment_str(text).collect::<Vec<_>>(),
                "the dictionary changes {text}"
            );
        }
    }

    fn boundaries(lcx: &mut crate::LayoutContext, text: &str) -> Vec<crate::analysis::Boundary> {
        let mut fcx = crate::FontContext::default();
        let builder = lcx.ranged_builder(&mut fcx, text, 1.0, true);
        _ = builder.build(text);
        lcx.info.iter().map(|(info, _)| info.boundary).collect()
    }

    #[test]
    fn a_context_asks_for_dictionaries_and_breaks_with_them_once_loaded() {
        fn send<T: Send>() {}
        send::<crate::LayoutContext>();
        send::<DictionaryBlob>();

        let thai = SAMPLES[0];
        let mut lcx = crate::LayoutContext::new();
        assert!(lcx.wanted_segmenter_dictionaries().is_empty());
        let plain = boundaries(&mut lcx, thai);
        let wanted = lcx.wanted_segmenter_dictionaries();
        assert!(wanted.southeast_asian && !wanted.chinese_japanese);

        let (sea, _) = baked_blobs();
        let sea = DictionaryBlob::new(sea).unwrap();
        lcx.load_segmenter_dictionaries(&sea);
        lcx.load_segmenter_dictionaries(&sea);
        assert!(lcx.wanted_segmenter_dictionaries().is_empty());
        assert!(lcx.segmenter_dictionaries().southeast_asian);
        let loaded = boundaries(&mut lcx, thai);
        assert_eq!(loaded.len(), plain.len());
        assert_ne!(loaded, plain, "the dictionary breaks Thai words");
    }

    #[test]
    fn text_names_the_dictionaries_it_wants() {
        assert!(SegmenterDictionaries::wanted_by("plain words, ½ é").is_empty());
        let thai = SegmenterDictionaries::wanted_by(SAMPLES[0]);
        assert!(thai.southeast_asian && !thai.chinese_japanese);
        let both = SegmenterDictionaries::wanted_by("ไทย 日本");
        assert!(both.southeast_asian && both.chinese_japanese);
        assert!(both.missing_from(thai).chinese_japanese);
        assert!(!both.missing_from(thai).southeast_asian);
    }

    #[test]
    fn malformed_blobs_are_refused() {
        assert_eq!(
            DictionaryBlob::new(b"not a blob".to_vec()).unwrap_err(),
            DictionaryBlobError::NotADictionaryBlob
        );
        let mut blob = encode_blob([(SOUTHEAST_ASIAN, "thaidict", [1_u16, 2, 3].as_slice())]);
        blob.pop();
        assert_eq!(
            DictionaryBlob::new(blob).unwrap_err(),
            DictionaryBlobError::Truncated
        );
        let mut blob = encode_blob([(SOUTHEAST_ASIAN, "thaidict", [1_u16].as_slice())]);
        blob[MAGIC.len()] = 9;
        assert_eq!(
            DictionaryBlob::new(blob).unwrap_err(),
            DictionaryBlobError::UnknownGroup(9)
        );
    }
}
