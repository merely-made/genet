# HTML forms: value state, validation and submission

**Status, 2026-10-08:** Genet's Forms value model and F4 repair at
`e84f9c7f9aec23320c539784961d1465f8a53a9b` are published on main, as is Mere's
permanent repin at `632c1d29` through published integration `eafb8643`. Fresh
public-source ThinkPad gates pass 362 Mere package tests and 63 native-host
tests, with zero failures and two existing ignored doctests; its Wasm
accessibility example compiles. F1-F6 are ruled, including password cluster
masking, app-owned Cambium fields and accessible leaves across projections.
The bounded F5 Turnstone edits are published through `1802a683`, after fresh
ThinkPad qualification at product `e77e1e2` with 680 passes and nine ignores.
Cleromancy's current public-pin integration passes 23 ThinkPad tests at
`9789d64e` and is published through `ff3b1f53`. Forms Phase A is published;
Phase B and C retain separate checkpoints, and sibling adoption of Mere's new
Forms dependency family remains distinct from F5 mechanical compatibility.

**Phase B, in progress:** Mark requested "Please continue with forms!" on
2026-10-08 after the completed Phase A publication report. This authorizes
constraint validation through Checkpoint B. Submission remains behind that
checkpoint. Phase B starts from clean published main `6cb2284a33d`; builds and
matched gates remain on ThinkPad to relieve Windows memory pressure.

The eleven-crate 1,943-pass receipt belongs to Genet
`b8a3ec1d6abe88e07438ca4d53b9ca4d2b92111d`, with zero failures and nine ignores.
Matched supplementary 60-second WPT maps have zero starting pass losses;
prescribed 15-second maps retain their Vano selection cutoffs explicitly.

The bounded F4 layout repair at `e84f9c7f` has fresh four-crate qualification
(841 passes) and Mere native consumer qualification (63 passes), followed by
passing catalog, highlighting, restored native-bridge and Wasm compile gates.
The fresh restored four-package Mere suite passes 355 tests. F4's fresh
optimized runner passes all 15 value fixtures on both engines and its CSS guard
reports `unexpected=0`, with all sixteen summaries equal to the starting guard.
The later public-source Mere integration qualifies below. The original
eleven-crate and WPT maps retain their own source snapshot.

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

### F7: reuse the native URL parser, 2026-10-08

Question as put: "One further dependency choice: may `genet-scripted-dom`
also reuse the already-locked `url` 2.5.8 parser for URL validity? The Forms
brief forbids new dependency edges. Its current scheme-only check accepts
malformed values such as `http://`; using the existing parser would keep
package versions and checksums fixed."

Options: use the existing URL parser in the DOM (recommended); keep URL
parsing behind an existing host bridge.

Mark's answer, verbatim: "use the existing one, or if it needs updating,
update the existing one and then use it".

The existing 2.5.8 API can parse these absolute URLs, so add the workspace
dependency to `genet-scripted-dom` and preserve its version and checksum.
URL-derived validity stays in the same native view read by script and CSS.

### F8: native pattern matcher, 2026-10-08

Question as put: "May Forms reuse the already-locked `regress` 0.11.1 library
directly for HTML `pattern` validation? This adds a dependency edge that the
brief forbids, but preserves package versions and lets script and CSS read the
same native result."

Mark's answer, verbatim: "Use the existing native matcher (Recommended)".

Consequence: add that direct edge with UTF-16 support, retaining all locked
package identities, versions and checksums. Compile HTML's UnicodeSets pattern
with both the library's Unicode and UnicodeSets flags enabled; omit malformed
patterns, match the whole applicable value, and check each multiple-email item.
Native validity remains authoritative for script and CSS. The library exposes
no public backtracking execution budget; document this implementation limit
and qualify bounded difficult-pattern fixtures without inventing a new policy.

### F9: native validation notice and accessible alert, 2026-10-08

Question as put: "Should `reportValidity()` show a native validation notice and
a screen-reader alert now? This requires a new host reporting path, including
fields inside iframes. Initial English messages would be replaceable by the
embedder; custom messages would stay exact."

Mark's answer, verbatim: "Include the native notice and accessible alert
(Recommended)".

Consequence: share native message generation between `validationMessage` and
host feedback; expose an embedder-replaceable initial English catalog and
preserve normalized custom message text exactly. Queue reports after uncanceled
invalid events, drain them during the normal document pump/render, preserve
realm and node identity, and paint feedback outside the authored DOM. Expose
the same notice as an assertive accessible alert, including child-frame reports.
Static `checkValidity()` keeps its existing event/boolean behavior. Submission
and new select/checkbox interaction widgets remain outside Checkpoint B.

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

The first bounded audit confirms that native validation is absent. The
input/textarea record already owns custom messages; validation needs native
listed-control coverage, derived flags and barring, live script wrappers,
trusted cancelable `invalid` events, selectors and genuine edit provenance.
Expand native listed controls as the brief requires, distinguishing barred
fieldset/output/object APIs from form-wide methods and aggregate selectors.
Form-associated custom-element internals are a separate existing gap, with
only callback-name recognition implemented today.

HTML fixes user-validity transitions and reset/clone behavior; these are not
new policy choices. Track user edits separately from script assignments so
length constraints and bad-input status cannot be guessed from sanitized
values. Preserve custom validity messages on reset and omit them on cloning;
reset user validity as HTML specifies. A live `ValidityState` getter must read
the same native state used by selectors, including while a control is barred.
Keep the shipped `FormControlState` field layout intact: append separate
capture records for interaction facts, option selectedness and nonlegacy
custom messages. The interaction metadata distinguishes last-change origin
from a pending committed edit and preserves an incomplete number draft with
its internal UTF-16 caret. The editor reads a derived editing view; script
still reads the sanitized value and its applicable selection APIs.

