# Line box model: CSS 2.1 §10.8 in Livery's line pass

**Status:** in progress, 2026-09-25. Mark ruled the home the same day:
genet-livery's line pass, with Parley kept upstream-shaped rather than
patched. Slice A (the model with nested alignment, the line height quirk,
and the scripted tier's quirks mode) is implemented and receipted; slice B
(atom baselines) is next.

## The gap

Knot's step 4c headed check found a tall inline-block sitting half a leading
too high. At genet `5621ca05768`, with `body { font-size: 16px; line-height:
20px }`, a 300px paragraph that wraps onto a second line holding a
`display: inline-block; height: 40px; width: 80px` span places the span at
y = 18. Line 1 occupies 0..20, so the atom overlaps it by 2px; it should sit
at y = 20. A button aligned on its own text baseline overlaps by more; Knot's
"Refresh comparison" button covered the line above.

Instrumenting Parley's line metrics for that paragraph shows why. For the
atom's line Parley records `line_height = 40` (the box), `ascent = 40` (the
box sits on the baseline) and `descent = 3.39` (the text), so its leading is
`40 - 43.39 = -3.39`. Parley splits that leading about the baseline, which
pulls the baseline up by half of it, and clamps only the line's top
coordinate at zero leading: the line's top lands at 18. The next line starts
at Parley's 60 while genet's corrected line 2 ends at 61, so it overlaps too.

## What CSS requires

CSS 2.1 §10.8 and §10.8.1 build the line box from the boxes on it rather than
from one line height:

- Every line box starts with a strut: a zero-width inline box with the block
  container's font and `line-height`.
- A non-replaced inline box's layout extent is its font's ascent A and
  descent D plus half the leading on each side, where the leading is
  `line-height - (A + D)` for that box's own font and `line-height`. It may be
  negative, and then the box is shorter than its glyphs.
- An atomic inline (replaced, inline-block, inline-table) contributes its
  margin box, positioned by its baseline: an inline-block's last line box, or
  its bottom margin edge when it has none.
- `vertical-align` places each box relative to its parent's baseline
  (`baseline`, `sub`, `super`, `text-top`, `text-bottom`, `middle`, lengths
  and percentages). `top` and `bottom` align to the line box itself, after
  the rest are placed, and can grow it.
- The line box's height is the distance from the uppermost box top to the
  lowermost box bottom, and its baseline follows from that. Line boxes stack:
  each starts where the previous one ends.
- §9.4.2's empty-line rule stands as landed in `5621ca05768`: a line with no
  content is zero-height unless it ends in a preserved newline or a forced
  break.
- In quirks and limited-quirks mode the Quirks Mode Standard's line height
  calculation quirk applies: a box with no text on the line counts as if its
  `line-height` were zero. Chromium applies it per box (see Findings), and
  genet follows Chromium.

For the gap's line that gives: text reaches about 15.5 above the baseline
and 4.5 below; the atom reaches 40 above. The line box's top is the atom's top
at y = 20, and it is 40 plus the text's space below, 45 once ascent and
descent are rounded as the ground truth shows, so the next line starts at 65.

## What genet did before the model

`text.rs` laid a line out with Parley, then corrected Parley's single-height
metrics after the fact: a requested line height taken from explicit
`line-height`s and in-flow atoms (across the whole formatting context, not
per line), extra leading split about the line, `text-top`/`text-bottom` edge
leading, an inline table's exported baseline, a shift for empty inline boxes,
and the break-line extents that `5621ca05768` added. `vertical_align_shift`
then moved items after the line's metrics were fixed, and
`format_inline_group` overrode the measured height for a zero `line-height`
strut and for an empty inline box. Two structural consequences followed:

- A line's top came from Parley's `block_min_coord`, so a line whose height
  genet corrected did not move the lines after it.
- The corrections saw glyph runs and atoms, not inline boxes: an outer span
  whose own `line-height` exceeds its text's contributed nothing, and a
  larger font's own half-leading was never computed.

## Ground truth

Chromium 152 against genet `663f86251c4` (before) and slice A, Arial
16px/20px in standards mode, each case a 300px block (fixtures in
`Code/testing/genet/line_box_model/`). Heights are the block's; atom tops are
relative to it. genet's heights are whole pixels because its block layout
rounds.

