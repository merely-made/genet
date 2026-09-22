# Remove MallocSizeOf and the inherited global allocator

**Date:** 2026-09-22

**Status:** founded. Mark ruled 2026-09-22: full removal, media crates
included, the global allocator dropped rather than kept as an opt-in, and
the publishing bumps authorized.

**Owns:** the `MallocSizeOf` derive surface across genet, the two inherited
Servo crates that supply it (`components/malloc_size_of`, published as
`servo-malloc-size-of`; `components/allocator`, published as
`servo-allocator`), their unit-test member, and the breaking publishes of
the shared crates whose types carried the derive.

**Does not own:** the media crates' other Servo residue (the
[servo cone retirement plan](2026-09-07_servo_cone_retirement_plan.md) keeps
them by consumer); memory reporting of any kind, which genet has never had
and which this plan does not found.

## Why

`MallocSizeOf` is Servo's `about:memory` reporting trait. Genet inherited the
derives with the code and never built a reporter: outside the trait crate
itself there is no `MallocSizeOfOps`, `.size_of(` or `conditional_size_of`
call in genet, mere, netrender, woodshed, knot-editor, turnstone or isometry
(searched 2026-09-21; 258 hits inside the crate as the positive control).
netrender dropped the same derives on `paint_list_api` deliberately.

The cost is not the 1,267-line trait crate but its impl surface: it depends
on resvg, tokio, ipc-channel, urlpattern, content-security-policy, upstream
taffy and two dozen others purely to implement the trait for their types.
Computed from the resolved graph and cross-checked against `cargo tree`,
script-free Ortet holds 43 crates that exist only because of it, including a
second SVG and text-shaping stack (resvg, usvg, tiny-skia, rustybuzz, fontdb,
ttf-parser) and upstream taffy 0.10.1 beside the `genet-taffy` fork.

`components/allocator` reaches every build only through `malloc_size_of`,
and it is not a helper: it declares `#[global_allocator]`, installing
jemalloc on every non-Windows binary that links genet. A library choosing
the host's allocator is the opposite of embeddable; the host chooses.
Windows already resolved to the system allocator, so no receipt to date is
affected.

## Surface

Seven crates carry the derive; nothing else names the trait.

| crate | files | notes |
|---|---:|---|
| `components/shared/paint-types` | 10 | derives only; reaches every build through `engine-observables-api` |
| `components/xpath` | 1 | derives only |
| `components/media/audio` | 18 | derives, `#[ignore_malloc_size_of]` / `#[conditional_malloc_size_of]` field attributes, and `MallocSizeOfTrait` as a bound on `PortKind::ParamId` / `Listener` in `graph.rs` |
| `components/media/media-thread`, `player`, `streams`, `traits` | 5 | derives and field attributes |

Removed outright: `components/malloc_size_of`, `components/allocator`,
`tests/unit/malloc_size_of`, the workspace members and the
`malloc_size_of`, `malloc_size_of_derive`, `servo-allocator`,
`tikv-jemalloc-sys` / `tikv-jemallocator` workspace entries and the
`[profile.dev.package.tikv-jemalloc-sys]` override, each only if nothing else
in the workspace still names it.

## Publishing

`genet-paint-types` 0.1.0 pins `servo-malloc-size-of =0.2.0`, so removing
the derive is a breaking publish: `genet-paint-types` 0.2.0, then
`engine-observables-api` 0.2.0 (its only registry dependent), then
`genet-scripted-dom` 0.1.2, which pins observables at `=0.1.1`.
`serval-scripted-dom` 0.1.0 also depends on observables 0.1.1 and is on the
release list; it is not republished. Mere pins `genet-scripted-dom` by git
rev and takes the change at its next repin. The pending
`servo-malloc-size-of` breaking bump recorded by the
[cadency plan](2026-09-18_cadency_selector_engine_plan.md) is retired: the
crate no longer publishes from here. The published `servo-malloc-size-of`
and `servo-allocator` names on crates.io are the Servo project's.

## Done when

- `cargo tree -i` over the workspace finds none of `servo-malloc-size-of`,
  `servo-allocator`, `malloc_size_of_derive`, `tikv-jemallocator`.
- `git grep -i "malloc_size_of\|MallocSizeOf\|global_allocator"` over
  `*.rs` and `*.toml` returns nothing outside dated docs and receipts.
- `cargo tree -p ortet -e normal --prefix none` (default features) lists at
  most 358 distinct crates, against 401 at the base commit `3e8a6d3b684`.
- Suites at their recorded counts: cadency 10, livery 206, genet-livery 557,
  genet-documents 51 (`--features livery`), ortet 32 and 36
  (`--features scripted-nova --lib`), genet-wpt 68, plus the media, xpath and
  paint-types suites at whatever they report at the base commit.
- The native standards-compositing receipt reproduces 25/25 probes on Boa
  and Nova at digest `0xa440137ccc503f9c` through the committed runner.
- The three crates publish in order, each verifying against the registry.

## Findings

- 2026-09-22. `components/allocator` was not a helper: it declared
  `#[global_allocator]` and installed jemalloc on every non-Windows binary
  linking genet, reached only through `malloc_size_of`. Windows resolved to
  `std::alloc::System` already. Mark ruled it dropped, not kept as an opt-in:
  the host chooses its allocator. Any Linux or macOS measurement taken
  against a jemalloc build is not comparable to one taken after this commit.
- 2026-09-22. The 43-crate estimate was low. Script-free Ortet resolves
  **401 -> 353** distinct crates, 48 fewer; the graph walk that produced 43
  did not follow platform-conditional edges. Upstream `taffy` 0.10.1 is gone
  from the cone, leaving the `genet-taffy` fork as the only layout library.
- 2026-09-22. genet's `Cargo.lock` has been gitignored since 2026-06-18, so
  the lock delta is local. `--locked` runs still pass because the lock is
  rewritten on the first resolve.
- 2026-09-22. `cargo fmt --check` reports `components/script-runtime-api/frames.rs`
  at the base commit; this lane did not touch it and did not reformat it.

## Progress

- 2026-09-22. Founded.
