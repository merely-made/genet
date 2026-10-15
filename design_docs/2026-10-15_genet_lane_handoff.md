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
  build. It does not forward to Netrender's feature of the same name, so
  consumers enable both. `compile_error!` off wasm.
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

## Open, roughly in order

1. **Coordinated Netrender adoption.** Genet pins Netrender `12b18970`. Mere
   bridges to Netrender main (`f4f9c0d87`: shadow repair, WebGL2 sparse, pen
   collapse, classic gate) with a `[patch]` marked "retire after coordinated
   Genet supplier adoption". Repin Genet to current Netrender main and run the
   render, render-host and Livery suites. Then `genet-render-host`'s feature can
   forward to `netrender/without-classic-vello`, and Mere can drop its bridge.
2. **Viewer size headroom (options, not tasks).** Mere's viewer artifacts sit at
   3,359,746 (WebGPU) and 3,322,097 (WebGL2) gzip against Mark's 3,460,300 cap,
   and fat-LTO layout has swung about 95 KB between builds. Genet levers if it
   creeps back: gate Genet's recent browser-capture and accessibility additions
   out of the read-only viewer cone (about 97 KB came with that pin), and align
   `genet-parley`/`genet-fontique` (with harfrust and skrifa) onto one
   `read-fonts` (0.39 and 0.41 both link, about 49 KB raw).
3. **Small cleanups.** `genet-render-host` uses the deprecated
   `AtomicU64::fetch_update` (4 warnings); buckram has 3 old clippy lints.

## Working notes

- Work directly on `main`; no PRs; no `Co-Authored-By` trailer. Main moves often
  (Codex Forms lane), so rebase right before pushing.
- ThinkPad: 14 GB RAM. Build with `nice`, `-j4`, one heavy build at a time; a
  memory-capped `systemd-run --user --scope -p MemoryMax=7G` works well.
- On wasm checks, `cargo check -p genet-render-host --target wasm32-unknown-unknown
  --features without-classic-vello` must stay clean, and the same with the
  feature on natively must fail.
