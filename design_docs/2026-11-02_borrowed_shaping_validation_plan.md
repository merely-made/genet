# Borrowed shaping validation implementation plan

> **For agentic workers:** Use superpowers:subagent-driven-development. Preserve the worktree, original ledgers and evidence. Steps use checkbox syntax.

**Goal:** reduce owned-input construction during exact prepared-result validation, measure the remaining mixed-script and single-long costs, and investigate shared shaped buffers only if the first cut leaves a worthwhile copying cost.

**Architecture:** validate current owner-thread inputs through borrowed normalized values. Reuse an existing owned key only after complete value, font/pass/group and current-seed fingerprint checks. Misses and ambiguous candidates retain the existing owned-key/late path; workers continue receiving owned inputs. A possible later shared-buffer probe is separate from this frontend change.

**Tech Stack:** existing Rust, local Parley0.10/Fontique and optional caller-owned Rayon. No dependency or toolchain upgrades.

**Spec:** user-approved chat description, “validate borrowed inputs, allocate an owned key on a miss,” followed by “go for it”; existing semantic contract in `2026-11-01_retained_parallel_shaping_plan.md` and source-bound checkpoint/receipt in `2026-11-01_shaping_cost_reduction_plan.md`. The previous whole-matrix cold-equivalence goal remains open; no new tolerance is inferred.

## Global Constraints

- Start from clean local branch `codex/retained-parallel-shaping-20261101`, handoff9d1e22de23463aa7b92f50743fd30c2bebb8adb4, qualified production24f743517497bec0d44316ab0794572441f70177. Preserve this worktree and all prior artifacts; no shared-main merge, push, consumer repin or deletion.
- Upstream checked2026-11-02: origin/main ad362723109b7e1738dfce13a625cf1a1b5e9ed1; no Livery/Buckram/local-Parley changes since reconciled0cceb7e2fbc. Keep this experiment's existing source/closure frozen rather than merge unrelated changes into the comparator.
- Exact text/property/range/slot/base-level/scale/quantization identity, current font generation, pass/group metadata, forced-collision safety, active-result lifetime, owner/source/paint bindings and transactional failure ordering remain binding. No pointer/token/hash-only substitute for complete input comparison.
- Keep the existing ShapeInput/Property/Slot equality and hashing semantics. Recompute the matching candidate's complete fingerprint with the CURRENT TextSystem seed after full current-value verification; do not trust its cached fingerprint or change the hash schema.
- Rust floor1.97.1; default and wasm serial; caller-owned pool only; no unsafe Send/Sync, borrowed worker payloads, paragraph splitting, IPC or orchestration work.
- Shared /home/markik/Code/testing/machine-heavy.lock; all builds/tests/profiles3GiB/zero cgroup swap/150% CPU/nice10, one Cargo job/test thread. Performance controls200% with the same memory/swap limits and common CPU0,2 affinity for all modes. Host swap stays enabled; no sibling reservation claimed.
- Frozen ignored lock SHA57d39f302153c1c245655663d096888cbcc34d85ec83ee63e16f1a24cccd1a06; locked/offline, no upgrades/installs. CARGO_TARGET_DIR=/home/markik/Code/testing/genet/shaping-target and explicit build.build-dir=/home/markik/Code/worktrees/genet-shaping-plan-20261101/target/build.
- Terra handles design/implementation/task review, one writer at a time. One most-capable final review; preserve rather than delete this plan workspace. External profilers/scripts/raw rows stay hashed under testing/genet.

## Review Focus

1. Borrowed comparison must match the prior complete owned input for every field and every property, including bit-exact floats, absent versus zero spacing and ordered features.
2. Foreign-seed/forged-fingerprint, wrong key/group/pass/font generation and multi-candidate paths must fall back safely without weakening the old error/publication order.
3. Font features/tab resolution and metric state must occur once in the existing order, including empty/error/hanging cases; allocation removal must not silently skip current normalization.
4. Zero retention, eviction, oversize, current atom geometry, min/max/used and current source owners must preserve active result and output behavior.
5. Performance/allocation evidence must distinguish constructor removal, resolution allocation, output-owned text and the separate whole-layout clone; no universal speed or total-memory claim.

### Task 1: borrowed validation for a single prepared result

Files: components/genet-livery/src/text.rs, text/prepared.rs, text/cache.rs; tests there plus document/tests.rs. No public API.

