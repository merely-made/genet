# Accessible names and descriptions

**Status (2026-09-29):** bounded DOM names/descriptions and retained generated
text implemented, including scripted sessions on Boa and Vano. Focused joint
semantic and native lowering gates pass. Cambium's exact-pin adoption is
published and recorded below. Full AccName/HTML-AAM conformance and
headed assistive-technology acceptance remain open.
*(2026-10-14: AccName WPT now runs through genet-wpt's `get_computed_label`
hook on branch `computed-label`. The baseline is 352/405 subtests; see
"WPT `get_computed_label` hook" below. 2026-10-15: family B, role-gated names, is fixed on branch
`role-gated-names`, not merged: 356/405 to 365/405; see "Role-gated names".)*

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
screen-reader receipt remain explicit future work.
*(2026-10-14: shadow scope and flat-tree traversal are fixed on branch
`shadow-a11y-walk`, not yet merged. Corrected 2026-10-14 by the
`get_computed_label` lane: the fix is on main, rebased as `42cabb5a886` and
`bd5f7ca0994`. Name from content follows AccName 2F's
rendered child nodes, and ID references resolve within the referring node's
own tree. See residual 11 and rulings 11.1–11.3 in
[the Shadow DOM plan](2026-09-07_shadow_dom_plan.md#residuals).)* Static Livery session wiring
uses the retained provider. At this September 26 receipt the scripted session
exposed fragments without styles; the September 29 provider closes that gap.
No new worktree, isolated
Cargo home or isolated target was created.


## Main integration (2026-09-27)

The merge preserves the remote text-field label/value fix and its regression
fixture: an input holding text as children keeps its wrapping label as name and
exposes the child text as value. TextField-role content is excluded from the
shared name fallback. Updated rendering dependencies and selector click delivery
from origin main are retained.

**Amended 2026-10-07 by Forms ruling F3:** native input/textarea values now come
from the shared form-control arena record. Authored child text remains default
content and cannot supply a native input's live accessibility value. Cambium's
app fields move to ordinary textbox elements with Mere-owned committed-value
projection, preserving their styled/IME/caret children. The regression now checks
the native arena value and stable label; Mere carries the app-field compatibility
fixture. Implementation and current qualification are recorded in the
[Forms plan](2026-10-07_forms_value_validation_submission_plan.md#f3-cambium-app-owned-fields-and-verified-repin-2026-10-07).

Downstream Taproot can consume `DocumentA11yProjection::nodes()` from the public
`genet_render::document_a11y_projection` family. Each node's `id.get()` matches
`LayoutDom::opaque_id`; `role`, `name` and `description` are owner-computed.
Explicit class/text selector policy remains downstream. This does not expose
private name traversal. The scripted generated-text consumer was added later,
as recorded in the September 29 section below.

Focused integration gates on `04c44e90397` passed: genet-render 38, genet-documents
with livery 52, document-session-api 16, genet-livery library 279 and generated
text integration 6, genet-text 2 unit and 3 official-corpus tests, Ortet a11y 5.
Commands used `CARGO_TARGET_DIR=C:/Users/mark_/Code/repos/genet/target` and
`--offline`. Existing repository sibling path patches were active; this is
local integration evidence, not a clean-clone or physical AT receipt.

## Taproot projection matching (2026-09-29)

`taproot::matching_with_projection(dom, selector, projection)` is the additive
consumer entry for retained semantic targets. For role selectors, it uses only
the supplied projection's role and computed accessible name, then applies any
DOM attribute filter. IDs must belong to this retained DOM and equal its opaque
IDs. Missing, hidden and unknown semantic roles do not fall back to raw DOM
labels or role attributes. Returned nodes retain DOM order; the host owns fresh
geometry and pointer delivery.

Class selectors retain shallow child-text/ARIA-label matching, and `assert text`
retains its existing DOM-text meaning. Legacy `matching`/`resolve` remain for
compatibility. Adopting hosts must opt into the new matcher and make a missing
projection authoritative for semantic selectors. This does not extend the
owner's bounded role or accessible-name coverage.

Integration tests exercise Genet's real projection for referenced, native,
wrapping and nested labels; computed-role overrides; native links/headings;
missing/hidden semantic nodes; DOM compatibility; attribute disambiguation;
and owner-supplied generated names. `cargo test --manifest-path
repos/genet/Cargo.toml -p taproot --lib --target-dir C:/t/cargo-targets/genet -j 2`
passed all 25 tests (21 existing, four new) on 2026-09-29, using Rust 1.97.1.
The command ran from the Code workspace root, outside Genet's ignored sibling
path overrides, and resolved the committed sibling dependency declarations
(including netrender `9607d16f`). The standard Genet target was reused; no
isolated target, Cargo home or worktree was created. Native host adoption and
physical assistive-technology acceptance remain separate gates. HTML `hidden`
and computed CSS visibility coverage are still owned by the projection, not
reconstructed by this matcher.


## Retained generated-name consumers (2026-09-29)

This increment connects scripted sessions and Cambium to the same retained
inline generated text already used by static Livery. `rendered_generated_text`
is a public Genet helper over a DOM and the exact retained style plane, factoring
the existing suppression/replaced-owner/visibility checks. The scripted CSSOM
and document expose additive style-bearing frame callbacks; the old geometry-only
callbacks remain available. `accesskit_tree_with_generated_text` lowers the same
provider through Genet's existing native projection.

Scripted Boa/Vano fixtures pass: names track rendered attribute changes, author
names retain precedence, semantic revisions advance and generated strings remain
outside source DOM selection. All 56 `genet-documents` library tests pass with
`scripted-nova` enabled. The command ran from the Code root using Rust 1.97.1,
`--offline`, and the existing `repos/genet/target`, without sibling overrides.
Cambium must select the same name through Mesquite and native AccessKit using a
published containing revision; its exact-pin receipt follows below. A physical
screen-reader run remains open.

The native bridge also has a joint decimal-counter fixture: retained CSS
`counter(action)` yields `1. Save` on a button, while an author's label keeps
precedence. All 39 `genet-render` library tests pass using
`cargo +1.97.1 test --manifest-path repos/genet/Cargo.toml -p genet-render --lib
--offline --target-dir C:/Users/mark_/Code/repos/genet/target -j 2` from the
Code root. This validates native tree data, not a physical screen-reader run.

The shared provider and scripted consumers are published in Genet
`c5470fcbc12805f0369c70f34a18178158fbe2d5`. The preserved downstream patch
is [mere-adoption.patch](receipts/2026-09-29_generated_names/mere-adoption.patch);
it contains only the Rootstock/provider/native projection and scenario fixture.

Mere adoption is committed and pushed as
`4120607076574ce9b6b6b9a9bccb4c67ec2b6ba7`, selecting that containing Genet pin
in its current root and standalone web manifests. All 207 Rootstock/native
host/accessibility/Mesquite tests and the standalone Graphshell Wasm check pass.
The scenario clicks `[Count 0]`, observes `[Count 1]` through both neutral and
native projections, then changes an attribute and observes `<Count 1]` with the
same DOM identity. Published sources and stable input hashes are verified;
separately qualified legacy Genet identities remain unchanged. Exact commands
and the tested ignored web lock are recorded in
`mere/design_docs/cambium_docs/testing/local-genet-development.md` under
the September 29 generated-name receipt. These are automated consumer/compile
gates; physical AT and browser-hosted interaction remain open.

## Scoped contributed consumers (2026-09-30)

Taproot selectors can explicitly name a host surface family with
`surface:contributed role:button Calculate` or `Selector::on_surface`.
Unscoped ordering and class/text/attribute meanings remain unchanged. A
host-computed Hit naming a different family is refused before the mutating
click hook. Without a matching authoritative Hit, an unknown probe family is
also refused. Miss/Unsupported may still reach a known family's host
deferred click hook; that host owns target identity, scrolling and refusal.

Genet-render publicly exposes `accesskit_tree_from_projection`, lowering the
owner's supplied neutral projection without recomputing labels or geometry.
Turnstone can attach its retained pane bounds once, then share that projection
between native AccessKit and semantic automation. This extends neither the
projection's role coverage nor full live-document semantics.

The exact four source files pass 28 Taproot tests and 40 Genet-render tests,
with zero ignores, using Rust 1.97.1 from the Code root, committed sibling
declarations, `--locked --offline -j 2`, zero dev/test debug info and the reused
`C:/t/cargo-targets/genet`. Commands select `--manifest-path repos/genet/Cargo.toml`
and the respective package. Zero doctests are not an additional gate.
Restoring the scope preflight fault makes the hook-call regression fail
with exit 101; the corrected source is restored and the full Taproot gate
passes. Logs and exact source hashes live in
`Code/testing/turnstone/contributed-semantics/genet-context.json`,
`genet-taproot.log`, `genet-render.log` and
`genet-taproot-preflight-negative.log`.
Turnstone's exact-pin retained/native consumer and human AT receipts remain
separate gates.

## Hidden content in Taproot and the projection (2026-10-05)

*Amends* the "Taproot projection matching (2026-09-29)" section above. That
section kept `assert text` at its existing DOM-text meaning and left CSS
visibility to the projection, which did not check it.

### Report and findings

A Knot-lane session (Conatus, physics, seiche status) reported on 2026-10-05,
from `Code/testing/knot-editor/p1-adapt/` (logs 30–31), that scenario steps
found `visibility:hidden` and `aria-hidden` measurement probes by
`role:button`, and that their text satisfied `assert text`. Verified at
`bf723d5d532`:

- **`taproot::matching` checks nothing hidden.** The legacy DOM matcher
  (`components/taproot/lib.rs:229`) has no hidden check.
- **`resolve` only needs a box.** `resolve` (`:247`) needs a layout box, so it
  skips `display:none`, but it still targets `visibility:hidden` and
  `aria-hidden` elements.
- **`text_present` counts every text node.** It does so (`:279`) even under
  `display:none`.
- **The projection misses only CSS visibility.** `projection_walk`
  (`components/genet-render/src/a11y.rs:658`) drops `aria-hidden` subtrees
  and nodes without a fragment, but not `visibility:hidden` or `collapse`.
  Names (`a11y/name.rs`) treat only `hidden` and `aria-hidden` as hidden; their
  module note says CSS visibility remains open. HTML-AAM and AccName exclude
  CSS-hidden nodes.
- **The projection has no computed styles.** It receives `LiveryLayout`
  fragments only. Style-derived facts already come from the style owner as
  providers (`rendered_generated_text`).
- **Only some consumers use the legacy path.** In Mere,
  `cambium-genet-winit-host`'s harness (`harness.rs:434`) and its tests use the
  legacy `matching`. Mesquite already uses `matching_with_projection`. Mere
  also calls `document_a11y_projection_with_generated_text`
  (cambium-rootstock), `accesskit_tree_with_generated_text`
  (cambium-winit-a11y) and `accesskit_tree` (pelt).

### Rulings (2026-10-05)

1. **What counts as hidden.** Asked what taproot's DOM path (`matching`,
   `resolve`, `text_present`) treats as hidden. The options were: what a
   person perceives; the projection's rule everywhere; retire the DOM path; or
   leave it. Mark: "What a person perceives (Recommended)". It follows that:
   - a click target needs a rendered, visible box;
   - role selectors also skip `aria-hidden`;
   - `assert text` counts rendered text only, and `aria-hidden` text still
     counts because it is on screen.
2. **The projection.** Asked whether genet-render's projection should stop
   exposing `visibility:hidden` elements. The options were: fix it, or leave
   it. Mark: "Fix it (Recommended)".
3. **Who implements it.** Asked who implements this. The options were: a brief
   to the agent, or this session. Mark first chose "Brief to the agent
   (Recommended)", then amended it: "Eh. You can implement it if you wanna
   orchestrate it". This session implements it.
4. **Where visibility comes from.** Asked where the projection gets
   visibility. The options were: a provider from the style owner; or the
   layout records it. Mark: "Provider from the style owner (Recommended)". It
   is read from current styles, so it cannot go stale on a visibility-only
   restyle.
5. **The API.** Asked how the projection's API changes. The options were: an
   additive entry point; or changing the existing signatures. Mark: "Additive
   entry point (Recommended)". It follows that:
   - new `*_with_style` functions take the provider;
   - genet's own callers move to them;
   - the existing functions keep their signatures and behaviour, and Mere
     opts in at its next repin.

### Phase and done-conditions

- **Livery.** `genet_livery::rendered_visible(dom, styles, node)` is false
  when the node or an ancestor computes `display: none`, or when the node's
  computed `visibility` is not `visible`. A text node answers for its parent.
  It sits beside `rendered_generated_text`, with
  `LiveryDocument::rendered_visible`.
- **genet-render.** `A11yStyleQueries { generated, rendered }`, used by
  `document_a11y_projection_with_style` and `accesskit_tree_with_style`.
  - A node that is not rendered visible is not projected, but its children
    are still walked: CSS lets a `visibility: visible` child of a hidden parent
    show.
  - Names treat it as hidden content, so it contributes only when directly
  referenced.
- **genet-documents.** The Livery and scripted engines use the new entry
  point.
- **taproot:**
  - role selectors skip a node with an `aria-hidden="true"` ancestor or self;
  - `resolve` needs a box and `rendered_visible`;
  - `text_present` resolves styles per surface and counts text whose parent
    is rendered visible.

**Done when:**
- fixtures pass on both paths. Each taproot fixture fails on `bf723d5d532`,
  and the genet-render fixture shows the existing entry point still exposing
  the hidden button;
- `genet-livery`, `genet-render`, `taproot` and `genet-documents` tests pass;
- the Knot lane is told the commit, and which Mere call sites to move at its
  next repin.

### Progress

- 2026-10-05: rulings 1–5 recorded. Implementation is in
  `worktrees/genet-hidden-automation`, using a copy of the primary checkout's
  lock, `--locked --offline`.
- 2026-10-05: implemented and verified. The target was
  `C:/t/cargo-targets/genet-hidden`; logs are under
  `Code/testing/genet-hidden-automation/`.
  - **Control.** The four new taproot fixtures were run before taproot
    changed (`genet-hidden-automation-control.log`). All four failed, and the
    28 existing tests passed:
    - a `visibility:hidden` decoy takes neither a role click nor a class
      click;
    - an `aria-hidden` button exposes no role, but a class selector still
      clicks it;
    - role matching skips an `aria-hidden` ancestor's buttons;
    - `assert text` counts only rendered text: `aria-hidden` text counts,
      `display:none` and `visibility:hidden` text does not.
  - **After the change:**

    | Crate | Passed / failed |
    |---|---|
    | taproot | 32 / 0 |
    | genet-render | 41 / 0 |
    | genet-livery library | 317 / 0 |
    | genet-documents, livery and scripted | 55 / 0 |
    | genet-documents, with scripted-nova | 56 / 0, the count recorded above |

    genet-render's new fixture shows `visibility:hidden` content leaving the
    styled projection and its names, while a visible child of a hidden parent
    stays. Its control shows the existing entry point still exposing the
    hidden button and its hidden name text.
  - **Builds and formatting.** `genet-render` checks without `accesskit`.
    `rustfmt --check` is clean for every hunk changed here. The two diffs it
    still reports predate this change: `a11y/name.rs:248`, and the order of
    `genet-render/src/lib.rs`'s `pub use` blocks.

## Scoped host-owned hits without probe inventory (2026-10-14)

**Status (2026-10-14): implemented and verified.** This corrects the September 30 preflight
for hosts whose current geometry is authoritative but whose shared-window DOM
cannot be exposed as an independent per-family probe surface.

### Findings

Turnstone's A4 composition supplies a current `SelectorTarget::Hit` attributed
to the requested pane family. `AutomatableExt::resolve` and `click` checked
`with_surfaces` before honoring that answer, refusing the selector despite a
matching physical target. A fabricated per-pane probe over the shared DOM would
cross owner boundaries. The supplier needs no additional host API.

### Implementation and done-conditions

Resolve honors a matching authoritative Hit before the legacy DOM fallback.
Scoped click validates a host Hit's family before its mutable hook; a matching
Hit does not require probe inventory. Miss and Unsupported retain the existing
inventory preflight for scoped hook admission. A known-family Miss may still
enter the deferred reveal hook, but resolution never falls back to stale DOM
geometry after an authoritative Miss.

Done when focused native tests demonstrate scoped resolution and actual
press/release without probe inventory, preserved hook admission for a matching
Hit, refusal of a mismatched Hit before mutation, and refusal of missing-family
Miss/Unsupported hooks. Existing known-family deferred reveal controls must
continue to pass. Exact commands, source hashes and control logs belong to
[`receipts/2026-10-14_taproot_owned_hits/`](receipts/2026-10-14_taproot_owned_hits/).
Turnstone's physical consumer and Mere's dependency adoption are separate gates.

### Progress and verification

The tests-only pre-fix control on `1ec5d08d16e78f936e9ab366af9f36b7d6abc044`
failed both matching-hit controls: scoped resolution returned `None`, and the
host hook was never admitted. The three other scoped tests passed. After the
repair, all **42 Taproot library tests passed, with zero failures or ignores**.
This includes existing known-family deferred Miss/Unsupported hook controls,
authoritative Miss without stale fallback, and unscoped physical delivery.

Both runs used Rust 1.99.0 from the Code root, committed sibling dependency
declarations, the same offline-generated lock, `--locked --offline -j 1`, and
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`.
`CARGO_TARGET_DIR=C:/t/cargo-targets/genet` and
`CARGO_BUILD_BUILD_DIR=C:/t/cargo-targets/genet/build` reused the stable target.
The manifest was `worktrees/genet-taproot-owned-hits/Cargo.toml`; the command
was `cargo +1.99.0 test --manifest-path <manifest> -p taproot --lib --locked
--offline -j 1`, adding `scoped_` only for the pre-fix control. Rustfmt and
`git diff --check` passed. Source hashes, exact commands, ignored lock copy,
and raw logs are retained in the linked receipt. No isolated target or Cargo
home was created. The worktree isolates the primary checkout's unrelated Forms
work. These are native supplier tests; downstream Wasm, browser-hosted and
physical Turnstone interaction remain consumer gates.

## WPT `get_computed_label` hook (2026-10-14)

**Status (2026-10-14): implemented and baselined on branch `computed-label`,
not merged.**
Mark asked for this after the Shadow DOM residual 11 fix
([Shadow DOM plan](2026-09-07_shadow_dom_plan.md#residuals), rulings
11.1–11.3). Until now, that fix had unit tests only. genet-wpt's testdriver
backend (`ports/genet-wpt/src/harness.rs`, `TESTDRIVER_VENDOR_JS`) implemented
only `action_sequence`, so every AccName WPT test threw at its first
`test_driver.get_computed_label` call. This lane adds the hook, runs `accname/`
before and after, and records the baseline.

### Findings

- **The corpus is 19 files, not 183.** `tests/wpt/tests/accname/` holds 183
  files. 164 of them are `-manual` tests, the `foo.jpg` they use, and
  `META.yml`. 19 are automated: 2 at the top level, 15 under `name/` and 2
  under `name/shadowdom/` (`basic.html`, `slot.html`). 18 of the 19 go through
  `AriaUtils.verifyLabelsBySelector` (`wai-aria/scripts/aria-utils.js:136`).
  It compares the label, ASCII-whitespace-collapsed, against
  `data-expectedlabel`. One file calls the hook directly. None uses
  `get_computed_role`.
- **The role hook is used elsewhere.** `get_computed_role` helpers appear in
  html-aam (12 files), wai-aria (28), svg-aam (4), core-aam (1) and
  graphics-aria (1).
- **The crossing.** The vendor JS queues a request and returns a pending
  promise. Each drive loop drains the queue with `rt.eval`, does the work on
  the host, and settles the promise. `__nodeRawId(el.__ref)` returns
  `NodeId::raw()`. That is what `ScriptedDom::opaque_id` returns
  (`genet-scripted-dom/lib.rs:1619`) and what the projection keys each node
  by (`neutral_node_id`, `genet-render/src/a11y.rs:670`). No translation is
  needed.
- **One production projection call.** genet-documents' scripted session
  (`engines/scripted.rs:217`) builds the projection with
  `document_a11y_projection_with_style` over
  `with_retained_styled_frame_and_dom`. genet-wpt holds the same
  `LiveryCssom`, so it can make the same call. The AccessKit lowering copies
  the neutral name into the label word for word (`a11y.rs:559`).
- **The projection covers rendered elements only.** `projection_walk` makes a
  node for every element that has a fragment and is visibly rendered,
  whatever its role. It skips `aria-hidden` subtrees and leaves out
  CSS-hidden elements.
- **Stale hashes in the residual 11 record.** The fix landed on main rebased,
  as `42cabb5a886` (failing-first test) and `bd5f7ca0994` (fix).
  `690cf560c04` and `c9d8faf7663`, cited in the Shadow DOM plan, are not
  ancestors of main.

### Rulings (2026-10-14)

1. **The role hook.** *Question:* should `get_computed_role` land in the same
   pass? AccName uses it in 0 files, and it needs a table mapping the 55
   neutral roles to ARIA role strings, with judgment calls. *Options:* label
   only, with roles as their own lane; or both now, running the AAM
   directories too. Mark: **"Label only (Recommended)"**. *Follows:* this
   lane implements `get_computed_label` only. `get_computed_role` keeps its
   throwing default, and its table comes to Mark as its own design round.
2. **The label source.** *Question:* which projection should the label come
   from? *Options:* the neutral projection
   (`document_a11y_projection_with_style`); the AccessKit lowering; or a new
   direct name query that runs on any element. Mark: **"Neutral projection
   (Recommended)"**. *Follows:* the hook makes the same call as the
   production scripted session, generated text and CSS visibility included,
   so WPT checks the names a host sees. genet-wpt needs no AccessKit feature.
3. **An element absent from the projection.** *Question:* what should the
   hook do for an element the projection leaves out (`display: none`,
   `visibility: hidden`, `aria-hidden`, no fragment)? *Options:* reject with
   a distinct message; or resolve `""`. Mark: **"Reject, distinct msg
   (Recommended)"**. *Follows:* the hook rejects with "element not in
   accessibility projection". The ledger counts these as their own family,
   separate from wrong-name failures.
4. **The dependency edge.** *Question:* may genet-wpt take a path dependency
   on the in-workspace `genet-render`? *Options:* add it with default
   features off; or route a name query through `genet-scripted` instead.
   Mark: **"Add the path dep (Recommended)"**. *Follows:* `genet-render` with
   `default-features = false, features = ["livery"]`. `image-decode` is
   forwarded through genet-wpt's existing feature.
5. **Name bugs the run finds.** *Question:* does any of the 52
   name-computation failures belong in this lane? The candidates put were
   the input value and alt names (family F) and `aria-label` on a `<slot>`
   (family I). *Options:* none, report only; fix F here; fix I here; fix
   both. Mark: **"None, report only (Recommended)"**. *Follows:* this lane
   ends at the baseline. The ranked list below is open work, and each fix
   gets its own lane, with this AccName run as its before/after instrument.
6. **Recording.** *Question:* should phase 4 also correct the residual 11
   record, whose hashes and merge status are stale? *Options:* record
   everything with dated annotations; or record the baseline only. Mark:
   **"Record all, annotate (Recommended)"**. *Follows:* dated annotations in
   the Shadow DOM plan, this plan's "Remaining gates" note, and DOC_README's
   two rows. The original wording is kept.

### Phases and done-conditions

1. **Design.** Done 2026-10-14. The rulings are above.
2. **Implement, failing first.** A genet-wpt test page calls
   `get_computed_label`. Before the hook it rejects. After it, the call
   resolves to the AccName-correct string for an element whose name is known
   (the control). A deliberately wrong expected string must still fail, and
   an element absent from the projection must reject with the ruling 3
   message.
3. **Baseline.** Run `accname/` before and after, recording per-subdirectory
   pass counts. `name/shadowdom/basic.html` and `slot.html` are named
   explicitly. Each failure family is listed with a representative test,
   with harness failures kept apart from name-computation failures.
   Receipts go under `Code/testing/genet/wpt-ledger/`.
4. **Record.** Findings and the baseline go here and into DOC_README's row.
   The stale residual 11 hashes are corrected with dated annotations. Bugs
   the run finds in name computation are reported as a ranked list, not
   fixed here.

### Progress

**2026-10-14.** Phase 1 done: design reported, four rulings recorded.

**2026-10-14.** Phases 2 and 3 done on `computed-label`. Failing first: all
four subtests of `test_driver_get_computed_label_reads_the_projection` threw
`TypeError: not a callable function` before the hook. After it, both known
names resolve, the wrong expectation fails on the name, and the hidden element
rejects with the ruling 3 message. One harness change was needed beyond the
hook. In the virtual-clock drive loop, a turn that settles a label no longer
advances the clock: the next `promise_test`'s request would otherwise meet its
own timeout first. genet-wpt: 72 passed, 2 ignored. `accname/`: **0/405 →
352/405** subtests, 0 → 5 all-pass files; `name/shadowdom/basic.html` 2/2,
`slot.html` 2/4. The receipt and failure families are in
`Code/testing/genet/wpt-ledger/2026-10-14_computed_label/results.md`. Phase 4
(recording the baseline and families here) waits for Mark.

### Baseline (2026-10-14)

From `Code/testing/genet/wpt-ledger/2026-10-14_computed_label/`, with the
dev-profile genet-wpt binary on Boa, `testharness accname --jobs 1`. 178
files are enumerated; 159 manual files are skipped and 19 run.

| Directory | Files | Before | After | All-pass after |
|---|---:|---:|---:|---:|
| `accname/` (top level) | 2 | 0/11 | 8/11 | 1 |
| `accname/name/` | 15 | 0/388 | 340/388 | 3 |
| `accname/name/shadowdom/` | 2 | 0/6 | 4/6 | 1 |
| Total | 19 | 0/405 | 352/405 | 5 |

Residual 11's pair: `shadowdom/basic.html` 2/2, `slot.html` 2/4 (family I
below). One subtest is a projection failure, not a wrong name:
`<dialog open>` is not in the projection and rejects
(`comp_name_from_heading.tentative.html`). One file errors on both sides
with a buckram layout panic (`comp_name_from_content.html`, 79
`data-expectedlabel` cases, not counted in the 405): `atomic root … was
formatted against AtomicBasis { width: 67.58594 } but sits in
AtomicBasis { width: 64.38594 }`.

The 52 wrong-name subtests are open name-computation gates (ruling 5),
ranked by how often real pages would meet them, then by count:

1. **F.** `input[type=button/submit/reset]` value and `input[type=image]`
   alt are not used (4). `comp_host_language_label.html`.
2. **B.** Name from content is used for roles that forbid it (group, dialog,
   alertdialog, article, table, abbr), so heading, caption or `title` is
   never reached (9). `comp_name_from_heading.tentative.html`,
   `comp_tooltip.html`, `comp_labeledby_non_standard.html`.
3. **E.** Whitespace across nested inline elements is dropped (5):
   `button label` → `buttonlabel`. `comp_text_node.html`.
4. **K.** The layout panic above, which hides a whole file.
5. **A.** An embedded control's value inside a label is not used (13).
   `comp_embedded_control.html`.
6. **I.** `aria-label` on a `<slot>` replaces its slotted content in name
   from content (2). `shadowdom/slot.html`. This is a follow-up to residual
   11.
7. **H.** Hidden-content edges (2): a visible child of a `visibility:hidden`
   parent is dropped, and a hidden `aria-labelledby` target does not fall
   back to `aria-label`.
8. **G.** `aria-owns` is not supported (3). `aria-owns.html`.
9. **D.** CSS `content` alt text (`content: "" / counter(seq)`) is ignored
   (6).
10. **C.** `::marker` text is missing from names (8; the tests are
    tentative).

Instrument notes:
- The hook answers from the same projection call a scripted host makes, so
  these results are what a host would see, not a separate name query.
- `get_computed_role` still throws (ruling 1), so the AAM directories
  remain unmeasured.

**2026-10-14.** Phase 4 done. Rulings 5 and 6 are recorded, the baseline and
ranked gates are above, and the residual 11 record is annotated.

## Input button and image names (family F, 2026-10-15)

**Status (2026-10-15): implemented and measured on branch
`input-value-names`, not merged.** Mark asked for this lane: family F, ranked
#1 in the Baseline above, using the AccName run as the before/after
instrument.

### Findings

- `Names::compute_inner` (`components/genet-render/src/a11y/name.rs`) had no
  host-language step for `input`. Its name-from-content branch blanks every
  `input`, so `value` was never read, and the `alt` step was gated on
  `tag == "img"`. Only `title` remained, so all four WPT inputs got `""`.
- Not fixed here, reported: `document_role` (`a11y.rs:115`) maps every
  `<input>` other than checkbox and radio to `TextField`. That includes
  `button`, `submit`, `reset` and `image`, so a host announces "Submit, text
  field". It belongs with the `get_computed_role` design round (ruling 1 of the
  hook section).
- `value` is exposed as the control value only for text-like types
  (`text_control_value`, `a11y.rs:130`), so the four types never expose it
  twice.
- No automated test besides `comp_host_language_label.html` covers these
  inputs. Manual AccName case 543 expects "Reset" for a bare `input[type=reset]`.
- The HTML-AAM order was taken from memory and the spec text was not checked
  locally. Treat the fallback order as unverified against the spec.

### Rulings (2026-10-15)

Mark: **"i accept the recommendations"**, on the five forks as put (the first
option of each).

1. **Default for submit and reset with no `value`.** *Options:* English
   "Submit" and "Reset"; return `""` and build a localization hook first;
   "Submit Query" for submit. Mark: recommendation 1(a). *Follows:* "Submit"
   and "Reset", English, in one function, `default_button_name`.
2. **Image input fallback order.** *Options:* alt, value, title, then
   "Submit Query"; alt, title, then "Submit Query"; alt, title only. Mark:
   recommendation 2(a). *Follows:* an empty `alt` or `value` counts as absent.
3. **Precedence.** *Options:* `aria-labelledby`, `aria-label`, `<label>`, then
   value or alt; or skip `<label>` for these types. Mark: recommendation 3(a).
   *Follows:* the existing label step stays ahead of value and alt.
4. **Value exposure.** *Options:* name only; or also expose as the value.
   Mark: recommendation 4(a). *Follows:* no change to value exposure.
5. **Scope.** *Options:* name only; or also map the four types to `Button`.
   Mark: recommendation 5(a). *Follows:* the role bug is a separate lane.
6. **Localization.** Mark: **"But note the localization step as unfinished"**.
   *Follows:* the default strings are English only. This is open work, noted
   at `default_button_name` in code and below.

### Open

- **Localization of "Submit", "Reset" and "Submit Query" is unfinished.**
  There is no localization table in genet-render; the strings need a hook once
  one exists.
- The `TextField` role for button-like inputs, above.
- The fallback order is to be verified against HTML-AAM.
- Title is the name for a button input with no value or alt, and so is not
  repeated as its description.

### Result

Failing first: `button_inputs_are_named_by_value_alt_title_then_a_default` and
`button_input_names_yield_to_aria_and_labels_and_title_stays_a_description`
failed before the fix (a `<button>` control and a deliberately wrong string,
`#[should_panic]`, are in the same module). AccName WPT, dev-profile
genet-wpt on Boa, `testharness accname --jobs 1`:

| | Before | After |
|---|---:|---:|
| Subtests | 352/405 | **356/405** |
| `comp_host_language_label.html` | 83/88 | 87/88 |

The four `input[type=button|image|reset|submit]` subtests moved fail to pass.
There were no other transitions. Receipts:
`Code/testing/genet/wpt-ledger/2026-10-15_input_value_names/`
(`transitions.txt`).

## Role-gated names (family B, 2026-10-15)

**Status (2026-10-15): implemented and measured on branch `role-gated-names`,
not merged.** Mark asked for this lane: family B, ranked #2 in the Baseline
above, using the AccName run as the before/after instrument.

### Findings

- `Names::compute_inner` (`components/genet-render/src/a11y/name.rs`) had no
  role test. Name from content was the default for every role, apart from
  inputs, the document, and the tags `html body main article nav form` when
  not referenced. `alertdialog` is not in the neutral `DocumentA11yRole`. There
  was no caption step and no heading step.
- AccName 2F lets a node take content when its role allows it, or when it is
  referenced or inside a content traversal. So the gate applies to the node
  whose name is asked for, not to its descendants. A barred role inside a
  `<button>` still contributes its text. The code threads a `descendant` flag
  for this.
- **Names are Genet's only text channel.** `projection_walk` visits elements
  only, and Genet never emits `StaticText` nodes. The AccessKit lowering copies
  the name into the label, and Taproot's `containing` filter matches the name.
  So for paragraphs, generics, list items and live regions, the name is how a
  host gets the visible text. A full ARIA allowlist would blank it. This is why
  ruling 1 is a denylist, and why a full allowlist waits for projected text
  nodes.
- None of the 18 running AccName files expects a plain div, span or paragraph
  to take a content name, so WPT cannot distinguish the options. The cost sits
  on the host side.
- **`<dialog open>` is absent from the projection** because
  `CAMBIUM_UA_DEFAULTS` (`components/genet-livery/src/lib.rs:128`) has no
  `dialog` rule, so a dialog is `display:inline`. An inline box holding only
  whitespace and block children gets no fragment: `<dialog open><h2>x</h2>`
  projects, `<dialog open>\n  <h2>x</h2>` rejects. Probed with temporary pages
  (not committed). By reading the sheet, a closed `<dialog>` renders its
  contents inline today; that was not rendered here.
- html-aam has no `get_computed_label` tests. Only `accname/` covers this.
- The AccName and HTML-AAM wording was taken from memory. No local copy of the
  spec was read, and nothing was downloaded.

### Rulings (2026-10-15)

1. **Role gate.** *Question:* which roles stop taking content. *Options:* a
   denylist (group, dialog, alertdialog, article, table); the full ARIA
   allowlist; the denylist plus the other author-named containers. Mark:
   **"Denylist (Recommended)"**. *Follows:* `group`, `dialog`, `alertdialog`,
   `article` and `table`, native and ARIA, stop taking content. Everything else
   is unchanged. `alertdialog` is read from the raw `role` attribute in
   `name.rs` only, not added to the shared enum. *Reading, not ruled:* the
   attribute is compared whole, trimmed and case-folded, as `document_role`
   does.
2. **Heading rule.** *Question:* implement the `.tentative` name-from-heading
   rule? *Options:* both roles and native elements; roles only; no rule. Mark:
   **"Implement, both (Recommended)"**. *Follows:* role `dialog`, `alertdialog`
   and `article`, and native `<dialog>` and `<article>`, take the first heading:
   DFS pre-order in flat-tree document order, any depth, hidden subtrees
   skipped, after `aria-labelledby` and `aria-label`, before `title`. No heading
   falls back to `title`, then to no name. **Native `<article>` now gets a
   heading name, which a host sees.** *Reading, not ruled:* a heading is
   anything `document_role` calls a heading (`h1`–`h6` or `role=heading`); a
   heading whose own name is empty falls through; the heading's name is its
   normal computation.
3. **Table and abbr.** *Options:* caption and `abbr` only; also the `summary`
   attribute; all generics title-first. Mark: **"Caption; abbr only
   (Recommended)"**. *Follows:* a native `<table>` takes its first `<caption>`
   child's text after `aria-labelledby` and `aria-label`; an empty caption falls
   through to `title`; cell text is never used. Only `<abbr>` becomes
   title-first (`title`, then content as today). No change to other generics.
   *Reading, not ruled:* the caption and `abbr` steps, like the gate, apply to
   the root of the computation only. Inside a content traversal an `abbr`
   contributes its content. An `abbr` whose `title` names it does not repeat
   the title as its description.
4. **The dialog gap.** *Options:* a separate lane; add the two display rules
   here; project fragment-less inline elements. Mark: **"Separate lane
   (Recommended)"**. *Follows:* the UA stylesheet and layout are untouched. The
   native `<dialog open>` subtest remains a projection rejection, and this lane
   tops out at 9 of 10 reachable subtests.

### Open gates

- **Dialog display rules** (own lane, ruling 4): `dialog { display: block }`
  and `dialog:not([open]) { display: none }` in `CAMBIUM_UA_DEFAULTS`, then top
  layer and centering. It changes layout and paint, so it needs the layout and
  reftest gate. After it, `comp_name_from_heading.tentative.html` should reach
  6/6. `abbr`, `summary`, `details`, `fieldset` and `legend` also have no
  display rule; not examined.
- **Full ARIA allowlist** for name from content: waits until the projection
  carries text nodes (`StaticText`). Until then the other author-named
  containers (region, form, navigation, menu, toolbar, tablist and the like)
  still take content.
- **Block children are joined with no space** in names (`<h2>A</h2><p>p</p>`
  gives `Ap`). Family E territory.
- **`document_role` lacks** `option`, `summary`, `legend`, `abbr`, tooltip,
  `alertdialog`, `columnheader` and `rowheader`.
- Still open from the earlier ranking: E, K, A, I, H, G, D, C.

### Result

Failing first: `roles_that_forbid_content_take_no_name_from_it`,
`dialogs_and_articles_are_named_by_their_first_heading_depth_first` and
`tables_take_their_caption_never_their_cells_and_abbr_prefers_its_title`
failed before the fix. Their controls pass on both sides: a `<button>` with
content, a barred role inside a button, a cell, `<p>`, `<div>`, `role=alert`,
and a `<span title>`. `deliberately_wrong_group_expectation_fails`
(`#[should_panic]`) did not panic before the fix and does after.
AccName WPT, dev-profile genet-wpt on Boa, `testharness accname --jobs 1`:

| | Before (`636da6bbc65`) | After |
|---|---:|---:|
| Subtests | 356/405 | **365/405** |
| All-pass files | 5 | 8 |

Nine subtests moved fail to pass and nothing else moved: `comp_name_from_heading.tentative.html`
alertdialog, article, dialog, group and DFS (5); `comp_tooltip.html` abbr and
`div[role=group][title]` (2); `comp_host_language_label.html` `table > caption`
(1); `comp_labeledby_non_standard.html` group (1). There were no pass-to-fail
transitions. `comp_name_from_content.html` still dies on the buckram panic and
holds no gated role. Rerun after the final rebase onto origin/main (`273431eff73`): still 365/405, 9 transitions against the baseline, none against the pre-rebase run. Tests: genet-render 55 (3 new behaviour tests, 1 wrong-expectation control), taproot 42, genet-documents 9 and genet-wpt 72 (2 ignored) all pass; genet-documents and genet-render were rerun after the rebase. Receipts:
`Code/testing/genet/wpt-ledger/2026-10-15_role_gated_names/` (`transitions.txt`).
