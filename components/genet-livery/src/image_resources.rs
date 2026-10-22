// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resolved image sources and their lazily shared decoded pixels.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    sync::{Arc, OnceLock},
};

use genet_document_resources::resolve_url;

/// One immutable decoded raster. Cloning it shares the pixel allocation.
#[derive(Clone, Debug)]
pub struct ImagePixels {
    pub width: u32,
    pub height: u32,
    pub data: Arc<Vec<u8>>,
}

fn decode(bytes: &[u8]) -> Option<ImagePixels> {
    let (width, height, data) = crate::image_decode::decode(bytes)?.into_rgba8();
    Some(ImagePixels {
        width,
        height,
        data: Arc::new(data),
    })
}

fn decode_data_url(url: &str) -> Option<ImagePixels> {
    let bytes = data_url::DataUrl::process(url)
        .ok()?
        .decode_to_vec()
        .ok()?
        .0;
    decode(&bytes)
}

pub(crate) fn is_data_url(url: &str) -> bool {
    url.get(..5)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("data:"))
}

/// Image lookup used by layout and paint. A plain host map remains usable.
pub trait ImageSourceLookup {
    fn get(&self, key: &str) -> Option<&Vec<u8>>;

    fn get_document(&self, authored: &str) -> Option<&Vec<u8>> {
        self.get(authored)
    }

    fn decoded(&self, key: &str) -> Option<ImagePixels> {
        if let Ok(data_url) = data_url::DataUrl::process(key) {
            return decode(&data_url.decode_to_vec().ok()?.0);
        }
        decode(self.get(key)?)
    }

    fn decoded_document(&self, authored: &str) -> Option<ImagePixels> {
        if let Ok(data_url) = data_url::DataUrl::process(authored) {
            return decode(&data_url.decode_to_vec().ok()?.0);
        }
        decode(self.get_document(authored)?)
    }
}

impl ImageSourceLookup for HashMap<String, Vec<u8>> {
    fn get(&self, key: &str) -> Option<&Vec<u8>> {
        HashMap::get(self, key)
    }
}

struct ImageEntry {
    bytes: Vec<u8>,
    decoded: OnceLock<Option<ImagePixels>>,
    #[cfg(test)]
    decode_attempts: std::cell::Cell<usize>,
}

impl ImageEntry {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            decoded: OnceLock::new(),
            #[cfg(test)]
            decode_attempts: std::cell::Cell::new(0),
        }
    }

    fn decoded(&self) -> Option<ImagePixels> {
        self.decoded
            .get_or_init(|| {
                #[cfg(test)]
                self.decode_attempts.set(self.decode_attempts.get() + 1);
                decode(&self.bytes)
            })
            .clone()
    }
}

/// Document-owned encoded source ledger and per-version decoded cache.
#[derive(Default)]
pub(crate) struct ImageResources {
    sources: HashMap<String, ImageEntry>,
    data_urls: RefCell<HashMap<String, Option<ImagePixels>>>,
}

impl ImageResources {
    pub(crate) fn set(&mut self, url: String, bytes: Vec<u8>) -> bool {
        if self
            .sources
            .get(&url)
            .is_some_and(|entry| entry.bytes == bytes)
        {
            return false;
        }
        self.sources.insert(url, ImageEntry::new(bytes));
        true
    }

    pub(crate) fn replace(
        &mut self,
        resources: impl IntoIterator<Item = (String, Vec<u8>)>,
    ) -> bool {
        let next = resources.into_iter().collect::<HashMap<_, _>>();
        if self.sources.len() == next.len()
            && next.iter().all(|(key, bytes)| {
                self.sources
                    .get(key)
                    .is_some_and(|entry| &entry.bytes == bytes)
            })
        {
            return false;
        }
        self.sources
            .retain(|key, entry| next.get(key) == Some(&entry.bytes));
        for (key, bytes) in next {
            self.sources
                .entry(key)
                .or_insert_with(|| ImageEntry::new(bytes));
        }
        true
    }

    pub(crate) fn has_data_urls(&self) -> bool {
        !self.data_urls.borrow().is_empty()
    }

    /// Retain only URLs still referenced by the current DOM or computed
    /// styles. Repaint-only and local-layout frames use the same rule.
    pub(crate) fn retain_data_urls(&self, active: &HashSet<String>) {
        self.data_urls
            .borrow_mut()
            .retain(|key, _| active.contains(key));
    }

    fn data_url(&self, key: &str) -> Option<ImagePixels> {
        if let Some(cached) = self.data_urls.borrow().get(key) {
            return cached.clone();
        }
        let decoded = decode_data_url(key);
        self.data_urls
            .borrow_mut()
            .insert(key.to_owned(), decoded.clone());
        decoded
    }

    #[cfg(test)]
    pub(crate) fn cached_data_url_count(&self) -> usize {
        self.data_urls.borrow().len()
    }
}

impl ImageSourceLookup for ImageResources {
    fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.sources.get(key).map(|entry| &entry.bytes)
    }

    fn decoded(&self, key: &str) -> Option<ImagePixels> {
        if is_data_url(key) && data_url::DataUrl::process(key).is_ok() {
            self.data_url(key)
        } else {
            self.sources.get(key)?.decoded()
        }
    }

    fn decoded_document(&self, authored: &str) -> Option<ImagePixels> {
        self.decoded(authored)
    }
}

/// A document-bound view. `<img>` uses this base; computed CSS URLs already
/// carry their owning stylesheet's base.
pub struct DocumentImageSources<'a> {
    sources: &'a dyn ImageSourceLookup,
    document_base_url: Option<&'a str>,
}

