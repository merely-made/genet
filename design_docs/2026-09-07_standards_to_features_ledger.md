# Standards-to-features ledger

**Date:** 2026-09-07
**Status:** founded, by Mark's ruling of 2026-09-07 on mere's
[lighter recall brief](../../mere/design_docs/eidetic_docs/research/2026-09-07_lighter_recall_and_standards_ledger_brief.md)
(cited by path; it is the seed and the argument). Living ledger, not a plan;
rows are added as lanes open and as consumers name a dependency.

**Baseline:** the [web platform WPT census](2026-09-06_web_platform_wpt_census.md).
Every census-measured row carries that run's subtests-passed-over-total, so a
later census diffs against it. These are historical baseline counts, not current
implementation status. See the dated reconciliation below before assigning work.

**Backend ownership and naming:** Vano is our fork of Nova, maintained and
available to improve or change as needed for the stack. It is Genet's primary
backend, pinned in
[`script-engine-nova/Cargo.toml`](../components/script-engine-nova/Cargo.toml).
The crate, `nova_vm`, `NovaEngine`, and `--engine nova` retain their technical
names; these names do not imply that the dependency is unmodified upstream Nova.
Older receipts labelled Nova must be read with their recorded dependency
revision; new status prose calls the current backend Vano.

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
  a dated receipt; a lane receipt does not imply full conformance or a new
  whole-platform census. Counts always retain their date, engine and scope.

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

## Implementation reconciliation — 2026-09-26

Documentation reconciliation only: no tests or census were rerun. The numeric
receipts below are Boa disk-mode measurements from the linked lane plans.
Their denominators grew as more tests could execute; they are not percentages
of implementation completeness. Focused coverage on both backends is separate
evidence from these WPT counts.

| Row | Recorded implementation and receipt | Remaining qualification / next proof |
|---|---|---|
| 2, Selection API | [Selection/Range](2026-09-07_selection_range_plan.md) landed 2026-09-07: selection **28,582/33,621**, 12 all-pass files; live ranges, Livery geometry and selection paint projection. Subsequent DOM node work removed additional range blockers. | Exercise the consumer's gesture-to-clip path; retain the plan's selection movement, scheduling and geometry qualifications. The founding 0/280 does not describe the landed surface. |
| 4, Intersection Observer | Founding census **0/104**; this reconciliation establishes no later implementation receipt. | Scope viewport geometry and delivery against a named consumer. Intersection evidence alone does not establish dwell or attention. |
| 6, Mutation Observer | [Observer implementation](2026-09-07_mutation_observer_plan.md) landed 2026-09-07, initially **+495** subtest passes. Range and DOM node work closed further failures; [parser/script interleaving](2026-09-08_parser_script_interleaving_plan.md) added parser-driven observation on 2026-09-08. | Verify the recall consumer's extraction refresh path. Missing-global census failures are historical, not the present implementation state. |
| 11, Workers | [Dedicated Worker](2026-09-07_worker_plan.md) landed 2026-09-07: **321/967**, 74 all-pass files. Separate-runtime thread, cross-agent messaging, both-backend automated tests and bounded native headed delivery are recorded. | Shared/module workers, nested-relay ordering, real buffer/view detachment, cooperative termination and browser-hosted scripting acceptance remain open. Native application indexing need not wait for full web Worker conformance. |
| 14, component model | [Shadow DOM](2026-09-07_shadow_dom_plan.md) landed 2026-09-07/08: shadow-dom **1,512/8,804**, 45 all-pass files; custom-elements **2,149/3,837** in that lane. Slots, flat-tree rendering, style scoping and retargeting exist. Parser/script interleaving subsequently closed declarative registry attachment regressions. | Named gaps include constructable/adopted stylesheets, focus delegation and `:host-context()`. Qualify a concrete Cambium-in-document consumer; these counts do not mean the model is a measured fraction complete. |

### Bounded standards implementations — updated 2026-09-29

- Row 1: [shared text boundaries](2026-09-26_text_boundaries_plan.md) wraps the
  existing Unicode 17 engine with explicit offset domains. Official break
  fixtures and the Genet editor consumer pass. Mere search and Cambium editing
  adoption is published as `ada6f265`, selecting Genet `19c206873ab`.
  Consumer gates passed 232 Cambium and 38 search tests (one existing timing
  test ignored); normalization and index policy remain with Mere.
