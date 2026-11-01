# genet's taffy fork — patch log

This is `genet-taffy 0.16.0`, based on upstream `taffy 0.15.1`,
re-vendored on 2026-10-28 from the current Genet fork of `taffy 0.13.0`.
The current fork changes were captured and reproduced against pristine 0.13.0
before porting; the previously committed complete delta had missed later
changes in five source files.

Buckram and genet-livery use the published package under the dependency key
`taffy`. Genet develops it through the root path patch. The separate, unused
workspace `taffy` declaration also follows upstream 0.15.1.

Upstream now supplies intrinsic sizing keywords, including the representation
of `flex-basis: content`; the old private compact-length tag and separate
`FlexBasis` representation are retired. `FlexBasis` is a compatibility alias
for upstream `Dimension`. The authored-cross-size content-basis rule and the
wrapped-column automatic-minimum marker remain necessary and have regression
tests. Upstream's definite stretched-cross-size behavior is preserved for
`auto` flex bases.

The new upstream out-of-flow pass and containing-block traits replace the old
formatter-local absolute-layout algorithms. Buckram implements those traits
while retaining its own CSS containing-block authority. Formatter static
positions, resolved positioned-grid areas, physical size containment, float
exclusion bands, CSS flex/grid order and the automatic-minimum cache partition
remain Genet extensions. Grid order is applied before placement, with paired
placement data kept in source order afterward.

The feature patch files and upstream drafts below are historical explanations,
not patches verified against 0.15.1. The complete delta is the reproduction
record; its byte-exact apply must be checked whenever source changes.

## How to keep it in sync

`0000-complete-fork-delta.patch` is the **authoritative record**: pristine
upstream plus that one file reproduces the vendored `src/` byte for byte. It is
generated mechanically and checked by applying it to pristine upstream and
comparing every source file; regeneration alone does not prevent drift. The per-feature
patches below exist for upstreaming individual fixes and are **not** a
reproduction record — see the accuracy note at the end of this section.

**0.16.0 upstream refresh (2026-10-28).** The complete delta is now based on
published upstream Taffy 0.15.1. It changes 19 source files and reproduces all
52 vendored `src/` files byte for byte when applied to the pristine registry
source. Its SHA-256 is
`cfbdbf78b6109b4a7f0c0a2def47f2a88d6dddb98f44089b9e9c7a83ae86e1d0`
and its size is 62,939 bytes. The standalone all-features run passes 196 unit
tests and five doctests, including the retained automatic-minimum/cache
regressions, borrowed-style grid order and an authored content-basis ratio
regression. The consumer migration also preserves bare alignment keyword
safety, uses the new minimum/maximum-size vocabulary, and corrects Livery's
obsolete `align-content` initial value to `normal`. The extracted package
passes the same 196 unit tests and five doctests. Buckram, Genet-Livery and
Livery together pass 1220 tests, with 12 pre-existing ignored tests.
See `testing/genet/receipts/taffy-upstream-0151-20261028.json` in the repository
for evidence hashes and the retained pre-fix failures.

Historical source and release receipts below describe their original
snapshots; their hashes and counts do not describe the current package.

**Historical 0.13.1 release refresh (2026-08-26).** Regenerated `0000` against the
published upstream `taffy 0.13.0` source after the grid static-position
callback gained its auto-edge, container-border, and border-box inputs. A
packaged-source all-feature test also exposed a missing `size_containment`
field in upstream's explicit `Style` default fixture; the fixture now records
the fork field's `false`/`false` default. A clean round trip reproduced all 50
vendored `src/` files byte for byte. The resulting patch SHA-256 is
`B54137572659DEA2384129C9A1BF13C31EC48313D8F035A7209ED12747C04521`.

**0.13.1 Row 18 content-basis refresh (2026-08-27).** Regenerated `0000`
after the `flex-basis: content` Step B path was corrected to derive an
aspect-ratio basis from an authored cross size only. In particular, a
main-axis minimum transferred through the ratio cannot overwrite an explicit
cross size and feed back into the content basis. A pristine-source apply and
full `src/` comparison reproduce the vendored tree byte for byte; the
standalone all-features run records 131 unit tests and 5 doc tests passing.
The resulting patch SHA-256 is
`BDE982DE04A4DF21FE50E071E07220044E973095793A31F11276D8EE8238AF7D`.

