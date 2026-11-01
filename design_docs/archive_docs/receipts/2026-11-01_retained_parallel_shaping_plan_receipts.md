# Retained and parallel shaping: qualification receipts

Date: 2026-11-01. This is Task 1's original-pipeline control, before retained shaping or parallel shaping. Both captures ran the same deterministic eight-paragraph article fixture, default features, one worker, debug profile, 800 × 600 then 640 × 600 viewport. The four stages are initial, unchanged, width change and one-paragraph edit. Each measurement is one sample; it is not evidence of a speedup. Timings are wall-clock nanoseconds and are sensitive to environment and instrumentation overhead.

| Stage | Pristine frame ns | Instrumented frame ns | Output FNV-1a 64 |
| --- | ---: | ---: | --- |
| Initial | 36,325,112 | 31,029,891 | `3b9e0f3d947c1734` |
| Unchanged | 20,188 | 13,285 | `3b9e0f3d947c1734` |
| Width change | 16,467,045 | 12,002,314 | `f77a30d8d5fbaca6` |
| One-paragraph edit | 23,492,016 | 18,329,715 | `d0acf5024767c211` |

The output digest hashes a postcard-serialized `PaintEnvelope`; the two controls matched in all four stages. The hash is an identity check, not a cryptographic integrity proof. Instrumented cumulative format requests / all layout builds / metric layout builds / formatting break operations were `8/12/4/8`, `8/12/4/8`, `16/20/4/16`, and `32/36/4/32`. Initial collection / shaping wall / placement were 152,878 / 6,334,883 / 16,079,366 ns. Pristine counters were unavailable. The unchanged result uses the existing document frame cache; Task 1 did not implement a retained shaping cache or parallel workers.

Pristine source commit: `7b86d24fd42199afbc5a351c116fdbef23182694`. Instrumented source commit: `0c10ad46331ffb4ef465faee7246021c856c2545`. The initial drivers inferred the Git revision from their runtime directory and hardcoded `features=default`; those labels describe these particular captures but were unsuitable for reusing the saved binary from another checkout. The fix requires a caller-supplied build source revision, reports compiled features, and records an executable fingerprint. The earlier results are preserved here without rewriting their provenance.

Both builds used the same locally resolved, ignored Cargo.lock, SHA-256 `3bd1d748fc464919f5007ffc8c1958f471a391ee25b1a04467abbbc18077b145`. It is a local resolved closure, not an upstream-published lock. The old main-checkout lock had SHA-256 `75a0a55e3241f794661e2c5d390005e1d56229700d6cbd54f9e05b7034b50381`; current manifests required reconciliation. In particular, current netrender main `11e208b84a981993df8077bb6025f8b3adbe2291` supplied the image payload API expected by Genet. The font was `/usr/share/fonts/google-noto-vf/NotoSans[wght].ttf`, SHA-256 `fb53504de62e767a54695416123d68d67f71c9b69722175184ad56e7dd13547b`; FNV-1a 64 `fe09e63b443e1e39`.

The original harnesses and binaries were preserved locally for later comparison:

| Artifact under `/home/markik/Code/testing/genet/controls/` | SHA-256 |
| --- | --- |
| `task1-pristine-source.rs` | `088ef0a727fe1534e1491bdadedf7cb280af22ed1128ee7f7bae9fb932f44360` |
| `task1-pristine-measure_text_shaping` | `f5fd39535f58c37166e66f488f589f132f9f1133988a960422029b8895360ba3` |
| `task1-instrumented-source.rs` | `14a42ad4bb57083e1160ee510537f1281b69cf1bdaee33581835c369351de8eb` |
| `task1-instrumented-measure_text_shaping` | `4f1867514e46b0fb5ba1320ee32db2399a3830de7f0d77c37cb505150a1f5f11` |