F8 and F9 approve the direct DOM edge to already-locked `regress` 0.11.1 and
a Genet-owned validation notice/accessible alert with replaceable initial
English messages. Their implementation and fresh qualification are in progress.
Both frozen starting engines depend on that matcher, including UTF-16 support.
The newer Boa pin uses 0.12.0 while Vano retains 0.11.1; F8's native
matcher uses already-locked 0.11.1. No package/version change is
proposed for that edge. Source review finds Unicode-set support in both
versions, but `Flags::from("v")` leaves the separate `unicode` flag false.
The native UTF-16 adapter must set both flags and qualify supplementary
characters, set operations and malformed-pattern omission. Its UTF-16
executor exposes no public backtracking budget.
Do not run validation through page-overridable RegExp properties. Do not
insert validation feedback into author-observable DOM.
F7 separately approves the already-locked URL parser edge. The first review
also corrects requiredness versus mutability, button/object/input barring,
radio-group requiredness, email grammar, textarea length flags, optional time
seconds, and reversed periodic time ranges. Those follow HTML rather than
requiring further semantic rulings. These changes are not qualified until
the native tests and matched WPT runs finish.

Starting controls are in
`Code/testing/genet/forms/phase-b-constraint-validation.html` (20 cases).
The fresh Linux corpus, source, lock, runner, environment and before maps must
be bound before implementation gates; Phase A's earlier Windows measurements
retain their own platform/source boundaries. Match baseline and candidate
deadline and worker count, preserve raw partial/error rows, and stop at
Checkpoint B before submission.

The ThinkPad starting maps are complete at `6cb2284a33d`, using the frozen
runner SHA `087c4347...`, the Phase A integration lock SHA `9809247c...`,
Linux corpus inventory SHA `6c9b599f...` and WPT manifest SHA `d5ec5be9...`.
Both engines record the same starting counts: constraints 0/877,
the-input-element 928/1,853, textfieldselection 667/739 and infrastructure
31/120. Those runner totals include partial rows. Completed-file input counts
are 920/1,806; all four directories have 1,618/3,542 completed-file passes
per engine. The input error files separately contain 47 partial rows:
eight passes, seven failures, 28 not-run and four timeout rows.
Each engine has 219 file records: 31 passing, 124 failing, nine
errored and 55 skipped. Assertion rows in errored files remain partial
evidence. All eight runner processes exit zero; that is not suite success.
The 20 added controls fail on both frozen starting engines. A separate
inventory pins all 5,047 tracked WPT resource files, including the harness
resources outside the focused archive. Builds and maps use installed
Rust 1.98.1 explicitly for both stages; the repository's toolchain pin is
unchanged. Runs are serialized with one worker to respect ThinkPad memory.

The in-progress compile preview is identified by source archive
`5cbe1cf0...` atop `6cb2284a33d` and controlled lock SHA `f540a7c8...`.
Its sole lock change adds the approved existing URL edge. This preview is
not Checkpoint B qualification. Its first crate stops at two compile errors
before tests: a sibling-module private helper and an overlapping mutation
journal borrow. The local repair exposes the helper within the arena owner
and snapshots change booleans before recording mutations. Subsequent review repairs public window
redispatch of trusted invalid events using agent-private weak bookkeeping,
tests per-interface receiver checks and snapshots invalid candidates before
any listener runs. These source changes require their own final gates.
Calendar range ordering compares validated components exactly, while step
calculations retain finite f64 precision for enormous years; that remaining
limit is not presented as an HTML rule. Step rounding tolerances scale with
the calculation's floating-point error rather than accepting a fixed small
nonzero remainder.

The repaired preview compiles its arena crate, then stops with 78 passing
tests and four fixture failures. Two radio fixtures used separate detached
trees, the listed-controls expectation omitted its fieldset, and the large
year overflow fixture set `min` instead of `max`. Corrections retain the
actual semantic assertions while fixing those inputs and expectations.
Cross-realm fixtures additionally cover borrowed validation getters,
adoption/clone state and public invalid-event redispatch. Agent-private weak
registries keep one validity object per canonical wrapper and allocate it
with the wrapper's creation-realm prototype. This repair awaits execution;
the new integration tests require completed validation messages as well.

The fixture-repaired preview (`de3aa02b...`, 22 owned files) executes 82 arena
tests with zero failures. Its runtime crate compiles and executes 173 passes
and two failures before later crate gates stop. Both failing runtime cases
append controls directly to a document that already has its document element;
the Vano error is `HierarchyRequestError`. The fixture now loads an HTML body
and appends its form and external control there. Preserve that failed run
and its source/resource guards; the correction needs a new source snapshot.
The selectedness review also repairs size parsing, first-enabled fallback,
reset normalization, multiple removal, option-list barriers and wrapped
placeholder eligibility against the current HTML algorithms. These changes
remain unqualified until their own remote gates execute. Pattern matching
and validation feedback still await the two explicit answers.

The subsequent select/runtime repair freeze (`4352315a...`, the same 22 owned
files and controlled lock) adds selected-option insertion and preserves
selection during moves within one list. Public `option.selected` assignments
ask for reset; `select.selectedIndex = -1` still permits no selection. A
two-selected-option subtree fixture checks that the last inserted selected
option wins without dirtying. The runtime fixture also reads live select
validity through both engines. Later local work adds the private native
`validationMessage` custom-message branch, returning empty for valid or barred
controls, and its interface-brand/adoption fixtures. The built-in catalog
and report feedback remain pending; that partial binding is not full message
conformance and is not part of the `4352315a...` freeze.
The next API preview is `3ef27ae3...`, including the custom-message branch;
its full cross-realm suite can now run independently of built-in wording.
Subsequent native review corrects current HTML button Auto candidacy:
missing/invalid type validates only without command/commandfor and outside
a direct select parent; explicit submit remains a candidate. That correction
and its native fixture are not part of the `3ef27ae3...` freeze and need final
source qualification. A pre-existing Mere/Djinn compiler delays the remote
preview start; it is preserved and is not interrupted by this lane.
The custom-message preview subsequently starts after that owner exits, with
11 GiB available, and stops before tests at a compile error: the new script
option setter calls `is_html_option` from its sibling module. Expose that
existing helper within the DOM crate (`pub(super)`) and retain the stopped
receipt, whose source, lock, WPT/CSS and runner guards pass. The next freeze
must include this repair and the reviewed button Auto correction.