Interfaces: a private borrowed-property visitor in prepared.rs is the sole CSS-to-normalized-property mapping, used by owned properties construction and borrowed equality. A canonical slot iterator is shared by owned construction and borrowed comparison. ShapeInput gains a private matches-current interface covering all fields. ShapeKey gains a private current-seed fingerprint recheck using the same seed.hash_one((&generation,&input)) as new(). Private interfaces:
- `visit_properties<'a>(style: &'a ComputedValues, source_index: usize, features: &'a [FontFeature], intrinsic: Option<IntrinsicSizeKind>, tab: f32, visit: impl FnMut(PropertyRef<'a>))`; PropertyRef mirrors normalized Property variants with borrowed family source and feature slices, preserving scalar bits and sequence.
- `properties(style: &ComputedValues, source_index: usize, features: &[FontFeature], intrinsic: Option<IntrinsicSizeKind>, tab: f32) -> Vec<Property>` uses that visitor and owns each resulting payload once.
- `slots<Id>(atoms: &[InlineAtom<Id>]) -> impl Iterator<Item = Slot> + '_` preserves the old enumerate-before-marker-filter rule and exact slot bits.
- `ShapeInput::matches_current<Id>(&self, text: &str, spans: &[SourceSpan<Id>], atoms: &[InlineAtom<Id>], root: &ComputedValues, intrinsic: Option<IntrinsicSizeKind>, resolved: &ResolvedShapeStyles) -> bool` compares every field with the shared normalization helpers.
- `ShapeKey::fingerprint_matches(&self, generation: u64, seed: &RandomState) -> bool` requires matching generation and recomputes the existing full owned-input hash; it is not a current-value matcher by itself.
The existing transactional block can combine these checks and return owned key/shaped clones without a new public or cross-task transport interface.

Only Candidates::Transactional with exactly one result may take the new path initially. Always resolve current style features/tabs once, in existing order. Require result pass/group and key font-generation match, full current ShapeInput value match, and current-seed fingerprint match. Then clone the existing key and shaped Arc, ending all borrowed views before a late append or placement. If any condition fails, construct one owned key with the ALREADY resolved styles, perform the existing full-key selection and late-batch handling. Multiple/zero candidates and direct/atomic callers retain old behavior. Current geometry materialization remains once per invocation. This intentionally leaves general cache lookup and multi-candidate allocation unchanged.

- [ ] Write an actual-construction work regression: the2048-group real fixture expects2048 eager owned keys plus2048 borrowed validations, instead of4096 owned keys. Observe runtime RED before production edits. A retained single-result repeat creates no owned key; empty-candidate late creates exactly one. Existing three-candidate4-key behavior stays unchanged.
- [ ] Add independent frozen expected-property tables/reference for the pre-change owned normalization, then test borrowed-match iff full owned equality across text/defaults/spans/ranges/slots/direction/scale/quantize and all13 properties. Do not build both oracle sides from the new visitor only. Test changed bits, family variants/list/whitespace, feature order/tag/value, all line-height/overflow policies and optional spacing.
- [ ] Implement canonical borrowed normalization/matching and fingerprint recheck without changing owned Eq/Hash. Avoid introducing extra feature-list allocations on misses: owned construction can consume borrowed resolved features once instead of cloning an intermediate Vec.
- [ ] Integrate only the guarded single-result transaction path. Preserve current normalization effects and key_materialize_ns coverage of borrowed comparison/rehash; formatting shape_wall_ns and empty/error boundaries remain as before. Test-only counters distinguish validation from owned construction rather than relabeling work.
- [ ] Exercise actual foreign seed/forced fingerprint, metadata, collision, failure/publication and retained-active cases. Reuse independent G/H output/glyph/font/source/caret/paint/minmax/used and late-failure oracles, extending only missing changed-path coverage. Returned output remains owned after dropping prepared data.
- [ ] Run default/parallel library and public text_shaping gates plus scoped fmt/diff, frozen closure and caps. Preserve failing receipts and observe GREEN. Commit locally and independently review spec plus quality before measuring/adopting more work.

### Task 2: targeted allocation and timing qualification

Files: existing measurement example only if a new work-observation needs it; hashed external scripts/artifacts; README and canonical receipt append after final evidence. Reuse exact preserved original source7b86d24/binarya5095df0 and pre-change source24f743/binarya4054216 from testing/genet/task2j-20261102 and Task3 records. Do not label the9d1 docs handoff as the source of that release.