| Case | Chromium | genet before | slice A |
|---|---|---|---|
| Knot's repro, three lines | 85, atom at 20 | 100, atom at 18 | 85, atom at 20 |
| an inline-block on one line with text | 45, atom at 0 | 43, atom at -2 | 45, atom at 0 |
| `vertical-align: super` / `sub` / `10px` span | 26.33 / 24.19 / 30 | 26 / 24 / 30 | 26 / 24 / 30 |
| a 32px span inside the 20px line height | 26 | 20 | 26 |
| an empty span with `line-height: 40px` | 40 | 40 | 40 |
| a `line-height: 40px` span around a 10px one | 40 | 20 | 40 |
| an inline-block at `top` / `bottom` / `middle` | 40 / 40 / 40, at 0 | 43 / 46 / 71 | 40 / 40 / 40, at 0 |
| an inline-block at `text-top` / `text-bottom` | 41 at 1 / 42 at 0 | 43 at -2 / 46 at 1 | 41 at 1 / 42 at 0 |

The durable test, `components/genet-livery/tests/line_box_model.rs`, renders
the same shapes in Ahem so it does not depend on a system font: twenty-two
standards-mode rows, five of them nested alignment, and twenty-one
quirks-mode rows, each against Chromium's Ahem value measured the same day.
Re-measured at a device pixel ratio of 2, Chromium gives every row the value
in the test; the rows first measured at 1 did not move.

## Design

In the line loop of `TextSystem::shape`, per line:

1. Collect the boxes on the line: the strut from the root style; one extent
   per inline box on the line, from its font's metrics
   (`TextSystem::box_metrics`, probed once per font) and used `line-height`;
   one per atomic inline from its margin box and baseline (an inline table
   exports its first row's). A box is on the line when an item it owns is:
   a glyph run, an atom, or one of its own edges; or when any of its text is
   in the line's text range, which is how a box whose only content on the
   line is a preserved newline or forced break is seen, since Parley gives
   such a line no item. A forced break carries a source span for this.
   Ascents and descents are rounded as the Findings describe.
2. Place each box by `vertical-align` relative to its parent box's baseline,
   holding back each `top` and `bottom` box with its aligned subtree.
3. Take the union of the placed extents as the line's space above and below
   the baseline, then place the held-back boxes against it, growing it if one
   is taller.
4. The line box's height is above plus below. Its top is where Parley started
   the line plus the drift the model's heights have added so far, so float
   placement Parley did stays; its baseline is its top plus above.
5. Emit every item at its CSS position: glyph runs translated from Parley's
   baseline to their own, atoms at their baseline minus their baseline
   offset. Every item's line fragment is the line box, which is what the
   formatting context's height is the union of.

In quirks and limited-quirks mode an inline box counts only where it holds
text or a forced break directly, or has inline-axis borders or padding; the
strut counts only where the root holds text, or a forced break on a line
where nothing else counts; atoms always count; a list item's own lines keep
strict line height. The document mode comes from `LayoutDom::quirks_mode`.

The corrections the model subsumes are deleted rather than layered on: the
requested line height, edge leading, the table-baseline line special case,
the empty-line shift, `vertical_align_shift`, and in `format_inline_group` the
zero-`line-height` strut probe and the empty-line height override. Parley
still breaks lines and shapes glyphs; only the vertical placement changed.

`vertical-align` is taken against the parent box: `sub` and `super` from the
parent's font size, `text-top`, `text-bottom` and `middle` from the parent's
font, and the raises chain from box to box, so a `sub` inside a `super` sits
where the two add up to. A `top` or `bottom` box heads its aligned subtree
(itself and the descendants not themselves `top` or `bottom`), which is
placed as one against the line box once the root's subtree is (CSS 2.1
10.8.1). An atom aligns within its innermost inline box the same way.

## Done-conditions

- Knot's repro: the atom sits at y = 20, its line is 45 tall, the next line
  starts where it ends, and no line in the genet-livery suite overlaps the one
  before it.
- Every row of the ground-truth table matches Chromium, within 0.5px where
  Chromium's value is fractional; the table becomes a genet-livery test, with
  the quirks-mode rows beside it.
- A document's quirks mode reaches layout and `document.compatMode` on both
  tiers: the static DOM and the scripted arena, including a `DOMParser`
  document.
- An inline table still exports its first row's baseline, per the existing
  tests.
