// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Shared pure HTML form encoders. Entry lists remain untouched; file bodies
//! are immutable bytes and never pass through charset or newline conversion.

use crate::{FormEntry, FormEntryValue};
pub use encoding_rs::{Encoding, UTF_8};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodedFormBody {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

#[cfg(test)]
mod phase_c_boundary_fixtures {
    use super::*;
    use crate::{FormEntryValue, FormFile};
    use std::sync::Arc;
    // RED-only missing automatic-boundary adapter uses the existing literal
    // primitive. This is not claimed to be an existing collision detector.
    fn boundary_adapter(entries: &[FormEntry], encoding: &'static Encoding) -> String {
        select_multipart_boundary(entries, encoding)
    }
    fn auto_adapter(entries: &[FormEntry], encoding: &'static Encoding) -> EncodedFormBody {
        encode_multipart_auto(entries, encoding).unwrap()
    }
    // RED-only missing chunk scanner, explicitly independent of final body.
    fn chunk_boundary_adapter(chunks: &[&[u8]]) -> String {
        select_multipart_boundary_from_chunks(chunks.iter().copied())
    }
    fn text(name: &str, value: &str) -> FormEntry {
        FormEntry {
            name: name.into(),
            value: FormEntryValue::Text(value.into()),
        }
    }
    fn file(name: &str, filename: &str, mime: &str, bytes: &[u8]) -> FormEntry {
        FormEntry {
            name: name.into(),
            value: FormEntryValue::File {
                file: FormFile::new(filename.into(), mime.into(), Arc::from(bytes)).unwrap(),
                source_input: None,
            },
        }
    }
    #[test]
    fn phase_c_encoding_boundary_empty_deterministic_control() {
        assert_eq!(boundary_adapter(&[], UTF_8), "GenetBoundary0");
        assert_eq!(boundary_adapter(&[], UTF_8), boundary_adapter(&[], UTF_8));
        assert_eq!(auto_adapter(&[], UTF_8).bytes, b"--GenetBoundary0--\r\n");
    }
    #[test]
    fn phase_c_encoding_boundary_actual_text_delimiter_collision() {
        let entries = [text("q", "\r\n--GenetBoundary0\r\ninside")];
        assert_eq!(
            boundary_adapter(&entries, UTF_8),
            "GenetBoundary1",
            "missing capability: collision-free automatic boundary"
        );
        assert_eq!(auto_adapter(&entries,UTF_8).bytes,b"--GenetBoundary1\r\nContent-Disposition: form-data; name=\"q\"\r\n\r\n\r\n--GenetBoundary0\r\ninside\r\n--GenetBoundary1--\r\n");
    }
    #[test]
    fn phase_c_encoding_boundary_actual_binary_delimiter_collision() {
        let entries = [file(
            "upload",
            "a.bin",
            "application/octet-stream",
            b"\0\x80\xff\r\n--GenetBoundary0\r\n",
        )];
        assert_eq!(boundary_adapter(&entries, UTF_8), "GenetBoundary1");
        let expected=b"--GenetBoundary1\r\nContent-Disposition: form-data; name=\"upload\"; filename=\"a.bin\"\r\nContent-Type: application/octet-stream\r\n\r\n\0\x80\xff\r\n--GenetBoundary0\r\n\r\n--GenetBoundary1--\r\n";
        assert_eq!(auto_adapter(&entries, UTF_8).bytes, expected);
    }
    #[test]
    fn phase_c_encoding_boundary_headers_filename_and_mime_scanned() {
        for entry in [
            text("GenetBoundary0", "safe"),
            file(
                "x",
                "GenetBoundary0.bin",
                "application/octet-stream",
                b"safe",
            ),
            file("x", "a.bin", "genetboundary0", b"safe"),
        ] {
            let selected = boundary_adapter(&[entry.clone()], UTF_8);
            // MIME is ASCII-lowercased natively: lowercase is not a collision
            // with the case-sensitive candidate; this is a positive control.
            let expected = if matches!(&entry.value,FormEntryValue::File{file,..} if file.mime_type=="genetboundary0")
            {
                "GenetBoundary0"
            } else {
                "GenetBoundary1"
            };
            assert_eq!(selected, expected);
        }
    }
    #[test]
    fn phase_c_encoding_boundary_decimal_prefix_trap() {
        let entries = [text("q", "GenetBoundary0 GenetBoundary12")];
        assert_eq!(
            boundary_adapter(&entries, UTF_8),
            "GenetBoundary2",
            "candidate1 is a prefix of an actual part token12"
        );
    }
    #[test]
    fn phase_c_encoding_boundary_chunk_spanning_bytes() {
        assert_eq!(
            chunk_boundary_adapter(&[b"\r\n--GenetBound", b"ary0\r\n", b"GenetBoundary12"]),
            "GenetBoundary2",
            "carry match/digit state across chunks, before delimiters"
        );
    }
    #[test]
    fn phase_c_encoding_boundary_legacy_actual_encoded_parts() {
        let encoding = Encoding::for_label(b"windows-1252").unwrap();
        let entries = [text("éGenetBoundary0", "é\n😀")];
        assert_eq!(boundary_adapter(&entries, encoding), "GenetBoundary1");
        assert_eq!(auto_adapter(&entries,encoding).bytes,b"--GenetBoundary1\r\nContent-Disposition: form-data; name=\"\xE9GenetBoundary0\"\r\n\r\n\xE9\r\n&#128512;\r\n--GenetBoundary1--\r\n");
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormEncodingError {
    InvalidBoundary,
}

pub fn select_form_encoding(
    accept_charset: Option<&str>,
    document_encoding: &'static Encoding,
) -> &'static Encoding {
    let encoding = match accept_charset {
        None => document_encoding,
        Some(labels) => labels
            .split(|c| matches!(c, '\t' | '\n' | '\u{000c}' | '\r' | ' '))
            .filter_map(|label| Encoding::for_label(label.as_bytes()))
            .next()
            .unwrap_or(UTF_8),
    };
    encoding.output_encoding()
}

pub fn encode_urlencoded(entries: &[FormEntry], encoding: &'static Encoding) -> Vec<u8> {
    let mut result = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if index != 0 {
            result.push(b'&');
        }
        percent_encode(
            &encoded(&normalize_newlines(&entry.name), encoding),
            &mut result,
        );
        result.push(b'=');
        percent_encode(
            &encoded(&normalize_newlines(pair_value(entry)), encoding),
            &mut result,
        );
    }
    result
}

pub fn encode_text_plain(entries: &[FormEntry], encoding: &'static Encoding) -> Vec<u8> {
    let mut result = String::new();
    for entry in entries {
        result.push_str(&normalize_newlines(&entry.name));
        result.push('=');
        result.push_str(&normalize_newlines(pair_value(entry)));
        result.push_str("\r\n");
    }
    encoded(&result, encoding)
}

/// Caller supplies a generated boundary. Require a safe unquoted MIME token,
/// independent of any randomness provider or script engine.
pub fn encode_multipart(
    entries: &[FormEntry],
    encoding: &'static Encoding,
    boundary: &str,
) -> Result<EncodedFormBody, FormEncodingError> {
    if boundary.is_empty()
        || boundary.len() > 70
        || !boundary
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'\'' | b'+' | b'_' | b'-' | b'.'))
    {
        return Err(FormEncodingError::InvalidBoundary);
    }
    let mut bytes = Vec::new();
    for entry in entries {
        bytes.extend_from_slice(b"--"); bytes.extend_from_slice(boundary.as_bytes()); bytes.extend_from_slice(b"\r\n");
        bytes.extend_from_slice(&multipart_part(entry, encoding)); bytes.extend_from_slice(b"\r\n");
    }
    bytes.extend_from_slice(b"--");
    bytes.extend_from_slice(boundary.as_bytes());
    bytes.extend_from_slice(b"--\r\n");
    Ok(EncodedFormBody {
        bytes,
        content_type: format!("multipart/form-data; boundary={boundary}"),
    })
}

