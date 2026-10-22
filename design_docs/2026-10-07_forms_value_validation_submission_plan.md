# HTML forms: value state, validation and submission

**Current status, 2026-10-16:** Phases A and B are qualified and published.
Phase C's implementation is published, including native activation; final
acceptance remains open. F12's published redirect-base repair passes the
original 285 affected tests on Windows as well as the earlier ThinkPad checks.
Mark directed this continuation onto O-PC, so F11's Windows qualification
continues there.

The closing review found three submission defects: missing submission
user-validity transitions, redundant submitter checks after listener mutations,
and silent script-free validation failures. Their repair passes 678 Windows
tests under Rust 1.99.1, with zero failures or ignores, including the new RED
regressions, existing validation/FormData checks, and all original 285 affected
native/scripted occurrences. The private native hook is deleted on both engines.
[Windows repair receipt](receipts/2026-10-16_forms_submission_review_windows.json).

Fresh O-PC native Livery, Boa and Nova transport, final accessibility and visual
checks pass, including the redirected relative image. The script-free invalid
notice also paints and exposes an assertive alert. The actual Chrome browser
submission run passes all 20 cases and the separate editing-only control under
the unchanged evidence checker, with zero uncaught JavaScript exceptions.

That browser run found and qualified three host repairs: release the canvas
borrow before synchronous focus callbacks; repaint the retained source after
submission so its hit-test layout remains usable; and reject superseded source
responses before fetching their dependencies. The browser checks bind the exact
Wasm artifact, trusted input, request bytes, response timing and painted proof
markers. They do not qualify general browser text rendering. See the
[Windows host receipt](receipts/2026-10-16_forms_native_browser_windows.json).

Windows covering gates, matched WPT/guard maps and final Checkpoint C reporting
remain open. Earlier ThinkPad receipts retain their recorded Linux scope.

The dated status paragraphs below record earlier qualification stages.

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

**Checkpoint note, 2026-10-09:** Phase B is qualified and published on main
at `a0d95ebd6e20220e3430870365112a608b4f83ff`. Mark approved proceeding with
Phase C after its publication report. Submission work is in progress under
C5, with builds and matched gates on ThinkPad. The Phase C execution section
below owns current tasks and done-conditions; earlier Phase B progress entries
remain historical evidence.

**Checkpoint C note, 2026-10-20:** the Windows qualification run is complete on O-PC (candidate
and control captures matched with zero lost passes, twelve covering gates green, three reused);
see "Checkpoint C qualification run on O-PC" at the end of Progress. **Accepted by Mark on
2026-10-20:** "Accept, with the formdata-event timeout as a follow-up (Recommended)". Phase C is
qualified at the prepared epoch; the follow-ups are named in that section and in
`receipts/2026-10-20_forms_checkpoint_c_windows.json`.

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

### F11: qualify Checkpoint C on the Windows box, 2026-10-15

Question as put: "Where to qualify. (a) This Windows box: a fresh matched
before/after on current main, with 'before' = main with the 56 Phase C files
reversed (a three-file merge). The Linux Checkpoint B numbers are cited for
reference only. (b) Resume the ThinkPad harness: the same platform as B, but
its tooling is frozen at an older main and another session is using that
machine." Options: (a) recommended, (b).

Mark's answer, verbatim: "This Windows box (Recommended)".

Consequence: candidate is current main; the control is current main with the
Phase C product changes of the six Phase C commits reversed. Both are built
and captured on the Windows box with the same toolchain, providers and
policies. Linux Checkpoint B and pre-integration numbers are reference only.
The six Linux Livery baseline failures are not an allowance here: any Livery
failure on Windows is a regression to investigate.

Continuation, 2026-10-16: Mark directed "then ssh into o-pc". Work resumed
on Windows O-PC under this ruling.

### F12: fix the scripted post-redirect base URL inside Phase C, 2026-10-15

