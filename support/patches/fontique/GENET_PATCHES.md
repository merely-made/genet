# genet's fontique fork - patch log

This is `genet-fontique 0.10.0`, a vendored copy of upstream `fontique 0.10.0`.

**Published identity (2026-09-14).** The package was renamed from `fontique` to
`genet-fontique` so it can be published to crates.io, following the
`genet-taffy` precedent. `[lib] name = "fontique"` is kept, so consumers keep
writing `fontique::` and name the crate as
`fontique = { package = "genet-fontique", version = "0.10.0" }`. The root
`[patch.crates-io] fontique` entry was removed in the same change: the workspace
dependency now carries the `package =` rename plus an explicit path, and
`genet-parley` names this fork as a sibling path dependency, so downstream Genet
consumers do not depend on a root patch entry to obtain the matching query API.

The rename is what `genet-livery` needs: livery 0.0.2 could not build from the
registry, because the text query below exists only here, and a registry crate
cannot depend on a `[patch.crates-io]` redirect.

Source: crates.io `fontique-0.10.0.crate`, SHA-256
`274fa4f0f0a926ae182c7c076c078cce8a38471d15e61a102a02cac984be9813`;
upstream source revision `1df9544bf0bd675d304001c0d0b35df2d220cd14`
(recorded in `.cargo_vcs_info.json`, which identifies upstream rather than this
patch).

## Windows codepoint fallback

Fontique's DirectWrite backend used `IDWriteFontFallback::MapCharacters`, but
called it with a representative script sample and cached the returned family
by script and locale. Common-script punctuation normalized to `Latn` could
therefore receive a Latin-sample family that did not cover the actual symbol.

The added text query is called only after Parley's authored candidates fail.
It passes the complete cluster, locale, and matching attributes to
`MapCharacters`; explicit Fontique fallback families remain first. A result is
accepted only when one mapped font covers the full UTF-16 range at scale 1.0.
Partial ranges and scaled results use the existing script fallback because the
family-only query API cannot carry their remaining-range or scale semantics.

Non-Windows targets retain Fontique's existing script-cache route. The text
route clears its cache identity, so a following same-key script query cannot
reuse text-specific families.

## Patch commits

Every commit that has touched this directory, newest first
(`git log --oneline -- support/patches/fontique`):

| Commit | Subject |
|---|---|
| `f2fb66aa1c9` | livery: transform property values, hit-testing and paint receipts, plus a workspace reflow |
| `02b173a9db7` | Fix Windows symbol fallback with paired Parley and Fontique patch |

The fork is one patch deep: `02b173a9db7` introduced the Windows codepoint
fallback described above, paired with the matching Parley-side change, and
`f2fb66aa1c9` carried it through a workspace reflow. There is no
`0000-complete-fork-delta.patch` here as there is for taffy; the delta against
pristine upstream is small enough to read as a diff against the crates.io
source named in the provenance block above.
