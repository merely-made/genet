// SPDX-License-Identifier: MPL-2.0
//! One test per behaviour the two former routines disagreed on, each naming the
//! WPT case (or "memory" of the SRI text) that decides it, plus known vectors.
//! `SI` is `subresource-integrity/subresource-integrity.html`; `FI` is
//! `fetch/api/basic/integrity.sub.any.js`.

use super::*;
use alloc::format;
use alloc::string::String;

/// `subresource-integrity/script.js`, the body every SI script case loads.
const SCRIPT: &[u8] = b"// nothing important.\n";
const SCRIPT_256: &str = "sha256-Bu681KMnQ15RYHFvsYdWumweeFAw0hJDTFt9seErghA=";
const SCRIPT_384: &str = "sha384-cINXh+nCzEHPWzXS7eoT+vYMBpyqczOybRLNU3XAButFWCRhHT5hLByIbPRqIm2f";
const SCRIPT_512: &str = "sha512-KZdenhzBd7X7Q/vmaOSyvFz1CGdoVt26xzCZjlkU9lfBEK+V/ougGys7iYDi0+tOHIQSQa87bIqx95R7GU7I9Q==";
/// SI's well-formed wrong hashes.
const WRONG_256: &str = "sha256-deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdead";
const WRONG_512: &str = "sha512-deadbeefspbnUnwooKGNNCb39nvg+EW0O9hDScTXeo/9pVZztLSUYU3LNV6H0lZapo8bCJUpyPPLAzE9fDzpxg==";

#[test]
fn known_vectors_of_abc() {
    assert!(matches(
        "sha256-ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0=",
        b"abc"
    ));
    assert!(matches(
        "sha384-ywB1P0WjXou1oD1pmsZQBycsMqsO3tFjGotgWkP/W+2AhgcroefMI1i67KE0yCWn",
        b"abc"
    ));
    assert!(matches(
        "sha512-3a81oZNherrMQXNJriBBMRLm+k6JqX6iCp7u5ktV05ohkpkqJ0/BqDa6PCOj/uu9RU1EI2Q86A4qmslPpUyknw==",
        b"abc"
    ));
    assert!(!matches(
        "sha256-ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0=",
        b"abd"
    ));
}

#[test]
fn wpt_script_vectors() {
    // SI "Same-origin with correct sha256/sha384/sha512 hash."
    for meta in [SCRIPT_256, SCRIPT_384, SCRIPT_512] {
        assert!(is_enforced(meta));
        assert!(matches(meta, SCRIPT), "{meta}");
    }
    // SI "Same-origin with incorrect hash."
    assert!(is_enforced(WRONG_256));
    assert!(!matches(WRONG_256, SCRIPT));
}

#[test]
fn undecodable_value_is_ignored() {
    // SI "Same-origin with non-Base64 hash." (`sha256-...`) loads; so does the
    // style case "Same-origin with non-Base64 integrity".
    assert!(!is_enforced("sha256-..."));
    assert!(matches("sha256-...", SCRIPT));
    assert!(matches("sha256-!!!", b"anything"));
}

#[test]
fn undecodable_stronger_beside_valid_weaker_leaves_weaker_governing() {
    // No direct WPT case. Follows from SI "non-Base64 hash": an undecodable
    // token is dropped in parsing, so it cannot be the strongest.
    let good = format!("{SCRIPT_256} sha512-...");
    assert!(matches(&good, SCRIPT));
    let bad = format!("{WRONG_256} sha512-...");
    assert!(!matches(&bad, SCRIPT));
}

#[test]
fn options_are_stripped_and_the_hash_checked() {
    // SI "Same-origin with correct hash, options."
    assert!(matches(&format!("{SCRIPT_256}?foo=bar?spam=eggs"), SCRIPT));
    // The stripped hash is still enforced, so a wrong one blocks.
    assert!(!matches(&format!("{WRONG_256}?foo=bar"), SCRIPT));
}

#[test]
fn base64url_and_missing_padding_are_accepted() {
    // FI "SHA-512 integrity with missing padding", "SHA-512 integrity base64url
    // encoded", "... base64url encoded with missing padding".
    let top = b"top";
    let padded = format!("sha512-{}", STANDARD.encode(sha2::Sha512::digest(top)));
    let unpadded = padded.trim_end_matches('=');
    let url: String = padded.replace('+', "-").replace('/', "_");
    let url_unpadded = url.trim_end_matches('=');
    for meta in [padded.as_str(), unpadded, url.as_str(), url_unpadded] {
        assert!(matches(meta, top), "{meta}");
    }
    // The FI constants themselves (top.txt is the three bytes "top").
    assert!(matches(
        "sha512-D6yns0qxG0E7-TwkevZ4Jt5t7Iy3ugmAajG_dlf6Pado1JqTyneKXICDiqFIkLMRExgtvg8PlxbKTkYfRejSOg",
        top
    ));
}

