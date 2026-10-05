# Genet census panic lane receipt

**Status:** Implementation and automated validation complete, 2026-10-04;
integrated on 2026-10-05 after Mark authorized push and merge. The validation
inputs and maps below retain their original frozen Vano pin.

Plan: [census execution](../../2026-10-03_census_panics_plan.md).
Evidence root: `C:/Users/mark_/Code/testing/genet-census-panics`.
Worktree: `C:/Users/mark_/Code/worktrees/genet-census-panics`, branch
`fix/census-panics`, based on published Genet `bcf1b08dbe6`.
Vano is frozen at `8ad0841255c2cbb679f7c704417d427b3cb3e961`.

## Starting records

All ten historical Vano findings reproduce with isolated workers. The current
Boa comparison uses the same source, runner, WPT manifest and collection policy.
Ordinary FAIL and subtest counts are distinct from a panic or evaluation error.

| Record | Boa | Vano | Qualified cause |
|---|---|---|---|
| P1 `Document-createElement` | GC policy panic, thrown object | Same | Policy rematerializes a weak-dead wrapper through `wrapNode`, calling an author custom-element constructor; `lib.rs` expects the hook to succeed. |
| P2 customized built-ins | FAIL 0/4 | GC policy panic, thrown object | Same policy/constructor path. |
| P3 `Node-cloneNode` | GC recursion panic | GC stack-limit panic | Same policy re-entry; constructor recursion escapes the private hook. |
| P4 parser-created upgrade | FAIL 1/2 | GC stack-limit panic | Same policy re-entry through parsed custom-element candidates. |
| P5 inactive autofocus | `live frame` panic | Same | Child parser script removes its iframe; the loader then assumes the removed record exists. |
| P6 XML-named input-stream bailout | `live frame` panic | Same | Parent callback removes the active child's record during parsing. XML/MIME behavior is a separate gap. |
| P7 `Node-textContent` | C3 colour panic | Same | Document setter illegally inserts bare text; its layout path reaches unresolved `System(CanvasText)` in `text.rs::brush`. |
| E1 interface objects | Event undeclared evaluation error | Same | Legitimate author deletion of the public constructor breaks later UA-owned lifecycle dispatch. |
| E4 SVG focus events | Not-callable evaluation error | Same | SVG focus is missing. |
| E5 SVG ResizeObserver | Constructor undeclared | Same | ResizeObserver is missing. |

The exact failing stale ID for P1-P4 was not dynamically captured. Symbolized
traces, source attribution, and the forced weak-death regressions qualify the
constructor re-entry mechanism. `checkpoint-a.md` retains the options and full
trace qualifications. The diagnostic in-process runner changes Vano outcomes
and is excluded from acceptance comparisons.

## Rulings and fixture controls

Mark selected live canonical wrappers and bookkeeping that cannot call author
code for P1-P4. He selected queued destruction before remaining child load work
for P5/P6, after the current script finishes.

| Record | Owning fixture/filter | Qualified failing control |
|---|---|---|
| P1 | `script-runtime-api`: `census_p1_create_element_policy_does_not_reenter_constructor` | `control-gc-final.log`, both engines panic at the original GC-policy edge. |
| P2 | `script-runtime-api`: `census_p2_customized_builtin_policy_does_not_reenter_constructor` | Same control, both engines panic. |
| P3 | `script-runtime-api`: `census_p3_clone_node_policy_does_not_reenter_constructor` | Same control, both engines panic. |
| P4 | `script-runtime-api`: `census_p4_parser_upgrade_policy_does_not_reenter_constructor` | `control-gc-parser-release.log`, both engines panic. GC production remains original; independent E1/frame/P7 fixes are present, explicitly recorded in the receipt. |
| P5 | `script-runtime-api/frame_lifecycle`: `self_removal_finishes_without_load` | `control-frame-self-removal.log`, both engines panic at `live frame`. |
| P6 | `script-runtime-api/frame_lifecycle`: `parser_callback_releases_barrier` | `control-frame-top-callback.log`, both engines panic at `live frame`. |
| P7 | `genet-scripted-dom`: `document_text_content_setter_preserves_its_children` | `control-p7.log`, original setter replaces children. |
| E1 | `script-runtime-api/parser_script_interleaving`: `readiness_events_survive_deleting_event` and `repeat_parse_after_event_deletion` | `control-e1.log` and `control-e1-repeat-pure.log`, both engines lose UA events. |

