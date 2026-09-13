# CSS 3D transforms, first-frame scaling, and the body fragment

**Date:** 2026-09-12

**Status:** founded 2026-09-12, no lane started. T4 added the same day from
the layer-scope finding below, by Mark's ruling. Founded on Mark's rulings of
2026-09-11 and 2026-09-12 on the wing's orthographic voxel presentation plan
(`isometry/mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md`),
whose L0 receipts are this plan's evidence and whose L1 and L5 are its named
consumers. Ruling 3 of that plan authorizes 3D transforms as genet scope and
says genet's own plan founds it; the 2026-09-12 rulings add the first-frame
slice, the host-side mutation harness, and the body-level fragment shape.

**Owns:** CSS Transforms Level 2 in Livery and its lowering to netrender's
`Transform`; the first-frame cost of a document with tens of thousands of
positioned boxes, and the phase-timing facility that makes that cost
attributable; a host-driven DOM mutation harness with real per-frame timing;
and a replaced-element kind whose paint is a retained netrender fragment
supplied by the host.

**Does not own:** what the wing draws or how a voxel body is meshed (the
appearance crate, wing-side); netrender's fragment, tile-cache and execution
graph internals (`netrender/netrender-notes/2026-09-04_wgpu_execution_graph_plan.md`),
with one recorded exception: T4 states the single change to netrender's
retained path that T3 requires, and its receipt, while the change itself lands
in netrender under that plan's owner;
the browsing-context tree and the child-document loading path (the
[iframes plan](2026-09-08_iframes_plan.md) owns those, and this plan reuses
its splice rather than extending it); scripted mutation semantics (the
[parser/script interleaving](2026-09-08_parser_script_interleaving_plan.md)
and [Realms](2026-09-08_realms_plan.md) plans); WPT harness behaviour (the
[harness repair plan](2026-09-07_wpt_harness_repair_plan.md)).

**Prior art and prerequisites in this repository:**

- [iframes and nested browsing contexts](2026-09-08_iframes_plan.md) — the
  paint-list splice (`FrameSlot`, `absorb_frame_commands`,
  `splice_frame_slots`, `record_frame_slot` in
  `components/genet-livery/src/paint.rs`) and the recorded reasons it was
  chosen over an `ExternalTextureDraw` producer texture. T3's replaced element
  reuses that mechanism, in a separate slot key space, with a host-supplied
  netrender fragment in place of a child document's paint list.
- [Shadow DOM](2026-09-07_shadow_dom_plan.md) — the wing's L8 cites its
  declarative-attachment residual; that residual was closed by
  [parser/script interleaving](2026-09-08_parser_script_interleaving_plan.md)
  part two on 2026-09-08. Recorded here so the wing plan's L8 wording can be
  corrected by its owner rather than silently disagreeing with genet.
- [Ortet](2026-09-03_ortet_founding_plan.md) — the headed host every receipt
  below is taken through, and the host T3's harness extends.
- [Standards-to-features ledger](2026-09-07_standards_to_features_ledger.md) —
  rows 15 and 16 are added by this plan.
- [WPT harness repair](2026-09-07_wpt_harness_repair_plan.md) gate F6 — the
  first and only measurement of `css/css-transforms` to date; T1's founding
  count is copied from it.

**Consumers on record (all four lanes):**
`isometry/mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md`
— its L1 (Livery 3D transforms), its L5 (live faces and the body-level
shape), and the two gaps its L0 verdict names. Mark's 2026-09-12 ruling
requires this plan to carry T1, T2 and T3 as that document's named
dependencies.

---

## Why these three are one plan

The wing's L0 measured both halves of the route. L0a
(`Code/testing/wing/l0a_netrender_face_ceiling.md`) holds 200,000 live
rectangles under a 16 ms frame through retained fragments placed by
`Scene::place_fragment`, with a per-instance transform and no re-encode for a
moving body. L0b (`Code/testing/wing/l0b_genet_element_ceiling.md`) holds
about 5,000 DOM elements before the **first frame** exceeds five seconds, and
grows roughly quadratically past that.

The engine is therefore not short of rasterization; it is short of a cheap way
to get many boxes into a scene, and of a cheap element to hang a fragment on.
T1 gives the wing the transform vocabulary it cannot express today, T2 removes
the element ceiling, and T3 removes the need for most of the elements in the
first place while making the live path measurable at all. Landing any one of
the three alone leaves the consumer blocked on the other two.

T4 is the fourth, found on 2026-09-12 by reading the handoff rather than
either side of it: the moment T3's fragment is clipped to its content box it
leaves netrender's retained path, so T3 without T4 measures the flat live
path L0a already rejected.

