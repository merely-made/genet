# WOFF2 sanitizer fixture

`NotoSansNko-regular-webfont.woff2` is an unmodified copy of
`tests/wpt/tests/fonts/noto/NotoSansNko-regular-webfont.woff2` at Genet commit
`69a2383b2ad777b884a72f31f8f8fb7ece275c0b`. It is 7,724 bytes with SHA-256
`d3ede808ce743034e14ff8cdaeb911aa49e50c0b3c94af38745744f57e564db4`.

The font's embedded name table identifies Noto Sans NKo, copyright 2013 Google
Inc. All Rights Reserved., and the SIL Open Font License, Version 1.1, with
license URL <http://scripts.sil.org/OFL>. `LICENSE_OFL.txt` contains that
copyright notice and the OFL 1.1 text already present at
`tests/wpt/tests/fonts/noto/NotoSansAdlam-hinted/LICENSE_OFL.txt` in the same
source revision. The font bytes and family name have not been changed.

The native test checks WOFF2 sanitization produces parseable SFNT with a
nonzero, in-range glyph for U+07CA (NKo letter A). This is a codec acceptance
test; shaping, platform font discovery, and full-script coverage require
separate measurements. The existing malformed-WOFF2 rejection and SFNT byte
identity tests retain their separate contracts.