- genet-livery, buckram and the Genet workspace suites keep their counts,
  except tests whose expectations encode the old model, each named and
  justified in the progress entry.
- WPT before and after, testharness and reftest, over `css/CSS2/linebox`,
  `css/CSS2/visudet`, `css/css-inline`, `css/CSS2/normal-flow`,
  `css/css-text/line-breaking`, `html/rendering/non-replaced-elements` and
  `quirks`, and testharness over `dom/nodes` and document.open's input-stream
  tests, with every `pass -> anything else` attributed; the checked baselines
  run through both binaries; and a positive control that flips.
- The Knot session is told, so its inline-block workaround can come out.

## Findings

All 2026-09-25.

- **Parley's line metrics** (`support/patches/parley/src/layout/line_break.rs`,
  `finish_line`). A line has one `line_height`, the running maximum of its
  items' heights, and one ascent and descent; an in-flow inline box counts as
  all ascent. Quantized, the baseline is `round(y) + round(A) +
  floor(leading / 2)` and the next line starts at `y + line_height`. The model
  recovers Parley's start as the inverse, `parley_line_top`.
- **`line-height: normal` is whole pixels in Chromium:** ascent, descent and
  line gap each round before they add, so 16px Arial is 18, not 18.4. With
  the unrounded 18.4 an image alone on its line landed at a fractional offset
  and antialiased against its reference; with the rounding the second WPT
  pass gained twelve `normal-flow` files over the first.
- **Chromium's rounding.** Ascent and descent round to whole pixels, then
  `above = floor(leading / 2)` and `below = leading - above`, so an odd
  leading's larger half goes below and a fractional leading's fraction stays
  in the line (two lines at `line-height: 19.2px` measure 38.40625 in
  Chromium, LayoutUnit precision). Parley rounds `below` as well,
  which makes its line box whole while it still advances by the fraction; the
  model follows Chromium.
- **`sub` and `super` are constants in Chromium,** not font metrics: the
  parent's font size / 5 + 1 down and / 3 + 1 up, at LayoutUnit precision
  (24.1875 and 26.328125 for 16px/20px). By Mark's ruling genet uses the same
  offsets, replacing its 0.2em and 0.4em.
