//! Byte-backed HTML input selection. Unicode input uses `DocumentEncoding::default()`.
//! The prescan is bounded to the first 1024 bytes; this module does not restart parsers.
use crate::form_encoding::{Encoding, UTF_8};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodingConfidence {
    Certain,
    Tentative,
    Irrelevant,
}
impl EncodingConfidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Certain => "certain",
            Self::Tentative => "tentative",
            Self::Irrelevant => "irrelevant",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct DocumentEncoding {
    pub encoding: &'static Encoding,
    pub confidence: EncodingConfidence,
}
impl Default for DocumentEncoding {
    fn default() -> Self {
        Self {
            encoding: UTF_8,
            confidence: EncodingConfidence::Irrelevant,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct HtmlDecodeOptions {
    /// Configurable fallback policy; UTF-8 is the compatibility default.
    pub fallback: &'static Encoding,
    pub override_encoding: Option<&'static Encoding>,
    /// Supply only for an eligible same-origin parent, excluding UTF-16.
    pub same_origin_parent_encoding: Option<&'static Encoding>,
}
impl Default for HtmlDecodeOptions {
    fn default() -> Self {
        Self {
            fallback: UTF_8,
            override_encoding: None,
            same_origin_parent_encoding: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct DecodedHtmlDocument {
    pub source: String,
    pub encoding: DocumentEncoding,
}

pub fn decode_html_document(
    bytes: &[u8],
    content_type: Option<&str>,
    options: HtmlDecodeOptions,
) -> DecodedHtmlDocument {
    use EncodingConfidence::{Certain, Tentative};
    let (encoding, confidence) = if let Some((bom, _)) = Encoding::for_bom(bytes) {
        (bom, Certain)
    } else if let Some(overridden) = options.override_encoding {
        (overridden, Certain)
    } else if let Some(transport) = content_type.and_then(transport_encoding) {
        (transport, Certain)
    } else if let Some(meta) = prescan(&bytes[..bytes.len().min(1024)]) {
        (meta, Tentative)
    } else if let Some(parent) = options.same_origin_parent_encoding.filter(|e| !is_utf16(e)) {
        (parent, Tentative)
    } else {
        (options.fallback, Tentative)
    };
    // decode consumes a BOM and returns the actual decoder selected by it.
    let (source, actual, _) = encoding.decode(bytes);
    DecodedHtmlDocument {
        source: source.into_owned(),
        encoding: DocumentEncoding {
            encoding: actual,
            confidence,
        },
    }
}
fn is_utf16(encoding: &'static Encoding) -> bool {
    matches!(encoding.name(), "UTF-16LE" | "UTF-16BE")
}
fn label(value: &[u8]) -> Option<&'static Encoding> {
    Encoding::for_label(value)
}
fn space(b: u8) -> bool {
    matches!(b, b'\t' | b'\n' | b'\x0C' | b'\r' | b' ')
}

fn http_whitespace(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | ' ')
}
fn http_token(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '!' | '#'
                | '$'
                | '%'
                | '&'
                | '\''
                | '*'
                | '+'
                | '-'
                | '.'
                | '^'
                | '_'
                | '`'
                | '|'
                | '~'
        )
}
fn http_quoted_token(c: char) -> bool {
    matches!(c, '\t' | '\u{20}'..='\u{7E}' | '\u{80}'..='\u{FF}')
}
/// MIME parsing validates the essence and parameter insertion rules before
/// charset precedence. Accepted unsupported labels and quoted-empty values
/// block later duplicates; skipped invalid parameters do not.
fn transport_encoding(header: &str) -> Option<&'static Encoding> {
    let header = header.trim_matches(http_whitespace);
    let (mime_type, rest) = header.split_once('/')?;
    if mime_type.is_empty() || !mime_type.chars().all(http_token) {
        return None;
    }
    let subtype_end = rest.find(';').unwrap_or(rest.len());
    let subtype = rest[..subtype_end].trim_end_matches(http_whitespace);
    if subtype.is_empty() || !subtype.chars().all(http_token) {
        return None;
    }
    let parameters: Vec<char> = rest[subtype_end..].chars().collect();
    let mut i = 0;
    while i < parameters.len() {
        i += 1; // semicolon
        while i < parameters.len() && http_whitespace(parameters[i]) {
            i += 1;
        }
        let start = i;
        while i < parameters.len() && !matches!(parameters[i], ';' | '=') {
            i += 1;
        }
        let name: String = parameters[start..i].iter().collect();
        if i == parameters.len() {
            break;
        }
        if parameters[i] == ';' {
            continue;
        }
        i += 1; // equals; leading value whitespace is not stripped
        if i == parameters.len() {
            break;
        }
        let mut value = String::new();
        if parameters[i] == '"' {
            i += 1;
            while i < parameters.len() {
                let c = parameters[i];
                i += 1;
                if c == '"' {
                    break;
                }
                if c == '\\' && i < parameters.len() {
                    value.push(parameters[i]);
                    i += 1;
                } else {
                    value.push(c);
                }
            }
            // MIME ignores trailing material after a collected quoted string.
            while i < parameters.len() && parameters[i] != ';' {
                i += 1;
            }
        } else {
            let start = i;
            while i < parameters.len() && parameters[i] != ';' {
                i += 1;
            }
            value = parameters[start..i].iter().collect();
            value.truncate(value.trim_end_matches(http_whitespace).len());
            if value.is_empty() {
                continue;
            }
        }
        if !name.is_empty()
            && name.chars().all(http_token)
            && value.chars().all(http_quoted_token)
            && name.eq_ignore_ascii_case("charset")
        {
            return label(value.as_bytes());
        }
    }
    None
}
/// HTML's prescan attribute reader. EOF inside an attribute aborts the tag.
fn attribute(b: &[u8], i: &mut usize) -> Result<Option<(Vec<u8>, Vec<u8>)>, ()> {
    while *i < b.len() && (space(b[*i]) || b[*i] == b'/') {
        *i += 1;
    }
    if *i >= b.len() {
        return Err(());
    }
    if b[*i] == b'>' {
        *i += 1;
        return Ok(None);
    }
    let mut name = Vec::new();
    while *i < b.len() {
        let c = b[*i];
        if c == b'=' && !name.is_empty() {
            break;
        }
        if space(c) || c == b'/' || c == b'>' {
            break;
        }
        name.push(c.to_ascii_lowercase());
        *i += 1;
    }
    while *i < b.len() && space(b[*i]) {
        *i += 1;
    }
    if *i >= b.len() {
        return Err(());
    }
    if b[*i] != b'=' {
        return Ok(Some((name, Vec::new())));
    }
    *i += 1;
    while *i < b.len() && space(b[*i]) {
        *i += 1;
    }
    if *i >= b.len() {
        return Err(());
    }
    let mut value = Vec::new();
    if matches!(b[*i], b'\'' | b'"') {
        let quote = b[*i];
        *i += 1;
        while *i < b.len() && b[*i] != quote {
            value.push(b[*i].to_ascii_lowercase());
            *i += 1;
        }
        if *i >= b.len() {
            return Err(());
        }
        *i += 1;
    } else {
        while *i < b.len() && !space(b[*i]) && b[*i] != b'>' {
            value.push(b[*i].to_ascii_lowercase());
            *i += 1;
        }
        if *i >= b.len() {
            return Err(());
        }
    }
    Ok(Some((name, value)))
}
fn content_encoding(value: &[u8]) -> Option<&'static Encoding> {
    let mut start = 0;
    while start + 7 <= value.len() {
        let relative = value[start..]
            .windows(7)
            .position(|s| s.eq_ignore_ascii_case(b"charset"))?;
        let mut i = start + relative + 7;
        while i < value.len() && space(value[i]) {
            i += 1;
        }
        if i >= value.len() || value[i] != b'=' {
            start = i;
            continue;
        }
        i += 1;
        while i < value.len() && space(value[i]) {
            i += 1;
        }
        if i >= value.len() {
            return None;
        }
        let begin;
        let end;
        if matches!(value[i], b'\'' | b'"') {
            let quote = value[i];
            i += 1;
            begin = i;
            while i < value.len() && value[i] != quote {
                i += 1;
            }
            if i == value.len() {
                return None;
            }
            end = i;
        } else {
            begin = i;
            while i < value.len() && !space(value[i]) && value[i] != b';' {
                i += 1;
            }
            end = i;
        }
        return label(&value[begin..end]);
    }
    None
}
fn prescan(b: &[u8]) -> Option<&'static Encoding> {
    prescan_meta(b).or_else(|| xml_encoding(b))
}
fn prescan_meta(b: &[u8]) -> Option<&'static Encoding> {
    // The XML prefix fallback also participates in HTML's bounded prescan.
    if b.starts_with(b"<\0?\0x\0") {
        return label(b"utf-16le");
    }
    if b.starts_with(b"\0<\0?\0x") {
        return label(b"utf-16be");
    }
    let mut i = 0;
    while i < b.len() {
        if b[i..].starts_with(b"<!--") {
            i += 2;
            if let Some(end) = b[i..].windows(3).position(|s| s == b"-->") {
                i += end + 3;
            } else {
                return None;
            }
            continue;
        }
        if b[i] == b'<'
            && i + 5 < b.len()
            && b[i + 1..i + 5].eq_ignore_ascii_case(b"meta")
            && (space(b[i + 5]) || b[i + 5] == b'/')
        {
            i += 5;
            let mut seen = std::collections::HashSet::new();
            let mut charset = None;
            let mut need_pragma = None;
            let mut pragma = false;
            loop {
                match attribute(b, &mut i) {
                    Ok(Some((name, value))) => {
                        if !seen.insert(name.clone()) {
                            continue;
                        }
                        match name.as_slice() {
                            b"http-equiv" => pragma = value == b"content-type",
                            b"content" => {
                                if charset.is_none() && need_pragma != Some(false) {
                                    if let Some(enc) = content_encoding(&value) {
                                        charset = Some(enc);
                                        need_pragma = Some(true);
                                    }
                                }
                            },
                            b"charset" => {
                                charset = label(&value);
                                need_pragma = Some(false);
                            },
                            _ => {},
                        }
                    },
                    Ok(None) => break,
                    Err(()) => return None,
                }
            }
            if need_pragma == Some(false) || need_pragma == Some(true) && pragma {
                if let Some(enc) = charset {
                    return Some(if is_utf16(enc) {
                        UTF_8
                    } else if enc.name() == "x-user-defined" {
                        label(b"windows-1252").unwrap()
                    } else {
                        enc
                    });
                }
            }
            continue;
        }
        if b[i] == b'<'
            && i + 1 < b.len()
            && (b[i + 1].is_ascii_alphabetic()
                || b[i + 1] == b'/' && i + 2 < b.len() && b[i + 2].is_ascii_alphabetic())
        {
            i += 1;
            while i < b.len() && !space(b[i]) && b[i] != b'>' {
                i += 1;
            }
            loop {
                match attribute(b, &mut i) {
                    Ok(Some(_)) => {},
                    Ok(None) => break,
                    Err(()) => return None,
                }
            }
            continue;
        }
        if b[i] == b'<' && i + 1 < b.len() && matches!(b[i + 1], b'!' | b'/' | b'?') {
            while i < b.len() && b[i] != b'>' {
                i += 1;
            }
        }
        i += 1;
    }
    None
}
fn xml_encoding(b: &[u8]) -> Option<&'static Encoding> {
    // XML declarations with an ASCII-compatible encoding label are a fallback
    // only after scanning for an HTML meta declaration.
    if b.starts_with(b"<?xml") {
        let end = b.iter().position(|&c| c == b'>')?;
        let s = &b[5..end];
        let p = s.windows(8).position(|v| v == b"encoding")?;
        let mut n = p + 8;
        while n < s.len() && s[n] <= 0x20 {
            n += 1;
        }
        if s.get(n) != Some(&b'=') {
            return None;
        }
        n += 1;
        while n < s.len() && s[n] <= 0x20 {
            n += 1;
        }
        let quote = *s.get(n)?;
        if !matches!(quote, b'\'' | b'"') {
            return None;
        }
        n += 1;
        let e = s[n..].iter().position(|&c| c == quote)?;
        let value = &s[n..n + e];
        if value.iter().any(|&b| b <= 0x20) {
            return None;
        }
        return label(value).map(|encoding| if is_utf16(encoding) { UTF_8 } else { encoding });
    }
    None
}