Question as put: "The Boa redirect-address bug: (a) reproduce it here, add a
failing test, fix it inside Phase C (the plan already requires that the final
address after a redirect becomes the document's address), or (b) record it as
a limitation." The finding: the ThinkPad native live run of 2026-10-14 loaded
the post-307 page on Boa, but requested its relative `proof.png` as
`/case/boa/proof.png` (404) rather than `/case/boa/final/proof.png`; Livery
was correct and Nova was not run.

Mark's answer, verbatim: "Reproduce, test, fix (Recommended)".

Consequence: reproduce on Windows, add a failing test before the fix, fix it
as Phase C work and run the live case on Livery, Boa and Nova.

### F13: newest wasm-bindgen, live browser check in Checkpoint C, 2026-10-15

Question as put: "The live browser (Wasm) submission check: (a) leave it out
of Checkpoint C as a named limitation (blocked on 2026-10-14 for want of a
WebGPU browser; the ThinkPad package pinned wasm-bindgen 0.2.127 and this box
has 0.2.129), or (b) run it in Chrome on this machine."

Mark's answer, verbatim: "pin the newest wasm-bindgen".

*Reading, not ruled:* pin the newest released wasm-bindgen with a matching
wasm-bindgen-cli, then run the live browser submission check in Chrome on this
box as part of Checkpoint C; stop and report if WebGPU is blocked for a reason
unrelated to the version. Checked 2026-10-15: crates.io's newest wasm-bindgen
and wasm-bindgen-cli are 0.2.129 (published 2026-09-25), which main already
pins exactly (`genet-render-host`, `genet-scripted-worker`, `ortet`,
`support/build-scripted-workers.ps1`) and this box has installed. No pin
change is needed.

### F14: Rust 1.99.1, and the ThinkPad lane's rulings R19-R26, 2026-10-15

Question as put: "The R19-R26 rulings in the ThinkPad lane's notes (e.g.
'qualify with Rust 1.99', 'a toolchain change needs a new matched
before/after'): copy them into the plan as dated history labelled 'agent
rulings, not Mark's', or confirm them as his."

Mark's answer, verbatim: "1.99.1 just dropped, which we should adopt. tells ya
how i felt about 1.99 too, huh".

*Reading, not ruled:* qualify under Rust 1.99.1 (released 2026-10-13,
installed minimal with the wasm32 target on 2026-10-15) via explicit
`cargo +1.99.1`. R19-R26 are recorded below as the ThinkPad lane's rulings,
with R19's toolchain amended by this ruling. The repository's
`rust-toolchain.toml` (channel 1.99.0) is not changed here; a family-wide
toolchain bump is decided separately.

### F15: Checkpoint C runs on O-PC, 2026-10-18

Question as put (after the 2026-10-18 state report): Checkpoint C is unstarted and F11's
Windows choice was recorded as pending a platform clarification; the ThinkPad's live
browser leg was blocked for want of a WebGPU-capable browser. Where does final
qualification run?

Mark's answer, verbatim: "Let's do it on o-pc. If the thinkpad doesn't have a webgpu-capable
browser, might as well".

Consequence: F11 stands and is no longer pending. All of Checkpoint C runs on O-PC (this
Windows box, Chrome 155.0.8059.80 installed): the fresh control, the 15 covering gates, the
matched capture, the native live submission and the F13 live Chrome check. The ThinkPad's
2026-10-13/14 and 2026-10-16 receipts stay as evidence of the candidate's behaviour on Linux,
not as Windows qualification. The control must be reconstructed to account for the F12
repair on the same 56 paths, as the 2026-10-16 entry says.

### F16: Checkpoint C qualifies the prepared epoch, 2026-10-18

Question as put: which source epoch does Checkpoint C qualify? The Windows lane prepared
everything at `e3911fb6b2d` plus `8fa73497` (both in main): candidate WPT runner built and
frozen, candidate controls assessed, native live run and Chrome 155 browser submission
passed, the 59-path control projection staged in `genet-forms-c-control`, 18 capture chunks
and 15 gate rows frozen. Current main is 55 commits and 311 files past that.

Mark's answer, verbatim: **"Prepared epoch (Recommended)"**.

Consequence: candidate is `e3911fb6b2d`+`8fa73497`; only the control is built new. The 55
later commits are a named limitation of the Checkpoint C report, with a follow-up delta gate
on them, not part of this qualification.

### F17: the guard's RAM floor is 6 GiB for this run, 2026-10-18

Question as put: the lane's guarded wrapper refuses to start unless 12 GiB of RAM is
available (4 GiB compiler cap plus the 8 GiB floor) and 30 GiB of disk; RAM was 9.9 GB free
of 31 with no single hog, and the control build was refused on that check on the 16th.
Options: free RAM and launch at 12 GiB; lower the floor to 6 GiB; hold until quiet.

Mark's answer, verbatim: **"Lower the floor to 6 GiB"**.

Consequence: a new wrapper revision with `FLOOR = 6 GiB` (cap unchanged at 4 GiB, so the
guard needs 10 GiB available at launch), content-hashed like its predecessor and named in
`build-wrapper-name.txt`; every receipt it writes records the new wrapper hash and floor. The
6 GiB floor is this run's ruling, not a family-wide policy. The STOP sentinel from the
2026-10-16 pause is removed at launch and remains the kill switch: recreating it stops the
sequence before its next job.

### F18: unattended, sequential, one report at the end, 2026-10-18

Question as put: remaining work in order is the cold control runner build, 76 capture rows
across both engines and both roles (2,468 submission and encoding records plus prior maps,
CSS slices and controls), 12 of the 15 covering gates (3 reused), then whole-change review
and the Checkpoint C report. Options: unattended sequential; stage by stage with check-ins;
gates first.

Mark's answer, verbatim: **"Unattended, sequential, report at the end (Recommended)"**.

Consequence: the stages chain under one owned job at a time, stopping on the first incomplete
receipt; the report carries every count, every refusal and the limitation list.

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

## Phase C execution, 2026-10-09

**Status:** in progress. Mark approved proceeding after the published Checkpoint B report with “Proceed, orchestrate at will”. Checkpoint B is published at `a0d95ebd6e20220e3430870365112a608b4f83ff`; remote main and a clean checkout match. Its 963 passing tests, six explicitly named baseline Livery failures and four existing ignores remain the starting evidence. Root owns qualification and publication on main under the carried approvals. No subsequent feature lane is authorized by this continuation.

**Goal:** complete C5 submission using native live control state, exact charset-dependent bytes and existing browsing-context navigation. The brief remains the spec, together with current HTML, URL, Encoding and XMLHttpRequest standards. The two external reconnaissance reports at `Code/testing/genet/forms/thinkpad/phase-c-{native,navigation}-recon.md` record verified seams; they do not substitute for execution evidence.

### Global Constraints

- Source authorities are WHATWG HTML, URL, Encoding, XMLHttpRequest, their normative Unicode data and unchanged WPT. Original implementation code is written from those algorithms; no unapproved source copying.
- Use the already-locked `encoding_rs` and `url`; only the original spec's approved `encoding_rs` workspace edge may be added to native scripted-dom so both hosts share the encoder. No new packages, versions, provider identities or other dependency edges. Preserve the original ignored Cargo.lock as a baseline and audit/freeze the sole native dependency-list change.
- Root owns every build, test launch, commit and publication. Workers do not launch Cargo or edit sources during an active source-bound gate. Gates use installed `+1.98.1`, `--release --locked -j1`, release debug0, incremental0, and test-threads1, after the Cargo/rustc owner guard.
- Continue in the current clean main checkout, as already approved. Create isolation only for an actual collision. No provider or corpus/expectation edits.
- Native DOM state owns current control values and selected-file bytes. Scripts project state and dispatch events; there is no page-script-owned submission value model. No arena/host borrow across authored JS callbacks.
- Existing LayoutDom implementors must not acquire mandatory new methods. File state can remain native arena-only; optional shared projection accessors require defaults.
- Matched Phase C captures cover all33 form-submission-0 records and all46 encode-form sources /584 variants per engine. The brief's584 total includes five base files absent from the literal trailing-hyphen glob. Preserve ERROR partial rows and exact duplicate occurrences. Require zero old pass losses in Phase A/B and CSS maps.
- New auxiliary windows, a native file picker and custom-element form association are existing capability gaps. Preserve them explicitly; implement existing-context targets, programmatic selected files and ordinary native controls completely. No fabricated new-window navigation.

### Review Focus

1. Listener reentrancy/mutations must clear per-form construction/submission flags and use the final mutable formdata list without holding native borrows.
2. Files retain exact binary bytes and JS identity through selected lists, FormData, adoption and events; realm teardown must not orphan adopted selections.
3. Charset conversion, newline normalization, numeric-reference replacement and multipart header escaping follow independently derived byte fixtures.
4. POST/redirect requests retain method, body, headers and initiator policies through host adapters; default URL-only hosts decline unsupported requests.
5. Target selection, detached contexts, canceled validation/submit/click and consecutive planned submissions preserve the existing WindowProxy/lifecycle contract.

### Task 1: request-aware document navigation

**Files:** `components/genet-host-api/lib.rs`, `components/genet-scripted/resource_bridge.rs`, `components/script-runtime-api/frames.rs`, `ports/ortet/src/fetch.rs`, `components/script-runtime-api/tests/navigation_requests.rs`; existing navigation and bridge tests as needed. No changes to submission/FormData surfaces in this task.

**Produces:** a host-owned ResourceRequest record carrying URL, method, headers, body and existing fetch policy fields; optional `ResourceFetcher::fetch_request(&ResourceRequest) -> Option<ResourceFetchResponse>` with a distinct richer response record whose default preserves plain GET compatibility and declines unsupported requests; request-aware `frames::navigate_request<E>(cx, from, target, request: FetchRequest, replace)` alongside the ordinary URL wrapper. The PendingNavigation owns its request until the existing navigation task executes. Product adapters forward rather than reinterpret method/body. The response final URL becomes the document/history identity. Extend named target resolution through existing contexts without inventing auxiliary windows. Keep script/frame loading behavior and same-document GET navigation intact.

- [x] Write real both-engine failing navigation tests using recording FetchHandler: POST bytes `[0,128,255]` and Content-Type survive, redirected final URL binds document and history, named child target preserves its WindowProxy, no loader-only downgrade. Add a product bridge recording ResourceFetcher test for rich forwarding and a URL-only default refusal test. Assert independent literal values, not helper output.
- [x] Hand root the exact red Cargo commands and pause product writes until their receipts. Root runs runtime integration test and bridge fixture filters under the serial gate contract.
- [x] Implement request queue, network fallback and host forwarding. GET fragment branches must not consume POST; release borrows before host fetch. Do not replace the existing navigation lifecycle.
- [x] Root qualified new fixtures and navigation/top-level/frame lifecycle plus host library:74passes; Cargo targetmetadata confirms private fixture is not auto-discovered; Ortet check passed. Scoped review resolved product loader redirect loss. Commit `0d86542939af8f946136c5b289c8338679cedade` owns Task1 only. Task3 completed the full runtime suite:195 library and674 integration passes across57 groups, zero failures/ignores. The earlier interrupted receipt remains preserved and unqualified.

### Task 2: native entries and charset encoders

**Files:** new `components/genet-scripted-dom/forms_submission.rs` and native directionality/classifier files as needed, arena `lib.rs`, `forms.rs`, `forms_validation.rs`; new native `form_encoding.rs`, Unicode source/license/generator/provenance and runtime `form_encoding.rs` re-export. Task1 interfaces are consumed without rewriting navigation.

**Produces:** public native FormFile /FormEntry /FormEntryValue records with stable file identity and immutable bytes; node-owned selected-file accessors; `form_submission_entries(form, submitter, encoding_name, image_position)` preserving native association/tree order and successful-control filtering; shared native pure encoding selection/urlencoded/text-plain/multipart functions returning exact bytes/content type, re-exported by runtime for scripted consumers. Keep FormControlState's existing capture layout intact. Expose shared disabled/datalist helpers within the arena rather than duplicate algorithms.

- [x] Add native failing tests for external association/order, disabled first-legend exception, datalist exclusion, current/default separation, checked controls, submitter/image entries, selected disabled options, empty/multiple files and reset/clone/adoption. Add pure byte fixtures for spaces/plus/ampersand/tilde, CR/LF, duplicates, windows-1252/Shift_JIS, unrepresentable scalars, UTF16 output fallback, multipart header CR/LF/quote and binary `[0,128,255]`.
- [x] Root executes applicable native and encoder red filters before production code. New APIs may be initially expressed through test-local adapters to reproduce semantic missing behavior; compile errors alone are not red evidence.
- [x] Implement native traversal and pure encoders from standards. `_charset_` takes the supplied encoding; FormData constructors will supply UTF-8. File content is never converted through text. Directionality uses actual native ancestry/auto rules for dirname entries. Generate classification data from official Unicode17 DerivedBidiClass with its license, source hash, generator and exhaustive scalar validation; no new dependency edge or copied implementation. Carried network approval permits fetching normative data.
- [x] Root executes green filters and full native DOM/runtime libraries. Report exports/signatures and exact fixture evidence for Task3. Only the approved native encoding_rs dependency-list change is permitted; root audits unchanged package/version/checksum/provider sets and freezes both locks.

### Task 3: document encoding through real loads

**Files:** new shared native `components/genet-scripted-dom/html_encoding.rs` and native document metadata in `lib.rs`; runtime `lib.rs`, `frames.rs`, DOM document bindings and focused integration tests; scripted document/livery source loading; unscripted Livery source preparation and document-resource frame loading as needed; WPT runner initial document source paths in `ports/genet-wpt/src/{net,testharness,harness,main}.rs`. These are byte-to-document seams, not form bindings. Use the native approved encoding_rs owner. Read `Code/testing/genet/forms/thinkpad/phase-c-document-encoding-recon.md` for verified paths and current standards.

**Produces:** one shared HTML byte decoder preserving source, selected canonical document encoding and confidence; native encoding per Document node, with UTF-8 defaults for already-Unicode documents. Carry chosen encoding into initial/top/frame document parsing before authored scripts execute, and preserve it through navigation. Form adoption observes destination document encoding. Existing public String-source entrypoints and legacy host records retain compatibility through additive optional helpers/setters, without new dependency edges.

- [x] Add meaningful failing byte/document fixtures: BOM versus conflicting header/meta, supported header versus meta, invalid header fallback, comments and false meta in tag attributes, duplicate attribute rules and http-equiv pragma, meta UTF-16/x-user-defined adjustments, configured fallback, real EUC-KR/Shift_JIS bytes and canonical characterSet. Add both-engine initial-load/navigation/secondary document/adoption assertions. The actual WPT initial netfetch loader must be exercised, not just a pure decoder. No compile-error-only RED.
- [x] Root runs RED while workers pause. Implement normative BOM/header/prescan precedence and at most first1024-byte prescan using original code, not regex shortcuts. Preserve Encoding::decode's actual selected encoding. User/default fallback is explicit input/configuration. Identify confidence and the late declaration/parser-restart limitation precisely; do not claim full Encoding lane conformance.
- [x] Runtime/document constructors initialize metadata before parser scripts. DOMParser and createHTMLDocument remain UTF-8 Unicode inputs; srcdoc/blank special rules are verified against primary standards. No mandatory LayoutDom methods or added mandatory fields on public ResourceResponse/SessionSpawnRequest. Do not parse live DOM meta as a substitute for original byte decoding.
- [x] Root runs GREEN, complete covering libraries/integration and WPT loader checks. Report exact exports/source gates for the subsequent FormData/submission worker. Preserve initial-load/final-URL identity and snapshot-template source metadata.

### Task 4: FormData and submission bindings

**Files:** new runtime `form_data.rs` / `form_submission.rs` as appropriate; `fetch.rs`, runtime `lib.rs`, `dom/mod.rs`, `dom/form_controls.rs`, `dom/bootstrap.js`; `tests/forms_submission.rs` and byte-safe FormData Fetch integration fixtures. Consume Task2 records/encoders and Task1 navigation. Extend the shared native encoder with an efficient deterministic collision-free boundary selection helper consumed by scripted and unscripted multipart callers. Select a valid RFC boundary absent from the actual encoded part headers/values/binary bytes before adding delimiters; no clock/counter-only collision assumption or new entropy dependency. Preserve caller-supplied primitive for literal byte fixtures. Include actual delimiter collisions and chunk-spanning bytes; do not search intended delimiters in the final body.

**Produces:** native entry-list bridge and private branded FormData/File storage; `FormData(form, submitter)`, selected FileList/DataTransfer bridge necessary for programmatic file assignment; SubmitEvent/FormDataEvent; form.submit/requestSubmit; private activation and implicit-submission hooks callable by native hosts. Methods perform receiver/submitter/owner validation before work. Share native records, preserve cross-realm File identity and mutable formdata event lists, and use encoding_rs for submission bytes. Fetch(fd) uses UTF-8 and the same binary-safe multipart primitive.

- [x] Write both-engine semantic red fixtures for constructor entries, duplicate set order/live iterators, Blob-to-File identity, shared formdata edits, recursive construction cleanup, cross-realm brands/adoption, submit() bypass versus requestSubmit validation/event behavior, canceled events, submitter overrides, `_charset_`, GET query replacement/fragment and POST binary body.
- [x] Root executes red on unchanged production and freezes each control. Implement only after meaningful failures, preserving constructor exposure and event initialization semantics. No page globals or mutable `_l`/`_b` properties are authority.
- [x] Complete shared native construction/encoder/navigation bridges, captured per-form flags and trusted UA event construction. Recheck connectedness/ownership after listener mutations as specified; multiple planned submissions replace the form's earlier queued task. Keep every callback outside native borrows. Share the native guarded implicit-submission decision and disabled projection between runtime and script-free hosts, so default-button tree order and normalized blocker types have one authority.
- [x] Root executes green and full runtime libraries/integration suite, including existing Fetch binary/realm/navigation tests. Report test counts and movement causes; retain every genuine limitation.

### Task 5: native activation and final qualification

**Host boundaries:** the native request-aware factory preserves full request fields through its chosen fetcher. The browser host uses an explicit limited source-fetch policy for exact bytes and representable request fields, because browser RequestInit rejects navigate mode and owns origin/referrer/forbidden headers. Known unsupported intent declines before transport; real browser receipts are separate from wasm compilation. Scripted focus/key routing for submission is in scope; existing script-free textarea editing is preserved, while absent scripted text editing and physical child hit geometry remain explicit. Existing named-child form navigation still requires qualification.

**Files:** `components/genet-scripted/livery.rs`, `components/genet-documents/src/engines/scripted.rs`, `components/genet-documents/src/engines/livery.rs`, `components/shared/document-session-api/src/session_engine.rs`; public host/session fixtures and Genet-owned Ortet shell/web submission routes and necessary existing web-sys features. Preserve legacy session submission/effect/input shapes. Add a distinct encoded submission record and optional default DocumentSession consuming accessor, paired with the latest input and cleared on the next input. Add an optional request-aware SessionEngine spawn seam if needed to retain the selected factory and its fetcher. Ortet consumes authoritative query/body bytes through actual transport, without a second GET after POST. Unsupported targets/transports decline explicitly. Reuse native entries/encoders; no second algorithm. Mere is outside scope and its legacy route limitations remain explicit. Canonical plan/index/ledger updated by root.

- [x] Add failing physical click/Enter fixtures for default submitter, disabled default button, multiple implicit blockers, single eligible text control, textarea newline, click cancellation, child-frame request routing and byte-exact session submission.
- [x] Root executes red, then connect scripted native events to the private activation hook and unscripted sessions to the shared native algorithm. Use actual top-session host policy; keep paint untouched unless a concrete defect requires it.
- [ ] Root qualifies seven required crates, runtime integration, both adapters and native host/session fixtures under serial gates; permit only the six recorded baseline Livery failures, retain ignores separately. Freeze actual final source/lock/resources/providers and build a fresh netfetch runner.
- [ ] Capture matched before/after all33 submission and584 encoding records on both engines using live WPT server and original deadlines. Preserve eight PhaseB maps, original20 and corrected20 controls, and sixteen CSS slices. Attribute every file/assertion movement by unchanged source/name/occurrence, keeping partial ERROR/timeouts separate. Inspect zero oldpassloss and all source/build/resource guards.
- [ ] Fresh whole-change review, resolve findings, reconcile canonical docs, commit/push qualified owned work under carried authority. Report Checkpoint C with hashes, fixtures/controls, WPT attribution, per-crate counts, limits and preserved evidence. Stop after this checkpoint.

### Phase C qualified progress, 2026-10-09

Task1 request-aware navigation is committed at `0d86542939af8f946136c5b289c8338679cedade`; the independent transport-supplied location fixture follow-up is `7fd3ddd0eaece16f0b3341d47790becc56ff785e`. Task2 native entries, Unicode17 directionality and shared encoders are committed at `80d481529c3e96775185f069755e7b0c7ade87ce`. Its shared native Auto-state button predicate is consumed by the remaining work. The locked879 package identities/versions/sources/checksums remain fixed; the sole approved native encoding_rs dependency-list edge changes lock SHA from `853946ec7ee2869dc61d87fc8fd9a407b2e22f7b5a1ac5e1d7b6fef7cf786a22` to `36a2ca937af642376141be6d01099a27476843974438678e9ac72f6a1a9bbba4`.

Task3 document encoding is committed at `c94733cea4bdb79f558277cc295b84d35d112092`. It carries canonical encoding/confidence through raw HTML loads, realms, navigation, native/prepared frames and actual WPT loader/template paths. First1024-byte prescan is supported; late declaration parser restart is not claimed. Independent semantic RED57 plus review-fix RED8 preceded production. Two Important findings, MIME empty-unquoted parameter handling and successful empty frame responses, were fixed and scoped re-reviewed clean. Root verified all16 committed product Git blobs against the qualified source, all63 frozen original scenario/helper bodies and both fix modules, and raw receipt hashes. Focused65; covering native168, full runtime869 (195 unit plus674 integration), scripted145, documents70, resources18 and WPT80 pass, zero failures. Three existing WPT ignores remain separate. Audit: `/home/markik/Code/testing/genet/forms/thinkpad/phase-c-task3-final-source-receipt-audit-v1.json`; Git proof: `phase-c-task3-postcommit-git-blob-verification-v1.json`.

Task4 FormData and submission bindings are committed at `5c45bdf0a75` and qualified: F1-F7 and the Nova private-alias finding are addressed, with an accepted scoped review and no new Critical or Important finding. Root verified 212 focused tests, 187 full native tests and 1,062 full runtime tests (211 unit plus 851 integration across 59 groups), zero failures or ignores, on unchanged reviewed source. Every prior 869 runtime and 168 native passing occurrence is retained. The earlier user-requested compile interruption remains preserved separately, with exit -2 and zero executed test summaries. The resumed receipt and qualification audit are `/home/markik/Code/testing/genet/forms/thinkpad/phase-c-thinkpad-forms-runtime-full-resume-v1.receipt.json` and `phase-c-task4-resumed-final-qualification-audit-v1.json`. Checkpoint C remains pending host and final qualification.

Task5 native activation and host submission routes are implemented after the frozen 85-scenario RED qualification. Typed submission records preserve exact native entry/encoding bytes through session factories and Ortet transport; scripted and native activation share the native submission decision. Redirects inherit fragments under ruling R14. The native editor clip correction preserves semantic page clips and suppresses editor-local selection; its scoped review is accepted. On the recorded pre-integration sources, the Documents covering gate passed 135 tests and the complete runtime gate passed 1,064 tests (211 unit plus 853 integration), with no failures or ignores. Raw receipts and root audits remain under `/home/markik/Code/testing/genet/forms/thinkpad`, including `phase-c-task5-native-editor-clip-fix2-controls-f1-documents-root-audit-v1.json` and `phase-c-task5-runtime-fragment-expectation-complete-root-audit-v1.json`.

Mark explicitly requested committing, pushing and merging this progress on 2026-10-11. This publication preserves the completed Forms commits and integrates upstream main, including the Vello overrides; it does not grant Checkpoint C. The remaining covering gates, fresh native/wasm/WPT artifacts, live native and browser transport receipts, matched Phase C WPT captures and final whole-change review remain pending. The browser adapter preflight returned no GPU adapter. Historical gate receipts retain their actual source identities and must not be presented as fresh tests of the upstream integration. Root retains the final qualification obligation.

### 2026-10-15, Livery baseline failures triaged and closed on Linux

The "six explicitly named baseline Livery failures" were recorded on the Fedora ThinkPad (Linux), not on Windows; the three further integration failures that surfaced alongside them were not in that baseline. All nine tests were written on Windows (2026-07-27 to 2026-09-29) and passed there, so the cause is the platform. Windows maps `monospace` to Consolas (advance 0.5498 em, content height 1 em) and default serif/sans to Times New Roman/Arial; Linux maps them to Noto. With serif/sans mapped to the metric-compatible Liberation fonts through a private fontconfig file, six of the nine passed unchanged, which attributes them to fonts.

| Test | Cause | Resolution |
|---|---|---|
| `positioned_descendant_extends_its_scroll_container_range`, `positioned_inset_reuse_updates_nested_scroll_range` | Platform fonts: Noto Serif line height exceeds the fixture's 20px box, adding 2px of scroll range | Bind Ahem to the CSS generics; positioned text set to 8px |
| `live_shape_outside_keeps_forced_break_lines_in_the_selected_float_band` | Platform fonts: Consolas content height equals 1 em, Noto Sans Mono's does not | Ahem generics |
| `live_multi_child_float_and_atomic_inline_use_intrinsic_subtrees` | Platform fonts: 104.46289 is 19 Consolas advances | Ahem at 5px; max-content now asserts 95 |
| `shared_inline_group_height_contains_its_painted_lines` | Platform fonts: Noto Sans content area (21.8px at 16px) exceeds the 20px line height | Ahem generics |
| `inferred_cells_with_nested_tables_match_html_table_glyphs` | Platform fonts, then a real defect: with wider fonts the fixture overflows its 784px container, which exposes the constrained-width defect below | Ahem at 16px so the columns fit; defect recorded by an ignored test |
| `counters_paint_like_literal_text_and_preserve_dom_selection` | Over-strict assertion: generated and literal text agree on glyphs but differ in the last float bits of the origins | Glyph identity exact, origins within 1e-3 |
| `live_shape_outside_reference_boxes_change_lines_but_not_float_placement`, `live_inline_lines_in_an_ordinary_wrapper_share_outer_float_exclusions` | Real bug, hidden by Consolas widths: Parley's line `advance` includes hanging trailing whitespace, so a line whose content exactly filled the band beside a float (126px counted against 120px) was moved below the float | `break_inline_lines` compares `advance - trailing_whitespace`; the field is added to the `genet-parley` fork's `LineBreakData` (see its `GENET_PATCHES.md`). Both tests fail without the fix, on Ahem |

Tests that depend on a platform font now bind the bundled Ahem face to every generic family (`replace_generic_font_resources`, `register_generic_font_bytes`) instead of being skipped.

Two defects found during triage are recorded with `#[ignore]`d repro tests, not fixed here:
- `layout::tests::inline_block_min_content_excludes_hanging_trailing_space`: an inline-block's min-content width includes the hanging space (50px; CSS gives 40px, so shrink-to-fit in 45px should be 45px).
- `table_sibling_blocks::nested_table_cell_does_not_change_constrained_column_distribution`: when table content exceeds its container, a nested `display:table` in an inferred cell changes the outer columns (196px against 261px for plain cells or an HTML table). Separately, the HTML-table case there is 784px wide in a layer whose content box is 782px.

Linux result at this change: `cargo test --no-fail-fast -p genet-livery` passes 663 with 0 failures and 9 ignored across 47 targets. The Livery "six known failures" baseline above no longer applies to Linux, and a Livery failure on this platform is now a regression to investigate. The parley field needs a `genet-parley` republish before the next `genet-livery` publish. The Windows suite was not re-run here.

### 2026-10-13/14, ThinkPad qualification attempt (recorded 2026-10-15)

This work was not written into the repository at the time. Its evidence is at
`thinkpad-l14-f:~/Code/testing/genet/forms/thinkpad` (`phase-c-20261013-*`,
`phase-c-20261014-*`), with the lane's own record in
`phase-c-20261013-resume-ledger-v1.md` and `phase-c-20261014-continuation-v1.md`.
R19-R26 are that lane's rulings, not Mark's; F14 amends R19.

- **R19:** qualify then-current main `2b84478e836` with installed Rust 1.99.0
  and committed renderer pins; earlier receipts keep their 1.98.1 identities.
  *Amended by F14: Rust 1.99.1.*
- **R20:** exact-match WPT accounting needs identical providers and toolchain
  on both sides, so a renderer or toolchain change needs a new matched control
  epoch. The original B and C-before captures are preserved, not reused.
- **R21:** the provider helper verifies Git modes, tracked blobs and gitlinks,
  allowing only Cargo's `.cargo-ok` marker inside Test262.
- **R22:** fresh epoch tooling binds both lock snapshots, all Cargo
  configuration locations, all 15 covering-gate rows and kernel-resource
  receipts.
- **R23:** four render-host hardware fault tests, added after `4bc7558` and
  declared ignored upstream, are accounted by name; no hardware fault
  qualification is claimed.
- **R24:** a candidate native build failed after the control build replaced
  a shared Cargo artifact; control builds must use their own target dirs.
- **R25:** control runner v1 was refused as a baseline because
  `document-session-api` was not compiled from control source; the control
  was rebuilt in isolated target/build dirs compiling all eight changed
  dependencies.
- **R26:** a rebuilt, cached control binds the exact raw Git status before
  Cargo; only its artifact audit qualifies as the matched baseline.

Results on `2b84478e836`, source freeze `ad7e87c4`, Rust 1.99.0: all 15
covering gates passed with no new failure (native-only 25, scripted 145,
Livery 323 with the six Linux failures and one ignore, render 42, WPT unit 80
with three ignores, Boa adapter 26, Nova adapter 9, Ortet 55, resources 18,
Documents 135, runtime 1,064, session API 22, native DOM 187, netfetcher
redirect 3, render-host 9 with the four R23 ignores). The control and
candidate runners were built and the capture configs hydrated, but **no
matched capture ran**. The last run, the native live submission v3 of
2026-10-14, passed on Livery, failed network verification on Boa (the
post-redirect relative image was requested from the pre-redirect directory;
F12) and never reached Nova. The live browser leg was blocked for want of a
WebGPU browser. The lane stopped there; the ThinkPad later rebooted.

### 2026-10-15, Windows qualification begins

F11-F14 are ruled above. Worktree `Code/worktrees/genet-forms-c`, branch
`forms-phase-c`, from origin/main `636da6bbc65`. Rust 1.99.1 installed; the
lock is generated offline from the cargo cache (`--offline`, nothing
downloaded), SHA256 `36934732b27f96da2e0bd1ea2e50bbdffce8c840e76e743d1b56b254915bb37c`.
Evidence: `Code/testing/genet/forms/phase-c-windows-20261015/`. Of the 56
Phase C product paths, only `ports/genet-wpt/src/{harness,main}.rs` and
`ports/ortet/Cargo.toml` changed on main after `2b84478e836`.

### 2026-10-16, F12 work resumes on ThinkPad

Mark reopened Forms work in the standards lane after the published AccName
slices. Current main is `9258e01d8e7`; the fresh implementer owns the repair.
The old owner remains a context resource and must not start another build.
The inherited F12 test design is already committed on main by `273431eff73`;
its earlier compile interruption is not RED evidence.

Rust 1.99.1 minimal plus the wasm32 target are installed here, without changing
the repository-wide toolchain pin. A fresh isolated release target and exact
lock snapshot are under
`Code/testing/genet/forms/thinkpad/phase-c-20261016-f12/`. The inherited Boa/Nova
ignored tests were run with `cargo +1.99.1 test --release --locked -j1`
under the shared heavy lock and kernel memory guard. Both failed: Boa and
Nova requested `https://forms.test/proof.css` and `/proof.png` instead of
the new document's `/final/` resources. A separate retained-CSSOM control
also failed when replacement and original DOM mutation counters matched.

The repair in `components/genet-scripted/livery.rs` reads the current host's
document base URL and tracks DOM arena identity. A replacement arena forces
resource/style reconciliation and invalidates retained layout and paint even
when mutation counters or resource ledgers match. The two inherited redirect
tests are enabled. The fresh release suites pass 137 Documents tests, 146
scripted tests and two integration tests, with zero failures or ignores.
All three observed RED tests pass in those suites. Independent read-only
review found no actionable F12 issue.

The fresh Ortet release build and live native rerun also pass. Livery, Boa and
Nova each exit zero and pass the unchanged network verifier: exactly four
requests per route, `q=é` encoded as `713de90d0a`, a 307-preserved POST to the
final address and the relative image requested from `/final/proof.png`.
All three screenshots are byte-identical and were individually inspected:
the final heading, green marker and green relative image paint correctly;
their final accessibility projections expose the final address, heading and
image. No stale source button remains in those final projections or images.
The historical failing receipt is unchanged. The compact source-bound
[local repair receipt](receipts/2026-10-16_forms_redirect_base.json) records
the raw log, compiler, exact lock, binary and review identities. All five
owned runs completed with zero OOM or memory-limit events.

F11's Windows final-qualification choice remains in force pending Mark's
answer to the explicit platform clarification; local repair work does not
close Checkpoint C. Source identities, RED and GREEN logs and resource
receipts are in the new evidence directory above. The paused Windows notes
below are historical and do not supersede these fresh local outcomes.

The six inherited capture configs were inspected: all their configured output
directories are absent. They describe the older `2b84478e836` source epoch;
their binaries are not fresh qualification for this repair. The original
56-path control projection also predates `273431eff73` and this F12 patch.
Constructing a fresh pre-C control must account for the later C repair/tests
on those same paths while preserving unrelated main changes; reversing only
the old six commits without auditing the resulting source is insufficient.

### Paused 2026-10-15

Mark paused this lane on 2026-10-15 to protect the weekly budget. Checkpoint C
is **not** reached. No covering gate, matched capture, live native run or
browser check ran on Windows.

Done on branch `forms-phase-c` (from `636da6bbc65`, not pushed):

- F11-F14 and the ThinkPad R19-R26 history recorded (`2f387790779`).
- Rust 1.99.1 installed (minimal, with `wasm32-unknown-unknown`); the offline
  lock above is saved as `candidate.Cargo.lock` in the evidence folder.
- F12 repro test written in `components/genet-documents/src/engines/tests.rs`:
  `{boa,nova}_phase_c_t5_scripted_redirected_subresource_base`. A top-level
  POST from `https://forms.test/source` installs
  `https://forms.test/final/index.html`, whose `proof.css` and `proof.png`
  must be requested from `/final/`, not from the initiating document's
  directory. **It has never compiled or run.** The RED build was stopped
  during dependency compilation when the lane paused (`f12-red.log`). Both
  tests carry `#[ignore]` so the branch cannot turn the Documents gate red;
  run them with `-- --ignored` for RED, then remove the attribute with the fix.
- The ThinkPad native live-run tool (`server.py`, SHA256 `34093d86…`, the
  same tool as the 2026-10-14 run) is copied to `native-tool/` in the
  evidence folder.

*Reading, not verified:* the F12 defect is the scripted host's live
stylesheet/image source. `LiveryCssom::install_live` stores the document URL
once in `LiveStylesheetSource::document_url`
(`components/genet-scripted/livery.rs`), and `synchronize_live_styles`
resolves every later stylesheet and image against it. A top-level navigation
inside the scripted runtime installs a new top host and arena. The document
keeps painting through its original `self.cssom`, so the new document's
subresources resolve against the first document's URL. Links are unaffected
because they read `host.document_base_url()`, which explains the
current-address/links fixtures passing. The candidate fix is to resolve
against the host's current `document_base_url()` and to re-resolve and
invalidate when the arena changes. It is not written yet.

Left for Checkpoint C, in order *(all done on O-PC by 2026-10-20; see the qualification run
section at the end of Progress)*:

1. Run the F12 tests RED (Boa and Nova), write the fix, then GREEN plus the
   Documents and scripted suites it touches.
2. Build the control. Its worktree is `Code/worktrees/genet-forms-c-control`
   (branch `forms-phase-c-control`, clean at `636da6bbc65`; delete it if the
   lane is abandoned). Plain `git revert` of the six Phase C commits stops at
   the first one: `d7b524140a7` conflicts in this plan and in
   `ports/ortet/Cargo.toml`. The reversal needs manual resolution limited to
   the 56 product paths, documentation untouched. The attempt was aborted.
3. Run the 15 covering gates on the candidate under `cargo +1.99.1 --release
   --locked -j1`, with zero Livery failures allowed on Windows (F11).
4. Build control and candidate `genet-wpt --features netfetch` runners in
   separate target dirs (R24/R25). Capture both engines on the 33
   form-submission and 584 encoding variants, the eight Phase B maps, the 16
   CSS slices and the original/corrected 20 controls. Attribute every
   movement; any lost earlier pass is a stop-and-report.
5. Run the native live submission on Livery, Boa and Nova.
6. Run the live Chrome submission check under wasm-bindgen 0.2.129 (F13).
7. Whole-change review, Checkpoint C report, ledger and DOC_README rows.

### 2026-10-18, state recorded during a branch sweep

Mark asked (2026-10-18) whether Phase C was still in limbo; it is. Facts checked against
the tree during the mere and genet branch sweep:

- The Phase C product commit `e3911fb6b2d` (Fix form submission validity, listener
  transitions and feedback) is in main; branches `forms-phase-c` and
  `forms-phase-c-control` are both merged and 53 behind main.
- Checkpoint C has not run: no covering gate, matched capture, live native run or browser
  check on Windows since the 2026-10-15 pause. The F12 repro tests still carry `#[ignore]`.
- `Code/worktrees/genet-forms-c-control` holds the aborted manual reversal: 59 staged
  changes (40 modified, 19 deleted, 636 insertions, 15,395 deletions) from 2026-10-16 04:59,
  uncommitted. `Code/worktrees/genet-forms-c` holds one uncommitted change,
  `ports/ortet/src/web.rs` (+21/-4), a guard that refuses a held source once a newer
  navigation has won.

Mark's ruling (2026-10-18): **"Keep both for resumption (Recommended)"**. The lane is paused,
not abandoned; both worktrees stay as they are. Main was fast-forwarded 61 commits the same
day, so a resumption starts by merging main into `forms-phase-c` and rebuilding the control
from the recipe above.


### Checkpoint C qualification run on O-PC, 2026-10-18 to 2026-10-20

**Status:** the run is complete; acceptance of Checkpoint C is Mark's call and is not claimed
here. Everything below ran on O-PC under toolchain 1.99.1 at the prepared epoch (F16):
candidate `e3911fb6b2d` with `8fa73497` and the v4/v5 review manifest's `ports/ortet/src/web.rs`;
control the 59-path inverse C projection over the same head. Evidence lives in `Code/testing/genet/forms/phase-c-windows-20261016/`,
bound by `checkpoint_c_report_20261020.json`.

**Runs.** Four chain runs and one standalone build under the guarded wrapper (floor 6 GiB, F17):
r1 and r2 stopped on memory refusals at the wrapper's 10 GiB launch check (the box sat on that
line); r3 completed the candidate side except CSS and was killed by a session restart; the
control build then ran alone (`qual-wpt-build-control-v2`, 1,790 s, all eight projection packages
compiled fresh, binary `e969c828f0d5…`); r4 ran 11:40 to 19:48 UTC on 2026-10-19 and completed
the rest. Three rows were relabelled: two memory refusals and the CSS slice below.

