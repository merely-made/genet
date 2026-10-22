# Coordinated Netrender adoption, 2026-10-15

Source basis: Genet `8649c0c3200` plus the three manifest changes hashed in
[identities.json](identities.json). Netrender is
`f4f9c0d87e6a6030b61075a45118055dfe6a12df`; the maintained Vello provider is
`276b46648c08f3d2062e70559afa999842e1873c`. Each of the four renderer crates
resolves once from the canonical `github.com/merely-made/netrender.git` URL.

The older Windows Forms B snapshot `b5813e22291` is superseded by qualified
Phase B `a0d95ebd6e2` and later fixes already published on main. It was not
replayed. The temporary duplicate archive tag was removed locally and remotely
at Mark's clarification; the primary checkout follows published main. Forms C
and three-lanes worktrees were not edited or merged.

## Gates

Rust 1.99.0, Windows MSVC. Commands run from `C:/Users/mark_/Code` with
explicit manifest paths, bypassing Genet's ignored machine-local supplier
patches. Build/cache output stays under `C:/t/cargo-targets/genet`, with
`CARGO_BUILD_BUILD_DIR=C:/t/cargo-targets/genet/build`, incremental compilation
off and dev/test debug information off. No isolated Cargo home was created.
The [frozen root lock](resolved-lock.toml) records all resolved package
identities; the root Cargo.lock is intentionally ignored by this repository.

- [Native suites](native-suites.log): 746 passed, zero failed, 13 ignored.
  [Per-package counts](native-counts.json): Livery 678/0/9, render 51/0/0,
  render-host 17/0/4 (passed/failed/ignored). This is fresh Windows evidence;
  it does not replace the earlier Linux source tuple and count.
- [Wasm opt-out](wasm-opt-out.log): `genet-render-host` compiles with
  `without-classic-vello` on `wasm32-unknown-unknown`. The host feature now
  forwards to Netrender's matching gate.
- [Native opt-out rejection](native-opt-out-rejection.log): fails with the
  intended Netrender wasm-only compile error, proving the forwarded feature
  reaches the supplier. This expected rejection is recorded as a passing gate.
- [Standalone smoke](standalone-smoke.log): native compilation passes against
  the same Netrender and Vello providers; runtime presentation was not run.

Each gate's `.result.json` retains its exact command, exit code and duration.
Warnings are retained in the logs; warning cleanup is outside this adoption.
The initial offline resolution failed because the new Git revision was absent
from Cargo's cache; [online resolution](resolution-online.log) fetched it.
No browser runtime, delivery-size or human/physical acceptance rerun is claimed.

The coordinated supplier/consumer execution plan lives in
`mere/design_docs/mere_docs/implementation_strategy/2026-10-13_browser_presentation_tenants_plan.md`,
section "Coordinated supplier adoption, 2026-10-15". Mere's own consumer
qualification and bridge retirement are recorded there after publication.
