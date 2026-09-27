// SPDX-License-Identifier: MPL-2.0
use genet_text::{BoundaryKind, segments};

fn check(data: &str, kind: BoundaryKind) {
    let mut cases = 0;
    for (line_number, line) in data.lines().enumerate() {
        let body = line.split('#').next().unwrap().trim();
        if body.is_empty() {
            continue;
        }
        let mut text = String::new();
        let mut expected = Vec::new();
        for token in body.split_whitespace() {
            match token {
                "÷" => expected.push(text.len()),
                "×" => {},
                scalar => {
                    text.push(char::from_u32(u32::from_str_radix(scalar, 16).unwrap()).unwrap())
                },
            }
        }
        let mut actual = vec![0];
        actual.extend(segments(&text, kind).map(|part| part.end.utf8_bytes));
        assert_eq!(
            actual,
            expected,
            "{kind:?}, official fixture line {}",
            line_number + 1
        );
        if kind == BoundaryKind::ExtendedGrapheme {
            for byte in text
                .char_indices()
                .map(|(i, _)| i)
                .chain(std::iter::once(text.len()))
            {
                let previous = expected.iter().copied().rfind(|&i| i < byte).unwrap_or(0);
                let next = expected
                    .iter()
                    .copied()
                    .find(|&i| i > byte)
                    .unwrap_or(text.len());
                assert_eq!(
                    genet_text::previous_grapheme_boundary(&text, byte),
                    Some(previous),
                    "previous at {byte}, line {}",
                    line_number + 1
                );
                assert_eq!(
                    genet_text::next_grapheme_boundary(&text, byte),
                    Some(next),
                    "next at {byte}, line {}",
                    line_number + 1
                );
            }
        }
        cases += 1;
    }
    assert!(
        cases > 500,
        "the official corpus must not silently become empty or truncated"
    );
    eprintln!("{kind:?}: {cases} official Unicode 17 cases");
}

#[test]
fn unicode_17_extended_graphemes() {
    check(
        include_str!("data/GraphemeBreakTest.txt"),
        BoundaryKind::ExtendedGrapheme,
    );
}
#[test]
fn unicode_17_words() {
    check(include_str!("data/WordBreakTest.txt"), BoundaryKind::Word);
}
#[test]
fn unicode_17_sentences() {
    check(
        include_str!("data/SentenceBreakTest.txt"),
        BoundaryKind::Sentence,
    );
}