**Tooling amendments**, each hashed in `continuation-20261018-amendments.json` and
`continuation-20261019-amendments.json`, the frozen 2026-10-16 tools untouched beside them: the
wrapper revision for F17; v2 manifests carrying the `~/.cargo/config.toml` that appeared on the
17th (`[profile.dev] debug = 0`, no effect on release builds); a per-row memory wait and relabel
retries in the capture runner; and, on Mark's answer **"Complete means ran to summary
(Recommended)"** (2026-10-19), a reftest row is complete when it ran to its summary and wrote
its map at exit 0 or 1, because `genet-wpt` exits 1 whenever any reftest fails in
write-expectations mode (`reftest.rs` 887-891); the ThinkPad's own B baseline carries such an
exit 1. The exit code stays in every `process.json`.

**Matched capture, candidate against control, 188 paired rows, zero lost passes.**

| Stage | Pairs | Same | Lost | Gained | Changed |
|---|---|---|---|---|---|
| Submission and encoding (c) | 158 | 430 | 0 | 974,758 | 126,832 |
| Controls (original, corrected, new, both engines) | 6 | 84 | 0 | 42 | 0 |
| Phase B maps (b) | 8 | 7,602 | 0 | 76 | 6 |
| CSS slices | 16 | 2,731 | 0 | 0 | 0 |

