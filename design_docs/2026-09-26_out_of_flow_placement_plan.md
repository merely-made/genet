# Out-of-flow placement: fixed boxes inside absolute ones, and shrink-to-fit around them

**Status:** implemented and receipted, 2026-09-26; the item under Next waits
for Mark. The Knot session reported the first gap while docking a Cambium
status bar. Mark ruled the same day that both gaps below are fixed here; that
the retained reposition route declines a subtree holding a fixed box; that
relative and sticky children join the intrinsic admission; and, once the
standards' answer was in (gap 3), that the one-shot entry's missing
shrink-to-fit is fixed here too.

## The gaps

Measured at genet `18e41e44c36`, 600x400 viewport, `body { margin: 0 }`,
through both `layout` and `LiveryDocument`, which agree unless noted.

**1. A fixed box inside an absolutely positioned box lands off by its
ancestors' move.**

| Case | genet | Chromium |
|---|---|---|
| Knot's: a 360px spacer, then `position: absolute; left: 0; right: 0; bottom: 0; height: 20px` holding a fixed box with all four insets 0 | (0, 20) | (0, 0) |
| the same absolute box alone | (0, 380) | (0, 0) |
| the fixed box in an absolute box at `left: 30px; top: 50px` | (30, 50) | (0, 0) |
| two nested absolute boxes, `top: 40px` then `top: 30px` | (0, 70) | (0, 0) |
| a fixed box with `top` and `left` auto in the `left: 30px; top: 50px` box | (30, 50) | (30, 50), its static position |
| a fixed box with `top` auto, `left: 0; right: 0`, in a box at `top: 50px` | (0, 50) | (0, 50) |
| a fixed box in flow, or in a relative, flex or relative flex box | (0, 0) | (0, 0) |

A fixed box's containing block is the viewport whatever its positioned
ancestors (CSS Positioned Layout 3, section 2.1), unless a transform, filter
or containment establishes one; none does here. An axis whose insets are both
`auto` takes the static position, which lies inside the moved ancestor, so on
that axis the fixed box does move with it.

**2. An out-of-flow child makes a shrink-to-fit box fill its containing
block.** An absolute box with `left: 0; top: 50px; height: 20px` and no width
is 0 wide empty and fits text, but is 600 wide around any absolute or fixed
child, even a 50px one; Chromium gives 0 around an out-of-flow child. An
inline-block holding only a fixed child is 600 wide through `layout` and 0
through `LiveryDocument`.

**3. The one-shot `layout` entry fills an inline-block Buckram does not
admit.** An empty inline-block with `padding-left: 10%` is 600 wide through
`layout` and 60 through `LiveryDocument`. The standards give 60: an auto-width
inline-block takes the shrink-to-fit width (CSS 2.1 10.3.9, by 10.3.5), 0
for no content, and its percentage padding resolves against its containing
block's width (8.4), 10% of 600. CSS Sizing 3 (5.2.1) zeroes percentage
padding only in a cyclic contribution, where the containing block's width
depends on the box; this line's does not. Chromium gives 60, and 77.6 around
"xx". The one-shot entry feeds `used_value_context`, and so CSSOM's resolved
values.

**4. A relative or sticky child makes a shrink-to-fit box fill.** An
absolute box around a 50px `position: relative` child is 600 wide; Chromium
gives 50, for `sticky` too, and a relative offset leaves the width alone.

## Causes

- **Placements are resolved before any box moves.**
  `apply_absolute_and_fixed_positioning`
  (`components/genet-livery/src/layout/positioned.rs`) takes every placement
  from `positioned_placements`, which reads each box's current rectangle, its
  containing block's and its static-position source's from the fragment tree
  up front, then moves each box's subtree by target minus current. Moving an
  absolute box moves its fixed descendant, whose placement still holds its old
  rectangle, so the descendant lands off by the ancestor's move. A descendant
  whose containing block moved with it cancels out (its containing rectangle
  is stale by the same amount), as does any axis resolved from the static
  position; the viewport never moves, so a fixed box with insets shows it.
- **The retained reposition route moves the same way.**
  `reposition_stable_positioned_subtree` (`layout/query.rs`) translates a
  positioned subtree whose insets alone changed, fixed descendants included.
- **Intrinsic admission rejects every non-static child.**
  `intrinsic_inline_style_is_admitted` (`components/buckram/src/taffy_adapter/run.rs`)
  refuses a child whose `position` is not `static`, so a subtree holding an
  absolute or fixed child leaves Buckram's intrinsic lane, although
  `measure_intrinsic_inline_subtree` already skips such children. An absolute
  root then gets no intrinsic width and fills its containing block, and an
  inline-block or float falls to `fallback_shrink_to_fit_width`. The same
  check refuses relative and sticky children, which are in flow and measure
  like static ones.
- **The one-shot entry has no atomic pass.** `layout_impl`
  (`layout/transaction.rs`) builds the whole tree without a text system and
  never calls `layout_atomic_subtrees`, so an atomic root Buckram does not
  admit never reaches `fallback_shrink_to_fit_width`, which the retained
  entry's atomic pass applies.

