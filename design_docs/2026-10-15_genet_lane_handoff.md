# Genet lane handoff, 2026-10-15

**Status, 2026-10-15:** Genet `main` is the only live line on the remote. Five
older lanes are kept as `archive/*` tags. Two Codex worktrees on the ThinkPad are
still live (below). Everything here is pushed.

## Landed 2026-10-14/15

- `f0a91f7` **ValueEngine** in `script-engine-api`: a value-level contract beside
  `ScriptEngine` (Mere Scenograph rulings SE39–SE42). Mere now pins a Genet that
  includes it, so Mere's rhai `ValueEngine` and the Scenograph rule runner are
  unblocked. That work is Mere's.
- `e8b8606` **Buckram context-free core**: layout runs over a context-free
  arena, so Taffy is instantiated once per style type instead of once per
  `(Context, Source)` pair. That saves about 113 KB gzip in Mere's viewer.
- `ae3d9f5` **`without-classic-vello`** on `genet-render-host`: an opt-out for wasm
  viewer artifacts. Boot refuses, and every `RenderCore` method starts
  unreachable, so classic Vello, wgpu boot and composition drop out of the
  build. That initial commit required consumers to enable Netrender's matching
  feature too; coordinated adoption below adds forwarding. `compile_error!`
  off wasm.
- `45e1b81` **Linux test triage**: the nine Livery tests that failed on Linux
  were one real float-band bug (fixed) and eight platform font differences
  (made deterministic). `cargo test --no-fail-fast -p genet-livery` passes on
  Linux: 663 passed, 9 ignored. The forms plan's "six baseline failures" were
  recorded on the ThinkPad.
- `66c123c` (Forms/Codex lane) **owned browser frame capture**:
  `RenderCore::renderer()` returns a `RendererAccess` guard on wasm that holds the
  browser frame gate. Consumers take it per operation and never store it. Mere
  has adopted it.

## Live work not owned by this lane

- `worktrees/genet-forms-c-matched-control-20261013`: Codex's Forms Phase C
  **baseline**. Its 56 uncommitted files deliberately *remove* forms submission
  and FormData. Never merge it. Prune it only after Phase C closes in
  `2026-10-07_forms_value_validation_submission_plan.md`.
- `worktrees/genet-three-lanes` (`codex/three-lanes-upstream-20261010`): already
  contained in main. Codex may still be using it; prune with Phase C.

## Coordinated Netrender adoption, 2026-10-15

Genet selects Netrender `f4f9c0d87` (shadow repair, WebGL2 sparse, pen
collapse, classic gate) and its Vello provider `276b46648`. The render-host
opt-out forwards to `netrender/without-classic-vello`; the standalone renderer
smoke uses the same sources.

Fresh Windows gates pass: 746 native tests, zero failures, 13 existing ignores;
wasm opt-out compilation; the expected native opt-out rejection; and standalone
smoke compilation. [Source identities, locks and raw logs](receipts/2026-10-15_netrender_adoption/README.md)
retain the actual gate basis. Browser runtime and delivery size were not rerun.

Mere's receiving adoption and retirement of its four temporary bridge tables
are owned by `mere/design_docs/mere_docs/implementation_strategy/2026-10-13_browser_presentation_tenants_plan.md`,
section "Coordinated supplier adoption, 2026-10-15".

## O-PC accessibility and Forms lanes, 2026-10-15

Added by the O-PC main session after Mark paused new work to protect the
weekly budget. Mark's words: "pause, commit, push, merge when they finish, and
add 'em to the handoff."

**Landed on main:**

| Commits | What | Record |
|---|---|---|
| `33348b8dd49`..`85252e20bee` | The accessibility projection walks the flat tree, so shadow-tree content reaches AccessKit. `aria-labelledby`, `aria-describedby` and `<label for>` resolve per tree. Scroll ancestors climb via the flat-tree parent. | [Shadow DOM plan](2026-09-07_shadow_dom_plan.md), residual 11, rulings 11.1–11.3 |
| `297f58b9bbc`..`0ce8e65ff6f` | genet-wpt implements `test_driver.get_computed_label`. AccName goes from 0 to 352/405 subtests. | [Accessible names plan](2026-09-26_accessible_names_plan.md); `Code/testing/genet/wpt-ledger/2026-10-14_computed_label/` |
| `061e54c8aa2` | Button-type inputs are named from `value`, image inputs from `alt`. AccName reaches about 356/405. | Accessible names plan |
| `15174020fdc`, `eeb9774ada9` | Group, dialog, alertdialog, article and table stop taking names from content. The heading rule applies to roles and native elements; a table takes its caption; `abbr` is title-first. AccName reaches 365/405. genet-render passes 55/55 after rebasing onto the Netrender adoption. | Accessible names plan |
| `cb735d928a8`, `273431eff73` | **Forms Phase C paused.** Rulings F11–F14 and the ThinkPad's R19–R26 are recorded as history. Two Boa/Nova tests for the post-redirect subresource base are `#[ignore]`d: they compile (checked) but have never run. | [Forms plan](2026-10-07_forms_value_validation_submission_plan.md), "Paused 2026-10-15" |