fn multipart_part(entry: &FormEntry, encoding: &'static Encoding) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"Content-Disposition: form-data; name=\"");
    escape_header(
        &encoded(&normalize_newlines(&entry.name), encoding),
        &mut bytes,
    );
    bytes.push(b'"');
    match &entry.value {
        FormEntryValue::Text(value) => {
            bytes.extend_from_slice(b"\r\n\r\n");
            bytes.extend_from_slice(&encoded(&normalize_newlines(value), encoding));
        },
        FormEntryValue::File { file, .. } => {
            bytes.extend_from_slice(b"; filename=\"");
            // Filename newlines are escaped directly, never normalized.
            escape_header(&encoded(&file.name, encoding), &mut bytes);
            bytes.extend_from_slice(b"\"\r\nContent-Type: ");
            let mime = if file.mime_type.is_empty()
                || !file.mime_type.bytes().all(|b| (0x20..=0x7e).contains(&b))
            {
                "application/octet-stream"
            } else {
                &file.mime_type
            };
            bytes.extend_from_slice(mime.as_bytes());
            bytes.extend_from_slice(b"\r\n\r\n");
            bytes.extend_from_slice(&file.bytes);
        },
    }
    bytes
}
pub fn select_multipart_boundary(entries: &[FormEntry], encoding: &'static Encoding) -> String {
    let parts: Vec<Vec<u8>> = entries
        .iter()
        .map(|entry| multipart_part(entry, encoding))
        .collect();
    select_multipart_boundary_from_chunks(parts.iter().map(Vec::as_slice))
}
pub fn encode_multipart_auto(
    entries: &[FormEntry],
    encoding: &'static Encoding,
) -> Result<EncodedFormBody, FormEncodingError> {
    encode_multipart(
        entries,
        encoding,
        &select_multipart_boundary(entries, encoding),
    )
}
fn select_multipart_boundary_from_chunks<'a>(chunks: impl IntoIterator<Item = &'a [u8]>) -> String {
    let prefix = b"GenetBoundary";
    let mut matched = 0;
    let mut number: Option<u64> = None;
    let mut digits = false;
    let mut leading_zero = false;
    let mut used = std::collections::HashSet::new();
    for chunk in chunks {
        for &byte in chunk {
            if matched == prefix.len() && byte.is_ascii_digit() {
                if !digits {
                    number = Some(0);
                    leading_zero = byte == b'0';
                }
                if !digits || !leading_zero {
                    number =
                        number.and_then(|n| n.checked_mul(10)?.checked_add(u64::from(byte - b'0')));
                    if let Some(value) = number {
                        used.insert(value);
                    }
                }
                digits = true;
                continue;
            }
            if matched == prefix.len() {
                matched = 0;
                number = None;
                digits = false;
            }
            if byte == prefix[matched] {
                matched += 1;
            } else {
                matched = usize::from(byte == prefix[0]);
            }
        }
    }
    let mut candidate = 0u64;
    while used.contains(&candidate) {
        candidate = candidate
            .checked_add(1)
            .expect("finite multipart candidate set");
    }
    format!("GenetBoundary{candidate}")
}