The shared-option-helper preview (`d713e251...`) passes all 89 arena tests
and all 177 runtime library tests with no failures or ignores. Gate three
stops during compilation of Livery: the new attribute-invalidation patterns
omit the existing `old_value` member. Add `..` to both patterns and retain
the stopped receipt with its passing source/resource guards. No gate-three
tests or later gates ran. The next preview also captures the built-in String
conversion for `setCustomValidity`, rejects primitive Symbols as WebIDL
requires, and preserves exceptions from authored `toString` methods. Two
engine fixtures check poisoned global String, newline normalization, one
conversion, exact thrown-object identity and receiver-brand ordering. These
new cases and the two Livery compile repairs require a fresh frozen run;
the unchanged arena preview keeps its separate 89-pass receipt.
Read-only review additionally finds that option-state mutations invalidate
their select but omit its external form owner. Both retained style and
formatting-damage paths now include form owners of the relevant ancestors.
A fixture selects a nonempty option in a required external select, checks
the form's computed width and paint, then returns to the empty placeholder.
The existing user-validity paint fixture now uses `mutate_dom`, handing each
exact recorded batch to the retained document instead of calling a nonexistent
mutable accessor. All of these new Livery checks still await execution.
Before launching the next gate, fixture review catches an unchanged-empty
assignment that cannot set user validity. The fixture now types one character
into a `minlength=2` control before commit: a real user change remains invalid,
then the two-character edit becomes valid. The transferred `c81aed33...`
snapshot is preserved as an applied-only preview with no tests, and replaced
by a separately frozen fixture correction. No provenance algorithm changes.
Further static review borrows the attribute's non-Copy qualified name in the
retained damage match instead of moving it from a borrowed mutation. Keep
the runtime gate's exact source unchanged and apply this correction only at
a stopped gate boundary; it needs the subsequent Livery compilation.
The borrowed-attribute runtime gate executes 177 passes and two new fixture
failures, with no ignores. Both report `Error:1` rather than `TypeError:1`
for the last brand check: its `document.createElement('div')` setup invokes
the intentionally replaced global String before `setCustomValidity` can run.
Create that wrong receiver before poisoning String. Preserve the stopped
run; no later gates ran, and this correction changes the fixture only.
Further read-only invalidation review finds a radio-group dependency:
checking one radio can clear another unchecked required radio's missing
value without changing its checkedness, leaving its separate external
fieldset's style stale. Both invalidation paths now conservatively widen
current-radio state changes and radio name/required/type changes to their
containing DOM tree, considering the old attribute when a type changes away
from radio. Unrelated inputs retain their prior scope. No reverse group index
exists, so this favors correct external-peer and fieldset styling at the cost
of a full-tree restyle for these radio mutations. A retained fixture covers
selection and grouping changes across two externally associated fieldsets.
The previously frozen arena sources stay unchanged; the repair needs execution.
Structural membership changes use the same containing-tree rule when a tree
contains radios, including when it contains no form or fieldset. A fixture
removes and reinserts a checked radio in a separate section from its required
peer, checking the peer's style both ways. Structural edits elsewhere in a
radio-containing tree therefore widen conservatively too; trees without
these controls retain the old scope. These local additions follow the frozen
`f45401dc...` preview and cannot use its unmodified-source receipt.
The `f45401dc...` runtime gate passes all 179 tests without failures or
ignores, including both corrected DOMString conversion fixtures. The runner
then advances to `genet-scripted`; later native rendering and cross-realm
gates remain unqualified until they complete.
That compile stops in Livery before tests: the new helper's `QualName` type
is not imported by `document/frame.rs`. Qualify it as
`layout_dom_api::QualName`; this is a compile-only correction. Preserve the
runtime 179-pass source binding and the stopped capture gate. The standalone
radio-membership archive (`abaca823...`) remains frozen-only, without a
remote application or tests; the next bundle combines membership coverage
with this type-path correction at an idle boundary.
The `0105ecef...` preview compiles Livery and passes all 88 scripted-document
library tests without failures or ignores, including capture compatibility.
Its Livery library gate stops with 304 passes and 18 failures. Ten failures
concern retained-root damage/formatting, two report foreign or retired NodeIds
(including the new radio-removal fixture), and six concern layout geometry or
scroll ranges. Preserve the full failed run and passing source/resource guards;
later package and integration gates did not start. Review invalidation safety
and scope before changing those assertions, and execute a matched clean-source
Livery baseline to attribute the geometry failures rather than assuming they
are platform differences. The stopped evidence archive is `a566f84a...`.
Further API review follows the DOM Standard's event-firing rule: borrowed
validation must create an invalid event in the target's relevant realm.
Register a private realm-local event factory alongside the existing canonical
wrapper validity factory, preserving constructor-override protection and the
agent-shared trust registry. The existing six-case integration target now
checks event realm/prototype and a replaced child Event constructor on both
engines. Those local runtime changes are outside the frozen `0105ecef...`
run and need their own executed control and restored-source qualification.
The negative event-realm control (`7c5e36ed...`) uses unchanged `0105ecef...`
production and only the strengthened integration fixture. It executes six
cases: four pass, while the borrowed-validation case fails once on each
engine. Its source, lock, resource and frozen-binary guards match before and
after. Preserve that failed control; the positive realm-factory repair still
needs execution. A receipt timing-field error is retained with a separate
amendment using its UTC start/end, rather than rewriting the raw receipt.
The matched tracked-source baseline at `6cb2284a33d` and original lock
`9809247c...` executes 318 Livery tests: 312 pass and six fail. Those failures
are exactly the two scroll-range and four float/shape-outside cases from the
preview. Its two new untracked preview files are retained but inert in this
library-only baseline; all tracked source blobs match the commit. None of the
ten retained-root failures or transferred-subtree panic reproduce. The
baseline archive is `5d32f350...`, with unchanged source/resource guards.
This attributes those twelve preview failures separately from the six
pre-existing Linux failures, without altering any baseline assertions.

The next local repair guards validity reads on retired/foreign IDs and keeps
ordinary ID mutations local in trees without form-related dependencies. ID
mutations in form-containing trees still widen, including a non-form element
that blocks a later form's first-ID association. Both selectors now trust a
native validity view's no-owner result instead of recreating ownership from
matching attributes. New engine and retained-style fixtures check duplicate
form IDs and toggling an earlier non-form blocker. Select structural mutations
invalidate their select and external form owner even if selectedness stays
unchanged; a placeholder move into/out of an optgroup checks that case. The
radio-membership fixture moves its node into a detached tree and back, since
the arena's destructive `remove` retires its ID.

