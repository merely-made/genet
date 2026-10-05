# Genet census panics and evaluation errors

**Status:** Implementation and automated validation complete under R1/R2,
2026-10-04; integrated on 2026-10-05 in `24bec750334` after Mark
authorized push, merge and continuation. Validation remains qualified to the
frozen Vano pin used for attribution. Starting published Genet commit
`bcf1b08dbe663ecfad1ed6c7b2ebe75e600aa39e`; Vano remains at the brief's
`8ad0841255c2cbb679f7c704417d427b3cb3e961` for starting attribution.

Authority: [census attribution](2026-09-06_web_platform_wpt_census.md#purpose),
the [reflector identity plan](2026-09-07_reflector_identity_plan.md), and the
[Realms plan](2026-09-08_realms_plan.md). The execution brief is
`Code/work/briefs/2026-10-02_genet_census_panics.md`.

## Scope and checkpoints

Reproduce P1-P7 and E1/E4/E5 on both Boa and Vano. Fix the seven Genet panics
and evaluation defects in existing features. ResizeObserver and other missing
features are scope forks; the two stream getter records belong to the Streams
lane. Changes to Vano or Livery's `text.rs` are outside this lane.

Checkpoint A precedes fixes: report all ten records on both engines, including
panic backtraces, root locations and engine specificity. P1-P4 always require a
choice about GC-policy failure handling and wrapper identity. A nonreproducing
record or a root in Vano/line breaking stops the whole lane. If P1-P4 are the
only checkpoint, independent work on other records may continue while the
question is pending. Ambiguous HTML lifecycle semantics also require a ruling.

## Phases and done-conditions

1. **Reproduce and attribute.** Freeze the starting runner, lockfile, manifest,
   source tree and exact commands. Run all ten records on both engines, retain
   logs/maps/backtraces, and bring the required checkpoint with concrete options.
2. **Implement the ruled scope.** One owning-crate fixture per fixed record,
   with a recorded failure on starting source and a passing result after the
   root fix. P7 must identify where the unresolved colour originates and repair
   its owning invariant; P5/P6 must follow HTML frame
   lifecycle semantics. Avoid panic catches or placeholder values.
3. **Verify.** P1-P7 have no panic on either engine; retain final file statuses
   and subtest counts. Run all touched crates plus `script-engine-boa`,
   `script-engine-nova`, `genet-scripted-dom`, `script-runtime-api`, and
   `genet-scripted`. Include `genet-livery` and `livery` if paint changes.
   Compare before/after on both engines for `custom-elements`, `dom/nodes`,
   `html/interaction/focus`, `html/webappapis/dynamic-markup-insertion`, and
   `svg/interact` if touched: zero pass-to-nonpass, every movement attributed.
   Paint/layout changes also require the 16-entry reftest guard at unexpected=0.
4. **Report and stop.** Commit only to `fix/census-panics`; report provenance,
   fixtures/controls, comparisons, crate counts and unresolved forks before
   any further integration decision.

## Findings

- 2026-10-03: the merged S1.1 manifests require the two previously approved
  direct ICU4X edges in the ignored lockfile. The primary lock was first copied;
  the accepted line-lane lock differs only by adding `icu_properties` and
  `icu_segmenter 2.2.0` to genet-livery's dependency list. The census worktree
  uses that tested lock; no package/version changed and the primary ignored
  lock remains untouched. Build inputs are recorded in
  `Code/testing/genet-census-panics/phase1-build-inputs.json`.
- 2026-10-03: source-only triage identifies the GC-policy error-to-panic edge
  at `components/script-runtime-api/lib.rs:1500-1501`, and the post-script
  frame-record assumption at `components/script-runtime-api/frames.rs:1344`.
  These are hypotheses pending the current runner's Phase 1 receipts.
- 2026-10-03: authored deletion of `window.Event` in the interface-objects
  WPT is legitimate. Lifecycle dispatch source uses the mutable public `Event`
  constructor (`components/script-runtime-api/parse.rs:349,357`); attribution
  must distinguish those UA dispatches from test evaluation before a fix.
- 2026-10-04: P7's copied original fixture panics on both engines with the
  frozen starting runner. Omitting only the Document `textContent` assignments
  produces ordinary fail 59/81 on both. The DOM implementation currently routes
  Document through replace-all, contrary to the DOM setter algorithm's no-op
  rule. These walk-discovery controls qualify upstream mutation attribution;
  they do not replace the manifest-pinned baseline or yet prove every legal
  used-colour path. No production patch has been applied.
- 2026-10-04: isolated symbolized P7 traces identify
  `System(CanvasText)` at `text.rs::brush`, reached while formatting the text
  improperly inserted into Document by its setter. The selected fix repairs
  that upstream DOM dispatch and its range/frame pre-hooks. The colour
  invariant, `paint.rs` and line-breaking `text.rs` remain unchanged.

## Progress

- 2026-10-03: Mark authorized "Push merge proceed". Vano's reviewed commits
  are published at `47f8d4f9`; Genet S1.0-S1.1 are merged and published at
  `bcf1b08dbe6`. The next lane starts in
  `Code/worktrees/genet-census-panics`, branch `fix/census-panics`, target
  `C:/t/cargo-targets/genet-census-panics`. Three GPT-6 Luna agents perform
  bounded read-only attribution; root owns source changes, builds and receipts.
  Network access is authorized by the earlier user clarification; locked
  package versions and remaining checkpoints remain in force.
- 2026-10-04: all ten historical Vano records reproduce. Boa also panics on
  P1, P3, P5, P6 and P7; P2 fails 0/4 and P4 fails 1/2. E1, E4 and E5 retain
  their corresponding evaluation failures on both engines. The frozen runner,
  exact commands and twenty raw maps are retained in
  `Code/testing/genet-census-panics`; `checkpoint-a.md` records the table and
  evidence-qualified roots and options. These are starting results, not fixes.
- 2026-10-04: an in-process diagnostic changes Vano behavior to no-results and
  therefore cannot replace the isolated-worker receipts. A separate isolated
  diagnostic rebuild prints both worker backtraces and the unresolved colour's
  caller/value, and retains symbols. Temporary changes to the WPT harness and
  paint panic message are restored after the build. The interrupted first
  rebuild produced no new runner; its harness source was restored byte-for-byte
  before restart. No production fix has been applied.
- 2026-10-04: the completed lanes' targets recorded as retained on October 3
  are absent in the live inventory. Saved runners and evidence remain. The
  census target is being rebuilt at its approved stable path. No isolated
  Cargo home exists. The primary Genet checkout is clean; the line-breaking
  `text.rs` hash remains unchanged.
- 2026-10-04: new E1 parser lifecycle tests fail on both engines with only
  the author event recorded (0 passed, 2 failed). P7's populated-Document
  regression fails because the setter replaces its original children (0
  passed, 1 failed). Controls and source diff identities are retained in
  `control-e1.*`, `control-p7.*` and `starting-regression-source.patch`.
- 2026-10-04: both-engine P5 self-removal and P6 parser-callback-removal
  controls reach the stale `FrameRecord` assumption at
  `components/script-runtime-api/frames.rs:1344` (`expect("live frame")`).
  The P6 owning-crate fixture uses the current text-only frame loader seam; it
  does not claim an XML navigable because that seam carries no MIME or document
  type. The scoped fix keeps the current parser script running, lets queued
  child destruction precede ancestor load completion, releases the parent load
  wait, and suppresses load on the detached owner. The next open-stream pump
  observes the detached realm and drops the parser without running later source
  scripts. Reentrant removal from a child Window `load` handler also defers the
  ancestor load walk until after queued destruction. Iframe removal does not
  dispatch `pagehide` or `unload`; navigation lifecycle events remain separate.
- 2026-10-04: eight broader starting-directory runs completed with unchanged
  manifest/runner identities and zero execution/provenance failures. Each
  engine's maps contain 859 records, including non-testharness classifications;
  the testharness scope is 833. Maps and exact commands are retained under
  `directory-results/before/initial`. Four isolated workers were used for these
  directory runs; the twenty initial per-file reproductions used one.
- 2026-10-04: R1/R2 released the three GPT-6 Luna agents to implement their
  bounded owner slices. Root retains Cargo, runner builds and receipt ownership.
  The latest GC controls reproduce P1-P3 on both engines at the original
  policy-to-panic edge; the poison-intrinsics control also fails there on both.
  P4's first revised fixtures pass and remain unqualified as failing controls.
  No GC production change has been released while that fixture is refined.
- 2026-10-04: both owning-crate P5/P6 starting controls reproduce `live frame`
  on both engines. The first R2 production pass removes that panic but fails
  the assertions that the currently executing script finishes. Preserve the
  executing realm's relationships until queued destruction; reentrant child
  `load` handlers also need the same destruction-before-owner-load rule.
- 2026-10-04: E1's retained private per-realm Event constructor passes the
  initial and repeat-parse deletion regressions on both engines (4 tests).
  Parser, deferred frame and WPT-owned load dispatch use the retained hook.
  These focused results do not yet qualify the final WPT record or full gates.
  P7's Document setter fix passed its owning-crate regression (1 test) and
  was restored after the GC starting-control window. Final combined gates
  remain open.
- 2026-10-04: P4's release-and-collect fixture now fails on both engines at
  the original GC-policy edge (`control-gc-parser-release.log`). R1 production
  filters the engine inventory to live weak targets and uses only existing
  public wrappers with captured bookkeeping operations. A further both-engine
  control (`control-gc-array-descriptor.log`) catches inherited author getters
  during property descriptor conversion; the correction uses a null-prototype
  descriptor. The focused combined GC gate passes all 12 tests across Boa and
  Vano (`fixed-gc.log`); full identity and reclamation gates remain open.
- 2026-10-04: the final frame gate passes 10 of 12 tests. Independent review
  found that the nested fixture's serialized srcdoc contains a literal closing
  script tag inside a containing script string. Its earlier handler-flag
  failures (`control-frame-ancestor-load.log`, `fixed-frames-final.log`) are
  unqualified because the handler never registered. The corrected fixture
  escapes `<` in serialized JavaScript literals; its focused gate is pending.
  Pending relations retain parent/top traversal through the current script
  while the removed container loses its content navigable synchronously.
- 2026-10-04: the corrected nested fixture passes on both engines. The full
  runtime unit gate passes all 156 tests, both GC soaks and both bounded-cost
  checks included. An older initial-blank reentrant removal fixture still
  expects `unload`; update that expectation to document destruction's no-unload
  semantics and rerun the full runtime gate, including its remaining suites.
- 2026-10-04: the valid nested fixture reaches its handler/removal checks and
  fails ancestor completion on both engines with original `frames.rs`, while
  passing on final frames. `control-frame-ancestor-valid.*` records the source
  composition and exact restoration. The full initial frame suite passes 12/12.
- 2026-10-04: the first broad after round loses some passing subtests to worker
  timeouts. Review identifies a quadratic scan in the new GC policy: each
  group's last member searches later groups before clamping. The correction
  slices each group with captured intrinsics and scans only that group. The
  initial runner/maps are exploratory and retained; only this lane's running
  after helper and verified workers were stopped, with an interruption receipt.
  A fresh immutable `captured-split` round and final runtime gates remain required.
- 2026-10-04: the group-local scan still has high end-to-end singleton costs.
  Comparison with original `gcPolicy` shows steep scaling predates this lane.
  The final parser uses captured native split and private null-prototype
  separators with trusted delimiter coercions. A deliberately broken separator
  prototype fails the Symbol.split poisoning fixture on both engines; the final
  combined 12 GC regressions pass. At 8,000 retained singleton roots, captured
  split measures Boa 16.365s and Vano 13.715s, versus original-hook 15.554s and
  13.102s; the intermediate unbounded scan measured 102.909s and 57.535s.
  These are diagnostic samples, not a general performance gate. Source inputs
  are restored exactly and the temporary diagnostic fixture is removed before
  the new runner build. Full crate/WPT gates remain pending.
- 2026-10-04: the final immutable captured-split runner completes all twenty
  per-file reruns. P1-P7 report ordinary FAIL with subcounts on both engines,
  with no observed Rust panic; Boa P6 no longer hits the whole-worker timeout.
  E1 reaches ordinary FAIL 1/2; E4/E5 retain missing-feature evaluation errors.
  All four completed custom-element/DOM-node maps preserve every previously
  passing file and subtest. Focus and dynamic-markup maps remain pending.
- 2026-10-04: final-source gates pass for script-engine-api (0 tests), Boa
  (26), Vano (43), genet-scripted-dom (75), and script-runtime-api (672), with
  none failed or ignored. The complete runtime sweep reaches every suite,
  including the 12 frame and 12 initial-blank tests. The genet-scripted gate
  and remaining directory comparisons are running. Full identities/counts
  and scoped remaining failures are recorded in the lane receipt.

- 2026-10-04: all final gates complete: 936 tests across six crates, zero
  failed or ignored; API compile/doctests pass. All eight WPT maps cover the
  same 859 records per engine and manifest with GC on, isolated workers.
  Comparison finds zero lost file passes and zero lost passing subtests.
  All 33 movements are attributed in the receipt, with 3,092 formerly absent
  subtests now reported rather than mislabelled as repaired assertions.
  P1-P7 have no observed Rust panic on either engine; E1 reaches FAIL 1/2;
  SVG focus and ResizeObserver remain explicit feature forks.
- 2026-10-04: production committed as `876c7a2cba4` on `fix/census-panics`.
  The source patch matches the final frozen runner's build receipt; protected
  paint/text/harness sources and both ignored locks remain unchanged.
  Primary advanced independently to `f4ae933c873`, which rules for a separate
  Vano repin after census integration. This lane stops locally at its final
  checkpoint; the worktree, stable target and evidence remain for integration
  review. No isolated Cargo home was created.

## Rulings and remaining scope

**R1, 2026-10-04:** Mark selected "Use live wrappers and prevent author-code
calls (Recommended)" for P1-P4. Source and Boa's embedded JS
trace support custom-element re-entry during policy wrapper materialization;
the exact failing stale ID is not dynamically qualified. The implementation
must preserve canonical wrapper identity and detached-component ephemerons
while preventing constructor calls from collection bookkeeping. The original
options remain recorded in `Code/testing/genet-census-panics/checkpoint-a.md`.

**R2, 2026-10-04:** Mark selected "Prioritize queued destruction before
remaining child load work (Recommended)" for P5/P6. Finish the current script,
then process destruction before remaining child document work. Release the
parent load wait and suppress the detached owner's load event.

The frame loader currently accepts only text through `ScriptResourceLoader`
and parses its scripted frame source through the HTML `document.open/write/close`
path. XML navigable parsing and the related XML `document.open()` exception are
not part of R2; keep them as a separate feature gap rather than adding MIME or
document-type state to frame records.

SVG focus and ResizeObserver are missing features outside this defect lane.
The Vano repin is a separate integration step after the Genet fixes have been
verified against the frozen `8ad08412` baseline and integrated. Main records
that order in `f4ae933c873`; rerun affected tests in the separate repin commit
so movements are attributed to the correct change.

The [lane receipt](receipts/2026-10-04_genet_census_panics/receipt.md) distinguishes
qualified controls from exploratory passes, completed final gates and open
feature forks.

## Main integration, 2026-10-05

Mark authorized push, merge and continuation. Both lane commits were pushed,
then merged on main; merge commit `24bec750334dd4024738ea860610aef90a5f2c56`. The production
patch SHA-256 matches the frozen acceptance runner exactly. Main's independent
Vano timing ruling `f4ae933c873` is preserved. The next step is a separate Vano
repin with exact-pin runtime and census regressions; the original maps stay at
`8ad08412`. The existing worktree and target are temporarily reused for that
portable-pin gate, since the primary checkout has local Cargo patch overrides.
They can be retired after the new-pin receipt is recorded and published.

## Vano repin follow-up, 2026-10-05

**Status:** exact-pin validation in progress after census integration. Mark
accepted repinning only after this lane lands, then authorized push, merge and
continuation. The two native/wasm64 `nova_vm` manifest rows move from
`8ad0841255c2cbb679f7c704417d427b3cb3e961` to the published Vano main
`47f8d4f9d6fca884e3472e405872ce9556ff9db0`. No Vano source changes are made.

Phases and done-conditions:
1. Resolve only the new Vano git source in the reused isolated worktree's
   ignored lock; retain a package/version/edge comparison and prove the new
   source in locked metadata. Keep primary local overrides and lock untouched.
2. Test the new pin with `script-engine-nova`, complete `script-runtime-api`,
   `genet-scripted` with `scripted-nova`, and WPT `vano_reporting`. Freeze a new
   dev/netfetch runner and rerun the three Vano panic records plus the ten
   Genet census records on Vano, retaining maps and comparing existing passes.
3. Publish the separate manifest repin and its validation receipt on main;
   retire the now-integrated census worktree after its gate is recorded. Old
   census maps continue to describe `8ad08412`, without relabelling.

Findings: the primary checkout uses a gitignored local Vano `[patch]` and
would not establish the portable published pin. The existing clean census
worktree has no local overrides and is temporarily reused for that actual
configuration collision, with its stable target. Evidence is stored under
`Code/testing/genet/vano-repin/`; no Cargo home or extra worktree is created.