**Forms Phase C, to resume.** This supersedes the "Forms C untouched" note
below. Mark ruled:
- qualify on O-PC under Rust 1.99.1, which is installed there, using `cargo +1.99.1`;
- reproduce, test and fix the Boa post-redirect URL bug, and run Nova too;
- pin the newest wasm-bindgen (0.2.129 is the newest release) and run the live Chrome check.

The likely cause, a reading and not verified: `LiveStylesheetSource::document_url`
in `genet-scripted/livery.rs` keeps the first page's URL. Remaining steps, in
order, are in the plan. "Before" needs a hand-resolved reversal of Phase C's 56
files, because a plain `git revert` conflicts. The worktree
`Code/worktrees/genet-forms-c` (branch `forms-phase-c`) and the clean control
worktree `genet-forms-c-control` remain on O-PC.

**Queued by Mark, not started:**
1. **Rust 1.99.1 family bump** ("Bump the 1.99.0 six"). Amend Mere's family
   toolchain policy as a dated ruling, then bump mere, isocosm, netrender,
   retinue, turnstone and woodshed. Genet stays on 1.97.1.
2. **Non-CSS WPT census refresh.** The last full run is from 2026-09-29
   (33,824/144,238). Run it on a calm machine.
3. **`get_computed_role`** in genet-wpt, beside `get_computed_label`. It
   unlocks `html-aam/` and `core-aam/`.
4. **Big-gap scoping**, after the census, for Editing/contenteditable, Popovers,
   Canvas 2D and IndexedDB. See the deferred-lanes scoping doc.

Also open in the remaining AccName families:
- spaces dropped between nested inline elements;
- `aria-label` on `<slot>` replacing its slotted content;
- `<dialog open>` missing from the projection, which belongs to a layout lane;
- `comp_name_from_content.html`, which crashes in layout (79 cases).

**O-PC housekeeping:**
- Empty folder `Code/worktrees/genet-input-value-names`, held open by a process.
- Worktree `genet-value-engine`, owner unknown, superseded by the landed
  ValueEngine. `genet-dep-currency` landed 2026-10-22; see the
  [dependency currency plan](2026-10-21_dependency_currency_plan.md).
- Genet's `Cargo.lock` is untracked, so a pin change on main needs a local
  re-resolve without `--locked`.

## Remaining options and cleanup

1. **Viewer size headroom (options, not tasks).** Mere's viewer artifacts sit at
   3,359,746 (WebGPU) and 3,322,097 (WebGL2) gzip against Mark's 3,460,300 cap,
   and fat-LTO layout has swung about 95 KB between builds. Genet levers if it
   creeps back: gate Genet's recent browser-capture and accessibility additions
   out of the read-only viewer cone (about 97 KB came with that pin), and align
   `genet-parley`/`genet-fontique` (with harfrust and skrifa) onto one
   `read-fonts` (0.39 and 0.41 both link, about 49 KB raw).
2. **Small cleanups.** `genet-render-host` uses the deprecated
   `AtomicU64::fetch_update` (4 warnings); buckram has 3 old clippy lints.

## Working notes

- Windows source reconciliation, 2026-10-15: the older local Forms B snapshot
  `b5813e22291` is superseded by qualified Phase B `a0d95ebd6e2` and its later
  fixes already on main. The temporary duplicate archive tag was removed at
  Mark's clarification; the primary checkout follows published main without
  replaying the older snapshot. Forms C and three-lanes were untouched.

- Work directly on `main`; no PRs; no `Co-Authored-By` trailer. Main moves often
  (Codex Forms lane), so rebase right before pushing.
- ThinkPad: 14 GB RAM. Build with `nice`, `-j4`, one heavy build at a time; a
  memory-capped `systemd-run --user --scope -p MemoryMax=7G` works well.
- On wasm checks, `cargo check -p genet-render-host --target wasm32-unknown-unknown
  --features without-classic-vello` must stay clean, and the same with the
  feature on natively must fail.
