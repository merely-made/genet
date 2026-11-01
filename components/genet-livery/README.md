# Genet Livery

`genet-livery` is Genet's integration path for the clean-room Livery CSS
engine. It adapts any `LayoutDom` to `cadency`'s `Element` trait, resolves a
concrete Livery style plane, lays the bounded Cambium lane out, and emits box
backgrounds, physical borders, and shared inline text runs through the neutral
`PaintList` API without importing Stylo. Text shaping uses the
MIT/Apache Parley crate directly; the MPL `netrender_text` adapter is not part
of this path.

Fullweb documents run on this path as well; the `genet-layout` and Stylo route
was deleted on 2026-08-21.

Retained motion frames can reuse laid-out boxes and shaped text when only
2D transforms and numeric z-index change on already transformed absolute or
fixed boxes. Paint and hit testing read the updated styles. This admission
requires style-only mutation batches and stable geometry, generated content,
resources and viewport; context establishment changes and active animations
keep the normal layout path. It does not cache raster output or bypass CSS
clipping and stacking order.

Incremental restyle batches share immutable HTML hints and shadow-scope facts,
and enumerate each relevant sibling group once for root ordinals and selector
identity storage. Root coalescing uses ancestor membership rather than comparing
every pair of hints. Existing conservative invalidation rules and per-element
cascades are unchanged. The [counted-work regression](tests/invalidation_work.rs)
checks sibling-batch scaling alongside full-cascade equivalence, mixed mutations,
scoped styles, and HTML hints; it makes no browser timing claim.

The retained `LiveryDocument` owns Parley's font database, shaping scratch
space, stable font resources, and a cached paint frame. Consecutive text and
inline-element children shape together, sharing line breaks, baselines, style
spans, and collapsed whitespace. Parley's positioned output also supplies the
paint geometry for inline elements: wrapped spans receive one fragment per
line, and `inline-block` children occupy atomic space in that line.
`genet-documents` can register this path as the opt-in `genet.livery` static
session rung. Flex and grid containers lower their bounded direction, track,
gap, alignment, order, and placement values into Taffy. Box shadows lower into
the neutral shadow primitive before backgrounds.

## Retained and parallel text shaping

`LiveryDocument::set_text_shaping_options` configures `TextShapingOptions`.
The initial defaults retain up to 512 entries and 4 MiB of key text; setting
either limit to zero disables retention. These limits bound entry count and
key text, not the Parley layouts or total heap. Oversized groups are shaped
but not retained. These are configurable starting settings, not a demonstrated
optimum for every host. `text_system().shaping_stats()` reports cumulative
builds, hits, late misses and pool work, plus current retention gauges. Optional
timings are disabled by default (`record_timings: false`).

Shaping is serial by default. On native targets, enable `parallel-text` and
supply an `Arc<rayon::ThreadPool>` with `set_text_shaping_pool`; the global
pool is never selected implicitly. Initial admission requires at least eight
unique misses and 16 KiB of text, both configurable. Ready inputs are batched
before their measurement consumers; inputs whose geometry is not ready can
shape synchronously later and are counted as late misses. One long paragraph
is one group and is not split across workers. Calls join before results publish.

Bounded measurements show mixed results. With a 4,096-entry cache, resize of
2,048 groups reused all keys and retained serial layout improved over the
original in two 30-sample cohorts. A one-key edit rebuilt one key. The default
512-entry cache did not hold that working set. In three cold cohorts, pooled
layout improved over the new retained serial implementation, but both were
slower than the original layout pipeline. A 129 KB long-edit process peaked at
194.5 MiB retained serial versus 86.4 MiB original; these peaks include font
and serialization work. Small and edit cases met the specified empirical noise
band, but 30 samples and different process counts in the independent control
leave p95 uncertainty. The long edit used no pool jobs. These CPU measurements
are not GPU or headed runtime evidence. See the [qualification receipt](../../design_docs/archive_docs/receipts/2026-11-01_retained_parallel_shaping_plan_receipts.md).

## Explicit host generic resources

A browser does not lend Parley its installed system fonts. Registering a face
under a named family called `sans-serif` does not bind the CSS generic.
`TextSystem::register_generic_font_bytes(bytes, generic)` explicitly binds the
host-selected face to `sans-serif`, `serif`, `monospace` or `system-ui`. It
returns the registered-family count or `GenericFontError::InvalidGeneric` /
`InvalidFont`; refusal preserves the existing generic mapping. Authored family
requests, weights, sizes and the cascade remain authoritative. This operation
does not install a global or system fallback.