The pristine harness differs from the instrumented harness only by removing the new stats/options APIs and labeling them unavailable. The pristine detached worktree is `/home/markik/Code/testing/genet/shaping-pristine-7b86d24`; the instrumented worktree is `/home/markik/Code/worktrees/genet-shaping-plan-20261101`. Builds and controls were serialized with `/home/markik/Code/testing/machine-heavy.lock` and run through `systemd-run --user --scope -p MemoryMax=3G -p MemorySwapMax=0 -p CPUQuota=150%`, nice 10, one Cargo job, one test thread, and `CARGO_TARGET_DIR=/home/markik/Code/testing/genet/shaping-target`. The instrumented control used:

```sh
/home/markik/Code/testing/genet/controls/task1-instrumented-measure_text_shaping --font '/usr/share/fonts/google-noto-vf/NotoSans[wght].ttf' --case article --paragraphs 8 --output shaping-task1-committed-control.txt
```

The pristine control used the same fixture arguments and `cargo --config 'build.build-dir="/home/markik/Code/worktrees/genet-shaping-plan-20261101/target/build"' run -p genet-livery --example measure_text_shaping --locked --offline --`, under the same wrapper. Complete original rows and logs are in `.superpowers/sdd/2026-11-01_retained_parallel_shaping_plan/` in the instrumented worktree; the table above retains the relevant results if that scratch area is unavailable.

Task 1 focused tests passed 2/2; the full Livery library suite passed 373 with two ignored. Existing deprecation warnings were present. Runtime/browser/platform qualification is outside these measurements.

For subsequent measurements, supply the full commit used to build the corrected driver, for example:

```sh
measure_text_shaping --source-revision <full-build-source-commit> --font <font-path> --case article --paragraphs 8 --output control.txt
```

The source revision is a caller-supplied claim; retain the source checkout and lock used for the build. Each row also records the executable's FNV-1a fingerprint and its compiled `image-decode` or `none` feature label. An executable hash identifies the build for comparison but is not cryptographic verification of the source claim.

## 2026-11-01: Task 2A construction/placement split

Source `0bb48169e0a`, unchanged resolved lock SHA256 `3bd1d748fc464919f5007ffc8c1958f471a391ee25b1a04467abbbc18077b145`. All commands used the same shared-lock systemd wrapper, 3 GiB/zero swap/150% CPU, one Cargo job and one test thread. `cargo test -p genet-livery --lib --locked --offline -- --test-threads=1`: 377 passed, 0 failed, 2 ignored. Focused `inline_measure_policy_and_width_match_exact_requests`: 1 passed; `split_shape_`: 3 passed. Existing deprecation warnings remain. Logs `/home/markik/Code/testing/genet/shaping-task2a-full-final.log` and `shaping-task2a-final-*.log`.

Temporary simultaneous original/split layout fingerprints matched for real Arabic (`0979d502b446285a`), Lato ligature (`84323ece945f5f92`), Ahem inline atom (`f745350b94d196ff`), mixed spans (`7a025ec7bbb052dc`), forced break (`95aad734e2c49ee2`), tabs (`60aa7fabd6d62eb2`) and UTF-8 source range (`d94cf3626ce94a8d`). Fingerprints included glyph/font/cluster/line/slot values. Old production builder and temporary oracle were removed after parity. Independent existing fixture assertions remain. The removed oracle code is not preserved as a second engine; this is bounded recorded evidence, not exhaustive equivalence proof. New input tests were added after production code, so there was no clean test-first cycle for that boundary.

Terra task review approved spec and quality, with no critical/important finding. Geometry-template preparation remains Task 2B; caching and parallelism are still unimplemented.

## 2026-11-01: Task 2B formatter comparison

The owner-side event tape was committed as `9d512b46f30`. A temporary matching diagnostic was applied to the original Task 1 instrumented source `0c10ad46331ffb4ef465faee7246021c856c2545` and the Task 2B source. It recorded each actual `format_inline_group` text, span selectability, current source and owner BoxIds, byte ranges, shaped runs and clusters, glyph IDs and positions, font-instance keys, colors, line fragments and baselines. A common document driver serialized complete postcard `PaintEnvelope` output. These are bounded comparisons of the formatter and resulting paint, not browser or general CSS conformance tests.

