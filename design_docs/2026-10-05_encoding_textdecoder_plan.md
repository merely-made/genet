# WHATWG Encoding: TextDecoder and TextEncoder

**Status:** native-lifetime checkpoint E1 answered; baseline preparation,
2026-10-05.
Census fixes are integrated; a separate Vano repin is being verified at
`d7f08fecdc7`. No Encoding implementation, dependency edge or baseline has
been changed. Mark authorized the private finalizer and Vano job queue path;
production changes wait for the frozen baseline and qualified failing controls.

Authority: the [standards ledger](2026-09-07_standards_to_features_ledger.md#conformance-targets-2026-10-02)
rulings C1, C2 and C4; the execution brief
`Code/work/briefs/2026-10-02_genet_encoding_textdecoder.md`; the
[WHATWG Encoding Standard](https://encoding.spec.whatwg.org/#interface-textdecoder)
and vendored WPT. The existing workspace dependency is `encoding_rs` 0.8.35.
Its [Decoder API](https://docs.rs/encoding_rs/0.8.35/encoding_rs/struct.Decoder.html)
provides stateful native decoding. No source is copied from another engine.

## Scope and done-conditions

This lane owns the TextDecoder/TextEncoder API on Window and Worker:
label resolution and canonical names, unknown/replacement RangeError,
fatal TypeError, BOM behavior, stream state and flush/reset, accepted
ArrayBuffer and ArrayBufferView inputs, and UTF-8 encodeInto counts and
character boundaries. Supported SharedArrayBuffer-backed views are included.
`read` counts UTF-16 code units; `written` counts UTF-8 destination bytes.
Astral code points and lone surrogates require explicit regressions.

Only `encoding_rs = { workspace = true }` may be added to script-runtime-api.
That crate is already locked; C2 does not authorize version or graph changes
beyond the direct workspace edge. URL query and form encoding in a document's
legacy charset belong to form submission. Legacy-charset iframe parsing and
TextDecoderStream/TextEncoderStream remain outside this lane. Shared UTF-8
helpers also serve Blob, Response and XHR; avoid changing those callers here.

## Phases

1. **Resolve the native-state checkpoint.** The brief fences fetch.rs edits to
   the encoder/decoder block and requires a checkpoint for a missing host seam.
   Record Mark's choice before native registration, host storage or adapter
   changes. Preserve the census R1 invariant: policy bookkeeping runs no author
   code and never rematerializes custom elements.
2. **Freeze a controlled baseline.** After Vano repin acceptance, freeze the
   release/netfetch runner, lock, source and WPT manifest. Measure the entire
   encoding directory on Boa and Vano, jobs 4, timeout 120, drive deadline 15,
   Livery, isolated workers and collection on. Classify the API subset separately
   and record which historical 30-second kills finish at these settings.
3. **Control and implement.** Add both-engine owning-runtime fixtures, then
   record their failures on unchanged production. Cover labels with whitespace
   and case, unknown/replacement labels, malformed fatal input, both BOM modes,
   multibyte stream splits/flush, view offsets, astral and lone-surrogate output,
   and partial destinations. Add the ruled native lifetime slice and narrow API
   implementation. Assert flush/reset release and collection/teardown lifetime.
4. **Accept and report.** Run script-runtime-api, genet-scripted and genet-wpt
   gates on the supported engines. Rebuild and freeze the final release runner,
   repeat the same WPT scope/settings, and match individual subtests including
   missing entries. Require API improvement on both engines and zero previously
   passing file or subtest losses across the whole directory. Attribute every
   movement, keeping timing and missing-feature outcomes separate. Commit and
   report at the brief's final checkpoint before further integration.

## Findings

- 2026-10-05: fetch.rs:850-870 remains the UTF-8 stub. TextDecoder ignores
  label and options, and encodeInto reports byte counts for read and can write
  part of a character. The shared utf8Encode helper also needs lone-surrogate
  care, but changing it would alter out-of-scope callers. Prefer an encoder
  helper inside the owned block.
- 2026-10-05: install_host_surface installs the same fetch bootstrap for Window,
  iframe realms and workers; worker_main owns and drops a separate Runtime.
  Native set_function registrations are in install_fetch_surface outside the
  fenced block. HostState provides per-realm native state ownership, but no
  decoder registry or arbitrary-object death hook.
- 2026-10-05: ScriptEngine dead/minted inventories are DOM-reflector-specific.
  Reusing fake node identities for decoder handles would cross that ownership
  boundary. No general native-resource finalization API exists today.
- 2026-10-05: both relevant engines contain FinalizationRegistry. Boa drains its
  cleanup jobs through run_jobs. Vano emits cleanup jobs, but GenetHostHooks
  currently inherits the default that ignores them. Its generic, promise and
  timeout jobs already use one queue, and the cleanup hook accepts that same
  Job type. This is a narrow adapter delivery gap rather than a missing VM
  primitive. Source inspection: Vano `47f8d4f9`, execution/agent.rs:424-449;
  Genet script-engine-nova/lib.rs:105-116 and pump; Boa `52cfb6ff`, finalization
  registry builtin and the adapter pump. No source is translated or copied.

## E1 native lifetime ruling, 2026-10-05

Mark selected "Private finalizer and Vano queue (Recommended)". Add a per-realm native decoder-handle map, with native
creation/decode/release sinks registered by the existing fetch installer.
Inside the trusted bootstrap, capture FinalizationRegistry and register, keep
only each opaque integer as the held value, and call a captured native release
sink from its trusted cleanup callback. Never capture the decoder target in
the held value or cleanup closure. Add Vano's missing cleanup-job override to
its existing queue. Cleanup occurs later during pump; it does not run in the
opaque-root policy. Flush releases decoder state immediately and subsequent
calls recreate it; realm/worker/runtime teardown drops leftovers. Releases are
idempotent and ids must not be recycled into stale finalizer callbacks.

The unselected alternative, if later needed for direct after-GC release without a JS cleanup job, is to add a generic engine weak-native-resource inventory/drain hook parallel to DOM
reflectors, implement it in both adapters, and drain it after force_gc. This is
a larger API change and needs a wider regression surface.

Both choices cross the brief's block fence. E1 explicitly authorizes native
registration/per-realm storage and the narrow Vano cleanup-queue override.
The selected option must prove collection and cleanup delivery on both engines,
worker and iframe lifetime, harmless author intrinsic replacement, no strong
reference from native state back to its wrapper, and R1 remaining intact.

Other checkpoints remain: specification/library disagreement; lock changes
beyond C2; and an excluded test becoming a dependency of this lane. Network
access is allowed by Mark's later instruction, superseding the brief's offline
and no-download rule while retaining dependency limits.

## Progress

- 2026-10-05: two read-only agents mapped the API ownership and native lifetime
  seams. Root verified the registration boundary and Vano Job-hook signature.
  The required boundary/lifetime question is pending. Existing census/Vano
  integration work continues independently; Encoding implementation is paused
  at this checkpoint. No Encoding worktree or target has been created.

- 2026-10-05: Mark answered E1 with the private finalizer and Vano queue option.
  Primary now has foreign image-decoding source WIP, in addition to its local
  Cargo overrides. Encoding is isolated at the brief's approved worktree and
  branch to preserve that actual collision; repin source remains frozen in the
  existing census checkout. The accepted repin lock is copied into the new
  checkout rather than importing primary's differing override resolution.

- 2026-10-05: Encoding checkout completed at `d7f08fecdc7` on
  `conformance/encoding`, without primary local patches. Portable lock SHA-256
  `070C2ABCC1F2C326BE560D4CCA053D5F526E65F872EC8ADF541818EAAC460907`;
  it differs from the preceding census lock only in three Vano git sources,
  not package versions or dependency edges. The plan/index/ledger edits were
  moved from primary with exact-block removal, preserving image-feature WIP.
  Owning-runtime fixture preparation is delegated separately from JS/native
  design; all production remains unchanged until failing controls are recorded.

- 2026-10-05: repin acceptance passed all 836 affected tests and paired WPT
  guards; exact repin `d7f08fecdc7` and receipt `2312c800c0c` are published.
  Encoding release/netfetch baseline is frozen from d7f, with runner SHA-256
  `A5E4B784C6550B7A18E7EED5F6082A39662AE05F2F35278E85EFE4A564FFBFA0`.
  Both full 1,313-record encoding runs are executing, same manifest/settings,
  engines in parallel. The 24 owning-runtime controls are compiling against
  unchanged production. No native code or workspace edge has changed yet.
  The reproducible API selector is saved in external evidence; it yields
  67 historical records/12,587 reported subtests, not the brief's unrecovered
  101/13,111 grouping. Fresh counts will use that named selector, including
  supported any-global variants, rather than equating different groupings.
  Native IDs will use a checked crate-wide monotonic allocator: snapshots
  retain JS associations while replacing HostState, so a per-host counter
  would allow stale cleanup callbacks to release unrelated new entries.

- 2026-10-05: both immutable baseline runs completed. Whole directory: 1,313
  records each; Boa 5 passing files and 10,577/1,333,564 passing reported
  subtests, Vano 6 passing files and 10,901/1,333,612. Named API subset: 67
  records each; Boa 4 passing files and 10,576/15,749 subtests, Vano 5 passing
  files and 10,900/16,123. Counts retain timeout/not-run/missing distinctions;
  they are not percentages of implemented Encoding behavior. External maps,
  counts and exact provenance are retained under encoding-textdecoder.
- 2026-10-05: owning-runtime controls ran against unchanged d7f production:
  24 tests, 23 expected assertion failures and the Boa finalizer control pass.
  Vano's finalizer queue control fails as expected. Each API behavior group
  fails on both engines. The fixture, SHA and log are frozen externally.
- 2026-10-05: adding only C2's direct workspace edge makes locked metadata
  reject the old ignored lock (exit 101). Source lock is unchanged. The exact
  proposed patch adds encoding_rs to script-runtime-api's dependencies only;
  all 867 package identities, versions and sources remain unchanged. Mark's
  required lock checkpoint is pending; implementation remains paused.
- 2026-10-05: cross-realm brands can share private WeakMap/WeakSet state through
  the existing per-agent __agentTimers value, with each decoder retaining its
  captured creation-realm native decode callable. Metadata never stores its
  wrapper. This avoids new engine API and preserves borrowed methods/getters.
  Navigable retirement does not itself clear retained wrappers' native state;
  actual HostState/runtime destruction is the teardown backstop. Flush frees
  the active decoder and pending byte buffer, retaining only reusable metadata.

- 2026-10-05: expanded baseline fixtures qualified on unchanged production:
  30 tests, 27 expected failures and 3 passing regression controls. The added
  cross-realm tests expose the missing prototype getter and lost streaming
  state after iframe retirement; the two captured public intrinsic controls
  pass already and guard against regression. Fixture SHA and log are frozen
  externally. The C2 manifest edit was temporarily removed for this locked
  baseline run; the ignored lock remains unchanged pending Mark's answer.
- 2026-10-05: the recovered historical map has 816 error records, all outside
  the canonical API selector. Fresh Boa reports 800 FAIL and 16 no-results;
  fresh Vano reports 798 FAIL, 16 no-results and 2 errors for those records.
  That separates reporting/speed changes from implementation work. The brief's
  two killed API files are not recoverable from this map; they are not equated
  with the two fresh out-of-scope Vano errors. The named API matrices now have
  ordinary records for every query variant, with missing subtests explicit.
- 2026-10-05: comparison helper positive controls pass for unchanged records,
  pass loss, missing records and duplicate-name occurrence identity. Actual
  after-run attribution remains pending implementation and matched runs.

## C2 lock checkpoint, 2026-10-05

Mark approved the exact one-edge ignored-lock update with "Approved, continue".
The patch adds encoding_rs to script-runtime-api's dependency row only. All 867
packages, versions, source revisions, checksums and other dependency edges are
unchanged. Locked metadata now succeeds. Accepted lock SHA-256:
`9B7C50EA33A181C424E871129A0CFB071CC8B65EE5BCECCD42CCD656118423D9`.
The original/proposed/accepted locks, metadata and graph-delta receipt remain in
external evidence. Phase 2 is open under E1 and C2; remaining checkpoints persist.

- 2026-10-05: Phase 2 implementation is assembled. The new native module owns
  per-realm decoder metadata and streaming state; process-wide checked IDs
  prevent stale cleanup aliasing after HostState resets. The JS block captures
  intrinsics and native callables, shares weak receiver brands across agent
  realms, and registers ID-only finalizers. Vano cleanup jobs now enter its
  existing queue. Native state holds no JS wrappers, and the R1 GC-policy code
  is unchanged. No source is imported or translated from another engine.
- 2026-10-05: root review corrected draft astral UTF-8 conversion, primitive
  dictionary conversion, required-argument ordering, nonconstructible getters,
  decoder reset borrowing and fatal-read accounting before runtime acceptance.
  The first compile caught an undefined-value API-name error; it was fixed
  using the existing CallCx::undefined contract. The failed log and original
  source are retained. A second focused gate is compiling.
- 2026-10-05: independent read-only review found no actionable issues. An exact
  outer-bootstrap comparison confirms unchanged code outside the assigned
  Encoding block plus the authorized registration/helper-visibility edits.
  Shared Blob/Response/XHR UTF-8 helpers are byte-for-byte unchanged. This is
  review evidence, not a passing implementation or WPT receipt.

- 2026-10-05: the focused gate passes all 164 runtime unit tests, including
  existing census R1 controls, decoder collection/flush/teardown, and snapshot
  stale-handle protection. It passes 30/32 Encoding integration controls. Both
  Worker failures were traced to the fixture's relative loader key while
  Worker resolves an absolute URL. The loader key is corrected and an explicit
  script-error assertion added; the next gate is running. The original Worker
  before failures are unqualified as Encoding behavior controls, because the
  script did not load. Window behavior controls and full WPT baseline remain
  qualified. All failed runs remain in evidence.

- 2026-10-05: a bounded read-only check found no specification/library mismatch
  in the GB18030-2022 cases. encoding_rs 0.8.35 records 2022 support; the relevant
  two-byte tables and four-byte range mappings match all listed updated WPT
  examples inspected. This qualifies those examples only; full API acceptance
  still requires the actual matched WPT run.

- 2026-10-05: all 32 Encoding integration controls now pass, including actual
  Boa/Vano worker scripts with corrected absolute fixture routing. The prior
  164-unit run also passed. Root is committing the reviewed implementation
  before broad crate gates and frozen post-change WPT acceptance. This is a
  candidate commit, not the lane's final acceptance or publication.
