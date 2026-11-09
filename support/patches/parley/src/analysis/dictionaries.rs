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
//! `[group: u8][name length: u8][name][kind: u8][trie…]`, the group being
//! [`SOUTHEAST_ASIAN`] or [`CHINESE_JAPANESE`] and the name ICU's dictionary
//! attribute (`thaidict`, `cjdict`, …). The kind is [`KIND_CHAR16_TRIE`], then
//! `[length in u16s: u32 LE][u16 LE…]`, or [`KIND_ZERO_TRIE`], then
//! `[alphabet length in u32s: u32 LE][u32 LE…][trie length: u32 LE][trie bytes]`,
//! the alphabet being the inversion list of the trie's code points. Blobs
//! starting with [`MAGIC_V1`], from before ICU 2.4, hold only Char16 tries and
//! no kind byte; they still load.

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::ops::Range;

use icu_collections::char16trie::Char16Trie;
use icu_collections::codepointinvlist::CodePointInversionList;
use icu_provider::buf::BufferMarker;
use icu_provider::prelude::*;
use icu_segmenter::provider::{
    DictionaryBreakData, SegmenterDictionaryAutoV2, SegmenterDictionaryExtendedV2,
};
use zerotrie::ZeroTrieSimpleAscii;
use zerovec::ZeroVec;

/// The first bytes of every dictionary blob this crate writes.
pub const MAGIC: &[u8; 8] = b"GSEGDIC2";
/// The first bytes of a blob from before ICU 2.4, which holds Char16 tries only.
pub const MAGIC_V1: &[u8; 8] = b"GSEGDIC1";
/// The kind byte of a dictionary stored as a Char16 trie.
pub const KIND_CHAR16_TRIE: u8 = 0;
/// The kind byte of a dictionary stored as an alphabet and a zero trie.
pub const KIND_ZERO_TRIE: u8 = 1;
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
    /// An entry names a trie kind other than the two known ones.
    UnknownKind(u8),
    /// An entry's trie does not parse.
    BadTrie,
}

impl fmt::Display for DictionaryBlobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotADictionaryBlob => write!(f, "not a segmenter dictionary blob"),
            Self::Truncated => write!(f, "the segmenter dictionary blob is truncated"),
            Self::UnknownGroup(group) => write!(f, "unknown segmenter dictionary group {group}"),
            Self::BadName => write!(f, "a segmenter dictionary name is not UTF-8"),
            Self::UnknownKind(kind) => write!(f, "unknown segmenter dictionary trie kind {kind}"),
            Self::BadTrie => write!(f, "a segmenter dictionary trie is malformed"),
        }
    }
}

impl core::error::Error for DictionaryBlobError {}

/// One dictionary's trie, as a blob stores it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DictionaryTrie<'a> {
    /// A Char16 trie's `u16` units.
    Char16(&'a [u16]),
    /// A zero trie's bytes, and the inversion list of the code points it is indexed by.
    ZeroTrie {
        /// The alphabet's inversion list.
        alphabet: &'a [u32],
        /// The trie's bytes.
        trie: &'a [u8],
    },
}

/// The trie of an ICU dictionary payload, owned, for [`encode_blob`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnedDictionaryTrie {
    /// A Char16 trie's `u16` units.
    Char16(Vec<u16>),
    /// A zero trie's alphabet inversion list and bytes.
    ZeroTrie {
        /// The alphabet's inversion list.
        alphabet: Vec<u32>,
        /// The trie's bytes.
        trie: Vec<u8>,
    },
}

impl OwnedDictionaryTrie {
    /// Take the trie out of an ICU dictionary payload.
    pub fn from_data(data: &DictionaryBreakData<'_>) -> Self {
        match data {
            DictionaryBreakData::Char16Trie(trie) => Self::Char16(trie.data.to_vec()),
            DictionaryBreakData::ZeroTrie { alphabet, trie } => Self::ZeroTrie {
                alphabet: alphabet.get_inversion_list_vec(),
                trie: trie.as_bytes().to_vec(),
            },
            // The enum is not exhaustive in ICU's eyes; a new kind needs a new blob kind.
            #[allow(unreachable_patterns)]
            _ => panic!("a dictionary kind this blob format does not carry"),
        }
    }