Native-editor review also clears stale bad-input drafts and caret/focus
metadata on genuine type-state changes, retaining user-validity and custom
validity. Existing editable date/month/week/time/datetime-local controls now
preserve partial user drafts through the same derived editing view used for
numbers. Script assignments clear those drafts. Length flags explicitly
require dirty value and last-user-change provenance. These corrections and
their three native fixtures await fresh execution; the earlier 89-pass arena
receipt does not qualify these changed native sources.
The combined positive preview is frozen as `690762af...` (22 owned paths,
controlled lock `f540a7c8...`). Its arena gate passes 92 tests with no failures
or ignores. The runtime library passes 181 tests, and all six cross-realm
integration cases pass, including the two event-realm assertions that failed
in the negative control. Later package gates are running. Subsequent review
keeps editing-UI cleanup
separate from value provenance: if a type change leaves the API value unchanged,
retain the last-user-change fact so newly applicable length constraints remain
observable. A new local fixture checks user-entered number `123` changed to
text with `maxlength=2`, then search, and a sanitizing transition that clears
the value. This inference follows HTML's last-value-change condition and type
transition steps; it does not assert additional origin rules for every
conversion. The correction is outside `690762af...`. The fixture-only negative
control `56b0c383...` executes exactly one failing case at the expected
`too_long` assertion after number-to-text, with 92 other cases filtered.
Source and lock guards pass. Its runner misclassifies the expected zero-pass,
one-failure summary, so preserve the original receipt and a separate
classification amendment. The restored positive snapshot `261dd437...`
passes all 93 arena tests with zero failures or ignores. Root verifies the
source manifests, exact fixture/production deltas, raw output hashes, runner,
lock and timing for both controls. The supplemental negative archive is
`eee6bd2f...`, and the positive archive is `dd0fc8c0...`. An incorrectly
reported amendment digest is corrected in a separate verification record;
the amendment and original raw receipt are unchanged.
Concurrent main commit `a26cd7b66734` adopts Boa `494ae680...` and its browser
host feature. It changes no owned Forms files, but the compatible lock now
resolves ICU 2.3 instead of the frozen previews' ICU 2.2. Keep the current
ThinkPad queue at `6cb2284a...` and lock `f540a7c8...`; its results qualify
that tuple only. Do not combine its before maps with a newer-pin candidate as
though dependencies matched. Final publication needs a new frozen starting
runner, lock, focused controls, WPT maps and CSS maps at the updated main
dependency tuple before applying Forms. Preserve the existing old-tuple maps
and controls as separate evidence. The stable Boa lane's adapter/compile
qualification is recorded in `2026-10-08_boa_stable_upstream_refresh.md`;
its aggregate Forms/WPT qualification remains separate.
The new baseline lock is prepared as `fd4a121e...` from the public qualified
878-package resolution, removing only the concurrent native-DOM URL edge.
Its frozen URL-only preview candidate is `28ee4fe8...`; the guarded accounting
confirms identical package identities, versions, sources, checksums and all
other edges. These locks are frozen inputs; current build evidence follows.
Published main `cca45fc7...` subsequently merges the line-breaking owner's
paragraph-boundary index and qualification documentation. Forms' eight-line
draft-display change in `text.rs` merges cleanly with that published code;
all other owned source bytes remain intact. The incoming index adds three
tests and one ignored timing observation. Final starting measurements move
to exact `cca45fc7...` with the same `fd4a121e...` lock before applying Forms.
Fresh locked ThinkPad metadata resolves 874 packages/nodes and confirms the
public Boa, Vano and netrender revisions without config or source overlays.
The fresh optimized ThinkPad starting runner builds successfully at exact
`cca45fc7...` and lock `fd4a121e...`, with binary SHA `ab486cec...` and
build receipt SHA `b64ead5a...`. Cargo reports 22m13s; the receipt envelope
includes preflight and post-build checks. Root verifies the complete 22-member
evidence archive `8ff2775b...`, raw output/exit hashes, metadata pins and resource
identities. Its four-font inventory `e1356987...` supplements the preserved
WPT and Linux corpus inventories. Upstream checkout status contains only
Cargo markers; a separate amendment `0b022bed...` verifies Vano's unchanged
test262 gitlink and empty tracked-source diff. Markers and raw evidence remain
intact. The fresh Livery baseline executes 322 cases: 315 pass, six fail and
one timing observation is ignored. Root verifies the raw results and exact
same six failure names as the old clean-source baseline. Its receipt
`284ee6af...` mistakenly embeds owner JSON in the start-time field; the
separate `fe49ac25...` amendment supplies the owner's timestamp while
preserving the original. The verified wall interval is 306.105 seconds and
test execution takes 8.22 seconds. The unchanged B01-B20 controls subsequently
execute on both engines with 0/20 passes, all twenty named failures and no
errors or skips. Root verifies their 25-member archive `15b69aad...`, raw
results and source/lock/binary/fixture/font bindings. Their constant config
boolean is not a separate per-run config observation; the runner's build
receipt contains that audit. Starting WPT and CSS maps still need to complete;
the Livery result remains a baseline with six known failures.
The starting-map capture review preserves the failed CSS preflight caused by
a missing Git working directory. The corrected preflights verify all 5,047
WPT resources, 60,998 CSS tracked files and four fonts before any maps run.
Root also catches a summary-parser error that preflight does not exercise:
named-group `findall` returns tuples, and the parser reads only the header.
The V4 wrapper `36f79365...` instead checks both header settings and final
stdout totals against each result map. Its replay verifies all eight archived
Forms maps and sixteen CSS maps, with failing controls for the old parser,
incorrect pass totals, duplicate summaries and a missing final summary.
Before and candidate retain identical wrapper bytes with explicit frozen
stage configs. The fresh before runner is separately frozen at 75,877,952
bytes with unchanged binary SHA `ab486cec...`. The V4 preflights pass and all
eight fresh starting Forms maps complete their capture checks. Root's strict
accounting verifies build/font/corpus/provider bindings and raw stdout totals.
Each engine has 219 files: 31 pass, 124 fail, nine error and 55 skip. Completed
files retain 1,618/3,542 subtest passes; errored files separately retain eight
passes, seven failures, 28 not-run rows and four timeouts, making 1,626/3,589
rows including partial results. Status comparison with the old dependency
tuple observes zero movements in all eight maps; it is not candidate acceptance.
The raw archive `7f490671...` contains 99 regular files and one directory entry.
All sixteen fresh CSS slices also complete their capture checks. Root verifies
2,731 file records: 1,044 verified passes, one reference-unverified pass, 872
failures and 814 skips, with zero errors. Their test processes exit 1 for the
baseline failures; the capture stage exits 0 with complete results and passing
guards. Each map records the static renderer engine `none`, despite the
historical `--engine boa` argument. Original expectation files, policy script
and reference map remain unchanged. The `f2160d39...` raw archive contains
215 regular files and one directory entry. Comparing statuses against the old
tuple observes zero movements; final acceptance still requires the matched
Forms candidate. Both fresh starting stages are now ready, and ThinkPad has
no active compiler or runner owner. Matcher and report-feedback choices
remain unanswered without a response timer.
The `690762af...` scripted-document gate also passes all 88 tests. Its Livery
gate executes 324 cases with 318 passes and the exact six clean-source baseline
failures, with no ignores. All ten retained-root regressions and the transferred
subtree panic are repaired, and the six added Forms/invalidation cases pass.
Record the baseline failures as such; this is not a zero-failure Livery gate.
The remaining old-tuple library gates pass nine editor tests, ten Livery
selector tests and 42 render/accessibility tests. The shared layout-DOM
library compiles and has zero tests. All raw hashes and before/after source
and lock guards are verified in the complete queue archive `64ae5f82...`.
The last command mistakenly requests `genet-wpt --lib`, which has no library
target and executes no tests. Preserve that invocation error separately from
test failures. The corrected binary-target gate passes 73 tests with zero
failures and three ignores on unchanged `690762af...` source and lock
`f540a7c8...`. Its source and lock guards pass. This queue still precedes the
final provenance correction and the updated Boa dependency tuple.