#[cfg(test)]
mod tests {
    fn current_decoder_adapter(
        bytes: &[u8],
        content_type: Option<&str>,
        fallback: &str,
        override_encoding: Option<&str>,
        same_origin_parent: Option<&str>,
    ) -> (String, &'static str, &'static str) {
        let document = super::decode_html_document(
            bytes,
            content_type,
            super::HtmlDecodeOptions {
                fallback: super::label(fallback.as_bytes()).unwrap(),
                override_encoding: override_encoding.and_then(|v| super::label(v.as_bytes())),
                same_origin_parent_encoding: same_origin_parent
                    .and_then(|v| super::label(v.as_bytes())),
            },
        );
        (
            document.source,
            document.encoding.encoding.name(),
            document.encoding.confidence.as_str(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn check(
        bytes: &[u8],
        header: Option<&str>,
        fallback: &str,
        override_encoding: Option<&str>,
        parent: Option<&str>,
        name: &str,
        confidence: &str,
        tail: &str,
    ) {
        let (source, actual_name, actual_confidence) =
            current_decoder_adapter(bytes, header, fallback, override_encoding, parent);
        assert_eq!(actual_name, name, "selected canonical document encoding");
        assert_eq!(actual_confidence, confidence, "decoder confidence");
        assert!(
            source.ends_with(tail),
            "decoded source {source:?} must end in {tail:?}"
        );
    }

    #[test]
    fn phase_c_document_encoding_utf8_bom_wins_header_meta_and_override() {
        check(
            b"\xEF\xBB\xBF<meta charset=windows-1252>\xC3\xA9",
            Some("text/html;charset=shift_jis"),
            "windows-1252",
            Some("euc-kr"),
            None,
            "UTF-8",
            "certain",
            "é",
        );
        let (source, _, _) =
            current_decoder_adapter(b"\xEF\xBB\xBF<p>x</p>", None, "UTF-8", None, None);
        assert_eq!(source, "<p>x</p>", "BOM is consumed by decoding");
    }
    #[test]
    fn phase_c_document_encoding_utf16_bom_preserves_actual_selected_encoding() {
        check(
            b"\xFF\xFE<\0p\0>\0x\0<\0/\0p\0>\0",
            Some("text/html;charset=utf-8"),
            "UTF-8",
            None,
            None,
            "UTF-16LE",
            "certain",
            "<p>x</p>",
        );
    }
    #[test]
    fn phase_c_document_encoding_transport_wins_conflicting_meta() {
        check(
            b"<meta charset=windows-1252>\x82\xA0",
            Some("text/html; charset=shift_jis"),
            "UTF-8",
            None,
            None,
            "Shift_JIS",
            "certain",
            "あ",
        );
    }
    #[test]
    fn phase_c_document_encoding_invalid_transport_uses_meta() {
        check(
            b"<meta charset=iso-8859-1>\xE9",
            Some("text/html;charset=bogus"),
            "UTF-8",
            None,
            None,
            "windows-1252",
            "tentative",
            "é",
        );
    }
    #[test]
    fn phase_c_document_encoding_quoted_transport_semicolon_and_first_duplicate() {
        check(
            b"\x82\xA0",
            Some("Text/Html; note=\"a;charset=utf-8;\"; CHARSET=\"shift_jis\"; charset=utf-8"),
            "UTF-8",
            None,
            None,
            "Shift_JIS",
            "certain",
            "あ",
        );
    }
    #[test]
    fn phase_c_document_encoding_override_wins_transport_without_bom() {
        check(
            b"\xE9",
            Some("text/html;charset=utf-8"),
            "UTF-8",
            Some("windows-1252"),
            None,
            "windows-1252",
            "certain",
            "é",
        );
    }
    #[test]
    fn phase_c_document_encoding_comment_fake_meta_is_skipped() {
        check(
            b"<!-- <meta charset=utf-8> --><meta charset=windows-1252>\xE9",
            None,
            "UTF-8",
            None,
            None,
            "windows-1252",
            "tentative",
            "é",
        );
    }
    #[test]
    fn phase_c_document_encoding_tag_attribute_fake_meta_is_skipped() {
        check(
            b"<div title=\"<meta charset=utf-8>\"><meta charset=windows-1252>\xE9",
            None,
            "UTF-8",
            None,
            None,
            "windows-1252",
            "tentative",
            "é",
        );
    }
    #[test]
    fn phase_c_document_encoding_duplicate_charset_keeps_first_attribute() {
        check(
            b"<MeTa CHARSET=shift_jis charset=utf-8>\x82\xA0",
            None,
            "UTF-8",
            None,
            None,
            "Shift_JIS",
            "tentative",
            "あ",
        );
    }
    #[test]
    fn phase_c_document_encoding_content_requires_http_equiv_pragma() {
        check(
            b"<meta content='text/html;charset=shift_jis'>\xE9",
            None,
            "windows-1252",
            None,
            None,
            "windows-1252",
            "tentative",
            "é",
        );
    }
    #[test]
    fn phase_c_document_encoding_http_equiv_content_handles_attribute_order() {
        for bytes in [
            b"<meta content='text/html;charset=shift_jis' http-equiv=content-type>\x82\xA0"
                .as_slice(),
            b"<meta http-equiv=content-type content='text/html;charset=shift_jis'>\x82\xA0"
                .as_slice(),
        ] {
            check(
                bytes,
                None,
                "UTF-8",
                None,
                None,
                "Shift_JIS",
                "tentative",
                "あ",
            );
        }
    }
    #[test]
    fn phase_c_document_encoding_duplicate_pragma_first_value_controls() {
        check(b"<meta http-equiv=refresh http-equiv=content-type content='text/html;charset=shift_jis'>\xE9", None, "windows-1252", None, None, "windows-1252", "tentative", "é");
    }
    #[test]
    fn phase_c_document_encoding_duplicate_content_first_value_controls() {
        check(b"<meta http-equiv=content-type content='text/html;charset=windows-1252' content='text/html;charset=utf-8'>\xE9", None, "UTF-8", None, None, "windows-1252", "tentative", "é");
    }
    #[test]
    fn phase_c_document_encoding_charset_overrides_content_both_orders() {
        for bytes in [
            b"<meta content='text/html;charset=utf-8' charset=shift_jis>\x82\xA0".as_slice(),
            b"<meta charset=shift_jis content='text/html;charset=utf-8'>\x82\xA0".as_slice(),
        ] {
            check(
                bytes,
                None,
                "UTF-8",
                None,
                None,
                "Shift_JIS",
                "tentative",
                "あ",
            );
        }
    }
    #[test]
    fn phase_c_document_encoding_invalid_first_meta_does_not_hide_later_valid() {
        check(
            b"<meta charset=bogus><meta charset=windows-1252>\xE9",
            None,
            "UTF-8",
            None,
            None,
            "windows-1252",
            "tentative",
            "é",
        );
    }
    #[test]
    fn phase_c_document_encoding_meta_utf16_adjusts_to_utf8() {
        check(
            b"<meta charset=utf-16le>\xC3\xA9",
            None,
            "windows-1252",
            None,
            None,
            "UTF-8",
            "tentative",
            "é",
        );
    }
    #[test]
    fn phase_c_document_encoding_meta_user_defined_adjusts_to_windows1252() {
        check(
            b"<meta charset=x-user-defined>\x80",
            None,
            "UTF-8",
            None,
            None,
            "windows-1252",
            "tentative",
            "€",
        );
    }
    #[test]
    fn phase_c_document_encoding_prescan_first1024_boundary_and_truncation() {
        let mut complete = vec![b' '; 998];
        complete.extend_from_slice(b"<meta charset=shift_jis>");
        complete.extend_from_slice(b"\x82\xA0");
        check(
            &complete,
            None,
            "UTF-8",
            None,
            None,
            "Shift_JIS",
            "tentative",
            "あ",
        );
        let mut late = vec![b' '; 1024];
        late.extend_from_slice(b"<meta charset=shift_jis>\xE9");
        check(
            &late,
            None,
            "windows-1252",
            None,
            None,
            "windows-1252",
            "tentative",
            "é",
        );
        let mut truncated = vec![b' '; 1010];
        truncated.extend_from_slice(b"<meta charset='shift_jis'>\xE9");
        check(
            &truncated,
            None,
            "windows-1252",
            None,
            None,
            "windows-1252",
            "tentative",
            "é",
        );
    }
    #[test]
    fn phase_c_document_encoding_configured_fallback_decodes_literal_legacy_bytes() {
        check(
            b"<p>\xE9</p>",
            None,
            "windows-1252",
            None,
            None,
            "windows-1252",
            "tentative",
            "é</p>",
        );
    }
    #[test]
    fn phase_c_document_encoding_parent_fallback_excludes_utf16() {
        check(
            b"\x82\xA0",
            None,
            "UTF-8",
            None,
            Some("Shift_JIS"),
            "Shift_JIS",
            "tentative",
            "あ",
        );
        check(
            b"\xE9",
            None,
            "windows-1252",
            None,
            Some("UTF-16LE"),
            "windows-1252",
            "tentative",
            "é",
        );
    }
    #[test]
    fn phase_c_document_encoding_euckr_header_decodes_actual_bytes() {
        check(
            b"<input value='\xB0\xA1'>",
            Some("text/html;charset=euc-kr"),
            "UTF-8",
            None,
            None,
            "EUC-KR",
            "certain",
            "가'>",
        );
    }
    #[test]
    fn phase_c_document_encoding_utf8_default_is_tentative() {
        check(
            b"<p>\xC3\xA9</p>",
            None,
            "UTF-8",
            None,
            None,
            "UTF-8",
            "tentative",
            "é</p>",
        );
    }
    #[test]
    fn phase_c_document_encoding_revision_mime_empty_unquoted_is_skipped() {
        check(
            b"\x82\xA0",
            Some("text/html;charset=;charset=shift_jis"),
            "UTF-8",
            None,
            None,
            "Shift_JIS",
            "certain",
            "あ",
        );
    }
    #[test]
    fn phase_c_document_encoding_revision_mime_invalid_name_is_skipped() {
        check(
            b"\x82\xA0",
            Some("text/html;charset =windows-1252;charset=shift_jis"),
            "UTF-8",
            None,
            None,
            "Shift_JIS",
            "certain",
            "あ",
        );
    }
    #[test]
    fn phase_c_document_encoding_revision_mime_invalid_type_rejects_transport() {
        check(
            b"<meta charset=shift_jis>\x82\xA0",
            Some("te xt/html;charset=windows-1252"),
            "UTF-8",
            None,
            None,
            "Shift_JIS",
            "tentative",
            "あ",
        );
    }
    #[test]
    fn phase_c_document_encoding_revision_mime_invalid_subtype_rejects_transport() {
        check(
            b"<meta charset=shift_jis>\x82\xA0",
            Some("text/ht ml;charset=windows-1252"),
            "UTF-8",
            None,
            None,
            "Shift_JIS",
            "tentative",
            "あ",
        );
    }
    #[test]
    fn phase_c_document_encoding_revision_mime_unsupported_first_accepted_label_stays_first() {
        check(
            b"<meta charset=windows-1252>\xE9",
            Some("text/html;charset=bogus;charset=shift_jis"),
            "UTF-8",
            None,
            None,
            "windows-1252",
            "tentative",
            "é",
        );
    }
    #[test]
    fn phase_c_document_encoding_revision_mime_quoted_empty_stays_first() {
        check(
            b"<meta charset=windows-1252>\xE9",
            Some("text/html;charset=\"\";charset=shift_jis"),
            "UTF-8",
            None,
            None,
            "windows-1252",
            "tentative",
            "é",
        );
    }
}
