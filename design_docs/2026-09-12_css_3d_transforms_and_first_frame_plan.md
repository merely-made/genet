# CSS 3D transforms, first-frame scaling, and scene viewport embedding

**Date:** 2026-09-12

**Status, 2026-09-16:** prerequisite assessment complete; **T1 is implemented
and measured** — the Level 2 surface parses, computes, serializes and
interpolates, each element's accumulated 4x4 reaches the shared paint and
hit-test matrix, `css/css-transforms` moves 291 -> 650 subtests and 534 -> 600
reftests with every movement attributed and zero `pass -> anything else` at
file level, and the wing fixture's silhouette agrees with the appearance
crate's bake to 98.73% with no pixel more than one pixel out. **T2 is
implemented and measured** — the L0b grid's first-frame growth falls from n^2.156 to
n^1.168 and the 50,000-element cell from 1,172.21 s to 7.68 s, with a
phase-timing instrument behind `--phase-timing` and a status-identical WPT
census; the 1-second bound at 50,000 is not reached and the narrower bound is
stated under T2. T3's bounded Genet
content-box and 2D paint/input seam is implemented with automated receipts.
Bounded native Bench B assembly is accepted on the downstream development
build. T3's **host mutation instrument is now built and measured** on the
bounded fixture, with actual work reported apart from presentation wait; the
large-element mutation sweep is measured at 1,000, 5,000 and 20,000 elements,
the last on five frames only; 50,000 was not attempted. The remaining broader
T3 acceptance (native composition rerun, WPT attribution) is still open.
**T4 is implemented and measured in netrender** (`06f3a12f4`): a placement
inside a rect-clip, alpha or element-filter layer now retains and reads back
byte-identical to an independently expanded reference, `fragment_lower_count`
stays at one lowering per fragment across placement-only frames, and the
clip-wrapped L0a rerun holds its 200,000-rectangle cell within 1.07x of the
same run's unclipped cell. Nothing ran through Genet.
Founded 2026-09-12 from the wing's L0 receipts;
Mark's 2026-09-13 ruling replaces T3's body-fragment proposal with a shared
scene viewport whose producer owns depth. T1's general CSS 3D work, T2's
large-document scaling and T4's planar retention remain independent lanes,
not prerequisites for the first specimen bench. Existing native canvas
acceptance is prior evidence, not a fresh rerun in this implementation.

**Consumer and cross-repository owner:** the specimen bench prerequisites in
`isometry/mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md#specimen-bench-prerequisites-2026-09-13`.
That section owns the application assembly and prerequisite order. This
document owns the Genet engine changes it calls for.

**Owns:** CSS Transforms Level 2 in Livery and its lowering to netrender's
`Transform`; the first-frame cost of a document with tens of thousands of
positioned boxes, and the phase-timing facility that makes that cost
attributable; host-driven mutation measurement through the engine host; and
the existing host-content paint seam's content-box placement and shared 2D
paint/input geometry for a scene image in ordinary document composition.

**Does not own:** what the wing draws or how a voxel body is meshed (the
appearance crate, wing-side); netrender's fragment, tile-cache and execution
graph internals (`netrender/netrender-notes/2026-09-04_wgpu_execution_graph_plan.md`),
with one recorded exception: T4 records the planar-retention requirement and
its receipt, while the change itself lands in netrender under that plan's
owner; Cambium's same-device producer lifecycle, resolved-style query hook,
and application style-to-instance mapping (Mere and the wing own those);
the browsing-context tree and the child-document loading path (the
[iframes plan](2026-09-08_iframes_plan.md) owns those); scripted mutation semantics (the
[parser/script interleaving](2026-09-08_parser_script_interleaving_plan.md)
and [Realms](2026-09-08_realms_plan.md) plans); WPT harness behaviour (the
[harness repair plan](2026-09-07_wpt_harness_repair_plan.md)).

**Prior art and prerequisites in this repository:**

- Existing custom-leaf slots (`HostLeafSlot`, `splice_host_leaf_slots` in
  `components/genet-livery/src/paint.rs`) keep host commands inside the active
  CSS paint scopes. T3 reuses this seam and the external-image translation;
  it adds neither a separate fragment slot list nor a compositor.