    /// Borrow as a [`DictionaryTrie`].
    pub fn as_trie(&self) -> DictionaryTrie<'_> {
        match self {
            Self::Char16(units) => DictionaryTrie::Char16(units),
            Self::ZeroTrie { alphabet, trie } => DictionaryTrie::ZeroTrie { alphabet, trie },
        }
    }
}

fn push_len(blob: &mut Vec<u8>, len: usize) {
    let len = u32::try_from(len).expect("a dictionary fits in u32 units");
    blob.extend_from_slice(&len.to_le_bytes());
}

/// Write a blob from `(group, name, trie)` entries.
pub fn encode_blob<'a>(
    entries: impl IntoIterator<Item = (u8, &'a str, DictionaryTrie<'a>)>,
) -> Vec<u8> {
    let mut blob = MAGIC.to_vec();
    for (group, name, trie) in entries {
        blob.push(group);
        blob.push(u8::try_from(name.len()).expect("dictionary names are short"));
        blob.extend_from_slice(name.as_bytes());
        match trie {
            DictionaryTrie::Char16(units) => {
                blob.push(KIND_CHAR16_TRIE);
                push_len(&mut blob, units.len());
                for unit in units {
                    blob.extend_from_slice(&unit.to_le_bytes());
                }
            },
            DictionaryTrie::ZeroTrie { alphabet, trie } => {
                blob.push(KIND_ZERO_TRIE);
                push_len(&mut blob, alphabet.len());
                for point in alphabet {
                    blob.extend_from_slice(&point.to_le_bytes());
                }
                push_len(&mut blob, trie.len());
                blob.extend_from_slice(trie);
            },
        }
    }
    blob
}

enum Trie {
    Char16(Range<usize>),
    ZeroTrie {
        alphabet: Vec<u32>,
        trie: Range<usize>,
    },
}

struct Entry {
    group: u8,
    name: String,
    trie: Trie,
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
        M: DataMarker<DataStruct = DictionaryBreakData<'static>>,
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
        let payload = match &entry.trie {
            Trie::Char16(range) => {
                let range = range.clone();
                blob.bytes.clone().map_project(move |bytes, _| {
                    DictionaryBreakData::Char16Trie(Char16Trie::new(
                        // Lengths were checked when the blob was made.
                        ZeroVec::parse_bytes(&bytes[range]).expect("checked u16 trie"),
                    ))
                })
            },
            Trie::ZeroTrie { alphabet, trie } => {
                let (alphabet, trie) = (alphabet.clone(), trie.clone());
                blob.bytes.clone().map_project(move |bytes, _| {
                    DictionaryBreakData::ZeroTrie {
                        // The alphabet was checked when the blob was made.
                        alphabet: CodePointInversionList::try_from_u32_inversion_list_slice(
                            &alphabet,
                        )
                        .expect("checked alphabet"),
                        trie: ZeroTrieSimpleAscii::from_store(ZeroVec::new_borrowed(&bytes[trie])),
                    }
                })
            },
        };
        Ok(DataResponse {
            metadata: DataResponseMetadata::default(),
            payload,
        })
    }
}

impl DataProvider<SegmenterDictionaryExtendedV2> for DictionaryProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<SegmenterDictionaryExtendedV2>, DataError> {
        self.load(SOUTHEAST_ASIAN, req)
    }
}

impl DataProvider<SegmenterDictionaryAutoV2> for DictionaryProvider {
    fn load(
        &self,
        req: DataRequest<'_>,
    ) -> Result<DataResponse<SegmenterDictionaryAutoV2>, DataError> {
        self.load(CHINESE_JAPANESE, req)
    }
}