**0.14.0 release (2026-08-27).** Regenerated `0000` against a
fresh extraction of the published upstream `taffy 0.13.0` crate after the
Row 18 automatic-minimum repair. The delta changes 16 `src/` files (855 added
lines, 127 removed, 86 hunks); applying it with `patch -p1` to a clean
upstream copy reproduces all 50 vendored `src/` files byte for byte. Its
SHA-256 is `29185D83256125C5BBD948354128353010807644A5518F0B3B5FD8BE609121B2`
and its size is 73,200 bytes.

The final 69-file packaged crate passed offline `cargo package` verification,
then its extracted source passed the all-features suite: 139 unit tests and 5
doc tests.

`genet-taffy 0.14.0` was published from `e8a67b06a4b`, then merged to `main`
at `8b4e14e7853`. The annotated `genet-taffy-v0.14.0` tag peels to that
release commit. The registry archive has 69 files and SHA-256
`50F2A560C4025930D7138D18BA78C55B3C40E562A2927B3D58E871771DB0676D`.
Consumer resolution is clean; the release-native receipts are Buckram 255 / 255,
Genet-Livery automatic minimum 1 / 1, flex-basis content 2 / 2, and its
content-basis repro 6 / 6.

`git diff --check` reports ten trailing-space diagnostics within the generated
delta. They are existing vendored `src/` bytes represented as added lines;
they remain deliberately so the authoritative patch preserves the byte-exact
round trip.

The repair adds the public
`SizingMode::ContentSizeForAutomaticMinimum` marker. It has the same leaf
measurement behavior as `ContentSize`, but confines the wrapped-column
automatic-minimum contribution to the Flexbox call site and partitions the
layout cache. Downstream code that exhaustively matches the public
`SizingMode` enum must add an arm or a wildcard before adopting 0.14.0. This
minor bump is required because adding a public enum variant is a breaking
change under 0.x semver.

To bump taffy:

1. Regenerate the delta against the release currently vendored, and prove it
   round-trips, before touching anything:
   ```
   diff -ruN <pristine-current>/src src > 0000-complete-fork-delta.patch
   ```
2. Re-vendor the new release over `src/`, `examples/`, `Cargo.toml`, `README.md`.
   Everything else in this directory is genet's and stays.
3. Apply the delta, resolve the rejects, and drop any hunk upstream has since
   made redundant (record the retirement in the ledger below).
4. Regenerate `0000-complete-fork-delta.patch` against the **new** pristine
   release, and re-prove the round trip.
5. Refresh the per-feature patches you touched, and bump every consumer's
   version requirement (`components/buckram`, `components/genet-layout`,
   `components/genet-livery`, `components/genet-render`, `components/paint`,
   `ports/genet-wpt`, `support/patches/stylo_taffy`). The workspace's separate
   plain `taffy 0.10.1` is unrelated and stays.

Verify by content, never by exit code. `git apply` run inside the
`worktrees/genet-k5-positioning` worktree reported success while writing
nothing, which briefly showed a stale patch set as applying cleanly. Use
`patch -p1 -d <dir>` and confirm with `diff -rq` plus a grep for a marker the
patch introduces.

**Accuracy note (2026-08-16).** The per-feature patch files do not reconstruct
this fork, and the previous claim here that `src/` was pristine except for the
listed files was wrong. Applying 0001, 0002, 0003 and 0006 to pristine 0.12.1
leaves eleven files still differing: 0004 and 0005 ship no `.patch` file at
all, `Layout::static_location` (`src/tree/layout.rs`) and a `src/util/print.rs`
change were documented nowhere, and 0006's own file no longer applies to its
stated base (2 of 3 hunks fail in `grid/alignment.rs`). `0000` exists so this
cannot recur; re-derive it whenever the tree changes.

## Patches

### 0000 — formatter static position (`0000-complete-fork-delta.patch`)

**Files:** `src/tree/layout.rs`, `src/compute/block.rs`,
`src/compute/flexbox.rs`, `src/compute/grid/alignment.rs`
**Upstream status:** genet-only, and load-bearing for Buckram's K5 positioning
work. Undocumented until 2026-08-16; it has no standalone `.patch` file and
travels in the complete delta.

