# Generated text: before and after

**Status (2026-09-26): bounded increment implemented and verified.** Bounded first increment of rendering
program Row 17, authorized alongside AccName, segmentation and WebVTT.

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
not full CSS Generated Content conformance. Counters, quotes, images, alternative
text, block/positioned pseudo layout and pseudo CSSOM remain open.

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
  accessibility projection consumes the same resolved text; the scripted retained
  projection still needs a style-bearing provider seam.

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

Counters, quotes, images, alternate text, block/positioned pseudo layout, complete
pseudo decoration and CSSOM remain open. The admitted rendering shape is inline,
static, non-floating text. Full standards conformance, headed visual/assistive
technology acceptance, and scripted generated-name projection need separate
receipts. Mere adoption requires promotion of a containing Genet revision.
