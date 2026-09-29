# Generated text: before and after

**Status (2026-09-29):** inline string/attr content and named decimal counters
implemented; focused counter gates pass, with the full package gate recorded
below. Scripted generated-name consumers pass on both engines. This remains a
bounded increment of rendering program Row 17.

## Scope and ownership

Implement CSS string lists and `attr()` in `content`, `normal` and `none`,
terminal `::before` / `::after` selectors (including legacy single colons),
independent inherited pseudo styles, and inline generated text through the
existing Buckram provenance and Livery text path. Source DOM stays unchanged.
Cadency owns selector parsing; Livery owns values/cascade; genet-livery owns
generated boxes and shaping. AccName consumes resolved text rather than parsing
CSS or inventing a second content source.

Reference: CSS 2.2 sections 12.1 and 12.2,
<https://www.w3.org/TR/CSS22/generate.html>. This is a bounded implementation,
not full CSS Generated Content conformance. The September 29 section adds named
decimal counters. Quotes, images, alternative text, block/positioned pseudo
layout and pseudo CSSOM remain open.

## Phases and done-conditions

1. Parse and cascade: token-based pseudo selectors never match the owner as an
   ordinary element; string escapes and attribute values retain their meaning;
   unsupported content grammar is rejected as a whole.
2. Render: inline generated text precedes/follows real children, inherits from
   its owner, wraps with surrounding text, and updates after attribute/style
   mutation. Named generated boxes preserve owner/pseudo provenance.
3. Accessibility: the AccName lane reads the same resolved text through a
   bounded provider seam; a joint test proves ordering without DOM mutation.
4. Verify focused parser/style/layout regressions and record any unavailable
   build or broad WPT gates. Do not claim full counters or pseudo-box acceptance.

## Findings

- 2026-09-26: Buckram already represents Before/After box origins, but Cadency
  rejects these selectors, Livery lists `content` as unimplemented, and Livery's
  text collector ignores non-marker pseudo boxes. All three seams need wiring.
- Existing unrelated out-of-flow layout edits are preserved. This lane avoids
  changing their positioned dispatch and uses independent inline pseudo text.

## Progress

- 2026-09-26: inventoried selector, cascade, generated-box and text consumers.
  Cadency now parses terminal generated pseudo selectors; Livery resolves string
  and attribute content with independent inherited pseudo styles. Buckram origins
  feed inline shaping and retained painting without adding DOM text nodes.
- Attribute changes, empty owners, wrapping, custom-property inheritance,
  display:contents, hidden ancestors and hidden pseudo text have focused coverage.
  Generated glyphs are excluded from DOM selection offsets. The static document
  accessibility projection consumes the same resolved text. This September 26
  receipt preceded the scripted style-bearing provider added September 29.

## Validation receipt (2026-09-26)

- `cargo test -p cadency -p livery --offline`: passed selector, value, cascade and
  integration suites.
- `cargo test -p genet-livery --lib --test generated_text --offline`: 279 library
  tests and 6 generated-text integration tests passed. Paint tests compare glyph
  identities and positions with literal-text reference documents.
- `cargo test -p genet-render -p genet-documents --features genet-documents/livery
  --lib --offline`: 37 render and 52 document tests passed, including the joint
  generated-name test and shared-boundary editor test. Existing feature-gated
  unused-import warnings remain.

These are focused receipts from the shared working checkout, not a new full WPT
census or a published clean-clone receipt. Existing repository target was reused.

## Remaining gates

Quotes, images, alternate text, block/positioned pseudo layout, complete pseudo
decoration and CSSOM remain open. The admitted rendering shape is inline,
static, non-floating text. Counter exclusions are listed in the September 29
section. Full conformance and headed visual/assistive-technology acceptance
remain separate gates. Scripted generated names now pass on Boa and Vano;
Mere consumer adoption is recorded in the accessible-name plan after promotion.


## Named decimal counters (2026-09-29)

**Status (2026-09-29): bounded implementation and package regression gates pass.**
This extends the inline generated-text slice above.
It does not expand the admitted pseudo-element geometry.

### Research and scope

