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
The [driver](run_census.py), [aggregator](aggregate.py), exact
[82-shard scope](scope.json), and compact
[historical membership](historical-membership.json.gz) and shared
[checkpoint validator](checkpoint.py) are archived here.
To reproduce, place these five files in that raw ledger directory; their
paths deliberately refer to the stable workspace locations above. The
membership archive contains exact historical keys/statuses and the SHA256 of
each original map, so historical-versus-new aggregation needs no bulky old
result maps. It contains 82 shards / 21,672 unique records and is 160,493 bytes,
SHA256 `dbe04effd6cb57ae5ee779adad7200258da56e18e963a2cf0bd7d639b708a0c6`.

The live process started with [run_census.started.py](run_census.started.py),
SHA256 `2cabe730f762f0138201f0e1b1d29ecd23bfbe007ced4632deafeae96a29a91f`,
which reads original sibling baseline maps. It was not restarted mid-shard.
The archived reproduction/resume driver reads the equivalent compact
membership and has SHA256
`ccac6b861bbeee132f17da8bfbd20176dfad4b43d881b85f3bbcd882de05738a`;
aggregator SHA256 is
`4849ea396b6662f627e06046040dd13459569748fd50ad476453a55673830c30`;
validator SHA256 is
`023ba73f7c8b2330b2ac5983309dac01fb2b08027c1d4e8e3b36f273e1d78b20`.
Console aggregation before/after this input substitution is identical as
parsed JSON. Both scripts parse, and the console checkpoint's runner SHA256
matches provenance and the still-frozen executable. CLI invocation and
test execution are unchanged. The reproduction/resume driver additionally
validates runner SHA, manifest SHA, engine, renderer, command, subset and exact
policy against provenance. Aggregation and resume fail on mismatches;
progress omits mismatched maps from qualified counts. In-memory deliberately
altered console records for each of those seven fields were rejected, while
the real console summary stayed identical. The live started driver retains
its earlier runner-only resume check; final aggregation uses the stricter
validator regardless of which driver produced the checkpoints.
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

The full 82-directory run remains active. The checkpoint below supersedes
this initial console-only progress count without replacing its detailed
comparison.
The census lane retains `Code/worktrees/genet-vano-census` and the named
runner copy while that process uses their source, working directory and
resources. The stable Genet target remains available for normal reuse.

## Qualified progress at 20:29 UTC

Five of 82 shards are complete and pass strict checkpoint validation:

| Directory | Files | Pass | Fail | Error | No results | Skip | External timeouts | Subtests pass / total |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| console | 21 | 2 | 15 | 1 | 1 | 2 | 1 | 10 / 51 |
| content-security-policy | 846 | 64 | 646 | 1 | 134 | 1 | 1 | 175 / 3748 |
| cookies | 82 | 1 | 63 | 0 | 18 | 0 | 0 | 3 / 968 |
| cors | 27 | 1 | 20 | 0 | 6 | 0 | 0 | 3 / 207 |
| custom-elements | 187 | 12 | 149 | 8 | 8 | 10 | 4 | 265 / 1456 |
| Checkpoint total | 1163 | 80 | 893 | 10 | 167 | 13 | 6 | 456 / 6430 |

External timeouts are a subset of errors. Errors contain 6 external timeouts
and 4 caught panics; no worker process crashed in these shards.
The 1163 records contain 1147 exact historical members (451/6398 subtests pass)
and 16 added variants (5/32 subtests pass). Subtest statuses across this
checkpoint are 456 pass, 2926 fail, 498 timeout, 2548 not-run and 2 precondition
failures; file errors, subtest timeouts and unreported tests are distinct
categories. DOM is the next active shard. The custom-elements subtest result
differs substantially from historical Boa; counts depend on runtime and
harness reporting, and the old denominator must not be reused for Vano.
**Final census totals and cleanup remain pending.**

## Custom-elements denominator diagnosis

Read-only comparison by exact file key confirms the same 187 members in both
maps, with no added or missing records. The denominator change is completely
accounted for: **3674 - 2406 - 40 + 228 = 1456**. Four externally timed-out
files previously reported 2406 subtests; two newly panicking files previously
reported 40. Twenty other files now report 228 additional subtests. This is
largely timeout/reporting censoring, not evidence by itself of broad feature
regression.

| Current external timeout | Historical subtests | Historical passes |
|---|---:|---:|
| `registries/valid-custom-element-names.html` | 1975 | 1859 |
| `builtin-coverage.html` | 327 | 72 |
| `ElementInternals-role.html` | 68 | 0 |
| `registries/adoption.window.html` | 36 | 0 |

All paths in this section are under `custom-elements/`. Each timeout is an
external kill at 30 seconds and contributes no current reported subtests.
The four caught panics are distinct from process crashes: `Document-createElement.html`
and `Document-createElement-customized-builtins.html` report
`registered realm GC policy: Engine("[object Object]")`; `upgrading/Node-cloneNode.html`
and `upgrading/upgrading-parser-created-element.html` report a maximum-call-stack
`RangeError` at that same GC-policy boundary. The latter two already had no
historical subtest results, so they do not decrease the denominator.

Current reported subtest statuses are 265 pass, 1186 fail, 2 precondition-failed,
2 not-run and 1 timeout. A reported fail may be an assertion failure or an
exception caught by testharness; these result maps do not retain enough detail
to label every failure an assertion. Ten skips are 6 non-testharness files and
4 XHTML files.

Eight files report no subtests. Seven previously had Boa `evaluation-threw`:
three form-associated ElementInternals accessibility/submit cases and four
scoped-registry upgrade/createElement/createElementNS/importNode cases.
`pseudo-class-defined-customized-builtins.html` already reported no subtests.
The current no-results records do not expose an exception, so their specific
script/setup versus completion/reporting causes remain unresolved. They are
not counted as either passing or ordinary assertion failures. No implementation
change or diagnostic rerun was used for this keyed comparison.

### Separate dominant-timeout diagnostic

One additional diagnostic raised only the external timeout to 180 seconds
for the exact `registries/valid-custom-element-names.html` record, using the
same frozen binary, GC enabled and 15-second drive deadline. It completed
normally with **1861/1975 pass and 114 fail**, compared with historical Boa's
1859/1975. There was no error, no-results outcome or reported subtest timeout.
This establishes that the dominant missing denominator in the fixed 30-second
census was timeout censoring. It does not replace that census record, establish
other timeout causes, or attribute the two-pass difference solely to Vano.

From the isolated census worktree:

```powershell
& C:/t/cargo-targets/genet/debug/genet-wpt-vano-census.exe testharness custom-elements/registries/valid-custom-element-names.html --tests-root C:/Users/mark_/Code/repos/genet/tests/wpt/tests --engine nova --renderer livery --jobs 1 --timeout 180 --drive-deadline 15 --write-expectations C:/Users/mark_/Code/testing/genet/wpt-ledger/2026-09-29_vano_platform_census/diagnostic_valid_names_180s_vano.json
```

Exit 0. Separate raw JSON SHA256:
`adeecb73e1031dbbcfd27e31ec96e4973e85d01fcc3bcead90e75013dcad038d`.
The [compressed exact JSON](diagnostic_valid_names_180s_vano.json.gz) and
[output log](diagnostic_valid_names_180s_vano.log) are archived with this receipt.
The hash above applies to the decompressed JSON bytes. Both files remain
outside `disk-fixed/`; the full census remains unchanged at a 30-second
external cap.