fn pair_value(entry: &FormEntry) -> &str {
    match &entry.value {
        FormEntryValue::Text(value) => value,
        FormEntryValue::File { file, .. } => &file.name,
    }
}

fn encoded(value: &str, encoding: &'static Encoding) -> Vec<u8> {
    // Encoding_rs applies output encoding and decimal references for scalars
    // unrepresentable in a legacy encoding, as required by Encoding/HTML.
    encoding.encode(value).0.into_owned()
}

fn normalize_newlines(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\r' => {
                result.push_str("\r\n");
                if characters.peek() == Some(&'\n') {
                    characters.next();
                }
            },
            '\n' => result.push_str("\r\n"),
            _ => result.push(character),
        }
    }
    result
}

fn percent_encode(value: &[u8], result: &mut Vec<u8>) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for &byte in value {
        match byte {
            b' ' => result.push(b'+'),
            b'*' | b'-' | b'.' | b'_' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' => {
                result.push(byte)
            },
            _ => {
                result.push(b'%');
                result.push(HEX[(byte >> 4) as usize]);
                result.push(HEX[(byte & 15) as usize]);
            },
        }
    }
}

fn escape_header(value: &[u8], result: &mut Vec<u8>) {
    for &byte in value {
        match byte {
            b'\r' => result.extend_from_slice(b"%0D"),
            b'\n' => result.extend_from_slice(b"%0A"),
            b'"' => result.extend_from_slice(b"%22"),
            _ => result.push(byte),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use encoding_rs::{SHIFT_JIS, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252};

    fn text(name: &str, value: &str) -> FormEntry {
        FormEntry {
            name: name.into(),
            value: FormEntryValue::Text(value.into()),
        }
    }
    fn file(name: &str, filename: &str, bytes: &[u8]) -> FormEntry {
        FormEntry {
            name: name.into(),
            value: FormEntryValue::File {
                file: crate::forms_submission::fixture_file(
                    1,
                    filename,
                    "application/octet-stream",
                    bytes,
                ),
                source_input: None,
            },
        }
    }
    fn url_bytes(entries: &[FormEntry], encoding: &'static Encoding) -> Vec<u8> {
        encode_urlencoded(entries, encoding)
    }
    fn plain_bytes(entries: &[FormEntry], encoding: &'static Encoding) -> Vec<u8> {
        encode_text_plain(entries, encoding)
    }

    #[test]
    fn phase_c_encoding_ascii_escaping_and_duplicates_control() {
        assert_eq!(url_bytes(&[text("a b", "+&~*")], UTF_8), b"a+b=%2B%26%7E*");
        assert_eq!(
            url_bytes(&[text("x", "1"), text("x", "2")], UTF_8),
            b"x=1&x=2"
        );
    }
    #[test]
    fn phase_c_encoding_plain_ascii_and_duplicates_missing_capability() {
        assert_eq!(plain_bytes(&[text("a b", "+&~*")], UTF_8), b"a b=+&~*\r\n");
        assert_eq!(
            plain_bytes(&[text("x", "1"), text("x", "2")], UTF_8),
            b"x=1\r\nx=2\r\n"
        );
    }
    #[test]
    fn phase_c_encoding_url_cr_lf_crlf_are_normalized() {
        assert_eq!(
            url_bytes(&[text("a\rb\nc\r\nd", "x\ny\rz\r\nw")], UTF_8),
            b"a%0D%0Ab%0D%0Ac%0D%0Ad=x%0D%0Ay%0D%0Az%0D%0Aw"
        );
    }
    #[test]
    fn phase_c_encoding_plain_cr_lf_crlf_are_normalized() {
        assert_eq!(
            plain_bytes(&[text("a\rb\nc\r\nd", "x\ny\rz\r\nw")], UTF_8),
            b"a\r\nb\r\nc\r\nd=x\r\ny\r\nz\r\nw\r\n"
        );
    }
    #[test]
    fn phase_c_encoding_windows1252_url_decimal_reference_bytes() {
        assert_eq!(
            url_bytes(&[text("q", "é€😀")], WINDOWS_1252),
            b"q=%E9%80%26%23128512%3B"
        );
    }
    #[test]
    fn phase_c_encoding_windows1252_plain_decimal_reference_bytes() {
        assert_eq!(
            plain_bytes(&[text("q", "é€😀")], WINDOWS_1252),
            vec![
                113, 61, 233, 128, 38, 35, 49, 50, 56, 53, 49, 50, 59, 13, 10
            ]
        );
    }
    #[test]
    fn phase_c_encoding_shiftjis_url_and_unrepresentable_scalar() {
        assert_eq!(url_bytes(&[text("q", "あ")], SHIFT_JIS), b"q=%82%A0");
        assert_eq!(
            url_bytes(&[text("q", "😀")], SHIFT_JIS),
            b"q=%26%23128512%3B"
        );
    }
    #[test]
    fn phase_c_encoding_shiftjis_plain_literal_bytes() {
        assert_eq!(
            plain_bytes(&[text("q", "あ")], SHIFT_JIS),
            vec![113, 61, 130, 160, 13, 10]
        );
    }
    #[test]
    fn phase_c_encoding_utf16_output_fallback_control() {
        for encoding in [UTF_16LE, UTF_16BE] {
            assert_eq!(url_bytes(&[text("q", "é")], encoding), b"q=%C3%A9");
        }
    }
    #[test]
    fn phase_c_encoding_utf16_plain_output_fallback() {
        for encoding in [UTF_16LE, UTF_16BE] {
            assert_eq!(
                plain_bytes(&[text("q", "é")], encoding),
                vec![113, 61, 195, 169, 13, 10]
            );
        }
    }
    #[test]
    fn phase_c_encoding_file_pair_uses_normalized_name_not_bytes() {
        let entries = [file("upload", "a\nb.bin", &[0, 128, 255])];
        assert_eq!(url_bytes(&entries, UTF_8), b"upload=a%0D%0Ab.bin");
        assert_eq!(plain_bytes(&entries, UTF_8), b"upload=a\r\nb.bin\r\n");
    }
    #[test]
    fn phase_c_encoding_multipart_text_newline_and_exact_content_type() {
        let body = encode_multipart(&[text("a\nb", "x\ny")], UTF_8, "B")
            .expect("native multipart capability is required");
        assert_eq!(body.content_type, "multipart/form-data; boundary=B");
        assert_eq!(
            body.bytes,
            b"--B\r\nContent-Disposition: form-data; name=\"a%0D%0Ab\"\r\n\r\nx\r\ny\r\n--B--\r\n"
        );
    }
    #[test]
    fn phase_c_encoding_multipart_header_escaping_and_exact_binary() {
        let body = encode_multipart(
            &[file("n\r\"", "f\r\"\n\\.bin", &[0, 128, 255])],
            UTF_8,
            "B",
        )
        .expect("native binary multipart capability is required");
        let mut expected=b"--B\r\nContent-Disposition: form-data; name=\"n%0D%0A%22\"; filename=\"f%0D%22%0A\\.bin\"\r\nContent-Type: application/octet-stream\r\n\r\n".to_vec();
        expected.extend_from_slice(&[0, 128, 255]);
        expected.extend_from_slice(b"\r\n--B--\r\n");
        assert_eq!(body.bytes, expected);
        assert_eq!(body.content_type, "multipart/form-data; boundary=B");
    }
    #[test]
    fn phase_c_encoding_multipart_empty_and_duplicate_order() {
        assert_eq!(
            encode_multipart(&[], UTF_8, "B")
                .expect("native multipart unavailable")
                .bytes,
            b"--B--\r\n"
        );
        let body = encode_multipart(&[text("x", "1"), text("x", "2")], UTF_8, "B").unwrap();
        assert_eq!(body.bytes,b"--B\r\nContent-Disposition: form-data; name=\"x\"\r\n\r\n1\r\n--B\r\nContent-Disposition: form-data; name=\"x\"\r\n\r\n2\r\n--B--\r\n");
    }
    #[test]
    fn phase_c_encoding_multipart_legacy_text_and_binary_file() {
        let entries = [text("q", "é€😀"), file("f", "x.bin", &[0, 128, 255])];
        let body =
            encode_multipart(&entries, WINDOWS_1252, "B").expect("native multipart unavailable");
        let mut expected = b"--B\r\nContent-Disposition: form-data; name=\"q\"\r\n\r\n".to_vec();
        expected.extend_from_slice(&[233, 128, 38, 35, 49, 50, 56, 53, 49, 50, 59]);
        expected.extend_from_slice(b"\r\n--B\r\nContent-Disposition: form-data; name=\"f\"; filename=\"x.bin\"\r\nContent-Type: application/octet-stream\r\n\r\n");
        expected.extend_from_slice(&[0, 128, 255]);
        expected.extend_from_slice(b"\r\n--B--\r\n");
        assert_eq!(body.bytes, expected);
    }
    #[test]
    fn phase_c_encoding_selection_ascii_tokens_and_output_fallback() {
        assert_eq!(select_form_encoding(None, WINDOWS_1252), WINDOWS_1252);
        for label in ["", "bogus", "windows-1252,Shift_JIS"] {
            assert_eq!(select_form_encoding(Some(label), WINDOWS_1252), UTF_8);
        }
        assert_eq!(
            select_form_encoding(Some("bogus\tShift_JIS windows-1252"), UTF_8),
            SHIFT_JIS
        );
        assert_eq!(select_form_encoding(Some("utf-16le"), WINDOWS_1252), UTF_8);
        assert_eq!(select_form_encoding(None, UTF_16BE), UTF_8);
    }
}
