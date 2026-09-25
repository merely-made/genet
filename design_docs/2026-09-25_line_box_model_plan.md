# Line box model: CSS 2.1 §10.8 in Livery's line pass

**Status:** plan, 2026-09-25. Mark ruled the home the same day: genet-livery's
line pass, with Parley kept upstream-shaped rather than patched.

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

For the gap's line that gives: text reaches 14.48 + 1.06 above the baseline
and 3.39 + 1.06 below; the atom reaches 40 above. The line box is 40 + 4.45 =
44.45 tall, its top is the atom's top at y = 20, and the next line starts at
64.45.

## What genet does today

`text.rs` lays a line out with Parley, then corrects Parley's single-height
metrics after the fact: a requested line height taken from explicit
`line-height`s and in-flow atoms, extra leading split about the line,
`text-top`/`text-bottom` edge leading, an inline table's exported baseline,
a shift for empty inline boxes, and the break-line extents that
`5621ca05768` added. `vertical_align_shift` then moves items after the line's
metrics are fixed. Two structural consequences follow, whatever the
individual corrections get right:

- A line's top comes from Parley's `block_min_coord`, so a line whose height
  genet corrects does not move the lines after it.
- A `sub`, `super` or length-shifted run never grows its line box, which
  §10.8 requires.

## Design

In the line loop of `format_inline_group`, per line:

1. Collect the boxes on the line: the strut from the root style; one extent
   per glyph run from its span's font metrics and used `line-height`; one per
   atomic inline from its margin box and baseline (an inline table exports
   its first row's).
2. Place each box by `vertical-align` relative to the baseline, holding back
   `top` and `bottom`.
3. Take the union of the placed extents as the line's space above and below
   the baseline, then place the held-back boxes against it, growing it if one
   is taller.
4. The line box's height is above plus below. Its top is the previous line
   box's bottom (0 for the first line); its baseline is its top plus above.
5. Emit every item at its CSS position: glyph runs translated from Parley's
   baseline to the line's baseline plus their own shift, atoms at the
   baseline minus their baseline offset. Fragments and line fragments take
   the new tops and heights.

The corrections the model subsumes are deleted rather than layered on;
Parley still breaks lines and shapes glyphs, and only the vertical placement
changes. Quantization stays compatible with Parley's (Chrome's rounding of
ascent and descent before splitting leading), so text-only lines at an
integral `line-height` keep their current positions.

The first slice aligns every box against the line's root baseline plus its
own shift. Nested inline boxes that carry their own `vertical-align`, such as
a `sub` inside a `super`, need the shift chained through their parent's
baseline; if the first slice does not chain it, that is recorded here as the
next gap rather than approximated.

## Done-conditions

- Knot's repro: the atom sits at y = 20, its line is 44.45 tall (or as
  quantization rounds it), the next line starts where it ends, and no line in
  the genet-livery suite overlaps the one before it.
- A `super` and a length-shifted span grow their line box as Chromium does,
  and a 32px span inside 16px/20px text gives Chromium's line height, each as
  a fixture checked against Chromium.
- An inline table still exports its first row's baseline, per the existing
  tests.
- genet-livery, buckram and the Genet workspace suites keep their counts,
  except tests whose expectations encode the old model, each named and
  justified in the progress entry.
- WPT before and after, testharness and reftest, over `css/CSS2/linebox`,
  `css/CSS2/visudet`, `css/css-inline`, `css/CSS2/normal-flow`,
  `css/css-text/line-breaking` and `html/rendering/non-replaced-elements`,
  with every `pass -> anything else` attributed, and a positive control that
  flips.
- The Knot session is told, so its inline-block workaround can come out.

## Risks

- Text lines can move by sub-pixel amounts where the old corrections and the
  model round differently; the WPT receipt attributes each transition rather
  than tolerating them.
- The line pass feeds painting, hit testing, carets and selection through the
  same fragments. Their suites (genet-render, genet-documents, genet-scripted,
  taproot) run with genet-livery's.

## Progress

None yet.