File statuses, each engine: encoding 584 files, candidate Boa 439 pass / 145 error, candidate
Nova 571 pass / 13 error, control 583 error / 1 fail on both engines; forms 33 files, candidate
8 pass / 18 fail / 7 error, control 27 fail / 6 error, identical on Boa and Nova. The 974 thousand
gains are encoding subtests the control enumerates as not-run under erroring files and the
candidate passes; the 126 thousand "changed" rows are placeholder subtests the control lists
under files that error on both sides and the candidate does not enumerate (sample:
`eucjp-encode-form-errors-han.html?12001-13000`, 1,000 placeholders against 400). CSS is
byte-for-byte the same on both sides and deviates from the checked-in Linux expectations on zero
of 16 slices. Controls: `original-b20` and `corrected-b20` 20/20 on both sides and engines;
`new-c20` 0/20 on the control and 20/20 on the candidate, both engines.

Forms movements, the same on both engines: gained `FormDataEvent.window`, `SubmitEvent.window`,
`form-data-set-empty-file.window`, `newline-normalization`,
`reparent-form-during-planned-navigation-task` (fail to pass) and `form-double-submit`,
`form-double-submit-3`, `url-encoded` (error to pass); `form-double-submit-2` moved from a harness
error to a run that fails its one assertion. Five files moved fail to error:
`text-plain.window`, `multipart-formdata.window`, `urlencoded2.window`, `submit-file.sub` and
`form-double-submit-multiple-targets`. In each the control fails every case outright, the
candidate passes the first case and then the formdata-event case hits the harness timeout,
leaving the remaining cases not run. That is attributed, not a lost pass, and is the named
follow-up of this run: the formdata-event submission path under the WPT harness on Windows.
The Boa/Nova gap on encoding errors (145 against 13 files) is recorded as observed. Phase B maps:
`the-input-element` gained 38 subtests per engine (button, checkbox and files cases) with three
changed; the other six maps are identical.