Read-only report-feedback tracing identifies the existing native request
pattern `HostState.scroll_into_view`, consumed during scripted layout.
Validation has no equivalent presentation queue, message provider or notice
UI. A timer-triggered report also needs a session drain, since input effects
alone cannot carry it, and child-frame targets require realm identity.
The host would own visible presentation and the transient accessible alert,
outside author DOM. This is seam evidence for the still-unanswered notice
choice, not an implemented queue, catalog or host feature.
The final method audit finds matching static-validation snapshots, negative
results after canceled invalid events and native external-form ownership.
Method calls leave user-validity unchanged; genuine text-edit commits own its
transition. `commit_select_user_selection` has direct native tests but no
production picker caller. Select interaction therefore remains a host seam,
and these API receipts do not qualify a native select picker.

The ThinkPad's faithful starting run of the checked-in 16-slice CSS guard
passes mediaqueries and css-position, then stops at text-align with
unexpected results against the Windows expectation file. That completed
slice has 52 passes, 34 failures, 28 skips and zero errors; the GPU starts
successfully. Preserve the stopped guard as a platform baseline failure.
Matched Linux starting maps for all sixteen unchanged slices are recorded
separately with the frozen starting binary; they do not rewrite repository
expectations or imply that the checked guard passed. Candidate qualification
must reject new losses against those Linux starting rows.

All sixteen Linux starting maps complete with zero errored rows across 2,731
files: 1,044 verified passes, one reference-unverified pass, 872 failures
and 814 skips. The evidence-only `--write-expectations` commands each exit
one because raw reftest failures exist; every file count agrees with its
complete map. That differs from the stopped checked-expectation guard and
does not establish a passing `unexpected=0` guard. Preserve the malformed
subset-parser helper attempt separately: it fails before rendering; the
corrected v2 maps bind the frozen runner and all input inventories.
Root verifies all 225 archived evidence files against their inventory. The
guard/map receipts' `before.lock_sha256 = f540a7c8...` records the then-active
dirty preview checkout, not the frozen baseline binary's build lock. The
binary's actual build tuple remains clean `6cb2284a33d`, lock `9809247c...`
and binary `087c4347...`. Preserve the original receipts with a provenance
amendment separating those two tuples; no baseline result is attributed to
compiled preview code.

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
  and create isolation only for an actual collision. Mark authorized Forms
  publication and push on 2026-10-08. Root owns commits and serial gates; agents own disjoint
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

The prescribed Livery/jobs4/timeout 120/drive15 maps remain immutable. The first
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

### Retained local checkpoint before publication

This records the earlier local proposal. The completed public-source repin and
its fresh ThinkPad evidence follow this historical checkpoint.

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

Mark authorized publication and push on 2026-10-08, superseding the brief's
"Do not push" instruction. The exact
two-file qualified Mere repin proposal is recorded in
`Code/testing/genet/forms/mere-forms-current-knot-repin-review.patch`; the exact
qualified candidate manifest and lock are named in its review-proposal JSON.
The normalized patch passes `git apply --check`. Verify the candidate files
against the published Genet source before accepting its repin. Mere's
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
The generated helper bytecode was removed on 2026-10-08 after Mark explicitly
authorized cleanup; its earlier automatic-review rejection remains historical.
Mark requested the remaining test work on ThinkPad to relieve local memory
pressure. Remote gates run serially with one Cargo job and one test thread.

### Public-source publication on ThinkPad, 2026-10-08

Genet main `260207fce6e8c361ac83c3bc498e3e90ff5fa69f` publishes product
`e84f9c7f9aec23320c539784961d1465f8a53a9b`. Mere's permanent repin is committed
at `632c1d298dee1ecd5c3dfe00879c95d4a012f8ea`, after integrating upstream
`4fd2f3f1`. Its modern 33-package locked Genet family moves to the published
product while Knot `eabd4434` retains its two explicit older Genet identities.
Baseline and candidate metadata preserve versions, package definitions, owner
edges and features after the authorized revision mapping. Resolved packages
increase from 1,566 to 1,568 only for the two protected identities. The selected
five Cambium roots retain their 605-package closure without either JavaScript
engine implementation. The permanent lock SHA256 is
`69eb637d64e7387179cda95b1a2e80768e076d95da29c24b9203d0c573551cd5`.