#[test]
fn mixed_alphabets_in_one_value_are_ignored() {
    // FORK, pending the owner: no WPT case and no confident recollection. A
    // value mixing `+`/`/` with `-`/`_` decodes under neither alphabet, so it is
    // ignored like any undecodable value. The former fetch routine accepted it.
    let meta = "sha256-ungWv48Bz-pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0=";
    assert!(!is_enforced(meta));
}

#[test]
fn unknown_algorithms_are_ignored() {
    // SI "Same-origin with unknown algorithm only." and "multiple sha256 hashes,
    // including unknown algorithm".
    assert!(!is_enforced(
        "foo666-U9WYDtBWkcHx13+9UKk/3Q5eoqDc4YGxYb07EPWzb9E="
    ));
    assert!(matches(
        "foo666-U9WYDtBWkcHx13+9UKk/3Q5eoqDc4YGxYb07EPWzb9E=",
        SCRIPT
    ));
    let meta = format!("{SCRIPT_256} foo666-deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdead");
    assert!(matches(&meta, SCRIPT));
    // Matched case-sensitively, as both former routines did (memory: the
    // algorithm tokens are lowercase).
    assert!(!is_enforced(
        "SHA256-Bu681KMnQ15RYHFvsYdWumweeFAw0hJDTFt9seErghA="
    ));
}

#[test]
fn empty_and_whitespace_metadata_impose_nothing() {
    // SI "Same-origin with empty integrity."; FI "Empty string integrity".
    // Whitespace only: no WPT case; parse metadata yields no tokens (memory).
    for meta in ["", "   ", "\t\n"] {
        assert!(!is_enforced(meta));
        assert!(matches(meta, SCRIPT));
    }
}

#[test]
fn tokens_split_on_ascii_whitespace_only() {
    // Memory: parse metadata splits on ASCII whitespace. A no-break space does
    // not separate two tokens, so this is one undecodable token.
    let meta = format!("{WRONG_256}\u{a0}{SCRIPT_256}");
    assert!(!is_enforced(&meta));
    let meta = format!("{WRONG_256}\t\n\x0c\r{SCRIPT_256}");
    assert!(matches(&meta, SCRIPT));
}

#[test]
fn any_value_of_the_strongest_algorithm_matches() {
    // SI "Same-origin with multiple sha256 hashes, including correct."; FI
    // "Multiple integrities: invalid as strong as valid".
    assert!(matches(&format!("{SCRIPT_256} {WRONG_256}"), SCRIPT));
    assert!(matches(&format!("{WRONG_256} {SCRIPT_256}"), SCRIPT));
}

#[test]
fn the_strongest_algorithm_governs() {
    // SI "Same-origin with sha256 mismatch, sha512 match" loads; "sha256 match,
    // sha512 mismatch" is blocked. FI "Multiple integrities: valid stronger than
    // invalid" / "invalid stronger than valid" agree.
    assert!(matches(&format!("{SCRIPT_512} {WRONG_256}"), SCRIPT));
    assert!(!matches(&format!("{WRONG_512} {SCRIPT_256}"), SCRIPT));
    let set = parse(&format!("{SCRIPT_256} {SCRIPT_384} {WRONG_256}")).unwrap();
    assert_eq!(set.algorithm, Algorithm::Sha384);
    assert_eq!(set.digests.len(), 1);
}

#[test]
fn enforcement_follows_usable_tokens() {
    // FI "SHA-* integrity for opaque response" must fail and "Empty string
    // integrity for opaque response" must pass: the opaque check asks this.
    assert!(is_enforced(SCRIPT_512));
    assert!(!is_enforced(""));
}

#[test]
fn every_wpt_blocking_value_is_well_formed() {
    // Each value a WPT case expects to block must parse, or it would be ignored
    // and the resource would load. SI script and style cases, FI invalid hashes.
    for meta in [
        WRONG_256,
        WRONG_512,
        "sha256-deadbeefcSLlbFZCj1OACLxTxVck2TOrBTEdUbwz1yU=",
        "sha256-deadbeef2S+pTRZgiw3DWrhC6JLDlt2zRyGpwH7unU8=",
        "sha256-deadbeef01Y0yKSx3/UoIKtIY2UQ9+H8WGyyMuOWOC0=",
        "sha256-deadbeefCzHgdJ7wOccM8L89n4bhcJMz3F+SPLT7YZk=",
        "sha512-deadbeef9wXDjd6Wq3H6nPAhI9zOvG7mJkUr03MTxaO+8ztTKnfJif42laL93Be/IF6YYZHHF4esitVYxiwpY2==",
        "sha256-dKUcPOn/AlUjWIwcHeHNqYXPlvyGiq+2dWOdFcE+24I=",
        "sha512-oUceBRNxPxnY60g/VtPCj2syT4wo4EZh2CgYdWy9veW8+OsReTXoh7dizMGZafvx9+QhMS39L/gIkxnPIn41Zg==",
    ] {
        assert!(is_enforced(meta), "{meta}");
        assert!(!matches(meta, SCRIPT), "{meta}");
    }
}
