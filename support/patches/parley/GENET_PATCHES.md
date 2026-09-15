# genet's parley fork - patch log

This is `genet-parley 0.10.0`, a vendored copy of upstream `parley 0.10.0`.

**Published identity (2026-09-14).** The package was renamed from `parley` to
`genet-parley` so it can be published to crates.io, following the `genet-taffy`
precedent. `[lib] name = "parley"` is kept, so consumers keep writing
`parley::` and name the crate as
`parley = { package = "genet-parley", version = "0.10.0" }`. Its `fontique`
dependency was renamed the same way, to
`fontique = { package = "genet-fontique", version = "0.10.0", path = "../fontique" }`,
with every feature forward (`libm`, `std`, `system`) unchanged because the
dependency key is still `fontique`. `parley_data` and the remaining
dependencies are untouched.

`exclude = ["/tests"]` was dropped in the same change. Upstream excluded its
large snapshot suite; the only file left under `tests/` here is genet's own
`join_controls.rs` receipt, which the `[[test]]` target names, so excluding it
would have made the package unbuildable from the registry.

Both root `[patch.crates-io]` entries (`parley` and `fontique`) were removed:
the workspace dependencies now carry the `package =` rename plus an explicit
path, so the fork travels by identity rather than by redirect. That is what
`genet-livery` needs, because a registry crate cannot depend on a
`[patch.crates-io]` entry in some other workspace's root, and livery requires
the `font-diagnostic` feature and three text APIs that exist only here.

## Join controls do not select a fallback face

Parley's font-coverage pass counts U+200C ZERO WIDTH NON-JOINER and U+200D ZERO
WIDTH JOINER because both are inherited-format characters. Many fonts omit
these default-ignorable controls from `cmap`. A cluster such as `f<ZWNJ>i`
therefore selects a system fallback even when the requested face covers both
visible letters. The fallback face then supplies the run's line metrics.

The patch excludes the two join controls from coverage scoring. It does not
remove them from the segment: `shape/mod.rs` still puts every source character
into the HarfRust buffer, so the controls continue to affect joining and
ligature formation. It also excludes their hidden zero-advance clusters from
letter-spacing, keeping `a<ZWNJ>b` the same width as unligated `ab`.

Retire this fork when an upstream Parley release makes default-ignorable join
controls non-covering during fallback selection while retaining them in the
shaping buffer.

## Receipts

- `support/patches/parley/tests/join_controls.rs` proves ZWNJ keeps the selected
  face and receives no letter spacing.
- `components/genet-livery/tests/k5d_font_feature_resolution.rs` proves that
  `fi`, `f<ZWNJ>i`, and the U+FB01 presentation ligature keep one authored
  face's line metrics, and proves join controls do not add letter spacing.

## Patch commits

Every commit that has touched this directory, newest first
(`git log --oneline -- support/patches/parley`):

| Commit | Subject |
|---|---|
| `5209012b2fb` | Restore inside list markers and explicit RTL layout |
| `f2fb66aa1c9` | livery: transform property values, hit-testing and paint receipts, plus a workspace reflow |
| `02b173a9db7` | Fix Windows symbol fallback with paired Parley and Fontique patch |
| `caa562d1e3a` | Add correlated Windows font fallback diagnostic and revise repair gates |
| `29f93579d92` | Reconcile Livery css-text behavior |
| `c36c34be0ad` | Implement Livery font feature resolution |