- **The quirk, as Chromium applies it,** measured in Chromium 152 quirks mode
  with Arial 16px/20px:

  | Case | Height |
  |---|---|
  | a 10px span alone / after root text | 20 / 22 |
  | a 40px `line-height` span around a 10px one, alone / after text | 10 / 20 |
  | an empty 40px span after text | 20 |
  | an inline-block inside a 40px span | 12 |
  | the same span with `padding-top: 1px` | 12 |
  | an empty 40px span with `padding-left: 5px` / `padding-top: 1px` | 40 / 0 |
  | `<br>` / `<br>` inside a 40px span | 20 / 40 |
  | an inline-block, then `<br>` / then a `<br>` inside a 40px span | 12 / 40 |
  | an empty 40px span with `margin: 0 5px` | 0 |
  | a 40px atom alone at `middle` / `text-top` | 40 / 40 |

  So a box counts where it holds text or a forced break directly, or has
  inline-axis borders or padding (Blink's `IsEmptyItem`); margins and
  block-axis padding do not make it count, whatever the Quirks Mode
  Standard's wording suggests. The strut counts where the root holds text,
  or where a forced break sits straight in the root on a line nothing else
  counts on. A list item's own lines are exempt: list items force strict
  line height (whatwg/quirks#38; WPT's `quirks/line-height-in-list-item.html`).
  Chromium passes all twenty cases of WPT's
  `quirks/line-height-calculation.html` this way, in quirks, limited-quirks
  and standards mode.
- **Parley gives a line that ends in a forced break or preserved newline no
  items,** but its `text_range` covers the newline. An atom-only line's range
  is inverted (`usize::MAX..0`), which the span walk skips.
- **A table cell reports its bottom edge as its baseline** in the live route:
  the cell formatter (`layout.rs`, `TableCellLayoutOutput`) reads the cell's
  baselines before `populate_inline_baselines` has fed the text's in. The
  `b3_inline_table_uses_its_first_table_baseline` test records the behavior.
  In standards mode an inline table alone on a line is then 48 tall where
  Chromium gives 37. By Mark's ruling this is slice B's.
- **The scripted arena had no quirks mode.** The parser recorded html5ever's
  decision in `ParserPolicy` and nothing read it
  (`2026-09-08_parser_script_interleaving_plan.md`, residual 6;
  `2026-09-07_dom_node_model_plan.md`, residual 6), `compatMode` was the
  constant `'CSS1Compat'`, and `ScriptedDom` reported `NoQuirks` to layout.
  By Mark's ruling slice A wires it end to end: a per-document side table in
  the arena, set by the parser sink and by `clone_into` and
  `from_serialized_document` from their source document, read by layout
  through `LayoutDom::quirks_mode` (`ScopedDom` asks for its own document's)
  and by script through `__documentCompatMode`.

## Next

- **Slice B, atom baselines.** An inline-block exports its last in-flow line
  box's baseline (overflow visible), and a table cell its first line's, so an
  inline table's first-row baseline comes from content. Chromium's rows:
  `text <inline-block, padding 8px, border 1px>in</inline-block> text` is 38
  with the atom at 0 (genet: 43); a two-line inline-block, 40 (45); a padded
  button, 38 (43); an `overflow: hidden` inline-block keeps its bottom edge,
  41 (41).
- **Device pixels.** At a device pixel ratio of 2 Chromium gives a 16px
  Arial `normal` line 18.5 and the Arial atom line 44.5, where at 1 it gives
  18 and 45: for a system font its rounding follows the device pixel. genet
  rounds in CSS pixels, which is Chromium at ratio 1 and every ground-truth
  row; following the device pixel needs the text system to know the ratio.
- **Floats** place lines by Parley's line heights; a line the model makes
  taller keeps Parley's float constraints.
- **Quirks mode's collapsed whitespace:** a box whose only text on a line is
  a hanging or collapsible trailing space still counts there.

## Risks

- Text lines can move by sub-pixel amounts where the old corrections and the
  model round differently; the WPT receipt attributes each transition rather
  than tolerating them.
- The line pass feeds painting, hit testing, carets and selection through the
  same fragments. Their suites (genet-render, genet-documents, genet-scripted,
  taproot) run with genet-livery's.

## Progress

- **2026-09-25, slice A.** The model replaces the line loop in
  `components/genet-livery/src/text.rs`, with nested alignment, the quirk,
  and the scripted tier's document mode (`genet-scripted-dom`,
  `genet-scripted`, `script-runtime-api`). Every Arial row above matches
  Chromium, and `tests/line_box_model.rs` holds 43 Ahem rows in both modes.
  genet-livery, genet-scripted-dom, script-runtime-api, genet-scripted,
  genet-documents, genet-render, taproot and buckram: 1710 passed, none
  failing. Lines cannot overlap: each starts where the one before it ends,
  unless Parley moved it down past a float. One test changed:
  `inline_replaced_image_uses_the_shaped_line_fragment` (`tests/paint.rs`)
  gains a doctype. Its page was a quirks document, where Chromium gives a
  line holding only the image no strut and so no room for `vertical-align:
  20px` to move it (0 and 0); in standards mode Chromium gives 26 and 6, which
  is what the test asserts and what slice A produces.

  WPT receipt: `Code/testing/genet/wpt-ledger/2026-09-25_line_box_model/`.
  Testharness is unchanged over all nine directories. Reftests: 43
  `fail -> pass`, twenty of them in `css/CSS2/linebox`, and 6
  `pass -> fail`, each attributed. Three wait on slice B's inline-block
  baseline (`inline-box-border-line-break` and `-padding-line-break` through
  their shared reference, `line-breaking-atomic-007`). Three were false
  passes that now fail for reasons outside this plan: WOFF1 web fonts are
  not decoded (`visudet/line-height-203`), the outline of an empty inline box
  is not painted (`css-inline/empty-span-size-002`), and genet has no
  fieldset or legend layout (`fieldset-border-gap-negative-margin`). The
  first attempt had nine regressions; four were real, from three causes,
  and led to rounding `normal`, exempting list items, and building the
  nested alignment in this slice rather than the next (Mark's ruling). The
  positive control flips.
  The checked baselines report the same unexpected entries before and
  after, save two `css/css-position` reftests that now pass
  (`position-absolute-in-inline-003` and `-margin-top`), repinned in
  `ports/genet-wpt/expectations/reftest/css_position_boa.json`.