GC intrinsics poison also fails on both original engines in
`control-gc-intrinsics.log`. Frame ordering review adds controls for the parser
tail (`control-frame-parser-tail.log`, next script wrongly runs) and reentrant
Window load (`control-frame-reentrant-load.log`, detached owner wrongly loads).
Each has both-engine failures against the intermediate production state before
its correction. Failed compiles and passing exploratory fixtures are retained
but are not treated as failing controls.

The descriptor/index poisoning control `control-gc-array-descriptor.log`
fails on both engines when an inherited author getter is called during GC
property descriptor conversion. The corrected descriptor has a null prototype.
`fixed-gc.log` passes all 12 P1-P4 and poisoning regressions across Boa and Vano.
The unescaped nested frame fixture fails on both engines in
`control-frame-ancestor-load.log` and `fixed-frames-final.log`. Independent review
found that a literal closing script tag in serialized nested srcdoc terminates
the containing script before the handler registers. These failures are
unqualified and retained only as exploratory evidence. The corrected fixture
escapes `<` in serialized JavaScript literals and supplies explicit bodies.
It reaches the nested handler and removal assertions on both engines, then
fails the ancestor completion assertion with original `frames.rs` in
`control-frame-ancestor-valid.log` (0 passed, 2 failed). The receipt records
the independent GC/E1/DOM/parser-abort fixes present and byte-for-byte frame
source restoration. The final fixture passes on both engines in
`fixed-frame-ancestor-instrumented.log`; the initial full frame suite passes
12/12.

The first broad after round is exploratory. It lost subtest passes to worker
timeouts. Review found that the new GC member scan searched all later groups
before clamping to its group boundary, making many detached singleton groups
quadratic. The first group-local correction reduced the measured cost but did
not resolve end-to-end scaling. The current parser uses captured native split
with private null-prototype separators; its own coercions return fixed ASCII
delimiters. This avoids author prototype hooks and the explicit suffix scan.
`directory-results/after/initial/interruption.receipt.json` preserves the owned
process stop and completed maps. The new immutable `captured-split` round completes below with zero lost passes;
the earlier runner and results remain untouched.

Read-only backend inspection used locked Boa
`52cfb6ff9efdaf0ff6d213ab107bbaefe5838ab0`,
`core/engine/src/builtins/string/mod.rs` and `core/string/src/{lib,str}.rs`,
and locked Vano `8ad0841255c2cbb679f7c704417d427b3cb3e961`,
`nova_vm/src/ecmascript/builtins/text_processing/string_objects/string_prototype.rs`
and `nova_vm/src/ecmascript/types/language/string{,/data}.rs`.
Neither materializes the entire receiver per search for these ASCII generated
specs: Boa uses string views and Vano borrows valid UTF-8. No backend code was
copied or translated.