`Layout` carries only the final, inset-applied `location`, so an embedding
engine cannot see where a formatter would have placed an out-of-flow box before
CSS positioning applied its insets. That pre-inset coordinate is the CSS
*static position*, and an engine owning its own containing-block graph needs it
as an input rather than a result.

This adds `Layout::static_location`, and has each formatter record its
alignment result there before insets are applied: block sets it at the
normal-flow cursor, flex hoists its main- and cross-axis alignment into
`static_offset_main` / `static_offset_cross` so the aligned position survives
inset placement, and grid records the position produced within the area 0006
selects. Buckram reads it through `AlgorithmTree::static_layout`.

It also carries the Row 18 flex content-basis correction. When a flex item
has both an explicit cross size and an intrinsic ratio, Flexbox Step B may use
that pair to form the content basis. The fork takes the cross min/max from the
authored cross axis at that point, rather than min/max values transferred from
the main axis. This keeps a main-axis constraint from replacing the explicit
cross size before the ratio is applied.

Note for the next re-vendor: taffy 0.13.0 reworked the flex abspos alignment
this hoist sits in (writing-mode-relative `start`/`end`, new
`AlignItems::SELF_START`/`SELF_END`), so the flex half needs a genuine rebase
rather than a mechanical re-apply.

It also carries the Row 18 automatic-minimum repair. The flex algorithm uses
the marker only while measuring a column item's max-content automatic minimum
with an auto main-axis minimum and non-scroll main-axis overflow. Wrapped
column min-content contributions sum only in that context; rows, explicit
minimums, definite sizing, and cross-axis-only scroll retain their prior
paths. The cache mode includes `SizingMode` so an inherent/content/automatic-min
measurement cannot reuse the wrong cached result.

### 0001 — `find_content_slot` width-fit (patch file deleted 2026-08-16)

> **RETIRED — fixed upstream in taffy 0.13.0.** Do not carry this patch past
> the 0.13 bump. Upstream added a `find_bfc_slot` API plus a caller-side loop
> in `compute/block.rs` that walks down to the first band wide enough for the
> item's border box, which is a strict superset of this fix: it also resolves
> the auto-width case against the slot's stretch width and sets
> `item_pushed_below_float` so the item's top margin stops collapsing with its
> parent's. `find_content_slot` still has the original width-blind behaviour,
> but it is no longer the API the BFC-avoids-floats path uses. Drop the PR
> draft in `UPSTREAM_PR.md` too.

**Files:** `src/compute/float.rs`, `src/compute/block.rs`
**Upstream status:** present on taffy `main` too (unfixed as of 0.12.1); PR
drafted (see `UPSTREAM_PR.md`, PR 1).

`FloatContext::find_content_slot` chose the first vertical band below `min_y`
without regard to whether the placed content is wide enough to fit there. A
full-width float makes that band zero-width (insets consume the whole
container), so a fixed-width BFC child was placed at the float's right edge and
overflowed, instead of dropping below the float to where it fits.

The fix threads the content's outer width through as `min_width`: the chosen
band must be at least that wide, otherwise the slot clears below all floats
(full container width). `min_width == 0` (auto-width / shrink-to-fit content,
which reflows to whatever the band offers) preserves the prior first-band
behaviour, so only fixed-width BFC children change. The block-layout caller
passes `item.size.width + non-auto x-margins` for fixed-width items and `0.0`
for auto-width.

Reftest moved: `css/CSS2/floats/floats-wrap-bfc-008` (fixed-width BFC clearing a
full-width float) now matches its reference.

### 0002 — float exclusion-band accessor (`0002-exclusion-bands.patch`)

