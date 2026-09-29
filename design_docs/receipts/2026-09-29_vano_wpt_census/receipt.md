# Vano platform WPT census receipt

**Status: in progress, 2026-09-29.** Pristine reporting defect diagnosed;
worker fix verified; full directory census running with rebuilt runner.
This is a runner-specific measurement, not full-browser conformance.

- Genet base: `19c206873ab08ae227217892d9e74d0df18b349a`, plus the archived
  isolated-worker repair published in `0c4aa9f60b8` (details below).
- Git tree: `56a490f4343bc6cdcb57d23c7a972f083e186709`.
- Vano: `https://github.com/merely-made/vano`, revision
  `8ad0841255c2cbb679f7c704417d427b3cb3e961`, package `nova_vm`.
- Toolchain: rustc 1.97.1, `x86_64-pc-windows-msvc`.
- Manifest SHA-256:
  `d5ec5be9bf1a75ed00d7e7ab28afe8a694a55e11682ba74305874d70b18dd422`.
- Ignored lockfile copied into clean detached source and archived as
  [cargo-lock.txt](cargo-lock.txt), SHA-256:
  `db8ff719fbd101c4a909ce76441dad4aacc2b2f66af5b1184c65673bcc72a4e6`.

The full scope is derived from the archived September 6 result maps: **82
shards and 21,672 unique files**. The historical document's 86-shard statement
is a counting error. No overlap exists between the archived maps. The current
manifest is byte-identical to that baseline.

The shared source checkout has concurrent Livery work. Compilation therefore
uses a detached sparse worktree at `Code/worktrees/genet-vano-census`; WPT
inputs use the unchanged primary `tests/wpt/tests` directory. Both source
checkout cleanliness and primary WPT dirty state are checked. Build uses the
stable `C:/t/cargo-targets/genet` target, no isolated Cargo home, no local
repository dependency overrides. A named executable copy keeps concurrent
Cargo builds from replacing the census runner.

From `C:/Users/mark_/Code`:

```powershell
$env:CARGO_TARGET_DIR='C:\t\cargo-targets\genet'
cargo build --manifest-path worktrees/genet-vano-census/Cargo.toml --locked --offline -p genet-wpt --features netfetch -j 2
python testing/genet/wpt-ledger/2026-09-29_vano_platform_census/run_census.py
```

Each invocation specifies `testharness <subset> --engine nova --renderer
livery --jobs 4 --timeout 30 --drive-deadline 15 --write-expectations <map>`
and the absolute primary `--tests-root`. `nova` is the retained technical
backend name for Vano. GC is enabled. This uses the **dev** profile, unlike
the old release baseline. Current subprocess isolation, GC scheduling and
external timeouts also differ. A result delta cannot be assigned solely to
Vano or to implementation changes.

Raw evidence and execution scripts live under the existing ledger convention:
`Code/testing/genet/wpt-ledger/2026-09-29_vano_platform_census/`.
The [driver](run_census.py), [aggregator](aggregate.py), and exact
[82-shard scope](scope.json) are also archived here. To reproduce, place these
three files in that raw ledger directory; their paths deliberately refer to
the stable workspace locations above and archived September 6 sibling maps.
`progress.json` records completed checkpointed files/shards, external timeouts
and the current subset. Within-shard tests are not counted as complete until
the result map is written. `runs-fixed.jsonl` preserves exact invocation/exit
code; `provenance.json` pins the executable; `disk-fixed/` holds per-file results
and logs. The initial broken run is preserved separately.

No server is supplied: network-dependent failures and skipped server handlers
remain limitations of this disk measurement. Files skipped by the runner,
script evaluation errors, worker crashes, external timeouts, no-result files
and ordinary failing subtests are kept separately. No universal percentage is
reported. Final totals and cleanup are pending.

## Reporting prerequisite and bounded worker repair