Fresh Fedora ThinkPad gates use Cargo 1.98.1, one build job and one test thread:

| Public-source Mere gate owner | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| Cambium | 257 | 0 | 1 |
| Rootstock | 72 | 0 | 1 |
| Native accessibility | 23 | 0 | 0 |
| Browser mirror | 10 | 0 | 0 |
| Six native-host targets | 63 | 0 | 0 |

The Wasm `a11y_page` example compiles with its existing getrandom setting. The
receipts confirm clean stable source and unchanged manifest/lock across each
run. Their prefixes are `thinkpad-forms-20261008-mere-packages-retry1`,
`thinkpad-forms-20261008-mere-native-host` and
`thinkpad-forms-20261008-mere-web-wasm` under
`Code/testing/genet/forms/thinkpad`. The first package compile is interrupted
after a separate Cargo owner appears, without executing tests; that attempt
remains unqualified. The foreign owner is preserved. The unchanged serial retry
supplies the complete result. These tests and compile checks do not constitute
new physical-window, browser-operation or human assistive-technology evidence.

The provenance audit verifies clean normal Cargo Git checkouts at the exact
published Genet and Knot revisions, without local source replacements or Git
URL rewrites. Root independently recounts all complete result blocks and binds
the receipt to the current integrated source. Published Mere `eafb8643` includes
upstream `9ed44a5e`; the extra source changes lie outside all 18 workspace
packages in the tested closure, and its manifest, lock and toolchain are
identical to tested `632c1d29`. Evidence is
`thinkpad-forms-graph-qualification.json`,
`thinkpad-forms-20261008-provenance.json` and
`thinkpad-forms-mere-publication-qualification.json` under
`Code/testing/genet/forms`. Prior publication inventory and proposal artifacts
retain their original source boundaries.

Turnstone product `e77e1e2db88344c651c1d4bf31e1802a2b142fc0` passes its complete
Linux library: 680 passes, zero failures, nine existing ignores and no filtered
cases. Its locked metadata resolves 1,283 packages, including Mere `3ded2cd7`
and Genet `965b64e2`; all 123 Git packages across ten revision groups have exact
clean normal public checkouts. Cargo config and Git URL rewrites are absent.
The manifest and lock remain unchanged, with lock SHA256
`de6ce134764e50cb82cbf7110121e2d8ac509484afc0d51060d52ba0bd9e6576`.
Published main `1802a68371327343539b3e73d4517248cf3361b1` adds only documentation
to that tested product, including intervening public `afe7964` documentation.
The raw `thinkpad-forms-20261008-turnstone-*` receipts are copied locally;
`thinkpad-forms-turnstone-integrated-publication-qualification.json` independently
recounts the complete result and binds the public source. This is F5 mechanical
compatibility on existing pins, not adoption of Mere's new Forms family.

An additional Windows Turnstone publication worktree isolates e77 from another
owner's later unpublished resource-content commits. Its documentation is merged
back into local main after the normal public push; the resource commits and
untracked `.github/` work remain preserved. The clean publication worktree and
its integrated temporary branch are then removed after public reachability and
live-owner checks.

Cleromancy integrates the upstream domain module at `e512763` before its fresh
Linux gate. That first run passes 19 library tests and fails the marked-textbox
fixture before either DOM target executes. The fixture clicked fixed coordinates
beneath its label. The bounded repair at `9789d64e5df4937c3005a2b3a25b5a733f67817b`
uses the semantic Question textbox and its live painted layout; production
routing and naming stay unchanged. The focused positive passes and disabling
only marked-textbox recognition makes it fail. Byte-exact restoration with a
fresh source mtime precedes the complete qualifying retry: 20 library tests,
one authoring DOM test and two headed consultation DOM tests, all passing with
zero failures, ignores or filtered cases. An intervening interrupted attempt
retains its partial result as unqualified; the foreign Cargo owner is preserved.

Locked metadata at the repaired source verifies 60 Git packages in exact clean
normal public checkouts: 36 Mere `8106c7c`, 20 Genet `34626a6c` and four Netrender
`c8c09f1`. Cargo configuration and Git URL rewrites are absent. The ignored root
lock is unchanged at SHA256
`6040b3e0030a9114c42c2a83e6e67a3ca1d12753308ca87467264d19efa0f3b1`,
and its tracked core lock is preserved. Published Cleromancy main
`ff3b1f5302e15b054b412bba8261dfc28ea81740` adds only the qualification document
to the tested source. Raw fixed, control, restoration and interrupted-attempt
receipts live under `Code/testing/genet/forms/thinkpad`.
`thinkpad-forms-cleromancy-fixed-publication-qualification.json` independently
binds all three completed targets and metadata provenance to the publication.
These are automated checks on existing dependency pins; the earlier physical
desktop receipt retains its own source snapshot.

Remote worktrees `/home/markik/Code/worktrees/mere-forms` and
`/home/markik/Code/worktrees/turnstone-forms` isolate real collisions with the
primary Knot lane and untracked Turnstone receipts. Serial gates reuse stable
repository targets. Both Forms worktrees are removed with `git worktree remove`
without force after public reachability, evidence transfer, clean status
including ignored output, and live-owner checks. The audit is
`thinkpad/thinkpad-forms-20261008-worktree-cleanup-audit.txt`; primary work and
other owners' worktrees remain preserved. Stable targets remain reusable caches.
The Windows cleanup preflight retains `C:/t/cargo-homes/genet-streams`, borrowed
by Forms from Streams, because two cached Vano `test262` submodule entries are
modified. Its audit is `forms-local-cache-cleanup-audit.json`; dirty source is
preserved and the stable repository targets remain reusable caches.

The final public-source binding is
`Code/testing/genet/forms/forms-phase-a-published-thinkpad-source-binding.json`.
Mere main advances again after this lane's qualified `eafb8643` publication,
including further Cambium and command-catalogue changes. This lane's receipt
retains the eafb source boundary; it does not qualify those later changes.

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

