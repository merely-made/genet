# Retained and parallel inline shaping implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task by task. Use superpowers:subagent-driven-development only if delegated execution is selected. Steps use checkbox syntax for tracking.

**Status (2026-11-01):** implementation starting with Terra implementers and bounded Luna assistance, authorized in the follow-up conversation. Source baseline `a3ea104bb9a4c8589f79732f5b1ba54cfcd17970`; no implementation or performance qualification completed yet.

**Goal:** avoid repeated shaping of unchanged inline content and shape independent dirty groups concurrently, while preserving layout, paint, selection and hit-testing behavior.

**Architecture:** collect owned inline inputs during existing traversal, retain unbroken Parley layouts by their actual builder inputs, and resolve line geometry on the layout owner. Build a flat batch of cache misses at dependency boundaries; an optional caller-supplied native Rayon pool processes that batch with reusable shaping contexts. Retain the serial implementation as the default and correctness control.

**Tech stack:** Rust 2024; Genet MSRV 1.97.1; maintained `genet-parley` and `genet-fontique` 0.10.0; existing workspace Rayon dependency, optional in genet-livery.

**Spec:** the accepted discussion is recorded in [Scope and design contract](#scope-and-design-contract) below. This document includes the design and implementation sequence; there is no separate design document.

## Scope and design contract

The user selected both opportunities: "avoid repeated shaping, and parallelize the shaping that remains," and the traversal / independent shaping / retained placement sequence. The larger host orchestration and IPC design goes to another thread.

The work unit is a complete existing inline group with its full shaping and bidi context. Do not split a paragraph at arbitrary DOM nodes, style spans, byte offsets or worker quotas. Preserve the grouping semantics of the existing atomic text path too; repairing its CSS grouping is separate work.

Separate three kinds of state:

1. **Prepared input:** collected text, ordered style spans, atom descriptions and current source/owner mapping. Collected from the DOM only on its owning thread. Resolve width-dependent values when geometry becomes available; the preparation template retains the expressions needed for that resolution.
2. **Unbroken text:** Parley layout immediately after `builder.build(text)`, plus its exact builder-input key and font generation. The cached object contains span/atom slot indices, never live DOM references or authoritative old `BoxId`s.
3. **Placed lines:** a working clone of unbroken text, broken/aligned at the current width, then projected through Livery's existing line-box model using current source maps and atom geometry. Paint and text interaction use that same placed result.

Cache equivalence is exact equivalence of the input actually supplied to Parley, not equality of formatting-owner IDs. Include text, base level, scale/quantization, ordered ranges, every default/span builder property, resolved font features/tab stops, and inserted inline boxes (index, kind, dimensions). Use exact normalized scalar values, not the existing approximate width comparison. Compare the full key on a hash match. Normalize min/max/used requests to their effective builder properties, so identical policies share an entry; different overflow-wrap policies do not.

Initially, changed atom dimensions or topology cause a cache miss. Width alone does not when it affects only breaking, indent or placement. Do not omit zero-width decoration edges or insert new placeholder edges differently from the current collector to manufacture cache hits. `inline_boxes_mut()` exists in Parley, but changing cached atom dimensions without rebuilding is a separate optimization requiring its own fresh-build equivalence proof; this plan does not depend on it.

Cache lookup rebinding uses current span/atom slots. A pass-local `InlineGroupId` identifies a group in the flat queue, and a pass generation guards results. Across box-tree rebuilds, key equality supplies shaping reuse while source mappings are rebuilt. Existing retained-root geometry/identity reconciliation continues to own actual retained boxes and fragments. A local edit need only shape cache misses, but this plan does not claim that every full-layout path stops walking unchanged DOM.

Batches are flushed after inputs are ready and before their first measurement consumer. There may be several batches: atomic-subtree construction, main inline groups, retained-root replacement, and K7 basis reformatting. Unknown-width cases resolve at measurement and may miss the cache synchronously. Count that fallback explicitly; do not precompute all possible widths, block a worker waiting for layout, or reorder the existing dependency passes.

## Global constraints

- Scope is genet-livery text work. IPC, process placement, service workers, parallel cascade, parallel line placement, and renderer/device changes are excluded.
- Keep the existing Rust 1.97.1 floor. No new shaping engine, dependency upgrades, broad lockfile churn, or Parley behavioral fork.
- No `Send`/`Sync` bound added to `LayoutDom` or `D::NodeId`; no unsafe trait implementations to make a worker payload compile. Workers receive owned numeric/text inputs only.
- Font authority and final font-resource interning stay with `TextSystem`; each worker gets a clone of the same admitted font context. New, removed or replaced fonts invalidate cache entries and worker snapshots before reuse.
- Keep default builds and wasm serial. Native parallel work is opt-in through a `parallel-text` feature and a caller-supplied `Arc<rayon::ThreadPool>`; never select the global Rayon pool implicitly. Armillary remains unchanged.
- Worker scratch is reusable and bounded by the attached pool's thread count. Calls join before publishing results; no background jobs outlive the formatting transaction. A smaller batch uses only as many scratch slots as it needs.
- Suggested initial settings, not measured tuning: 512 retained entries; 4 MiB retained key-text bytes; parallel admission requires at least 8 unique misses AND 16 KiB of text. Every limit is configurable. Zero in either retention limit disables retention. A group larger than the byte limit is processed but not retained. These limits bound entry count and source text, not all Parley heap allocations; measure actual peak RSS separately.
- Serial remains the default even after the feature lands. Pool injection, cache limits and admission thresholds survive font-ledger rebuilds. Pool detachment releases idle worker scratch after the current synchronous call.
- Serialize expensive checks through `/home/markik/Code/testing/machine-heavy.lock`. Initial test caps: one Cargo job, one test thread, `MemoryMax=3G`, `MemorySwapMax=0`, `CPUQuota=150%`, nice 10. Do not silently raise caps. For the separate two-worker performance experiment, use a recorded `CPUQuota=200%` and the same memory/swap limits for every control.
- Execution is authorized by the follow-up request to orchestrate Terra/Luna subagents. Local task commits are allowed; pushes, shared-main integration and consumer repins are outside this execution. Preserve existing checkouts and worktrees.

## Review focus

1. Same text with newly allocated/reordered boxes must select and hit-test the current nodes (Task 3).
2. Min-content `overflow-wrap: break-word`, percentage decorations, loaded images and K7 basis changes must not reuse an incompatible layout (Tasks 2–4).
3. Font replacement, fallback, generic bindings and face features must be identical in serial and worker paths (Tasks 3 and 5).
4. Bidi, Arabic joining, ligatures, combining characters, tabs, generated text, forced breaks and empty inline boxes must preserve glyphs and UTF-8 source ranges (Tasks 2 and 5).
5. Tiny edits, one huge paragraph, repeated navigation, cache eviction and feature-disabled hosts must remain correct without thread or cache growth proportional to history (Tasks 3, 5 and 6).

## Findings (2026-11-01)

- [`text.rs`](../components/genet-livery/src/text.rs): `format_inline_group` collects, shapes and places synchronously. `shape` calls `builder.build`, then line breaking, alignment, CSS line-box placement and font interning. `shape_count` counts calls to this combined operation, not individual HarfRust invocations, and excludes some metric probes.
- [`layout.rs`](../components/genet-livery/src/layout.rs): `InlineMeasure.layouts` already caches placed results by width, intrinsic flag and float constraints within a measure context. Its `intrinsic: bool` cannot express all policy distinctions. Retaining unbroken text is an additional layer, not a replacement for placed-result retention.
- [`layout/build_block.rs`](../components/genet-livery/src/layout/build_block.rs): atomic text leaves eagerly format min/max widths during `BuildState::build_box`; `wrapped_text_height` formats again at finite width. Construction therefore has a real consumer of shaping before the later main inline pass.
- [`layout/transaction.rs`](../components/genet-livery/src/layout/transaction.rs): contribution, main inline, and optional K7 basis passes have ordered dependencies. Tables can measure before `compute_layout_with_measure`. [`layout/retained.rs`](../components/genet-livery/src/layout/retained.rs) supplies a separate retained-root entry that needs the same preparation path.
- `BoxInlineCollector::push_atomic_box` resolves contribution widths and percentage margins; `push_edge` conditionally inserts nonzero edges. `overflow_wrap` changes builder policy for min-content. These cannot be treated as universally width-independent input.
- `Brush` carries color and `source_index`; font keys already derive from font content through `intern_font`. Cached color must either participate in the key or be refreshed explicitly. The initial design includes it in the key and preserves existing paint-only shortcuts.
- [`document/resources.rs`](../components/genet-livery/src/document/resources.rs) replaces `TextSystem` during font rebuild. New execution settings must be preserved, while retained text and worker font snapshots must be discarded. Fontique collection clones are not live-updating by default (`CollectionOptions::shared = false`).
- [`phase.rs`](../components/genet-livery/src/phase.rs) records disjoint caller-thread phases. Summed worker time cannot be added to its layout wall time without double counting.
- [June parallel-cascade scope](../docs/2026-06-13_parallel_cascade_scope.md) describes `shape_inline_leaves` / `shape_leaf` in an older pipeline. Those symbols are absent from current `components`; the historical claim does not qualify this implementation. Preserve the existing [line-box model](2026-09-25_line_box_model_plan.md) and [text boundaries](2026-09-26_text_boundaries_plan.md).

## File boundaries and interfaces

New modules under `components/genet-livery/src/text/`:

- `prepared.rs`: owned preparation templates, normalized builder inputs, current source sidecars and pass-local IDs.
- `shaping.rs`: Parley construction and reusable `ShapeWorker`; move only the pre-break responsibilities from `text.rs`.
- `cache.rs`: document-local bounded LRU and exact-key comparisons.
- `parallel.rs`: optional native pool batch execution, worker scratch and deterministic result ordering.
- `stats.rs`: counters plus opt-in timing; no dependency on a UI or global observer.
- `reuse_tests.rs`: private tests for preparation, cache and worker equivalence, registered under `#[cfg(test)]` in `text.rs`.

Keep CSS line placement, glyph/source projection and `intern_font` in `text.rs`; keep geometry scheduling in `layout/`. Add `layout/text_batch.rs` for the flat queue and flush helpers. Add public document tests in `tests/text_shaping.rs` and a benchmark driver in `examples/measure_text_shaping.rs`. Export only configuration and observations from `lib.rs`; preparation/cache types stay private.

Proposed Rust interfaces below are declarations for implementation, not compile-ready code. Types named here are defined by the owning task; details not fixed here follow existing Rust conventions.

```rust
// Task 1/3, public configuration and observations
pub struct TextShapingOptions {
    pub retained_entries: usize,
    pub retained_text_bytes: usize,
    pub parallel_min_groups: usize,
    pub parallel_min_text_bytes: usize,
    pub record_timings: bool,
}
impl TextSystem {
    pub fn set_shaping_options(&mut self, options: TextShapingOptions);
    pub fn shaping_options(&self) -> &TextShapingOptions;
    pub fn shaping_stats(&self) -> TextShapingStats; // cumulative; callers take deltas
}
impl<D: LayoutDom> LiveryDocument<D> {
    pub fn set_text_shaping_options(&mut self, options: TextShapingOptions);
    // Same existing NodeId bounds as other document methods.
}

// Task 2, private; ShapeInput has no DOM/BoxId ownership.
struct ShapeInput { /* owned normalized builder properties, text and atom slots */ }
struct PreparedInlineGroup { /* ShapeInput plus current BoxId sidecar and placement data */ }
struct UnbrokenText { /* immutable Parley Layout<Brush> */ }
impl ShapeWorker {
    fn shape(&mut self, input: &ShapeInput) -> UnbrokenText;
}
impl TextSystem {
    fn place_inline_group(&mut self, group: &PreparedInlineGroup,
        shaped: &UnbrokenText, geometry: InlineGeometry<'_>) -> InlineLayout<BoxId>;
    // InlineGeometry names width, intrinsic kind and optional float constraints.
}

// Task 3, private; font generation is included in ShapeKey.
impl ShapeCache {
    fn get(&mut self, key: &ShapeKey) -> Option<Arc<UnbrokenText>>;
    fn insert(&mut self, key: ShapeKey, value: Arc<UnbrokenText>);
    fn clear(&mut self);
}

// Task 4, private; results are consumed in request order.
struct InlineGroupId(usize); // transaction-local, not persistent box identity
struct ShapeJob { /* input, exact key, pass generation, group ID */ }
struct ShapeResult { /* matching key, generation, group ID, unbroken layout */ }
impl TextSystem {
    fn shape_batch(&mut self, jobs: &[ShapeJob]) -> Vec<ShapeResult>;
}

// Task 5, native parallel-text only; both setters accept None to detach.
impl TextSystem {
    pub fn set_shaping_pool(&mut self, pool: Option<Arc<rayon::ThreadPool>>);
}
impl<D: LayoutDom> LiveryDocument<D> {
    pub fn set_text_shaping_pool(&mut self, pool: Option<Arc<rayon::ThreadPool>>);
}
```

## Tasks and done-conditions

### Task 1: establish counters and reproducible controls

**Files:** modify `text.rs`, `lib.rs`; create `text/stats.rs`, `examples/measure_text_shaping.rs` and the initial `text/reuse_tests.rs`.

**Interfaces:** define `TextShapingStats` with `format_requests`, `layout_builds`, `metric_layout_builds`, `line_breaks`, `cache_hits`, `cache_misses`, `unique_jobs`, `late_misses`, `batches`, `retained_entries`, `retained_text_bytes`, and opt-in `collect_ns`, `shape_wall_ns`, `worker_task_ns`, `place_ns`. Counters are cumulative `u64` except the two retained-size `usize` gauges. Durations are cumulative nanoseconds. `worker_task_ns` sums task elapsed time, not OS CPU accounting; `line_breaks` counts formatting break operations, not emitted lines. Keep the existing `shape_count` semantics and document the distinction.

- [x] Add `shaping_stats_count_builds_separately_from_formats`: a known plain fixture records builder operations, metric probes and formatting separately; a cached frame changes none. Instrument actual builder sites, not an assumed call ratio. Confirm the test fails before adding instrumentation, then passes.
- [x] Implement counters and owner-thread wall spans; collect worker task totals separately when Task 5 lands. Keep timers disabled unless requested; never nest these subtimings into the existing disjoint phase totals.
- [x] Implement a driver with `--mode uncached-serial|cached-serial|cached-parallel`, `--case small|article|mixed|single-long|atomic|churn`, `--paragraphs`, `--workers`, `--iterations`, `--font`, `--font-family`, `--cache-entries`, `--cache-text-bytes`, `--parallel-min-groups`, `--parallel-min-text-bytes`, and `--output`. Before later modes exist, reject them explicitly. Generate deterministic content with unique paragraph suffixes; otherwise inter-group deduplication can hide the workload.
- [x] Capture initial-frame, unchanged-frame, width-change and one-paragraph-edit results. Each row records source revision, features, case/size, font hash, options, workers, counts, wall durations, and output digest. Preserve this Task 1 build as the instrumented original-pipeline control. For the pristine parent revision, run the common fixture/frame timing harness without the new stats API and label its unavailable counters; new uncached code alone is not an original-baseline measurement.
- [x] Run `cargo test -p genet-livery --lib shaping_stats --locked --offline -- --test-threads=1` under the bounded wrapper below. Record the baseline in the receipt, then checkpoint only this task if implementation publication has been authorized.

**Done:** measurements distinguish preparation, actual builds, metric probes, breaking and total frame work; no speedup inferred yet.

### Task 2: split collection, shaping and placement without caching

Execution subdivision (2026-11-01): 2A isolates normalized builder inputs and Parley construction while retaining the existing collector; 2B adds the owner-side geometry-resolvable collection template. Review each subtask; both remain required for this task to close.

**Files:** create `text/prepared.rs`, `text/shaping.rs`; modify `text.rs`, `text/reuse_tests.rs`, `layout.rs`.

**Interfaces:** define `PreparedInlineGroup`, normalized `ShapeInput`, `ShapeWorker`, `UnbrokenText`, `InlineGeometry` and `place_inline_group` above. The collector produces a geometry-resolvable template; materialization yields exact builder inputs and a current source sidecar. Keep direct overlay/control entry points working through the common split where applicable.

- [x] Add fresh-versus-split tests `split_shape_matches_fresh_inline_group` and `split_shape_preserves_intrinsic_policy`: compare glyph IDs/advances, font content keys, line sizes/baselines, decoration positions and per-source ranges. Cover mixed spans, bidi/Arabic/ligatures, combining/emoji, generated markers, tabs, forced breaks, empty inlines, atom-only lines, nested vertical-align and quirks mode. Include `overflow-wrap: break-word` min-content versus used width. Run them against the original combined path before removing it.
- [x] Move Parley builder setup/build into `ShapeWorker::shape`; clone its unbroken result before breaking. Keep indent, float bands, alignment, CSS line extents and font interning on the owner. Avoid capturing `&mut TextSystem` in any job payload.
- [x] Normalize builder properties in one shared conversion consumed by both key construction and the builder. A newly supported property must not silently be missing from the cache key. Resolve required tab metrics/font-face features on the owner before dispatch.
- [x] Replace `InlineLayoutEntry.intrinsic: bool` with the actual `Option<IntrinsicSizeKind>` (or an equally precise internal policy enum) and update cache matching/used-layout preference. Preserve current finite width tolerance; explicitly match equal infinities before subtracting widths. Add a test that min/max/used policies cannot alias at the same width.
- [x] Run `cargo test -p genet-livery --lib split_shape --locked --offline -- --test-threads=1`, then existing text and layout unit groups. Retire the temporary combined-path test oracle after recording parity; retain independent expected assertions and fresh uncached comparisons.

**Done:** uncached serial output agrees with the original for the exercised cases; the split has introduced no reuse and supplies the only production shaping implementation.

### Task 3: retain unbroken text safely across measurements and frames

**Files:** create `text/cache.rs`, `tests/text_shaping.rs`; modify `text.rs`, `text/prepared.rs`, `text/stats.rs`, `document.rs`, `document/resources.rs`, `lib.rs`.

**Interfaces:** define `ShapeKey`, `ShapeCache`, public `TextShapingOptions` and the option/stat accessors above. Cache values are `Arc<UnbrokenText>`; current BoxId mappings remain outside them. Implement a bounded LRU using standard collections; eviction must not scan the entire cache per hit.

- [x] Add `plain_resize_reuses_unbroken_text`: after first layout, two finite widths change line placement with zero new inline layout builds when text/builder inputs are unchanged. Add `intrinsic_policy_reuses_only_equal_builder_inputs`: ordinary min/max/used passes reuse; differing overflow-wrap policy builds its own entry.
- [x] Add `text_reuse_rebinds_current_sources` and `one_paragraph_edit_preserves_other_shapes`: insert/reorder/delete siblings, including identical text in separate owners; compare current selection/hit targets and paint against fresh layout. Removed nodes never survive through cached source references. For an admitted one-group text edit, unrelated unchanged keys produce no new builds.
- [x] Implement exact-key lookup and retention limits; include brush color until a separately proven paint-independent key is introduced. Preserve existing paint-only document shortcuts. Resolve atom geometry afresh before key construction; width-sensitive changes must miss when builder inputs differ.
- [x] Add `font_generation_invalidates_shapes`, including replace/remove, generic binding and face features, direct TextSystem font registration and document-ledger rebuild. Increment/replace the font generation at every admitted resource change; clear caches and scratch as appropriate. Preserve shaping options across reconstruction. Invalid generic replacement retains the previous valid state.
- [x] Add `retention_limits_bound_churn`: small capacities, zero capacity, oversized paragraph and repeated evictions remain equivalent to uncached output; retained counts/text bytes respect configured limits. Cache keys and font generations remain document-local. Evicted objects may remain alive in an active transaction until its join, then release.
- [x] Run `cargo test -p genet-livery --test text_shaping --locked --offline -- --test-threads=1` and `cargo test -p genet-livery --lib text:: --locked --offline -- --test-threads=1`; compare driver original/uncached/cached serial before enabling any parallelism.

**Done:** demonstrable fewer builds, correct source rebinding and bounded retention; performance evidence distinguishes saved work from added key/clone costs.

### Task 4: prepare a flat queue during traversal

Execution subdivision: 4A adds reusable prepared formatting and the serial batch API; 4B integrates atomic pending measures and their consumers; 4C integrates main traversal, retained-root and K7 queues. All are required.

**Files:** create `layout/text_batch.rs`; modify `layout.rs`, `layout/build_inline.rs`, `layout/build_block.rs`, `layout/transaction.rs`, `layout/retained.rs`, `text.rs`, `text/reuse_tests.rs`.

**Interfaces:** define the pass-local IDs/jobs/results and serial `TextSystem::shape_batch` above. `TextBatch` owns the ordered prepared groups and deduplicated ready misses. Late requests use the same cache/shape function and increment `late_misses`.

- [x] Add `prepared_batch_matches_lazy_measurement`: eager batch and lazy uncached controls agree; repeated identical keys submit one unique job and bind to each current source sidecar. A `CountedDom` fixture confirms preparation does not revisit descendants for each width query.
- [x] Queue preparation at `InlineBuildState::build_inline_group`. Materialize ready inputs and flush before table measurement and `compute_layout_with_measure`, not after they have already shaped everything. Do the same in retained-root replacement and each K7 pass. Dependent geometry is resolved when supplied, with a keyed synchronous miss if necessary.
- [x] Replace eager shaping in `BuildState::build_box`'s text leaf branch with pending text-measure initialization. Add `BuildState::prepare_text_measures(&mut self) -> Result<(), LayoutError>`: flush ready jobs, calculate min/max and baselines on the owner, then populate the existing `TextMeasure` fields. Invoke it before every consumer of those fields, including scratch intrinsic queries, table measurement and atomic layout. Keep the no-TextSystem fallback behavior. Do not leave provisional estimated widths observable to a consumer.
- [x] Store prepared group handles in `InlineMeasure` and `TextWrap`; reuse preparation for finite-width remeasurement. Exhaustively audit `BuildState` construction and measurement entry points with `rg 'BuildState|compute_layout_with_measure|apply_buckram_table_layout' components/genet-livery/src/layout*`.
- [x] Add `prepared_batch_preserves_geometry_dependencies`: floats, percentage inline padding/margins, inline tables, loaded/replaced images, out-of-flow inline descendants, K7 contribution/basis passes and retained text edits match a fresh uncached layout. Verify unresolved geometry does not publish a guessed result, and batch output never overwrites a newer pass.
- [x] Run focused `prepared_batch` tests plus `cargo test -p genet-livery --lib layout:: --locked --offline -- --test-threads=1` and existing retained-root document tests. Driver counters must show ready jobs before their first measurement, not a queue populated after the work has happened.

**Done:** all named paths share preparation/reuse; the batch is useful while still serial. Report readiness and late-miss counts for each fixture. Dependent geometry remains ordered.

### Task 5: execute independent shaping jobs in parallel

**Files:** create `text/parallel.rs`; modify `Cargo.toml`, `text.rs`, `text/shaping.rs`, `document.rs`, `document/resources.rs`, `lib.rs`, `text/reuse_tests.rs`, `tests/text_shaping.rs`, benchmark driver. Use the existing workspace Rayon version.

**Interfaces:** add the two native feature-gated pool setters above. Keep `shape_batch`'s input/output contract unchanged. `ShapeWorker` owns cloned FontContext and LayoutContext; refresh it from the owner when its font generation differs. Never create a new system font collection per job.

- [ ] Add compile-time `Send` checks for actual job/result payloads and runtime `parallel_shape_matches_serial` controls at 1 and 2 workers. Keep a deliberately `!Sync` test DOM usable through the public document API. No DOM access is allowed inside the worker closure.
- [ ] Execute at most one mutable scratch slot per active shaping task in `pool.install`. Let slots take ready jobs from a bounded batch work index; preserve result IDs and reassemble in request order. Allocate output per slot, join, then publish/cache on the owner. Avoid a shared TextSystem mutex around shaping, per-job font contexts, detached jobs or nested pool creation.
- [ ] Add `parallel_shape_font_snapshots_match` with explicit fixture fonts, missing coverage/fallback, face features and a font rebuild between batches. Verify worker font bytes and metrics match serial and owner `intern_font` retains all resources needed by output. With system fonts, report selected byte identities; fixture-only success is narrower evidence.
- [ ] Add `parallel_shape_threshold_and_lifetime`: no pool, feature disabled, below-threshold batches and one huge group remain serial; two ready independent jobs can overlap using a synchronization latch (no timing-based speed assertion). Detach/swap pools, repeat document creation and drain/drop scratch. Propagate worker panic without committing partial results; retain ownership until Rayon joins.
- [ ] Preserve options and the supplied pool across font-ledger rebuild, clearing obsolete scratch. Read fresh stats from joined results; caller layout wall time already includes pool wait and merge time.
- [ ] Run `cargo test -p genet-livery --features parallel-text --test text_shaping --locked --offline -- --test-threads=1`, private `parallel_shape` tests, and the same public controls with default/no-default features. Check `cargo check -p genet-livery --target wasm32-unknown-unknown --no-default-features --locked --offline`; ensure the native worker dependency is not activated there.

**Done:** serial/parallel equivalence, visible concurrent work and bounded lifecycle are established. Parallel speedup remains a separate Task 6 result.

### Task 6: qualify total latency, memory and integration

**Files:** finish `examples/measure_text_shaping.rs`, `tests/text_shaping.rs`; update `components/genet-livery/README.md`, this plan and its indexed status. Append results to `design_docs/archive_docs/receipts/2026-11-01_retained_parallel_shaping_plan_receipts.md` only when runs exist.

- [ ] Benchmark original baseline, new uncached serial, retained serial and retained parallel with the same release settings, pinned font bytes, CPU/memory limits and fixture inputs. Use unique 32/256/2048-paragraph articles, 8 short labels, mixed-script paragraphs, one long paragraph, atomic/float content and 100 create/edit/drop cycles. Match the fixture corpus across modes; one giant paragraph is intentionally not split.
- [ ] For each workload, measure cold initial frame, warm unchanged frame, alternating widths and one paragraph edit. Separate warm worker/font-cache runs from cold creation; record 30 measured iterations after 5 warmups for warm cases, with mode order alternated. Run both the proposed default cache limits and an explicitly sized retention control (at least 4096 entries and 32 MiB key-text budget for the 2048-group fixture); the smaller default cache cannot promise zero builds across a working set it cannot hold. Report median/p95 total frame and layout wall time, task-summed shaping time, preparation/key/clone costs, hit/build/late-miss counts and peak RSS. Record limits, actual workers, CPU model, source/lock/font hashes and output digest. A throttled two-worker result is not an unrestricted scaling claim.
- [ ] Acceptance: no glyph/font/source/line/paint differences attributable to this work; plain resize within the retained working set has zero additional inline builds; admitted small edit rebuilds only changed keys; retention respects its limits. Retained serial must improve repeated text reflow, and retained parallel must improve total layout time over retained serial on at least the cold or broadly dirtied 2048-group workload by more than measured run variation. A warm all-hit cache has no parallel work and is not a parallel speedup test. Small-label/edit median and p95 must stay within 5% of the serial control or the recorded noise band, whichever is larger. If not, tune configurable admission thresholds or leave affected workloads serial and record the limitation; do not claim a broad win from shape time alone.
- [ ] Run `cargo test -p genet-livery --locked --offline -- --test-threads=1`, then the parallel-feature suite, plus relevant `genet-documents` Livery consumer tests. Re-run only failures/changed areas after repairs. Check format with `cargo fmt -p genet-livery -- --check`.
- [ ] Use the current `genet-wpt` runner's documented CLI for matched before/after reftests in `css/css-text`, `css/css-writing-modes`, `css/CSS2/linebox`, `css/CSS2/floats`, `css/css-sizing`, and `css/css-inline`, plus existing selection/caret controls. Record exact commands and zero pass-to-fail/error/timeout transitions; skipped/unsupported directories are not passing gates. Fix attributable regressions rather than updating expectations to hide them.
- [ ] Keep CPU text/layout, CPU raster, Wasm compilation, WPT, and native headed host evidence separate. No GPU speed claim follows from this CPU optimization. Update the README with configuration, defaults, memory-limit meaning and the measured ready/late work boundary.

**Done:** measured reuse and parallel gains are attributable separately, correctness gates hold, regressions and untested surfaces are explicit, and defaults do not assume every host has spare cores.

## Execution commands and evidence discipline

Cargo commands above run from a fresh implementation worktree reconciled with origin/main. Use an isolated target directory and the shared heavy-work lock. Example wrapper (shell-ready once substituted with the actual Cargo command):

```bash
flock /home/markik/Code/testing/machine-heavy.lock \
  systemd-run --user --scope --quiet \
  -p MemoryMax=3G -p MemorySwapMax=0 -p CPUQuota=150% \
  nice -n 10 env CARGO_BUILD_JOBS=1 \
  CARGO_TARGET_DIR=/home/markik/Code/testing/genet/shaping-target \
  cargo test -p genet-livery --lib shaping_stats --locked --offline -- --test-threads=1
```

If locked/offline resolution fails, record the exact cause and make only the narrow justified dependency repair. A missing target or unsupported sanitizer is an untested boundary, not a pass. For Task 5, also use an available supported native race-detector configuration; record its toolchain/flags and limitations separately rather than installing or upgrading toolchains as an implicit part of this plan.

Make independently reviewable checkpoints after Tasks 1–5 and a qualification/documentation checkpoint after Task 6, if commits are authorized at execution time. Fetch and reconcile again before integration. Do not push from the planning worktree.

## Decisions

- **retained_parallel_shaping D1 — user scope, new (2026-11-01):** "Down to plan these. I can split off the bigger orchestration idea to another thread." The two selected ideas are retained shaping and flat-batch parallel shaping. Implementation is not requested in this turn.
  Links: area layout · plans retained_parallel_shaping · summary Scope is reuse and parallel shaping; orchestration is separate.
- **Implementation proposals, not separately user-approved:** exact cache limits, admission thresholds, optional caller-owned Rayon pool, exact builder-input keys, and synchronous transaction completion. Task 6 may tune limits based on evidence; changing semantic ownership or adding IPC requires revisiting scope.

## Progress

- 2026-11-01: fetched Genet origin/main, inspected current shaping/measurement/resource paths, and prepared this plan in an isolated planning worktree. All task checkboxes remain open. Planning-file checks are not implementation evidence.

- 2026-11-01: user selected Terra/Luna orchestration; implementation starts on `codex/retained-parallel-shaping-20261101`.

- 2026-11-01: Task 1 complete at `5285e5e5859` after Terra implementation/review and Luna scoped fix review. Focused 2 passed; library 373 passed/2 ignored; original/instrumented four-stage paint digests match. Measurements are single debug controls, not speed claims. [Baseline receipt](archive_docs/receipts/2026-11-01_retained_parallel_shaping_plan_receipts.md).

- 2026-11-01: Task 2A implemented/reviewed at `0bb48169e0a`: owned exact builder inputs, one serial worker, owner-side placement, precise intrinsic policy cache. Library 377 passed/2 ignored; selected old/split controls match. Task 2B collection template remains open.

- 2026-11-01: Task 2 complete at `080e634066a` after Terra implementation/review and Luna fix review. Geometry tape matches 27 original-collector cases; eight end-to-end formatter and paint controls match Task 1. Full library 378 passed/2 ignored before the test-only offset fix; focused fix test passed. Explicit fixture coverage and preserved oracle provenance are in the canonical receipt; broader conformance and parallel-script coverage remain qualification work.

- 2026-11-01: Task 3 complete at `f8e0e452e55`, reviewed after behavioral-test strengthening. Exact-key retention, font invalidation and current-source rebinding are covered by 6 public tests; full library at the production commit passed 381/2 ignored. Small debug controls show saved builds with matching paint; release performance and memory qualification remain Task 6.

- 2026-11-01: Task 4A complete at `ce2001cbbd7`, reviewed after atomic batch rejection fix. Four focused batch tests pass, including active results after eviction; text suite 49 passed/1 ignored before scoped fix. Task 4B scheduling/readiness remains open.

- 2026-11-01: Task 4B complete at `b37f103ffc5`, reviewed. Atomic text metrics are deferred and batched before consumers; TextWrap retains preparation/results. Supporting Buckram context-update API has one-pass cache invalidation and passed 275 library tests. Livery focused7, layout136/1ignored, retained26, text51/1ignored passed. Main inline scheduling remains Task 4C.

- 2026-11-01: Task 4 complete at `f396679f197`, reviewed across API, atomic and main integration. Geometry controls now compare descendant fragments/paint, prove changed K7 basis and reject missing atoms before publication. Driver article control: 8 ready unique jobs, 0 late misses, 1 initial batch; four-stage paint matches original. Details in canonical receipt.
