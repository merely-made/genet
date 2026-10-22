// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Remove CSS comments without changing the surrounding token stream.

use std::ops::Range;

use cssparser::{Parser, Token, TokenSerializationType};

struct CommentEdit {
    range: Range<usize>,
    replacement: &'static str,
}

fn finish_comments(
    pending: &mut Vec<Range<usize>>,
    previous: TokenSerializationType,
    next: TokenSerializationType,
    edits: &mut Vec<CommentEdit>,
) {
    let separator_needed = previous.needs_separator_when_before(next);
    for (index, range) in pending.drain(..).enumerate() {
        edits.push(CommentEdit {
            range,
            replacement: if index == 0 && separator_needed {
                "/**/"
            } else {
                ""
            },
        });
    }
}

fn next_token(input: &str) -> Option<(usize, TokenSerializationType, bool)> {
    let mut parser = Parser::new(input);
    let token = parser.next_including_whitespace_and_comments().ok()?;
    let kind = token.serialization_type();
    let is_comment = matches!(token, Token::Comment(_));
    Some((parser.position().byte_index(), kind, is_comment))
}

fn comment_edits(css: &str, edits: &mut Vec<CommentEdit>) {
    let mut previous = TokenSerializationType::Nothing;
    let mut pending = Vec::new();
    let mut offset = 0;
    while offset < css.len() {
        // One parser per token lets cssparser's tokenizer identify strings,
        // URLs, comments, and escapes without its Parser skipping nested
        // blocks on the next call. Closers are ordinary root-level tokens.
        let Some((consumed, kind, is_comment)) = next_token(&css[offset..]) else {
            break;
        };
        if consumed == 0 {
            break;
        }
        let start = offset;
        offset += consumed;
        if is_comment {
            pending.push(start..offset);
            continue;
        }

        finish_comments(&mut pending, previous, kind, edits);
        previous = kind;
    }
    finish_comments(
        &mut pending,
        previous,
        TokenSerializationType::Nothing,
        edits,
    );
}

pub(crate) fn without_comments(css: &str) -> String {
    let mut edits = Vec::new();
    comment_edits(css, &mut edits);

    let mut clean = String::with_capacity(css.len());
    let mut cursor = 0;
    for edit in edits {
        clean.push_str(&css[cursor..edit.range.start]);
        clean.push_str(edit.replacement);
        cursor = edit.range.end;
    }
    clean.push_str(&css[cursor..]);
    clean
}