fn parse(blob: &[u8]) -> Result<Vec<Entry>, DictionaryBlobError> {
    let v1 = blob.starts_with(MAGIC_V1);
    if !v1 && !blob.starts_with(MAGIC) {
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
    let take_len = |at: &mut usize| -> Result<usize, DictionaryBlobError> {
        let len = &blob[take(at, 4)?];
        Ok(u32::from_le_bytes([len[0], len[1], len[2], len[3]]) as usize)
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
        let kind = if v1 {
            KIND_CHAR16_TRIE
        } else {
            blob[take(&mut at, 1)?.start]
        };
        let trie = match kind {
            KIND_CHAR16_TRIE => {
                let units = take_len(&mut at)?;
                Trie::Char16(take(
                    &mut at,
                    units.checked_mul(2).ok_or(DictionaryBlobError::Truncated)?,
                )?)
            },
            KIND_ZERO_TRIE => {
                let points = take_len(&mut at)?;
                let bytes = take(
                    &mut at,
                    points
                        .checked_mul(4)
                        .ok_or(DictionaryBlobError::Truncated)?,
                )?;
                let alphabet: Vec<u32> = blob[bytes]
                    .chunks_exact(4)
                    .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                CodePointInversionList::try_from_u32_inversion_list_slice(&alphabet)
                    .map_err(|_| DictionaryBlobError::BadTrie)?;
                let len = take_len(&mut at)?;
                Trie::ZeroTrie {
                    alphabet,
                    trie: take(&mut at, len)?,
                }
            },
            kind => return Err(DictionaryBlobError::UnknownKind(kind)),
        };
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
        fn trie<M>(name: &str) -> OwnedDictionaryTrie
        where
            M: DataMarker<DataStruct = DictionaryBreakData<'static>>,
            Baked: DataProvider<M>,
        {
            let attributes = DataMarkerAttributes::try_from_str(name).unwrap();
            let response: DataResponse<M> = Baked
                .load(DataRequest {
                    id: DataIdentifierBorrowed::for_marker_attributes(attributes),
                    ..Default::default()
                })
                .unwrap();
            OwnedDictionaryTrie::from_data(response.payload.get())
        }
        let sea: Vec<_> = ["burmesedict", "khmerdict", "laodict", "thaidict"]
            .map(|name| (name, trie::<SegmenterDictionaryExtendedV2>(name)))
            .into();
        let cj = trie::<SegmenterDictionaryAutoV2>("cjdict");
        (
            encode_blob(sea.iter().map(|(n, t)| (SOUTHEAST_ASIAN, *n, t.as_trie()))),
            encode_blob([(CHINESE_JAPANESE, "cjdict", cj.as_trie())]),
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
        let mut blob = encode_blob([(
            SOUTHEAST_ASIAN,
            "thaidict",
            DictionaryTrie::Char16(&[1, 2, 3]),
        )]);
        blob.pop();
        assert_eq!(
            DictionaryBlob::new(blob).unwrap_err(),
            DictionaryBlobError::Truncated
        );
        let mut blob = encode_blob([(SOUTHEAST_ASIAN, "thaidict", DictionaryTrie::Char16(&[1]))]);
        blob[MAGIC.len()] = 9;
        assert_eq!(
            DictionaryBlob::new(blob).unwrap_err(),
            DictionaryBlobError::UnknownGroup(9)
        );
    }

    #[test]
    fn blobs_from_before_icu_2_4_still_load() {
        let units = [0x30_u16, 0x61, 0xb0, 0x62, 0x8064];
        let mut v1 = MAGIC_V1.to_vec();
        v1.extend_from_slice(&[CHINESE_JAPANESE, 6]);
        v1.extend_from_slice(b"cjdict");
        v1.extend_from_slice(&(units.len() as u32).to_le_bytes());
        for unit in units {
            v1.extend_from_slice(&unit.to_le_bytes());
        }
        let blob = DictionaryBlob::new(v1).unwrap();
        assert!(blob.dictionaries().chinese_japanese);
        let mut provider = DictionaryProvider::default();
        assert!(provider.add(&blob));
        let attributes = DataMarkerAttributes::try_from_str("cjdict").unwrap();
        let response: DataResponse<SegmenterDictionaryAutoV2> =
            DataProvider::<SegmenterDictionaryAutoV2>::load(
                &provider,
                DataRequest {
                    id: DataIdentifierBorrowed::for_marker_attributes(attributes),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(matches!(
            response.payload.get(),
            DictionaryBreakData::Char16Trie(trie) if trie.data.len() == units.len()
        ));
    }
}