**Covering gates.** Twelve run in release under the wrapper, 1,743 passed, 0 failed, 0 historical
pass losses, 0 issues: native-only 25, Livery 342 with 2 ignored, render 67, WPT unit 81 with
3 ignored, Boa adapter 26, Nova adapter 9, Ortet 55, resources 18, runtime 1,078, session API 22,
netfetcher redirect 3, render-host 17 with the four R23 ignores. Three reused on the same product
bytes per the plan's policy: scripted 145, Documents and native DOM from `review-green-native-v2`.
Livery on Windows: zero failures, so F11's no-allowance rule is met.

**Already on record from 2026-10-16 on this box:** the F12 repair review (678 passed), the native
live submission on Livery, Boa and Nova, and the Chrome 155 browser submission check under
wasm-bindgen 0.2.129 (F13).

**Limitations.** Main is 55 commits past the candidate (F16); that delta is not qualified here.
The ThinkPad's C-before maps are not on this machine, so the only cross-epoch reference used is
the checked-in CSS expectations. The run spanned two session restarts and memory contention from
other builds; every job's receipt records its own memory floor and minimum. `checkpoint_c_complete`
stays false in every receipt until Mark accepts.

**Accepted, 2026-10-20.** Mark, asked whether Checkpoint C is reached: **"Accept, with the
formdata-event timeout as a follow-up (Recommended)"**. `checkpoint_c_complete` is true in
`checkpoint_c_report_20261020.json` and its repo copy. Follow-ups, unnumbered until Mark numbers
them: the formdata-event harness timeout on Windows; the delta gate for the 55 main commits past
the candidate epoch.

