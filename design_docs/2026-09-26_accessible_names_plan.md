# Accessible names and descriptions

**Status (2026-09-26):** implemented bounded DOM slice and retained inline
generated-text integration; focused, joint semantic and Ortet host-lowering
gates pass. Full AccName/HTML-AAM conformance and headed
assistive-technology acceptance remain open.

## Sources and ownership

The algorithm reference is [AccName 1.2, 23 September 2026 working
draft](https://www.w3.org/TR/2026/WD-accname-1.2-20260923/), especially ordered
ID references, traversal termination and description precedence. Native labels
and image alternatives follow the corresponding sections of
[HTML-AAM](https://www.w3.org/TR/html-aam-1.0/), consulted 2026-09-26. The latter
is a moving reference, not a claim of whole-document conformance. The tests
encode independently stated expected strings for this bounded subset; they are
not an imported WPT census or evidence for the unimplemented rules.

`genet-render` owns the DOM-to-semantic projection. `document-session-api` owns
the neutral name/description fields. Ortet owns lowering those fields to its
installed AccessKit tree. Mere's Reader produces its own neutral projection;
its constructor adoption is pending the Genet revision update described below. Application meaning,
speech synthesis and assistive-technology policy remain outside these owners.
This work does not depend on the JavaScript runtime, whether Boa or Vano.

## Phases and done-conditions

### DOM alternatives

Replace inherited-label/direct-text naming with a document-local ID index,
ordered reference lookup and native label association. A valid but empty
reference must suppress a lower-priority author label; invalid references may
fall through. Reference cycles must terminate, and an ID reference already
being traversed must not recursively chain `aria-labelledby`. A self-reference
still contributes the node's ordinary author/native/content alternative.

Explicit `label for` associations use the first ID match, wrapping labels name
their first labelable descendant, and multiple labels contribute in tree order.
The named control is excluded from its label traversal, so editing its value
does not change its name. Images preserve empty alternatives as meaningful
results. Ordinary button contents collect nested spans and image alternatives.

Done when focused tests cover precedence, missing/duplicate references,
self-reference and cycles, explicit/wrapping labels, non-associated controls,
empty images and nested button contents, without regressing existing tests.

### Descriptions and native lowering

Carry descriptions independently of names and editable values. Prefer valid
`aria-describedby` references, then `aria-description`, then an unused title.
An applicable empty description remains empty rather than falling through.
The additive serialized field defaults when absent from an older projection;
Rust constructors explicitly supply it. Both the direct render lowerer and
Ortet's host lowerer forward the field to AccessKit.

Done when semantic fixtures establish precedence and a direct AccessKit test
checks that a control exposes distinct expected name and description strings.
This is a native tree construction receipt, not a physical screen-reader test.

### Generated text interface

The parallel generated-content lane owns style resolution and rendered text.
The new `document_a11y_projection_with_generated_text` entry accepts an owner
callback returning admitted inline before/after text. It concatenates those
around content, without inserting spaces that the inline boxes do not contain.
Author-provided names retain precedence. Existing projection entries remain
DOM-only. A separate joint fixture exercises CSS, a retained `LiveryDocument`
frame, and its generated-text provider through the semantic projection.

Done when the style-aware consumer provides its retained strings and a joint
fixture verifies CSS through layout to accessible name. Block pseudo spacing,
counters, markers and CSS alternative text require their own admission rules.

## Findings (2026-09-26)

- The previous implementation used `aria-label`, direct child text and inherited
  label context. It did not resolve label IDs or carry descriptions.
- Neutral projections are already partial. This slice preserves that claim.
  CSS visibility, complete role-dependent naming prohibitions, flat-tree and
  shadow-scope references, embedded control values and remaining HTML native
  alternatives are still outside the receipt. DOM `hidden`/`aria-hidden`
  filtering alone does not establish CSS-hidden conformance.
- Shared checkout contains unrelated positioned-layout changes. They are
  preserved and are not part of this slice's acceptance claim.

## Progress

- 2026-09-26: Initial `cargo test -p genet-render --lib a11y --offline` passed
  27 tests, including eight new DOM-alternative tests, in the existing repository
  target. Additional description-lowering and generated-provider tests added;
  rerun initially encountered the parallel style-property edit in progress.
- 2026-09-26: After that edit compiled, `cargo test -p genet-render --lib
  --offline` passed 36/36. Adding the joint CSS-generated-name fixture then
  passed `cargo test -p genet-render --lib a11y --offline`, 31/31. The joint
  fixture proves `attr(data-prefix)` and a literal suffix produce `[Save]`,
  while source DOM text remains `Save`. Contract tests passed 16/16 using
  `cargo test -p document-session-api --lib --offline`.
- 2026-09-26: `cargo test -p ortet --lib a11y --offline` passed 5/5,
  including the host lowerer's separate description assertion. This tests
  native tree publication and retained action routing without a visible window
  or physical assistive technology.

## Remaining gates

At initial implementation, Mere pinned an older Genet/document-session-api revision (`ad20ad...`).
Its two Reader constructor additions are preserved as
[`mere-adoption.patch`](receipts/2026-09-26_accessible_names/mere-adoption.patch),
not applied to that checkout. Apply the patch from Mere's repository root when
Mere adopts a Genet revision containing this description-field change, then run
its document-lanes checks against that committed dependency graph. The source was committed in `b36ac01d3f2` and integrated with newer main in
`04c44e90397`. Downstream owners must select a containing published revision;
local path overrides are not an adoption receipt.

Complete role rules, computed visibility, shadow scope and flat-tree traversal,
embedded controls, generated block spacing, WPT coverage and a headed physical
screen-reader receipt remain explicit future work. Static Livery session wiring
uses the retained provider; the scripted session currently exposes retained
fragments without the style plane, so generated text in that projection needs
an additional CSSOM-owner query before adoption. No new worktree, isolated
Cargo home or isolated target was created.


## Main integration (2026-09-27)

The merge preserves the remote text-field label/value fix and its regression
fixture: an input holding text as children keeps its wrapping label as name and
exposes the child text as value. TextField-role content is excluded from the
shared name fallback. Updated rendering dependencies and selector click delivery
from origin main are retained.

Downstream Taproot can consume `DocumentA11yProjection::nodes()` from the public
`genet_render::document_a11y_projection` family. Each node's `id.get()` matches
`LayoutDom::opaque_id`; `role`, `name` and `description` are owner-computed.
Explicit class/text selector policy remains downstream. This does not expose
private name traversal or make the scripted generated-text gap disappear.

Focused integration gates on `04c44e90397` passed: genet-render 38, genet-documents
with livery 52, document-session-api 16, genet-livery library 279 and generated
text integration 6, genet-text 2 unit and 3 official-corpus tests, Ortet a11y 5.
Commands used `CARGO_TARGET_DIR=C:/Users/mark_/Code/repos/genet/target` and
`--offline`. Existing repository sibling path patches were active; this is
local integration evidence, not a clean-clone or physical AT receipt.
