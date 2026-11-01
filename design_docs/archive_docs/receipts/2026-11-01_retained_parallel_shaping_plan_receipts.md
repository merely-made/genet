# Retained and parallel shaping: original pipeline controls

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