### Findings, 2026-10-22: the formdata-event timeout is a stale target name

**Cause, product.** The "(formdata event)" case is the second case in each enctype file, and the
first to reuse the iframe name `formtargetframe`. Case one's cleanup removes its iframe:
`detach_subtree` drops the realm-to-context mapping at once but leaves the context in
`frames.tree` until a queued `setTimeout(0)` teardown. testharness starts case two from
microtasks, before that task runs. Case two appends a fresh iframe under the same name and calls
`form.submit()`. `navigation_target` (`components/script-runtime-api/frames.rs`) took the first
tree context with that name, which is the dead one. Its realm lookup then returned `None`, and
the submission was dropped silently. No fetch was issued, the new iframe never fired `load`, and
testharness's file timeout fired after the runner skipped to it. The formdata event and the
Windows server port are not involved. Through a logging proxy in front of the lane's
`windows_wpt_server.py` (unmodified) on current main `d00361b7c6f`, case one sends one
`POST /FileAPI/file/resources/echo-content-escaped.py` and gets 200; case two sends nothing.

**Fix.** A named lookup matches only contexts that still serve a realm. Regression tests:
`phase_c_forms_submission_reused_target_name_{before,after}_teardown` in
`script-runtime-api/tests/forms_submission.rs`, both engines. `before_teardown` failed on main
with zero requests; `after_teardown` is the positive control and passed both before and after.
Form submission is the only product caller of `navigation_target`.