- Row 3: [accessible names](2026-09-26_accessible_names_plan.md) implements the
  bounded DOM name/description algorithm and native projection plumbing, with a
  joint generated-text fixture. Scripted sessions now read the retained style
  plane; both Boa and Vano attribute-update/name fixtures and all 56 document
  tests pass. Cambium adoption is published as Mere `41206070`, with 207 native
  tests and a standalone Wasm compile gate passing against Genet `c5470fcbc12`.
  Full conformance and physical AT remain open.
- [Generated text](2026-09-26_generated_text_plan.md) now implements bounded
  inline before/after strings, `attr()` and named decimal `counter()`/`counters()`
  with reset/increment/set and source DOM preserved. This does
  not close the broader rendering Row 17 or establish a new WPT count.

Generated content is tracked as **Row 17 of the separate
[Buckram/Livery rendering program](../docs/2026-08-21_buckram_livery_lane_program_plan.md)**,
not Row 17 of this ledger. Implicit list counters, reversed/additional styles,
broader list and marker behavior, and general pseudo-box layout remain open.

### Measurement reconciliation, 2026-09-30 UTC

The full **Vano** refresh completed all 82 archived shards with recorded
source/dependency revisions, unchanged WPT content/manifest and a frozen
runner. The isolated worker's snapshot/window-proxy reporting defect was
fixed and published as `0c4aa9f60b8`; real synchronous/microtask sentinels pass
on both engines and both GC modes. The [census receipt](receipts/2026-09-29_vano_wpt_census/receipt.md)
keeps the broken pristine evidence separate from the repaired run.

It reports 23,999 current records and 33824/144238 passing reported subtests,
with 1090 external timeouts, 10 caught panics, 5 evaluation exceptions and no
worker-process crashes. All 21,671 genuine historical records are present;
one extra historical `test` key was a diagnosed fallback bookkeeping artifact,
and 2328 current records are newly discovered worker-related variants. Skips,
no-results, subtest timeouts and not-run outcomes remain separately visible.

The September 6 baseline rows remain Boa-only historical numbers. The new
dev profile, GC, isolation and timeout policy differ, so this is not a controlled
engine-only delta or a universal conformance percentage. A longer-budget
custom-element diagnostic recovered 1861/1975 passes from a census-timeout
file; it is excluded from the fixed-budget totals. A fully paired run with
identical profiles/budgets, the optional in-process snapshot route, server-mode
families and full-browser rendering acceptance remain separate open gates.

Candidates a lane should add when it opens: `editing` (contenteditable for
a writing product), `streams` (extraction starts before the page finishes),
`service-workers` (offline products).

## Conformance targets, 2026-10-02

**Forms execution, 2026-10-08:** the
[forms plan](2026-10-07_forms_value_validation_submission_plan.md) carries C1/C5/C6.
Phase A's arena value model, native consumers and bounded value/paint/layout
repair are published on main at `e84f9c7f`, with Mere's verified published-source
repin and qualified Turnstone/Cleromancy compatibility. F1-F9 record Mark's
rendering, consumer, accessibility and native URL-parser rulings. Mark authorized
Phase B through its constraint-validation checkpoint. Checkpoint B is qualified
for main publication on 2026-10-09: 963 passes, exactly six allowed Livery
failures and four existing ignores; 924 additional Forms assertions per engine
with every old pass retained, including partial ERROR rows. All 2,731 matched
CSS results are unchanged (1,044 verified passes plus one unverified pass).
F8/F9 provide native validity, notices, a configurable catalog and an Alert
projection; F10 records the native-only matcher correction. The plan preserves
the invalid original probe, corrected 20/20 supplemental control and native
backtracking timeout limit. Phase C submission remains stopped.