impl<'a> DocumentImageSources<'a> {
    pub fn new(sources: &'a dyn ImageSourceLookup, document_base_url: Option<&'a str>) -> Self {
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

    fn decoded(&self, key: &str) -> Option<ImagePixels> {
        self.sources.decoded(key)
    }

    fn decoded_document(&self, authored: &str) -> Option<ImagePixels> {
        self.sources
            .decoded(&document_image_key(self.document_base_url, authored))
    }
}

pub(crate) fn document_image_key(document_base_url: Option<&str>, authored: &str) -> String {
    resolve_url(document_base_url, authored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "image-decode")]
    fn png() -> Vec<u8> {
        let pixels = image::RgbaImage::from_pixel(2, 3, image::Rgba([0, 0, 255, 255]));
        let mut bytes = Vec::new();
        pixels
            .write_to(
                &mut std::io::Cursor::new(&mut bytes),
                image::ImageFormat::Png,
            )
            .expect("encode fixture PNG");
        bytes
    }

    #[cfg(feature = "image-decode")]
    #[test]
    fn unchanged_source_shares_decode_across_document_views_and_replacement() {
        let bytes = png();
        let mut sources = ImageResources::default();
        assert!(sources.set("https://example.test/image.png".into(), bytes.clone()));
        let first = DocumentImageSources::new(&sources, Some("https://example.test/"))
            .decoded_document("image.png")
            .expect("intrinsic query decodes image");
        let paint = sources
            .decoded("https://example.test/image.png")
            .expect("paint lookup");
        assert!(Arc::ptr_eq(&first.data, &paint.data));
        assert!(!sources.replace([("https://example.test/image.png".into(), bytes)]));
        let next = sources
            .decoded("https://example.test/image.png")
            .expect("unchanged decode");
        assert!(Arc::ptr_eq(&first.data, &next.data));
        assert_eq!(
            sources.sources["https://example.test/image.png"]
                .decode_attempts
                .get(),
            1
        );
    }

    #[cfg(feature = "image-decode")]
    #[test]
    fn changed_or_removed_source_drops_old_decode() {
        let mut sources = ImageResources::default();
        let original = png();
        sources.set("image.png".into(), original.clone());
        let first = sources.decoded("image.png").expect("first version");
        let replacement = image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]));
        let mut replacement_bytes = Vec::new();
        replacement
            .write_to(
                &mut std::io::Cursor::new(&mut replacement_bytes),
                image::ImageFormat::Png,
            )
            .expect("encode replacement PNG");
        assert!(sources.replace([("image.png".into(), replacement_bytes)]));
        let second = sources.decoded("image.png").expect("replacement version");
        assert_eq!((second.width, second.height), (1, 1));
        assert!(!Arc::ptr_eq(&first.data, &second.data));
        assert!(sources.replace(std::iter::empty()));
        assert!(sources.decoded("image.png").is_none());
    }

    #[test]
    fn failed_decode_is_cached_until_source_changes() {
        let mut sources = ImageResources::default();
        sources.set("broken.png".into(), b"not an image".to_vec());
        assert!(sources.decoded("broken.png").is_none());
        assert!(sources.sources["broken.png"].decoded.get().is_some());
        assert_eq!(sources.sources["broken.png"].decode_attempts.get(), 1);
        assert!(sources.decoded("broken.png").is_none());
        assert!(!sources.set("broken.png".into(), b"not an image".to_vec()));
        assert!(sources.sources["broken.png"].decoded.get().is_some());
        assert!(sources.set("broken.png".into(), b"new version".to_vec()));
        assert!(sources.sources["broken.png"].decoded.get().is_none());
    }

    #[cfg(feature = "image-decode")]
    #[test]
    fn malformed_data_key_uses_replaceable_supplied_bytes() {
        let key = "data:;base64";
        assert!(data_url::DataUrl::process(key).is_err());
        let mut sources = ImageResources::default();
        sources.set(key.into(), png());
        let first = sources.decoded(key).expect("supplied malformed data key");
        assert_eq!((first.width, first.height), (2, 3));
        let red = image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]));
        let mut replacement = Vec::new();
        red.write_to(
            &mut std::io::Cursor::new(&mut replacement),
            image::ImageFormat::Png,
        )
        .expect("encode replacement");
        sources.replace([(key.into(), replacement)]);
        let changed = sources.decoded(key).expect("changed supplied data key");
        assert_eq!((changed.width, changed.height), (1, 1));
        assert!(!Arc::ptr_eq(&first.data, &changed.data));
        sources.replace(std::iter::empty());
        assert!(sources.decoded(key).is_none());
    }

    #[cfg(feature = "image-decode")]
    #[test]
    fn data_url_cache_reuses_active_source_and_retires_removed_source() {
        use base64::Engine as _;
        let url = format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(png())
        );
        let sources = ImageResources::default();
        let first = sources.decoded(&url).expect("data image");
        let second = sources.decoded(&url).expect("cached data image");
        assert!(Arc::ptr_eq(&first.data, &second.data));
        sources.retain_data_urls(&HashSet::from([url.clone()]));
        assert_eq!(sources.data_urls.borrow().len(), 1);
        let active = sources.decoded(&url).expect("active data image");
        assert!(Arc::ptr_eq(&first.data, &active.data));
        sources.retain_data_urls(&HashSet::new());
        assert!(sources.data_urls.borrow().is_empty());
    }

    #[cfg(not(feature = "image-decode"))]
    #[test]
    fn feature_off_caches_absent_pixels() {
        let mut sources = ImageResources::default();
        sources.set("image.png".into(), b"image bytes".to_vec());
        assert!(sources.decoded("image.png").is_none());
        assert!(sources.sources["image.png"].decoded.get().is_some());
    }
}
