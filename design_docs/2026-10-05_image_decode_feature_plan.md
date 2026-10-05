# genet-livery image decoding behind a default-on feature

**Status:** landed on local `main`, 2026-10-05; not pushed (pushing waits on
Mark). Progress below records the receipts.

## Why

Mere's mer3ly site canvas lane is building a read-only viewer cone of
`graphshell-web` for mer3ly.net. Its 2026-10-01 size attribution put the image
decoders (zune_jpeg, image_webp, image, png) at 397,074 B, 2.8% of the
shipped wasm. They do not come in through `graphshell-web` itself. They come
through `genet-livery`, which cambium-rootstock and mesquite need for style and
layout, so the cone cannot drop them from Mere's side.

The lane's ruling is in
`mer3ly/docs/2026-09-30_graphshell_site_canvas_plan.md`, Ruling 106 ("the
image decoders"). The options were: ask genet for a feature; accept them in
the viewer; decide after measuring. Mark: "Ask genet for a feature
(Recommended)". The lane then asked this repo, on 2026-10-04, for
`genet-livery` image decoding behind a default-on feature, forwarded by
`genet-render`.

## Findings (verified 2026-10-05 at `d7f08fecdc7`)

- **`image` is a plain dependency.** `genet-livery` takes `image` from the
  workspace row: 0.25 with avif, rayon, bmp, gif, ico, jpeg, png and webp.
- **There are exactly two decode sites:**
  - `components/genet-livery/src/layout.rs:2941`, the natural size of an
    `<img>`;
  - `components/genet-livery/src/paint.rs:431`, the RGBA8 image resource.

  Both already treat a failed decode as `None`.
- **Tests depend on decoding.** `tests/paint.rs`, `tests/interaction.rs` and
  `tests/replaced_image.rs` build their PNG fixtures with the `image` crate.
- **Cargo unifies features, so forwarding from `genet-render` alone is not
  enough.** Five genet crates depend on `genet-livery`: `genet-render`,
  `taproot`, `genet-documents`, `genet-scripted` and `genet-wpt`. Mere's web
  port (`mere/ports/graphshell/web/Cargo.toml`) depends directly on
  `genet-render` and on `taproot`, which took `genet-livery` with default
  features. Any one default-featured path would turn decoding back on.
- **One existing row disables defaults.** Genet's workspace row for
  `genet-documents` already sets `default-features = false`. Whether a
  consumer keeps decoding depends on whether something else in its graph asks
  for it. That is checked with `cargo tree` (Progress).
- **Ortet lost decoding on the first pass, caught before landing.** `ortet`
  is that row's only consumer (`ports/ortet/Cargo.toml:39`, with
  `features = ["livery"]`). It reached `genet-livery` only through
  `genet-documents`. With forwarding in place, nothing in its graph asked for
  `image-decode`, and `cargo tree -p ortet` showed no `image`. `ortet` now
  asks for `["livery", "image-decode"]`.
- **Fifteen tests assert on decoded images.** With the feature off they
  failed, as expected: 13 in `tests/paint.rs` and 2 in
  `tests/interaction.rs`. Each is now gated with
  `#[cfg(feature = "image-decode")]`.
- **The primary checkout's ignored lock is stale against `main`.** It predates
  S1.1 (`icu_properties` and `icu_segmenter 2.2.0` on `genet-livery`) and the
  Vano repin `d7f08fecdc7` (`8ad08412` to `47f8d4f9`). Cargo's git cache does
  not hold Vano `47f8d4f9`, so an offline `--locked` build from a copy of
  that lock fails. That is the copy the lane briefs tell agents to take.
  Verification here therefore patched `nova_vm` to the local `crates/vano`,
  which is exactly `47f8d4f9`, and let the throwaway lock re-resolve. None of
  this is caused by this change; it is reported to Mark.

## Rulings (2026-10-05)

Asked of Mark in this repo after the lane's request:

1. **Shape.** Asked how genet should shape it. The options were: a feature
   with one decode point; a host decoder seam now; or decline. Mark: "Feature,
   one decode point (Recommended)". It follows that:
   - there is one `image-decode` feature, on by default;
   - `image` becomes optional, and stays a dev-dependency for the tests;
   - both sites call one private decode point
     (`components/genet-livery/src/image_decode.rs`), so a host decoder can
     replace it later.

   With the feature off, images get no natural size and are not painted, as
   for a failed decode today.
2. **Forwarding.** Asked which genet crates forward the feature. The options
   were: all five dependents; or only `genet-render` and `taproot`. Mark:
   "All five dependents (Recommended)". It follows that each takes
   `genet-livery` without default features and forwards a default-on
   `image-decode`.