**Encoding execution, 2026-10-05:** the
[API plan](2026-10-05_encoding_textdecoder_plan.md) retains C1/C2/C4 and the
corrected API-only scope. E1 authorizes per-realm native storage, a private
finalizer and Vano job delivery. E2's adapter guard preserves empty streaming
state in unchanged encoding_rs 0.8.35. E3's bounded timeout/completion correction
is verified on matching optimized baseline/candidate runs with a 60-second
drive deadline. All 1,313 Encoding records preserve every existing passing
identity: 5,054 assertions improve on Boa and 5,053 on Vano. The fixed 67-record
API selection reaches 19,210/19,408 and 19,209/19,408 passing assertions,
respectively. The fresh code gate passes 971 tests, with three existing ignores.
Every movement, failing control and the unchanged fatal UTF-8 standard/WPT
conflict is recorded in the plan. Mark authorized branch publication with
"push" after the final report. The verified lane is published on
conformance/encoding; main integration remains pending.

**Encoding integration, 2026-10-06:** the qualified main merge f97f0df038c
preserves intervening image and accessibility work and passes 1,100 tests
across seven crates, with three existing ignores. All eight optimized merged
reporting controls qualify on both engines. The plan records an accounting
correction: the earlier 971-test gate contains Runtime 731 and Vano adapter
43, rather than 752 and 22; shared `realms` target basenames had confused the
old counter. Totals, outcomes and frozen WPT maps are unchanged. Per-crate
counts now use exact executable depfile sources. Mark authorized continued
integration and the Streams assessment; the plan records the source/config
qualification and retained resources.

**Streams assessment, 2026-10-06:** the
[Streams plan](2026-10-06_whatwg_streams_plan.md) carries C1/C3. Both C3 records
still throw on both engines using the fresh merged runner. Prerequisite probes
pass async-generator and Promise controls but fail real ArrayBuffer transfer
on both locked engines. Vano has implementation TODOs; Boa's implementation
is behind its broad, disabled experimental feature. The brief's engine
checkpoint is open; narrow engine fixes/repins, a private adapter contract,
or a narrowed first phase await Mark's ruling. No Streams implementation or
full directory baseline is claimed. Existing host incremental response hooks
are recorded separately from the brief's whole-body description.

**Streams foundation qualified, 2026-10-07:** the exact published engine pins
pass 1,142 Genet consumer tests, transfer 7/7 and prerequisites 3/3 on each
engine. All eight starting Rust controls executed and failed. Matched engine-only
directory measurements lose zero passing files/assertions; four assertion gains
inside errored records are attributed to real detachment. Streams modules and
S2's reader-backed Fetch seam are drafted; candidate C3/Streams acceptance and
the final local checkpoint remain open. The linked plan owns full receipts.

**Streams candidate measured, 2026-10-07:** the first candidate passes 1,185
consumer tests, all four authored fixture categories on each engine, and its
optimized runner gate. C3 ceases evaluation throws on both engines. Matched
maps improve non-transferable Streams to 78 passing files on Boa and 77 on
Vano, while strict old-pass losses remain under bounded repair and an explicit
experimental-extension checkpoint. The linked plan retains exact source,
runner, per-crate counts and immutable raw comparisons; acceptance remains open.

**Streams S1 ruling, 2026-10-06:** Mark answered "Ok, agreed. Proceed" to the
recommended narrow engine route: complete Vano's standard transfer methods,
expose only Boa's transfer family, qualify both, then repin Genet. The plan
retains the original prerequisite failures and C3 baselines; broad Boa
experimental enablement, private adapter expansion and BYOB deferral are not
selected. The remaining fetch-body and final-integration checkpoints stand.

**Streams regression candidate qualified, 2026-10-07:** clean implementation
`5a539ae3f40` passes 1,209 tests with three existing ignores and all four
fixture categories on each engine. Ten matched directory maps improve
non-transferable Streams to 88 passing files on Boa and 87 on Vano; both C3
records cease evaluation throws. Fresh comparisons repair all 46 genuine
cross-engine old-pass losses from the first candidate. The 36 remaining raw
keys are 24 independently revalidated label equivalents and 12 experimental
owning/transfer rows awaiting Mark's explicit ruling, with zero unexplained
losses. Independent source review qualifies all 8,446 normalized movements
with zero gaps. The linked plan owns exact source, runner, lock, per-crate
counts and the accepted attribution artifact. Genet remains unpublished at the brief's
final checkpoint; this is not final acceptance while the ruling is pending.

