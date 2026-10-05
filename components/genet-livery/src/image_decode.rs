// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The one place genet-livery decodes image bytes.
//!
//! Decoding is the default-on `image-decode` feature. Without it every image
//! fails to decode, which layout and paint already treat as an absent image:
//! no natural size and nothing painted. A host decoder can replace this module
//! later without touching either caller.

/// A decoded raster. Uninhabited without the `image-decode` feature.
#[cfg(feature = "image-decode")]
pub(crate) struct DecodedImage(image::DynamicImage);

/// A decoded raster. Uninhabited without the `image-decode` feature.
#[cfg(not(feature = "image-decode"))]
pub(crate) enum DecodedImage {}

/// Decodes `bytes` in any format the build supports, or `None`.
pub(crate) fn decode(bytes: &[u8]) -> Option<DecodedImage> {
    #[cfg(feature = "image-decode")]
    {
        image::load_from_memory(bytes).ok().map(DecodedImage)
    }
    #[cfg(not(feature = "image-decode"))]
    {
        let _ = bytes;
        None
    }
}

#[cfg(feature = "image-decode")]
impl DecodedImage {
    /// Natural width and height in image pixels.
    pub(crate) fn dimensions(&self) -> (u32, u32) {
        (self.0.width(), self.0.height())
    }

    /// Width, height and straight RGBA8 pixels, row-major.
    pub(crate) fn into_rgba8(self) -> (u32, u32, Vec<u8>) {
        let rgba = self.0.to_rgba8();
        let (width, height) = rgba.dimensions();
        (width, height, rgba.into_raw())
    }
}

#[cfg(not(feature = "image-decode"))]
impl DecodedImage {
    pub(crate) fn dimensions(&self) -> (u32, u32) {
        match *self {}
    }

    pub(crate) fn into_rgba8(self) -> (u32, u32, Vec<u8>) {
        match self {}
    }
}

#[cfg(test)]
mod tests {
    use super::decode;

    /// A 2x3 opaque blue RGBA PNG, built with zlib so the test needs no encoder.
    const BLUE_2X3_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x03, 0x08, 0x06, 0x00, 0x00, 0x00, 0xB9,
        0xEA, 0xDE, 0x81, 0x00, 0x00, 0x00, 0x10, 0x49, 0x44, 0x41, 0x54, 0x78, 0xDA, 0x63, 0x60,
        0x60, 0xF8, 0xFF, 0x1F, 0x82, 0xD1, 0x19, 0x00, 0x95, 0x85, 0x0B, 0xF5, 0x0D, 0x46, 0x61,
        0x32, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[cfg(feature = "image-decode")]
    #[test]
    fn decodes_size_and_pixels_with_the_feature() {
        let image = decode(BLUE_2X3_PNG).expect("feature build decodes PNG");
        assert_eq!(image.dimensions(), (2, 3));
        let (width, height, rgba) = image.into_rgba8();
        assert_eq!((width, height), (2, 3));
        assert_eq!(rgba.len(), 2 * 3 * 4);
        assert!(rgba.chunks(4).all(|pixel| pixel == [0, 0, 255, 255]));
    }

    #[cfg(not(feature = "image-decode"))]
    #[test]
    fn decodes_nothing_without_the_feature() {
        assert!(decode(BLUE_2X3_PNG).is_none());
    }
}
