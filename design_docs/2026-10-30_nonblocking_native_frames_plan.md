# Nonblocking native frame qualification

**Status (2026-10-30): landed; bounded native Metal qualification complete.** Native hosts need checked GPU frames
without making their UI thread wait for every surface's completion. Genet owns
admission, quality attribution and device-loss cancellation; hosts retain their
window scheduling and presentation policy.

## Frame admission

The existing checked raster operation qualifies its current shader counters
before returning a texture. Returning that texture earlier as an ordinary
successful checked result would weaken the contract. A native frame batch
instead provides an explicit staging interval on one submitting thread. Explicit batch
rasters join that interval's error scopes and loss admission. Staging textures
are not eligible for publication until the sealed batch is accepted.

One Core owns one batch at a time. Independent windows sharing that Core must
retry admission when it is busy. A legacy checked draw outside a batch retains
its existing synchronous semantics, including typed quality refusal. Such a
draw cannot interleave with an active or pending batch. Existing
raw renderer APIs remain unqualified escape hatches and must not be used to
interleave unowned rendering with an admitted batch.

**Done conditions:** native submission does not wait for quality completion;
the quality FIFO still contains the current submitted work when staging is
sealed; overlapping checked draws refuse; and no caller can collect a staged
value through the pending API before acceptance.

## Completion and lifetime

One demand-started completion worker per shared Core drives bounded device
completion and the provider's typed curve-quality ledger. UI collection reads
a channel and never enters a device poll. Validation, internal and out-of-memory scope results all
participate in acceptance. Provider errors are retained while their outstanding
counter results drain. Only complete scopes and a zero counter/byte backlog can
accept the batch. Completion keeps the original five-second deadline, and an
actual device-loss notice takes priority over a timeout or provider error.

The pending batch holds no strong Core or application borrow. Device loss
cancels admission eagerly so an unpolled frame cannot hold the shared recovery
barrier indefinitely. A canceled batch drops its staged values and leaves its
qualification to drain on the existing shared completion worker. No worker
is created per frame. No frame-token destructor waits for the GPU; explicit Core
shutdown joins the worker. A timeout or otherwise unprovable outcome
refuses attribution rather than making the next batch inherit stale counters.

**Done conditions:** actual shader refusal never publishes a value; a later
valid frame recovers on the same healthy device; loss while pending or after
acceptance refuses publication; disposal is bounded; and cancellation cannot
qualify a later batch with the wrong counters.

## Native receiving

Turnstone must retain its staged layers while completion is pending, service
input on intervening host turns, and present only through accepted publication.
The primary window and lens windows use the same mechanism. Resize, closed
windows and generation replacement discard obsolete staging. Host invalidation
survives a pending frame so a newly typed value or pane change cannot lose its
next repaint. The previously accepted surface remains the visible fallback.

**Done conditions:** the default native product and lens paths compile;
ordinary input, resize and close controls pass; and existing native image,
curve-quality and controlled loss regressions retain their original assertions.
Browser current-frame ownership and browser-specific scope adapters keep their
separate existing qualification boundary.

## Measurement

The retained optimized Turnstone measurement supplies an initial reproduction.
Refresh the application to current upstream before fixing its consumer path,
then retain a matching before/after binary and source cohort for attribution.
Repeat the palette cases with fresh synthetic profiles, identical dimensions
and no compiler overlap. Include an open Arrange pane and a live retained
document tile because those are the user's strongest lag reports. Report
presentation cadence separately from render duration, and keep forced redraw,
running-physics and paused-physics controls distinct. Native frame and adapter
captures establish the actual rendered fixture. A local measurement does not
establish another machine's throughput or a physical driver-reset guarantee.

## Findings

- 2026-10-30: the pinned checked native raster path waits in
  `qualify_native_raster` for the current Classic quality counters. The prior
  iMac sample placed most main-thread wall observations in that completion wait.
- 2026-10-30: the browser already has explicit pending-frame acceptance over
  asynchronous scopes and the same quality ledger. Native completion can use
  the existing loss monitor and ledger without adding a product-specific queue.

## Progress

- 2026-10-30: isolated the owner change from unrelated canonical documentation
  edits; implementing the native batch and receiving controls.