**Streams S5 and final checkpoint ruling, 2026-10-07:** Mark answered
"Approved, authorized" to the completed report and recommended exact 12-row
experimental owning/transfer exception. Standard behavior remains selected;
all twelve row identities are preserved in the separate ruling receipt.
Local acceptance is complete. Publication and main integration are authorized
and follow fresh qualification preserving current main's fontsan/WOFF2 work.

**Streams landed and published, later 2026-10-07:** main/origin now contain
exact tested integration `e410a1e1dab8`. Fresh qualification passes 1,209 tests,
two WOFF2 controls, the ordinary optimized runner and all eight fixture runs.
All ten directory maps match the accepted candidate with zero movements or
new old-pass losses. The final raw accounting separately records 24 label
equivalents, 12 approved experimental rows and zero unexplained losses.
The linked plan preserves complete artifacts, remaining conformance limits,
and the read-only Forms Phase A preparation with its pending paint-scope choice.

Mark asked for the next lanes to come from WPT and specification
conformance. These targets were chosen from the 2026-09-29 Vano census
(`receipts/2026-09-29_vano_wpt_census/outcomes.json.gz`), broken down by
subdirectory, and from the code. Shards that need server mode to pass were
set aside; disk mode cannot measure them: `referrer-policy` 2/8,448,
`mixed-content` 2/2,281, `upgrade-insecure-requests` 0/1,000 and `cookies`
3/968 subtests.

| Target | WPT, 2026-09-29 (Vano, dev, 30 s cap) | What the code shows (2026-10-02) |
|---|---|---|
| WHATWG Encoding | `encoding` 7,561/35,303 subtests; `legacy-mb-japanese` 0/21,813; 814 legacy-CJK files killed at the cap; `encoding/streams` 0/225 | `TextDecoder` ignores its label and decodes UTF-8 only (`components/script-runtime-api/fetch.rs:859`). `encoding_rs` 0.8, which implements the Encoding Standard, is a workspace dependency used by `genet-scripted` (`document.rs:1573`). |
| HTML constraint validation | `html/semantics/forms/constraints` 0/877; `form-control-infrastructure` 0/120; `the-input-element` 218/1,853; `form-submission-0` 0/159 | There is no `ValidityState`, `checkValidity` or `setCustomValidity` in `components/`, and no value model for form controls: no dirty-value flag, value modes or sanitization. The native editor writes typed text into the `value` attribute (`genet-documents/src/engines/livery.rs:1154`), and accessibility reads it from there (`genet-render/src/a11y.rs:116`). |
| WHATWG Streams | `streams` 547/2,389. By subdirectory: readable 186/722, piping 58/458, byte streams 68/464, writable 101/392, transform 117/267 | A "buffered model" of Readable, Writable and Transform streams in the fetch bootstrap (`fetch.rs:872-1078`). Async streaming, BYOB readers and `pipeTo`/`pipeThrough` are marked deferred there. |
| CSS-wide WPT | Last full CSS map 2026-08-24 (`Code/testing/genet/wpt-ledger/2026-08-24_k6_census_v3/`, 36,311 records). Reftest pass/fail then: CSS2 3,304/2,645, writing-modes 186/927, grid 285/858, break 83/832, text 562/824, counter-styles 14/196 | Since then the transforms, line box, K7, form-control, out-of-flow and selector lanes have landed, so the map is stale. `genet-wpt conformance css` already joins full result maps to the manifest (`docs/2026-07-28_absolute_css_conformance_ledger.md`, §Workflow). |

Rulings, in Mark's words:

- **C1.** Asked "Which conformance targets should I brief next?", with the
  four targets above as options, any number selectable. Mark selected all
  four: "Encoding via encoding_rs (Recommended), HTML constraint validation,
  WHATWG Streams, CSS-wide WPT census refresh". Each is briefed to a separate
  implementing agent.