Four fixtures were each run at viewport widths 120 and 220: normal styled Latin text and percentage edges; Arabic RTL with an explicitly registered Noto Naskh font; decimal inside marker, generated before/after text and a forced `<br>`; and an inline-block atomic box with following text and a percentage edge. The generated fixture's collected text was `1. PRE alpha\nbeta gamma END`; the bidi trace includes RTL clusters. In all eight cases, the ordered formatter trace and serialized paint bytes matched exactly between the Task 1 and Task 2B builds. SHA-256 below identifies the matching raw formatter trace and paint output for each pair; counts represent formatter calls per fixture/width.

| Fixture | Width | Calls | Formatter SHA-256 | Paint SHA-256 |
| --- | ---: | ---: | --- | --- |
| normal | 120 | 1 | `ba8d78212a510724c270e0f4abc80071585002e37e7b5dc9d1e82a97d8ff4121` | `18e2595929d578d93dc246cad70d3852dd19a7c953df7592914263f92b69fcfc` |
| normal | 220 | 1 | `af015ffcb38f007c0ff894ee47340ba64c623ebb80501b55e1e377ddbd9974b2` | `348646d9e2fd790a47641407c70505db9ca6b7ee68c522d565740daa768e6beb` |
| bidi | 120 | 1 | `7d86ee291b1754aa7f6f055a008f45830097e2010fa5222559478513545a30f1` | `d66814fd0787f7fc20f2dc9272c780d7992792af5b42ed6e972a0861e6a4a207` |
| bidi | 220 | 1 | `6fab2c047606c00f5b25ad491ea2c50a2bc88c1f1eccbebf666f30d6ef718acb` | `0893bf0363a338b45cc6cd32f9ba9dd0d1fe15d061ba2ce4c2ca33dbdb5b846a` |
| generated-break | 120 | 2 | `d5156f258260b58e34f2d9ef44a3576b7c82e8896d580c581915ef90449d24d8` | `f77cb0ba039cbdc4b91fb53a7459d8310b123b1ce87fea21286c27a9d6d9c22a` |
| generated-break | 220 | 2 | `d0f9d2d3a5faa23c68f4cfc488f3513e0fd98f021d9f52776eecbdb2ad611db3` | `212a803245a9e38a106ad73ce81b13f9807bdfbd580cb5c96425d6d93bbb448c` |
| atomic | 120 | 4 | `0cda82ee1066fd8a3319abb3ed665889d07deb71a506c67321349aada1dc0b8f` | `2c8a96c69078ea6bc529030e96be7c72f97a5ea6ad750483de03cdbdf1cc1943` |
| atomic | 220 | 4 | `1d5d3960efe8faba298c22f1a5549cba68591cf16678ef92c354f4ae35e58d0e` | `d6c5b943e6515c6b826c16d3fac6f5ef9a634c5adb0cc729622fab5616741bab` |

The common fixture source, diagnostic script, applied baseline/current patches, comparison script, binaries and per-case serialized paint are retained under `/home/markik/Code/testing/genet/controls/` as `task-2b-format-oracle.rs`, `task-2b-instrument.py`, `task-2b-baseline-format-instrument.patch`, `task-2b-current-format-and-offset.patch`, `task-2b-compare.py`, `task2b-task1-oracle`, `task2b-current-oracle`, `task2b-baseline-paint/` and `task2b-current-paint/`. Their fixture-source, instrumentation-script and comparator SHA-256s are `b18c9c920decfc36e822141c6c458e50b1611ae54eefaafe1d511d1c0834b667`, `05434bdbd64db132b4c36f01556803ba891e4d57af5a37cf1aa658c4492362b0`, and `a1c10e801012a36b27d1779d92d3c97a7cb0da9cffe8c0e054d2184e8514a863`. Full logs and comparison are `/home/markik/Code/testing/genet/shaping-task2b-fix-{baseline,current,comparison}.log`. Font SHA-256s: Noto Sans `fb53504de62e767a54695416123d68d67f71c9b69722175184ad56e7dd13547b`, Noto Naskh Arabic `a0b1ab8d9808470190e8e72dd0a3a3d0a7f16e4029f5f9c80fc56fbcb3b03c1f`.