- 2026-10-08: Mark explicitly answers F8 and F9, approving the already-locked
  native matcher and native notice with an accessible alert. Three bounded
  Luna agents implement matching, the message/report bridge, and host painting
  in separate ownership seams; root implements the session alert projection.
  Native reports retain physical realm/node ownership and agent enqueue order.
  The native alert is outside the authored DOM and uses the actual notice's
  top-session viewport bounds, independent of target scroll or child-frame
  geometry. Repeated reports receive fresh semantic identity; Escape dismisses
  the current notice. Source formatting is limited evidence: these new edits
  have not been compiled or executed, and are not Checkpoint B qualification.
  Runtime catalog replacement exists; the public document construction/options
  seam still needs review so embedders can set wording before authored scripts.
- 2026-10-08: Mark requests a handoff because this orchestration chat is long,
  and chooses a fresh Forms chat on the ThinkPad. Stop local agents after their
  stable source checkpoint, freeze the owned source and approved F7/F8 lock,
  and transfer those inputs plus the handoff record to ThinkPad evidence.
  Preserve the complete fresh CCA starting maps, font/resources and raw
  receipts. No new candidate build, source application, WPT run, commit or push
  is inferred from preparing this handoff. The next chat owns candidate
  qualification and publication through Checkpoint B, then stops before C.

- 2026-10-08, ThinkPad Forms takeover: all 31 transferred source hashes, lock and 17 input evidence hashes verified. Inherited report-red disappeared during temporary-server handoff without test output or receipt; immutable partial evidence remains and unchanged rerun reproduced both-engine form.reportValidity failures. Fixed queueing the first unhandled entry. Narrow-overlay regression reproduced out-of-viewport bounds; scaled margin/padding and background clipping pass the restored regression. Custom whitespace already passed before explicit Preserve shaping, correcting the earlier unconfirmed review finding. Added both-engine adoption/realm-teardown feedback fixtures. Native editor draft dom_mut calls did not compile; existing mutate_dom capture supplies the correct retained mutation path. Child alert fixtures now pump queued iframe parsing before access. Pattern bypass, missing catalog installation, and suppressed native sink have executed intended failures; exact production bytes restored before full qualification. No Checkpoint B acceptance or Phase C work is inferred. Raw receipts and continuation ledger live under Code/testing/genet/forms/thinkpad/phase-b-thinkpad-*.

- 2026-10-08, restored library qualification: the nine-target run binds unchanged source and records 60/2 document tests, 323/6/1 Livery, 42 render passes, 131 scripted passes, 95 DOM passes, 26 Boa adapter passes, nine Vano adapter passes, and 185/2 runtime tests; layout-dom has no tests. The six Livery failures exactly match the refreshed baseline. All new host catalog/adoption/teardown fixtures pass. Four additional failures block acceptance: the runtime ordering fixture illegally redeclared window.top (renamed its control variable); retained subtree merging omitted virtual native-control sources (collect those existing arena carriers in DOM order); and captured scripted pointer release outside its target returned Miss (consume the captured release without navigation, as the existing test contract requires). These corrections await restored verification. The collector adds the previously clean components/genet-livery/src/document.rs to the owned source set, now 32 files. No provider or lock change is involved.


### 2026-10-08 final review repair gate

The restored nine-library run is source-stable: runtime187/0, documents61/1,
Livery 323/6/1, render42/0, scripted131/0, arena95/0, layout0/0, Boa 26/0 and
Vano 9/0. Navigation capture and cross-realm report ordering are now positive.
The six Livery failures remain the named baseline. The remaining document
failure has a real missing highlight: initial and selected scene rectangle
lists are empty, and the post-relayout control range returns None.

Fresh whole-change review adds two P2 findings. Four new arena cases execute
RED (95 old passes, four intended failures): required whitespace-only option
placeholder, normalized option value/setter, space-separated datetime bounds,
and space-separated datetime step bases. The repair shares HTML-aware option
text/value between native getters, setters, validity and public script
projections; explicit value attributes stay exact and non-ASCII whitespace
stays intact. Datetime comparison and step conversion accept the same two
separators as existing value sanitization. Both-engine public integration
cases are added; their execution remains pending.

The textarea failure comes from retained-root formatting omitting the atomic
and native control text preparation that complete layout performs. The repair
prepares that formatting subtree before its text frame is merged, alongside
the earlier virtual-source order preservation. It visits the changed subtree,
rather than traversing the full document once for every retained root. The
previously clean layout/query.rs and layout/retained.rs are added to owned
source guards (34 paths total); exact HEAD preimages are preserved in the
ThinkPad receipt directory. This is a bounded retained text repair required
by the native editor fixture, without adding interaction widgets or submission.
The RED/diagnostic gate is immutable and source-stable. These repairs require
the next full gate and matched WPT/CSS maps before acceptance.


### 2026-10-09 crate qualification and runner freeze

The final source-stable library/integration gate executes documents62/0,
Livery 323/6/1 (exact six named baseline failures), render42/0, scripted131/0,
arena99/0, layout0/0, Boa 26/0, Vano 9/0, runtime187/0 and the cross-realm Forms
integration 8/0. The original textarea highlight assertion now passes; all
four reviewer regressions execute RED then GREEN. Public option/datetime
normalization passes on both engines. The netfetch genet-wpt binary gate
executes75/0/3 ignored. No additional crate failure is accepted. Raw receipts
and complete failure lists remain under Code/testing/genet/forms/thinkpad.

The next frozen candidate manifest binds34 owned paths and the approved lock.
Its executable code is identical to these passing gates; this dated progress
entry is a documentation-only addition. The optimized runner, unchanged20,
eight exact Forms maps, sixteen CSS maps, loss rejection and movement
attribution remain required before qualified publication. Checkpoint B is
not yet declared complete; Phase C remains stopped.

## Checkpoint B qualified main publication, 2026-10-09

ThinkPad Forms owns the transferred work. The sending chat stopped source edits
and gate launches. The inherited report-red disappeared without a receipt;
its interruption remains recorded and the unchanged rerun reproduced the two
intended form-report failures before repair. All subsequent gates bind actual
source, lock, argv, toolchain and raw output. Phase C submission remains stopped.

### F10: native nested-negation correction

Mark answered the concrete dependency choice, verbatim: "Apply and qualify the
vendored fix (recommended)". The native DOM edge now uses the vendored
`support/patches/regress-0.11.1` path with unchanged version, features and
dependency requirements. The workspace excludes that directory from membership.
Boa and Vano retain their registry dependencies and original source pins. All
878 prior lock identities/checksums remain intact; one local identity is added.
All 46 upstream files match the checksum-verified registry archive. The only
upstream code delta retains the complement returned by `CodePointSet::inverted()`.