- 2026-10-30: native batching, demand-started completion and channel-only host
  collection pass the owner regression suite, including actual shader refusal,
  scope errors, cancellation and loss during submission, completion and final
  publication. Turnstone receiving and matched measurements remain in progress.
- 2026-10-30: an actual-loss control inside the publication callback exposed
  an idle-versus-active admission distinction. Final publication now holds a
  separate active interval, while unpolled idle acceptance remains cancellable.
- 2026-10-30: Turnstone receives pending frames in its primary and lens paths,
  pipelines the next demanded batch without an extra redraw turn and retains
  staged capture waiters. Controlled palette, Arrange and document cadence
  comparisons improve; resize, tear-out, lens input/capture and quiet paused
  admission controls pass. The Arrange CPU geometry projection cost remains
  separate from GPU completion, as does receiving newer supplier cohorts.
- 2026-10-30: hosts sharing the Core also include native workshop windows
  using checked drawing. The status-only busy hint exposes canceled work still
  retiring after a host drops staging, without admitting another draw or
  polling the GPU. Host scheduling remains responsible for fair window turns.

- 2026-11-03: supplier shutdown qualification at `adfa1905d9e` adds an
  explicit shared-Core close that seals admission and joins completion workers
  for current and retired generations. It waits for a publication already in
  progress, rejects reentrant close, and revokes accepted frames after close.
  Timeout diagnostics report admission, seal and worker elapsed times and the
  last observed scope/backlog counts, or `unknown` before observation. The
  five-second qualification deadline and channel-only UI collection remain.
  The CPU gate passed 5 tests; the actual-GPU native-frame gate passed 16.
  The original process-exit crash has not been rerun, and the new tests were not
  observed failing before implementation.

  The initial offline CPU attempt failed before compilation because this fresh
  worktree had no ignored `Cargo.lock` and resolution rejected yanked
  `clipboard-win 5.4.1`. The retry copied the ignored lock from
  `/home/markik/Code/worktrees/genet-native-render-20261103/Cargo.lock`; the
  copied lock's SHA-256 was
  `1f981cb6af462aafdcefec737dff310af2f8bde8f155a490437f967c379482ff`.
  No lockfile change is part of the supplier commit. All runs used the same
  wrapper and environment below, from
  `/home/markik/Code/worktrees/genet-native-shutdown-20261103`, with shell
  `pipefail` and output captured through `tee`:

  ```sh
  flock /home/markik/Code/testing/machine-heavy.lock \
    systemd-run --user --scope --quiet -p MemoryMax=3G -p MemorySwapMax=0 \
    -p CPUQuota=150% nice -n 10 timeout 600 env CARGO_BUILD_JOBS=1 \
    RUST_TEST_THREADS=1 CARGO_TARGET_DIR=/home/markik/Code/target \
    CARGO_BUILD_BUILD_DIR=/home/markik/Code/target CARGO_INCREMENTAL=0 \
    CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
    cargo test --locked --offline -p genet-presenter native_frame::tests --lib -- --test-threads=1
  ```

  The GPU gate used the same command with `--ignored` before
  `--test-threads=1`. Evidence is retained under
  `/home/markik/Code/testing/genet-native-shutdown-20261103/`:

  | Artifact | SHA-256 |
  | --- | --- |
  | `cpu-tests.log` (initial failure) | `3edc042165b571307540a36db5bc8645797d7639f07851c147588305e5f7c025` |
  | `cpu-tests-retry.log` | `ff93d3fb50ce76506477417c5a1c2210b3ae79c02b0e9b632490c230d61f7caa` |
  | `gpu-tests.log` | `fbba5330982d1626a9ebd0c6e545497d0f802d9551819d5398cdfb71a01f7d65` |
  | `inventory.json` | `bf4973ab3a56ce1e27c35a52e72d88afa2a15ccb3d0a4ea37289c209128dad20` |

  Changed-file `line-ceiling` passed. Scoped Clippy used the same wrapper and
  environment with `cargo clippy --locked --offline -p genet-presenter
  --all-targets`; it exited successfully with warnings. Those include missing
  Errors/Panics documentation for the new shutdown APIs and a collapsible
  conditional, alongside pre-existing package and dependency warnings.
  `cargo deny --locked check bans` was not run because `cargo-deny` was not
  installed. The source and dependency graph were otherwise unchanged by these
  checks. Further evidence under the same artifact directory:

  | Artifact | SHA-256 |
  | --- | --- |
  | `line-ceiling.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
  | `clippy.log` | `8dfc8c6107c90b624ded27e6817d1a3407c1029dac33de71b1f3b7d49616d72d` |
  | `source-checks.json` | `3a3c7873f04f4653f59ec076fc74500a49dde6f203280f7bd1d6d10b006329cc` |

- 2026-11-03: native qualification/collection separation at `33b23ed6882`.
  The completion worker still checks the original five-second deadline through
  worker-scope observation. Once that worker has qualified a frame, later
  collection and publication check Core/device liveness and loss rather than
  expiring the already-qualified result. Worker failures stay failures. Host
  identity, cancellation and redraw policy are unchanged. Timeout detail now
  includes sample age and worker phase durations; setting
  `GENET_NATIVE_PROGRESS_DIAGNOSTIC=1` also logs a successful worker result
  collected after its original deadline, including worker-finish age.

  The GPU regression was first run with only the three collection/publication
  checks restored to deadline enforcement; it failed on collection after a
  successful worker result. The same regression passed with the separation.
  An initial test compile failure from a corrected texture tuple is retained.
  The green suite passed 5 CPU controls and all 17 native GPU controls, including
  the original deadline and device-loss cases. Package-scoped Clippy exited zero
  with warnings; changed-file line-ceiling passed. The existing ignored lockfile
  was copied from the preceding shutdown qualification; its SHA-256 was
  `1f981cb6af462aafdcefec737dff310af2f8bde8f155a490437f967c379482ff`.

  Runs used `/home/markik/Code/worktrees/genet-frame-progress-diagnostic-20261103`
  with `flock /home/markik/Code/testing/machine-heavy.lock`, a user systemd
  scope capped at `MemoryMax=3G`, `MemorySwapMax=0`, `CPUQuota=150%`, `nice 10`
  and `timeout 600`. `CARGO_BUILD_JOBS=1`, `RUST_TEST_THREADS=1`,
  `CARGO_INCREMENTAL=0`, and both Cargo debug profiles were zero; target and
  build directories were `/home/markik/Code/target`. The red command was
  `cargo test --locked --offline -p genet-presenter native_frame::tests::qualified_frame_can_be_collected_and_published_after_deadline --lib -- --ignored --exact --test-threads=1`.
  Green used `cargo test --locked --offline -p genet-presenter native_frame::tests --lib -- --test-threads=1`,
  then the same command with `--ignored` before `--test-threads=1`, and
  `cargo clippy --locked --offline -p genet-presenter --all-targets`.

  Evidence is under `/home/markik/Code/testing/genet-frame-progress-diagnostic-20261103/`:

  | Artifact | SHA-256 |
  | --- | --- |
  | `red-compile-failure.log` | `e736af1c042e6cb9ed772077fbcd9bdc75e98a10a1933461c24dd55ed38bbef7` |
  | `red.log` | `6a99274b5b7e69bee5536055ad0c8f7b4b69e432f8989e06ecd622b17823a492` |
  | `green-cpu.log` | `9a79a3928d52d1a85348d19686bb45a194e18726632ca83f81c976b9706214fd` |
  | `green-gpu.log` | `2072ec7b4d82976fe5e864e49cc21511f921a6eb2e3dc2b3100ac46023bad250` |
  | `clippy.log` | `18e28a28e64c913123589a3e568b8a28c432773786f116b791c8981e99c8967c` |
  | `green-gates.sh` | `273b59ca6167cd28b19697cb51c0c66cc1ed26fd5d59d04fb03b7d37ae20cb3b` |
  | `inventory.json` | `2d0419d8fb3dd35fdd3cb5a5bdec21190b43cc9e94a86a547b873d7454b81ade` |

  The actual application timeout has not yet been rerun against this change.
  Separately, the blank window-frame raster has not yet been attributed to a
  particular provider failure; these tests do not establish a rendering fix.
