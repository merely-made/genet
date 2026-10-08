# HTML forms: value state, validation and submission

**Status, 2026-10-08:** Genet's Phase A source `b8a3ec1d6abe88e07438ca4d53b9ca4d2b92111d`
qualifies with 1,943 tests passed, zero failures and nine existing ignores.
All 15 value-model fixtures pass on both engines and the paint guard reports
`unexpected=0`. The decimal-range regression is repaired. Matched supplementary
60-second maps have zero starting pass losses; the prescribed 15-second maps
are retained with Vano selection cutoffs explicitly accounted. Password cluster
masking and Mere's Cambium migration are local implementations. Mere's local
consumer gates pass. F5 authorizes the bounded sibling edits and F6 chooses
accessible leaves across all app projections. F6 qualifies on current integrated Mere
`9105b1ef` with 362 package passes, 63 native host passes and a Wasm compile
check. F5 public-pin gates qualify Turnstone with 674 passes and nine existing
ignores and Cleromancy with 15 passes, zero failures or ignores. Publication
and the verified published-source repin remain pending, so Forms Phase A is not accepted or
published. Phase B and C retain separate checkpoints.

The bounded F4 layout repair at `e84f9c7f` has fresh four-crate qualification
(841 passes) and Mere native consumer qualification (63 passes), followed by
passing catalog, highlighting, restored native-bridge and Wasm compile gates.
The fresh restored four-package Mere suite passes 355 tests. F4's fresh
optimized runner passes all 15 value fixtures on both engines and its CSS guard
reports `unexpected=0`, with all sixteen summaries equal to the starting guard.
Mere's current integration qualifies below; publication and public-source
verification remain required. The original eleven-crate and WPT maps above
retain their own source snapshot.

