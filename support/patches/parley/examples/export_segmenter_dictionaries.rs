// Copyright 2026 the Genet Authors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Write ICU's segmenter dictionaries as the blobs a host loads at runtime:
//! `segmenter-southeast-asian.dict` (Thai, Lao, Khmer, Burmese) and
//! `segmenter-chinese-japanese.dict`, into the directory given.
//!
//! `cargo run -p genet-parley --features complex-scripts --example export_segmenter_dictionaries -- <dir>`

use icu_provider::prelude::*;
use icu_segmenter::provider::{
    Baked, SegmenterDictionaryAutoV1, SegmenterDictionaryExtendedV1, UCharDictionaryBreakData,
};
use parley::segmenter_dictionaries::{CHINESE_JAPANESE, SOUTHEAST_ASIAN, encode_blob};

fn trie<M>(name: &str) -> Vec<u16>
where
    M: DataMarker<DataStruct = UCharDictionaryBreakData<'static>>,
    Baked: DataProvider<M>,
{
    let attributes = DataMarkerAttributes::try_from_str(name).expect("a dictionary name");
    let response: DataResponse<M> = Baked
        .load(DataRequest {
            id: DataIdentifierBorrowed::for_marker_attributes(attributes),
            ..Default::default()
        })
        .unwrap_or_else(|error| panic!("{name}: {error}"));
    response.payload.get().trie_data.to_vec()
}

fn main() {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).expect("an output directory"));
    let sea: Vec<_> = ["burmesedict", "khmerdict", "laodict", "thaidict"]
        .map(|name| (name, trie::<SegmenterDictionaryExtendedV1>(name)))
        .into();
    let cj = trie::<SegmenterDictionaryAutoV1>("cjdict");
    for (file, blob) in [
        (
            "segmenter-southeast-asian.dict",
            encode_blob(
                sea.iter()
                    .map(|(name, trie)| (SOUTHEAST_ASIAN, *name, trie.as_slice())),
            ),
        ),
        (
            "segmenter-chinese-japanese.dict",
            encode_blob([(CHINESE_JAPANESE, "cjdict", cj.as_slice())]),
        ),
    ] {
        std::fs::write(dir.join(file), &blob).expect("write a dictionary blob");
        println!("{file}: {} bytes", blob.len());
    }
}