The diagnostic was removed from the production source. The retained geometry matrix now places UTF-8 `é` before and `ok` between events, and checks inner atom byte index 2 with outer owner and subsequent positioned/atomic byte index 4. Focused test: 1 passed, 0 failed, 379 filtered. The prior Task 2B full suite passed 378, 2 ignored. Both diagnostic builds and focused test used the shared-lock systemd wrapper and caps above with `--locked --offline`; the same frozen lock SHA-256 `3bd1d748fc464919f5007ffc8c1958f471a391ee25b1a04467abbbc18077b145` was used. Existing dependency deprecation warnings remain.

## 2026-11-01: Task 3 exact-input retention

Production source `6dcc065929d`, reviewed test-strengthening commit `f8e0e452e55`. Frozen lock and standard 3 GiB/zero-swap/150% CPU wrapper unchanged. Public integration tests passed 6; text tests 46 passed/1 ignored and full library 381 passed/2 ignored before the test-only fix. Existing dependency warnings remain. Tests prove post-mutation hit deltas with current selection/hit owners, changed resize geometry matching uncached output, and same-key font replacement/removal/generic/face-feature output matching fresh documents. Ahem and Lato-Medium-Liga fixtures are hashed in the preserved task report.

Small debug article control: 8 paragraphs, three iterations, same pinned Noto Sans font and four frame stages. All modes matched paint digests. Median frame ms, original / uncached / retained: initial 29.632 / 30.409 / 30.193; width 11.993 / 12.371 / 6.610; one-paragraph edit 18.132 / 18.762 / 8.005. Retained resize added eight hits and zero inline builds; edit added one new key. This is bounded saved-work evidence, not release performance acceptance. Retention bounds entries and source-key text bytes, not total allocations; LRU duplicate keys and cloning costs remain to measure.

Raw controls: `/home/markik/Code/testing/genet/shaping-task3-{original,uncached,cached}.txt`. Current diagnostic binary SHA256 `18647b44f4bc540e9743e2454a07600f53f97e5b860098dea5402e9e2ccea7e2`; pristine binary identity is recorded above. Current was built from dirty source later committed as `6dcc065929d`; its caller-supplied source label names the base. Original binary's runtime Git label is also not its build provenance. Binary hashes and preserved sources identify these controls. Full commands, artifact/font hashes and limitations are retained in `.superpowers/sdd/2026-11-01_retained_parallel_shaping_plan/task-3-report.md`.

## 2026-11-01: Task 4 traversal and serial batch integration

Reviewed checkpoints: API `ce2001cbbd7`, atomic integration `b37f103ffc5`, main integration and geometry refusal `f396679f197`. Ready work is collected before consumers; width queries retain current preparation and active results even with caching disabled. Stale batches fail atomically; missing in-flow atom geometry rejects the layout transaction. Percentage/math edges conservatively take the keyed late path.

Buckram supporting context-update API clears caches once per update and passed 275 library tests. Atomic controls: focused7, layout136/1ignored, retained26, text51/1ignored. Final main production fix: focused13, layout142/1ignored, retained27; later test-only out-of-flow and selected-root assertions passed separately. Existing upstream warnings remain. Main CountedDom fixture adds zero reads across width queries; two identical groups produce one unique ready job before any format request. Full geometry matrix compares all generated-box rectangles and emitted text/paint commands. Fresh ready/late counts: float1/0, percentage0/1, inline-table3/0, image1/0, positioned3/0, K7 5/0. Actual K7 basis changed320 to140 and atomic width104.944016 to70. Image geometry is checked; the convenience emitter lacks image-resource pixels.

Retained geometry edits compare complete paint with fresh zero-retention output. Percentage edges exercise selected-root replacement; inline-block/float/table fixtures exercise existing full-layout fallback. No claim of every retained path staying incremental. Error propagation is exercised by missing-geometry transaction refusal; a dedicated injected late ShapeBatchError through a consumer remains a minor test gap.