---

## T1. CSS Transforms Level 2 in Livery

Livery parses no 3D transform today: the wing's earmark table records zero
mentions across the livery crates on 2026-09-11, and netrender's `Transform`
is already a column-major 4x4
(`netrender/netrender/src/scene/geometry.rs`), so the gap is entirely on the
CSS side.

**Surface.** `matrix3d()`, `translate3d()` and `translateZ()`, `rotate3d()`
with `rotateX()` / `rotateY()` / `rotateZ()`, `scale3d()` and `scaleZ()`,
`perspective()` as a function and `perspective` / `perspective-origin` as
properties, `transform-style: flat | preserve-3d`,
`backface-visibility: visible | hidden`, the three-component
`transform-origin`, and the individual `translate`, `rotate` and `scale`
properties with their specified order relative to `transform`.

**Lowering.** Each element's accumulated matrix projects into netrender's
existing column-major 4x4 `Transform`, of which netrender's rasterizers read
the planar six cells (`transform_to_affine` in
`netrender/netrender/src/vello_rasterizer/mod.rs`), so depth sorting and
backface culling are genet's to do before paint, not netrender's; a 3D rendering context
(`transform-style: preserve-3d`) z-sorts its participating boxes by
transformed depth before paint, and `backface-visibility: hidden` culls a box
whose transformed normal faces away. Vello is affine, so the plane the
projection lands on is the wing's orthographic case first; a perspective
divide that cannot be expressed affinely is reported as a named gap, never as
a silent wrong answer.

**Census, copied and not estimated.** `css/css-transforms`, disk mode, Boa,
2026-09-07: **924 files — 13 pass, 77 fail, 13 error, 1 no-results, 820 skip
— 143 / 2,296 subtests**. Source:
[WPT harness repair plan](2026-09-07_wpt_harness_repair_plan.md) gate F6
(residual, 2026-09-07), raw map at
`Code/testing/genet/wpt-ledger/2026-09-07_css_support_residual/results.md`.
That run is the directory's first measurement, so it has no prior baseline to
diff against; it is the founding count for ledger row 15.

**Done when:** the functions and properties listed under Surface parse,
compute and serialize; `css/css-transforms` is re-measured against the
2026-09-07 founding count of 143 / 2,296 subtests with every movement
attributed and zero `pass -> anything else`; a wing fixture body of rigid
parts renders through Ortet with the same silhouette the appearance crate's
bake produces; an individual `rotate` animates one part's yaw without the
parent's matrix being rewritten; and a `preserve-3d` context with two
overlapping faces paints them in transformed depth order in a headless
readback.

---

## T2. First-frame scaling

**Target.** Linear growth in element count, and roughly two orders of
magnitude less cost per element, for documents of tens of thousands of
positioned boxes. This is the wing's L0 verdict restated as an engine target:
without it, live-all-the-time is closed at Paredros scale whichever shape L5
takes.

**Starting hypotheses,** from the read-only attribution in
`Code/testing/wing/l0b_first_frame_attribution.md` (its per-factor table is
the evidence; these are the code locations that attribution names). The
factor that moves the growth exponent is `position: absolute`, and only that:
20,000 elements with the same inline styles and the same transforms cost
97.90 s over baseline absolutely positioned and **3.13 s** in normal flow, an
exponent of 2.24 against 1.28.

1. **The dominant superlinear term is per-positioned-box whole-tree rework in
   the fragment tree.** `apply_absolute_and_fixed_positioning` in
   `components/genet-livery/src/layout/positioned.rs` runs one loop iteration
   per absolutely or fixed positioned box, and each iteration calls
   `FragmentTree::resize_leaf` and `FragmentTree::translate_subtree` in
   `components/buckram/src/fragment_tree.rs`. `resize_leaf` scans every
   fragment in the document to prove the target is a leaf; `translate_subtree`
   scans every fragment in the document and walks each one's ancestor chain to
   decide subtree membership; both then call `recompute_overflow`, which
   clones the whole fragment-id vector and makes two further whole-tree
   passes. That is four whole-document passes and two whole-vector
   allocations per positioned box — O(n^2) in the number of positioned boxes.
   `apply_relative_positioning` in the same file has the same shape for
   `position: relative`, and `components/genet-livery/src/layout/query.rs`
   repeats the pattern on the incremental and query paths.
2. **`TextFrame::translate_subtree`** (`components/genet-livery/src/text.rs`)
   is a third whole-corpus scan per positioned box — every prepared group,
   inline fragment, line key and text cluster — cheap only because the L0b
   fixtures carry no text. A real document pays it.