3. **Who lands it.** Asked who lands it. The options were: this session, on
   genet `main`; or a brief to the implementing agent. Mark: "I land it on
   genet main (Recommended)". It follows that pushing waits on Mark's word.

## Phase and done-conditions

- **The decode point.** `image_decode::decode(bytes) -> Option<DecodedImage>`.
  `DecodedImage` wraps `image::DynamicImage` with the feature, and is an empty
  enum without it. Layout reads `dimensions()` and paint reads
  `into_rgba8()`, so each site keeps today's decode cost.
- **Manifests.**
  - `genet-livery`: `[features] default = ["image-decode"]`, with
    `image-decode = ["dep:image"]`. `image` stays a dev-dependency. The
    `replaced_image` test target requires `image-decode`.
  - The five dependents: `genet-livery` taken with
    `default-features = false`, and a default-on `image-decode` forwarding
    `genet-livery/image-decode`. `genet-documents` and `genet-scripted`
    forward through `?` and also forward to `genet-render`. `genet-scripted`
    keeps `genet-render`'s `livery` and `accesskit` explicitly.

**Done when:**
- `genet-livery` tests pass with and without `image-decode`. The
  decode-point unit test proves both states, and any test that needs decoded
  pixels is gated, not deleted;
- `genet-livery` checks for `wasm32-unknown-unknown` with the feature off;
- `cargo tree` shows `image` absent from `genet-render` (without defaults,
  with `livery`) and from `taproot` (without defaults), and present in every
  default build that had it before;
- the dependents check, and `genet-render` and `taproot` tests pass;
- the lane is told the commit, plus the Mere-side rows it must change (below).

## What Mere must change to use it

This is reported to the lane; it is not genet's to edit. Every Mere row that
reaches `genet-livery` in the cone has to drop default features:
- the root workspace's `genet-livery` row, used by cambium-rootstock;
- the `genet-render` row;
- the `taproot` row;
- `ports/graphshell/web`'s own rows.

Each keeps the non-image features it uses: for `genet-render`, `livery`, and
`accesskit` where the native bridge needs it. Desktop builds keep decoding as
long as one path in their graph asks for it.

## Progress

- 2026-10-05: rulings 1–3 recorded. The implementation is written, and
  verification is running in `worktrees/genet-image-decode` with a copy of
  the primary checkout's lock, `--locked --offline`. The worktree is used
  because a build in the primary checkout would rewrite its ignored lock with
  local-path entries. Pending lanes copy that lock.
- 2026-10-05: verified, then landed. The worktree carried byte-identical
  copies of every changed file (checked with `cmp`). It was built with target
  `C:/t/cargo-targets/genet-image-decode`, `--offline`, and `nova_vm` patched
  to `crates/vano` at `47f8d4f9`. Logs are under
  `Code/testing/genet-image-decode/`.
  - **`genet-livery` with the feature:** 655 passed, 0 failed, 6 ignored
    over 47 test targets. One more than S1.1's 654 receipt: the decode-point
    test.
  - **Without it:** 638 passed, 0 failed, 6 ignored over 46 targets, with no
    warnings. That is 655 minus the 15 gated tests and `replaced_image`'s two;
    the off build runs the "decodes nothing" test instead.
  - **wasm32:** `cargo check -p genet-livery --target wasm32-unknown-unknown`
    passes with and without the feature.
  - **`cargo tree -e normal`, `image` rows:**

    | Package and features | Default | Off |
    |---|---:|---:|
    | `genet-livery` | 1 | 0 |
    | `genet-render` (off: `--features livery`) | 1 | 0 |
    | `taproot` | 1 | 0 |
    | `genet-documents --features livery` | 1 | 0 |
    | `genet-scripted` (off: `--features livery`) | 1 | 0 |
    | `ortet` (default; 0 before the ortet fix) | 1 | — |
    | `genet-wpt` (default) | 2 | — |

    `genet-render-host` shows 0 because it does not depend on
    `genet-livery`.
  - **Dependents:** `cargo check` passes for `genet-render`, `taproot`,
    `genet-scripted`, `genet-render-host` and `ortet` (default), for
    `genet-documents --features livery`, for `genet-render` and `taproot`
    without defaults, and for `genet-wpt`. `cargo test -p genet-render -p
    taproot`: 68 passed, 0 failed.
  - **Formatting:** `rustfmt --check` is clean for `src/image_decode.rs`. The
    one diff it reports in `tests/paint.rs` (line 555) is already in `HEAD`.