Authority: [standards ledger](2026-09-07_standards_to_features_ledger.md), C1,
C5 and C6, and `Code/work/briefs/2026-10-02_genet_forms_value_validation_submission.md`.
The HTML standard supplies the [input value modes](https://html.spec.whatwg.org/multipage/input.html#dom-input-value),
[textarea state](https://html.spec.whatwg.org/multipage/form-elements.html#the-textarea-element)
and [selection rules](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#textFieldSelection).
Source policy remains the brief's policy; network access and the delegated
model substitutions were separately authorized in the orchestration thread.

## Rulings

### F1: control text rendering, 2026-10-07

Question as put: "For Forms Phase A, the brief asks to migrate control text
painting to the arena value record. I found no existing input-text drawing
path; textarea text currently comes from DOM children. Should Phase A add
value, placeholder and password text drawing for inputs too, or preserve
textarea display and leave new input drawing for a follow-on? I recommend
including bounded text-control rendering so visible controls share the new
value model."

Options: include text rendering (recommended); defer new input rendering.
Mark's answer, verbatim: "Proceed".

Reading, not ruled: this accepts the recommended bounded rendering scope.
Input values, placeholders and password display join textarea's live-value
projection. Composition/IME algorithms, validation and submission retain
their own scope. F2 below resolves the password bullet-count question.

### F2: password character clusters, 2026-10-07

Question as put: "For password fields, should each visible character cluster get
one bullet, or should each Unicode code point get one bullet? Character clusters
keep accented letters and joined emoji together."

Options: one bullet per character cluster (recommended); one per Unicode code point.
Mark's answer, verbatim: "One bullet per character cluster (Recommended) and yes
to the other recommended one. When you stop working and finish your response,
the prompt disappears, for the record".

Consequence: use the existing ICU grapheme segmenter, draw U+2022 per cluster,
retain the original arena value and keep only the mask in retained display text.
This drawing ruling does not expand native password editing or composition.

### F3: Cambium app-owned fields and verified repin, 2026-10-07

Question as put: "May I migrate Cambium's text fields in Mere to app-owned textbox
elements, preserving highlighting, IME, carets and accessible values, then verify
a Genet repin? This crosses the Forms brief's repository scope and version restriction."

Options: approve Cambium migration and verified repin (recommended); stop at the
compatibility checkpoint. Mark's answer is the same verbatim response recorded in F2;
"yes to the other recommended one" approves the migration and repin scope.

Consequence: Mere owns app-field tags, child text projection, edit routing and
committed-value accessibility decoration. Genet native inputs/textarea consume
their arena state. Verify the changed Mere consumer against the Forms source;
published-source repinning follows integration. Text-valued accessibility action
routing remains a separately identified pre-existing gap.

The prior rulings remain: C1 HTML constraint validation, C5 value model then
validation then submission, C6 node state in the DOM arena read through
LayoutDom. Luna agents substitute for the brief's unavailable models under
Mark's explicit authorization to favor cheap agents.

### F4: bounded app textbox layout repair, 2026-10-07

Question as put: "May Forms include a bounded Genet layout repair for unsized
app textboxes? The live host correctly resolves both the 10em default and 12em
override, but paints child-dependent widths of 87px and 400px. This crosses
into the CSS layout owner; I would add a failing layout fixture, fix the
measured handoff, and rerun the consumer tests and existing CSS reftest guard."

Options: repair the Genet layout path (recommended); defer this repair and
leave the width regression open. Mark's answer, verbatim: "Repair the Genet
layout path (Recommended)".

Consequence: add the reproducing CSS fixture before changing the positioned
sizing handoff, qualify the repaired candidate, refresh Mere consumer evidence
and rerun the existing CSS guard. This does not authorize broader containment
work. F5 and F6 below resolve sibling scope and accessible-leaf policy.

### F5: bounded Turnstone and Cleromancy compatibility, 2026-10-07

Question as put: "The Forms checks now pass. May I include the mechanical
selector, caret-routing, and existing-label updates in Turnstone and Cleromancy
so they can adopt Mere's DIV textboxes? Currently unnamed fields would stay
unchanged."

Recorded selected option and Mark's answer, verbatim: "Include Turnstone and
Cleromancy (Recommended)".

Consequence: update only existing field selectors, native caret classification
and attachment of existing visible labels. Preserve currently unnamed fields.
The siblings still use their existing published pins, so compatibility must
cover those native tags and the explicit new Cambium marker until verified
adoption. This ruling does not authorize a broad dependency-family repin.

### F6: app textboxes are accessible leaves, 2026-10-07

Question as put: "Which accessible-tree policy should Cambium app textboxes
use? Both choices expose committed values and preserve all children for drawing."

Options: app textboxes are leaves across neutral, native and browser projections
(recommended); prune children only in the browser mirror.
Mark's answer, verbatim: "Make app textboxes accessible leaves across neutral,
native, and browser projections (Recommended)".

Consequence: Mere prunes accessibility descendants of its explicit app textbox
marker and clears the textbox's projected child links. Neutral and native
adapters must agree; the browser mirror consumes the neutral projection. Retain
the textbox ID, existing name, committed value, bounds, state, actions and focus.
Drawing children remain in the DOM. Ordinary native HTML inputs and textareas
keep their existing Genet projection. Text-valued SetValue routing remains the
separate pre-existing gap identified under F3.

## Phases and done-conditions

### A: one live control value

Store input/textarea current value, dirty value, checkedness and dirty
checkedness, selection offsets/direction and custom validity storage on each
arena node. Default value remains authored attributes or textarea child text.
Default optional LayoutDom and LayoutDomMut seams preserve other implementors.
Implement input value modes, per-type sanitization and reset; regenerate IDL
metadata and wire live/default values, checkedness and UTF-16 selection.
HTML cloning copies only the fields specified for input/textarea; adoption
preserves the moved node's state. Existing select/option algorithms remain
separate gaps rather than being silently expanded in this phase.

The native editor, accessibility and retained text painting must consume the
same arena record. Textarea authored children remain default content. Input
text fits inside the content box; textarea wraps and preserves line breaks.

Done when focused fixtures have executed failing starting controls, all seven
brief crates and both engine adapters qualify, generated metadata is current,
the paint baseline guard reports unexpected=0, and frozen before/after WPT
maps cover the-input-element, textfieldselection and form-control-infrastructure
on Boa and Vano. Record every movement and separate ERROR rows. Checkpoint A
reports record shape, consumers, painting, fixture controls, per-crate counts,
source/lock/runner/corpus identities and local commits. Stop before B.

### B: constraint validation

After Checkpoint A approval, implement ValidityState, barring, validation
methods/messages, invalid events and validity selectors. Done when focused
controls and both-engine constraints plus Phase A maps qualify, every movement
is attributed and no previous pass is lost. Stop at Checkpoint B before C.

### C: submission

After Checkpoint B approval, implement submission triggers, form data set,
charset-specific encoders and target navigation through existing owner seams.
Done when form-submission-0 and encoding form files move on both engines with
attribution, earlier gates hold and paint baselines remain qualified. No new
dependency or version change is authorized beyond the named workspace edge.

## Findings, 2026-10-07

- Starting main is clean at `161b1a8984553f4c26f8931ea0490105abccba11`.
  Its product sources equal the already qualified Streams integration
  `e410a1e1dab8b64cf9dd3b7eff6648015e9c9bd8`; the later commit is documentation.
- Input has no existing value glyph consumer. Textarea reaches InlineCollector
  through authored text; replacing textContent would destroy its default value.
- Native editing duplicates value and byte-based selection. HTML selection
  uses UTF-16 offsets, requiring explicit conversions at that consumer seam.
- The primary ignored Cargo config substitutes local engines/rendering. Gates
  must distinguish those sources from the qualified published-source lock.
- Current workspace instructions supersede the older brief's mandatory branch
  and worktree: work on clean main, preserve the separate browser-fonts tree,
  and create isolation only for an actual collision. No Forms push is authorized
  at this checkpoint. Root owns commits and serial gates; agents own disjoint
  source files and cannot launch builds independently.

## Genet qualification for Checkpoint A, 2026-10-07

This qualifies the Genet portion only. The approved Mere compatibility gate
must finish before the complete checkpoint and publication decision.

### Record and consumers

`FormControlState` in [layout-dom/lib.rs:33](../components/shared/layout-dom/lib.rs#L33)
stores `value: String`, `dirty_value: bool`, `checked: bool`,
`dirty_checkedness: bool`, `selection_start/end: Option<u32>` in UTF-16 units,
`selection_direction: SelectionDirection` (`None`, `Forward`, `Backward`) and
`custom_validity_message: String`. The arena node owns the optional record at
[genet-scripted-dom/lib.rs:194](../components/genet-scripted-dom/lib.rs#L194).
The optional getter and default-false setter preserve other LayoutDom
implementors; this introduces no required implementation seam for them.
Custom validity storage does not implement Phase B validation.

| Consumer | Current owner and source |
| --- | --- |
| Current/default value modes, sanitization and checkedness | [forms.rs:81](../components/genet-scripted-dom/forms.rs#L81), [setter:108](../components/genet-scripted-dom/forms.rs#L108), [checkedness:174](../components/genet-scripted-dom/forms.rs#L174) |
| Reset and HTML cloning steps | [reset:192](../components/genet-scripted-dom/forms.rs#L192), [clone:240](../components/genet-scripted-dom/forms.rs#L240); adoption preserves the moved node's arena record |
| Script value and selection APIs | [form_controls.rs:24](../components/script-runtime-api/dom/form_controls.rs#L24), [setter:72](../components/script-runtime-api/dom/form_controls.rs#L72), [selection:106](../components/script-runtime-api/dom/form_controls.rs#L106) |
| Native editing and caret/selection | [engines/livery.rs:999](../components/genet-documents/src/engines/livery.rs#L999), [write:1008](../components/genet-documents/src/engines/livery.rs#L1008); edit paths convert between UTF-16 offsets and Rust byte offsets |
| Accessibility current value and checkedness | [a11y.rs:130](../components/genet-render/src/a11y.rs#L130), [checkedness:203](../components/genet-render/src/a11y.rs#L203) |
| Retained control text | [text.rs:632](../components/genet-livery/src/text.rs#L632), separate `prepared_controls` at [text.rs:2265](../components/genet-livery/src/text.rs#L2265) |
| Capture and replay state | [capture.rs:112](../components/genet-scripted/capture.rs#L112), [state capture:197](../components/genet-scripted/capture.rs#L197), [copy_state:173](../components/script-runtime-api/dom/form_controls.rs#L173) |

Livery's producer reads the live record, creates virtual text inside the
control's content box and places shaped/clipped commands in its retained
`TextFrame`. Textarea preserves line breaks and wraps; input stays on one line.
Empty values use the placeholder. Password drawing uses one U+2022 per extended
grapheme cluster and retains only the mask. The paint path takes the retained
frame in [paint.rs:651](../components/genet-livery/src/paint.rs#L651), prepares
inline children at [paint.rs:1012](../components/genet-livery/src/paint.rs#L1012)
and drains text commands into the paint list at
[paint.rs:1092](../components/genet-livery/src/paint.rs#L1092). Authored value
attributes and textarea children remain defaults rather than paint input.

### Tests and controls

`candidate-safe-decimal-range-crates` runs all eleven selected packages at the
fixed source above, restores primary config/lock bytes and records exact
executable/doctest owners in its `-crate-counts.json` artifact.

| Crate | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| genet-documents | 57 | 0 | 0 |
| genet-livery | 664 | 0 | 6 |
| genet-render | 42 | 0 | 0 |
| genet-scripted | 121 | 0 | 0 |
| genet-scripted-dom | 95 | 0 | 0 |
| genet-wpt | 77 | 0 | 3 |
| script-engine-boa | 26 | 0 | 0 |
| script-engine-nova (Vano compatibility identifier) | 43 | 0 | 0 |
| script-runtime-api | 818 | 0 | 0 |
| layout-dom-api | 0 | 0 | 0 |
| genet-idl-interface-table | 0 | 0 | 0 |
| Total | 1,943 | 0 | 9 |

The generated metadata drift fixture passes in its owning crate. The value-model
fixture improves from two of fifteen starting controls to fifteen on each
engine; those two starting passes are not claimed as new gains. Its assertions
cover live/default separation, dirty flags, type modes, sanitization, selection,
reset, cloning and adoption. Paint fixtures additionally cover live/default
separation, clipping, multiline defaults, placeholder replacement, visibility
transitions with a retained neighbor and cluster masking. Disabling the producer
causes exactly those six behavioral failures; restored sources then pass the
450-test focused gate. That negative control was run at `93c9a738`; subsequent
product changes affect only `forms.rs`. The current `text.rs` still has the
original SHA256 `A50491D40A7E255651A5F7B3421047AE39AECFE6FBB4CE4CA5709BF88B26822E`.
The corrected range fixture also covers decimal `.1`/`.29`, negative bases,
upper-bound correction and representable small steps at a large base.

The single value-model fixture has these assertions. The starting outcomes are
the same on Boa and Vano; the final repaired runner passes every row on both.

| Assertion | Starting control | Repaired candidate |
| --- | --- | --- |
| Live input preserves authored default | fail | pass |
| Empty assignment sets dirty value flag | fail | pass |
| Clean input follows default | pass, unchanged control | pass |
| Textarea preserves default and normalizes line endings | fail | pass |
| Checkedness and default/on value have separate defaults | fail | pass |
| Text sanitizer removes CR/LF | fail | pass |
| Email sanitizer trims ASCII whitespace | fail | pass |
| Number rejects invalid floating-point value | fail | pass |
| Range defaults and clamps to current bounds | fail | pass |
| Filename mode rejects nonempty script assignment | fail | pass |
| Value-to-default/on transition preserves old live value in default | pass, unchanged control | pass |
| Input cloning preserves dirty live value | fail | pass |
| Textarea cloning preserves raw value without rewriting children | fail | pass |
| UTF-16 selection and direction reset after value change | fail | pass |
| Form reset restores defaults and clears dirty flags | fail | pass |

Additional controls retain their own source-qualified receipts:

| Fixture | Deliberate failing control |
| --- | --- |
| `live_input_value_paints_without_replacing_its_default_attribute` | Disabled retained control-text producer |
| `live_input_text_is_clipped_to_its_content_box_and_uses_only_current_value` | Same disabled producer |
| `live_textarea_value_projects_multiline_text_and_preserves_author_children` | Same disabled producer |
| `empty_live_input_projects_placeholder_instead_of_default_value` | Same disabled producer |
| `live_control_text_updates_after_hiding_and_showing_with_a_retained_neighbor` | Same disabled producer |
| `password_input_masks_character_clusters_without_retaining_secret_text` | Same disabled producer |
| `abspos_single_line_field_uses_contained_intrinsic_width_with_descendants` | Added before F4 repair; expected 178px, observed 146px in its first case |
| `native_form_state_reads_through_the_window_view` | Disabled Rootstock state forwarding, sole expected failure |
| `native_form_state_change_rebuilds_only_its_owning_window` | Disabled owning-window mutation routing, sole expected failure |
| Neutral committed-value/leaf projection | Disabled pruning; one intended failure |
| Browser mirror app-textbox leaf | Same disabled pruning; one intended failure |
| Native committed-value/leaf projection and nested focus | Same disabled pruning; two intended failures |
| Cleromancy marked-DIV Question-slot typing | Disabled marked-field routing; sole expected failure |

The pruning control passes 83 other cases and fails exactly four. Its unchanged
production/fixture hashes are carried to the fresh current Mere positive gates.
Cleromancy's routing control passes 11 other cases, then the restored source
passes the library and both DOM targets. Turnstone's marked-DIV discovery
fixture uses an independently authored marker and passes against its existing
public family. These sibling fixtures demonstrate compatibility preparation.

### Frozen WPT and paint evidence

Evidence lives under `Code/testing/genet/forms`. The optimized runners use the
same build settings and public dependency lock:

- Before source `2f4f82652495af72798c7ac02be8a7b73152a357`, runner SHA256
  `5DB68DE6ECF285B2DBE254E6EC0CB685D1D654BFF6AFFA718DD9E30F38F2BE3F`.
- Candidate source `b8a3ec1d6abe88e07438ca4d53b9ca4d2b92111d`, runner SHA256
  `4BFC91627C02871033FE0A9E1F4354D26E866F88EFA8062B576CA54990619213`.
- Dependency lock SHA256 `9809247CAEE772F61F2228A92FA324D3E68E40681A8591B7BFA06185B593B850`;
  WPT manifest SHA256 `D5EC5BE9BF1A75ED00D7E7AB28AFE8A694A55E11682BA74305874D70B18DD422`;
  1,649-file corpus SHA256 `C24C04EC10FC9924D334AEBFE5789A20F179F3971683202193C5652C93B3C50C`.

The prescribed Livery/jobs4/timeout120/drive15 maps remain immutable. The first
corrected run has no original baseline pass losses, but Vano `select-event.html`
stops at 265/270 rows; an unchanged repeat stops at 222/270 and leaves five
original passing rows unobserved. These are accounted as pass-to-missing, not
waived. The file requests WPT's long timeout and executes 270 serial promises
with two animation-frame steps per wait. Host drive15 cuts it off before the
already implemented 60-second testharness allowance.

Supplementary `before-drive60` and `after-safe-decimal-range-drive60` repeat all
six directories with both frozen binaries, the same corpus and settings, and
only the drive cap changed equally to sixty. All selection rows complete on
both engines. Completed-file counts are:

| Directory, each engine | Before pass/fail | Candidate pass/fail |
| --- | --- | --- |
| the-input-element | 217 / 1,589 | 920 / 886 |
| textfieldselection | 92 / 598 | 667 / 72 |
| form-control-infrastructure | 0 / 120 | 31 / 89 |

Candidate input ERROR files separately retain eight passing, seven failing,
28 not-run and four timeout rows, with two ERROR files reporting no rows.
Those eight passes are excluded from the table. Seven newly passing radio rows
remain partial observations. The old selection-range harness error had zero
rows; its candidate file has 48 observed passes and one failure, with no
invented baseline assertions. The numeric `valueAsNumber` setter's expected-100
row is incidental to preceding range-value clamping; setters expecting zero
and fifty still fail, so numeric API conformance is not claimed.

`after-safe-decimal-range-drive60-vs-before-drive60-attribution.json` attributes
94 file movements and 2,634 changed rows (1,317 unique case sources across both
engines), with zero baseline pass losses and frozen WPT hashes. Exact/template
source matches and dynamic line hints are distinguished. Selection/event rows
use the case-insensitive selection classifier; earlier mislabeled attribution
artifacts are retained and superseded by this artifact. The two truncated
drive15 maps and their comparisons/attribution remain available independently.

`after-safe-decimal-range-reftest-baselines` passes with `unexpected=0` and fixed
source/runner. All baseline JSON files match the starting Git source. The guard
hash matches the actual recorded starting invocation, including the frozen-runner
parameter, even though that parameter was committed after the starting binary's
source commit. No baseline changes or new product harness changes were made for
the supplementary timing checks.

### Remaining checkpoint work

F1-F6 are ruled. Current integrated Mere `9105b1ef` qualifies against exact Genet product
`e84f9c7f`: 362 package passes, two existing ignored doctests, 63 native host
passes and a Wasm accessibility-example compile. Its graph moves from 1,564 to
1,566 resolved packages: the modern family preserves versions, dependency
definitions, edges and features after approved revision/checkout-path
normalization, while published Knot `eabd4434` retains two Genet `965b64e2`
identities (Fleece 0.5.0 and LayoutDom 0.1.1). Neither JavaScript engine
activates. The unchanged leaf production
and fixture hashes carry the earlier deliberate pruning control, followed by
fresh integrated positive tests. This is a local Git candidate receipt.
The frozen 1,690-row candidate lock SHA256 is
`3537E6067130E1D8993DAC7DB2F143CB9AA60099879F20EBD654DAE79CA97735`.
The first two-thread package attempt stalls in GPU tests and is terminated;
its source/config/lock restoration succeeds, but it remains unqualified.
The unchanged full suite passes at one test thread, with all GPU cases run.
Receipts are `mere-forms-current-knot-packages-serial-retry`,
`mere-forms-current-knot-native` and `mere-forms-current-knot-wasm-a11y`.

| Current Mere gate owner | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| Cambium | 257 | 0 | 1 |
| Rootstock | 72 | 0 | 1 |
| Native accessibility | 23 | 0 | 0 |
| Browser mirror | 10 | 0 | 0 |
| Six native-host targets | 63 | 0 | 0 |

The Wasm example check confirms compilation only. Every guarded gate preserves
unowned source bytes/mtimes and restores the starting manifest/lock. The older
`2a89d8dc` integration and repin proposal retain their own historical receipts.

Turnstone `8e06a85f` passes its complete public-pin library: 674 passes, zero
failures, nine existing ignores. Cleromancy `85c8f77` resolves its existing
public graph after a network retry of an uncached registry index entry. Its
routing control qualifies 11 passes and the intended sole failure; the restored
library and two DOM targets qualify 15 passes, zero failures or ignores. These F5
checks qualify mechanical preparation on the siblings' existing pins, not
adoption of the unpublished Mere/Genet candidate.

Publication remains gated by the brief's "Do not push" instruction. The exact
two-file qualified Mere repin proposal is recorded in
`Code/testing/genet/forms/mere-forms-current-knot-repin-review.patch`; the exact
qualified candidate manifest and lock are named in its review-proposal JSON.
The normalized patch passes `git apply --check`. Verify the candidate files
against the published Genet source after publication is authorized. Mere's
shared unpublished history also contains four other-owner Vault/Lattice
documentation commits; a normal main push would include them. Native password
composition and text-valued accessibility actions remain separate existing
gaps. Phase B/C require their next checkpoint decisions.

The complete commit subjects/hashes and known published bases are recorded in
`Code/testing/genet/forms/forms-phase-a-publication-inventory.json`. Current
documentation-only descendants are bound to their qualified sources in
`Code/testing/genet/forms/forms-current-checkpoint-source-binding.json`.

The stable `genet-encoding` target and published-source `genet-streams` Cargo
home remain owned by the pending Forms publication/repin gate. Mere and
Turnstone reuse their stable repository targets and normal dependency cache;
Cleromancy's initially empty approved `C:/t/cargo-targets/cleromancy` is built
for its F5 checks and retained for reuse. No Forms worktree or new isolated
Cargo home was created. Original primary overrides and locks remain preserved.

## Progress

- 2026-10-07: Read-only preparation is preserved at
  `Code/testing/genet/forms/phase-a-ownership-assessment.md`. F1 authorizes the
  text projection. Two bounded Luna agents prepare arena and projection changes;
  root owns runtime/IDL glue, documentation, baseline and serial qualification.
  Evidence stays under `Code/testing/genet/forms/`. Reuse the approved stable
  `C:/t/cargo-targets/genet-encoding` target and existing published-source cache;
  do not create numbered or timestamped output directories.
- 2026-10-07: Documentation ruling commit `2f4f8265249` precedes the freshly
  built frozen runner (published-source lock `9809247C...`, runner
  `5DB68DE6ECF285B2DBE254E6EC0CB685D1D654BFF6AFFA718DD9E30F38F2BE3F`).
  Both starting engines execute all 15 value fixtures, passing two controls
  and failing thirteen target behaviors. The unchanged clean-default and
  type-transition controls remain passing; they are not reported as failing
  controls. Receipts: `before-runner.receipt.json` and
  `before-{boa,nova}-phase-a-value-model.receipt.json` under the Forms evidence.
  Freeze covers all Forms-family and shared harness/common/IDL/font files
  (1,649 including the manifest), rather than unrelated WPT asset families.
  Primary ignored Cargo config and original lock were restored byte-for-byte
  after the gate. Arena and projection agents now edit their separate owners;
  the original frozen runner supplies the directory baseline independently.
- 2026-10-07: All six whole-directory starting maps are recorded. ERROR-file
  assertions remain separate in `before-summary.json`; the input runner's
  218 passes include one row from an errored file, so its completed-only count
  is 217. The starting reftest guard reports unexpected=0. The guard now accepts
  an explicit frozen runner and uses locked Cargo commands on its fallback
  path, avoiding an unlocked metadata call in the shared checkout.
  Review corrects textarea direct-child defaults, raw/API newline distinctions,
  sanitizer edge cases and state-change invalidation. Capture state records
  carry the raw arena snapshot; serialized variants are appended to preserve
  existing postcard discriminants. Password display policy remains pending.
- 2026-10-07: Arena and non-password projection chunks are handed back for the
  first serial crate gate. This is a local implementation draft, not Checkpoint
  A acceptance. Internal arena cursor positions also serve email/number and
  other previously editable value controls; runtime selection APIs independently
  apply HTML's type restrictions. Focused coverage includes retained repaint,
  UTF-16 editing, defaults, textarea lines, placeholder, checkedness and capture
  payload roundtrip. Password painting remains dependent on the pending answer.
- 2026-10-07: The first draft crate gate (`draft-serial-crates.receipt.json`)
  stops at compilation with a duplicate `tree_root` method. The Forms helper
  is renamed `form_control_tree_root`: radio grouping follows ordinary DOM
  parents, while the existing GC opaque-root helper also follows shadow hosts.
  No tests executed in that failed gate. The original lock and local config
  were restored; a fresh receipt will record the repaired draft gate.
- 2026-10-07: `draft-repaired-crates` stops before tests at a missing
  `LayoutRect` import in the retained control projection. The import is fixed.
  Form association now shares an arena owner query across bindings and radio
  grouping, respecting first-ID lookup, detached ancestor fallback and shadow
  tree scope. Focused fixtures also correct newline glyph accounting, retained
  hit-test setup and implicit checkbox/radio accessibility roles. These remain
  draft checks; password painting and the ID/tree radio rescan are still open.
- 2026-10-07: `draft-owner-crates` compiles the library changes and stops in
  document tests on an early `note` reference and two uses of a nonexistent
  input method. These fixture errors are fixed. Transient pre/post radio
  association snapshots now cover ID edits and tree replacement/removal,
  including shadow connection, without expanding the retained control record.
  Reconciliation follows form-owner change and becoming-connected triggers;
  a detached generic-container insertion alone does not clear checked peers.
  A focused paint fixture checks that overflowing live-value glyphs are
  enclosed by the content-box clip. All changes await executable qualification.
- 2026-10-07: `draft-reconciliation-crates` is invalidated by an agent's final
  source edit after its commit. Root stops only the verified owning Cargo/rustc
  processes; the receipt records source drift and byte-for-byte restoration.
  The final checkedness guard is committed before the next run. Qualification
  now distinguishes a separate Mere build with its verified `cargo-targets/mere`
  owner, rather than blocking unrelated workspace work globally.
- 2026-10-07: The fixed-source `draft-frozen-crates` reaches Livery's tests
  and stops on two nonexistent DOM helper calls. Assertions now read the
  authored child nodes directly. Reset scans the form's ordinary tree, rather
  than its owner document, so explicit associations inside a shadow root reset
  together; the two-engine owner fixture covers this case. No executed test
  pass is claimed from this compile-only gate.
- 2026-10-07: `draft-behavior-crates` executes the fixed-source suite at
  `6e4a0a9f4064f8bfb05d73f256c815da7c18bced`, with the published-source lock and
  unchanged sources; primary overrides and lock restore byte-for-byte. Seven
  failures remain in three library targets. The value/owner runtime fixtures
  pass on both Boa and Vano. Four direct Livery control fixtures expose a missing
  text-frame seed when there is no ordinary text; the session fixture happens
  to seed one through its button label. Native action/edit assertions must read
  the live value separately from authored defaults. Structural inspection still
  reports authored DOM content; accessibility is the live-value projection.
  The association walk is changed to an explicit stack, with a deep small-stack
  preorder fixture, because mutation snapshots now traverse general trees.
- 2026-10-07: Compatibility fork F3 is pending, without a prompt deadline.
  Mere's `crates/cambium/cambium/src/controls/field.rs` and `src/styled_field.rs`
  deliberately render `TextInput` buffers, syntax spans, IME text, ghosts and
  carets as input/textarea children. Native HTML form-state projection suppresses
  those app-owned children. Synchronizing only the value would lose the styled
  projection and introduce a second editing owner. Proposed scope: migrate
  Cambium fields to app-owned textbox elements, preserving child rendering and
  routing, and provide an explicit committed-value accessibility path, then
  verify the eventual Genet repin in Mere. This crosses the Genet-only brief and
  its version-change restriction; user authorization is requested before the
  cross-owner implementation. Mere currently pins Genet `965b64e206a47d1c8808472de9aa461233638768`;
  that existing pin is not compatibility evidence for the local Forms draft.
  The old child-value regression is retained pending this decision, rather than
  accepting empty Cambium fields or adding a native HTML child-text fallback.
- 2026-10-07: Local repair commit `501591635df` seeds control-only text frames
  through the same eligibility classifier used by the producer, preserves
  structural/default text in native assertions and removes recursive association
  walking. `draft-control-projection-crates` stops before tests at a CSS/Taffy
  `Display` type collision in its new visibility predicate. The CSS enum is
  explicitly qualified before a fresh gate. Sources stayed fixed and primary
  overrides/lock restored; no pass is claimed from this compile-only attempt.
  Read-only follow-up confirms Mere's `cambium-winit-a11y::project_tree_with_actions`
  already decorates Genet's mutable AccessKit tree from Cambium-owned metadata.
  Committed-value decoration can use that existing Mere owner seam without
  introducing Genet-specific app markers; raw child text includes IME/ghost/caret
  projections and cannot serve as the committed value. Text-valued accessibility
  action routing is a separate existing gap (the current SetValue path is numeric).
- 2026-10-07: `draft-visible-control-crates` executes 57 native-document and
  67 arena tests successfully, including both-engine native behavior and the
  deep small-stack association walk. Livery has 319 passes and five failures:
  four zero-glyph fixtures and the newly added empty-frame assertion. Direct
  tracing disproves the earlier missing-frame diagnosis: `layout_inline_groups`
  always stores `Some(TextFrame)` even without ordinary text. The redundant
  frame seed/classifier and incorrect frame-absence assertion are removed.
  Paint fixtures register the existing local Ahem font with an explicit font
  face, retaining the exact live/default, clipping and multiline assertions.
  The atomic text traversal uses an iterative preorder walk and prunes
  display:none subtrees; a control beneath a hidden ancestor remains unpainted.
  This corrected draft awaits a fresh focused gate; font setup is not yet a
  qualified explanation of the original zero-glyph failures.
- 2026-10-07: `draft-deterministic-control-crates` repeats the 57 native and
  67 arena passes; Livery has 320 passes and the same four zero-glyph failures.
  Ahem alone does not repair them. Diagnostics show valid control geometry but
  only the outer inline-box shape and zero retained fonts. The actual conflict
  is `TextFrame::prepared_sources`: inline-box layout marks the control's node
  there, and the virtual-value producer treats that as prepared text. Native
  session fixtures use display:block and avoid the marker, explaining their
  different outcome. A separate `prepared_controls` marker tracks completed
  value preparation, including retained subtree copies, while the existing
  inline-box marker continues serving geometry and paint traversal. This repair
  awaits fresh qualification; the deterministic font fixtures are retained.
- 2026-10-07: `draft-prepared-control-crates` qualifies commit
  `3020e3c8b9b7a3c7c6a033390543d5779dd99260`: 57 native-document, 324 Livery
  and 67 arena tests pass with no failures or ignores. The source tuple stays
  fixed and the original primary config/lock are restored. This repairs all
  four zero-glyph failures. The ownership preflight now reads only selected
  target settings from live process environments and verifies foreign workspace
  defaults/config hashes, preserving unrelated builds even while they have no
  active compiler child. Receipt and executable-owner counts are under
  `Code/testing/genet/forms/draft-prepared-control-crates*`.
  Read-only Luna review finds no concrete marker defect but identifies a
  visibility-transition gap. A new fixture paints two controls, hides one,
  changes its live value while hidden and shows it again, checking current text
  and the retained neighbor without duplicated glyphs or rewritten defaults.
  That fixture awaits its own fresh focused gate. Password and F3 remain pending;
  the complete candidate gate, negative control and after maps follow those rulings.
- 2026-10-07: `draft-control-visibility-crates` qualifies source commit
  `dd7ce9f21e467bfd234a2c7dc51b34dba791a144`: 57 native-document, 325 Livery
  and 67 arena tests pass, with zero failures/ignores. The visibility/value
  transition and retained-neighbor fixture passes. Sources remain fixed, and
  the primary config and lock restore byte-for-byte. No full-suite or WPT after
  qualification is inferred from this focused gate. The prepared negative-control
  helper now names all four `live_` paint fixtures, including the new transition,
  and records its exact Cargo arguments; it has parsed but has not been executed.
  No Forms push or integration occurs. Stable target `C:/t/cargo-targets/genet-encoding`
  and published-source home `C:/t/cargo-homes/genet-streams` stay owned by this
  unfinished Forms qualification. No Forms worktree was created.
- 2026-10-07: Mark approves F2 cluster bullets and F3 Mere migration/verified
  repin. Livery now uses its existing ICU grapheme segmenter for masked display,
  preserving the original arena/default values and excluding the secret from
  retained text. A deterministic fixture covers combining accents, joined emoji,
  flags and empty-value placeholder replacement. The native accessibility
  child-value fixture is updated under F3 to assert the arena value, stable label
  and unchanged authored child text. Two Luna agents own the Mere field/selector
  migration and committed-value accessibility decoration in disjoint files.
  Native password editing/composition and text-valued accessibility actions are
  existing separate gaps. The previous 449-test receipt predates these changes;
  no new pass is inferred from it.
- 2026-10-07: Source `93c9a738ce32037f76d82768edff4e45a74ebfeb` qualifies in
  `candidate-cluster-mask-crates`: documents 57, Livery 664 (six ignored), render
  42, scripted 121, arena 94, WPT 77 (three ignored), Boa adapter 26, Vano adapter
  43 and runtime API 818. Layout DOM and IDL table targets execute zero tests;
  generated metadata's drift fixture passes in its owning crate. Total: 1,942
  passed, zero failed, nine ignored. All eleven selected packages and their
  executable/doctest owners are accounted for, with fixed source tuple and
  restored primary overrides/lock.
  `candidate-disabled-control-text` temporarily returns false from the virtual
  producer; exactly six named live-value, clipping, multiline, placeholder,
  visibility/neighbor and password fixtures fail behaviorally (exit 101), with
  no compiler failure. Source bytes, config and lock restore exactly. A fresh
  rebuild in `candidate-restored-control-crates` then passes documents 57,
  Livery 326 and arena 67 (450 total). This closes the stale-binary control.
  Mere's approved field migration is local commit `019e07a0`; its adapter now
  restricts the committed-value marker to explicit textboxes, and both native
  and neutral projection fixtures cover Unicode values and transient child
  content. Mere tests have not yet executed. Its pre-repin freeze records three
  current-family manifests, 33 current Genet root-lock packages and two separately
  pinned legacy Knot packages to preserve. The Turnstone/Cleromancy mechanical
  selector/caret/existing-label scope question is open without a response timer.
- 2026-10-07: Frozen after runner `D127C6CD...` at source `1fc2102aec8` executes
  the 15 value-model assertions successfully on both engines, versus two in the
  starting controls. All six first-candidate directory maps are complete and
  accounted in `after-vs-before.json`, preserving ERROR rows separately. Each
  engine has completed-file passes of 919 input, 667 selection and 31 form
  infrastructure assertions, versus 217, 92 and zero before. Input ERROR files
  additionally hold eight passes, seven failures, 28 not-run rows and four
  timeouts; those are not added to the completed-file counts. One starting pass
  is lost on each engine: `range-2.html` assigns `.6` with step `.1`, and the
  candidate reports `0.6000000000000001` instead of `0.6`. The frozen diagnostic
  `after-range-regression-repro-boa.log` confirms the exact failure. These maps
  are preserved as the first candidate, not accepted as Checkpoint A.
  The repair interpolates decimal range steps after scaling their canonical
  decimal representations, including bound correction, with a finite-arithmetic
  fallback for extreme values. Its arena fixture covers aligned fractional and
  exponent inputs, a real mismatch, negative base, a nonaligned upper bound and
  the decimal `.29` step. No dependency changes. Fresh crate tests and a new
  frozen runner/maps must qualify this repair. Luna prepares movement attribution
  against the immutable first candidate; final-source attribution will be checked
  again after the corrected maps.
- 2026-10-07: Luna's review of `532a02984ac` finds that finite decimal scaling
  alone can erase a representable small step above f64's exact-integer range
  (base `1e15`, step `.1`). Root stops only the verified owning Cargo process
  tree in `candidate-decimal-range-crates`; its exit-15 receipt and interruption
  record preserve the unqualified compile and exact config/lock restoration.
  The repair now bounds every scaled arithmetic intermediate by the exact-
  integer limit before using it, otherwise retaining the original floating-point
  calculation. The arena fixture adds the observed large-base live-value case.
  A fresh source gate follows; no test pass is claimed from the stopped run.
  `after-vs-before-attribution-first-candidate-final.json` accounts 94 first-
  candidate file movements and 2,636 changed rows with frozen WPT source hashes.
  The two old-pass losses are the same range row, once per engine. Seven new
  passes in each timed-out `radio.html` remain partial observations; the
  selection-range file's harness-error-to-file-failure transition adds 48 observed
  passes and one failure, with no invented baseline assertions. Final attribution
  must be regenerated against the final source/maps.
- 2026-10-07: Mere candidate preparation at `019e07a0` resolves the exact
  Genet `b8a3ec1d` source family. The 1,688-package lock preserves every version,
  dependency array and checksum; exactly 33 Genet, nine Boa and three Vano
  revisions move. The two legacy Knot-owned Genet identities remain unchanged.
  The default metadata graph does not activate a JavaScript engine. The catalog
  acceptance program passes and regenerates the two HTML goldens; this prepares
  artifacts, not consumer qualification. Root restores the original Mere
  manifest and lock and verifies all 3,409 tracked source bytes and mtimes
  outside owned outputs. Luna finds no concrete field/projection defect and
  identifies the optional `highlight` library feature as an additional gate.
  Fresh Mere consumer tests and their disabled-marker control follow. The
  sibling scope question remains unanswered, with no response timer.
- 2026-10-07: `mere-forms-default-packages` exits 101 before executing tests:
  Rootstock's multi-window mutation router lacks the new
  `FormControlStateChanged` variant. Its receipt preserves stable sources and
  exact manifest/lock restoration. The bounded Mere compatibility repair routes
  the event by its node and forwards the new arena state through the existing
  `WindowDom` read wrapper. New fixtures distinguish current value from its
  unchanged default attribute and verify only the owning window rebuilds.
  Independent routing/forwarding controls and fresh combined consumer gates
  remain pending; the old-pin checkout is not accepted for publication without
  its verified repin. No Genet product source changes.
- 2026-10-07: `mere-forms-window-bridge-packages` stops before tests because
  Rootstock's new forwarding method used a borrowed state return. The
  `LayoutDom` seam returns an owned `FormControlState` snapshot. Correcting that
  signature preserves the existing seam; both failed compile attempts retain
  stable-source and exact manifest/lock restoration receipts. A fresh retry
  follows, with no pass inferred from either failure.
- 2026-10-07: `mere-forms-owned-window-state-packages` qualifies Mere `40272b30`:
  Cambium 252, Rootstock 72, native accessibility 22 and browser mirror nine
  passes, zero failures and two existing ignored doctests. Counts distinguish
  Rust's two Cambium doctest result blocks. The first focused native-routing
  build finds a missing `LayoutDom` trait import in the migrated fixture and
  executes no tests; the import is corrected for a fresh retry. Luna also finds
  that the browser mirror carries the committed field value and its projected
  decoration descendants. A separate question asks whether app textboxes should
  be accessible leaves across adapters or only in the browser mirror. No
  pruning is implemented while that ruling and sibling scope remain pending.
- 2026-10-07: `mere-forms-native-routing-imported` executes 59 passes and one
  failure across five targets, then stops before `text_caret`: an unsized app
  field takes child-dependent widths of 87px and 400px. The native input's
  former default is 20 columns at half an em each. Mere restores that intrinsic
  width with a font-relative `contain-intrinsic-size` substitute and exposes
  `--cambium-field-intrinsic-width` for host overrides; explicit CSS widths keep
  their existing behavior. The existing regression and an override fixture
  must pass on fresh source, and component/catalog receipts must be refreshed.
  Three preflight-only starts had falsely classified a separate
  `mere-grammar-g3` worktree's nested target as this gate's output; the helper
  now compares explicit Cargo roots and actual profile output directories,
  preserving the independent lane. Those starts mutated no files.
- 2026-10-07: The substitute-width attempt remains unqualified:
  `mere-forms-intrinsic-width-native-routing` executes all six selected targets,
  with 61 passes and two failures. The unsized short field is still 87px and the
  long field still 400px, including the host's requested 12em substitute. The
  receipt verifies stable sources and exact manifest/lock restoration. Mere's
  existing native width assertions now report properties from the host's live
  computed-style reader, to distinguish resolution from positioned layout
  before choosing another repair. Genet product source remains unchanged.
- 2026-10-07: `mere-forms-field-sizing-diagnostics` has six passes and the same
  two behavioral failures, with stable source and exact restoration. The live
  host resolves `position:absolute`, `width:auto`, `contain:inline-size`,
  `font-size:16px` and `contain-intrinsic-size:10em 1.2em` or `12em 1.2em` as
  requested. This excludes failed custom-property substitution as the cause.
  The positioned solver consumes substitute intrinsic sizes, but its scratch
  reformat handoff and descendant-bearing fragment publication remain to be
  measured; the Taffy adapter separately omits `computed.contain` from its
  size-containment flags. That omission is not yet established as the cause.
  A new checkpoint asks permission for a bounded Genet CSS layout repair with
  a failing fixture and fresh consumer/reftest gates. That decision, sibling
  scope and accessible-leaf projection are all pending without response timers.
  Independent native-state routing/forwarding controls can proceed meanwhile.
- 2026-10-07: Mark approves the bounded Genet layout repair as F4 above.
  Reproduction, a measured handoff repair and fresh qualification follow.
- 2026-10-07: The unchanged-layout reproduction at `b96112de` executes two
  passes and the intended new descendant-bearing field failure: expected
  178px, actual 146px including preedit/caret spans. Its source tuple is stable
  and the original config/lock are restored. Buckram correctly excludes
  size-contained roots from content-intrinsic measurement; Livery's reformat
  gate nevertheless requires that map entry even when its positioned solver
  used an explicit containment substitute. Final leaf-only resize cannot
  update the field with descendants. The bounded repair carries whether the
  positioned solver received that substitute and admits its same-flow
  constrained reformat. It leaves content measurement and the general Taffy
  containment adapter unchanged. Fresh fixture, Livery/consumer and CSS guard
  qualification remain required; no pass is inferred from the implementation.
- 2026-10-07: The repaired field fixture passes all six fallback/override/
  definite-width cases, and the first broader Livery gate passes. Review
  confirms the explicit-width and same-flow guards, but finds the alternate
  admission could bypass the formatter's existing Block/Leaf root-kind
  boundary. The substitute route is restricted to those same root kinds,
  preserving Flex/Grid admission. Fresh Livery qualification follows this
  narrowing. A possible nested-positioned descendant size issue belongs to
  the existing single-reformat limitation and remains unmeasured; this flat
  app-field repair does not claim general containment or nested-layout
  conformance. The field's preedit/ghost/caret runs are ordinary inline spans.
- 2026-10-07: `candidate-contained-width-consumer-crates` qualifies the narrowed
  product `e84f9c7f9aec23320c539784961d1465f8a53a9b`: Livery 665, documents 57,
  render 42 and WPT 77 passes, zero failures and nine existing ignores. Counts
  cover all 54 selected completed targets. Original config/lock restore
  exactly. Mere's new frozen candidate lock changes 33 Genet source revisions
  and 31 dependency source references, preserving package versions, graph
  shape/checksums and both Knot-owned legacy identities. The mechanical lock
  preparation is not a consumer pass; locked resolution and fresh Mere tests
  follow. Both unruled questions remain open without response deadlines.
- 2026-10-07: Locked Mere resolution fetches only the exact Genet revision
  through the process-local local-checkout rewrite after the offline start
  reports an uncached revision. The fresh 1,564-node metadata graph equals its
  prior graph after revision/checkout-path normalization: 28 modern Genet
  packages resolve, all 33 remain locked, both legacy identities remain and
  neither JavaScript engine activates. `mere-forms-contained-width-native-routing`
  then qualifies all six native targets with 63 passes and zero failures,
  including both formerly failing widths, caret/scroll/selection and routing.
  Source bytes/mtimes are stable and original manifest/lock restore exactly.
  Catalog receipts, independent bridge controls and remaining consumer gates
  follow. Accessible-leaf policy and sibling scope remain unanswered.
- 2026-10-07: Catalog regeneration changes only the single-line width fallback
  in both HTML receipts, and both catalog acceptance tests pass. Independent
  disabled-routing and disabled-forwarding controls each record 71 passes and
  their intended single behavioral failure; the fresh restored Rootstock
  library passes all 72 tests. Cambium's optional highlighting library passes
  254 tests and the browser accessibility example compiles for Wasm. The
  guarded Mere transactions restore the starting manifest/lock and preserve
  unowned source bytes/mtimes. A read-only multiline review finds no newline
  or multiline-state loss, but no full-pipeline newline fixture is claimed.
  A fresh optimized runner and the F4 CSS guard follow. The two pending
  questions retain their original scope and have no response deadlines.
- 2026-10-07: The optimized F4 runner freezes at documentation head `79a7f011`
  over product `e84f9c7f`, SHA256
  `36088821A7E876098A4F4C639E5DC474547AA25AA321B56712D8CC0A85B4E0A2`.
  All fifteen value-model assertions pass on Boa and Vano. The unchanged CSS
  guard reports `unexpected=0`, all sixteen summary counts equal the starting
  guard, and baseline JSON files remain unchanged. The comparison artifact is
  `after-contained-width-reftest-comparison.json` under the Forms evidence root.
  This completes F4's requested fresh regression checks.
- 2026-10-07: The committed-value negative control initially produces the two
  intended failures but is rejected because its expected native test module
  is `tests` instead of `dpi_tests`. That attempt remains unqualified. The
  corrected fresh control records 74 passes and exactly the two intended
  failures, then restores both production lookups with fresh mtimes. The fresh
  restored four-package Mere gate at `ee699000` passes 355 tests with zero
  failures and two existing ignored doctests. All source bytes/mtimes outside
  owned restoration remain unchanged. No publication, repin or pruning policy
  follows without resolving the two outstanding questions.
- 2026-10-07: Mark answers both outstanding questions as F5 and F6. Two bounded
  Luna agents implement sibling mechanical compatibility and Mere accessibility
  leaves in disjoint files. Root owns numbered rulings, review, commits and
  serial fresh qualification. No dependency repin or publication occurs during
  implementation; source gates must finish before the complete Checkpoint A.
- 2026-10-07: Mere `7d133ddc` implements F6 in Rootstock's neutral projection and
  the native AccessKit adapter; the browser mirror inherits the neutral leaf
  topology. Prune descendant rows and routes after producer decoration, retain
  the owner ID/value/name/bounds/actions, and resolve nested descendant focus
  to its surviving outer textbox. The drawing DOM stays intact. The updated
  Unicode/newline fixtures and full browser pipeline cover this boundary;
  generated pseudo rows are traversed by the implementation but do not have a
  separate pseudo fixture in this slice.
- 2026-10-07: `mere-forms-accessible-leaves-disabled` disables only pruning,
  preserving committed-value decoration. It qualifies with 83 passes and four
  intended failures: one neutral, one browser and two native (including nested
  focus). Source bytes/mtimes remain stable and the original files restore.
  The first restored four-package run is unqualified because Windows rejects
  the native library executable at DLL initialization (`0xc0000142`) before
  any tests run. Preserve that attempt as a launch failure, not fixture evidence.
  The unchanged `mere-forms-accessible-leaves-restored-retry` qualifies all ten
  result blocks: Cambium 252, Rootstock 72, native adapter 23 and browser mirror
  10 passes, zero failures and two existing ignored doctests. The fresh Wasm
  accessibility-example compile also passes. Both guards restore the original
  manifest/lock and preserve all unowned source bytes/mtimes.
- 2026-10-07: F5 source is committed in Turnstone `c951afa` and Cleromancy
  `523d54d` with fixture correction `b6b521d`. View builders copy only their
  existing visible labels; Mere retains sole ownership of its value marker.
  Native tag and explicit marked-DIV recognition coexist for the current
  public pins. Sibling source gates run outside checkout-local Cargo overlays
  and use each existing lock without a dependency repin. Qualification follows.
- 2026-10-07: Read-only upstream checks find published source divergence in
  Mere and a documentation-only divergence in Turnstone. Clean merge-tree
  previews precede local main integration of exact Mere `45f5a80c` and
  Turnstone `50d44f63`. Preserve the incoming owner work. Mere integration
  `2a89d8dc` brings command/edit-history changes and two workspace lock rows,
  so historical consumer passes are not carried as its positive receipt.
  Fresh starting/candidate metadata compare all 1,566 resolved packages and
  nodes: equal versions, dependency definitions, edges and features after
  authorized revision/checkout normalization. The 1,690-row frozen lock has
  SHA256 `06BCF3765C5FDA5282F24C4D675A919E297A3587AA4D9BFFCB0CB928DB4D1EF9`.
  Both legacy identities remain; neither JavaScript engine activates. The
  pruning-control production/fixture files are byte-identical to the accepted
  negative control. Fresh integrated gates pass Cambium 257, Rootstock 72,
  native accessibility 23 and browser mirror 10, with zero failures and two
  existing ignored doctests; the six native host targets pass all 63 tests and
  the Wasm accessibility example compiles. Source guards pass and original
  manifest/lock inputs restore exactly. No browser operation or human AT
  receipt is inferred from the Wasm compile.
- 2026-10-07: Turnstone's initial F5 library attempt fails compilation because
  its new CSS selector contains unescaped Rust string quotes; preserve that
  attempt as unqualified. Correction `609bba1` uses the valid unquoted CSS
  attribute identifier. The documentation-only merge yields `8e06a85f`;
  corrected metadata and the fresh complete library qualify 674 passes, zero
  failures and nine existing ignores, including marked-DIV Knot discovery and
  Sky's Civil date name. All 70 Mere packages use public `f1d169c7`; all 29
  Genet packages use public `965b64e2`. Preserve the unrelated untracked
  `.github/` directory and all source bytes/mtimes. Luna's read-only sibling
  review is limited source evidence; the subsequent compiler gates catch API
  compatibility errors in Cleromancy that the review missed.
- 2026-10-08: Cleromancy's F5 name update initially calls `.attr` on `MapState`,
  then on `OnKey`; neither method exists at its current public Mere pin. Preserve
  both compile failures as unqualified. A small consumer-owned `NamedText` view
  sets only the existing visible name on Mere's field node, forwards its retained
  lifecycle/messages unchanged and preserves Mere's representation/marker
  ownership. It adds a direct edge to already-locked Meristem 0.2.0 in the same
  Mere `8106c7c` family. The 1,005 lock packages retain all versions, sources and
  checksums; the 668-package resolved graph retains every identity and feature,
  with only that direct declared/resolved root edge added. Lock SHA256 is
  `6040B3E0030A9114C42C2A83E6E67A3CA1D12753308CA87467264D19EFA0F3B1`.
  Preserve the interim borrowed-setter and test-only-node-import compile
  failures as unqualified. Final source `85c8f77` uses the existing generic
  `LayoutDomMut` node type. `cleromancy-forms-generic-public-metadata` qualifies
  all 36 Mere packages at public `8106c7c` and all 20 Genet packages at public
  `34626a6c`. The fresh routing-disabled control qualifies 11 passes and exactly
  the intended marked-DIV Question-slot failure, then restores source bytes
  with a fresh mtime. The fresh restored gate passes all 12 library tests, the
  authoring DOM test and both consultation DOM tests: 15 passes, zero failures,
  ignores or filtered cases. Source/config/lock guards pass. These are runner
  DOM checks; H4's earlier desktop receipt retains its own source snapshot.
- 2026-10-08: Integrate published Mere `f67f5080` at `9105b1ef`, preserving
  Pelt, dataset and Knot-owner changes; Turnstone's `56aff960` integration at
  `f6149b3` contains documentation changes only. Turnstone's qualified source
  bytes are unchanged, so its 674-test result carries. Mere's published Knot
  repin removes its older d851 identities from the starting lock. Prepare a
  new candidate lock preserving Knot's explicit 965 source while adopting e84
  for Mere's modern family. Versions and modern graph definitions/edges/features
  remain equal after revision mapping; only the two protected identities split
  out. Package count is 1,564 to 1,566 and lock rows are 1,688 to 1,690.
  Fresh Mere gates qualify 362 package and 63 native passes plus Wasm compilation.
  The first package attempt's GPU stall is preserved as unqualified; only its
  owned test executable is stopped, cleanup restores inputs, and the complete
  unchanged serial retry passes with all GPU cases. The current two-file review
  patch passes an application check; exact candidate byte hashes and protected
  sources are recorded in `mere-forms-current-knot-repin-review-proposal.json`.
  No source substitution or public-source acceptance is inferred from these
  local-candidate gates. Checkpoint A publication remains the open decision.