The new native test is RED before that two-line fix and GREEN afterward: nested
negation on either side of an intersection, subtraction and a UTF16 emoji range.
The original unchanged20 probe is 0/20 before and 19/20 after on both engines.
Its B09 pattern `[a-z&&[^aeiou]]+` is invalid v syntax: a range needs a nested
class when used as an intersection operand. The original fixture and results
remain unchanged. A separately named copy changes only that pattern to
`[[a-z]&&[^aeiou]]+`, preserving all 20 names and assertions. It is 0/20 before,
19/20 with the unpatched matcher and 20/20 with the fixed candidate, on both
engines. Node 24.19.0 independently confirms the anchored grammar and values.
See [ECMAScript ClassIntersection grammar](https://tc39.es/ecma262/multipage/text-processing.html#prod-ClassIntersection).

### Qualified source and behavior

The arena supplies live validity flags, barring, custom messages and interaction
state to script and selectors. Form validation visits associated controls in
tree order, dispatches trusted invalid events and preserves false results when
listeners cancel. Native owner-routed reporting supplies a clipped Livery
notice and an assertive Alert projection in the top session, including child
realms. The message catalog is installed before authored scripts and is
replaceable by the embedder; custom-message whitespace and full accessible
text are preserved. Repeated reports replace the active notice. Adoption and
realm teardown drop stale routes. The native paint path is
`components/genet-livery/src/paint.rs` (`validation_notice_layout` and notice
painting), with shaping in `text.rs` and session accessibility in
`genet-documents/src/engines/scripted.rs`.

Fresh whole-change review found two further defects. HTML-aware option text
normalization now serves option/select value and required-placeholder checks;
datetime-local bound/step parsing accepts a space or T separator. Four native
RED cases and both-engine public fixtures pass after repair. Retained editing
now prepares atomic control text only inside the changed subtree before merge;
the original textarea selection rectangle assertion passes. Captured pointer
release outside is consumed without navigation. Tiny notice geometry is tested
at viewport widths 1, 4, 24, 40 and 320. Whitespace already passed before the
explicit Preserve style; no whitespace RED claim is made.

Pattern bypass and catalog omission each produced the two intended engine
failures. Suppressed native reporting produced the intended host/catalog,
adoption, top-alert and corrected child-alert failures. The first sink attempt
was a compile failure and the first child cases were setup failures; neither
is counted as semantic evidence. Production bytes were restored before the
positive gates.

### Matched qualification

Release `+1.98.1 --locked -j1`, `--test-threads=1`: Documents 62, Livery 323,
Render 42, Scripted 131, native DOM 100, WPT bin 75, layout seam 0, Boa 26, Vano 9,
Runtime 187 and public Forms integration 8 pass. Total 963 passes, exactly six
allowed Livery failures and four existing ignores. The six retained names are
recorded in `phase-b-thinkpad-vendored-crates-forms-wpt.summary.json`; no extra
failure exists. The optimized netfetch runner is an actual successful build
with unchanged source and lock, frozen SHA256 `54acfcc6cd2bb7cba840400dc2adbd9ef9b0e772efbba72986749e38c22b68b2`.

The immutable v4 wrapper completes eight Forms maps (219 source files per
engine) and sixteen static Livery CSS slices (2,731 files), with one worker,
timeout 120 and drive-deadline 15. All source/resource/build guards pass.

| Matched slice, each engine | Before | Candidate |
|---|---:|---:|
| constraints | 0/877 | 825/911 |
| the-input-element | 928/1853 | 957/1853 |
| textfieldselection | 667/739 | 667/739 |
| form-control-infrastructure | 31/120 | 101/120 |

The table includes partial error rows, preserving their exact named identities
and duplicate occurrence numbers. Completed-only totals are 1,618/3,542 before
and 2,541/3,576 after; partial rows separately contribute 8/47 and 9/47 passing
rows. Observed totals are 1,626/3,589 to 2,550/3,623 on each engine. That is 924
additional passes per engine. Every old file and subtest pass survives,
including passes in ERROR files. All 88 file movements and 1,850 subtest movements
have source/case/occurrence attribution, with raw before text where available.
The aggregate logger omits individual failure text for the partial radio row;
its exact fail-to-pass identity is retained and its validity cause is explicitly
inferred from the unchanged source and pre-B absent projection.

CSS has zero movements or pass losses: 1,045 passes, of which 1,044 are verified
and one is reference-unverified; 872 failures and 814 skips remain. The existing
Windows checked-expectation guard is distinct from this fresh Linux matched
pass-preservation qualification. Expectations and corpus files were not changed.

The corpus inventory/archive, 5,047 resource inventory entries, four font bytes
and all provider pins match the before build. Fresh font inventory SHA256 is
`e13569870b0ddc0c2673b67cceb61ca3c809e7e708d0c27d446282d0fa780ea2`.
The final freeze owns 82 paths; publication updates only the three canonical
documents after the map freeze, with executable source hashes held identical.
Existing provider/test262 `.cargo-ok` markers are preserved. Preflight status
aggregation and workspace-exclusion hash corrections have separate preserved
config versions and failure receipts; the v4 wrapper and map policies are unchanged.

### Evidence and remaining bounds

Evidence root: `/home/markik/Code/testing/genet/forms/thinkpad`. The final source
manifest, library summary, runner receipt, map stage receipts, strict accounting,
movement attribution, native vendor provenance and prepublication verification
use `phase-b-thinkpad-vendored-*` / `phase-b-thinkpad-checkpoint-b-*` names.
Original archives, raw controls, before binaries and preliminary captures remain.

The matcher has no public execution budget. On both engines, the unchanged
`infinite_backtracking.tentative.html` moves from an unsupported-API FAIL to
ERROR/hang-killed; this is a liveness limitation, not a gain. Huge-year step
arithmetic retains its f64 precision limit. Alert/notice geometry and routing
are qualified projections, not headed assistive-technology validation. Existing
picker/widget interaction gaps and the remaining WPT failures stay visible.
These are scoped results; a browser-wide WPT percentage was not refreshed.
Phase C submission requires the next human checkpoint.
