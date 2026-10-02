# Livery line breaking: Parley as a library

**Status:** plan, 2026-10-02. Mark ruled "Plan Stage 1 now". Stage 0 (non-wrapping
lines follow `text-align`) landed on main as `6fca091dc26`. Stage 1 has not
started, and its bidi-level question is open (see [Open for Mark](#open-for-mark)).

## Why

Livery hands line breaking, alignment, justification, `text-indent` and bidi
line reordering to Parley's breaker. Parley's model and CSS's disagree in
places that a Parley patch cannot fix without changing Parley's public types.
The text-align lane of 2026-10-02 showed this directly.

- **The gap that started it.** Pictograph's centred card titles
  (`white-space: nowrap`) came out left-aligned. The cause was general: Livery
  breaks every non-wrapping paragraph as one unbounded Parley line, because
  Parley breaks at an inline box whatever the text's `TextWrapMode` is. Parley
  then aligns an unbounded line within the longest line, so `text-align` could
  not move a single non-wrapping line.
- **Three prototypes, one 14-directory WPT reftest run.**
  - **L**: a Livery-only per-line shift; no Parley patch.
  - **C**: a Parley seam giving a line an alignment width separate from its
    breaking width.
  - **B**: Parley's inline boxes honour `NoWrap`, and Livery breaks
    non-wrapping text within the width.

  L and C each fix `text-align-end-015`, `text-align-end-017` and
  `text-align-start-014`, with no other transitions.

  B regresses `css/css-text/line-breaking/line-breaking-031` and `-032`. Those
  tests check CSS Text's rule that the `white-space` of the nearest common
  ancestor of a character and an atomic inline governs breaking between them.
  Parley's `InlineBox` (`support/patches/parley/src/inline_box.rs:6`) carries
  an id, kind, byte index, width and height, but no style. So a breaker patch
  can only use the wrap mode of the text before the box. B cannot be right
  without changing Parley's public `InlineBox`.
- **What Livery already owns.** Livery already builds the line box itself
  (CSS 2.1 §10.8, [line box model](2026-09-25_line_box_model_plan.md)), with
  Parley kept "upstream-shaped rather than patched" by that plan's ruling. The
  breaking and placement of lines is the remaining large piece that CSS
  defines and Parley decides.

Evidence: probes, test logs and WPT maps are under
`C:/Users/mark_/Code/testing/genet-gaps/`. WPT is under `wpt/`: binaries with
sha256 in `wpt/build_variants.out`, maps in `base/`, `C/`, `B/` and `L/`, and
per-file diffs in `diff_base_{B,C,L}.txt`.

## Rulings

Mark's words, verbatim, as recorded in
`mere/design_docs/mere_docs/implementation_strategy/2026-09-25_graphshell_one_tree_plan.md`
§1 "Genet gaps" (mere `7b2fcae1`):

1. On the fix, after C was first prototyped: "Is it possible to reach spec
   correctness while treating parley like a lib (much as we're doing with
   taffy…?) and building our implementations…? I lean to b but would accept c
   if the rationale were proven out by prototyping b"
2. On patching genet-parley: "I would prefer a livery only solution. So
   what's the path to css conformation look like and necessitate?"
3. Choosing among L, C and B after the WPT run: "L, if we can address the
   gaps later. if not, C". L's two gaps are this plan's named targets (below),
   so L landed.
4. On the path to conformance: "Plan Stage 1 now". The alternatives were
   patching case by case, or upstream first.

*Reading, not ruled:* Stage 1 keeps genet-parley unpatched unless Mark rules the
bidi seam (Open, below), following ruling 2 and the line box model plan's
"upstream-shaped rather than patched".

## The taffy precedent

Genet uses Taffy as a library under Buckram:

- **What Buckram owns.** Buckram (`components/buckram`, about 13.9k lines)
  owns the box tree, the fragments, and block, float, table and
  fragmentation layout.
- **What Taffy supplies.** Flex and grid, through its trait API
  (`components/buckram/src/taffy_adapter.rs:17-20`: `LayoutPartialTree`,
  `compute_flexbox_layout`, `compute_grid_layout`, `compute_root_layout`).
- **It is still a fork.** Taffy is `genet-taffy`, a 16-file delta (+855/−127)
  with patches for exclusion bands, flex order, grid static position and a
  size-containment seam (`support/patches/taffy/GENET_PATCHES.md`).

So the precedent is to own the algorithm where the library falls short, and
keep any library change a small, upstreamable seam. It is not "no patches".

Applied to text:

- **Parley keeps:** font selection and fallback (fontique), itemisation,
  shaping (HarfRust), clusters and glyph advances, UAX#14 and UAX#29
  segmentation, and bidi analysis for shaping direction.
- **Livery takes:** deciding where lines break, placing them, and reordering
  them.

## What Livery owns and delegates today

At main `6fca091dc26`. Line numbers are in `components/genet-livery/src/text.rs`
unless named.

| Concern | Owner today | Where |
|---|---|---|
| Inline item collection, owner chains | Livery | `InlineCollector` :3412, `BoxInlineCollector` :3015 |
| White-space collapsing and preservation | Livery | `normalized_text` :3859, `append_preserving_breaks` :3999, `collapse_css_whitespace` :4064 |
| Text-transform, soft-hyphen stripping for `hyphens: none` | Livery | `transform_text` :3916 |
| Atomic inlines, inline edges (padding, border, margin), empty-line boxes, markers | Livery builds; Parley places as `InlineBox` | `push_atomic_box` :3165, `push_edge` :3202; `push_inline_box` :1218 |
| Font fallback, itemisation, shaping, bidi analysis, UAX#14 | Parley | `builder.build` :1230; CSS mapped to Parley styles in `push_defaults` :4084, `push_span` :4118 (word-break :4283, overflow-wrap :4294, text-wrap-mode :4317, tab size, letter and word spacing) |
| Line breaking, float bands | Parley's breaker, driven by Livery | `break_inline_lines` :111 (`break_all_lines`, or a `BreakerState` loop using `set_line_x/y/max_advance`, `revert`, `committed_y`); bands from Buckram's `FloatLineConstraints` (`components/buckram/src/block.rs:1165`, :1192, :1214) |
| Non-wrapping lines | Parley breaks unbounded; Livery places (Stage 0) | `places_lines` :1341, `non_wrapping_line_shift` :4230 |
| `text-indent` | Parley, with Livery's amount (hanging punctuation included) | `set_text_indent` :1237, `first_hanging_advance` :1170 |
| `text-align`, `text-align-last`, justification | Parley | `layout.align` :1255; mapping `text_alignment` :4211, `last_line_alignment` :4267 |
| Bidi line reordering (UAX#9 L2) | Parley | `reorder_line_items` (parley `layout/line_break.rs:1448`) |
| Line-box height, baseline, `vertical-align`, line-height quirk | Livery | line loop from :1346; `vertical_align_raise` :4356, `inline_box_placement` :4431, `LineExtents` :4485, `line_height_quirk` :4381 |
| Fragments, hit-testing, caret, selection | Livery, from its own retained clusters | conversion :1529-1690; `text_position_at_point` :2358, `caret_rect` :2503, `text_selection` :2545 |
| Min- and max-content of inline content | Parley's breaker (and its `calculate_content_widths`, parley `layout/data.rs:570`) | via `intrinsic_kind` into `push_defaults` |
| Hyphen glyph at a soft-hyphen break, `hyphens: auto` | nobody | — |

What Parley exposes without a patch:

- **Per cluster** (`layout/cluster.rs`): `is_soft_line_break` :183,
  `is_hard_line_break` :192, `advance`, `glyphs`, `text_range`,
  `is_space_or_nbsp`, `is_rtl`.
- **Per run:** font, size and `is_rtl` (`layout/run.rs:95`).
- **Not exposed: bidi embedding levels.** Only their parity is public.

## Stages

### Stage 0: non-wrapping lines follow `text-align` (landed 2026-10-02)

`6fca091dc26`, L. Its two known gaps are Stage 1 targets T1 and T2.

### Stage 1: Livery owns line breaking

Livery decides break points, line placement, alignment, justification,
`text-indent` and reordering over Parley's shaped clusters. Parley still
builds and shapes the paragraph once, as today. Livery then reads it as a
logical item stream, with `break_all_lines(None)` giving one line per
mandatory break. Shaping before breaking is what Genet does today, so this
loses no shaping quality.

**Named targets:**

- **T1, case 15:** a `white-space: nowrap` span holding an atomic inline,
  inside wrapping text, must not break at the box. Fixture: a 60px block,
  `aa <span style="white-space:nowrap">cc <inline-block>bbb</inline-block>
  dd</span> ee`, gives three lines `aa ` / `cc bbb dd` / `ee`. Today it breaks
  at the box.
- **T2, non-wrapping `justify-all`:** `text-align: justify-all` justifies a
  non-wrapping line across its containing block. Today a 200px nowrap `a b c`
  stays at its natural width.
- **T3, common-ancestor `white-space`:** `line-breaking-031` and `-032` stay
  passing. They are the cases B broke.

**WPT gate set.** Reftests, release build, `--renderer livery`, these 14
directories:

- `css/css-text/`: `text-align`, `white-space`, `text-indent`,
  `text-justify`, `line-breaking`, `overflow-wrap`, `word-break`,
  `letter-spacing`, `hanging-punctuation`;
- `css/css-inline`;
- `css/CSS2/`: `linebox`, `text`, `floats`, `bidi-text`.

Measured at the main tree (the L build, which is byte-identical to main's
tree; binary sha256 `a5951ddc…3325f` in `wpt/build_variants.out`):

| Directory | Pass | Fail | Skip | Files |
|---|---:|---:|---:|---:|
| css-text/text-align | 53 | 33 | 28 | 114 |
| css-text/white-space | 168 | 243 | 41 | 452 |
| css-text/text-indent | 11 | 9 | 7 | 27 |
| css-text/text-justify | 4 | 11 | 6 | 21 |
| css-text/line-breaking | 68 | 59 | 4 | 131 |
| css-text/overflow-wrap | 30 | 19 | 8 | 57 |
| css-text/word-break | 49 | 22 | 34 | 105 |
| css-text/letter-spacing | 7 | 23 | 3 | 33 |
| css-text/hanging-punctuation | 7 | 13 | 3 | 23 |
| css-inline | 11 | 119 | 129 | 259 |
| CSS2/linebox | 162 | 28 | 59 | 249 |
| CSS2/text | 291 | 97 | 183 | 571 |
| CSS2/floats | 65 | 36 | 43 | 144 |
| CSS2/bidi-text | 63 | 42 | 3 | 108 |
| **Total** | **989** | **754** | **551** | **2294** |

The 33 text-align failures that remain: `text-align-start` 7, `-end` 7,
`-justifyall` 5, `-justify` 4, `-justify-tabs` 4, `text-align-last` and its
justify variants 4, `-match-parent` 2.

#### Phase S1.0: committed baseline

- Commit reftest expectations for the 14 directories, measured at the commit
  Stage 1 starts from, under `ports/genet-wpt/expectations/reftest/`.
- Add them to `support/wpt/check-reftest-baselines.ps1`, a local GPU guard
  (not CI).

**Done when:** the check script reports `unexpected=0` for all 14 at that
commit, and its counts are recorded here against the table above.

#### Phase S1.1: a logical item stream with break opportunities

- **The stream.** In Livery, read Parley's shaped paragraph into one logical
  sequence of clusters and inline boxes. Each cluster carries advance,
  glyphs, source span, and Parley's soft and hard break flags.
- **Opportunity rules.** Livery decides opportunities from that stream plus
  CSS's rules:
  - the `white-space` of the nearest common ancestor at each boundary, using
    the owner chains Livery already keeps (`SourceSpan` and `InlineAtom`
    owners);
  - opportunities around atomic inlines;
  - `overflow-wrap` and `word-break` emergency opportunities;
  - preserved and hanging spaces, and tabs.

**Done when:** a unit fixture per rule passes, including T1's and T3's
boundaries. On the existing `css_text_lane` and `line_box_model` fixtures,
the opportunity set matches the breaks Parley produces today, wherever
today's output is correct.

#### Phase S1.2: the Livery breaker

- **The breaker.** Greedy line fitting over the S1.1 stream. It covers:
  - float bands, porting the `BreakerState` loop at `text.rs:111-183`,
    including moving a line down to a wider band;
  - `text-indent`, including each-line and hanging;
  - hanging punctuation as today;
  - min- and max-content from the same rules.
- **Wiring.** The line loop at :1346 consumes Livery lines. It no longer
  calls Parley's `break_lines`, `break_all_lines(Some(_))` or
  `set_text_indent`.

**Done when:**

- every genet-livery, livery, buckram and dependent test passes, including
  the K3p receipt `live_nowrap_nested_inline_content_uses_float_bands_in_both_directions`
  (`components/genet-livery/src/layout/tests.rs:5605`) and
  `table_sibling_blocks.rs:178` and `:257`;
- T1 passes, and T3's two WPT tests stay passing;
- the WPT gate set shows no `pass -> fail` against S1.0.

#### Phase S1.3: alignment and justification

- **Alignment in Livery.** Livery places each line by `text-align` and
  `text-align-last` within its line box or float band. Justification
  distributes free space over word separators. `text-justify: none` and
  `auto` are covered; `inter-character` is Stage 2.
- **Retirements.** This retires `layout.align` and `non_wrapping_line_shift`.

**Done when:** T2 passes, Stage 0's `text_align_places_non_wrapping_lines`
stays passing, and text-align and text-justify show no `pass -> fail`
against S1.0.

#### Phase S1.4: bidi line reordering

- **Reordering in Livery.** Livery reorders each line's items visually
  (UAX#9 L1 and L2) from embedding levels. Where the levels come from is the
  open question below.

**Done when:**

- a fixture with three embedding levels on one line (European digits inside
  an RTL span inside LTR text, levels 0, 1 and 2) orders as UAX#9 does;
- `CSS2/bidi-text` shows no `pass -> fail`.

#### Phase S1.5: retire the Parley breaker from Livery

**Done when:**

- Livery calls no `BreakLines`, `BreakerState`, `Layout::align` or
  `set_text_indent`;
- Parley is used for building, shaping and one `break_all_lines(None)` per
  paragraph;
- every gate above passes;
- the WPT set's final counts are recorded in Progress against the S1.0
  baseline.

### Stage 2: CSS beyond Parley's model

Each item is Livery-only once Stage 1 owns breaking. These are not scheduled.

- `white-space: break-spaces` and the pre-wrap hanging rules. White-space is
  the largest failing directory.
- `line-break: loose | normal | strict | anywhere`. Needs Livery's own UAX#14
  tailoring; `icu_segmenter` 2.2 is in the lock.
- `hyphens: manual`: shaping and placing a hyphen glyph at a soft-hyphen
  break.
- `hyphens: auto`: needs a hyphenation dictionary dependency. That is a
  dependency decision for Mark when Stage 2 is planned.
- `text-justify: inter-character`, `text-wrap: balance | pretty`,
  letter-spacing at line ends, and the rest of `hanging-punctuation`.

### Stage 3: shaping at line boundaries

- Reshape line-end segments where a break interrupts joining or ligatures
  (Arabic, Indic). This means re-running Parley's builder on those segments,
  which has a performance cost to measure.
- Then `text-autospace` and `text-spacing-trim`.

## Open for Mark

**Bidi embedding levels (Phase S1.4).** Parley exposes only parity
(`Run::is_rtl`). Parity cannot tell level 0 from level 2, so it cannot
reorder nested embeddings correctly.

1. **Livery runs its own UAX#9 paragraph analysis** with `unicode-bidi`
   (0.3.18, already in the lock), over the same text and base level Parley
   uses. No Parley patch. The cost is a second bidi pass, and the risk that
   it disagrees with the levels Parley shaped with. A fixture must compare
   them.
2. **A one-accessor genet-parley seam** exposing each run's bidi level. One
   bidi pass, an upstreamable patch, and a new entry in
   `support/patches/parley/GENET_PATCHES.md`.

*Reading, not ruled:* option 1 follows ruling 2 and the line box model
plan's "upstream-shaped rather than patched"; option 2 follows the taffy
precedent of small seams.

## Findings

All dated 2026-10-02, verified at Genet `4ac56bbbe0b` and `6fca091dc26`.

- **The unbounded line and the longest-line alignment.** Livery broke
  non-wrapping text unbounded: `break_all_lines(None)`, or an infinite line
  max-advance on the float path. Parley's `BreakLines` drop then sets an
  unbounded line's `inline_max_coord` to the longest line
  (parley `layout/line_break.rs:1183`), and `align` measures free space
  against that. A single non-wrapping line therefore never moved, and an RTL
  one sat at the left edge.
- **Re-breaking within the width (option A) wraps no-wrap lines.** Parley's
  breaker marks a break opportunity after every inline box ("We can always
  line break after an inline box", `layout/line_break.rs:529`). That is
  despite its own `calculate_content_widths` treating a box under `NoWrap` as
  unbreakable (`layout/data.rs:620`). Overflowing nowrap lines holding an
  inline-block or padded span went from 1 line to 3-5.
- **B's two regressions** are the nearest-common-ancestor rule. In
  `line-breaking-031` the boundary is between `<span style="white-space:
  pre">X</span>` and a following `<span style="white-space: pre"><img></span>`
  inside a `white-space: normal` block. The break is allowed by the block,
  but forbidden by the text before the box, which is all Parley's breaker can
  see.
- **The K3p nowrap receipts do not depend on the unbounded line.** B removes
  it, and all 838 genet-livery and livery tests still pass, K3p's included.
  The `CSS2/floats/float-nowrap-*` statuses are identical across base, B, C
  and L.
- **Coverage.** The WPT harness has no committed css-text or css-inline
  expectations yet (`ports/genet-wpt/expectations/` holds 18 maps, none of
  them for text or inline). The last full css-text reftest ledger
  (2026-08-24, `testing/genet/wpt-ledger/2026-08-24_css_text_v2/`) recorded
  979 pass, 407 fail and 578 skip of 1964 files.

## Progress

- 2026-10-02: Stage 0 landed as `6fca091dc26` ("livery: non-wrapping lines
  follow text-align"). `css_text_lane::text_align_places_non_wrapping_lines`
  fails without it and passes with it; text-align WPT went from 50 to 53
  passing. The B and C prototypes stay on throwaway branches
  (`proto/nowrap-parley-b` `7c85004ebd8`, `proto/nowrap-text-align`
  `84701e00dd3`). Plan written.
