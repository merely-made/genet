# Livery line breaking: Parley as a library

**Status:** S1.0–S1.1 complete on `livery-line-breaking`, awaiting main integration, 2026-10-03. Mark ruled "Plan Stage 1
now". Stage 0 (non-wrapping lines follow `text-align`) landed on main as
`6fca091dc26`. This lane stops after S1.1, before S1.2. The bidi-level source
is ruled: Livery runs its own UAX#9 pass (ruling 5).

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

Recorded in this repo on 2026-10-02, in the round that followed the plan:

5. **Bidi levels for S1.4.** Asked where Livery gets embedding levels, given
   that Parley exposes only parity. The options were: an own UAX#9 pass with
   `unicode-bidi`, a one-accessor genet-parley seam, or deciding at S1.4.
   Mark: "Own UAX#9 pass (Recommended)". It follows that Stage 1 needs no
   genet-parley patch at all, and S1.4 must carry the levels-agreement
   fixture named under [Open for Mark](#open-for-mark).
6. **The next Genet lane.** Asked which Genet lane goes next. The options
   were: line breaking S1.0–S1.1, census panic triage, the realms next slice,
   or generated text Row 17. Mark: "1, but prepare a brief for me to paste to
   another agent. then let's keep identifying good targets to make briefs for
   the agent on deck to orchestrate implementation for." So S1.0 and S1.1 run
   in another agent from a pasted brief, and stop before S1.2. *Reading, not
   ruled:* S1.2 onward waits for Mark to review S1.1's result.

The briefing program that ruling 6 started continued in a second round the
same day. Its rulings are recorded here because they extend ruling 6; each
target's own document points back to them.

7. **Further briefs.** Asked "Which of these should I turn into briefs next?",
   with four options, any number selectable:
   - Genet's 7 census panics and 5 evaluation errors;
   - Knot's 3 reported test failures;
   - `text-overflow: ellipsis`;
   - the realms metadata slice.

   Mark selected all four. It follows that the panics, Knot's failures and
   the realms slice are briefed now. Ellipsis is governed by ruling 8.
8. **Where ellipsis belongs.** The options were: this plan, after S1.3; its
   own plan, built now over Parley-placed lines; or record only. Mark:
   "Line-breaking plan, after S1.3 (Recommended)". It is now named target T4
   below. *Reading, not ruled:* its brief is written once S1.3 lands, since
   the placement code it builds on does not exist before then.
9. **Vano's test262.** Asked whether the Vano panic lane may fetch the
   unfetched `tests/test262` submodule (tc39/test262 at `e0d8f66a`). The
   options were: allow, pinned commit only; or don't fetch. Mark: "Allow,
   pinned commit only (Recommended)". The census doc records this beside
   the Vano ruling.
10. **Implementation models.** For this lane Mark authorized GPT-6 Luna,
    Terra, or GPT-6.1 Sol, with the cheaper suitable model preferred.
11. **S1.1 boundary source.** Parley reports selected line breaks, not every
    candidate opportunity, and its unbounded `break_all_lines(None)` does not
    expose those opportunities. Mark ruled: "Use existing ICU4X for S1.1
    candidate boundaries". The implementation may add the existing workspace
    `icu_segmenter` 2.2.0 as a direct `genet-livery` dependency edge; versions
    remain unchanged, and this does not expand the lane into S1.2.
12. **Network for required gates.** Mark authorized normal network access:
    "ok, authorized" and "you needn't stay offline, eh, that's a bit of
    overzealousness". This supersedes an offline-only constraint for required
    build and test gates; it does not authorize version changes, extra
    dependencies, or scope beyond S1.0–S1.1 and the approved ICU4X edge.
13. **Differently styled inline boundary ownership.** For line-break,
    word-break and overflow-wrap at a boundary between differently styled
    inline boxes, Mark ruled: "Nearest common ancestor (Recommended)". This
    settles the ownership context that CSS Text leaves undefined; collapsing-
    space-generated opportunities continue to use the directly containing
    box's properties as CSS Text specifies.
14. **Atomic opportunities under `line-break:anywhere`.** Mark answered: "Allowed. Proceed", approving the recommended interpretation that `line-break:anywhere` overrides GL, WJ and ZWJ restrictions on atomic-inline break opportunities. Apply the override only after resolving the boundary's ruling-13 common-owner style and passing its wrapping/nowrap gate; explicit forced breaks retain their existing path and classification. The ordinary NBSP compatibility exception remains unchanged. This is still S1.1 and does not authorize S1.2.

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
- **T4, `text-overflow: ellipsis`** (added 2026-10-02, ruling 8; it follows
  S1.3).
  - **The gap.** A line that overflows its block under `overflow: hidden` and
    `text-overflow: ellipsis` ends in an ellipsis glyph rather than being
    clipped mid-glyph. Livery has no `text-overflow` code (no match in
    `components/livery/src` or `components/genet-livery/src`, 2026-10-02).
  - **Consumer.** Knot's Navigator status message, where Mark ruled the gap
    be reported to Genet's roadmap
    (`knot-editor/design_docs/2026-09-23_knot_workspace_slice1_plan.md`,
    the 2026-09-26 chip-squeeze entry). Cambium's doc says the ellipsis
    waits on Genet (mere `518ca1dc`).
  - **Why it waits for S1.3.** The ellipsis is placed at the line end, which
    Livery owns only once S1.3 does placement itself. Its WPT files are
    outside the 14-directory gate set: 51 `text-overflow*` files in
    `css/css-ui` and 19 in `css/css-overflow` (counted 2026-10-02). They are
    measured when T4's brief is written.

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

**Landed 2026-10-03:** baseline expectations and the 14 guard entries are
committed at the S1.0 starting source. The locked release runner was built
with `cargo build --manifest-path Cargo.toml --release --locked -p genet-wpt
-j 2`; binary SHA-256 is
`7345f0705cf23264e76dd664d1206211a86df5dda7e993129fe5adec3f06095e` and the
WPT manifest SHA-256 is
`d5ec5be9bf1a75ed00d7e7ab28afe8a694a55e11682ba74305874d70b18dd422`. Two
same-binary runs used Livery, jobs=1 and timeout=30 defaults. Each measured
989 pass, 754 fail, 551 skip, 0 timeout, 0 other across 2,294 tests; per-file
status differences were 0. All 14 directory counts match the table above.
The full baseline guard ended with `unexpected=0`. Detailed maps, native logs,
hashes and the comparison receipt are under
`C:/Users/mark_/Code/testing/genet-line-breaking/`.

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
- **Candidate source.** Use the existing workspace ICU4X line segmenter
  (`icu_segmenter` 2.2.0) to enumerate candidate byte boundaries over the exact
  transformed and collapsed paragraph text Parley shaped. Keep UTF-8 byte
  offsets and source mapping explicit, then intersect candidates with shaped
  cluster and atomic-inline constraints. Parley's `is_space_or_nbsp` is not a
  generic breakable-whitespace test: NBSP is not a normal whitespace
  opportunity.
- **Comparison meaning.** Check that Parley's selected finite-width breaks
  remain permitted by the candidate opportunity model where today's output is
  correct. The full candidate set is not expected to equal selected breaks.

**Done when:** a unit fixture per rule passes, including T1's and T3's
boundaries. On the existing `css_text_lane` and `line_box_model` fixtures,
Parley's selected breaks are permitted by the candidate opportunity model
wherever today's output is correct, with a fault-sensitive control.

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

*Ruled 2026-10-02 (ruling 5):* option 1. S1.4 runs `unicode-bidi` over the
paragraph text with Parley's base level. Its done-conditions gain one more:
a fixture that compares each run's computed level parity with `Run::is_rtl`
and fails on any disagreement.

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

### 2026-10-03 S1.1 boundary implementation

- CSS Text 3 §5.5 leaves the `line-break`, `word-break`, and `overflow-wrap`
  ownership context undefined at character and atomic-inline boundaries, but
  assigns `white-space` to the nearest common ancestor. Ruling 13 supplies
  that missing ownership choice for the three undefined properties. At
  opportunities created by disappearing spaces, the directly containing
  box still supplies those properties as §5.5 specifies.
- CSS Text 3 §5.5 creates opportunities on both sides of atomic inlines,
  including where ordinary text rules suppress a break. S1.1 consults ICU4X
  `Line_Break` property data for adjacent GL, WJ, and ZWJ in the ordinary
  wrapping context, with NBSP compatibility and punctuation cases covered.
  Ruling 14 resolves the §5.2/§5.5 interaction: `line-break: anywhere` permits these atomic opportunities after common-owner style resolution and the wrapping/nowrap gate; forced breaks and the ordinary NBSP exception are unchanged. The U+FFFC shadow feeds ICU's surrounding text-candidate
  context, with offsets mapped back to the original paragraph. Atomic
  opportunities carry regular/emergency kind separately
  (`components/genet-livery/src/text.rs`, `logical_break_stream` at :3104 and
  `atomic_side_opportunity` at :3419).
- Synthetic inline edge and empty-line boxes are not logical neighbors. Owner
  lookup and selected-break classification skip them while preserving the
  insertion order of actual atomic inlines (`atom_side_style` at :3752 and
  `selected_break_position` at :3363). Collapsing-space opportunities keep
  their directly containing source's style through
  `disappearing_space_owner_style` at :3723.
- The unbounded public projection supplies the complete shaped clusters, but
  its soft-break flags cannot describe the later finite break choice. After
  Parley's existing finite breaker runs, S1.1 refreshes only each cluster's
  soft/hard flags from the actual finite-layout cluster with the identical
  UTF-8 range. A test-only receipt compares every retained range and flag
  against those finite Parley clusters; glyphs, fonts, source slices, and
  coordinates remain from the original shaped stream.
- The `wrapped-atom` probe mirrors `line_box_model.rs`'s Ahem 16px/20px,
  160px block, 80×40 atom, exact source text, and registers the same vendored
  `tests/wpt/tests/fonts/Ahem.ttf` bytes (SHA-256
  `b719ecb31c5b21fc573c03f6421c74ac63c271a5a3ff841e34f9705fb94b8448`). The
  `break-all` probe mirrors `css_text_lane.rs`'s 16px/20px sans-serif,
  100px width, and exact word.

## Progress

- 2026-10-02: Stage 0 landed as `6fca091dc26` ("livery: non-wrapping lines
  follow text-align"). `css_text_lane::text_align_places_non_wrapping_lines`
  fails without it and passes with it; text-align WPT went from 50 to 53
  passing. The B and C prototypes stay on throwaway branches
  (`proto/nowrap-parley-b` `7c85004ebd8`, `proto/nowrap-text-align`
  `84701e00dd3`). Plan written.
- 2026-10-02: rulings 5 and 6 recorded: an own UAX#9 pass for S1.4, and
  S1.0–S1.1 briefed to a separate agent. While writing the brief, it was
  found that the prototype runner `testing/genet-gaps/wpt/run_wpt.sh`
  hard-codes `REPO=C:/Users/mark_/Code/worktrees/genet-gaps`. That worktree
  was retired the same day, so a rerun must point `REPO` at its own checkout.
  The maps and binaries under `testing/genet-gaps/wpt/` are unaffected.
- 2026-10-03: rulings 10–12 record model preference, the approved existing
  ICU4X 2.2.0 candidate-boundary source, and network access for required gates.
  S1.0 baseline expectations and guard entries are committed; two runs of the
  same release binary reproduced all 14 directory counts with no per-file
  status changes, and the full guard passed with `unexpected=0` (receipt above).
- 2026-10-03: ruling 13 assigns the nearest common ancestor as the ownership
  context for line-break, word-break, and overflow-wrap at differently styled
  inline boundaries. S1.1 now uses the atomic-marker shadow for surrounding
  text candidates, ICU4X line-break classes for atomic blockers, stable real
  neighboring owners, and separate normal/emergency atom opportunities.
- 2026-10-03: focused S1.1 lib suite passes 24/24. Temporary source fault
  controls independently made the NCA ownership fixture fail when ownership
  was forced to the root, the GL blocker fixture fail when ICU's Glue class
  was omitted, and the synthetic-edge fixture fail when non-atomic inline
  boxes were treated as real neighbors. Each mutation was restored. Chrome
  154.0.8037.95 permits the probed atomic boundaries beside GL, WJ, and ZWJ
  under ordinary wrapping too, so this measurement does not distinguish
  `line-break: anywhere` precedence. The raw result is
  `testing/genet-line-breaking/atomic-anywhere-probe.chromium.json`. At that
  point the checkpoint remained open; the user later resolved it in Ruling 14.
- 2026-10-03: correct-today fixture roster review found that the current
  stream comparison mirrored only two cases. The compact in-source comparison
  now covers all seven unbreakable-word styles, CJK keep-all vs normal, the
  nowrap inline's owned words and outside separator, pre-line/hard breaks,
  preserved tabs, hanging ideographic-space overflow, manual vs removed soft
  hyphens after the existing text transform, and the exact Ahem wrapped atom.
  Line-box cases that measure only height or vertical alignment do not exercise
  candidate selection; automatic hyphen insertion is outside S1.1.
- 2026-10-03: after updating soft/hard flags from matching finite Parley
  clusters, the expanded stream and context tests pass 25/25. Three more
  temporary source faults failed as intended when ordinary candidates,
  emergency candidates, or finite break flags were suppressed; NCA, GL class,
  and synthetic-neighbor controls also fail under their isolated faults. The
  control names and restored source hash are in
  `testing/genet-line-breaking/s1.1-control-runs.json`; the fixture/control
  list is `testing/genet-line-breaking/s1.1-fixture-roster.md`. Full final
  crate gates and WPT comparison remain open.

- 2026-10-03: CSS Text 3 §5.5 also requires soft-wrap assumptions between typographic letter units when required lexical/orthographic analysis is unavailable. The ICU `ComplexContext` (`SA`) path therefore adds ordinary candidates at boundaries between adjacent SA letter units, intersected with grapheme and shaping-cluster edges; emergency policy is not needed. Khmer and Thai normal-policy fixtures cover that fallback, including Khmer `U+1780 U+0301 U+1781`, where the boundary is byte offset 5 rather than inside the first grapheme at offset 3. A fault disabling the fallback fails its focused assertion. This follows CSS's explicit fallback where ICU's non-complex-script segmenter supplies no lexical analysis.
- 2026-10-03: the final fault-control roster now has separate failing invocations for the requested per-fixture rules, including T1's actual shaped Ahem case, each WPT031/032 mirror, each word-family variant, CJK normal/keep-all, space ownership, pre-line and forced breaks, tabs/hanging metadata, manual/removed SHY, finite soft/hard flags, RTL and atom mapping, ICU GL/WJ/ZWJ/NBSP behavior, synthetic edges, adjacent atoms, and SA fallback. The exact roster and per-run receipt is `testing/genet-line-breaking/s1.1-fixture-roster.md` and `C:/Users/mark_/Code/testing/genet-line-breaking/s1.1-control-runs.json`. The only fault invocation that passed intentionally targeted the internal `nowrap` rule at a collapsed-space opportunity, whose directly containing box is the specified owner; it is recorded as a negative control, not counted as fault evidence.
- 2026-10-03 (historical, pre-R14 finalization): after source restoration and removal of test-only selectors, `genet-livery --lib` passed 315 tests on source SHA-256 `CCD2FD7704559E99921A4CA648FE1FE6EC611CD113A61182A04551A80E02A46F`. This provisional receipt was superseded by the final R14 source and gates below.

### 2026-10-03 S1.1 CSS fallback and final-source receipts

- CSS Text 3 §5.5 requires a soft-wrap opportunity between typographic letter units when the user agent lacks the requisite lexical/orthographic analysis ([CSS Text Level 3, line breaking details](https://www.w3.org/TR/css-text-3/#line-break-details)). The implementation keeps ICU's non-complex-script segmenter and adds `complex_context_fallback_boundaries` at `components/genet-livery/src/text.rs:3587`, using ICU SA letter-unit classification, then intersects those byte boundaries with grapheme and shaped-cluster edges. It therefore handles Khmer/Thai fallback as regular opportunities without adding a dictionary or emergency breaks. The combining-mark Khmer fixture verifies boundary byte 5 and rejects an interior boundary at byte 3.
- Finite Parley soft/hard flags are refreshed by exact UTF-8 range after the existing breaker at `components/genet-livery/src/text.rs:1294`. The check compares retained flags with every matching finite-layout cluster; glyph/font/source/coordinate data and chosen line layout remain unchanged. Fault controls independently remove soft and hard flags.
- Historical pre-R14 receipt: source `CCD2FD7704559E99921A4CA648FE1FE6EC611CD113A61182A04551A80E02A46F` passed `genet-livery --lib` 315/0/0 and the pre-R14 release guard. This does not replace the later ruling-14 receipts.
- 2026-10-03 (pre-R14): the release `genet-wpt` build on the provisional frozen S1.1 source passed, and the 16-entry WPT guard completed with `unexpected=0`. The 14 baseline directories total 989 pass, 754 fail, 551 skip, matching both S1.0 binary runs; `mediaqueries` and `css-position` are the two pre-existing extra entries. Exact frozen binary SHA-256 is `5C3F29D584EFD4A3B4285FD8CB341E076A6151C11253BE704CEC87637A4642C9` (53,964,288 bytes). Inputs and raw results are `s1.1-parent-wpt-guard-inputs.json`, `s1.1-parent-wpt-guard.log`, and `s1.1-parent-wpt-counts.json` under `testing/genet-line-breaking/`. Root also verified primary/lane lockfiles contain the same 867 unique package/version pairs, with no package/version change; the only new dependency edges are the approved direct ICU4X edges in genet-livery.
- 2026-10-03 (historical, pre-R14 finalization): full package gates passed on the provisional source: genet-livery 653 passed/0 failed/6 ignored across 47 test targets; livery 210/0/4 across 15; buckram 271/0/0 across 2. Its focused library suite was 315/0/0. The pre-expansion genet-livery receipt of 647/0/6 is explicitly not a base-source measurement. These provisional counts are superseded by the final R14 gates below.

- 2026-10-03 (historical, before final release guard): ruling 14 was recorded verbatim: "Allowed. Proceed". `line-break:anywhere` overrides GL/WJ/ZWJ atomic-inline blockers after common-owner style resolution and the `nowrap` gate; forced-break classification and NBSP normal compatibility remain unchanged. The dedicated common-owner fixture tests GL on both atom sides and confirms nowrap still blocks; the composite blocker fixture tests both sides for U+2007, U+2060 and U+200D, checking that Anywhere makes each Regular. Thirteen focused atomic tests pass; removing only the override makes the dedicated common-owner fixture fail (GL boundary `None` versus `Some(Regular)`). Fault and restoration hashes are in `s1.1-ruling14-control.json`; original ruling13-era receipts remain separate.
- 2026-10-03: ruling-14 source SHA-256 `A223B68FE8C9E53A3C7E3CDE7FB75173ED4128D282675AA95C25AEF5E932D26B` passes `genet-livery --lib` 316/0/0 and all full crate gates: genet-livery 654/0/6 across 47 targets; livery 210/0/4 across 15; buckram 271/0/0 across 2. Rustfmt and diff checks pass. A genuine override-removal fault fails the dedicated NCA GL fixture and restores exactly. Root has the frozen manifest/source hashes and is rebuilding release `genet-wpt` before the final 16-entry guard; no commit until those receipts are accepted.
- 2026-10-03: R14 release and 16-entry guard passed on source `A223B68FE8C9E53A3C7E3CDE7FB75173ED4128D282675AA95C25AEF5E932D26B`. All entries report `unexpected=0`, errors 0. The 14 S1.0 baseline directories remain at 989/754/551; totals across all 16 entries are 1,052 pass, 865 fail and 814 skip. The release binary is `82C4A8816A800A9396BE166DFA25824D5431EA682AB7BCF1C8D7F08DC9B1F0AF`. All three final crate gates passed: genet-livery 654/0/6, livery 210/0/4, buckram 271/0/0. S1.0–S1.1 are complete on the lane branch and await main integration. Full provenance and the per-entry table are in `testing/genet-line-breaking/final-checkpoint.md`.
