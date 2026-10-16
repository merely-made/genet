// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Image byte lookups keep CSS computed URLs separate from document `src`.

use std::collections::HashMap;

use genet_document_resources::resolve_url;

/// Byte lookup used by layout. Source-less callers retain authored-key behavior.
pub trait ImageSourceLookup {
    fn get(&self, key: &str) -> Option<&Vec<u8>>;

    fn get_document(&self, authored: &str) -> Option<&Vec<u8>> {
        self.get(authored)
    }
}

impl ImageSourceLookup for HashMap<String, Vec<u8>> {
    fn get(&self, key: &str) -> Option<&Vec<u8>> {
        HashMap::get(self, key)
    }
}

/// A document-bound view of the resolved image ledger. `img` uses this base;
/// computed CSS URLs already carry their owning stylesheet's base.
pub struct DocumentImageSources<'a> {
    sources: &'a HashMap<String, Vec<u8>>,
    document_base_url: Option<&'a str>,
}

impl<'a> DocumentImageSources<'a> {
    pub fn new(
        sources: &'a HashMap<String, Vec<u8>>,
        document_base_url: Option<&'a str>,
    ) -> Self {
        Self {
            sources,
            document_base_url,
        }
    }
}

impl ImageSourceLookup for DocumentImageSources<'_> {
    fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.sources.get(key)
    }

    fn get_document(&self, authored: &str) -> Option<&Vec<u8>> {
        self.sources
            .get(&document_image_key(self.document_base_url, authored))
    }
}

pub(crate) fn document_image_key(document_base_url: Option<&str>, authored: &str) -> String {
    resolve_url(document_base_url, authored)
}