The implementation reference is the [CSS Lists and Counters Level 3 editor's
draft](https://drafts.csswg.org/css-lists-3/), retrieved 2026-09-29, section 4.
The published [17 November 2020 working draft](https://www.w3.org/TR/2020/WD-css-lists-3-20201117/#auto-numbering)
was inspected but its parent-name-first inheritance rule is superseded for
this implementation by the current draft's creator-qualified rule. Decimal integer
representation follows [CSS Counter Styles Level 3](https://www.w3.org/TR/css-counter-styles-3/#decimal).
These references do not constitute a conformance receipt. Tests below state
expected outcomes for the admitted subset, independently of implementation
state and without copying an implementation from another engine.

The first slice admits `counter-reset`, `counter-increment`, and `counter-set`
with named counters and optional literal integers, together with `counter()`
and `counters()` in existing inline `::before`/`::after` content. The optional
style argument may be `decimal`; omitted style is decimal. Named counters are
case-sensitive. Reversed counter syntax and unsupported style arguments reject
the entire declaration instead of silently choosing a different meaning.

Properties participate in Livery's existing cascade and CSS-wide keyword
handling. Counter-reset creates an instance, increment changes it, and set runs
after increment. An omitted integer is zero for reset/set and one for increment.
Duplicate reset names honor the final reset; repeated increments accumulate;
repeated sets finish at the last value. The implementation range is i32, with
saturating arithmetic rather than overflow, wrapping or panic.

### Ownership and implementation

Livery owns parsing, typed counter operations, serialization and generated
property metadata. Genet-livery owns counter instance identities and values.
The state is resolved after style cascading, then written into the existing
generated-text plane. No source nodes, anonymous layout nodes, selection byte
offsets or new text rendering paths are introduced. The previously admitted
generated-box and text-shaping path consumes the resolved strings. Accessible
names consume the same strings through the existing retained-text provider.

Counter sets carry a name, creator identity and value. The evaluator preserves
the distinction between inheriting counter identities from the previous sibling
(or parent if there is no sibling) and inheriting values from the immediately preceding element in flat
tree order. Descendant increments therefore affect subsequent siblings while
nested resets remain nested. A later sibling reset replaces its predecessor's
same-level instance. Name and creator both qualify inherited instances, so a
parent's counter does not erase a same-named sibling's nested instance. Owner, before pseudo, children and after pseudo are visited
in that order. Pseudo creators remain distinct from their source owner.

`display:none` suppresses the entire subtree's counter effects. An absent or
unadmitted pseudo has no effects. `display:contents` owners do not themselves
operate counters under the pinned no-box rule; their admitted pseudo/element
children remain eligible. Visibility-hidden boxes still operate counters.
Replaced elements retain their own operations but their descendants and pseudos
are excluded consistently with the current generated-box admission.

The evaluator runs once after a full cascade or a batch of partial subtree
restyles. This is necessary because an earlier element's mutation can affect
generated strings outside the restyled subtree. It does not recascade unchanged
elements. When no admitted generated content reads a counter, the pass returns
before traversing the tree. Its cost is proportional to visited nodes and
active counter sets, not constant-time incremental counter maintenance.

### Phases and done-conditions

1. Parse and cascade: properties serialize with explicit defaults; escaped
   names, negative values, duplicate operations and decimal functions work;
   unsupported styles/reversed syntax and malformed items reject.
2. Resolve: fixtures establish nested/sibling scope, previous-descendant values,
   owner/before/after order, missing counter zero, no-box suppression and i32
   saturation. A partial mutation must update later readers and match a fresh
   document.
3. Render: a numbered fixture must match literal-reference glyph identities
   and positions. DOM text and selected source text must remain unchanged.
4. Validate focused Livery and Genet-livery suites; record the exact commands,
   source boundaries and remaining gates. Parent owns documentation index and
   combined ledger changes.

### Findings (2026-09-29)

- Counter-reset/increment were metadata-only unimplemented entries, counter-set
  had no entry, and content parsed strings and attr only. No named counter state
  evaluator existed in the generated-content path.
- StylePlane already retained pseudo styles with resolved strings. Updating
  those strings reuses both rendering and accessible-name ownership boundaries.
- Partial restyles operate on disjoint roots; a counter pass inside each root
  would miss following readers or repeatedly recompute incomplete state. The
  hook therefore runs after the batch.
- The current editor's draft section 4.4.1 differs from the published 2020 TR.
  The fixture with parent `n=10`, sibling reset `n=3`, following increment and
  another reset explicitly requires `10.3`, `10.4`, `10.8`. This guards creator
  identity independently of the old name-only inheritance rule. The related
  [CSSWG issue 10491](https://github.com/w3c/csswg-drafts/issues/10491) records
  the discrepancy in older scope descriptions/examples.

### Progress

- 2026-09-29: research and bounded ownership agreed; typed properties, decimal
  content functions, flat-tree counter pass, invalidation hook and independent
  integration fixtures implemented. From the Code workspace root, using
  `CARGO_TARGET_DIR=C:/t/cargo-targets/genet`, these gates passed:
  - `cargo +1.97.1 test --manifest-path repos/genet/Cargo.toml -p livery --lib --offline -j 2`: 10/10.
  - `cargo +1.97.1 test --manifest-path repos/genet/Cargo.toml -p genet-livery --test generated_counters --offline -j 2`: 8/8.
  The incremental fixture changes an earlier item from one increment to fifteen
  in a separate fixed-height block. The later block's resolved value and glyphs
  match a fresh document; removal also matches fresh glyph identities and
  positions. This did not expose a retained-layout admission defect, so that
  code remains unchanged.
  - `cargo +1.97.1 test --manifest-path repos/genet/Cargo.toml -p genet-livery --offline -j 2`:
    full package passes, with 620 tests passed and six pre-existing K6
    continuation-contract tests ignored. This includes 291 library tests,
    329 integration tests across 43 files, and the documentation-test target.

### Explicit remaining gates

Parent integration also passed the full Livery package, including generated
property/catalog roundtrips and cascade/value integrations:
`cargo +1.97.1 test --manifest-path repos/genet/Cargo.toml -p livery --offline
--target-dir C:/Users/mark_/Code/repos/genet/target -j 2` from the Code root.
The existing repository target was reused. The native decimal-counter/name
joint fixture and 39 render tests are recorded in the accessible-name plan.

Implicit `list-item` counters, HTML start/value/reversed numbering, `::marker`
integration, reversed counters, custom or additional built-in counter styles,
style containment, counter animation, integer calculations, unsupported pseudo
box shapes, full WPT and headed visual/assistive-technology acceptance remain
open. The existing list-marker renderer is not replaced or claimed covered.
All builds use the shared `C:/t/cargo-targets/genet` target; no isolated target,
Cargo home or worktree is required by this lane.
