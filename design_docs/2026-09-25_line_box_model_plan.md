# Line box model: CSS 2.1 §10.8 in Livery's line pass

**Status:** in progress, 2026-09-27. Mark ruled the home on 2026-09-25:
genet-livery's line pass, with Parley kept upstream-shaped rather than
patched. Slice A (the model with nested alignment, the line height quirk,
and the scripted tier's quirks mode) and slice B (atom baselines) are
implemented and receipted. Since 2026-09-27, under `line-height: normal`,
each text run paints over its box's content area inside its line box. The
items under Next remain. Ruling 379 (2026-09-27) authorizes the explicit-line-height and wrapped inline-edge bounds fixes; the combined implementation and CPU verification are complete, with independent review and consumer acceptance still pending.

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
   formatting context's height is the union of. A glyph run's painted
   fragment, which its text node and the inline boxes around it report and
   paint, is under `line-height: normal` its box's content area: the
   rounded ascent and descent the box's extent was built from, about the
   run's baseline. Under an explicit `line-height` it is still the line
   box's height at the run's baseline offset, or Parley's extent where that
   is taller (see Next).

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

All 2026-09-25, except where an entry gives its own date.

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
  Chromium gives 37. By Mark's ruling this is slice B's, which fixed it.
- **A table's own baselines did not outlive the next layout run.** Every
  `compute_layout_with_measure` ends in `AlgorithmTree::propagate_baselines`
  (`components/buckram/src/taffy_adapter.rs`), which gave every node a
  baseline synthesized at its block end and then re-chose each parent's from
  its children, a `Table` node's included. The K4d5 baselines that
  `commit_table_block` (`components/genet-livery/src/table_block.rs`) writes
  on the grid node therefore lasted only until the next run anywhere in the
  tree, such as the next cell's formatting: a table nested in a cell reported
  its bottom edge. Invisible while every cell reported its bottom edge too;
  with slice B's text baselines a row holding a nested table grew by 11px
  (`inferred_cells_with_nested_tables_match_html_table_glyphs`). A `Table`
  node now keeps its declared baselines through a run, as it keeps the size
  it was given.
- **K4d5 synthesized a row's baseline from measured content.** With no
  baseline-aligned cell taking part, `align_table_cells`
  (`components/buckram/src/table/rows.rs`) took the lowest cell content edge
  from each cell's measured content. genet hands a cell's specified height
  to Buckram as a row constraint rather than content, so an empty 40px cell
  measured 0, and an inline table holding one hung from the baseline. It
  now takes each cell's box as filling its row. The previous finding hid
  this: the inline table read its grid's bottom edge, which is right for one
  row with no bottom padding.
- **genet's cell formatter gave an empty cell a baseline** at its measured
  content's end, so an empty baseline-aligned cell took part in its row's
  baseline. Chromium leaves it out, as K4d5's own unit test intends
  (`a_row_without_baseline_cells_synthesizes_from_the_lowest_content_edge`).
  It now reports none.
- **livery does not parse `display: inline-flex` or `inline-grid`:** the
  `Display` keywords (`components/livery/src/values/property/animation.rs`)
  stop at `inline-table`, so the declaration is dropped and the element keeps
  its UA display. Chromium aligns both on their first baseline (a column of
  two flex items and a grid of two rows alike).
- **An inline-block holding only an atom is too short.** With a 40px
  inline-block alone inside it, Chromium makes the outer inline-block 45 tall
  (its line box puts the strut's descent under the atom) and genet 40. The
  line around it agrees (45), because the outer box's baseline comes from
  that inner line; its height does not. Not traced yet.
- **genet's UA sheet lacks some of Chromium's control edges.** Chromium's
  default button has `padding: 1px 6px` and a 2px outset border, and its text
  input `padding: 1px 2px` and a 2px inset border. genet's sheet (the UA
  string in `components/genet-livery/src/lib.rs`) gives the button that
  padding but no border and the input neither, so with the ground-truth font
  a two-line default button is 42 tall where Chromium's is 46, and a default
  input 20 where Chromium's is 26. The ground-truth rows set both explicitly,
  so only the baseline rules differ.
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
- **The text fragment kept Parley's extent** (2026-09-27; Isometry's wing
  design record, ruling 329). Slice A built each line box from rounded
  metrics but left each glyph run's painted fragment (`ShapedRun::fragment`
  in `text.rs`) at `line_height.max(content_height)`, where `content_height`
  is Parley's quantized extent for the line. Parley rounds ascent and
  descent but takes the leading from the unrounded line height and splits it
  floor above and round below (Parley's line metrics, above). Under
  `line-height: normal` its extent therefore carries the rounding of ascent
  and descent, while the line box rounds the line gap on its own. At 11, 12
  and 13px Arial the fragment was 13, 15 and 16 in line boxes of 12, 14 and
  15. Lato (`tests/wpt/tests/fonts/Lato-Medium.ttf`) overhung at 14 of the 33
  sizes from 8 to 40px. The fragment also took the whole line box's height at
  the offset of its own baseline. So text beside a taller box reported the
  line box (16px text beside a 32px span: 37 at 0), and raised text left it
  (a 13px `super` span's: 20.33 at -5.33). Before slice A, with no requested
  line height, the row and the fragment were both Parley's
  `line_height.max(content_height)` (`5ae30ca`), one number by construction.
  Isometry's side panel receipt
  (`crates/isometry-genet/src/host_zoom/text_rows.rs`) found 39 of its 187
  text rows, across three panel states, a pixel shorter than their text.
- **Chromium paints text over its content area** (2026-09-27): the font's
  rounded ascent plus rounded descent, about the text's baseline, whatever
  the line height. Measured in Chromium 153 on Windows at a device pixel
  ratio of 1: one 300px Arial block per case, its text read through
  `Range.getBoundingClientRect` over the text node, tops relative to the
  block:

  | Case | Line box | Text fragment |
  |---|---|---|
  | 11 / 12 / 13 / 14px, `normal` | 12 / 14 / 15 / 16 | 12 / 14 / 15 / 16, at 0 |
  | 16 / 18px, `normal` | 18 / 21 | 17 / 20, at 0 |
  | 16px text, then a 32px span, `normal` | 37 | 17 at 15; the span's 36 at 0 |
  | 13px text, then a `super` span, `normal` | 20.33 | 15 at 5.33; the span's 15 at 0 |
  | 16px/20px | 20 | 17 at 1 |
  | 13px/1 / 16px/10px | 13 / 10 | 15 at -1 / 17 at -4 |
  | 18px/1.5 | 27 | 20 at 3 |
  | 16px/20px, a `super` / `sub` span | 26.33 / 24.19 | the span's 17 at 1 / 17 at 5.19 |

  A span's own rect is the same content area: 15 at 0 around 13px text, 17
  at 0 around 16px. Under `normal` the line box is that content area plus the
  rounded line gap, so text stays inside it. Under an explicit `line-height`
  shorter than the content area, text overhangs the line box on both sides,
  half the difference each with the odd pixel above (16px/10px: 4 above, 3
  below). Chromium 153 reproduces the Arial ground-truth table above exactly
  as Chromium 152 measured it.
- **An inline box's edges still span the line box** (2026-09-27). genet
  emits an inline box's borders and padding, and its margins unpainted, as
  separate items (`push_edge`) whose fragment is the whole line box. The
  box's fragment on a line is the union of its painted edges' and its
  text's (`TextFrame::record_inline_fragment`). With the text at its
  content area, a box paints the line box on a line holding one of its
  edges and its content area on its other lines. Painted backgrounds, per
  line, of a span
  wrapping onto three lines in 16px Arial, `normal` (Chromium's through
  `getClientRects`):

  | Span | genet before | genet after | Chromium |
  |---|---|---|---|
  | background only | 18, 18, 18 | 17, 17, 17 | 17, 17, 17 |
  | `border: 1px solid` | 20, 20, 20 | 20, 19, 20 | 19, 19, 19 |
  | `padding-left: 4px` | 18, 18, 18 | 18, 17, 17 | 17, 17, 17 |

- **Wrapped lines sit off whole pixels** (2026-09-27; since slice A). A
  line's top is `parley_line_top` plus `drift` (`TextSystem::shape`).
  `parley_line_top` recovers Parley's quantized start, `round(y)`, but
  `drift` sums the model's heights less Parley's unquantized ones. So a
  line lands `round(y) - y` from where the model's heights put it, up to
  half a pixel, wherever Parley's line height is fractional, as under
  `normal` in most fonts (18.4 for 16px Arial). Glyph baselines of one
  wrapped paragraph, identical before and after the text fragment change:

  | Arial, `normal` | Chromium | genet |
  |---|---|---|
  | 13px | 12, 27 | 12, 27.05 |
  | 16px | 14, 32, 50 | 14, 31.6, 50.2 |
  | 18px | 16, 37, 58 | 16, 37.3, 57.6 |

  Consecutive line boxes therefore overlap or gap by that much. Slice A's
  progress entry says each line starts where the one before it ends. That
  holds only where Parley's line heights are whole pixels, as in Ahem and
  the whole-pixel explicit heights the tests use.

## Slice B: atom baselines

Mark set the scope on 2026-09-25: inline-blocks and buttons, table cells,
atom-only lines, and form controls; the subtree-scoped baseline propagation
lives in buckram.

**What CSS and HTML require, and what Chromium does** (Ahem 16px/20px,
standards mode, device pixel ratio 1; genet after slice A in the last
column):

| Case | Chromium | genet |
|---|---|---|
| an inline-block with padding 8px and a border around text | 38, atom at 0 | 43 |
| a two-line inline-block | 40 | 45 |
| an `overflow: hidden` inline-block (baseline at its bottom edge) | 41 | 41 |
| an inline-block holding only a 40px atom | 45, the inline-block 45 tall | 45, the inline-block 40 tall |
| an empty inline-block (bottom edge) | 25 | 25 |
| a padded button | 38 | 43 |
| an inline table with a padded cell | 36 | 41 |
| a baseline-aligned row of a 16px and a 40px cell | 27, the small cell's text at 9 | 20, at 0 |

- An inline-block's baseline is its last in-flow line box's, or its bottom
  margin edge when it has none or is a scroll container (CSS 2.1 10.8.1). A
  line holding only a forced break or a preserved newline counts: it is no
  phantom (9.4.2; WPT's `visudet/inline-block-baseline-016.html`).
  A button takes its last line's too (WPT's
  `html/rendering/widgets/button-layout/inline-level.html`: "1<br>2" aligns
  on the "2"), and keeps it when it scrolls: Chromium aligns buttons with
  `overflow: hidden`, `visible` and `auto` alike, where an `overflow:
  hidden` inline-block drops to its bottom edge
  (`button-layout/scrollable-button-centering.html`).
  A table inside an inline-block is not one of its line boxes: Chromium
  skips it, so an inline-block holding only a table sits on its bottom edge
  (the line 41, where the row's baseline would give 36), and one holding a
  line and then a table sits on that line.
- A table cell's baseline is its first in-flow line box's or table row's,
  whichever comes first (CSS 2.1 17.5.3): a cell holding only a nested
  table aligns on the table's first row. An inline table exports its first
  row's. CSS 2.1 gives a cell with neither the bottom of its content box;
  Chromium leaves it out of its row's baseline instead, which genet
  follows: beside a text cell, an empty 40px cell leaves the row on the
  text's baseline. A row with no part sits on the bottom
  content edge of its lowest cell, each cell's box filling the row: an
  inline table holding one empty 40px cell sits on 40 (44 with `padding:
  4px 0 6px`), and an 80px inline table of top- and bottom-aligned cells on
  80.
- A table's caption is no line box of what holds the table: a cell and an
  inline table align on the first row below a top caption (Chromium: the
  row 56, its marker at 31), and an inline-block holding a captioned table
  sits on its bottom edge.
- A line holding only atoms still has a baseline; a block container's first
  and last baselines count it.
- Form controls, measured in Chromium with `font: inherit`: a single-line
  text input, an `input` button and a dropdown `select` centre one line box
  of their own font and `line-height` in the content box, and the baseline
  is that line's (18 in a 26px default input, 28 in a 40px-tall one, 25 in a
  40px-tall select). A textarea and a listbox `select` are scroll containers
  and take the bottom edge; a checkbox, radio or range sits on its
  border-box bottom.

**Design.**

1. buckram: `AlgorithmTree::propagate_declared_baselines_within(root)`, the
   same first/last selection as `propagate_declared_baselines` over one
   subtree in post-order.
2. A table cell (`format_table_cell`) feeds its line formatting contexts'
   baselines into the cell's subtree after formatting it and propagates them
   there, so the cell reports its first line's baseline
   (`feed_line_baselines`, which counts a nested table's rows for a cell,
   hides them for an inline-block, and never counts a caption's lines).
3. The line pass records each line's baseline, lines of atoms or of a forced
   break alone included, and `InlineLayout::baselines` reports the first and
   last.
4. Each atomic root, once laid out on its own, exports its baseline through
   the plane inline tables already use: an inline-block its last line's
   unless it scrolls, a button its last line's even when it scrolls, a
   single-line input or dropdown its centred line's (`form_control_baseline`,
   whose `ControlBaseline::BottomEdge` keeps a textarea, a listbox or a
   control without text of its own on its bottom edge).
5. buckram's `propagate_baselines` leaves a `Table` node's baselines as its
   table algorithm declared them (see Findings), so a table in a cell reports
   its first row's.
6. A cell with no line box or row reports no baseline, and K4d5 synthesizes
   a row without one from its cells' boxes as they fill the row
   (`align_table_cells`, with `CellBlockOffsets::block_end`).

**Done-conditions.**

- Every row above matches Chromium in a genet-livery test, with the form
  control rows beside them.
- The three WPT files that waited on this pass again
  (`inline-box-border-line-break`, `inline-box-padding-line-break`,
  `line-breaking-atomic-007`).
- The suites keep their counts, except tests that encode the bottom-edge
  baselines, each named and justified.
- WPT before and after over slice A's directories plus
  `html/rendering/widgets`, `css/css-tables`, `css/CSS2/tables`,
  `css/css-flexbox`, `css/css-grid` and `css/css-align`, every
  `pass -> anything else` attributed, the checked baselines through both
  binaries, and a positive control that flips.

## Next

- **Button centring.** Chromium centres a button's content in a button
  taller than it (a 40px `<button>` has its baseline at 25); genet lays it
  out from the top, so its exported baseline follows genet's layout (18).
- **An inline-block holding only an atom** is 40 tall where Chromium's is 45
  (Findings); its baseline is already right.
- **`inline-flex` and `inline-grid`:** parse them and lay them out as atomic
  inlines aligned on their first baseline (Findings).
- **Control edges in the UA sheet:** a button's 2px border and a text
  input's padding and border (Findings). Adding them moves every
  default-styled control, so they want their own WPT pass over
  `html/rendering/widgets` and the form element directories.
- **Device pixels.** At a device pixel ratio of 2 Chromium gives a 16px
  Arial `normal` line 18.5 and the Arial atom line 44.5, where at 1 it gives
  18 and 45: for a system font its rounding follows the device pixel. genet
  rounds in CSS pixels, which is Chromium at ratio 1 and every ground-truth
  row; following the device pixel needs the text system to know the ratio.
- **Floats** place lines by Parley's line heights; a line the model makes
  taller keeps Parley's float constraints.
- **Quirks mode's collapsed whitespace:** a box whose only text on a line is
  a hanging or collapsible trailing space still counts there.
- **Text fragments under an explicit `line-height`.** Under an explicit
  `line-height`, genet's fragment is still the line box's height at the
  run's baseline offset, or Parley's extent where taller. Chromium's is the
  content area (Findings, 2026-09-27):

  | Case | genet | Chromium |
  |---|---|---|
  | 16px/20px Arial | 20 at 0 | 17 at 1 |
  | 13px/1 | 15 at 0 | 15 at -1 |
  | the table's `super` row, the raised text | 26.33 at -6.33, over the line box's top | 17 at 1 |

  Following Chromium would also move every such inline box's background and
  border onto its content area. So it waits on Mark's ruling; ruling 329
  fixed `normal` only.
- **Inline box edges under `normal`.** A box with borders or padding paints
  the line box on the lines holding its edges and its content area
  elsewhere (Findings, 2026-09-27). Two ways out, for Mark:
  - Place each edge on its box's content area as the text is, which is
    Chromium's painting.
  - Keep inline boxes on the line box, and move only the text node's own
    fragment.
- **Lines off whole pixels.** Carry Parley's unquantized line start, or the
  model's own running top where no float moved the line, so wrapped
  `normal` lines land on Chromium's whole pixels (Findings, 2026-09-27).

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
- **2026-09-25, slice B.** Atoms align on their own baselines. The line
  pass records each line's baseline, a line of atoms or of a forced break
  alone included (`components/genet-livery/src/text.rs`). A table cell feeds
  its subtree's line baselines in and carries them up through buckram's new
  `propagate_declared_baselines_within`, counting a nested table's rows and
  no caption's lines (`feed_line_baselines`, `format_table_cell`). The
  atomic pre-pass exports an inline-block's or button's last line baseline
  and a form control's (`form_control_baseline`, `layout/transaction.rs`).
  In buckram, `propagate_baselines` keeps a `Table` node's declared
  baselines, and K4d5 synthesizes a row with no part from its cells' boxes as
  they fill it (`table/rows.rs`). `tests/line_box_model.rs`'s
  `atom_baselines_match_chromium` holds 29 rows, every one matching
  Chromium: the table above, the form controls, and the nested-table,
  caption, empty-cell, scrollable-button and forced-break cases.
  genet-livery, buckram, genet-scripted-dom, script-runtime-api,
  genet-scripted, genet-documents, genet-render and taproot: 1712 passed,
  none failing. One test changed its expectation:
  `b3_inline_table_uses_its_first_table_baseline` asserted that the first
  cell's bottom edge was the table's baseline; CSS 2.1 17.5.3 and Chromium
  put it on the cell's text, so the test now reads both baselines from the
  painted glyphs and asserts they agree, with the table's top on the line's,
  as Chromium measures. Two buckram tests gained assertions:
  `an_owned_table_context_keeps_the_geometry_it_was_given` (the declared
  baselines survive the walk) and
  `a_row_without_baseline_cells_synthesizes_from_the_lowest_content_edge` (a
  row stretched by its table's height sits on its end).
  `TextFrame::first_inline_baseline` and its test-only record are removed:
  b3 was their last reader.

  WPT receipt: `Code/testing/genet/wpt-ledger/2026-09-25_atom_baselines/`.
  Reftests over slice A's directories and the six added: 29 `fail -> pass`,
  among them the three that waited on this slice, seven table baseline
  alignment files and seven fixed table layout files. One `pass -> fail`,
  `css-flexbox/flex-inline.html`, was a false pass: livery does not parse
  `inline-flex`, and its inline-block reference, on its bottom edge, had
  been pulled off screen with it. Testharness has no `pass -> anything
  else`; one file's `fail -> no-results` was load, shown by rerunning the
  directory with the binaries alternating. The first attempt had six
  regressions; five were real, from three causes, and led to K4d5's row
  fallback and empty cells reporting none, forced-break lines counting, and
  scrollable buttons keeping their content's baseline. The positive control
  flips. The checked baselines report, after, exactly the stale entries
  slice A's receipt recorded, and nothing else.
- **2026-09-27, the text fragment** (Isometry's ruling 329). Under
  `line-height: normal` a glyph run's painted fragment is its box's content
  area (`TextSystem::shape`, `components/genet-livery/src/text.rs`); explicit
  `line-height` is unchanged (Next). The new
  `tests/text_fragment_line_box.rs` measures text fragments against their
  line boxes. Lato from 8 to 40px must not overhang, on every platform. On
  Windows, Arial at 11, 12, 13, 14, 16 and 18px and the two mixed lines must
  match Chromium 153, and the Arial ground-truth table must still match
  Chromium. Before the change the first two fail (Lato overhangs at 14
  sizes; Arial at 11 to 13px, and every row whose Chromium fragment is
  shorter than its line box); the ground-truth rows pass before and after.
  genet-livery, buckram, genet-scripted-dom, script-runtime-api,
  genet-scripted, genet-documents, genet-render and taproot: 1716 passed,
  the 1713 before plus these three, none failing. The other crates that
  depend on genet-livery, genet-scripted-worker, genet-wpt and ortet: 104
  passed, none failing. Isometry's
  `every_side_panel_text_row_holds_its_text`, unignored in a scratch copy of
  Isometry `b981fee` with its genet `0cf4f30b` crates patched to this tree:
  all 187 rows hold, where 39 were a pixel short.

  WPT before and after over slice B's directories, with slice B's runner,
  both binaries built from this tree. Testharness has no transition.
  Reftests have one `pass -> fail`,
  `css/CSS2/linebox/inline-formatting-context-004.xht`, a false pass. The
  test draws a 100px left border on an inline box of default-font text,
  and the reference is a float whose black padding fills its line box.
  genet painted both line-box tall (18), so they matched. Now the
  reference's white span paints its content area (17), as in Chromium, and
  the float shows a row of black below the text. The test's border still
  spans the line box, because its edge does (Findings). Chromium fails the
  file too: its test stripe is 17 and its reference 18, 167 pixels apart.
  The positive control, two reftest pairs of an Arial span's background at
  13 and 16px against Chromium's 15 and 17, fails before and passes after;
  Chromium renders both pairs identical. The checked baselines report,
  after, exactly the entries slice B's run did, name for name, both reftest
  baselines none. The before binary reported one more,
  `dom/ranges/Range-mutations-replaceData.html` hang-killed, while a cargo
  build ran beside it.

### 2026-09-27 annotation: ruling 379, combined text-bound scope

Mark selected A, "Fix both now (recommended). Add explicit-line-height and
wrapped-border fixtures, correct the bounds, and verify the combined change
before merging." The canonical ruling is 379 in
`isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md`, extending 329.
This supersedes the two waiting-for-ruling statements under Next, without
rewriting their dated evidence. Livery's line pass remains the owner; Parley
and the CSS line-box layout model stay unchanged.

The approved change makes text paint fragments use font content bounds under
explicit line-height too, and makes non-replaced inline border/padding edges
attach to those content bounds rather than expanding them to the line box.
CSS 2.2 sections 10.6.1 and 10.8 permit content to bleed beyond short line
boxes; the line-height itself must remain the authored formatting extent.

Lane L reconciled current main `c858738c4cda7436dc3ec953aa7bda89dafd7280`
with the existing normal-fragment patch, preserving generated-text and
nonselectable-source filtering. Only the documentation index conflicted;
its current-main entries and the lane's line-box status were both retained.

The pre-change receipt at
`Code/testing/genet/receipts/2026-09-27/text-fragment` freshly passes three
CPU layout tests on Rust 1.97.1 and Arial 7.07: 16px Arial content is 17px
inside an 18px normal line. Its SHA256SUMS is
`430e5158c7b13b83a62e88de95f3e0b25091b3057bd7a9d3277e242aa68c704b`.
The explicit and decorated defects are source-proven there, not freshly
pixel-measured. The older 187-row, Chromium 153 and WPT statements above
remain historical; their raw September 27 receipts were not located by the
bounded audit and are not claimed as current acceptance.

Done conditions for this slice: measured explicit-height and wrapped-border
fixtures, independent broken controls for both corrections, combined normal /
explicit / decorated regression checks on reconciled source, and qualified
consumer evidence before main integration. Current Isometry's ignored
`every_side_panel_text_row_holds_its_text` is the consumer gate; its live row
count must be reported from the run. Independent review and applicable
regression / consumer gates still block integration. No broader line-box,
UA control styling, fractional wrapping, or fallback-font redesign is ruled.

### 2026-09-27 implementation checkpoint: ruling 379

The combined code now uses each box's rounded font ascent and descent for text
content under normal and explicit line heights. Edge fragments use their own
box's font and aligned baseline, including empty decorated boxes. The separate
formatting line fragments, line-height calculation, glyph baselines and the
current-main generated-text selection filter are preserved.

Six CPU fixture tests pass: the existing three normal/line-box tests, four
explicit-height rows, six wrapped decoration cases (four painted fragments
each), and ten aligned edge cases (baseline, super, sub, top, bottom, each
empty and nonempty). Fresh Arial 7.07 measurements are:

| Fixture | Formatting line | Content top | Content height |
|---|---:|---:|---:|
| 16px / 20px | 20 | 1 | 17 |
| 16px / 8px | 8 | -5 | 17 |
| 16px / 0 | 0 | -9 | 17 |
| 13px / 1 | 13 | -1 | 15 |

Wrapped 16px Arial with a 1px border has four 19px-high painted rectangles
under normal, 24px and 8px line heights; with 2px vertical padding it has four
21px-high rectangles. The normal pre-fix border heights were 20, 19, 19, 20.
Reverting only the explicit text-bounds hunk fails the explicit fixture;
reverting only the edge-bounds hunk fails the wrapped fixture. Both failures
and exact temporary sources are preserved, then the fixed source was restored
byte for byte. These are layout/paint-command bounds, not glyph-ink or GPU pixels.

The broader suite initially found four assertions that equated font content
bounds with line-box bounds. The tests now separately assert: an absolute
child stays at the empty first formatting marker; float bands preserve the
vertical alignment of an equivalent un-floated reference; following block flow
consumes the line-group height rather than the background's content height;
and vertical padding/borders attach by their exact widths to text content.
The original failing logs remain in the receipt. Full regression completion,
consumer evidence and independent review are pending this checkpoint.

Raw evidence is under
`Code/testing/genet/receipts/2026-09-27/text-fragment-combined`. The ignored lane
lock was preserved before offline resolution against current-main manifests;
this receipt uses the published netrender `9607d16f1907f6c2085648ae96abcaa30d7c3d41`
closure, without primary Genet's local dependency overrides. Rust 1.97.1,
source, lock and both font hashes are recorded there. The separate known
fractional wrapped-line start drift is still visible and remains outside 379.

**CPU gate completion, 2026-09-27:** 867 tests passed across the affected
Livery/Buckram suites, with 6 existing ignored tests. This is an aggregate of
`regression-final` (866 pass, one paint assertion failure) and the corrected
`paint-final` target (78/78 pass, replacing its earlier 77/78 result). The last
failure was in the new test's measurement: `DrawText` placement names the
container, not the text content fragment. The final assertion reads the text
node's retained fragment and verifies exact top/bottom decoration offsets.
The production source did not change during these assertion repairs. The
broader source run includes the generated-text, line-box, out-of-flow and
selection suites. No GPU or WPT gate was run.

Changed Rust passes formatting except two unchanged existing formatting hunks
in the old layout/paint test files; `baseline-format-config` reproduces exactly
those hunks from the branch's pre-change files with the same rustfmt config.
`text.rs` and the new fixture file pass rustfmt; `git diff --check` passes.
The full source/lock/font manifest and all failed attempts remain in the raw
receipt. Consumer metadata at Isometry `7cdd5eb` still selects 19 Genet packages
at `0cf4f30`; a patched consumer build has not yet run, and its dependency
closure must be reconciled explicitly before this lane can merge.

**Sealed receipt:** tested code commit
`9b730e7b02dba56a8a486f1b6560c1fa054eb7bd`; 71 raw files in
`Code/testing/genet/receipts/2026-09-27/text-fragment-combined/SHA256SUMS`,
manifest SHA-256
`f44b8ee9acf90c820661650e5fc78e6d6b09c356a8c956f94851a297d1f25d59`.
This seals the CPU checkpoint and its historical attempts. Later consumer or
pixel evidence must be a new receipt, with its own source qualification.

### 2026-09-27 consumer resolution boundary

A bounded Isometry consumer dry run at clean main `7cdd5eb` preserved its exact
tracked lock and configs, then used an external Cargo config covering all 19
Genet packages at Lane L `367626adb64`. `cargo metadata --offline --locked`
stopped before compilation: Lane L's Genet render host requests Netrender
`9607d16` / netrender-vello `^0.10.1`, while Mere `7bb5bfda`'s Cambium host
selects Netrender `c8c09f16` with locked netrender-vello `0.10.0`. Configured
paths do not establish a successfully resolved coherent graph. No consumer
row test ran, and the primary lock remained byte-identical.

The rendering closure and separate test-lock approach now need coordinator
disposition before that gate proceeds. A bounded local compatibility test
holding all four rendering crates at the consumer's existing revision would
qualify that combination only; it would not prove a portable Genet repin to
this lane's manifest closure. No rendering repin is part of ruling 379.

The 16-file dry-run receipt is
`Code/testing/genet/receipts/2026-09-27/text-fragment-consumer-resolution/SHA256SUMS`,
SHA-256 `7a92f9869a8e07d48dd8922305def5b66889bace78f7651f1598a3ba05ea5109`.
The CPU implementation receipt and its source qualifications remain unchanged.

### 2026-09-27 bounded local consumer diagnostic

The coordinator authorized a diagnostic combination holding the consumer's
existing Netrender `c8c09f16` family while replacing all 19 Genet packages with
Lane L. Installed Cargo 1.98.1 demonstrably reads a separate external lock:
a malformed external `Cargo.lock` fails at that path despite the valid primary
lock. The diagnostic uses only an external config and external lock; primary
Isometry manifests, pins, lock and configs remain unchanged.

The complete default-feature graph has 798 packages in both arms. All 19 Genet
packages resolve uniquely to Lane L, all four Netrender packages uniquely to
the same cached `c8c09f16` checkout, and wgpu 30.0.1 / Vello 0.10.0 each retain
one identity. Every other Git and registry package identity is unchanged.
The first audit caught the old `genet-taffy` registry alias; correcting that
external alias produced the fully coherent graph before compilation.

The existing ignored `every_side_panel_text_row_holds_its_text` was explicitly
run with `--ignored`, using the stable Isometry target, four build jobs and
one test thread. It freshly measures 187 rows: expanded 61, composing 64,
picking 62. The local combination passes with zero short rows. The same test
against the unchanged published dependency closure fails with 39 short rows,
13 in each state. The paired control is fresh evidence that the receipt can
detect the original fault, rather than an inference from the historical row
count. These remain retained text-fragment measurements, not glyph ink or pixels.

The initial primary head was `ec43e606`; the local test command ran at
`c040e698`, and the published control at `61def6a9`. Intervening changes are
only Isometry coordination docs. Lane L was `b18b67d0`, with production text
source unchanged from `9b730e7b`. Exact primary/diagnostic locks, configs,
source and Arial family hashes compare unchanged after the pair; both binary
hashes and the source-traced font-selection qualification are preserved.

The 58-file receipt is
`Code/testing/genet/receipts/2026-09-27/text-fragment-consumer-local-c8/SHA256SUMS`,
SHA-256 `e7d5579880fe628ee978c6f92c0e86571812eb60296ea797b8d8afa00e09996f`.
This qualifies only the named local combination. It does not establish a
portable Isometry repin to Genet's `9607d16` rendering closure. The original
ignored gate remains ignored on Isometry's unchanged published pins. Separate
CPU boundary suites for genet-render, genet-documents, genet-scripted and
taproot on Lane L's own published `9607d16` lock are running before the
coordinator assesses Genet integration. Independent source/control review
has passed; no WPT or pixel gate is inferred.

**Boundary gate completion, 2026-09-27:** all 200 requested boundary tests pass
on Lane L's own published `9607d16` / Vello 0.10.1 / wgpu 30.0.1 lock:
genet-documents 54, genet-render 38, genet-scripted 85 plus worker-service 2,
and taproot 21. No failures or ignored cases; four doctest targets contain no
cases. `genet-documents/scripted` enables the Livery/Boa path; optional Vano is
not part of this run. The previous 867-test Livery/Buckram suite was not repeated.
No source or lock changed during the run. A focused existing capture test also
passed during an output-buffering check and is excluded from the 200 count.

The 17-file receipt is
`Code/testing/genet/receipts/2026-09-27/text-fragment-boundary-suites/SHA256SUMS`,
SHA-256 `042b4296ce48469e638b5262783fb862ba15545cae6504625e05799405dbd18f`.
Its exact external `Cargo.lock` copy preserves the lane's ignored lock bytes
(SHA-256 `6206e72891345c6d5baf6884a6e876175f6e341c4760eb614c57aa7e08c5201e`).
Command-time lane head was `b18b67d0`; the production fix remains exactly
`9b730e7b`. Together with independent source/control review and the qualified
local consumer pair, these complete the source fix's bounded checks. Portable
Isometry repinning remains a separate task, as do WPT and pixel acceptance.

Integration remains with the coordinator. A fresh primary Genet inspection
found concurrent retained-motion work at main `c858738`; this lane must not
merge underneath that owner. Its clean-lane code/lock receipts remain valid,
and the existing worktree stays available until a safe integration window.
