# Buckram K7: atomic inline boxes against their containing block

**Status:** first slice landed 2026-09-24; a second slice, closing the two
pre-pass gaps the first one found, is next. Execution note under the
[Buckram master plan's K7](2026-07-26_buckram_css_layout_engine_plan.md#k7-foundational-sizing-and-dispatch-closure).

## Why this slice

Knot's editor, a `textarea` styled `width:100%`, painted at the window's width
inside a 900px column (Knot workspace slice 1, defect D1, 2026-09-24). The cause
is Livery's atomic-inline pre-pass. `layout_atomic_subtrees` formats every
atomic inline root with the viewport as its containing size, because it runs
before the main layout has placed any containing block. Probe on
`16ca28adda3`, with the child in a 500px parent under a 1100px viewport:

| child | width |
|---|---:|
| inline-block, `width:100%` | 1100 |
| the same with `box-sizing:border-box` and padding | 1100 |
| `textarea`, `width:100%` | 1100 |
| block, `width:100%` | 500 |

Knot has since made its editor a block, so no consumer waits on this slice. It
is K7 step 1 extended to atomic inline boxes, with the continuous-media part of
the `IndefiniteInlineSize` row. A consumer defect chose it rather than K7's
reconciliation, and it claims neither K7's closure nor its order.

## What CSS requires

- An atomic inline box's containing block is formed by its nearest block
  container ancestor (CSS 2.1 §10.1). Its percentage `width`, `min-width`,
  `max-width`, padding and margins resolve against that block's inline size, on
  all four sides for padding and margins (§10.2, §8.3, §8.4). A percentage
  height resolves only against a definite height (§10.5).
- An auto-width inline-block is shrink-to-fit against the available inline
  size its containing block leaves it (§10.3.9).
- When the containing block's own inline size depends on its content (a float,
  an inline-block, a positioned or flex or grid box sized from content), the
  atom's cyclic percentages follow CSS Sizing 3 §5.2.1. For its contribution,
  percentage preferred and max sizes act as their initial values and percentage
  padding and margins as zero. The percentages then resolve against the
  containing block's resulting size, and that size is not re-resolved.

## Design

Two passes, not a fixed point.

1. **Contribution pass.** The existing pre-pass, with the viewport still the
   available inline size, except that an atom root whose own style holds a
   containing-block percentage is formatted with those percentages treated as
   §5.2.1 says. The main layout sizes content-dependent containers from these
   contributions, as it sizes them from the pre-pass today.
2. **Basis pass.** After the main layout, each atom root's containing block
   comes from the box tree (`CssBox::containing_block`) and its content box from
   the main layout's fragments. An atom is *basis-sensitive* when its style
   holds such a percentage, or when it is auto-width and not replaced and its
   pass-one outer width reached the smaller of the pre-pass's available size
   and the real one. The second trigger catches a containing block narrower
   than the window and also one wider: under a 1100px viewport the pre-pass
   clamps a 1500px max-content atom in a 2000px parent to 1100. If any atom is
   sensitive, the pre-pass runs again with a per-atom basis, sensitive atoms
   against their real containing block and the rest as in pass one. The plane
   keeps each sensitive atom's pass-one width as its intrinsic contribution
   (`AtomicLayoutPlane::intrinsic_inline`, which `push_atomic_box` already reads
   for min-content and max-content queries), and the main layout runs once
   more. Lines take the basis-pass sizes while intrinsic queries keep the
   contributions, so a content-sized container is not re-resolved.
3. **Stability.** A debug assertion checks that each sensitive atom's
   containing basis after the second main layout is the one it was formatted
   against. A mismatch is loud; a release build keeps the second pass.

The retained root formatter is untouched. It already refuses subtrees that hold
inline-level element boxes, so an edit inside an atom takes the full path,
where both passes run.

## Done-conditions

Structural fixtures in genet-livery, each failing on `16ca28adda3` where it
names a change:

- inline-block and `textarea` at `width:100%` in a 500px parent come out 500,
  with and without `box-sizing:border-box` and padding;