**WPT, current main against the fix**, release runner, `--timeout 120 --drive-deadline 15`, lane
server, Boa and Nova identical:

| File | Before | After |
|---|---|---|
| `text-plain.window` | 1/62, harness-timeout at case 2 | 21/21 passed, drive-deadline |
| `multipart-formdata.window` | 1/62, harness-timeout at case 2 | 21/21 passed, drive-deadline |
| `urlencoded2.window` | 1/62, harness-timeout at case 2 | 21/21 passed, drive-deadline |
| `submit-file.sub` | 0/1, harness-timeout | unchanged |
| `form-double-submit-multiple-targets` | 0/1, harness-timeout | unchanged |

**What still stops the three enctype files: testharness's normal file timeout.** With
`--drive-deadline 120`, all three stop at case 22 on both engines, with the harness timeout
firing mid-case. A diagnostic copy of `text-plain` with `timeout=long` (untracked, deleted
after) ran all 62 cases on Boa: 56 pass and 6 fail. Nova reached 43 passes in 164 s before the
long timeout. The page clock is wall time plus skipped time (`timing.rs`), and each submission
round-trip costs about 0.75 s of it on Boa, so 62 cases do not fit in a normal-timeout file. The
six real failures are encoding: characters outside a non-UTF-8 `accept-charset` are sent raw
instead of as `&#NNN;` references (four cases), and a lone surrogate in the normal-form case is
sent as `\uD800` rather than U+FFFD (two cases).