Clean-source article driver at `f396679f197`, same frozen lock, pinned Noto Sans bytes and standard resource wrapper: 8 unique ready jobs,0 late misses,1 batch,8 initial format requests. Four digests match original controls: `3b9e0f3d947c1734`, `3b9e0f3d947c1734`, `f77a30d8d5fbaca6`, `d0acf5024767c211`. Preserved binary SHA256 `53b199d1274142b912021084bfbc53b7e4af554bc65f15b12689c9c00a2f1478`; raw rows `/home/markik/Code/testing/genet/shaping-task4c-article.txt`. This is correctness and readiness evidence, not a speed claim. Exact commands, logs and scoped limitations are in the preserved task-4a/4b/4c reports under the plan workspace.

## 2026-11-01: Task 5 optional native parallel shaping

Implementation `c7105be1b7c`, reviewed execution-evidence fix `a4250372d71`. A caller-supplied pool receives only owned inputs, with scratch bounded by admitted active slots. Results join before publication; an injected panic leaves new cache results and cumulative counters unpublished and clears scratch. Defaults and wasm remain serial.

Feature library suite398 passed/2ignored; final public feature10, private2, default public6, default stats1. No-default public6 and wasm32 no-default compilation passed before the observability-only fix. Wasm dependency query with parallel-text also showed no Rayon target dependency. A synchronization barrier proves two worker tasks overlap; public comparisons assert positive parallel job/batch deltas per font generation and exact scratch slots. Eight repeated document lifecycles populate scratch, detach or drop, and release pool references. Font rebuild, swap and detach preserve the intended configuration and output.

Paint commands, selected font bytes and caret/source offsets match serial at1/2 workers, including combining/emoji, nested alignment, quirks, face features and fallback fixtures. Selected byte FNV identities included Ahem `a1f8fac5beca8448`, Lato `0171fc2f3b96937c`, and system fallback `f6d85ac48d0884e6`, `fe09e63b443e1e39`, `d0cb82104c944127`; equality is not a claim that every glyph/script is supported. Existing ICU4X Chinese/Japanese segmentation-model messages and dependency deprecations remain in raw logs.

Original ignored lock preserved as pre-parallel-Cargo.lock in the plan workspace; new parallel-Cargo.lock SHA256 `b63d420e7861eb9239dda7622c2eaa9a7aa395ab3b913ac1fa5ae934b50f2d40`. Only genet-livery's dependency list gains existing rayon; package versions/sources are unchanged. All tests used the established150% CPU/3GiB/zero-swap wrapper and shared lock. Rust race detector is UNTESTED because no supported installed setup was found; no toolchains installed. Wasm evidence is compilation, not browser execution. Performance is not yet qualified. Exact commands and linked logs: preserved task-5-report.md.

## Task 6: bounded qualification, 2026-11-01

Measurements used `30901973a284c265044d649c5d3ec95095e3d7c4`, with original `7b86d24fd42199afbc5a351c116fdbef23182694`. Current lock SHA-256 `b63d420e7861eb9239dda7622c2eaa9a7aa395ab3b913ac1fa5ae934b50f2d40`; original `3bd1d748fc464919f5007ffc8c1958f471a391ee25b1a04467abbbc18077b145`. The timers-off primary and fresh repeat used 5 warmups and 30 observations per non-churn case, a 200% CPU quota, 3 GiB and zero cgroup swap, and matched paint and selected font fingerprints. Source, raw records, scripts, binary/font hashes, exact protocols and historical failures are in `.superpowers/sdd/2026-11-01_retained_parallel_shaping_plan/task-6a-report.md` and `/home/markik/Code/testing/genet/task6-all-off-20261101/` and `task6-off-repeat-20261101/`.

