# Standards-to-features ledger

**Date:** 2026-09-07
**Status:** founded, by Mark's ruling of 2026-09-07 on mere's
[lighter recall brief](../../mere/design_docs/eidetic_docs/research/2026-09-07_lighter_recall_and_standards_ledger_brief.md)
(cited by path; it is the seed and the argument). Living ledger, not a plan;
rows are added as lanes open and as consumers name a dependency.

**Baseline:** the [web platform WPT census](2026-09-06_web_platform_wpt_census.md).
Every census-measured row carries that run's subtests-passed-over-total, so a
later census diffs against it.

## Purpose

*Develop and adhere to these standards, and we unlock these features.* The
left three columns are genet facts: the standard, where genet stands, and
what adhering means in this engine. The right column is what mere and the
products get for it. The ledger orders the grind by payoff rather than by
WPT directory order, and it gives a mere plan one place to say "we depend on
row N" instead of re-deriving the engine's state.

Rules for a row:

- The census count is copied, never estimated. A row with no WPT directory
  says so.
- "Adhering means" names the engine surface, not a percentage.
- "Unlocks" names a consumer that exists or a plan that names it. A feature
  nobody has asked for is not a row.
- When a lane closes a row's gap, the row keeps its founding count and gains
  a dated closing count; the census is the authority for the current number.

## Ledger

| # | Standard | Census 2026-09-06 | Adhering means | Unlocks |
|---|---|---|---|---|
| 1 | Unicode segmentation, UAX #29 | no WPT directory; `Intl.Segmenter` under `intl` | one conformant word, sentence and grapheme segmenter as a genet component (ruled to found, 2026-09-07) | mere's search tokenizer, find-in-page word mode, `esp` lexical features, reading time, selection by word |
| 2 | Selection API | 0 / 280 | `getSelection`, ranges over the layout DOM | web clip as a real gesture, quote with provenance, find-in-page highlighting |
| 3 | Accessibility tree, ARIA and AccName | not measured (no testharness lane) | roles and names computed per spec | field-weighted recall index from the reader's model, agent-driven pages, the inspector as a test oracle |
| 4 | Intersection Observer | 0 / 104 | viewport intersection callbacks | dwell and "interesting interaction" for frecency, lazy media, attention receipts for the trail pane |
| 5 | High Resolution Time, Performance Timeline | 0 / 14, 0 / 73 | monotonic clocks, performance entries | `dwell_ms` filled honestly, page-load receipts, the timing half of the capture record |
| 6 | Mutation Observer | 9 files fail on the missing global | DOM change notifications | re-extract body text on SPA navigation so the index tracks what was read |
| 7 | URL | 351 / 519 | WHATWG parsing and canonicalization | page identity starts from canonical URLs; browser-history import matches ours |
| 8 | Encoding | 7,109 / 1,329,450 (legacy multibyte slices dominate) | labels and decoders | history and bookmark import from every browser's export |
| 9 | IndexedDB, Storage | 5 / 880, 0 / 75 | the storage APIs and quota | muniment's OPFS lane hosted by our own engine; browsing memory in the browser |
| 10 | Web Crypto | 3 / 199 (86 files miss `crypto`) | SubtleCrypto over our primitives | pack signing and sealing in the browser lane; personae in a web host |
| 11 | Workers | 17 / 574 | dedicated workers, message passing | indexing and embedding off the document thread; `esp` in the browser |
| 12 | Web Messaging, BroadcastChannel | 49 / 209 | channels across contexts | graphshell's remote-projection wire planes hosted in-page |
| 13 | JSON-LD and microdata in documents | no WPT directory | parse and expose `application/ld+json` and microdata | `mere-linked-data` ingest straight from visited pages |
| 14 | Custom Elements, Shadow DOM | 2,041 / 3,674, 18 / 8,654 | the component model | Cambium widgets hosted inside web documents |
| 15 | CSS Transforms 2 | 143 / 2,296 — `css/css-transforms`, measured 2026-09-07, not in the 2026-09-06 census ([harness repair plan](2026-09-07_wpt_harness_repair_plan.md) gate F6, its first measurement) | Livery parses, computes and serializes `matrix3d`, `translate3d`, `rotate3d`, `scale3d`, `perspective` / `perspective-origin`, `transform-style`, `backface-visibility`, 3D `transform-origin` and the individual `translate` / `rotate` / `scale` properties, and lowers each element to netrender's column-major 4x4 `Transform` with z-sorting inside a 3D rendering context | the games wing's live voxel faces and per-part rotation: `isometry/mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md` L1 and L5, founded in genet by the [CSS 3D transforms and first-frame plan](2026-09-12_css_3d_transforms_and_first_frame_plan.md) T1 |
| 16 | First-frame scaling for many positioned boxes | no WPT directory: this is an engine cost, not a conformance surface. The measurement of record is `Code/testing/wing/l0b_genet_element_ceiling.md` (first frame 4.6 to 4.9 s at 5,000 elements, 60 to 132 s at 20,000, 15 to 17 min at 50,000) | linear rather than quadratic first-frame cost in the number of positioned boxes — the per-box whole-tree passes in `FragmentTree::resize_leaf` / `translate_subtree` / `recompute_overflow` removed — plus parse, style, layout and paint spans behind a flag so cost is attributable without fixture variants | the games wing's live-all-the-time default: `isometry/mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md` L0 verdict and L5, founded in genet by the [CSS 3D transforms and first-frame plan](2026-09-12_css_3d_transforms_and_first_frame_plan.md) T2 and T3 |

Candidates a lane should add when it opens: `editing` (contenteditable for
a writing product), `streams` (extraction starts before the page finishes),
`service-workers` (offline products).

## Consumers on record

| Row | Consumer | Where it is named |
|---|---|---|
| 1, 4, 5, 6, 7 | mere trail recall, W6 | `mere/design_docs/mere_docs/implementation_strategy/2026-08-12_search_surface_wiring_plan.md` |
| 2 | mere capture plan C3 (web clip) | `mere/design_docs/mere_docs/implementation_strategy/2026-06-26_capture_provenance_consent_plan.md` |
| 9 | muniment OPFS lane | `mere/design_docs/eidetic_docs/implementation_strategy/2026-08-22_redb_opfs_feasibility_plan.md` |
| 15, 16 | the games wing's orthographic voxel presentation, lanes L1 and L5 and the L0 verdict | `isometry/mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md` |