`LiveryDocument::replace_generic_font_resources` retains the complete host
ledger and validates a replacement batch before admission. A bad late entry
cannot replace earlier bindings. Ordinary font-resource and stylesheet-driven
text-context rebuilds reapply the admitted ledger; removing a binding rebuilds
from the remaining resources. The glyph IDs and exact selected face bytes still
travel together in the normal paint font side-table.

Production hosts choose covering, licensed faces appropriate to each requested
role. A face mapped to `monospace` must be selected and qualified for fixed-width
use; assigning any available UI face to that generic proves resource delivery,
not monospace fidelity. The same applies to serif and locale/script coverage.
Mere's shared recovery delivery fixture intentionally assigns its checked-in
Roboto face to all four roles. That choice is confined to that fixture and is
not the production face-selection policy.

The no-system-font control
`text::tests::explicit_generic_font_shapes_without_system_and_invalid_replace_preserves_face`
checks actual shaping and selected bytes. The retained document control
`document::tests::generic_font_ledger_survives_rebuild_and_invalid_batch_is_atomic`
checks paint output after resource/viewport reconstruction. These are CPU font
proofs; host presentation and full composed pixels have separate receipts.

## Retained layout and paint

Retained sessions first use Taffy to resolve atomic `inline-block` sizes, then
represent each consecutive inline group as one Parley-measured Taffy leaf. The
same shared line layout therefore drives paint geometry and parent block
height, including mixed text, styled spans, and atomic boxes. Inline padding
and border edges enter Parley as zero-height atoms, consume horizontal advance,
paint vertical overflow without changing line height, and use default slice
semantics across wrapped fragments. Parley's visual item order is retained as
one paint stream per inline group, including bidi runs, while the group's
decorations paint first. Block containers with non-visible overflow emit
axis-aware padding-box clips around their descendants; nested clips balance and
intersect through the neutral paint-list stack. Within each numeric or document
stacking context, positioned block and inline-level elements with numeric
`z-index` paint in stable negative, normal-flow, then nonnegative order while
each reordered subtree stays atomic. Numeric stacking roots flatten through
intervening ancestors into their nearest context, replaying any ancestor
overflow clips around the extracted subtree. Shaped inline commands retain
their ownership so positioned spans and atomic inline-blocks reorder without
losing shared line geometry or bidi visual order. Opacity values below one
establish an atomic level-zero context and wrap the whole subtree in a neutral
compositing layer. Non-`none` transforms on block and inline-block boxes also
establish an atomic level-zero context and emit a neutral coordinate-space
scope around the subtree. The first transform grammar supports 2D translate,
scale, and rotate functions composed around the default center origin; ordinary
inline spans remain non-transformable. The current property ratchet also
lowers inset and min/max geometry into Taffy, carries corner radii through
neutral border commands, applies Parley text alignment and spacing, and keeps
hidden boxes in layout while suppressing their paint. The retained document
now supplies bounded viewport scrolling, pointer-events-aware hit testing,
retained link rectangles, fragment scrolling, and focus-state routing. A
host-driven opacity clock can invalidate and repaint intermediate frames through
the same retained session. Bounded `transition-property`/`transition-duration`
metadata also starts opacity, background-color, text-color, and border-top-color/
border-bottom-color
transitions from that clock; `transition: all` and explicit two- and
three-property lists interpolate those paint values together on the same clock.
Two-stop
linear-gradient backgrounds paint as an ordered neutral layer over the color
fill and share the rounded clip. Raster `data:` background URLs now lower into
the neutral image side-table and stretch to the element box. Nested scroll
containers route wheel deltas into their retained offsets, chain at their
boundaries to the viewport, and replay descendant paint through transforms.
Bounded `@keyframes` declarations animate opacity through the same retained
clock, with linear, ease, ease-in, ease-out, and ease-in-out timing functions.
Host-resolved local image bytes use the same neutral image side-table, with
bounded intrinsic tiling and position/repeat modes. Replaced `<img>` elements
now use intrinsic data/local dimensions, preserve their aspect ratio under a
bounded CSS width or height, and paint through the neutral image side-table.
The session consumes host-supplied image bytes, including remote URLs resolved
by `genet-documents`; fetch policy and caching remain outside the engine.
`genet.livery` therefore stays an explicit pin rather than the default static
route.