- **Capacity reuse:** all seven measured capacity-cache resize fixtures had zero builds and late misses; 2,048 groups produced 2,048 hits. Capacity edits built one changed key. The default 512-entry cache rebuilt 1,536 keys for that larger working set. Observed post-frame entry and key-text gauges stayed within configured limits, which do not bound total heap.
- **Repeated reflow:** 2,048-group capacity retained-serial layout median was 215.67/214.67 ms versus original 234.70/232.56 ms in primary/repeat; all 30 paired layout samples favored retention in each cohort.
- **Pooled work:** 2,048-group cold pooled layout improved versus new retained serial by paired medians 27.412/27.850/27.210 ms in three cohorts, with measured joined jobs. The new uncached serial control still uses prepared templates and batch deduplication; it is not the original pipeline. Single-long edits had zero pooled jobs.
- **Cold regression:** paired retained-serial minus original layout medians were +46.556/+41.700/+42.310 ms; pooled minus original +19.019/+12.710/+15.230 ms. This is not a net cold win over original. Frame results were noisier.
- **Memory:** corrected 129 KB long-edit process peak was 194.5 MiB retained serial versus 86.4 MiB original with default limits. Peaks include font collection and serialization; they do not establish a leak.
- **Small/edit latency:** differences exceeding 5% fell within the independent centered serial-repeat empirical spread. That spread is a per-frame variability proxy, not a p95 confidence interval; the control had two resident processes versus four in primary, and 30 samples leave tail uncertainty.

At `30901973a28`, five matched WPT cases executed: two passed, three failed locally in both original and current; one script-dependent case was unsupported and skipped in both. There were no status transitions in these six cases. The selected wgpu adapter was not logged. `genet-documents` with Livery passed 101 tests and Livery wasm compile check passed at that source; neither proves wasm runtime. The default and parallel package gates initially failed the same old resize assertion after 688 and 690 passes respectively, with eight ignored in each; Cargo stopped at paint and later suites did not run. Failed logs and exact commands: `/home/markik/Code/testing/genet/shaping-current-qualification-20261101/`; original WPT controls `/home/markik/Code/testing/genet/shaping-baseline-wpt-20261101/`. Native sanitizer, headed host/runtime and GPU performance remain untested. Rust 1.97.1 was not freshly built.

The behavioral test repair was tested at `9c16ebda1b2`: focused default and parallel each passed 1; full package default passed 773 with 8 ignored, parallel passed 779 with 8 ignored, including later targets and doc tests. Commands used the shared lock, 3 GiB memory, zero cgroup swap, 150% CPU, nice 10 and one Cargo job/test thread; logs are in `/home/markik/Code/testing/genet/shaping-6b-repair-20261101/`. Subsequent `2cc8fc98f7e` is rustfmt-only: the branch's export and example comma hunks plus seven files byte-identical to original and the remaining pre-existing ordering drift in `src/lib.rs`. `cargo fmt -p genet-livery -- --check` passes after this mechanical cleanup. No full rebuild was repeated for formatting. Final documentation is separate from both tested source and formatting checkpoints.

## Final review fix: cumulative document shaping observations

On top of `e93b74729f7`, the font-ledger rebuild now constructs a fresh font context while preserving cumulative shaping counters, timings, legacy `shape_count`, options and the caller pool. Cache and worker-scratch gauges describe the new empty context. A document-level regression exercises actual shaping, hits and timings before admitting Ahem font data and rebuilding the ledger, verifies all cumulative fields persist and gauges clear, then verifies further work increments counters. It does not establish that Ahem was the selected shaping face. The parallel case populates both worker slots before rebuilding and verifies the pool survives and repopulates.

At this final source delta, default and parallel public `text_shaping` suites passed 7 and 11 tests respectively; the document font-ledger control passed once in each configuration. The regression failed as expected before the fix. These focused runs do not replace the 773/779 full-package evidence at `9c16ebda1b2`; those full suites were not rerun after this fix. Formatting and diff checks passed. Commands and log hashes are in the final-fix report in the plan workspace. Scoped re-review at `b65ac762a3c` found the cumulative-statistics finding addressed and no new Critical or Important breakage. Deferred coverage and key-copying concerns remain nonblocking; this is not a universal performance endorsement. Cold and RSS regressions, empirical-band uncertainty, WPT failures and skip, unverified adapter, and runtime, sanitizer and fresh MSRV limits above remain unchanged.