3. **Parse and style are linear and are not the wall**, which corrects
   finding 4 of `Code/testing/wing/l0b_genet_element_ceiling.md`. A document
   whose every face is `display: none` — full parse, DOM construction and
   style resolution, no boxes — costs 0.59 s at 5,000 elements and 2.52 s at
   20,000, about 0.13 ms per element at an exponent of 1.05. That per-element
   constant is still the second target: `ComputedValues` is a 160-field
   generated struct (`components/livery/build.rs`) inherited by clone per
   element, and the cascade descent in
   `components/genet-livery/src/style.rs` holds each level's computed values
   on the stack across its whole subtree. Removing the exponent gets the wing
   to tens of thousands of elements; removing this constant is the "two
   orders of magnitude per element" half of the target.

A lane that fixes only the first hypothesis has not met the target; the
attribution file is the starting point, not the answer.

**The instrument is part of the lane.** Genet emits no parse, style, layout or
paint spans at info or debug level today, which is why the wing's attribution
had to be done by fixture variant at all. This lane owes a phase-timing
facility behind a flag so the next probe reads spans instead of generating
documents.

**Done when:** the L0b grid is rerun on the same host and the 50,000-element
cell's first frame is **under 1 s**, or an explicitly narrower bound is stated
together with the rationale for why the wider one is not reachable in this
lane; the measured first-frame times across the 1,000 / 5,000 / 20,000 /
50,000 element cells fit linear growth rather than the quadratic recorded on
2026-09-12; a phase-timing facility behind a flag reports parse, style, layout
and paint spans for one frame, and its output attributes the remaining cost
without a fixture variant; and the WPT census over the layout and CSS
directories shows zero `pass -> anything else`.

---

## T3. Host-side DOM mutation harness and the body-fragment replaced element

Two halves of one thing: the wing's live path is the Rust host mutating the
DOM per frame, not page script, and the thing it mutates should be one element
per body rather than one per face.

**The harness.** A host-side facility that mutates element transforms from
Rust once per frame and reports real per-frame timing. It exists because the
L0b live cells could not be measured at all: the Boa-scripted fixtures
rendered byte-identical frames, and wall time around a Fifo-presenting window
cannot resolve one frame. The harness measures the path the wing actually
uses.

**The replaced element.** A replaced-element kind whose paint is a retained
netrender fragment supplied by the host, spliced into the paint list exactly
the way the [iframes plan](2026-09-08_iframes_plan.md) splices a child
document into an `<iframe>`'s replaced box: recorded as a slot while the
ancestors' clips and transforms are still live, filled by the host afterwards,
clipped to the content box, with the same image-key re-keying discipline and
the same awareness that a recorded command index is invalidated by every later
insertion. It gets its own slot list and key space, as `FrameSlot` did beside
`HostLeafSlot`, because its key is a host-supplied fragment identity rather
than a node id or an author attribute.

**The netrender API it consumes.**
`Scene::place_fragment(FragmentId, Transform)`
(`netrender/netrender/src/scene/build.rs`), over `SceneFragment` and
`Scene::append_fragment` (`netrender/netrender/src/scene/fragment.rs`), with
the 4x4 from `netrender/netrender/src/scene/geometry.rs`. That fragments carry
a per-instance transform, and cost a placement rather than a re-encode on a
transform-changing frame, is proved by the wing's L0a receipt,
`Code/testing/wing/l0a_netrender_face_ceiling.md`: 200,000 rectangles on the
live fragment path at 10.0 ms per frame against 50.9 ms for 50,000 on the flat
rebuild path, with lowering staying at one per body across every
transform-changing frame, and the receipt's explicit finding "No API refusal.
Fragments already carry a per-instance transform." Qualified 2026-09-12: that
receipt proves planar placement at layer depth zero. The per-instance
transform is the six-cell affine, so a part's fragment holds one
orientation's projection and a turn is new content (the wing's L0c prices
it); and a placement inside any layer scope is not retained at all today,
which is T4.

**A finding, not a lane.** Under Ortet's Boa scripted profile the L0b live
fixtures — which mutate `style.transform` from a `requestAnimationFrame`
callback, on a timer-driven loop — render **byte-identical frames with no wake
events**, so this Ortet build does not present script-driven mutation.
Recorded in `Code/testing/wing/l0b_genet_element_ceiling.md`, finding 3. It is
not a lane here, because the wing does not need it, but it should be fixed or
explained in the [Ortet founding plan](2026-09-03_ortet_founding_plan.md)'s
scripted-profile gates, beside the browser-hosted scripting residual already
open there. Whether the gap is in Ortet's wake and present loop or in the
scripted document's animation-frame dispatch is **unverified**: it was not
investigated by this plan.