**The other two files fail for other reasons.** The capture report grouped them with the three
above, but neither passes a first case.
- `submit-file.sub` posts to `//www1.localhost:18800/...`. On O-PC, `getaddrinfo("www1.localhost")`
  fails (11001), netfetcher resolves through the OS (tokio `lookup_host` in murm), and no request
  reaches the server. This is the WPT host setup that upstream's `wpt make-hosts-file` provides,
  absent on this box: a harness environment defect, not product.
- `form-double-submit-multiple-targets` submits one form three times, expecting three
  navigations. HTML's planned navigation cancels the form's earlier planned navigation on each
  submit, and genet pins that in `pending_replacement`. The test's own header says the
  expectation "is not explicitly specified". This is a spec-versus-browsers fork.

**Proposals, unnumbered, for Mark:**
1. Land the target-name fix (branch `formdata-timeout`).
2. Per-file pacing for the enctype files: leave them as a recorded limitation; give server-mode
   runs a timeout multiplier, as wptrunner's `--timeout-multiplier` does; or treat the
   per-navigation cost as product work.
3. `submit-file.sub` on Windows: install the WPT hosts entries on O-PC (an OS change, so Mark's
   to make); have netfetcher treat `*.localhost` as loopback (RFC 6761 allows it, Chrome does it);
   or record it as a Windows limitation.
4. `form-double-submit-multiple-targets`: keep the spec's single navigation (spec is the guide),
   or match browsers.

**Crate gate.** The full script-runtime-api suite (debug, both engines) with the fix: 1,208
passed, 10 failed. All ten fail the same way on main `d00361b7c6f` with the fix reverted, so
they are not caused by it: `forms_submission` `implicit_default_button`, `implicit_one_blocker`
and `implicit_key_modifiers`; `cross_arena_replacement` `atomic_refusal`; and
`frame_arena_boundary` `associated_subtree_boundary`, each on both engines. The runtime gate
passed 1,078 at the candidate in release, so these fall to the delta gate already named above.
Evidence: `Code/testing/genet/forms/phase-c-windows-20261016/formdata-followup-20261022/`.