- percentage padding on all four sides of an inline-block resolves against the
  500px parent;
- an auto-width inline-block holding a paragraph wraps within its 300px column;
- an inline-block with a 1500px max-content in a 2000px parent under a 1100px
  viewport comes out 1500;
- a float holding a `width:50%` inline-block whose content is 300px wide comes
  out float 300, atom 150;
- an auto inline-block holding a 200px text line and a `width:50%` child comes
  out outer 200, child 100, in one settle;
- an atom inside a flex item and one inside a table cell resolve against their
  own item and cell;
- a row of auto-width buttons whose labels fit needs no basis pass.

Suites: buckram, genet-livery and the Genet workspace (genet-parley's test
target aside) at their counts before this slice. WPT before and after, reftest
and testharness, over `css/CSS2/visudet`, `css/CSS2/normal-flow`,
`css/CSS2/linebox`, `css/css-sizing`, `html/rendering/widgets` and
`html/rendering/replaced-elements`, with every `pass -> anything else`
attributed. Receipt under
`Code/testing/genet/wpt-ledger/2026-09-24_k7_atomic_basis/`.

Three of these wait on the second slice, their fixtures ignored with the gap
they name: percentage padding, the percentage child, and the auto
inline-block that wraps in its column.

## Named gaps, not closed here

- **Next slice.** An atom that Buckram does not admit to intrinsic
  shrink-to-fit, such as one with percentage padding or a percentage child,
  fills its containing block instead of shrinking to fit. Before the first
  slice it filled the viewport.
- **Next slice.** The atomic pre-pass measures an atom's text at one line
  whatever width it wraps at, so the text of a narrowed atom overruns its
  box. The `pre-wrap` case below is one face of it.
- A replaced atom keeps its natural-size path, so its percentages (such as
  `max-width:100%`) stay unresolved in the pre-pass, as before.
- One pass-one width answers both min-content and max-content queries for an
  atom, as before this slice.
- An inline-block with `white-space: pre-wrap` is measured with its newlines
  collapsed. A 200px inline-block holding six pre-wrap lines gets a 73px box
  while its text paints to y=133, with or without the retained preparation of
  its text (found 2026-09-24).
- Nested atoms are formatted inside their outer atom's subtree, which this
  slice does not change.
- A percentage height sees a definite containing height only when that block
  container's own height is a length.

## Progress

- **2026-09-24, first slice.** The two passes as designed, with three things
  the fixtures found. A root that was not shrink-to-fit had no parent to
  resolve its percentages against, so it filled whatever width it was given;
  a width of 100% had only looked right because 100% and fill agree. The
  basis pass now wraps every sensitive root in a block the size of its
  containing block. An inline context's cached layouts were keyed by width
  alone, so a shrink-to-fit container's definite layout reused the one its
  intrinsic query had left at the same width, with atoms at their
  contribution widths. Entries now record whether they answered an
  intrinsic query, and placement prefers the definite one at an equal width.
  Replaced roots keep their natural-size path and are not basis-sensitive,
  since formatted against a definite width they stretch to it.

  Fixtures: percentage widths 1100 -> 500 for the inline-block, the
  border-box inline-block and the textarea; an inline-block in a 2000px
  block reaches its max-content (1100 before); atoms in a flex item and a
  table cell take their own block (1100 before); the §5.2.1 float comes out
  float 300 (550 before) and atom 150; a row of fitting buttons takes no
  basis pass. Ignored for the second slice: percentage padding and the
  percentage child (the atom fills its block, 500 and 600 where it should be
  120 and 200, and 1100 before), and the auto inline-block that wraps in its
  column (one line tall).

  genet-livery 567 with 3 ignored, buckram 269; the Genet workspace without
  genet-parley's test target, 2,966 passed and 1 failed, the failure
  genet-render's accessibility bounds test from before this slice. WPT over
  the six directories: no `pass -> anything else` in 12 runs, and
  `css/css-sizing/intrinsic-percent-replaced-011.html` fail -> pass. Receipt
  `Code/testing/genet/wpt-ledger/2026-09-24_k7_atomic_basis/results.md`.