- [ ] Preserve one release checkpoint with image-decode + parallel-text and exact source/lock/features/toolchain/binary hashes. Sanity before sustained work: real fonts/complete selected-face byte sets, paint, fresh current stats and actual constructor/borrow work. Mixed includes PakType Naskh Basic SHA dbd7efb102063f715afda2442e5245e6bda7ca86c2388756315857da0528a6bf alongside pinned Noto SHA fb53504de62e767a54695416123d68d67f71c9b69722175184ad56e7dd13547b.
- [ ] One first-frame allocation sample each mixed256/single-long2048 and a normal2048 control, confirming removed owned-key sites while output text and remaining resolution allocations are not claimed removed. Cumulative traffic is not live/copy volume.
- [ ] Three fresh balanced five-warmup/30-measured cohorts for mixed256, single-long2048, small2048 and article2048; controls originalA/originalB/pre-change serial/new serial/new two-worker pool. Full rotations/reversed blocks balance every position, all modes common CPU0,2/200%/3GiB/zero/nice10. Shaping timers off, layout phase timing enabled/common overhead. Per-row font/paint/complete-byte-set/fresh-stat validation, actual pool admission/scratch and scope counters. Direct paired frame/layout/cycle comparisons with signs, ranges/order strata, not differences of medians. No paragraph splitting for long.
- [ ] Width/edit/zero-retention/atomic smoke checks verify unchanged work and output on changed paths; run full final default/parallel package, consumer and wasm compile gates once after the selected source/fixes. Reuse prior WPT only with its old source bound, current WPT/runtime/MSRV/sanitizer UNTESTED.
- [ ] Record a task/phase report and bounded docs receipt. Do not reinterpret the old cold-equivalence condition. Evaluate whether the long-path copy still merits Task3 rather than claim victory from constructor counts or add another unmeasured tweak.

### Task 3: conditional shared-buffer diagnosis

This is a separate cut with a reviewed brief before implementation; no shared-buffer production change is preselected by this plan. If Task2 removes the relevant cost, omit unnecessary work and record why. If long-copy cost remains, audit local Parley buffer writes and quantify the clone in the long path, then produce an isolated copy-on-write/shared-buffer diagnostic. Tabs and justification mutate advances; preserve those exact semantics, source/caret/font output, caches/vector accounting, and caller-owned worker behavior. Compare complete outputs and bounded performance before considering adoption. No direct Arc replacement of mutable buffers without an ownership design. Supplier/public/API expansion or a production shared-buffer cut needs its own concrete reviewed brief; existing authorization permits this investigation, not silently weakened behavior.

**Done for this pass:** the borrowed path is correctly implemented and independently reviewed, targeted current-source effects are measured and documented, and shared-buffer follow-up is either justified with a reviewed diagnosis or omitted with evidence. The earlier whole-matrix cold goal is closed only if its explicit criterion is actually established or the user changes it.


## Local completion status, 2026-11-03

Tasks 1–3 are complete with independent task reviews and scoped fixes. Borrowed-validation production source is `5ba4d1fa282`; targeted qualification is in the canonical retained-shaping receipt. The separate shared-buffer diagnostic measured `463deba6b9a`, with test-only corrections through `00cf12c7d3a`. It supports a selective long-buffer follow-up design, while the small guard and controls argue against general adoption. Shared buffers remain experimental on their own branch. The prior whole-workload cold-equivalence goal remains open. The final broad review and local handoff are recorded in the preserved plan workspace.


## Upstream reconciliation, 2026-11-03

**Status: locally qualified, 2026-11-03.** The completed borrowed-validation pass is integrated with fetched upstream `70a54f6f43c` on a separate preserved candidate branch, `codex/shaping-upstream-20261103`.

Done-conditions: preserve the automatic merge and existing shaping runtime behavior; satisfy upstream's source-size rule without new exceptions; narrowly reconcile the declared package/pin changes; pass current-source Livery default/parallel, consumer, wasm and focused style gates; independently review the integration. Existing source-specific performance and runtime limits continue to apply.

Findings, 2026-11-03: upstream's changes since the shared base add RTC work, the genet-events release, docs and house-style checks, with no Livery/Buckram/Parley runtime edits. The new source-size rule applies to the shaping lane's cache, prepared-input and integration-test files. Their tests can move into smaller modules while keeping production bodies and the independent normalization oracle intact. Full-workspace offline metadata also reaches uncached RTC dependencies, beyond the selected layout qualification cone.

Progress, 2026-11-03: automatic merge `b150f229f7d`, test relocation `068c3fbd020` and the test-import guard `ed891681872` are committed locally. Independent source review found no actionable issue. Current-source layout, consumer, wasm and focused style qualification passed, including covering checks for the import-only correction. The existing canonical receipt holds commands, lock provenance, gate counts and resource evidence. Full workspace hooks need the unavailable cargo-deny tool and uncached RTC dependencies; current merged-source performance and broader runtime qualification remain open.