The separator positive control `control-gc-split-separator.log` removes only
the two private separators' null prototypes. Both engines panic at the private
GC hook after calling the poisoned inherited `Symbol.split` getter. The final
captured parser passes all 12 constructor/intrinsic/descriptor regressions in
`fixed-gc-captured-split.log`. This matches
[String.prototype.split's separator dispatch](https://tc39.es/ecma262/multipage/text-processing.html#sec-string.prototype.split):
the private separator has no inherited symbol hook and only a trusted own
coercion. The parser uses own array indices populated by the native operation.

Temporary real-runtime singleton cost diagnostics retain 2,000/8,000 distinct
detached wrappers, time only policy application, then assert count, identity,
detached state and node markers. Sources are restored exactly after each run.
`gc-singleton-cost-controls.receipt.json` records the initial explicit-suffix
scan and provisional group-local scan; its label `linear` was chosen before
measurement and does not establish linear end-to-end runtime cost. The
same-fixture original-hook comparison also scales steeply (Boa 1.086/15.554s;
Vano 1.026/13.102s), so end-to-end scaling is partly pre-existing. The current
captured-split result is Boa 1.286/16.365s and Vano 1.732/13.715s, near the
original hook at the larger size. These single diagnostic samples qualify the
explicit scan correction, not a general benchmark or a linear-runtime claim.
`gc-captured-split-controls.receipt.json` records all controls, exact source
compositions and byte-for-byte restoration. No diagnostic test is retained in
production sources.

## Authority and scope

The [DOM textContent setter](https://dom.spec.whatwg.org/#dom-node-textcontent)
does nothing for Document/DocumentType. The upstream setter and pre-mutation
hooks are corrected; the paint invariant and Livery source stay unchanged.

[Iframe removing steps](https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element:html-element-removing-steps)
destroy the child navigable. [Document destruction](https://html.spec.whatwg.org/multipage/document-lifecycle.html#destroy-a-document)
aborts its parser and removes pending document tasks. Removal is distinct from
navigation's unload procedure. Mark's R2 resolves the remaining ordering choice
between queued destruction and load completion.

Deferred features: SVG focus, ResizeObserver, XML navigable parsing/MIME
metadata and the XML `document.open()` exception. Stream getter traps remain
in the Streams lane. Genet's Vano repin is a separate integration step after
this lane lands, as ruled on main in `f4ae933c873`; validate affected tests at
the new pin in that separate commit.

## Final gates and retained state

The final isolated per-file round uses the immutable
`final-captured-split-genet-wpt.exe`, SHA-256
`3BFE9996FF931CF0C9339E0622F0055A1553B50271E49072A7E065E0D57EF2A0`.
Build receipt `final-captured-split-build.receipt.json` records successful
`+1.97.1 build --locked -p genet-wpt --features netfetch -j 2`, identical
before/after production-and-test source patch SHA-256
`DBFA2C1568A5131A0E17DB4F7CD33452B873EE122DCA367C0BBE1D79DB7C6AAE`,
and unchanged ignored lock hash
`8409C86F0AC7EF8009E0B2A421EA46DBD0A9CFDA13D9B4AB38E0C3B8032BCACB`.
Starting source is `bcf1b08dbe663ecfad1ed6c7b2ebe75e600aa39e` plus that patch;
no temporary diagnostic source remains. All twenty runs retain jobs=1,
timeout=30s, drive-deadline=15s, Livery, collection enabled and isolated workers.
Exact commands/maps are in `final-captured-split-file-runs.json`.

| Record | Boa final | Vano final |
|---|---|---|
| P1 | FAIL 11/36 | FAIL 11/36 |
| P2 | FAIL 0/4 | FAIL 0/4 |
| P3 | FAIL 4/6 | FAIL 4/9 |
| P4 | FAIL 1/2 | FAIL 1/6 |
| P5 | FAIL 0/1 | FAIL 0/1 |
| P6 | FAIL 0/4 | FAIL 0/4 |
| P7 | FAIL 59/81 | FAIL 59/81 |
| E1 | FAIL 1/2 | FAIL 1/2 |
| E4 | ERROR, evaluation threw | ERROR, evaluation threw |
| E5 | ERROR, evaluation threw | ERROR, evaluation threw |

No P1-P7 final file reports a Rust panic. The initial Boa P6 worker kill is
resolved in this round; its ordinary XML-feature failures remain. The P3 Boa
result includes a subtest timeout, distinct from a whole-worker kill.
Both P6 results also contain a beforeunload subtest timeout.
Remaining conformance includes custom-element ordering/construction behavior,
P5 retained inactive-document focus, P6 XML typing/exception behavior, P7
Document getter and null conversion, and E1 EventTarget enumerability. These
ordinary failures are not reported as full WPT passes. E4/E5 remain missing
features; their original evaluation errors persist.

All six final-source crate gates pass in `captured-split`, with no failures or
ignored tests. Commands, exit codes and log hashes are recorded in
`final-crate-gates.receipt.json` and `crate-gates.json`.

| Crate | Tests passed | Result blocks | Cargo exit |
|---|---:|---:|---:|
| script-engine-api | 0 (compile and doctests) | 2 | 0 |
| script-engine-boa | 26 | 2 | 0 |
| script-engine-nova (Vano) | 43 | 5 | 0 |
| genet-scripted-dom | 75 | 5 | 0 |
| script-runtime-api | 672 | 46 | 0 |
| genet-scripted, with scripted-nova | 120 | 3 | 0 |
| Total | 936 | 63 | |

The runtime gate includes 156 unit tests, both GC soaks/cost checks, 12 frame
lifecycle tests and all 12 initial-blank tests. The first runtime run's two
obsolete removal-unload expectations remain in the initial log; corrected
expectations keep the listeners and prove no removal unload occurs. The six
final Cargo exits and every result block are zero/green. The orchestration
session itself returned 1; fresh evaluation of its latest-record failure
predicate finds zero failures. The receipt retains that wrapper result
separately rather than claiming the wrapper exited successfully.

No paint or layout source changed. The conditional reftest guard and Livery
crate gates therefore do not apply to this lane. The protected-file audit,
unchanged dependency manifests, lock hashes and clean primary checkout are
recorded in `final-source-audit.json`. Primary main advanced independently to
`f4ae933c873` with the Vano repin ruling; its source and ignored lock were not
changed by this worktree.

## Broader WPT comparison

All eight directory runs complete successfully with matching manifest, scope,
collection settings and four isolated workers. There are 859 records per
engine, including 833 testharness variants and 26 non-testharness entries.
Manifest SHA-256 is
`D5EC5BE9BF1A75ED00D7E7AB28AFE8A694A55E11682BA74305874D70B18DD422`.
The starting frozen runner is SHA-256
`D2AFF5CDB059F45BDB789A9475C04138AB00E2D780E946D3DEA413927F9FA1C6`;
the final runner and source patch identities appear above. Exact arguments,
source snapshots and map digests are in each round's `run-index.json`.
Both indices report no execution or provenance failure.

`directory-results/after/captured-split/comparison.json` compares every file and
individual subtest by name plus occurrence. Missing subtests count as absent.
It records **zero file pass losses and zero passing-subtest losses**, 33 file
movements, and 3,092 newly visible subtests: 2,155 pass, 917 fail and 20 timeout.
All subtest movements are absent-to-reported; there are no paired status
changes or standalone reason-only changes. These visibility gains are not
2,155 repaired assertions. The runner contains several fixes, and hang-killed
or no-results recoveries have not been individually bisected.

Counts below are pass / fail / error / no-results / skip.

| Directory | Engine | Records | Before | After | Passing subtests before -> after |
|---|---|---:|---|---|---:|
| `custom-elements` | Boa | 187 | 11 / 149 / 8 / 9 / 10 | 11 / 158 / 0 / 8 / 10 | 368 -> 2249 |
| `custom-elements` | Vano | 187 | 12 / 149 / 8 / 8 / 10 | 12 / 156 / 1 / 8 / 10 | 265 -> 392 |
| `dom/nodes` | Boa | 330 | 127 / 147 / 13 / 1 / 42 | 127 / 148 / 12 / 1 / 42 | 6703 -> 6762 |
| `dom/nodes` | Vano | 330 | 127 / 147 / 13 / 1 / 42 | 127 / 148 / 12 / 1 / 42 | 6703 -> 6762 |
| `html/interaction/focus` | Boa | 180 | 6 / 166 / 1 / 4 / 3 | 6 / 167 / 0 / 4 / 3 | 117 -> 117 |
| `html/interaction/focus` | Vano | 180 | 6 / 166 / 1 / 4 / 3 | 6 / 167 / 0 / 4 / 3 | 117 -> 117 |
| `html/webappapis/dynamic-markup-insertion` | Boa | 162 | 85 / 58 / 9 / 6 / 4 | 85 / 66 / 2 / 5 / 4 | 109 -> 125 |
| `html/webappapis/dynamic-markup-insertion` | Vano | 162 | 84 / 62 / 6 / 6 / 4 | 85 / 66 / 2 / 5 / 4 | 112 -> 125 |

Every movement is listed next. Newly reported counts are pass / fail / timeout.
`R1` refers to the private GC-hook correction with the P1-P4 and poisoning
controls; additional custom-element records share that panic mechanism but
were not separately bisected. `P7` is the no-op Document/DocumentType setter
control. `R2` is the script-completion/destruction correction with the P5/P6,
parser-tail and reentrant-load controls. `Visibility` qualifies a recovery from
worker kill or missing subtests; it makes no narrower causal or conformance
claim. The raw comparison retains every exact subtest name and occurrence.

| Engine | File | Before -> after | Newly reported P/F/T | Attribution |
|---|---|---|---:|---|
| Boa | `custom-elements/Document-createElement.html` | error (panic) -> fail | 11 / 25 / 0 | R1; ordinary custom-element failures now reported. |
| Boa | `custom-elements/ElementInternals-role.html` | error (panic) -> fail | 0 / 68 / 0 | R1; ordinary custom-element failures now reported. |
| Boa | `custom-elements/adopted-callback.html` | error (hang-killed) -> fail | 0 / 71 / 0 | Visibility; no individual causal bisection. |
| Boa | `custom-elements/registries/Element-customElementRegistry-exceptions.html` | error (panic) -> fail | 0 / 3 / 0 | R1; ordinary custom-element failures now reported. |
| Boa | `custom-elements/registries/valid-custom-element-names.html` | error (hang-killed) -> fail | 1861 / 114 / 0 | Visibility; no individual causal bisection. |
| Boa | `custom-elements/throw-on-dynamic-markup-insertion-counter-construct.html` | error (hang-killed) -> fail | 5 / 6 / 0 | Visibility; no individual causal bisection. |
| Boa | `custom-elements/throw-on-dynamic-markup-insertion-counter-reactions.html` | error (hang-killed) -> fail | 0 / 11 / 0 | Visibility; no individual causal bisection. |
| Boa | `custom-elements/upgrading/Node-cloneNode.html` | error (panic) -> fail | 4 / 1 / 1 | R1; ordinary custom-element failures now reported. |
| Boa | `custom-elements/upgrading/upgrade-custom-element-error-event.html` | no-results (no-subtests) -> fail | 0 / 3 / 1 | Visibility; no individual causal bisection. |
| Vano | `custom-elements/Document-createElement-customized-builtins.html` | error (panic) -> fail | 0 / 4 / 0 | R1; ordinary custom-element failures now reported. |
| Vano | `custom-elements/Document-createElement.html` | error (panic) -> fail | 11 / 25 / 0 | R1; ordinary custom-element failures now reported. |
| Vano | `custom-elements/ElementInternals-role.html` | error (hang-killed) -> fail | 0 / 68 / 0 | Visibility; no individual causal bisection. |
| Vano | `custom-elements/adopted-callback.html` | error (hang-killed) -> fail | 0 / 71 / 0 | Visibility; no individual causal bisection. |
| Vano | `custom-elements/builtin-coverage.html` | error (hang-killed) -> fail | 111 / 333 / 0 | Visibility; no individual causal bisection. |
| Vano | `custom-elements/upgrading/Node-cloneNode.html` | error (panic) -> fail | 4 / 5 / 0 | R1; ordinary custom-element failures now reported. |
| Vano | `custom-elements/upgrading/upgrading-parser-created-element.html` | error (panic) -> fail | 1 / 5 / 0 | R1; ordinary custom-element failures now reported. |
| Boa | `dom/nodes/Node-textContent.html` | error (panic) -> fail | 59 / 22 / 0 | P7; getter/null-conversion failures remain. |
| Vano | `dom/nodes/Node-textContent.html` | error (panic) -> fail | 59 / 22 / 0 | P7; getter/null-conversion failures remain. |
| Boa | `html/interaction/focus/the-autofocus-attribute/skip-not-fully-active.html` | error (panic) -> fail | 0 / 1 / 0 | R2; retained inactive-document focus remains. |
| Vano | `html/interaction/focus/the-autofocus-attribute/skip-not-fully-active.html` | error (panic) -> fail | 0 / 1 / 0 | R2; retained inactive-document focus remains. |
| Boa | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/abort-refresh-immediate.window.html` | error (hang-killed) -> fail | 4 / 0 / 2 | Visibility; no individual causal bisection. |
| Boa | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/abort-while-navigating.window.html` | error (hang-killed) -> fail | 4 / 3 / 2 | Visibility; no individual causal bisection. |
| Boa | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/active.window.html` | error (hang-killed) -> fail | 0 / 6 / 1 | Visibility; no individual causal bisection. |
| Boa | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/bailout-exception-vs-return-origin.sub.window.html` | error (hang-killed) -> fail | 0 / 6 / 0 | Visibility; no individual causal bisection. |
| Boa | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/bailout-exception-vs-return-xml.window.html` | error (panic) -> fail | 0 / 3 / 1 | R2; XML exception failures and beforeunload timeout remain. |
| Boa | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/bailout-side-effects-ignore-opens-during-unload.window.html` | no-results (no-subtests) -> fail | 0 / 2 / 1 | Visibility; no individual causal bisection. |
| Boa | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/event-listeners.window.html` | error (hang-killed) -> fail | 2 / 14 / 2 | Visibility; no individual causal bisection. |
| Boa | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/tasks.window.html` | error (hang-killed) -> fail | 6 / 2 / 2 | Visibility; no individual causal bisection. |
| Vano | `html/webappapis/dynamic-markup-insertion/document-write/032.html` | error (hang-killed) -> pass | 1 / 0 / 0 | Visibility; no individual causal bisection. |
| Vano | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/abort-while-navigating.window.html` | no-results (no-subtests) -> fail | 4 / 3 / 2 | Visibility; no individual causal bisection. |
| Vano | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/bailout-exception-vs-return-xml.window.html` | error (panic) -> fail | 0 / 3 / 1 | R2; XML exception failures and beforeunload timeout remain. |
| Vano | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/event-listeners.window.html` | error (hang-killed) -> fail | 2 / 14 / 2 | Visibility; no individual causal bisection. |
| Vano | `html/webappapis/dynamic-markup-insertion/opening-the-input-stream/tasks.window.html` | error (hang-killed) -> fail | 6 / 2 / 2 | Visibility; no individual causal bisection. |

Boa's exploratory 112 absent prior passes are preserved in the final round:
111 per-tag `builtin-coverage` passes and the `htmlconstructor/newtarget`
prototype-getter exception pass. The timeout correction is accepted on the
complete final comparison, not by discarding those exploratory losses.

## Remaining forks and integration checkpoint

- **SVG focus (E4):** still fails evaluation because the SVG focus callable is
  missing. A dedicated SVG focus feature slice or continued explicit deferral
  are the available scope choices; this lane does not add that feature.
- **ResizeObserver (E5):** still fails evaluation because the constructor is
  missing. Its own implementation/notification design lane or continued
  deferral is required.
- **XML document metadata and parsing:** P6 now reports ordinary exception
  failures, while the frame loader still supplies text to an HTML stream.
  MIME/XML document state belongs in the queued Realms metadata work; a
  dedicated XML parser/exception slice can follow it. It is not an R2 failure.
- **Retained inactive-document focus:** P5 can finish but retained document
  `activeElement` does not expose the expected body. Its retained DOM/focus
  view needs a Realms follow-up, either in the metadata lane or a separate
  bounded focus slice. Retained body wrappers already remain usable.
- **Other ordinary conformance:** custom-element ordering/construction,
  Document textContent getter/null conversion and EventTarget enumerability
  remain failing assertions. They can be prioritized as separate bounded
  defect slices; expanding this panic lane would obscure its controls.
- **Reflector follow-ups:** the existing between-tick detached-wrapper window
  remains open in the reflector plan. Singleton policy cost also scales
  steeply in the original-hook control; the diagnostic samples justify
  removing the introduced suffix scan, not claiming general linear cost.
- **Streams getter traps:** unchanged and owned by the queued Streams lane.
- **Vano repin:** make its own commit after census integration, from `8ad08412`
  to the published Vano fixes at `47f8d4f9`, and rerun affected gates. Main's
  independent repin ruling is `f4ae933c873`. No repin is mixed into these maps.

R1/R2 are answered and implemented; there is no remaining semantic question
for this bounded lane. The brief's final checkpoint says to commit locally,
report and stop before push/merge. Production commit:
`876c7a2cba4cd04ff84b2e1f85f6ada38f3ac049`:
`fix(runtime): eliminate census GC and frame panics`.
The following documentation commit records this final receipt and canonical
pointers; both hashes/subjects are recorded after commit in the external
`local-commits.receipt.json`. The acceptance runner was built from the exact
production patch now committed; documentation edits do not alter that source.

The lane worktree, stable target `C:/t/cargo-targets/genet-census-panics`, frozen
runners/symbols and raw evidence are retained for verification and review.
No isolated Cargo home was created.

## Integration addendum, 2026-10-05

Mark authorized push, merge and continuation. The two lane commits were pushed
and merged on main as `24bec750334dd4024738ea860610aef90a5f2c56`. The merged components/ports
patch from the original base has SHA-256
`DBFA2C1568A5131A0E17DB4F7CD33452B873EE122DCA367C0BBE1D79DB7C6AAE`,
identical to the acceptance runner. Main's independent Vano repin ruling is
preserved. `integration.receipt.json` records publication and the subsequent
worktree retirement; the target is temporarily reused for the separate exact-pin
repin gate. No old result map is relabelled as a new-pin result.


## Separate Vano repin accepted, 2026-10-05

Published repin `d7f08fecdc7bfc45d6b2b3a611513caeee925d4c` updates the two
native/wasm64 manifest rows to Vano `47f8d4f9d6fca884e3472e405872ce9556ff9db0`.
The portable ignored lock changes only the three Vano git source revisions;
all 867 package versions and dependency edges are unchanged. Primary's local
patch resolution was not used. Runner SHA-256:
`EBB204DFA6EB471E5A8BCA3B4E1745C16AC048458BD04E3442BD30AF47728D30`.

| Affected gate | Passed | Failed / ignored |
|---|---:|---:|
| script-engine-nova | 43 | 0 / 0 |
| script-runtime-api | 672 | 0 / 0 |
| genet-scripted with scripted-nova | 120 | 0 / 0 |
| genet-wpt vano_reporting | 1 | 0 / 0 |

The old/new Vano comparison uses immutable dev/netfetch runners and the same
WPT manifest, isolated workers, collection on, jobs 1, timeout 30, drive 15,
Livery. IndexedDB file support changes from panic to FAIL 0/3; module imports
from panic to PASS 5/5; dynamic-import promise-result compilation from panic
to FAIL 2/4. All ten named Genet records retain their existing assertion
results after qualified reruns, without observed Rust panic. E4/E5 keep their
missing-feature evaluation errors.

P1/P2 initially timed out under the new runner, losing 11 visible P1 passes.
Those raw maps remain. Subsequent paired old/new runs recover exactly the same
11 pass / 25 fail and 0 pass / 4 fail subtest identities respectively; zero
previously passing files or subtests are lost in the accepted comparison.
Initial compilation and checkout copying had finished before the pair, while
remaining crate gates continued. Encoding compilation began after the pair.
This qualifies recovery without proving the timing cause. All other named
Genet results match the previous frozen census map.

Raw logs/maps, lock/source audits, gate counts and paired comparison are in
`Code/testing/genet/vano-repin/`; `acceptance.receipt.json` names the exact
source. Original census directory measurements remain at `8ad08412`.
Concurrent image-feature work is outside this receipt. Checkout/cache retirement
is recorded in `Code/testing/genet-census-panics/integration.receipt.json`.