**Files:** `src/compute/float.rs`, `src/compute/block.rs`, `src/compute/mod.rs`
**Upstream status:** genet-only so far (the inline IFC seam it feeds is
genet's parley-measured leaf, which upstream taffy does not model). Additive —
no existing taffy behaviour changes. PR drafted (`UPSTREAM_PR.md`, PR 2).

Inline text wrapping *around* a float needs each line box to know the width the
floats leave at its own y. taffy places floats (the `float_layout` feature) but
exposes only `find_content_slot` (one slot for one block child); it has no way
to hand a paragraph's line breaker the full set of exclusion bands.

This adds a read-only accessor and a small value type, leaving placement
untouched:

- `float.rs`: `InlineFloatBand { y_start, y_end, left, right }` and
  `FloatContext::exclusion_bands(min_y) -> Vec<(Range<f32>, [f32; 2])>` — a thin
  filter over the existing `segments` walk (segments at/below `min_y` that
  impose an inset on either side), in BFC-root space.
- `block.rs`: `BlockContext::inline_exclusion_bands(min_y) -> Vec<InlineFloatBand>`
  — the same coordinate handling as `find_content_slot` (subtract `y_offset` for
  block-local y; `max` each segment inset with the content-box inset and re-base
  to the content-box edge), but returning every band rather than one slot.
- `mod.rs`: re-export `InlineFloatBand`.

Consumed in genet-layout: the box tree snapshots these bands per inline-context
leaf into `TextMeasureCtx`, and the parley measure drives `Layout::break_lines()`
with per-line `set_line_x` / `set_line_max_advance` so lines wrap to a float's
side and reclaim the column below it (the float-wrap first cut;
`docs/2026-06-18_float_wrap_spike.md`). Known limit: only x-axis content-box
insets are tracked, so a top padding/border on the leaf is not yet reflected in
the band's `y` (fine for the common no-top-padding case).

### 0003 — flex `order` support (`0003-flex-order.patch`)

**Files:** `src/style/flex.rs`, `src/compute/flexbox.rs`
**Upstream status:** genet-only. This taffy version does not model CSS `order`
at all — `FlexItem.order` is the document index (used only for paint/output
ordering), and the flex algorithm processes items in document order. PR
drafted (`UPSTREAM_PR.md`, PR 3).

CSS `order` lays flex items out (and paints them) in *order-modified document
order*: items sort by ascending `order`, ties broken by document order.

- `flex.rs`: add `FlexboxItemStyle::order() -> i32` (default 0). Adapters that
  don't override it keep document order, so existing behaviour is unchanged.
- `flexbox.rs`: `FlexItem` gains a `css_order: i32` field, populated from
  `child_style.order()` in `generate_anonymous_flex_items`; after collection the
  item vec is `sort_by_key(|i| i.css_order)` — a *stable* sort, so equal-`order`
  items (the common case, 0) keep document order. The pre-existing `order: u32`
  field is left as the document index for paint/output ordering, so paint order
  is unchanged (a deliberate first-cut limit: CSS `order` also re-orders
  painting, but genet paints in document order regardless, and flex items rarely
  overlap).

Consumed in genet-layout: `box_tree.rs`'s `CssStyle` flex-item wrapper overrides
`FlexboxItemStyle::order()` to read `get_position().order` off the cascade (the
same wrap-and-override pattern it already uses for grid placement; no
`stylo_taffy` patch needed). Verified by `flex_order_reorders_items` and
`flex_order_is_stable_and_handles_negative`.

### 0004 — physical-axis size containment

**Files:** `src/style/mod.rs`, `src/compute/block.rs`,
`src/compute/flexbox.rs`, `src/compute/grid/mod.rs`
**Patch file:** none — travels in `0000-complete-fork-delta.patch`.
**Upstream status:** genet-only. The API shape should be reviewed against
taffy's current containment direction before upstreaming.

CSS size queries require the query container's size to be independent of the
descendants whose styles the query can change. Taffy had no style input for
excluding child content from intrinsic sizing, so `container-type: size` could
oscillate on a size it created itself and `inline-size` could feed the queried
axis from its descendants.

This adds `CoreStyle::size_containment() -> Size<bool>` and the matching
`Style::size_containment` field. Each boolean names a physical axis whose
intrinsic content contribution is zero:

- block layout substitutes the content-box inset for contained intrinsic width,
  gives a contained auto height a zero-content definite size, and prevents
  child margin collapse through the containment boundary;
- flex and grid inject the padding-and-border floor only when a contained axis
  has no incoming or authored definite size, so ordinary stretched axes remain
  stretched while shrink-to-fit axes ignore child content;
- grid keeps that contained outer size through final track sizing instead of
  replacing it with the intrinsic track sum.

Livery maps `container-type: size` to both physical axes and
`container-type: inline-size` to width or height according to `writing-mode`.
The `stylo_taffy` full-style literal initializes the additive field to false,
preserving the incumbent Stylo route until it opts into the seam. Native
receipts cover block, flex, grid, horizontal inline-size, and vertical
inline-size containers.

### 0005 — positioned grid-area diagnostics

**Files:** `src/compute/grid/mod.rs`
**Patch file:** none — travels in `0000-complete-fork-delta.patch`.
**Upstream status:** additive and genet-only so far. The API should be reviewed
alongside Taffy's existing detailed grid output before upstreaming.

`DetailedGridInfo` previously retained ordinary grid-item placements but
discarded the finalized area used to lay out an absolutely positioned child.
An embedding layout engine could therefore observe the child after Taffy had
applied insets and self-alignment, but could not receive the distinct grid area
that CSS Positioned Layout uses as the child's containing block.

This adds `positioned_items`, keyed by Taffy's child `NodeId`, with the exact
`Rect<f32>` constructed by the grid algorithm after line resolution, implicit
track selection, writing direction, borders, scrollbars, and track alignment.
It changes no layout. Buckram reads the rectangle as formatter evidence and
keeps CSS containing-block selection and used geometry outside Taffy.

### 0006 — positioned grid static-position area (`0006-grid-static-position-area.patch`)

> **Its `.patch` file is historical and must not be applied**: 2 of 3 hunks in
> `grid/alignment.rs` no longer apply even to pristine 0.12.1, and the callback
> gained three more inputs after the 0.13 bump. The current source is
> recorded by `0000-complete-fork-delta.patch`.


**Files:** `src/tree/traits.rs`, `src/compute/grid/mod.rs`,
`src/compute/grid/alignment.rs`
**Upstream status:** additive and Genet-specific so far. The callback belongs
with a future upstream discussion of whether a grid formatter should expose a
static-position rectangle separately from its positioning area.

CSS Positioned Layout makes two different choices for a direct absolutely
positioned grid child: the grid's content edges define the static-position
rectangle by default, while its specified grid area does so only if this grid
also established the child's actual containing block. Taffy previously used
the grid area for both jobs, which cannot express that rule for an embedding
engine with its own containing-block graph.

This adds a backwards-compatible `LayoutGridContainer` callback. It receives
the direct child, finalized grid area, grid content box, which grid-area edges
came from `auto`, and the container's border inputs, then returns the alignment
area used only to calculate `Layout::static_location`. Its default follows upstream 0.15.1 containing-block selection. Buckram selects the content box unless K5a selected the same
grid as the actual containing block; in that case, each `auto` grid line uses
the corresponding padding edge. Detailed grid-area diagnostics and ordinary
absolute layout remain unchanged.

## What upstream support does *not* cover

Read this before deleting genet code that a taffy release note appears to
make redundant. The trap is real: it was walked into on 2026-08-16 during
the 0.13 bump and caught only by checking taffy's source.

### `self-start` / `self-end` alignment

taffy 0.13.0's notes announce `AlignItems::SELF_START`/`SELF_END`, resolved
against the item's own direction, for both in-flow and absolutely positioned
flex and grid items. That reads like it subsumes Livery's hand-mapping in
`components/genet-livery/src/layout.rs`, `map_grid_static_self_alignment`,
`self_alignment_for_axis`, `subject_side_on_axis`,
`map_vertical_grid_static_alignment`, `set_physical_self_alignment`,
`same_physical_axis`. **It does not. Do not delete them.**

taffy's `Alignment::resolve_self_relative` flips only when
`axis_is_inline && item_direction != container_direction`, and `Direction`
has exactly two variants, `Ltr` and `Rtl`. Its own doc comment states the
boundary: taffy supports only the `horizontal-tb` writing mode, so in the
block axis `SelfStart`/`SelfEnd` always collapse to `Start`/`End`.

Livery's `subject_side_on_axis` resolves against the subject's *full* flow
axes, it tests whether the subject's inline axis coincides with the target
physical axis and falls back to the subject's block axis, which is exactly
the vertical-writing-mode case upstream excludes. Genet supports those flows
and carries receipts for them.

Deleting this would move absolutely positioned static positions in vertical
writing modes while every horizontal test kept passing, so the suite would
not catch it. Revisit only if `Direction` grows vertical variants, or taffy
takes a real writing-mode model.