## Design

1. The positioned pass places each box from the tree as its ancestors left
   it: `positioned_placements` splits into collecting candidates and
   resolving one placement, and `apply_absolute_and_fixed_positioning`
   resolves each candidate just before moving it, ancestors first by box-tree
   depth.
2. `reposition_stable_positioned_subtree` declines a subtree holding a fixed
   box, so the ordinary layout places it (Mark's ruling).
3. Buckram admits an absolute or fixed child to an intrinsic query without
   inspecting it, since the measurement already skips it, and admits a
   relative or sticky child as it does a static one.
4. The one-shot entry gives an atomic root Buckram does not admit the same
   shrink-to-fit fallback the retained entry's atomic pass does.

## Done-conditions

- A genet-livery test holds every case above and matches Chromium through
  both entry points.
- A buckram test: an out-of-flow child leaves a shrink-to-fit root admitted
  and its intrinsic sizes unchanged.
- A test: the reposition route declines a subtree holding a fixed box.
- The suites keep their counts, any changed test named and justified.
- WPT before and after over `css/css-position`, `css/CSS2/abspos`,
  `css/CSS2/positioning`, `css/CSS2/floats`, `css/CSS2/normal-flow` and
  `css/css-sizing`, with `css/cssom` in testharness for the resolved values
  the one-shot entry feeds, every `pass -> anything else` attributed, the
  checked baselines through both binaries, and a positive control that
  flips.

## Findings

All 2026-09-26.

- The causes above, each verified at runtime: a probe of the gap 1 cases
  gives offsets equal to the ancestors' moves (Knot's box moves from its
  static 360 to 380, and the fixed box lands at 0 + 20).
- `positioned_placements` stores the static position relative to its source
  fragment and reads that fragment live, so collecting the static-position
  records up front stays correct; only the rectangles go stale.
- The box tree adds a box only under an existing parent, so an element's
  positioned box precedes its descendants' in `CssBoxTree::iter`; the pass
  sorts by depth anyway rather than rely on that.
- A relatively positioned child fails the same admission check (`position`
  is not `static`) though it is in flow (gap 4). The check came in with the
  first intrinsic query (`f72d35ff164`), when K3 routed all positioning to
  K5; it was phasing, not a correctness constraint.
- **A positioned intrinsic query sizes an empty fixed-width child at 0.**
  `positioned_intrinsic_inline_sizes` runs with
  `fixed_leaf_intrinsics_enabled: false`, which `run.rs` records as the
  positioned queries' "separately admitted empty-leaf contract", so an
  absolute box around an empty `width: 50px` child is 0 wide where Chromium
  gives 50; a child holding text contributes its width. Older than this
  change, which only made it visible for relative children; changing it
  overturns a recorded contract, so it waits for Mark.

## Next

- **The positioned empty-leaf contract** (Findings): whether an empty
  fixed-width child counts in a positioned root's intrinsic width, as it does
  in Chromium and in the float and inline-block lane.

## Progress

- **2026-09-26, all four gaps.** The positioned pass resolves each
  placement from the live fragment tree just before its move, ancestors
  first (`layout/positioned.rs`: `positioned_candidates`,
  `positioned_placement`); the reposition route declines a subtree holding a
  fixed box (`layout/query.rs`); Buckram admits out-of-flow children without
  inspecting them and relative and sticky ones as static ones
  (`buckram/src/taffy_adapter.rs`, `taffy_adapter/run.rs`); and the one-shot
  entry measures an unadmitted atomic root on a scratch build and takes
  `fallback_shrink_to_fit_width` against its real containing block
  (`layout/build_block.rs`, `BuildState::measuring_root`). Receipts:
  `tests/out_of_flow_placement.rs`, 20 Chromium rows through both entries;
  `positioned_inset_mutation_over_a_fixed_box_keeps_it_on_the_viewport`
  (`document/tests.rs`); and buckram's
  `positioned_intrinsic_sizes_skip_out_of_flow_children_and_count_relative_ones`.
  Each fails with its fix reverted: the fixed boxes land at the ancestors'
  moves, the shrink-to-fit boxes at 600, the one-shot inline-blocks at 600 and
  300, the repositioned fixed box at x = 60, and the buckram query returns
  nothing. No existing test changed.

  The eight suites: 1718 tests, 1717 passing. The one failure,
  `phase::tests::enabled_spans_accumulate_per_phase_and_drain`, is a timing
  test that compares sleep lengths; it lost on a loaded machine (a 2ms sleep
  measured 10.6ms) and passes alone three times out of three. Flagged
  separately.

  WPT receipt: `Code/testing/genet/wpt-ledger/2026-09-26_out_of_flow_placement/`.
  Zero transitions of any kind over all thirteen maps, reftest and
  testharness; the positive control flips, so no file in those directories
  depends on these cases. The checked baselines report the same entries
  through both binaries; their extra `hang-killed` timeouts over slice B's
  run of the same before binary are load.
