# HTML forms: value state, validation and submission

**Status, 2026-10-07:** Phase A implementation in progress; starting runner,
behavior controls and whole-directory baseline are frozen. The focused arena,
native-document and Livery repair gate passes all 449 tests. Password cluster
masking and Cambium migration are approved and being implemented; full candidate
qualification and matched WPT after maps are pending. Phase B and C retain separate checkpoints.
Forms Phase A is not accepted or published.

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