- **C2.** Asked whether the Encoding lane may add `encoding_rs` as a
  dependency of `script-runtime-api`. The options were: allow the workspace
  edge, or checkpoint it. Mark: "Allow the workspace edge (Recommended)". So
  it is added as `encoding_rs = { workspace = true }`; no new crate and no
  version change.
- **C3.** Asked where the two census evaluation errors that are stream
  getter traps go (`fetch/api/basic/stream-safe-creation.any.html`,
  `streams/readable-streams/patched-global.any.html`). The options were:
  move them to the Streams lane, or keep them in the census-panic lane.
  Mark: "Move them to Streams (Recommended)". The census-panic brief drops
  them (its E2 and E3), and the Streams brief carries them.

**Correction, 2026-10-02 (later the same day).** The Encoding row above
overstated what `TextDecoder` can reach, and Mark was told "The biggest
subtest gap for the smallest change". Classifying the 2026-09-29 `encoding`
records by file name splits the shard as follows:

| Mechanism | Records | Subtests pass / total | Killed at cap |
|---|---:|---:|---:|
| Form submission in a legacy charset (`*-encode-form-*`) | 584 | 0 / 20,951 | 570 |
| `TextDecoder`/`TextEncoder` API | 101 | 7,548 / 13,111 | 2 |
| URL query encoding in the document's charset (`*-encode-href-*`) | 230 | 0 / 862 | 228 |
| Legacy-charset documents decoded through an iframe | 385 | none reported (369 no results) | 16 |

So `TextDecoder` through `encoding_rs` can recover at most about 5,600
subtests. The `legacy-mb-japanese` 0/21,813 lies almost entirely behind form
submission, which Genet does not have (`form-submission-0` 0/159). C1's words
stand as given. The Encoding lane's scope goes back to Mark with this
evidence.

Rulings on the corrected picture:

- **C4.** Asked how the Encoding lane is scoped. The options were:
  `TextDecoder`/`TextEncoder` only; Encoding across the platform (adding URL
  query encoding and legacy-charset frame documents); or drop Encoding for
  now. Mark: "TextDecoder/TextEncoder only (Recommended)". So the lane covers
  labels, `fatal`, `ignoreBOM`, streaming decode and `encodeInto` through
  `encoding_rs`. URL query and form encoding in a legacy charset come with
  form submission (C5). Legacy-charset frame documents remain unassigned.
- **C5.** Asked where form submission goes, given that 20,951 encoding
  subtests sit behind it. The options were: a later phase of the forms lane,
  its own lane later, or not now. Mark: "Later phase of the forms lane
  (Recommended)". So the forms lane runs the value model, then constraint
  validation, then form submission: the form data set, the urlencoded,
  multipart and text/plain encoders in the form's charset through
  `encoding_rs`, then navigation of the target.
- **C6.** Asked where a form control's current value (dirty value,
  checkedness, selection) lives. The options were: node state in the DOM
  arena; script-side state only; or let the lane bring options. Mark: "Node
  state in the DOM arena (Recommended)". So there is one per-control record in
  `genet-scripted-dom`, read through `LayoutDom`. Script's `.value`, the
  native editor (`genet-documents/src/engines/livery.rs:1154`), Livery paint
  and accessibility (`genet-render/src/a11y.rs:116`) all share it, and the
  `value` attribute becomes `defaultValue`.

*Reading, not ruled:* constraint validation cannot be built correctly without
the form-control value model. Where the current value lives is shared by
script, the native editor, paint and accessibility, so that brief's first
checkpoint brings the representation back to Mark.

## Consumers on record

| Row | Consumer | Where it is named |
|---|---|---|
| 1, 4, 5, 6, 7 | mere trail recall, W6 | `mere/design_docs/mere_docs/implementation_strategy/2026-08-12_search_surface_wiring_plan.md` |
| 2 | mere capture plan C3 (web clip) | `mere/design_docs/mere_docs/implementation_strategy/2026-06-26_capture_provenance_consent_plan.md` |
| 9 | muniment OPFS lane | `mere/design_docs/eidetic_docs/implementation_strategy/2026-08-22_redb_opfs_feasibility_plan.md` |
| 15, 16 | the games wing's orthographic voxel presentation, lanes L1 and L5 and the L0 verdict | `isometry/mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md` |