- [Ortet canvas composition acceptance](2026-09-03_ortet_founding_plan.md#canvas-composition-follow-up-2026-09-11)
  — same-device source staging into an ordinary `SceneImage`, with bounded
  native proof of content-box placement, 2D affine transforms and origins,
  rectangular overflow, group opacity and source-over. T3 carries those
  checks to the host-owned scene producer.
- [iframes and nested browsing contexts](2026-09-08_iframes_plan.md) — the
  adjacent child-document splice and its command-index and resource-key
  discipline remain owned there; the scene viewport is not a child document.
- [Shadow DOM](2026-09-07_shadow_dom_plan.md) — the wing's L8 cites its
  declarative-attachment residual; that residual was closed by
  [parser/script interleaving](2026-09-08_parser_script_interleaving_plan.md)
  part two on 2026-09-08. Recorded here so the wing plan's L8 wording can be
  corrected by its owner rather than silently disagreeing with genet.
- [Ortet](2026-09-03_ortet_founding_plan.md) — the minimal engine host for
  Genet's receipts and mutation instrument. Cambium's native host supplies
  the application-side specimen bench receipt under Mere ownership.
- [Standards-to-features ledger](2026-09-07_standards_to_features_ledger.md) —
  rows 15 and 16 are added by this plan.
- [WPT harness repair](2026-09-07_wpt_harness_repair_plan.md) gate F6 — the
  first and only measurement of `css/css-transforms` to date; T1's founding
  count is copied from it.

**Consumers on record:** the wing's broader L1 and L5 presentation work and
its L0 findings motivated these lanes. The specimen bench now consumes T3's
bounded engine seam plus Mere's producer bridge. Its bodies are renderer
instances in one shared-depth scene, not thousands of transformed DOM boxes.

---

## Why these lanes remain separate

The wing's L0 measured both halves of the route. L0a
(`Code/testing/wing/l0a_netrender_face_ceiling.md`) holds 200,000 live
rectangles under a 16 ms frame through retained fragments placed by
`Scene::place_fragment`, with no re-encode for planar affine placement at
layer depth zero. It does not prove changing a body's 3D orientation. L0b
(`Code/testing/wing/l0b_genet_element_ceiling.md`) holds
about 5,000 DOM elements before the **first frame** exceeds five seconds, and
grows roughly quadratically past that.

L0c then compared planar fragments, sprites and resident geometry with depth.
Its renderer-only evidence and Mark's ruling select one depth-owning scene
viewport for the first bench. The producer performs the 3D work; the document
composes its result as an ordinary image among a bounded set of controls.

T1 supplies a general CSS 3D vocabulary. T2 measures and reduces large-DOM
first-frame cost. T4 repairs retention for planar fragments under layers.
Those remain useful independently, but none is required to display or
interact with the bench's shared-depth image. T3's bounded content-box,
composition and 2D input mapping, followed by the Mere-owned producer bridge,
are the relevant prerequisites. The large-element mutation sweep remains
separate measurement work rather than a bench admission gate.

---

## T1. CSS Transforms Level 2 in Livery

Livery parses no 3D transform today: the wing's earmark table records zero
mentions across the livery crates on 2026-09-11, and netrender's `Transform`
is already a column-major 4x4
(`netrender/netrender/src/scene/geometry.rs`). That representation alone does
not establish 3D rendering: the affine lowering and admitted rendering
contexts below still need their own correctness proof. T3's shared-depth
viewport does not depend on this CSS surface.

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

### T1 implementation, 2026-09-15

**Status: implemented and measured.** Receipts in
`Code/testing/genet/t1_css_3d_20260915/results.md` and
`Code/testing/genet/wpt-ledger/2026-09-15_t1_css_transforms/`. Committed as `d61978378c1`.

**Where the matrix lives.** `Matrix3D` in
`components/livery/src/values/transform_matrix.rs` holds sixteen cells in
`matrix3d()` argument order — the column-major cells of the column-vector
matrix CSS composes with, which is also `euclid::Transform3D`'s row-vector
row-major order, so lowering to `LayoutTransform` is a field-for-field copy
and the existing 2D mapping is the same mapping. `Matrix2D` stays, and is now
the projection of a composed `Matrix3D` rather than a parallel composer:
`Matrix2D::from_functions` returns a matrix only when the composition never
left the plane, which is what keeps the interpolation path honest about lists
it cannot decompose.

**Where the 4x4 accumulates.** `paint::element_matrix` composes, in the
spec's order, the parent's perspective over `transform-origin` (all three
components) over `translate`, `rotate`, `scale`, then the `transform` list.
`transform_spec` factors the origin back out so the existing `TransformSpec`
contract — matrix, then placement origin — is unchanged, which is what keeps
`placement.rs` and `layout/hit_testing.rs` reading paint's exact resolved
matrix rather than a second CSS parser. T3's shared helpers are untouched.

**Flattening.** The renderer composes the pushed 4x4s and projects the
product, so a box outside a 3D rendering context has its own matrix projected
here — `flattens_own_matrix`, true when the element is `flat` *and* its parent
is not `preserve-3d`. Projecting once at each boundary reproduces the spec's
flattening of the accumulated matrix, because projection composes: two nested
`rotateX(45deg)` under the default `flat` are two foreshortenings and not a
half turn.

**The sort.** `sort_preserve_3d_runs` reorders stacking items by the depth of
their transformed centre in the accumulated space. Items reach the stacking
list already flattened out of their DOM parents, so the context is keyed off
each item's own `preserve-3d` parent rather than off the walk's parent: a
context whose own box establishes no stacking context still owns its
children's depth order. Depth replaces the z-order key inside a context rather
than refining it.

**The cull.** `backface-visibility: hidden` drops a box whose
`front_facing_z` — the Z of the cross product of the accumulated X and Y axes,
which is the determinant of the projected affine part — is negative. The
accumulation stops at the element's own 3D rendering context, not at the
document root, which is what CSS Transforms 2 means by "only transforms that
affect the child itself".

**The perspective gap.** `record_transform_gap` reads the element's matrix
*before* flattening and, when its fourth row is not `0 0 0 1`, records a
`TransformGap { PerspectiveDivide, border_rect, projection: [m14, m24, m34] }`
on the paint list with a one-line `message()`. The orthographic projection of
the same matrix is then painted. The `perspective` property and the
`perspective()` function both reach it, and an affine 3D transform reports
nothing.

**Orthographic exactness.** `Matrix3D::rotation` writes cardinal axes out
directly instead of through Rodrigues, whose unrotated diagonal cell is
`(1 - cos) + cos` and is not exactly 1 in f32. Without that, every
`rotateZ()` composed to something that only looked like a 3D matrix, and
`transform-2d-getComputedStyle-001` said so.

---

## T2. First-frame scaling

**Status, 2026-09-15:** implemented and measured. The quadratic term is gone:
the L0b grid's fitted growth exponent falls from **2.156 to 1.168** and the
50,000-element cell's first frame from **1,172.21 s to 7.68 s** over baseline,
with every cell's rendered digest unchanged. The phase-timing facility is built
and it attributes the remaining cost without a fixture variant. The 1-second
bound at 50,000 is **not** reached; the narrower bound and its rationale are
recorded below under "What was reached". The WPT census over six layout and CSS
directories shows zero transitions of any kind.

**Target.** Linear growth in element count, and roughly two orders of
magnitude less cost per element, for documents of tens of thousands of
positioned boxes. This is the wing's original L0 large-element verdict
restated as an engine target. A scene viewport with instance data does not
inherit that DOM count, so this target does not gate the first bench.

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

**What was reached, 2026-09-15.** Receipt:
`Code/testing/wing/l0b_t2_2026-09-15/results.md`, with raw timings, the phase
receipts and the fitted exponent beside it. Host, fixtures and method are the
2026-09-12 receipt's, unchanged.

| elements | before, over baseline | after, over baseline | speed-up |
|---:|---:|---:|---:|
| 1,000 | 0.239 s | 0.078 s | 3.1x |
| 5,000 | 4.976 s | 0.372 s | 13.4x |
| 20,000 | 102.416 s | 2.015 s | 50.8x |
| 50,000 | 1,172.213 s | **7.682 s** | 152.6x |

Fitted exponent over the four cells: **2.156 (R² 0.995) before, 1.168
(R² 0.993) after**. Growth is linear rather than quadratic, which is the second
done-condition met.

**The narrower bound, and why the wider one is not reachable in this lane.**
The 50,000-element first frame is **7.68 s**, not under 1 s. The instrument
attributes 6.04 s of it to the four phases — parse 0.15 s, style 2.03 s, layout
2.89 s, paint 0.96 s — and a finer split inside style, taken once and removed,
puts 2.25 s of the 2.03–2.56 s style phase in the cascade alone. The cascade's
inner loop matches **every rule in the style set against every element**
(`for rule in &style_set.rules` in `resolve_subtree_on_this_stack`,
`components/genet-livery/src/style.rs`) with no selector bucketing by tag, class
or id, and parses one inline `style` declaration block per element. Cutting the
resulting ~45 µs per element to the ~8 µs the 1-second bound needs is a selector
index plus rule-match caching plus a change to how `ComputedValues` is stored —
hypothesis 3's "two orders of magnitude" half, a change to the cascade's shape
with real conformance risk, and not something the L0b fixtures can validate.
It is left as its own lane rather than attempted blind here.

One residual is named and **not** attributed: the layout phase's own per-element
cost still grows across the last step (30.3 µs/element at 20,000 against
57.9 µs at 50,000) while style, paint and parse stay flat, and that residual is
the whole of the remaining 1.17 exponent.

**What changed.** Three things, in descending order of effect.

1. *The fragment tree learned its own children.* `FragmentTree` now carries a
   `children` index in slot order beside `slots` and `by_box`. `resize_leaf`'s
   leaf test becomes a map lookup instead of a whole-document scan;
   `translate_subtree` walks the subtree instead of testing every fragment's
   ancestor chain; `structural_children` and `subtree_ids` read the index, which
   also removes the quadratic in `match_retained_subtree`'s descent.
2. *Aggregate overflow is rebuilt once per layout pass, not once per mutation.*
   `translate_subtree`, `resize_leaf` and `reconcile_parent` mark
   `overflow_dirty`; `flush_overflow` does the rebuild, and the positioned,
   relative, sticky, retained-root and incremental-query passes each call it
   once at their end. The result is identical — a focused test replays a
   mutation sequence against a tree that recomputes after every single mutation,
   including a shrinking leaf resize, and compares fragment for fragment.
   `set_overflow` stays eager on purpose: it is the table grid's route, called
   once per grid, and its result is read immediately.
3. *Two per-element style costs.* `resolve_container_relative_styles_with_images`
   cost a clone, a whole-plane structural comparison and a second clone for
   every document; the descent now reports whether it moved anything and the
   common document pays one clone. And resolving relative lengths went through
   `ComputedValues::get`/`set`, building a tagged `PropertyValue` and cloning
   every non-Copy value once per property per element per pass; a generated
   `resolve_relative_lengths_in_place` walks the fields directly, and a new
   `ResolveViewport::RESOLVES_RELATIVE` constant folds the identity families
   away at compile time.

`TextFrame::translate_subtree` gained the matching bound: it asks first whether
the translated subtree owns any retained text and returns immediately when it
does not, and it walks the keyed inline planes by subtree node rather than by
corpus entry. The two `Vec` scans it keeps — prepared paint groups and text
clusters — are still whole-corpus **when the subtree does own text**; indexing
those is left open, and the L0b fixtures carry no text, so this lane could not
measure it either way.

---

## T3. Scene viewport embedding and host mutation measurement

**Status, 2026-09-15:** bounded engine implementation, automated receipt and
downstream native Bench B assembly complete. The host mutation instrument is
built and measured (receipt below); broader T3 acceptance remains open. The
specimen bench prerequisite section named above is the cross-repository
assembly authority. T3 specifies the Genet portion, not an application renderer.

**The element and paint route.** Reuse the existing `<custom-leaf>` slot and
`splice_host_leaf_slots` in `components/genet-livery/src/paint.rs` to place one
`DrawExternalTexture`. The translator in
`netrender/paint_list_render/src/lib.rs` emits an ordinary `SceneImage` at
that position, retaining the active transform and layer scopes. There is no
new fragment slot list, iframe, per-body surface or post-document overlay.
The producer owns ground and mutually occluding body parts in one depth
attachment; Genet owns placement of the resulting image in the document.

**Existing evidence to reuse.** Ortet's native canvas route already stages
sources before rasterization (`ports/ortet/src/shell.rs` and `webgl.rs`) via
`Renderer::stage_external_image` and `unregister_external_image`
(`netrender/netrender/src/renderer/mod.rs`). Staging samples on the shared GPU,
converts alpha as needed and registers an image; it is not zero-copy. The
producer must advance its content generation and never reuse a live key for
another source. Unchanged key, size and generation skip staging.

The [2026-09-11 canvas acceptance](2026-09-03_ortet_founding_plan.md#canvas-composition-follow-up-2026-09-11)
at Genet `62e1a0fad82` and netrender `3961aca919` passed 25 analytic native
pixel probes on Boa and Nova at display scale 2. Its bounded scope is 2D
affine transforms and authored origins, rectangular overflow, content-box
placement, group opacity and source-over. Netrender's GPU gate separately
covered scales 1, 1.5 and 2 and registration, refresh, resize and removal.
Those receipts were inspected during this assessment, not freshly rerun.
Cambium's scene producer and transformed picking have the separate downstream
consumer receipt below.

**Genet changes.** The custom-leaf slot now uses the same content-box geometry
as `emit_canvas_external_texture`, excluding borders and padding from image
placement. `element_geometry` and `ElementGeometry` in
`components/genet-livery/src/placement.rs` expose that size and inverse
content-local mapping; ordinary controls have a separate border-local map.
The caller adds document scroll before querying. Producer content is stationary
inside its own box; its DOM descendants receive its own scroll, and ancestor
scroll applies to both. The existing content-box helper is exact for resolved
pixel padding/borders; its percentage-padding basis remains the element's own
width, a pre-existing approximation that this slice does not make conformant.

The slot adds no unconditional clip or layer. A producer's
`DrawExternalTexture` rectangle bounds its content, while ordinary CSS overflow
and clip paths retain their existing scopes. A plain retained custom leaf must
remain at layer depth zero, preserving the fast path independently of T4.
The command-index and resource-key discipline stays in the existing splice.

Livery's hit test (`components/genet-livery/src/layout/hit_testing.rs`) now
uses the same resolved transform-origin, transform and clip helpers as paint,
with inverse accumulated placement and the content-box offset. It follows
paint's negative, normal and nonnegative stacking phases. Flattened stacking
items now carry intervening ancestor scroll scopes along with overflow clips.
A pointer in a transformed bounding rectangle is not necessarily inside the element;
clipped points and singular transforms must not produce a scene pick. This
is bounded 2D affine geometry, independent of T1's general 3D rendering contexts.
`StylePlane::used_color` supplies encoded sRGB foreground channels and straight
alpha after the existing palette and color-scheme resolution; application
conversion into a linear scene tint remains outside Genet.

**Mere-owned consumer bridge.** Cambium's Rootstock owns the same-device
producer registration, content generation, physical size, suspension and
retirement hooks, staging before its ordinary raster pass, and a read-only
resolved-style query for application hooks. Its existing custom-leaf
constructor and Sprigging scene buffers alone do not establish the producer's
native acceptance. The implementation belongs under
`mere/crates/cambium/cambium-rootstock`, not in Genet or a second compositor.

The host converts an admitted input point through Genet's inverse mapping
into content-local coordinates. The application then maps it to its render
pixels/camera and picks stable body/part identities at the relevant scene
epoch. Cambium retains pointer capture, focus and DOM hit precedence.
Accessible specimen/part controls reference those same identities; the image
does not automatically supply an accessibility subtree. Supported style
properties map explicitly to instance data in the application. Arbitrary
per-part CSS filters, masks or opacity are not implied by the viewport.

**Mutation measurement.** Keep the host-side Rust mutation instrument and
real per-frame attribution, with presentation wait separate from the work.
For the bench, measure its viewport, bounded controls and producer updates;
do not manufacture a DOM element for every part or face just to reuse L0b.
The separate 1,000 / 5,000 / 20,000-element mutation sweep remains a general
engine measurement alongside T2. Its completion does not gate the first bench.

The instrument landed 2026-09-15 as three pieces. The **seam** is two defaulted
methods on `DocumentSession<F>` in
`components/shared/document-session-api/src/session_engine.rs`:
`apply_host_mutations(&[HostMutation]) -> HostMutationReport` and
`element_ids_with_prefix(&str) -> Vec<String>`. A `HostMutation` names an
element by its `id` attribute and carries one of `SetAttribute`,
`RemoveAttribute`, `AppendChild`, `RemoveLastChild`; the report says how many
applied, how many missed, and how many elements the engine restyled. The Livery
lane implements both in `components/genet-documents/src/engines/livery.rs` over
the existing `LiveryDocument::mutate_dom`, so the batch reaches the same
`drain_mutations` / `IncrementalStyle` path a script would. It is a trait-level
seam rather than a Livery-specific method because a host that mutates a document
is not asking for anything lane-specific, and Cambium's Rootstock drives exactly
this shape against its own DOM today. It duplicates none of Cambium's logic:
Rootstock owns its DOM directly and does not route through a session.

The **driver** is Ortet's `--mutate '<id-prefix>:<op>[:<per-frame>]; …'`, with
`op` one of `style`, `class` or `child`, applied once per presented frame from
the second frame onward. Batches are a pure function of the frame index, so two
runs of one spec on one document issue identical mutations. The **instrument**
is `ports/ortet/src/timing.rs`: per-frame spans for host mutation,
`session.frame`, rasterization and compose (**work**) kept disjoint from
swapchain acquire and present (**presentation wait**), with everything else
(pump, accessibility publication, receipt conditions) reported as a named
residual rather than folded into either. `--timing-json <path>` writes every
frame plus medians and p95.

### Host mutation instrument receipt, 2026-09-15

Source: Genet `5ae30cad0ea` plus this change, uncommitted, in the
`t3-mutation-harness` worktree; netrender `3961aca91`; Rust 1.97.1; release
profile; Ryzen 9 7940HS / RTX 4060 Laptop, Windows 11, display scale 2.
Raw receipts, commands, executable hashes and the accounting re-derivation:
`Code/testing/genet/ortet-t3-20260915/`. The bounded-fixture runs were taken
with the final source's binary; the sweep cells were taken with an executable
differing only in two comments, recorded there rather than re-measured at 80
minutes a cell.

**Bounded fixture** — `ports/ortet/fixtures/bench_b_mutation.html`: one
`<custom-leaf>` viewport, eight controls, one caption. No DOM element per body
or face. 960x640 physical, 120 frames, summary over the 119 frames after the
cold first one. `--mutate 'ctl:style:4; ctl:child; view:style'` rewrites four
control styles, grows or shrinks one control subtree, and moves the viewport
element's own style, every frame.

| run | work median / p95 | present wait median / p95 | total median | `session.frame` median |
|---|---:|---:|---:|---:|
| no mutation | 0.86 / 1.12 ms | 5.01 / 5.22 ms | 5.911 ms | 0.02 ms |
| mutating | 2.54 / 3.61 ms | 2.91 / 3.42 ms | 5.830 ms | 0.93 ms |

713 mutations applied, 1 missed (the first `child` frame removes before
anything has been appended), 654 elements restyled. **The totals are the same
and the composition is not**: the mutation costs +1.68 ms of work per frame and
the presentation wait gives back 2.10 ms, for a 1.4 % difference in frame time.
A run reporting only frame time would have concluded the mutation was free,
which is the reason the plan asked for the split.

**Large-element sweep** — the L0b fixtures at
`Code/testing/wing/l0b/n<N>_b50_static.html`, mutating the `style` attribute of
a fixed 10 % of the body elements per frame (the faces carry no `id`, so the
fraction is of bodies; each body's change invalidates its own 50-face subtree).
960x640, warm-up one frame.

| elements | bodies | mutated/frame | frames | work median (mutating) | work median (control, no mutation) | status |
|---:|---:|---:|---:|---:|---:|---|
| 1,000 | 20 | 2 | 60 | 531.9 ms | 1.85 ms | measured |
| 5,000 | 100 | 10 | 60 | 9,820 ms | 4.49 ms | measured |
| 20,000 | 400 | 40 | 6 | 560,417 ms | not run | measured, 5 frames only |

Presentation wait in every sweep cell is 0.14–0.20 ms: at these costs the
document never keeps up with the display and never blocks on it, so the work
number is the whole story — the opposite regime from the bounded fixture.

The 20,000 cell took about 80 minutes of wall clock for six frames, so its
median and p95 (988,088 ms) are over five values and should be read as an
order of magnitude, not a stable percentile. Not run, deliberately: a 20,000
control, the 50,000 cells, and any `b200` variant. The receipt names each and
why.

**Finding the sweep produced.** A host mutation of two elements in a
1,000-element document costs 532 ms of work, against 1.85 ms for the same
document unmutated — 287x — and at 5,000 elements the ratio is about 2,200x.
`LiveryDocument::apply_dom_mutations` marks layout dirty document-wide and the
next `frame` rebuilds geometry for the whole document, so the per-frame cost of
*any* mutation is the document's full layout cost, including the quadratic
positioned-box term T2 owns. The instrument therefore prices mutation at
first-frame layout minus parse, and the sweep's shape is T2's shape. This is
not repaired here: T2 owns it, and `apply_dom_mutations`' own doc comment
already records the deliberate full-geometry rebuild.

L0b's byte-identical Boa scripted frames and missing wakes remain a finding
of that fixture and build (`Code/testing/wing/l0b_genet_element_ceiling.md`,
finding 3). This assessment did not diagnose it or rerun it. Ortet's native
O5 acceptance is recorded separately; the old fixture is not evidence that
all current native scripting is unable to present mutation.

**Genet seam done when:**

- A host-owned image in the existing leaf slot has correct content-box size,
  clipping and painter position among normal DOM siblings, including an
  overlapping translucent sibling and a half-opacity ancestor.
- Independent interior and adjacent edge probes cover border/padding,
  scrolling, rectangular overflow, authored origins and accumulated 2D
  translation, scale and rotation. The same geometry maps admitted pointer
  points back to the expected content location and rejects clipped/outside
  points and singular transforms.
- The existing native composition receipt and affected input/reftest guards
  are rerun at recorded source identities; relevant WPT movements are
  attributed with zero `pass -> anything else`. Prior acceptance is not
  substituted for this changed-source receipt.
- The engine-host mutation instrument reports actual work and separate
  presentation wait for the bounded fixture. The large-element sweep has
  its own results/status and remains open until measured.

The wing's bench closes only with the Mere producer lifecycle, same-identity
picking/accessibility and application update-cost receipts specified by its
canonical prerequisite section. Genet's seam acceptance alone does not close
that application gate.

### Bounded engine receipt, 2026-09-13

The source is `components/genet-livery` at Genet
`101d9e9ade8671564e723443d9f0498e899a33f1`.
`tests/host_content_geometry.rs` supplies literal interior and adjacent edge
coordinates independently of the query implementation, and evaluates emitted
paint commands through euclid separately from the query's matrix inversion.
Its ten tests cover pixel border/padding; own and ancestor scroll; nested
translation, scale, rotation and noncentral origins; padding-edge overflow;
rejection of a rotated AABB's empty corner; overlay ordering and half-opacity
scope; singular/nonfinite input; hidden ancestors; palette-resolved color;
and the plain retained leaf's zero clip/layer depth. These are CPU paint-list
and input checks, not native pixel readbacks or WPT conformance receipts.

Fresh focused checks, with `CARGO_TARGET_DIR=C:/Users/mark_/Code/targets/genet-bench-b`:

| Command after `cargo test --locked --offline -p genet-livery` | Result |
|---|---:|
| `--test host_content_geometry` | 10 passed |
| `--test interaction` | 25 passed |
| `--lib paint::` | 23 passed |
| `--lib hit_test` | 2 passed |
| `--lib stacking_paint_children` | 1 passed |

The host-content and paint filters were rerun after the final scope correction;
the other checks cover unchanged input code. `rustfmt` on the touched files and
`git diff --check -- components/genet-livery` pass. The ignored lockfile SHA256
is `005313842e13d744c738685595a2cf22007ad9636d23095c4487e783f94f69be`;
the host-content test executable SHA256 is
`54f85fb413ee98b802ee90af4374ed7d3287222bba4a2284789d95bf2bdc226e`.
The existing local `.cargo/config.toml` overrides for netrender, Boa, Vano and
Piccolo were used unchanged. No workspace/all-target, native Ortet, or WPT
rerun is claimed by these focused checks. The downstream native assembly
receipt below has its own source and acceptance scope.

### Downstream native consumer receipt, 2026-09-13

The canonical wing plan's **Bench B** receipt accepts the bounded native
assembly on a development-path build: Genet
`101d9e9ade8671564e723443d9f0498e899a33f1`, netrender
`3961aca919f707ab09a786379eb4ce8bb121258e`, and Cambium's producer source,
which was uncommitted at that run. Compact receipts and executable/source
identities live at
`isometry/mesocosm/testing/bench/receipts/2026-09-13/`; full captures live at
`Code/testing/specimen-bench/native-final/`.

A separate Mesocosm release workspace check with all features and targets
passes without development path configuration, using Mere
`4f4de1d05ec99461f7fa3cdc4e514e904a999213` and the Genet/netrender commits
above. From `isometry/mesocosm`, Rust 1.97.1 exits 0 for:

```text
cargo check --release --offline --workspace --all-features --all-targets -j 3
```

The native captures remain the development-path build's receipt;
the committed-dependency check does not relabel or repeat that native run.

The native acceptance passes **167 frames and 15 captures at actual output
scale 2**. Each tint comparison changes 102,660 body pixels while 1,128,060
sampled background pixels remain fixed. Decorative CSS preserves all 1,230,720
sampled content pixels and upload counters. Real host input passes transformed
centre clicks, UI zoom 1.25, DOM overlay priority, resize, removal/recreation
and selection expiry on preview replacement. The authored 7-degree/0.9
transform is calculated independently for the native click fixture.

The habitat run assembles ten bodies and 343 parts beside the terrain. It
establishes native assembly; exact body/terrain occlusion remains covered by
Bench A's pixel fixtures, and literal transform/clip edges by the Genet and
Cambium focused fixtures. The deliberate failure exits 1 with `ok: false` and
two fresh captures. These are bounded consumer checks, distinct from an Ortet
or WPT rerun. They preserve the 61-check Genet receipt above and leave broader
T3 mutation/measurement acceptance, T1, T2 and T4 open.

---

## T4. Retained fragments inside layer scopes

**Status, 2026-09-16:** implemented and measured in netrender, committed as
`06f3a12f4`. All four done-conditions below hold on this host. The contained
shape recorded under **The change** was not taken: the run's sub-scene is a
netrender op list lowered only at flush, not a `vello::Scene`, so layers now
push and pop on the master scene instead. The 2026-09-16 progress entry holds
the receipt. Nothing ran through Genet.

**Finding (2026-09-12).** Genet's paint-list translator lowers every
`PushClip` to a netrender layer (`emit_push_clip` in
`netrender/paint_list_render/src/emit.rs`); netrender has no standalone clip
op, only `PushLayer(SceneLayer { clip, alpha, blend_mode, filters, .. })`.
The retained path in `netrender/netrender/src/vello_tile_rasterizer/retained.rs`
appends a registered fragment's lowered scene to the master only while
`layer_depth == 0`; inside any open layer it takes the warned fallback and
inlines the fragment un-retained, re-encoding it every frame. The original
body-fragment proposal put each fragment inside a content-box clip, with
ancestor clips besides, so that proposal took the un-retained path through
normal document composition. That is L0a's flat live path, ten to
fifteen thousand rectangles under a 16 ms frame, not its fragment path at two
hundred thousand. L0a did not see this because the probe bypasses Livery and
places fragments at the scene's top level.

The same fallback applies to planar retained content under `opacity`,
`clip-path`, `mask-image` or `filter` layers. T3's scene image does not use a
retained fragment, so this repair is not a prerequisite for its viewport.
Effects on instances inside that viewport belong to its producer and the
application's supported style mapping.

**The change.** Netrender's retained path honours a placement inside an open
layer without re-encoding. The fallback's own comment names the obstacle: the
open layer lives in the run's sub-scene, so an append to the master would
escape it. Appending the retained lowered scene into the run's open sub-scene
instead, with the placement affine, is the contained shape; the execution
graph plan's owner may prefer another. This plan records the requirement and
the receipt; the code lands in netrender under
`netrender/netrender-notes/2026-09-04_wgpu_execution_graph_plan.md`, which
this document does not amend.

**Measured 2026-09-12** by the wing's L0c
(`Code/testing/wing/l0c_three_paths.md`, finding 3): one clip layer around a
scene of retained fragments costs 3.3x at 200 bodies (6.8 to 22.6 ms) and
6.7x at 1000 bodies (13.7 to 91.9 ms, 181k rectangles), with
`fragment_lower_count` at zero throughout because the fallback never
retains. The same receipt prices the re-lower a turning part needs at about
1.1 µs per rectangle, CPU-bound, and records resident geometry with depth at
2.6 to 3.8 ms in the thousand-body cells. These are the probe's workloads,
not end-to-end Genet costs or a correctness proof for every case; its
articulated residual, quantized-yaw comparison and sprite-cache qualification
remain recorded in the wing's L0c assessment. Nothing ran through Genet.

**Ruled 2026-09-13 (Mark):** T3 embeds the shared depth-owning scene viewport
described above. Its bounded native document composition, host lifecycle and
interaction now have the separate T3 consumer receipt above. T4 retains its
independent planar-content scope; the measured netrender fallback does not
make T4 a viewport gate.

**Done when:** a fragment placed inside a rect-clip layer, an alpha layer and
a filter layer each matches an independently expanded reference under the
same layer in a headless readback; `fragment_lower_count` stays flat across
placement-only frames in all three cases; the layer-scope warning no longer
fires on that planar fixture; and the L0a probe rerun with every body wrapped
in a content-box clip layer holds its 200,000-rectangle cell within the
2026-09-11 frame time plus a stated tolerance. Record this separately from
T3's ordinary image-composition receipt.

---

## Deliberate exclusions

- A perspective-correct rasterization path. Vello is affine; a perspective
  divide that cannot be expressed affinely is reported as a gap.
- Extending the browsing-context tree, child-document loading, or anything else
  the iframes plan owns. T3 reuses the existing custom-leaf slot, not a new
  browsing context.
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
  `paint_list_render/src/emit.rs` and `vello_rasterizer/mod.rs`. Initially
  unmeasured; L0c subsequently measured the layer fallback as recorded in T4.
- **2026-09-13** — native canvas composition already uses an ordinary
  `SceneImage`; T3's original separate fragment-slot proposal duplicates an
  available seam. At assessment, `paint.rs` recorded the custom-leaf slot at the outer
  fragment origin, and `layout/hit_testing.rs` had no accumulated transform
  inversion. Content-box geometry and shared 2D paint/input mapping are the
  bounded Genet prerequisites. Cambium's producer/style hooks remain Mere's.
- **2026-09-13** — the bounded T3 implementation closes those engine placement
  gaps and preserves ancestor scroll when stacking roots flatten out of a
  non-context scroll container. An unconditional content-box clip would have
  forced legacy retained leaves into T4's fallback; it is deliberately absent,
  with a zero-layer regression. The existing percentage-padding approximation
  remains explicit. Automated evidence is recorded in T3 above.

- **2026-09-15** — host mutation of a Livery document costs a full-document
  geometry rebuild, whatever the mutation's size.
  `LiveryDocument::apply_dom_mutations` marks layout dirty document-wide and
  the next `frame` rebuilds; measured at 532 ms of work per frame for two
  mutated elements in the 1,000-element L0b fixture against 1.85 ms unmutated,
  about 9.8 s against 4.5 ms at 5,000, and about 560 s at 20,000. The
  per-frame mutation cost is
  therefore first-frame layout minus parse, and the sweep's growth is T2's
  growth. Repairing it is T2's lane, not this instrument's; the retained style
  plane is already incremental (`RestyleStats.restyled_elements` reports only
  the mutated subtrees), so the cost is geometry, not cascade.
- **2026-09-15** — on the bounded fixture the mutation is invisible in frame
  time and obvious in the split: 0.86 ms work / 5.01 ms wait unmutated against
  2.54 ms work / 2.91 ms wait mutating, with total medians 5.911 ms and
  5.830 ms. The presentation wait gives back what the work took, which is why
  T3 asked for the two numbers separately rather than a frame time.

- **2026-09-15** — T2's exponent was four whole-document passes and two
  whole-vector allocations per positioned box, exactly as hypothesis 1 read it.
  Removing them takes the L0b grid's fitted growth from n^2.156 to n^1.168 and
  the 50,000-element first frame from 1,172 s to 7.68 s, with every cell's
  rendered digest unchanged and a status-identical WPT census.
- **2026-09-15** — with the exponent gone, the per-element constant is the
  **cascade**, not parse and not `ComputedValues`' size on its own: at 50,000
  elements the style phase is 2.03 s of a 7.68 s frame and `IncrementalStyle::update`
  is 2.25 s of the measured style work, because every rule in the style set is
  matched against every element with no selector bucketing. This sharpens
  hypothesis 3: the 160-field struct is real but secondary, and the 1-second
  bound needs a selector index, not a cheaper clone.
- **2026-09-15** — the `css/css-transforms` founding count in ledger row 15
  (143 / 2,296, 2026-09-07) is not comparable to anything the current harness
  measures. The same directory reads **291 / 5,441** at this lane's base
  commit `b3250d3d91b`, on the same host and the same invocation. T1 diffs
  against 291 / 5,441; the ledger keeps 143 / 2,296 as the founding count it
  is.
- **2026-09-15** — a property sitting in the catalog's `[[unimplemented]]`
  table round-trips its declaration text verbatim, which makes WPT parsing
  tests pass by accident. Nine of `perspective-origin-valid`'s ten passing
  subtests were keyword position forms passing that way; implementing the
  property loses them to Livery's origin canonicalization and gains the
  canonical two-component form. The same shape explains
  `scale-interpolation`: multi-component `scale` was invalid, so the element
  under test and the expectation element both computed to `none` and every
  comparison passed vacuously. **Implementing a property can lower a subtest
  count without regressing anything.**
- **2026-09-15** — netrender composes the pushed 4x4s and projects the
  product, so `transform-style: flat` is not free: without an explicit
  projection at the boundary, nested 3D transforms compose in 3D all the way
  to the root. Two nested `rotateX(45deg)` collapsed the box to nothing
  (`transform-flattening-001`). Projecting at each `flat` boundary is exact,
  because projection composes.
- **2026-09-15** — Rodrigues rotation about a cardinal axis is not exact in
  f32: the unrotated diagonal cell is `(1 - cos) + cos`. Every `rotate()` then
  serialized as a `matrix3d()`. Cardinal axes are written out directly.
- **2026-09-15** — `bake_facing` rotates voxel *positions* and always stamps
  an axis-aligned cube, so a true rotation of the unit box lands one cell off
  at every non-zero facing. The wing fixture compensates explicitly; without
  the compensation the two silhouettes are the same hexagon, offset by one
  voxel unit (69.05% agreement against 98.73%). This is a convention of the
  bake, recorded here for the appearance crate's owner, not a defect.
- **2026-09-15** — the CSS Animations clock is not sampled in either WPT
  harness mode. Every `scale-interpolation` subtest that newly passes is at
  progress 0 or 1, and `rotate-animation-with-will-change-transform-001`
  renders its static reference rotated and its animated subject not. Adjacent
  to this lane; named, not fixed.
- **2026-09-15** — genet paints `position: absolute; z-index: auto` boxes in
  the normal-flow walk rather than in the positioned-descendants phase. A
  `translateZ` face correctly establishing a stacking context makes that
  visible (`3d-rendering-context-and-z-ordering-001`/`-002`). Next door to
  this lane.
- **2026-09-15** — the layout phase's own per-element cost still grows between
  20,000 and 50,000 elements (30.3 to 57.9 µs/element) while style, paint and
  parse stay flat over the same step. That is the whole of T2's residual 1.17
  exponent and it is unattributed; a next probe reads it from the spans rather
  than from a fixture.
- **2026-09-16** — netrender loses element-filter content from the second
  frame when a reused `Renderer` draws a filtered layer whose content moves.
  The filter passes make a fresh GPU texture under the same sentinel
  `ImageKey` each frame while cached tile scenes keep the previous image
  handle, so the layer renders blank. It reproduces with no retained fragment
  in the scene, so it is independent of T4 and was not repaired there;
  netrender's comment on `preprocess_filters` names a tile-cache-aware key and
  handle reuse scheme as the fix. Genet content under an animating CSS
  `filter` would be exposed; this was found in netrender tests, not observed
  through Genet. Recorded for netrender's owner.

## Progress

- **2026-09-12** — founded. No lane started.
- **2026-09-12** — T4 added by Mark's ruling after the layer-scope finding.
  No lane started.
- **2026-09-12** — T4's cost measured by the wing's L0c (3.3x at 200
  bodies, 6.7x at 1000); L0c's verdict, resident geometry with depth as the
  body unit, is recorded against T3 above for Mark's ruling on what the
  replaced element carries.
- **2026-09-13** — Mark ruled: T3 embeds a depth-owning scene viewport,
  parts as instances inside it, style-to-instance mapping as the bridge to
  build and measure. Recorded under T3.
- **2026-09-13** — prerequisite assessment complete; T3 and the dependency
  summary reconciled with the existing image route and the wing's canonical
  specimen bench prerequisites. T1, T2 and T4 remain independent work.
  Documentation only: implementation not started and prior native acceptance
  not freshly rerun.
- **2026-09-13** — T3's bounded Genet content-box, shared 2D paint/input and
  typed used-color implementation passed 61 focused checks. At this source
  checkpoint native Bench B integration remained pending. Prior native canvas
  receipts were not rerun.
- **2026-09-13** — downstream Bench B native assembly accepted on the recorded
  development-path source, with 167 frames/15 captures and independent tint,
  decorative-CSS and input checks. The qualified consumer receipt is above;
  a separate committed-dependency Mesocosm workspace check passes. Broader T3
  acceptance and mutation measurement, and independent T1/T2/T4 work remain
  open. This is not an Ortet or WPT rerun.
- **2026-09-15** — T3's host mutation instrument built and measured. A
  trait-level `apply_host_mutations` / `element_ids_with_prefix` seam on
  `DocumentSession`, implemented by the Livery lane over the existing
  `mutate_dom` path; Ortet's `--mutate` driver; and per-frame phase timing that
  keeps actual work apart from presentation wait, written to a JSON receipt by
  `--timing-json`. Nine focused tests added (four seam, five timing/accounting)
  and the existing Livery and Ortet suites rerun; the accounting is also
  re-derived over all 480 measured frames. The bounded fixture and the 1,000
  1,000, 5,000 and 20,000-element sweep cells are measured, the last on five
  frames only and with its reservations and un-run cells named in the receipt;
  50,000 was not attempted. Script-driven mutation remains a deliberate
  exclusion and was not touched. Native composition rerun and WPT attribution
  for broader T3 acceptance remain open, as do T1, T2 and T4.
- **2026-09-15** — T2 implemented and measured. A `children` index and a
  deferred aggregate-overflow rebuild in `buckram`'s `FragmentTree`, the same
  bound applied to the relative, sticky, retained-root and incremental-query
  paths and to `TextFrame::translate_subtree`, and two per-element style costs
  removed in `genet-livery` and `livery`. A `--phase-timing` flag on Ortet over
  a new `genet_livery::phase` recorder reports parse, style, layout and paint
  for one frame and wrote this lane's whole attribution without a fixture
  variant. The L0b grid is rerun on the same host: fitted exponent 2.156 →
  1.168, the 50,000-element first frame 1,172.21 s → 7.68 s, every digest
  unchanged. **The 1-second bound at 50,000 is not reached**; 7.68 s is, and the
  rationale is recorded under T2. Five focused fragment-tree tests, three phase
  tests and two host-instrument tests added; buckram, genet-livery, livery,
  ortet and genet-documents suites rerun green. WPT census over
  `css/css-position`, `css/CSS2/abspos`, `css/CSS2/positioning`,
  `css/CSS2/normal-flow`, `css/css-transforms` and `css/css-values`, before and
  after on the same harness sources: zero transitions of any kind. Committed as `b3250d3d91b`. T1, T4 and the broader T3 acceptance remain open, and so does T2's
  per-element constant.
- **2026-09-15** — T1 implemented and measured. CSS Transforms Level 2 in
  Livery: a `Matrix3D` composer under the existing `Matrix2D`, the Level 2
  transform functions, the `translate` / `perspective` / `perspective-origin`
  / `transform-style` / `backface-visibility` properties moved out of the
  catalog's unimplemented table, and axis and three-component forms for the
  individual `rotate` and `scale`. In genet-livery the accumulated 4x4 reaches
  the `TransformSpec` paint and hit-test helpers unchanged, a `flat` boundary
  projects, a `preserve-3d` context z-sorts its participating boxes,
  `backface-visibility: hidden` culls against the element's own 3D rendering
  context, and a perspective divide is recorded as a named `TransformGap`
  before the orthographic approximation is painted. Seventeen focused tests
  added (nine livery, eight genet-livery); livery, genet-livery,
  genet-documents and ortet suites rerun green (206 / 527 / 51 / 32 passed,
  none failed). `css/css-transforms` re-measured on this host at the base
  commit and on the lane: testharness 291 -> 650 subtests with zero
  `pass -> anything else` and two attributed subtest losses; reftest 534 ->
  600 passing with 70 fixed and 4 attributed new failures. Wing fixture
  silhouette against `isometer-mesh`'s bake: 98.73% agreement, no pixel more
  than one pixel out. Yaw and depth-order headless readbacks pass 13 of 13
  assertions. Committed as `d61978378c1`. T4 and the broader T3 acceptance remain open, and
  so does T2's per-element constant.
- **2026-09-16** — T4 implemented and measured in netrender on branch
  `t4-retained-in-layers` from `3961aca91`, committed to netrender main as
  `06f3a12f4`. The contained shape this plan recorded, appending into the
  run's open sub-scene, was not taken: that sub-scene is a netrender op list
  lowered only at flush, not a `vello::Scene`. Instead `PushLayer` and
  `PopLayer` flush the pending run and push or pop on the master
  `vello::Scene`, so a placement appends its cached lowering inside the open
  layer, the move `compose_master` already makes per tile. Seven headless GPU
  tests in `netrender/tests/pe4b_retained_in_layers.rs`: rect-clip, alpha and
  element-filter layers each read back byte-identical to a second `Renderer`
  inlining the same content under the same layer, each with a non-vacuity
  control; nested layers hold painter order around a placement;
  `fragment_lower_count` stays at one across four placement-only frames in all
  three kinds; a per-body clip fixture emits no fragment-path warning. The L0a
  probe was copied out of mere unmodified and given a mode that wraps every
  body in its own content-box clip layer, the shape `emit_push_clip` produces.
  On an RTX 4060 Laptop GPU over three settled runs, the 1,000-body,
  200,000-rectangle cell measures 40.46 / 44.26 / 52.04 ms before and
  8.80 / 9.56 / 9.37 ms after; the 200-body cell 9.86 / 10.41 / 14.58 ms
  before and 3.78 / 3.98 / 4.03 ms after. `fragment_lower_count` goes from
  zero to one per body. The tolerance, fixed before the after numbers were
  read, is the same run's unclipped cell plus 15%; measured ratios are 1.00x,
  1.07x and 1.02x at 1,000 bodies and 1.07x, 1.06x and 1.05x at 200. The
  clipped median, 9.37 ms, is also under the 2026-09-11 absolute of 9.96 ms.
  This host's own spread on the unclipped control was 8.34 to 15.45 ms, so the
  in-run ratio is the primary statement. A first cold run is kept in the
  receipts and excluded; its 1,000-body clipped cell also passes the
  tolerance, at 0.86x. `cargo test --locked --offline -p netrender` passes 272
  with none failing, in the lane and again in an independent rerun on the
  final tree. Nested fragments and unregistered ids still do not retain and
  still warn. The paint-list translator was read, not changed. Receipts:
  `Code/testing/netrender/t4_20260916/results.md`; netrender's record is in
  `netrender-notes/2026-09-04_wgpu_execution_graph_plan.md`. T1's named gaps,
  T2's per-element constant and the broader T3 acceptance remain open.