**Done when:** a host-side harness mutates the transforms of every body element
in a fixture once per frame from Rust and reports a measured per-frame time
that is not floored by the presentation interval, for at least the 1,000 /
5,000 / 20,000 element cells of the L0b grid; a replaced element whose paint is
a host-supplied retained netrender fragment renders through Ortet, clipped to
its content box and carried by its ancestors' live clip and transform stack,
with a headless readback proving it composites at the right paint-list
position; an unchanged such element costs a placement and no re-encode across a
frame in which its own transform changed; and the reftest guards and the WPT
census show zero `pass -> anything else`.

---

## T4. Retained fragments inside layer scopes

**Finding (2026-09-12).** Genet's paint-list translator lowers every
`PushClip` to a netrender layer (`emit_push_clip` in
`netrender/paint_list_render/src/emit.rs`); netrender has no standalone clip
op, only `PushLayer(SceneLayer { clip, alpha, blend_mode, filters, .. })`.
The retained path in `netrender/netrender/src/vello_tile_rasterizer/retained.rs`
appends a registered fragment's lowered scene to the master only while
`layer_depth == 0`; inside any open layer it takes the warned fallback and
inlines the fragment un-retained, re-encoding it every frame. T3's replaced
element is clipped to its content box, and every real document carries
ancestor clips besides, so through the paint list as lowered today every body
fragment takes the un-retained path. That is L0a's flat live path, ten to
fifteen thousand rectangles under a 16 ms frame, not its fragment path at two
hundred thousand. L0a did not see this because the probe bypasses Livery and
places fragments at the scene's top level.

The same fallback is what any per-part `opacity`, `clip-path`, `mask-image`
or `filter` would hit, since each lowers to a layer, so the wing's earmarked
tier-1 effects on parts are gated here as well.

**The change.** Netrender's retained path honours a placement inside an open
layer without re-encoding. The fallback's own comment names the obstacle: the
open layer lives in the run's sub-scene, so an append to the master would
escape it. Appending the retained lowered scene into the run's open sub-scene
instead, with the placement affine, is the contained shape; the execution
graph plan's owner may prefer another. This plan records the requirement and
the receipt; the code lands in netrender under
`netrender/netrender-notes/2026-09-04_wgpu_execution_graph_plan.md`, which
this document does not amend.

**Done when:** a fragment placed inside a rect-clip layer, an alpha layer and
a filter layer each renders identically to the same fragment placed at depth
zero in a headless readback; `fragment_lower_count` stays flat across
placement-only frames in all three cases; the layer-scope warning no longer
fires on T3's fixture; the L0a probe rerun with every body wrapped in a
content-box clip layer holds its 200,000-rectangle cell within the 2026-09-11
frame time plus a stated tolerance; and T3's done condition is re-checked
against that rerun rather than against the unclipped receipt.

---

## Deliberate exclusions

- A perspective-correct rasterization path. Vello is affine; a perspective
  divide that cannot be expressed affinely is reported as a gap.
- Extending the browsing-context tree, child-document loading, or anything else
  the iframes plan owns. T3 borrows its splice; it does not amend it.
- Making script-driven mutation present under Ortet. Recorded above as a
  finding for the Ortet plan, not adopted as a lane here.
- A general layout incrementalization program. T2's target is the **first**
  frame; retained-layout reuse across later frames has its own owners.
- Any wing-side work: the appearance crate, ground tile layers, the sprite
  bake. This plan's consumers are named here, not implemented here.
- Amending the wing plan from this document. The L8 correction above is
  recorded for its owner.

---

## Findings

- **2026-09-12** — `css/css-transforms` stands at 143 / 2,296 subtests, 13 of
  924 files passing, in the 2026-09-07 disk-mode Boa run (its first
  measurement; no prior baseline). Source: harness repair plan gate F6.
- **2026-09-12** — the first-frame superlinearity is attributed read-only in
  `Code/testing/wing/l0b_first_frame_attribution.md`; the code locations it
  names are listed under T2's hypotheses.
- **2026-09-12** — the Shadow DOM declarative-attachment residual the wing's L8
  cites was closed by the parser/script interleaving plan on 2026-09-08.
- **2026-09-12** — netrender's retained fragments are not retained inside any
  layer scope, and every genet clip is a layer; fragment placement reads the
  planar six cells of the 4x4. Both from reading `retained.rs`,
  `paint_list_render/src/emit.rs` and `vello_rasterizer/mod.rs`; neither is
  measured yet. T4 and the wing's L0c carry them.

## Progress

- **2026-09-12** — founded. No lane started.
- **2026-09-12** — T4 added by Mark's ruling after the layer-scope finding.
  No lane started.
