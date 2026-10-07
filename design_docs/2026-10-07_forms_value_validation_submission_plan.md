# HTML forms: value state, validation and submission

**Status, 2026-10-07:** Phase A authorized and assembling its starting gates.
Phase B and C remain behind separate checkpoints. No Forms qualification or
publication is claimed yet.

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
their own scope. The password bullet-count question is pending; it has no
response timer and dependent implementation waits for the answer.

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