The pristine binary SHA256 is
`360735eb86b6b5314961dbbe19025440262a7516d86c6012c2e4fbe42ede4e96`.
It produced 19 no-results and 2 skips for console (21 discovered variants,
versus 14 archived). Boa on the same runner produced 2 all-pass, 15 failing,
1 error, 1 no-results and 2 skips, with 10/51 subtests passing. GC-off Vano
still produced 19 no-results. The initial CSP shard was deliberately stopped;
its incomplete log is not a result map.

A trivial `test(() => assert_true(true))` also produced no Vano results.
Direct `__reportResult` worked. Catching the ordinary `test()` call exposed
`Error: realm operation refused: the context has no window proxy`. The
prepared-runtime snapshot replaces Rust host state while retaining harness
heap state; the evidence identifies broken realm/window-proxy restoration,
not a generic inability of Vano to execute JavaScript. Missing load listeners
was an initial hypothesis and is not asserted as the diagnosed cause.

The default isolated worker runs only one test, so it now uses the existing
fresh-runtime route instead of cloning a prepared template. The optional
`--in-process` template path remains unresolved and is not used here.
[Exact patch](isolated-worker-reporting.patch), SHA256:
`8a5cd5918483f3ddb515b105e6adc65d21206e7a071590f552a07a564cbfb9b2`.
Measurement source is **19c206 plus this patch**, not pristine 19c206.

The real CLI integration regression passed synchronous and microtask
sentinels (2/2) on Boa and Vano, with GC both enabled and disabled. Command:

```powershell
$env:GENET_WPT_TESTS_ROOT='C:\Users\mark_\Code\repos\genet\tests\wpt\tests'
cargo test --manifest-path worktrees/genet-vano-census/Cargo.toml --locked -p genet-wpt --features netfetch --test vano_reporting -j 2
```

One integration test, four engine/GC combinations, passed in 11.56 seconds.
Original broken evidence stays in `disk/` and `pinned-broken-*.json*`; the
restarted census writes `disk-fixed/` and `runs-fixed.jsonl`.

The 7 additional console records are all dedicated-worker variants now
admitted by discovery. Consequently, the new directory run covers a superset
of the old file set. Aggregation separates exact archived membership from
new variants and checks for missing archived records. Shared manifest identity
alone does not establish identical runner discovery.

The worker repair and integration sentinel are published in Genet
`0c4aa9f60b8`. The isolated patch's three code files are byte-equivalent under
Git diff to that revision; the commit adds documentation beyond the patch.
The counter and accessible-name lanes are not included in this measurement.

The standard (non-test) dev build completed successfully. Its frozen runner
SHA256 is `dcc1e8ee404b852979f51b9c917f73b6e2aee63f9ddef0e1d91bfa5300428abc`.
That exact executable passed both reporting sentinels on Vano (2/2), recorded
in `diagnostic_post_fix_standard_vano.log`. The full restarted run began at
2026-09-29 20:13 UTC. This binary uses the standard build graph rather than
the integration-test binary's dev-dependency feature unification.

## First checkpoint: console

The repaired runner completed console in 41.844 seconds: 21 current variants,
2 all-pass files, 15 files with failures, 1 external-timeout error, 1 no-results
file and 2 non-testharness skips. Reported subtests: 10 pass, 40 fail, 1 timeout
(10/51 pass). No worker crashed. This matches the aggregate from the same
pristine runner's Boa diagnostic; it is not a claim that their per-test
behaviour or other suites match.

The exact 14 archived console members produce 1 all-pass, 10 failing,
1 external-timeout error and 2 skips, with 5/28 subtests passing. Relative to
historical Boa, ten files remain failing, two remain skipped, one remains
passing and one formerly passing file hits the current external timeout.
The seven newly discovered dedicated-worker variants produce 1 all-pass,
5 failing and 1 no-results file, with 5/23 subtests passing. These additional
records are not included in the historical-membership comparison.

Only this checkpoint is complete at this receipt stage. The full 82-directory
run continues into content-security-policy (846 currently discovered records).
The census lane retains `Code/worktrees/genet-vano-census` and the named
runner copy while that process uses their source, working directory and
resources. The stable Genet target remains available for normal reuse.
