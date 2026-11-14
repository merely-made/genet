// SPDX-License-Identifier: MPL-2.0
//! Subresource Integrity: parse `integrity` metadata and match bytes against it.
//!
//! One routine for every Genet caller (fetch and `<script integrity>`), written
//! against the SRI "parse metadata", "get the strongest metadata" and "do bytes
//! match" steps. Where the steps leave room, the WPT `subresource-integrity/`
//! and `fetch/api/basic/integrity.sub.any.js` cases decide; the unit tests name
//! the case behind each behaviour.
//!
//! - Tokens are split on ASCII whitespace; a `?options` suffix is dropped.
//! - A token is `alg-value`. Unknown algorithms are ignored.
//! - A value that is not base64 (standard or URL-safe alphabet, padding
//!   optional) is ignored, like an unknown algorithm.
//! - Only the strongest algorithm present governs (sha512 > sha384 > sha256);
//!   the bytes match when their digest equals any of its values.
//! - No usable token means no requirement: the bytes match.
#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use base64::Engine as _;
use base64::alphabet;
use base64::engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig};
use sha2::Digest as _;

/// A hash algorithm SRI supports, ordered weakest to strongest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Algorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl Algorithm {
    /// The algorithm a metadata token names, matched case-sensitively.
    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "sha256" => Some(Self::Sha256),
            "sha384" => Some(Self::Sha384),
            "sha512" => Some(Self::Sha512),
            _ => None,
        }
    }

    /// The digest of `bytes` under this algorithm.
    pub fn digest(self, bytes: &[u8]) -> Vec<u8> {
        match self {
            Self::Sha256 => sha2::Sha256::digest(bytes).to_vec(),
            Self::Sha384 => sha2::Sha384::digest(bytes).to_vec(),
            Self::Sha512 => sha2::Sha512::digest(bytes).to_vec(),
        }
    }
}

/// The strongest algorithm in a metadata string, with every expected digest
/// listed for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Strongest {
    pub algorithm: Algorithm,
    pub digests: Vec<Vec<u8>>,
}

impl Strongest {
    /// Whether the digest of `bytes` equals any expected digest.
    pub fn matches(&self, bytes: &[u8]) -> bool {
        self.digests.contains(&self.algorithm.digest(bytes))
    }
}

const fn lenient(alphabet: &'static alphabet::Alphabet) -> GeneralPurpose {
    GeneralPurpose::new(
        alphabet,
        GeneralPurposeConfig::new()
            .with_decode_padding_mode(DecodePaddingMode::Indifferent)
            .with_decode_allow_trailing_bits(true),
    )
}

const STANDARD: GeneralPurpose = lenient(&alphabet::STANDARD);
const URL_SAFE: GeneralPurpose = lenient(&alphabet::URL_SAFE);

/// One token's algorithm and decoded digest, or `None` when it is unusable.
fn parse_token(token: &str) -> Option<(Algorithm, Vec<u8>)> {
    let expression = token.split('?').next().unwrap_or(token);
    let (algorithm, value) = expression.split_once('-')?;
    let algorithm = Algorithm::from_token(algorithm)?;
    let digest = STANDARD
        .decode(value)
        .or_else(|_| URL_SAFE.decode(value))
        .ok()?;
    Some((algorithm, digest))
}

/// Parse `metadata` and keep its strongest algorithm's digests. `None` when no
/// token is usable, which imposes no requirement.
pub fn parse(metadata: &str) -> Option<Strongest> {
    let mut strongest: Option<Strongest> = None;
    for (algorithm, digest) in metadata.split_ascii_whitespace().filter_map(parse_token) {
        match &mut strongest {
            Some(set) if set.algorithm == algorithm => set.digests.push(digest),
            Some(set) if set.algorithm > algorithm => {},
            _ => {
                strongest = Some(Strongest {
                    algorithm,
                    digests: alloc::vec![digest],
                });
            },
        }
    }
    strongest
}

/// Whether `metadata` imposes a requirement at all. A response whose body
/// cannot be read (an opaque response) fails an enforced check.
pub fn is_enforced(metadata: &str) -> bool {
    parse(metadata).is_some()
}

/// Whether `bytes` satisfy `metadata`.
pub fn matches(metadata: &str, bytes: &[u8]) -> bool {
    parse(metadata).is_none_or(|strongest| strongest.matches(bytes))
}

#[cfg(test)]
mod tests;
