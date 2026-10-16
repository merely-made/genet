# Genet architecture and removal audit

## W1/W2 repair and verification, 2026-10-16

**W1 and W2 are repaired in the retained native shader path.** This follow-up
supersedes their open status in the historical post-cut snapshot below. Exact
source hashes, command, adapter and logs are in the `linked_shader_repair` entry
of the [audit receipt](receipts/2026-10-16_architecture_post_cut_audit.json).

The ESSL frontend owns a linked plan for varying locations and uniform member
order/type agreement. Both stages emit from that plan. Naga parses and validates
the emitted WGSL inside the guarded worker, supplies offsets, sizes, matrix
column strides and block span, and verifies the stages agree. CPU uploads consume
those facts. The backend's independent offset/alignment/size table is removed;
mat3/mat4 serialization uses reflected strides. Source-key lookup no longer
recompiles shaders. The wider I11 compiled-artifact lifecycle remains open.

The final offline, locked test run passed **492 frontend tests, 78 backend library
tests and 12 backend integration tests: 582 passed, zero failed or ignored**.
Actual draw/readback regressions passed on **AMD Radeon Graphics (RADV RENOIR),
Vulkan** for reordered varying declarations, shared/reordered and disjoint
uniforms read by both stages, the scalar after a mat2, and reflected mat3 column
strides. Conflict rejection, unused outputs, and unused uniforms have regressions.
Existing texture, framebuffer, matrix and compile/validation tests also passed.

Independent review found an unused-bool member-index regression in the draft;
it was corrected and covered before the final gate. The existing interface-error
info-log contract was preserved after a test failure. Final scoped review approved
the implementation. Builds held the shared lock with one job, serial tests,
5 GiB memory maximum, zero swap allowance and 150% CPU quota. The final run took
21.86 seconds and reported zero swaps; the observed cgroup peak during the run
was approximately 767 MiB. A disposable 17 MiB probe executable was removed,
while its source/results and shared build caches were preserved.

This establishes the named native retained-path behavior. Browser/wasm, other GPU
backends, full WebGL conformance, unrelated compiler cleanup and Forms acceptance
remain outside this result. Other audit findings retain their previous status.


## Post-cut re-audit, 2026-10-16

**Current source: `6ab57c124e1`. The bounded cuts are implemented and focused tests passed. The broader architecture is not repaired or qualified.** This section supersedes the original snapshot's status claims below; the original evidence remains historical.

The fresh review covered the production changes, their consumers and regression boundaries, then revisited the retained implementations behind the finding register. The complete 996-file core inventory was reconciled by hash: **968 unchanged, 23 modified, 5 removed**, with **5 new core files**. Workspace metadata still reports **52 packages**. Source identity is not semantic review: unchanged files retain their original review depth, and the new receipt explicitly lists the files receiving fresh targeted reads. This is a repository-wide architectural re-audit with selective body reads, not an every-line security or conformance audit. Unchanged upstream vendor bodies and external historical runs were not independently rereviewed.

[Post-cut source and probe receipt](receipts/2026-10-16_architecture_post_cut_audit.json). [Retained-code test receipt](receipts/2026-10-16_architecture_reduction_tests.json).

### Disposition of the original findings

| Disposition | IDs | What the retained source establishes |
|---|---|---|
| Named source defect/cut addressed; qualification limits below still apply | H1, H4, H7, H8, H11, I12, I14, I15, L2 | Removed GL stub and deprecated compositor alias; browser scroll defaults consume input outcomes; canonical dependency gate fails on resolution errors; native PowerShell commands are checked; two stale implementation comments corrected; general shader reflection replaces positional fields; orphan media-thread and obsolete launchers removed; prospective break ledger is test-only. |
| Partial migration | I1 | Native host calls DOM-owned text editing, including current eligibility and selection collapse. Scripted key input still ignores key releases, consumes Tab before page dispatch, and does not use this editor. Text/IME/editable-focus defaults remain unwired there. |
| Partial migration | H6 | Setup/execution/completion are separated; async modules use the module route; completion is privately retained; failures cannot satisfy negatives merely through diagnostic text. Actual VM errors still lack trustworthy phase/type, so negative tests report Unsupported. This is safer accounting, not complete Test262 qualification. |
| Partial migration | L6 | Form/radio predicates and affected-owner walks are shared. Style invalidation and formatting damage retain their own mutation switches and root mapping. Broader DOM dependency classification has not been consolidated. |
| Still open | I2-I11, I13; R1-R15; L1, L3-L5, L7-L13; H2-H3, H5, H9-H10; W1 | Retained implementations still exhibit the recorded mechanisms. No closure follows from unrelated passing tests or removal counts. |

That is **9 addressed source findings, 3 partial, and 43 still open**, plus new W2 below. Supplemental X1 is partial: the old environment script is removed, while the debugger helper remains. No platform cut was made to H9's Wayland synchronization path.

H4 is source-addressed, but the browser callback has not received a fresh wasm/browser interaction test. H8's earlier Windows fake-command result remains dated evidence; it was not rerun on Linux. I12 removes a genuine unused projection and rejection; it does not establish coherent shader linking. The original text gave conflicting ordering for I12 (safe narrow cut versus W1 prerequisite). The precise boundary is: unused-field cleanup may stand independently, while claims of correct linking or subsequent layout optimization require W1/W2 to be resolved.

### W2: CPU uniform layout disagrees with emitted WGSL within one stage

**High priority; isolated current-translator reproduction.** This is additional to W1's cross-stage mismatch and existed before the cuts.

`UniformKind::Matrix2` reserves 32 bytes with 16-byte alignment in [shader.rs](../components/webgl-wgpu/shader.rs). The frontend lowers `mat2` with 8-byte columns and a 16-byte total in [lower.rs](../components/webgl-essl/lower.rs). The CPU reflection independently computes offsets in [shader/lowering.rs](../components/webgl-wgpu/shader/lowering.rs), and [webgl/programs.rs](../components/webgl-wgpu/webgl/programs.rs) writes values at those reflected offsets.

A vertex shader with no uniforms and this fragment shader are accepted by the current translator:

```glsl
precision mediump float;
uniform mat2 u_matrix;
uniform float u_alpha;
void main() { gl_FragColor = vec4(u_matrix[0], u_alpha, 1.0); }
```

The CPU reflection places `u_alpha` at byte **32**. Parsing the actual emitted WGSL with the same built Naga reports its second member at byte **16**. Thus a valid scalar write goes to a different byte range from the shader read. This discrepancy does not require disjoint stage uniforms or a matrix upload API. A GPU draw was not run, so the receipt establishes translation/layout disagreement, not a captured pixel outcome.

A control with `vec3 u_color; float u_alpha;` **does agree at byte 12** after WGSL emission. The intermediate SPIR-V offset alone looked suspicious but did not survive emission. Do not file that control as another defect.

The retained probe compiles the current shader module and its production lowering against the already-built frontend; the only copied-source edit is an explicit module path needed by the standalone probe. Its source, hashes, emitted WGSL and Naga layouts are recorded in the receipt and external audit directory. Repair W1/W2 through an authoritative emitted layout/interface contract; copying another table of offsets into the CPU writer would leave the ownership problem intact.

### Retained production blockers rechecked

- **Input and scheduling (I1/I2/H3):** `engines/scripted.rs:387-408` still has no text editing or keyup route. `genet-scripted/document.rs:1434-1463` pumps timers, workers and microtasks but neither runs nor schedules rAF-only work. WPT's separate callback invocation remains in `harness.rs:903`. A working DOM edit helper is not live scripted input parity.
- **Resources and observation (I3-I5/L3):** `synchronize_live_styles` still resolves the whole resource set on a new mutation suffix; image maps still insert authored aliases; capture still drains the rendering journal. Paint still clones encoded source maps and decodes into each new list. These are retained operations, not inferred performance measurements.
- **Execution/error ownership (R4-R8/R13-R15):** timer errors still become zero work; parser source/evaluation failures are discarded; module resolution ignores the supplied referrer; bounded eval defaults to unbounded eval; worker shutdown detaches unfinished threads. Resolver cleanup is still manual after lifetime erasure. Registration leaks and the synthetic elapsed clock remain. The unwind reuse issue remains a source-derived risk, not a reproduced unsafe dereference.
- **Origin/network policy (I7-I9/H5):** invalid explicit ports can still collapse into absent ports; enabled cache accumulation has no decoded-byte counter; native Ortet still buffers `response.bytes()` without its advertised host budget. Cookie storage still omits public-suffix/prefix checks. SameSite retrieval does have code despite the cookie module's stale introductory comment; do not call the whole cookie policy absent.
- **Parsing and projections (R1-R3/R9-R12/L1/L4-L5):** private selector parsing/count-item traversal, semicolon splitting, quote-blind comment stripping, no-factory context objects, mutable WebGL brands, lossy XPath/history adapters, transform-context disagreement and estimated geometry remain. Their existing preservation boundaries still apply.
- **Layout and presentation (L7-L13/H2/H9-H10):** retained plane clones, eager aggregate rebuilding, real Taffy fallback, prospective fragmentation, duplicated axis vocabulary and historical census remain. Font diagnostics still allocate before capture admission. Browser renderer access can still panic on checked refusal; Wayland still allocates its dormant timeline and logs presentation errors. Existing platform contracts and ignored continuation fixtures must remain.

The review did not establish a new regression caused by the narrow removals. That statement is limited to the inspected paths. W2 is a newly established pre-existing defect, and the passing shader tests did not cover it.

### Test boundary and next work

Final retained-code results: Livery library **334 passed / 2 existing ignored**; focused shader translation **19 passed**; DOM editing **13 passed**; Test262 policy **9 passed / 1 existing ignored**; private-completion JavaScript regression passed; dependency-gate regressions **6 passed**. Cargo check passed for genet-wpt and servo-webgl-wgpu and their selected dependencies. The DOM-edit run and W2/control probe were added during this review. Tests were not extended for mechanical helper movement.

These results do not establish GPU pixels, browser input, Windows presentation, matched engine comparisons, Forms Checkpoint C, or complete Test262 negatives. The next correctness work is W1/W2 for graphics and I1/I2/H3 for live Forms; origin/cache/worker bounds remain independent serious obligations. There is no basis to delete retained owners wholesale or to call the twelve-lane plan complete.

---

## Original source audit at `8fa73497952`

**Status: repository-wide structural and ownership review, 2026-10-16. Implementation and runtime qualification are separate and unperformed.**

Source snapshot: `8fa73497952ecda37cf234c4a845354dab1af599`. The two local Forms repair commits are included. The cached remote reference was `6186c81ad70`; this audit did not fetch, publish, or change that state.

## Judgment

Genet has substantial real implementations and useful ownership boundaries. Its recurring architectural failure is that a consumer reconstructs facts already owned elsewhere, then grows its own parser, projection, scheduler, compatibility record or test adapter. The copies drift. Several successful-looking APIs also hide missing capabilities, ignored limits or failed work. Those two patterns make local repairs accumulate without establishing the behavior a browser user needs.

Forms sits on that foundation. Native current-value, selection, validation, successful-control construction and encoding are useful owners to preserve. Scripted physical input and live scheduling do not share enough of the tested/native path. Adding submission cases cannot establish those missing paths. The next Forms work should remove those discrepancies, then use matched evidence to qualify the resulting path.

This report covers all workspace package families, nonmember maintained patches, orphan code, name reservations, hosts, runners and support machinery. It has no finding quota. Detailed findings below include specific removal targets, preservation boundaries and validation obligations, including findings against the evidence-producing tools themselves. A file inventory is explicitly distinguished from a semantic read. This is not an assertion that every source line or every browser/security requirement has been audited.

## Reading map

- [Scope and evidence](#scope-and-evidence)
- [What the repair commits establish](#what-the-repair-commits-establish)
- [Ownership decisions](#ownership-decisions)
- [Consolidated finding register](#consolidated-finding-register)
- [Removal sequence and done conditions](#removal-sequence-and-done-conditions)
- [Coverage by workspace package](#coverage-by-workspace-package)
- [Integration evidence](#integration-evidence)
- [Runtime and DOM evidence](#runtime-and-dom-evidence)
- [Layout, style and text evidence](#layout-style-and-text-evidence)
- [Host, rendering and tooling evidence](#host-rendering-and-tooling-evidence)
- [Independent shader-link and remaining-domain review](#independent-shader-link-and-remaining-domain-review)

## Scope and evidence

The tracked source/configuration inventory contains **996 files** under `components`, `ports`, `support`, `scripts`, `.cargo` and `.github`, selected by Rust, JS, Python, PowerShell, shell and TOML extensions. Their hashes and mechanical symbol/risk indexes were recorded. The merged review depth is **53 full-file reads, 529 targeted reads, and 414 inventory-only files**. “Targeted” means module structure and selected relevant bodies/contracts were traced; it does not mean every line was reviewed. Generated tables, tests and unchanged upstream bulk account for substantial inventory-only coverage. Every package family has an ownership disposition; an inventoried file has no implied correctness clearance.

Offline locked Cargo metadata reports **52 workspace packages**. The core inventory contains **69 tracked Cargo manifests**, including nonmember patches, publishing reservations and the orphan media thread. The repository has **77 tracked Cargo manifests** in total: the additional eight are the root workspace manifest, one standalone example and six historical receipt/probe manifests. Maintained vendor review is primarily activation, provenance and local delta review. It does not claim an independent audit of all upstream implementation. No upstream release-status lookup was made.

A separate **799-file supplemental inventory** covers workflows and fixtures/data outside the vendored `tests/wpt` tree. This includes 712 inherited test assets, of which 522 belong to the Blink performance corpus. Four inherited Python tools and the Blink import shell script were separately inspected. Root policy/configuration, the standalone example and the remaining third-party debugger helper were also checked as supplemental surfaces. Upstream WPT/Test262/WebGL corpora, general dependency internals, sibling applications and all historical prose are not independently audited line by line. Sibling sources were searched to verify actual consumers of proposed cuts.

External Forms machinery was also inventoried: **2,409 files, 255,556,936 bytes, 234 top-level files and 73 top-level Python scripts** in the October 16 Windows run directory. Those are artifact counts, not counts of active programs or individually reviewed scripts. Representative build/capture/control/provenance paths were traced. Historical receipts and failed/interrupted runs remain evidence to preserve.

[The per-file coverage receipt](receipts/2026-10-16_architecture_removal_audit.json) records exact source hashes, review depth and package counts. Detailed working inventories and the isolated probe output are at `/home/markik/Code/testing/audits/genet-architecture-20261016/`.

Evidence levels used throughout:

| Evidence | What it supports |
|---|---|
| Source-confirmed | A traced implementation fact, missing route, duplicate operation or ignored result. It does not by itself prove a user-visible exploit or measured performance cost. |
| Isolated reproduction | A small extracted helper was executed. It establishes that helper's behavior, not integrated caller reachability. |
| Source-derived counterexample | Concrete inputs expose incompatible ownership/layout from code inspection; the integrated draw, browser interaction or network route has not been executed. |
| Candidate cut | Source/consumer evidence supports removal, subject to the stated migration or platform proof. |
| Retain | A real consumer, invariant, distinct lifetime or preservation contract defeats the proposed broad deletion. This is not a conformance certificate. |

No product build, Cargo test suite, browser qualification, hardware run or dependency download occurred. `cargo metadata --no-deps --offline --locked` established membership without compiling. An extracted Origin helper was compiled and run in isolation: an invalid explicit port collapsed to the clean HTTPS tuple; backslash and encoded-host examples showed the handwritten parser's distinct interpretation. The layout review also used an isolated comment-stripper probe. Neither probe qualifies the product. Existing receipts remain dated historical results, not fresh passes at this audit snapshot.

## What the repair commits establish

`e3911fb6b2d` contains some genuine consolidation: validation message selection moved into the native DOM owner, both sessions share alert projection construction, and submitter capture is distinguished from later listener-mutated state. The private form bridge retains genuine owned-node extraction and private bootstrap capture. Calling every private bridge “stringly” would miss those useful identity guarantees. Its serialized records and extra bridge orchestration still need the broader boundary review described in R3/R4 and I6.

`8fa73497952` makes concrete local corrections: it releases a `RefCell` host borrow before synchronous focus callbacks, renders the still-active source after a submission effect, and checks navigation generation before preparing/fetching dependencies from a superseded response. These corrections address real failure mechanisms visible in the diff. They do not establish complete input, cancellation, scheduling or resource ownership.

The recorded focused checks, live submissions and Chrome cases support their named cases. The unmatched Boa submission captures do not establish candidate/control changes; the unfinished control runner, matched engine comparisons, encoding captures, remaining regressions and Checkpoint C remain unfinished. This audit did not rerun or invalidate those unrelated checks by association. In particular, the Test262 classification defect below is **not** evidence that Forms' testharness results use the same broken classifier.

The defensible disposition is to retain the semantic corrections and repros, consolidate the surrounding ownership, and requalify the changed path. A blanket rollback is not justified by this audit. Neither is defending these commits as a reliable or completed Forms foundation.

Relevant existing evidence: [review receipt](receipts/2026-10-16_forms_submission_review_windows.json), [native/browser receipt](receipts/2026-10-16_forms_native_browser_windows.json), and [Forms plan](2026-10-07_forms_value_validation_submission_plan.md). Their acceptance status is not changed here.

## Ownership decisions

| Family | Required owner and disposition |
|---|---|
| Native DOM, forms and arena | Keep native values, validity, immutable files, full-width IDs, adoption/liveness, shadow policy and shared encoders. Consumers should request these facts. Do not reconstruct them from attributes or author-readable properties. |
| VM backends and runtime | Keep backend-neutral identity/realm/GC boundaries and separate worker heaps. Replace falsely bounded operations, detached lifetime claims, manual resolver lifetime restoration, leaked registrations and source-eval internal dispatch. Piccolo remains an opt-in Lua adapter, not a JS browser backend. |
| DOM query and CSS | Reuse shared selector/token parsing through an adequate live-DOM adapter. Remove private limited grammars, split-semicolon parsing and quote-blind comment stripping. Preserve DOM scope/errors and CSSOM ordering/priority/custom-property semantics. |
| Document/session scheduling and input | Share lifecycle, task/default-action and pending-work facts between live/headless/native/scripted consumers. Keep host event translation and deterministic test clocks distinct. Route real editing and rAF through the production path. |
| Resource acquisition and network | Retain protocol-agnostic Fetch policy and Murm carriage separation. Own URL/origin/site identity canonically, retain resource dependencies and source identities, enforce actual byte/deadline/admission policy. Remove wholesale reacquisition and ambiguous aliases. |
| Style, layout, text and paint | Keep Cadency selectors, Livery style, Buckram formatting, Parley shaping and actual retained geometry. Remove duplicate lexical preprocessing, unused prospective hot-path work, payload copies and shadow geometry interpretation. Table diagnostics and backend fallback remain real obligations. |
| Rendering and platform hosts | Keep the shared device, checked leases/recovery, readback attribution and real OS export consumers. Remove the no-op GL capability. Migrate panic compatibility access and report actual presentation outcomes. Gate platform cuts on their stated proof. |
| WebGL compiler/backend | Keep lexer/AST/check/validate/lower phases and wgpu resource ownership. Link stage interfaces and uniform layout coherently, retain compiled artifacts, remove narrow reflection and source-text shader surgery. A shared pair layout is missing. |
| Accessibility and automation | Keep generated text/name projection and native accessibility custody. Make Taproot consume owner semantics and retained geometry before removing its shadow implementation. Raw DOM queries may remain a deliberately different contract. |
| Fleece, text and clipboard | Keep render-free extraction with raw JSON-LD evidence, explicit Unicode coordinate ownership, and actual system/memory clipboard service. No broad removal justified. Stale comments do not prove missing implementation. |
| Media | Keep active audio/playback/streams/WebRTC and platform backend contracts. The old nonworkspace media thread and unused root alias are a narrow removal candidate. A wgpu browser renderer does not alone eliminate video interchange contracts. |
| Maintained forks | Retain documented Taffy/Parley/Fontique semantic fixes and necessary IPC, regex, allocation-version and Wasm RNG graph changes until their precise retirement gates pass. Unchanged upstream bulk is not a duplicate Genet owner. |
| Qualification and support | Keep assertion identity, negative controls, source hashes, immutable artifacts and strict missing/error outcomes. Correct false-pass paths and failed-command continuation. Consolidate repeated launching/provenance functions after their individual gates remain observable. Retire obsolete entrypoints. Name reservations and generated IDL shapes have distinct purposes. |

## Consolidated finding register

IDs match the detailed evidence sections. These are issues and removal decisions, not a count of independently reproduced bugs. Related rows share a repair boundary rather than requiring separate frameworks. W1 is the independent review's shader-link finding; the remaining independent observations corroborate I findings and are not counted twice.

| ID | Finding | Disposition / dependency |
|---|---|---|
| I1 | Scripted session lacks native text/IME/default-action integration; Tab is consumed before page keydown | Share actual editable-control defaults and input outcomes; live typing proof precedes Forms acceptance. |
| I2 | Headless lifecycle tests and live lifecycle state have different owners | Share lifecycle facts; wire freeze/resume before advertising live support. |
| I3 | Generic DOM synchronization re-resolves all document resources | Retain dependency-aware acquisition results; remove whole-resource resolution from unrelated mutations/style reads. |
| I4 | Authored image aliases collapse different stylesheet bases | Carry resolved source identity into computed image/paint keys. |
| I5 | Capture drains rendering's mutation journal | One consumption owner or non-stealing observation cursors; preserve historical capture format. |
| I6 | Request and response compatibility representations duplicate and discard metadata | Migrate actual callers to a shared host contract; retain transport/VM separation. |
| I7 | Handwritten origin grammar normalizes malformed URLs inconsistently | Derive tuples from canonical parsed URLs; preserve opaque/inherited origin policy. |
| I8 | Cache limit checks Content-Length but not accumulated decoded bytes | Stop cache accumulation at configured streamed-byte cap; preserve body delivery. |
| I9 | Default cookie jar defers public-suffix and prefix restrictions | Complete or explicitly constrain one policy owner; keep injected storage and SameSite context. |
| I10 | Taproot reconstructs names/roles and estimated geometry | Migrate live consumers to owner projection/retained layout, then delete shadow semantics. |
| I11 | Shader compilation repeats for validation, cache-key discovery and translation | Retain compiled shader artifacts under source/stage/options identity. |
| I12 | Obsolete narrow shader reflection still rejects input sets | Remove unused compatibility fields and positional inference; preserve general reflection. |
| I13 | Raw shader text replacement and precision scanning duplicate compiler grammar | Move supported normalization/reflection into AST ownership. |
| I14 | Orphan nonworkspace media thread and unused dependency alias remain | Remove after final consumer/API check; preserve history and active media family. |
| I15 | Inherited Servo benchmark launchers remain as obsolete gates | Retire/archive launchers and instructions; preserve useful imported assets/licenses. |
| R1 | Runtime selector grammar silently narrows selectors and hides parse errors | Shared selector parser with faithful live-DOM adapter and explicit invalid-list outcome. |
| R2 | Count/item queries repeat complete selection for every item | One selection with static membership, stable identity and bounded result lifetime. |
| R3 | JS splits CSS declarations and mirrors a private Rust record codec | Whole-list parse in CSS owner; adequate structured boundary or one specified codec. |
| R4 | Internal task source-eval and result coercion can erase errors/work | Retained private callable entrypoints with realm/GC ownership and explicit outcomes. |
| R5 | Parsed script loading/evaluation errors disappear behind scheduling counters | Typed nonfatal execution results and proper exception reporting. |
| R6 | Parser module resolver drops referrer identity | Resolve/cache using actual importer/final URL and inline document base. |
| R7 | Worker shutdown detaches uninterruptible VM execution | Enforceable interruption/isolation; truthful lifetime and configurable shutdown policy. |
| R8 | Budget-shaped API defaults ignore limits | Separate execution interruption from job count; refuse unsupported required bounds. |
| R9 | Missing WebGL factory manufactures a context-shaped capability | Return unsupported/null when actual graphics capability is absent. |
| R10 | WebGL objects use public mutable identity and permissive text coercion | Private context-owned brands and genuine value extraction. |
| R11 | XPath adapter loses attributes, namespaces, mode and detached context | Replace lossy live adapter; preserve language core and real result identity. |
| R12 | History uses JSON state despite existing structured clone; fallback duplicates history | Share serialization semantics, then migrate fallback owner deliberately. |
| R13 | Module resolver lifetimes are erased and manually restored without unwind guard | Owned/scoped resolver context and unwind-safe restoration; preserve GC roots. |
| R14 | Ordinary engine/realm registration leaks process-lifetime allocations | Proper owned registrations or fixed static names; keep required rooted release queues. |
| R15 | Product elapsed-time API uses timer-driven test-like clock | Explicit host elapsed clock and separate deterministic test profile. |
| L1 | Two comment strippers remove comment-like text inside quoted CSS strings | Token-aware CSS lexical authority; isolated probe confirms helper corruption. |
| L2 | Prospective line-break stream runs while Parley still selects lines | Remove unused production work or gate it as explicit experimental diagnostics. |
| L3 | Paint clones source image payloads and decodes repeatedly | Share versioned resource payload/decoded ownership; preserve invalidation. |
| L4 | Containing-block transform facts disagree with paint transforms | Shared facts with rule-specific predicates, not one indiscriminate boolean. |
| L5 | Public estimated geometry diverges from retained text/resource geometry | Use live retained owner for observable geometry; identify estimates explicitly. |
| L6 | Style invalidation and layout damage separately classify DOM dependencies | Share affected semantic owners; retain different style/formatting root outputs. |
| L7 | Local retained rendering clones whole style/fragment planes | Borrow/overlay immutable generations while preserving rollback/isolation. |
| L8 | Per-table ledger publication rebuilds aggregate diagnostics eagerly | Aggregate lazily or by deltas; keep actual divergence/ownership evidence. |
| L9 | Buckram/backend styles and genuine block fallback debt overlap | Narrow conversion/ownership after accepted contracts replace fallback; retain real flex/grid/backend functions. |
| L10 | Prospective fragmentation kernel coexists with live unsupported markers | Feature-gate prospective code; retain explicit unsupported results and six continuation repros. |
| L11 | Logical-axis facts repeat across value/geometry layers | Share small vocabulary if dependency direction permits; otherwise parity witnesses. |
| L12 | Historical property census is mistaken for current runtime consumption | Label historical or generate/check against current readers; retain honest unsupported classification. |
| L13 | Feature-enabled font diagnostics construct events before capture check | Avoid discarded collection; keep deterministic font evidence and RAII scope. |
| H1 | Unused GL capability/handle stubs claim success without work | Delete stub surface after API consumer check; preserve real wgpu Core capability. |
| H2 | Browser compatibility renderer access bypasses checked ownership | Migrate to browser frame transaction; preserve leases/loss/scope outcomes. |
| H3 | WPT runs animation callbacks that live session pumps omit | Share production turn contract and pending-work wake facts; preserve virtual clock. |
| H4 | Browser key route discards typed input outcome and performs scroll defaults | Gate defaults using handled/editable/cancelled outcome, as native host does. |
| H5 | Native fetch ignores declared byte/deadline/concurrency policy | Wire configurable policy through counted streaming/admission/deadline enforcement. |
| H6 | Test262 negative/module/async paths can turn setup/wrong failures into passes | Typed setup/execution/completion and shared exact negative matching; combine flags correctly. |
| H7 | Old profile gates turn cargo failure into empty clean result | Retire obsolete entrance; strengthen canonical CI cone witness with failing controls. |
| H8 | PowerShell wrappers can continue after native build/bindgen failure | Checked native-command helper and environment restoration; stale outputs must not imply success. |
| H9 | Dormant Wayland timeline allocation can reject a same-queue route | Conditional cut after queue/release audit and Wayland hardware proof. |
| H10 | Compositor failures/logged capture fallback lack observable presentation outcome | Typed host-visible display/capture/failure status; preserve real OS consumers. |
| H11 | Live implementation contains stale proposal/lineage comments | Correct current scope; keep historical receipts as history. |
| W1 | Independently lowered stages disagree on varying locations and uniform layout | Coherent pair interface/layout before final lowering, or correct distinct stage bindings. Source-derived counterexamples; GPU draw unexecuted. |

Smaller observations remain explicit in the appendices: scheduler trace schema omits the emitted animation-frame boundary; clipboard and random-source comments are stale; invalid custom-method fallback and recursive Fleece input limits remain bounded review questions rather than asserted integrated failures. They are not hidden acceptance passes or reasons for broad deletion.

## Removal sequence and done conditions

This is an implementation disposition, not authorization to delete every candidate blindly. Existing tests and receipt provenance survive migration. Each stage is done when the stated old authority has disappeared from real callers and its preserved behavior is observable through the replacement. Code size alone is not a done-condition.

### Establish trustworthy results and capability claims

Correct H6/H7/H8 before relying on their affected counts or launchers. Distinguish setup failure, unsupported capability, script exception, timeout, completion, assertion failure and pass. Exercise deliberately broken harness/native commands with old artifacts present: they must fail, produce no success marker and avoid launching stale output. Preserve old raw runs; do not rewrite them into passes. Keep unrelated Forms testharness evidence separate from Test262's classifier.

R7/R8/R9 and H10 require truthful unsupported/bounded/presentation outcomes. A limit must stop or refuse the work it claims to bound. An unsupported context must fail detection. Capture fallback must remain distinguishable from visible presentation. Worker hangs require an external kill boundary for qualification.

### Make narrow dead cuts

After direct consumer/API checks, remove H1's no-op GL surface, I12's unused narrow reflection fields/rejection, I14's orphan media thread/alias, and obsolete I15/H7 launch entrypoints. X1 adds a historical environment script and unused-in-current-entrypoints debugger helper, with the stated external-consumer check. Remove L2's unused prospective break-stream work from the production hot path. Keep the actual line-break selection, general shader reflection, active media backends, fixture licenses and archival provenance. These cuts should not introduce a replacement framework.

H9 is conditional platform work, not part of this safe source-only batch. L10 must preserve explicit unsupported state rather than pretend deleting a future kernel delivers fragmentation.

### Repair the production path Forms depends on

Unify I1/I2/H3/H4 input/lifecycle/scheduling facts. A single control fixture must accept physical typing, selection, IME, focus, key release and script-cancelled defaults through native and scripted production sessions. A document with only rAF work must wake and run the callback in the correct turn; hidden and frozen states have distinct effects. The deterministic harness drives this same semantic contract with its own clock. Retain settle policy that does not wait forever for perpetual animation.

Migrate R4/R5/R6 and I6 using actual engine callable/identity capabilities. Do not invent a large generic value framework merely because string records exist. Begin with retained private entrypoints and a narrowly specified transfer contract that the backends can implement. Preserve realm attribution, adopted-node resolution, one checkpoint per task, script error reporting and referrer-relative module identity. Remove old dispatch/representation paths once callers migrate.

Then repeat the existing Forms gates against that production path, including matched candidate/control captures on both engines and the remaining encoding/regression requirements. Checkpoint C remains a human acceptance decision. This audit has not supplied it.

### Remove competing parsers and projections

For R1/R2/R3/L1, make selectors and CSS lexical/declaration facts come from their established parser owners. Invalid selectors must differ from no match; one query must not perform N+1 traversals; quoted comments/semicolons must survive parsing. Preserve static result membership, shadow scope, CSSOM priority/order and custom-property behavior.

I7/I9 similarly consume canonical URL/host/site facts without collapsing sandbox and opaque-origin policy. R11/R12 replace lossy XPath/history adapters while preserving live identity and structured state. I10/L5 move automation and public geometry to the visible retained owner before removing approximations.

### Make resource and retained-data ownership singular

For I3/I4/I5/L3/L6/L7/L8, preserve document/base/resource identity, one journal consumption boundary and versioned immutable payloads. Counting witnesses should show no resource fetch from unrelated text mutation, one relevant reacquisition on dependency change, no capture-induced change to incremental decisions, and no copying of whole image payloads merely to paint a retained frame. Two sheets using the same relative image spelling must retain their different bases. Preserve correctness fallback until the new journal/plane contract is proven.

I8/H5 must enforce host-configured actual bytes, deadlines and admission rather than header declarations or a whole-run timeout. Unknown-length and compressed bodies need witnesses. Cache-copy abandonment should continue stream delivery when policy allows; native response buffering is a separate bound.

### Fix shader linking before optimizing the compiler path

W1 is a correctness prerequisite for I11/I12/I13 simplification. Reordered same-type varyings and disjoint/reordered uniforms must have coherent stage locations, binding offsets and cross-stage type checks. Confirm accepted layouts with an actual draw. Then retain source-revision compiled artifacts and remove recompilation for cache identity, obsolete positional reflection and raw text normalization. Preserve original source and phase diagnostics. Rejecting valid reordered interfaces merely to silence the mismatch is a narrowed feature, not a completed link repair.

### Narrow layout/backend and platform debt under their actual contracts

L4/L9/L11/L12 reconcile CSS facts, backend conversion and current property-consumption evidence. Retain Taffy's active low-level algorithms, Buckram table ownership, actual retained line boxes, SizingMode cache identity and deep-recursion guards. L13 reduces opt-in diagnostic overhead while preserving Windows font receipts. Fork retirement needs the exact accepted replacement and existing regression cases, not an assumption about current upstream.

H2/H9/H10 platform work must preserve shared device/queue custody, external image release, recovery leases, deadline/readback attribution and actual OS handoff. Its done-condition includes the relevant browser/native/platform witness; CPU source inspection cannot supply it.

## Rejected broad cuts and confidence limits

The runtime is not all mocks: arena/GC/adoption, streams, messaging, form values, frames and real host/GPU seams have substantive implementations and regression families. Native form value state is not just attributes. Private branding and captured intrinsics enforce real trust/identity boundaries. Keep them.

Damage tracking is used. The table shadow ledger is used. Distinct text, intrinsic-size, backend and stylesheet caches have distinct identities. Generated text is an intentional projection. Font rebuild after removal prevents stale faces. Static and mutable DOM stores have different lifetimes. Shared paint/input placement helpers are not a second transform parser. Removing these by name or size would introduce failures.

A handwritten implementation can be appropriate: Fleece preserves raw invalid JSON-LD, duplicate keys, order and number spelling. HTML legacy parsing differs from CSS. The specific defects concern duplicate or lossy authorities, not a prohibition on custom parsing.

Known unexecuted obligations remain visible in the evidence sections: Forms qualification, K6 continuation, table distribution, contextual colors, Windows font geometry, browser scheduling/input, worker interruption, network-policy behavior, GPU linking and platform presentation. These are limits on implementation/qualification claims. They do not justify postponing the ownership/removal decisions already supported by source.

## Additional root and standalone surfaces

The root `Cargo.toml` owns workspace composition and source patches; preserve it as the declared dependency authority. The six manifests under historical receipt/probe directories belong to evidence snapshots, not current workspace packages. Keep them with their receipts. The package inventory must not silently count them as active engine crates.

`examples/netrender_smoke` is a deliberate standalone workspace. Its manifest restates source patches because Cargo patches do not propagate from another workspace root. Its program boots an adapter, submits a Scene and checks completion/validation; its header explicitly disclaims pixel or host-presentation parity and requires a process timeout. Retain it under that narrow meaning. It is not another document engine, nor evidence of visible presentation. The example was read, not executed.

Root formatting, toolchain, licensing, nextest and dependency-policy files are configuration with distinct consumers. Keep them. The nextest default has a finite slow-test termination policy; it does not prove backend interruption inside the application. `deny.toml` contains inherited advisory exemptions and duplicate-version allowances with historical explanations. This audit did not run cargo-deny, verify current advisory status or certify the dependency graph secure. Revalidate active exemptions against the actual resolved graph when changing that policy; old comments are not a current justification.

**X1, additional legacy cut candidate:** `.cargo-check-logs/cargo-check-env.ps1` still advertises retired `servo-layout`/`servo` and mozjs build setup; `third_party/mozdebug` is a debugger-discovery helper whose Python module imports mozinfo. Searches of current components, ports, support, scripts, workflows, root README/manifest and canonical index found no current references to either. Retire the environment script as historical build evidence; consider removing the vendored helper after verifying manual/external tooling consumers. Do not infer that an unreferenced manual utility is unreachable everywhere. Neither was executed. The Blink import shell script is different: it imports a pinned Chromium fixture corpus and license. Preserve import provenance and assets; an importer is not a current benchmark acceptance runner.

## Coverage by workspace package

The following counts use the 996-file core inventory and assign files to the most specific workspace package directory. They exclude supplemental assets and external Forms artifacts. Nonmember code/support appears in the coverage receipt and domain dispositions. A zero inventory-only count still does not mean an every-line semantic audit when targeted reads predominate.

| Package | Core files | Full | Targeted | Inventory only |
|---|---:|---:|---:|---:|
| [browsing-context-api](../components/shared/browsing-context/Cargo.toml) | 2 | 0 | 2 | 0 |
| [buckram](../components/buckram/Cargo.toml) | 23 | 2 | 21 | 0 |
| [cadency](../components/cadency/Cargo.toml) | 7 | 3 | 4 | 0 |
| [document-session-api](../components/shared/document-session-api/Cargo.toml) | 7 | 0 | 2 | 5 |
| [engine-observables-api](../components/shared/engine-observables-api/Cargo.toml) | 9 | 0 | 2 | 7 |
| [fleece](../components/fleece/Cargo.toml) | 17 | 0 | 8 | 9 |
| [genet-clipboard](../components/genet-clipboard/Cargo.toml) | 3 | 1 | 2 | 0 |
| [genet-compositor](../components/genet-compositor/Cargo.toml) | 15 | 2 | 13 | 0 |
| [genet-document-resources](../components/genet-document-resources/Cargo.toml) | 2 | 0 | 2 | 0 |
| [genet-documents](../components/genet-documents/Cargo.toml) | 12 | 0 | 6 | 6 |
| [genet-fontique](../support/patches/fontique/Cargo.toml) | 23 | 0 | 5 | 18 |
| [genet-host-api](../components/genet-host-api/Cargo.toml) | 3 | 0 | 2 | 1 |
| [genet-idl-interface-table](../support/idl-interface-table/Cargo.toml) | 5 | 2 | 3 | 0 |
| [genet-livery](../components/genet-livery/Cargo.toml) | 86 | 3 | 82 | 1 |
| [genet-paint-types](../components/shared/paint-types/Cargo.toml) | 11 | 0 | 1 | 10 |
| [genet-parley](../support/patches/parley/Cargo.toml) | 43 | 0 | 15 | 28 |
| [genet-render](../components/genet-render/Cargo.toml) | 6 | 1 | 5 | 0 |
| [genet-render-host](../components/genet-render-host/Cargo.toml) | 7 | 1 | 6 | 0 |
| [genet-scripted](../components/genet-scripted/Cargo.toml) | 8 | 0 | 7 | 1 |
| [genet-scripted-dom](../components/genet-scripted-dom/Cargo.toml) | 17 | 1 | 16 | 0 |
| [genet-scripted-worker](../components/genet-scripted-worker/Cargo.toml) | 2 | 2 | 0 | 0 |
| [genet-static-dom](../components/genet-static-dom/Cargo.toml) | 3 | 1 | 2 | 0 |
| [genet-text](../components/genet-text/Cargo.toml) | 3 | 1 | 2 | 0 |
| [genet-winit-host](../components/genet-winit-host/Cargo.toml) | 3 | 1 | 2 | 0 |
| [genet-wpt](../ports/genet-wpt/Cargo.toml) | 20 | 0 | 18 | 2 |
| [layout-dom-api](../components/shared/layout-dom/Cargo.toml) | 2 | 0 | 1 | 1 |
| [livery](../components/livery/Cargo.toml) | 51 | 0 | 51 | 0 |
| [netfetcher](../components/netfetcher/Cargo.toml) | 26 | 0 | 23 | 3 |
| [ortet](../ports/ortet/Cargo.toml) | 22 | 1 | 12 | 9 |
| [script-engine-api](../components/script-engine-api/Cargo.toml) | 3 | 2 | 1 | 0 |
| [script-engine-boa](../components/script-engine-boa/Cargo.toml) | 2 | 1 | 1 | 0 |
| [script-engine-nova](../components/script-engine-nova/Cargo.toml) | 5 | 1 | 4 | 0 |
| [script-engine-piccolo](../components/script-engine-piccolo/Cargo.toml) | 2 | 1 | 1 | 0 |
| [script-runtime-api](../components/script-runtime-api/Cargo.toml) | 103 | 2 | 101 | 0 |
| [servo-media](../components/media/servo-media/Cargo.toml) | 2 | 0 | 2 | 0 |
| [servo-media-audio](../components/media/audio/Cargo.toml) | 28 | 0 | 4 | 24 |
| [servo-media-auto](../components/media/backends/auto/Cargo.toml) | 2 | 1 | 1 | 0 |
| [servo-media-derive](../components/media/servo-media-derive/Cargo.toml) | 2 | 0 | 1 | 1 |
| [servo-media-dummy](../components/media/backends/dummy/Cargo.toml) | 2 | 0 | 1 | 1 |
| [servo-media-examples](../components/media/examples/Cargo.toml) | 24 | 0 | 1 | 23 |
| [servo-media-gstreamer](../components/media/backends/gstreamer/Cargo.toml) | 15 | 0 | 4 | 11 |
| [servo-media-gstreamer-render](../components/media/backends/gstreamer/render/Cargo.toml) | 2 | 0 | 1 | 1 |
| [servo-media-gstreamer-render-android](../components/media/backends/gstreamer/render-android/Cargo.toml) | 2 | 0 | 1 | 1 |
| [servo-media-gstreamer-render-unix](../components/media/backends/gstreamer/render-unix/Cargo.toml) | 2 | 0 | 1 | 1 |
| [servo-media-player](../components/media/player/Cargo.toml) | 7 | 0 | 4 | 3 |
| [servo-media-streams](../components/media/streams/Cargo.toml) | 5 | 1 | 1 | 3 |
| [servo-media-traits](../components/media/traits/Cargo.toml) | 2 | 1 | 1 | 0 |
| [servo-media-webrtc](../components/media/webrtc/Cargo.toml) | 4 | 0 | 2 | 2 |
| [servo-webgl-essl](../components/webgl-essl/Cargo.toml) | 54 | 0 | 4 | 50 |
| [servo-webgl-wgpu](../components/webgl-wgpu/Cargo.toml) | 19 | 0 | 9 | 10 |
| [servo-xpath](../components/xpath/Cargo.toml) | 9 | 2 | 7 | 0 |
| [taproot](../components/taproot/Cargo.toml) | 4 | 0 | 4 | 0 |

## Detailed evidence

These sections preserve the reviewers' evidence and explicit no-cut decisions. Their relative source references refer to this repository unless a sibling path is named. The independent remaining-domain review repeats some findings deliberately as corroboration; W1 is its additional shader-link defect. Statements about read-only work describe the source review; the final audit artifacts and index are the documentation edits made afterward.

## Integration evidence


Source: Genet `8fa73497952ecda37cf234c4a845354dab1af599`. This is a source audit. The isolated Origin helper probe is recorded separately. No Cargo build or product test was run. Offline `cargo metadata --no-deps --offline --locked` established workspace membership without compiling.

#### I1 Scripted input has a separate and incomplete default action path

`components/genet-documents/src/engines/scripted.rs:389-408` accepts pressed keys, consumes Tab before dispatching a page key event, and routes other keys through `dispatch_host_keydown`. It implements neither `text_input` nor `ime_input` nor `editable_focus`; their defaults in `components/shared/document-session-api/src/session_engine.rs:916-938` ignore editing. Native Livery implements editing and composition in `engines/livery.rs:2120-2244`. The private scripted key bridge at `script-runtime-api/dom/bootstrap.js:6942` only dispatches keydown; `form_data/bootstrap.js:327` adds implicit Enter submission, not text insertion. Key releases are ignored by the scripted session. These are source-confirmed missing paths, not a fresh physical-input observation.

Consolidate editable-control default operations, focus facts and input outcomes under a shared owner, retaining separate native event translation and script event cancellation. Delete the form-specific substitute for general input dispatch after migration. Preserve IME/preedit, selection offsets, validation interaction state, beforeinput/input/change ordering, trusted events, Tab cancellation and keyup. Tests of value setters or prefilled submission fixtures do not qualify typing. Add a live native/scripted same-fixture input witness before claiming parity.

#### I2 Lifecycle tests exercise an implementation the live document does not share

`components/genet-scripted/document.rs:604-619` independently stores hidden/frozen state on `LiveryScriptedDocument`. Its constructor initializes frozen=false at line1002 and its pump reads it at1435; source search finds no setter on this type. The separate headless `ScriptedDocument` has `freeze`/`resume` at509-526. The visibility/lifecycle fixture at2067-2069 constructs that headless type. The live frozen branch is therefore prospective state, not proof of a wired freeze capability.

Share one lifecycle owner across both document presentations; expose capabilities and migrate actual host control before removing duplicated fields/pumps. Keep visibility distinct from freeze, and keep timer/worker/resource delivery rules explicit. A hidden page must not become permanently frozen merely to simplify scheduling. This complements the missing live animation-frame wiring in host finding H3.

#### I3 Generic mutation synchronization repeats complete resource resolution

`components/genet-scripted/livery.rs:214-253` calls `ResolvedDocumentResources::resolve_with_limits` after every new journal suffix or arena replacement, regardless of the mutation kind. The resolver at `components/genet-document-resources/lib.rs:506-517,599-647` starts fresh fetch caches and walks stylesheets and document resources. `ComputedStyleHandler::computed_value` calls synchronization at `livery.rs:1503`; frame construction calls it at671. `ScriptResourceBridge` delegates synchronous resource fetches at `resource_bridge.rs:365-378`. Thus an unrelated text mutation followed by a style read or frame can invoke resource fetching again. A lower transport cache may change wire traffic, but not this call topology.

Remove resource reacquisition from generic style synchronization. Retain acquisition results under explicit document/resource identity and update dependencies when URL/base/stylesheet/owner facts change. Preserve redirects, imported sheet bases, dynamic resource replacement, source identity and negative results. Require a counting fetcher witness proving unrelated text mutation and computed-style reads issue no new resource fetches; relevant dependency changes must still fetch.

#### I4 Source identity is lost in image aliases

`components/genet-scripted/livery.rs:175-193` indexes each image by both authored spelling and resolved URL. The shared authored alias can be overwritten when different stylesheet bases use the same relative spelling. `genet-livery/src/style.rs:77-91` retains the original stylesheet source; only font descriptors are explicitly URL-remapped at123-129. Painting uses the computed background URL string at `paint.rs:4078-4079` and looks it up directly at525. Resource resolution itself retains the distinction at `genet-document-resources/lib.rs:1359-1387`.

Remove ambiguous aliases; carry source-relative resource identity into computed image values and paint resource keys. Preserve CSSOM authored text and image replacement semantics. The map collision and input identity loss are source-confirmed; a two-stylesheet pixel fixture remains unexecuted.

#### I5 Capture consumes facts owned by rendering

`components/genet-scripted/capture.rs:420-443` reads and then drains the same DOM mutation journal as rendering. `LiveryScriptedDocument::evaluate` flushes capture before a subsequent frame (`document.rs:1489-1495`). `livery.rs:686-692` responds to a missing suffix by invalidating retained style/frame state. This is a conservative correctness fallback, but the instrumentation changes the incremental path it observes. The ordinary frame also drains the journal at793-794.

Remove independent recorder drain authority. Publish the consumed mutation batch from the document's journal owner to observers, or give observers non-stealing cursors with a defined compaction boundary. Preserve captured node translation, baseline epochs, historical postcard discriminants and disabled-capture behavior. Verify identical incremental decisions with capture on and off as well as identical output. Do not delete historical capture files or the fallback before the owner contract changes.

#### I6 Request and response compatibility layers preserve overlapping representations

`script-runtime-api/fetch.rs:27-52` and `genet-host-api/lib.rs:131-142` independently carry nearly identical request fields. `genet-scripted/resource_bridge.rs:317-329` clones all of them into the host representation. The host's response pair at `genet-host-api/lib.rs:164-222` is expressly retained for old struct literals: promotion supplies 200/OK/basic metadata; demotion drops actual status/header/redirect facts. Ortet builds the rich response through that promotion and then overwrites metadata (`ports/ortet/src/fetch.rs:234-243`).

Make the host-owned request contract the shared boundary and migrate legacy byte-only callers deliberately before deleting the overlapping records and conversions. This must not invert the VM/transport dependency. Keep arbitrary protocol methods open; use bounded enums for closed policy choices where appropriate. Preserve intentional local-file synthetic response behavior as an explicit adapter, rather than a universal success default.

#### I7 Origin interpretation is independently handwritten

`components/shared/browsing-context/lib.rs:72-117` splits schemes, authority, credentials and ports by string operations. Invalid port parsing becomes None at103/108, the same state as a clean default port. The isolated verbatim helper probe shows `https://good.test:garbage/` and `https://good.test/` becoming the same tuple, and records backslash and percent-host differences. `mint_origin` uses this helper at578 and frame-policy target-origin checking uses it at `genet-documents/src/frame_policy.rs:225`. Netfetcher separately uses parsed `url::Url` origins.

Remove independent URL grammar from origin computation and derive the tuple from one parsed URL owner. Preserve opaque-origin identity, sandbox minting, default-port normalization and policy/value separation. The probe proves helper behavior only; exploit reachability or the exact effect of normalized loader URLs was not established. Exercise malformed ports, encoded/IDNA hosts, backslashes, IPv6 and opaque URLs at every caller boundary before making security claims.

#### I8 The cache body cap is not a streamed byte bound

`components/netfetcher/src/fetch/cache_phase.rs:85-88` tests Content-Length only. `CachingBody` has no cap field and extends its buffer for every decoded chunk at119-120. Finalization installs it after decoding at `fetch/finalize.rs:114,141-159`. Missing length and compressed-to-larger bodies therefore bypass the intended cache buffering bound. Existing `fetch/tests.rs:620-645` tests a known five-byte body against a declared cap, not either bypass. Default `FetchContext::permissive` has NoHttpCache, so this issue requires an enabled cache; it does not prove default Ortet caches unbounded bodies.

Keep the streaming/cache separation, but remove header-only proof of bounded buffering. Enforce the configured cap on accumulated decoded bytes, stop retaining a cache copy after crossing it, and continue delivering the stream if that is the host policy. Preserve incomplete/error-body rejection and exact cache metadata. Native Ortet's separate whole-response cap gap is H5, not the same code path.

#### I9 Cookie policy remains partial behind a broad default context

The default `InMemoryCookieJar` documents deferred public-suffix and __Secure-/__Host- restrictions (`netfetcher/src/cookie_jar.rs:14-18`). Its Domain handling at206-217 only checks suffix matching, even though PSL support is already available in netfetcher. This is a capability boundary, not a dead implementation to delete wholesale. The source demonstrates missing policy checks; no live cross-site test or exploit was run.

Consolidate canonical host/site facts with the parsed URL/PSL owner. Complete or explicitly constrain the default jar before broad browser qualification. Preserve host storage injection, raw persisted records, expiry, path order and SameSite context. Keep partial-cookie limits explicit rather than adding a second cookie policy in each host.

#### I10 Taproot retains a second semantic and geometry interpretation

`components/taproot/lib.rs:206-237` maintains a legacy role/name approximation, while `projection.rs:24-66` uses the retained owner projection. `resolve` constructs styles and a fresh estimated layout at `lib.rs:263-283`. It can disagree with the visible retained font/resource geometry. These paths are live: Turnstone's `src/cambium_pane.rs:310` calls resolve and `src/inspector_pane.rs:440` calls matching; other family consumers also use these entrypoints.

Migrate real callers to owner-computed semantic targets and retained geometry, then remove the legacy role policy and shadow-layout resolver. Retain a class/raw-DOM selector only if callers deliberately need that different query. Preserve miss versus unsupported, family scoping and stable identity. Do not delete Taproot wholesale; its shared automation contract and scenario outcomes remain useful.

#### I11 WebGL recompiles shaders to discover a cache key

`webgl-wgpu/shader/lowering.rs:43-81` fully parses, checks, validates and lowers shaders for compile validation and again for `canonical_essl_cache_key`; cold translation at27-36 does it again. `webgl/programs.rs:47-65,175-193` follows those three stages for a normal compile then cold link. Lowering separately reruns type checking at `webgl-essl/lower.rs:86-93`. The pair cache lookup consequently happens after another full shader compilation. This is source-confirmed repeated work, not a measured cost claim.

Retain compiled shader artifacts/reflection under the shader object and let linking consume them. Compute identity from actual source/stage/options without translating just to hash it. Remove duplicated phase orchestration while retaining explicit parse/check/validate/lower diagnostics, invalidation after shaderSource, resource reflection, stage interface rules, and driver pipeline creation. The Naga worker's finite lifetime and stack protection serve a separate purpose and should remain.

#### I12 Old narrow shader reflection remains an acceptance condition

`webgl-wgpu/shader.rs:60-70` retains five narrow convenience fields after general attribute/uniform/sampler vectors were added. Search found production construction but only test readers of those five fields. `shader/lowering.rs:145-156` requires the first vertex input solely to fill this legacy view, rejecting empty input sets before general linking. Other fields are guessed from vector positions. The comments claiming production draw still needs them are stale.

Remove the narrow fields and their positional inference; test the general reflection contract. Preserve all real attribute/uniform/sampler bindings and source interfaces. Verify empty-input and multiple/reordered-input programs through compilation/linking and rendering. Do not remove the actual general linker or assume the source-only audit proves shader correctness.

#### I13 Shader semantics are edited or reparsed outside the compiler

`webgl-wgpu/webgl/programs.rs:34-42` performs raw text replacement of gl_FragData[0]. `shader/lowering.rs:295-317` searches raw text for precision declarations after the ESSL frontend already parsed source. These shortcuts create another grammar authority and admit whitespace/comment discrepancies. The shader source shown to diagnostics must remain identifiable.

Remove source-wide substitution and secondary declaration scanning; put any supported normalization and precision metadata in the parsed shader owner. Preserve currently supported primary output semantics; unsupported multiple outputs should be explicit. The actual shared stage-link layout is separately assessed by the remaining-domain reviewer and takes priority over naming cleanup.

#### I14 An old media thread survives outside the active workspace

Offline metadata reports 52 workspace packages and excludes `components/media/media-thread/Cargo.toml`. Its only declared alias is the unused root `Cargo.toml:352` dependency. Searches across Genet, Mere, Turnstone, Woodshed, Hocket and Isometry found no current callers of this package/WindowGLContext/GLPlayerThread. Its source still carries the former constellation/WebRender/GL player integration (`media-thread/lib.rs:78-169`, `media_thread.rs:5-85`).

Remove the orphan alias and retired integration subtree after checking external consumers and preserving source history. Keep media player/traits/streams contracts: Turnstone and Woodshed manifests still consume them. Active GStreamer GL-backed video is distinct from the no-op compositor GL capability stub and cannot be deleted on that finding. CPU media decoding and raw frame output must remain available.

#### I15 Inherited benchmark launchers are not current Genet gates

`tests/dromaeo/run_dromaeo.py:21-24` launches an executable with Servo's old positional URL/-z/-f interface. `tests/jquery/run_jquery.py:19-42` retains old mozjs crash-based module exclusions. `tests/power/PowerMeasure.py:45-58` invokes servoshell and process-wide PowerMetrics cleanup. A search of current components, ports, CI, support and the active documentation index found no references to these launchers. The Blink corpus updater is an import tool, not a test runner; classify it separately.

Retire or archive the obsolete launch instructions and runners rather than adapting them with more aliases. Keep reusable fixtures, licenses and imported benchmark provenance unless a separate reference audit establishes they can be removed. No benchmark was run, no process was stopped and no legacy fixture was deleted.

#### Retain decisions for remaining families

* Fleece's render-free extraction, raw JSON-LD preservation, stable wire records, source anchors, tables, metadata and normalization have distinct consumers and declared contracts. Preserve number spelling, duplicate/order evidence and raw invalid blocks before considering replacement of its private JSON parser. The recursive parser/tree walkers deserve a separate bounded-input stress check; no overflow was reproduced here and no parser replacement is declared safe by this audit.
* Genet Text is one Unicode boundary owner with generated tables and corpus fixtures. Keep grapheme/word/sentence and byte/scalar/UTF-16 distinctions. No second owner or justified deletion was found.
* Genet Clipboard is one host service with real in-memory/system adapters. Its current system adapter supports custom representations despite older introductory prose; stale prose is not evidence the code lacks that feature. Keep the service, native error mapping and representation contracts.
* Shared layout-dom, paint-types, observables and document-session contracts express distinct rendering/data/host facts. Consolidate duplicated pending-work interpretation and origin grammar, not all contracts into a monolith. SessionPendingWork intentionally has different settle policy from full idleness; rAF wake omission should not be repaired by waiting forever for animation loops.
* Netfetcher phase separation, transport injection, cookie/cache/HSTS/preflight/referrer/SRI/decode owners are useful. The Murm wire adapter should preserve that split. DefaultTransport compatibility can be migrated by real callers, not removed solely because it is an adapter. Invalid custom method fallback-to-GET in transport/hyper.rs:76 is a further explicit-boundary question, not an asserted wrong request from the validated Fetch path.
* Media's audio graph, GStreamer/dummy platform backends, typed timing buffers, stream registry, WebRTC/data channels and platform render adapters remain active contracts. The historical per-channel AudioRenderer fallback discards new timing/layout semantics for legacy callbacks; migrate actual old implementors before deleting it. Whole-media removal would break checked family dependencies.
* Native WebGL's device/queue texture contracts, state/resource validation, framebuffer/readback and actual ESSL frontend are required. The compiler's lexer/parser/checker/validator/lowerer are successive phases, not interchangeable duplicates. Cut repeated execution and legacy projections rather than the phases themselves.
* Maintained regress is a documented native UnicodeSets correction with an explicit retirement criterion. IPC-channel's in-process patch currently preserves media type consumers; replacing serialization with typed channels is a migration, not a safe blanket deletion. Sonic's manifest feature change removes a known unwanted RNG edge; GPU allocator and other maintained platform forks require provenance/consumer checks before retirement. Unchanged upstream bulk was scanned as inventory, not rereviewed as first-party logic.

#### Evidence and coverage

`inventory.json`, `structural-scan.json`, `workspace-metadata.json`, `root-coverage.json`, supplemental inventory, and the three other domain reviews record the source snapshot and review depth. Core source/config scanning is mechanical; full and targeted reads are reported separately. Maintained patch review concerns activation and local deltas, not the correctness of every upstream line. External standards breadth, runtime/browser/hardware and downstream rebuilds remain unexecuted validation obligations, not passing claims.

## Runtime and DOM evidence


Checkout: `/home/markik/Code/repos/genet`, HEAD `8fa73497952ecda37cf234c4a845354dab1af599`. Read-only source audit, 2026-10-16. No builds, test execution, dependency downloads, product edits, or Git mutations were performed. All findings below are **source-confirmed** unless explicitly marked hypothesis. None is runtime-proven by this audit. Existing tests are source evidence of intended coverage, not fresh passing receipts.

Scope: script-engine-api, script-engine-boa, script-engine-nova, script-engine-piccolo, script-runtime-api, genet-scripted-dom, genet-static-dom, genet-scripted-worker, xpath. Every inventory file in these packages received structural inspection; `runtime-coverage.json` distinguishes full reads from targeted source/index inspection. This is a complete package/module-family removal review, not a claim that every line in this large domain received equal semantic scrutiny. Test indexes and generated tables were generally inspected structurally. Targeted reads followed the risk-bearing implementations and selected relevant tests. No finding count cap was applied.

#### Findings and cut directions

##### R1. Remove the private limited selector grammar; retain a DOM adapter over the shared selector engine

Evidence: `components/script-runtime-api/selector.rs:25`, `:35`, `:63`, `:313` define an independent AST, two combinators, three attribute operations, and a parser that turns any unsupported/invalid member into an empty selector list. `dom/query_traverse.rs:42`, `:58`, `:76`, `:98` use it directly. `dom/bootstrap.js:1463`, `:2090` expose results without a parse-error channel. Cadency already provides generic `Element`, `PseudoClass` and fallible `SelectorList` parsing in `components/cadency/src/lib.rs:18` onward. The runtime dependency manifest does not use Cadency.

Consequence: sibling selectors and attribute operators supported elsewhere in the engine become empty DOM results; malformed selectors are indistinguishable from valid selectors with no matches. The module header is also stale about pseudo support: it now implements four validity pseudos at `selector.rs:51` and `:190`.

Cut: replace the private grammar/matcher with the shared selector parser and a `ScriptedDom` element adapter, preserving native validity facts, ordinary-tree scope, case/namespace behavior, and shadow policy. Do not merely extend another local grammar. Verify Cadency's actual DOM-query suitability before promising identical semantics.

Tests: `selector.rs:572-604` cover basic compounds and explicitly require unsupported selectors to match nothing; these encode the compatibility ceiling and need replacement, not blind preservation. `dom/tests.rs:396` and `:2528` cover validity ownership and must remain. Add shared parser differential tests plus valid sibling selectors, escapes, invalid-list exceptions and shadow-root scope.

##### R2. Delete the count/item query protocol that repeatedly parses and traverses the same DOM

Evidence: `selector.rs:78-91` collects all descendants even for the first match. `dom/query_traverse.rs:69-103` recomputes `query_all` for count and every indexed item. `dom/bootstrap.js:2091-2097` first requests count then calls the item sink once for every result.

For N matching descendants, the current all-results path performs approximately N+1 full parse/walk/filter operations. This is a source-derived complexity conclusion, not a measured performance claim. It is independent of which selector parser wins R1.

Cut: one native selection produces a stable list of genuine reflectors or a retained result token with bounded lifetime. Preserve static NodeList membership, order, full-width IDs, and wrapper identity. Direct callable support alone does not provide an engine-neutral array/object extraction API.

Tests: basic DOM query tests exist in `dom/tests.rs`; add an instrumented one-traversal assertion for a large all-matching tree and ensure later mutations do not change already-returned membership.

##### R3. Remove JS declaration splitting and the duplicated declaration record codec

Evidence: `dom/bootstrap.js:1509-1520` splits declarations at every semicolon and first colon. This feeds inline attribute reads at `:1755`, cssText assignment at `:1783`, and rule declaration reads around `:1911`. Values are only subsequently handed to `__inlineStyleValue` at `:1650`; native property canonicalization cannot recover text already split incorrectly. The same area owns separate escaped newline/tab codecs at `:1524-1574`, paired with Rust `dom/mod.rs:480-563`. Missing inline-style handlers return PassThrough at `dom/mod.rs:490`.

Cut: the CSS owner should parse a whole declaration list and return a defined structured result. Keep JS property ergonomics and shorthand pending-substitution state as needed, but remove the second tokenizer and its private serialization grammar when an adequate typed boundary exists. Until then, consolidate a rigorously specified codec rather than scattering escapes. Preserve authored ordering, custom properties, priority, shorthand expansion/reconstruction, pending substitution, and malformed-result atomicity.

Tests: `dom/tests.rs:1214-1247` exercises simple declaration text. Handler tests beginning `:1260` and shorthand tests around `:1383` are valuable preservation tests, not proof of full declaration parsing. Add quoted semicolons, URL values, comments, escaped identifiers, custom-property case, and `!important` through both attribute reads and cssText assignment.

##### R4. Replace source-eval task entry points with retained callables and explicit error outcomes

Evidence: both JS backends implement `ScriptEngine::call_function` (`script-engine-boa/lib.rs:680`, `script-engine-nova/lib.rs:1202`). The runtime still synthesizes dispatch source at `script-runtime-api/lib.rs:1196`, timer source at `:1914`, fetch completion source at `:2117` onward, and animation source at `:1865`. `eval_fetch_completion` at `:2093-2112` discards errors and removes terminal routing afterward. `run_timers` at `:1832` converts errors to zero work. `has_animation_frame_callbacks`/`next_timer_delay` convert failures to absent work. Generic event dispatch does propagate eval errors, but a result-coercion error defaults to allowing the default action at `:1214-1218`. Therefore the provisional claim that all event errors are hidden is too broad.

Cut: retain private trusted callable handles at installation, route through the correct realm, and distinguish task failure from no work. Use shared reporting for exceptions that should be reported rather than thrown to the embedding host. Preserve one checkpoint per task, trace boundaries, fetch realm routing, cancellation, and decimal-string u64 identity. Bootstrap-private handles should survive author replacement of public globals without giving authors access to internal records.

Tests: `tests/realms.rs:134` covers child fetch routing; realm animation, harness-completion and timer-order tests exist. Add deletion/replacement/throwing of each old global entry point and verify failures cannot silently erase pending work or authorize a default action. `ValueEngine` is a separate neutral value-computation trait (`script-engine-api/value.rs:100`); none of the three backend files implements it here. It does not solve this DOM transfer boundary.

##### R5. Unify parsed-script failure reporting with explicit load/evaluation outcomes

Evidence: `parse.rs:526-552` skips absent loader output, discards classic-script evaluation errors, clears currentScript and continues. Deferred classic and module paths similarly discard errors (`:566`, `:598`). Parser report counters are incremented by scheduling paths at `:303-305`, `:335-346` independently of successful source load/evaluation. Worker startup, in contrast, forwards initial script errors (`worker.rs:352-357`).

Cut: one script execution result should distinguish skipped, fetch refused/failed, unsupported module, thrown script, and executed successfully. Continue parsing when HTML requires it, but report the failure instead of losing it. The report's counters must have named meanings. Preserve currentScript restoration, checkpoints, parser insertion point, and already-started behavior.

Tests: `tests/parser_script_interleaving.rs` covers order, document.write, readiness, custom elements and module ordering; `tests/pending_script_adoption.rs` covers deferred work after adoption. Add error listeners/console evidence for missing external source and thrown blocking/deferred/module scripts while proving later scripts still execute. This does not recommend treating every thrown script as a fatal parse error.

##### R6. Remove document-base-only module resolution from the parser's resolver adapter

Evidence: the engine API provides `(specifier, referrer_url)` to the resolver (`script-engine-api/lib.rs:212` onward). The parser closure explicitly ignores `_referrer`, calls `loader.resolve(specifier)` and `loader.load(specifier, ...)` at `parse.rs:589-592`. Its loader API only receives the specifier and document-held loader context (`:70-85`). Inline module base is an empty string at `:574`.

Consequence: the adapter drops importer identity needed when `/a/one.js` imports `./two.js` and the document is elsewhere. Exact resulting URL depends on the installed loader; the information loss is source-confirmed, the end-to-end misfetch is an unexecuted hypothesis.

Cut: resolve each specifier against its provided referrer URL with the shared URL owner, then pass that resolved identity into loading and module caching. Inline modules need the document base snapshot. Preserve redirects/final URLs and graph cycle/diamond semantics.

Tests: parser module ordering tests and backend module tests exist, but do not establish this nested-relative route. Add a document in `/page/`, entry in `/modules/a/`, nested imports in another directory, a redirect, and diamond imports with different original specifiers.

##### R7. Remove the false worker lifetime guarantee; add enforceable cancellation before claiming thread ownership

Evidence: `worker.rs:154-157` hardcodes 30-second resource wait and 500-ms shutdown join window. `shutdown` at `:835-863` sends a cooperative signal, then drops unfinished `JoinHandle`s. Its own comment at `:849-852` says a spinning VM script cannot observe termination and is detached. `Runtime::drop` claims no worker thread outlives the agent (`lib.rs:2369`) but cannot enforce that. Worker startup calls ordinary unbounded eval (`worker.rs:352`), and the loop invokes unbounded microtasks (`:388`).

Cut: cooperative messaging is appropriate for ordinary shutdown but cannot own arbitrary script lifetime. Introduce a backend interruption contract or process isolation for untrusted workers, and expose unsupported cancellation explicitly. Until then describe detachment honestly, make policy configurable, and do not represent termination as reclamation. Retain independent worker engine/heap construction and page-owned resource transport.

Tests: `tests/worker.rs:303` proves cooperative terminate/close only. Add an isolated process test for infinite startup script, infinite task callback and self-replenishing microtasks; assert process/thread/resource completion after terminate and document drop. Do not run those hangs in-process without an external kill boundary.

##### R8. Cut budget-shaped APIs that silently run unbounded

Evidence: `ScriptEngine::eval_bounded` defaults to `eval` at `script-engine-api/lib.rs:267-280`; capability limits are comments rather than an executable refusal. Boa's `pump` ignores Budget and drains fully (`script-engine-boa/lib.rs:798-805`); Piccolo's pump also ignores it (`script-engine-piccolo/lib.rs:430-440`). Piccolo does meter main eval (`:328-367`), while Nova meters reaction jobs (`script-engine-nova/lib.rs:1454`), not arbitrary work inside one job. Runtime checkpoints call `pump_microtasks` without a host-visible outcome (`script-runtime-api/lib.rs:1144-1163`).

Cut: expose separate capabilities/outcomes for bounded VM execution and bounded job count. Unsupported bounded execution should refuse when a caller requires a bound. A counted job is not an execution interruption. Keep trusted unbounded operation available as an explicit choice. This reinforces R7 but also affects main-page responsiveness.

Tests: Boa's `pump_drains_fully_regardless_of_budget` (`:1666`) explicitly memorializes the limitation; Nova's budgeted-chain test (`:2711`) is useful but only a job-count guarantee; Piccolo's bounded-eval tests are useful. Preserve these truthful distinctions and add capability/refusal tests rather than promising one common safety level.

##### R9. WebGL's no-factory path should fail capability detection instead of manufacturing a working-looking context

Evidence: `webgl/commands.rs:14-24` returns -1 without a factory. JS still creates a `WebGLRenderingContext` with `_ctx=-1` (`webgl/bootstrap.js:81-95`); canvas getContext caches and returns it (`dom/bootstrap.js:3510-3533`). `webgl/mod.rs:152-169` silently returns defaults for negative/stale indices. The public runtime comment explicitly promises no-op/NO_ERROR without a factory (`lib.rs:2053`).

Cut: distinguish absence/failure of a host context from an actual context. Return null or a defined failure path at getContext when creation fails. Keep the optional graphics seam and independent contexts. Retain resize/external texture plumbing, triangle smoke and real handler readback tests; those prove a wired route, not truthful unwired feature detection.

Tests: `tests/webgl_binding.rs` contains a real wgpu-backed handler and broad smoke coverage. Add an absent-factory test, failed-creation test, context loss, and a valid handler's legitimate NO_ERROR result. Do not delete the real GPU tests or infer they ran in this audit.

##### R10. Replace WebGL's public mutable identity fields and permissive text coercion with branded context-owned values

Evidence: `webgl/bootstrap.js:41-45` exposes freely writable `_id`/`_loc`; `:67-71` accepts arbitrary objects carrying `_id`; `:90` exposes `_ctx`. Numeric argument parsers in `webgl/mod.rs:184-232` string-coerce and turn parse failures into zeros/defaults. Buffer data uses comma-separated numeric text (`bootstrap.js:54-59`, `mod.rs:232`). Context native routing trusts the supplied registry index.

Cut: use private per-agent weak brands and per-context resource ownership, following the already-established Streams/FormData pattern, then remove duck-typed ids and repeated numeric-list parsing as typed primitives permit. Preserve cross-realm legitimate receiver use. This is a script API/validation problem; this audit does not establish a host memory-safety exploit.

Tests: existing graphics tests focus on successful rendering and translation. Add forged resources, modified ctx fields, cross-context buffer/program/texture use, stale IDs, nonfinite/invalid numeric input and borrowed methods. Preserve actual shader/link diagnostics.

##### R11. Remove the lossy XPath result adapter; retain the generic evaluator

Evidence: XPathTree snapshots the full ordinary document (`dom/xpath_eval.rs:100-151`, `:526`) and maintains separate attribute nodes, but `raw_node` returns only node-data IDs (`:220-222`). Node-set serialization filters through it at `:546`, silently dropping attributes. JS `Document.evaluate` accepts a resolver argument but never passes it to native (`dom/bootstrap.js:3037-3053`); native uses `NoNamespaces` and parses with hardcoded HTML mode (`xpath_eval.rs:76-89`, `:535`). Detached context nodes are refused because only the main document snapshot is searched (`:527-533`). Generic `xpath` already has a namespace-resolver trait (`xpath/src/lib.rs:71`) and typed node/value model.

Cut: adapt live owner-resolved nodes and genuine Attr wrappers, or use a snapshot result type that preserves both node kinds. Pass namespace resolution and actual document mode. Retain generic XPath AST/parser/tokenizer/evaluator/value modules; XPath is a distinct language, not a redundant CSS parser. Avoid assigning DOM wrapper responsibilities to the evaluator.

Tests: `dom/tests.rs:2027-2066` covers strings, counts, element results and axes. Add `//@id` as a node result, namespace resolver on XML/SVG, detached subtree contexts, text/comment contexts, invalidation after mutation and cross-owner contexts. Scalar `string(//@id)` passing would not prove attribute-node result fidelity.

##### R12. Reuse structured serialization for history state; remove JSON-only state semantics and eventually the fallback history owner

Evidence: `platform.rs:803-827` implements state with JSON.stringify/parse despite an existing structured-clone subsystem. Cycles and BigInt throw JSON errors; Maps and Dates have JSON semantics, not preserved type semantics. `context_history` at `:390` selects browsing-context history, while paths around `:107`, `:485-522` retain a separate HostState fallback when no context exists.

Cut: use the shared serialization record format with an explicit history-state policy and deserialize in the receiving/current realm. Move history ownership into the browsing-context owner when all production callers have one, then remove fallback bookkeeping. Do not remove bare-runtime testing support by accident; make its profile explicit first.

Tests: platform history tests at `:1126`, navigation tests and top-level history tests cover JSON objects and traversal. `tests/realm_clone.rs` already tests cycles/recipient intrinsics for messaging. Add history cycles, Maps, Dates, aliases, BigInt, uncloneable values and traversal after realm replacement.

##### R13. Remove manual lifetime erasure without unwind-safe restoration in both module loaders

Evidence: Boa `HostModuleLoader::with_resolver` (`script-engine-boa/lib.rs:280-290`) and Nova `GenetHostHooks::with_resolver_in_realm` (`script-engine-nova/lib.rs:86-103`) transmute a borrowed resolver pointer to static storage, set it, call arbitrary `f()`, then manually clear pointer/cache. There is no scope guard in these functions. Nova also manually restores module realm.

The normal-return lifetime argument is documented. A panic in the host resolver/evaluation before cleanup can leave dangling scoped state if an embedding layer catches unwind and reuses the engine. That reuse failure is a **hypothesis**, not a reproduced unsafe dereference in this audit.

Cut: use an owned/scoped upstream resolver API if available; otherwise install a scope guard that restores the previous pointer/realm/cache on every exit and rejects or safely handles reentrancy. Preserve synchronous resolver behavior, cache roots and original exception identity. Tests should catch a deliberate host resolver panic, then perform a clean second evaluation, in an isolated boundary suitable for the engines' unwind constraints.

##### R14. Remove process-lifetime allocations from ordinary engine/realm registration paths

Evidence: Nova `leak_host_hooks` uses Box::leak and retains an Rc job queue (`script-engine-nova/lib.rs:201-208`); new and snapshot_clone invoke it (`:1128`, `:1045`). Registration leaks function names at `:1431`, `:1949`, `:2396`. Piccolo leaks dynamic names in set_global and set_function (`script-engine-piccolo/lib.rs:376-399`). These allocations have no owner that frees them at engine/realm destruction. Comments that setup registration is bounded are insufficient across repeated document, worker, snapshot or engine creation.

Cut: an upstream owned host-hooks/atom API is the clean solution; until then intern genuinely fixed names once and put dynamic strings/hooks under an engine owner if the upstream lifetime contract permits it. Do not free static references behind a live VM. Keep NovaValue release-queue machinery: that solves a different real root-lifetime problem and is not redundant with this finding.

Tests: reflector/rooting soak tests and worker/realm tests exist, but DOM live-node counts do not detect leaked hooks or registration strings. Add bounded repeated engine/snapshot/realm creation with allocation accounting and pending-job teardown. Exact retained heap impact is unmeasured here.

##### R15. Split the test clock profile from the product's elapsed-time source

Evidence: `timing.rs:33-40` defines performance.now as virtual timer time plus 0.001 per read. Every call advances time even when no elapsed time passes; long synchronous work does not advance the underlying timer clock. Worker event-loop time jumps directly to next delay (`worker.rs:372-396`). These are useful deterministic harness mechanics installed by the same runtime surface used by product hosts.

Cut: install a host clock seam for product elapsed time and an explicitly selected virtual-clock profile for deterministic tests. Keep ordering determinism, performance observer task delivery and shared-agent timer semantics. Remove synthetic per-read time advancement from the product profile. `timeOrigin` and Date consistency need a defined source.

Tests: `tests/cheap_globals.rs:21` and performance tests cover monotonic/read shapes. Add a fake configurable clock with controlled advances and host-driven elapsed time; do not introduce sleeping wall-clock tests for basic clock semantics. This finding does not claim a measured timeout bug.

#### Provisional finding disposition

* Limited DOM selector vs Cadency: verified, R1 and R2.
* Source-eval timer/fetch/event bridge with hidden errors: verified but qualified, R4. Dispatch eval errors often propagate; several helper/coercion paths suppress them.
* CSS split-semicolon parser and custom record protocol: verified, R3. Installed native canonicalizers do not replace whole-list tokenization.
* Scripted keyboard only dispatches keydown and lacks text editing: boundary evidence only in this owned scope. `genet-scripted/document.rs:1315-1321` dispatches form keydown; native form value/selection state exists in `genet-scripted-dom/forms.rs`. This is insufficient to claim the entire host input stack lacks editing without the root audit's input-host analysis. Retain native value state; do not describe it as merely attribute projection.
* call_function exists in both JS backends: verified. ValueEngine does not by itself enable structured DOM transfers: verified.

#### Retain/no-cut decisions across all package and module families

| Family | Architectural conclusion and preservation boundary |
|---|---|
| script-engine-api lib/value | Retain backend-neutral VM values, native callbacks, realm identity, promise tokens and reflector operations. Separate ValueEngine has a legitimate smaller consumer contract. Cut stale comments saying Nova cannot call retained functions, and make capabilities/bounds truthful. Do not merge neutral computation with DOM identity. |
| Boa backend and embedded tests | Retain wasm32 backend/oracle and native type isolation, reflector roots, caller provenance, promise tables and realm lifecycle. R8/R13 target concrete boundaries; deleting the backend is unsupported. |
| Nova backend; realms, regex, WTF8 tests | Retain native backend, rooted NovaValue release queue, ephemeron/reflector support, per-realm provenance, snapshot refusal when work is pending. Keep regex and UTF16 regression tests; partial overlap is independent regression provenance, not enough for deletion. R13/R14 target loaders and leaked allocation owners. |
| Piccolo | Retain as opt-in Lua/mod-scripting adapter with documented nil/await deviations and metered main eval. It is not compatible with JS browser bootstraps and should not be advertised as another browser backend. Consider a future smaller adapter over ValueEngine only when actual consumers no longer need reflectors/promises; that migration is not implemented here. |
| genet-static-dom lib/shadow | Retain script-free parse route, XML parser reuse, immutable indexed storage and one-time declarative-shadow/slot tables. Static and scripted TreeSinks serve different lifetimes and parser-interleaving needs. Shared policy/helper extraction may reduce drift, but collapsing both stores is not justified. Preserve parse_fragment_source inertness. parse_auto's source heuristic should remain a fallback to explicit MIME-based selection, not become a new authoritative MIME parser. |
| genet-scripted-dom arena/adoption | Retain full-width node identity, owner resolution, transfer preflight/atomic refusal, mutation records, pins, collection closure and retained detached subtrees. These are ownership invariants, not accidental framework code. Keep replacement_retention, root_closure and subtree_transfer tests. No broad cut based on file size. |
| scripted DOM parser/serialize | Retain html5ever pause/resume, treebuilder-held-node guards, document.write insertion point and html5ever serializer. These already reuse mature parsers; replacing them with source-text surgery would add debt. |
| scripted DOM shadow | Retain separate ordinary/flat trees, template inert owner, eager affected-root reassignment and pruning after sweep. Static slot assignment is one-time; mutable slot assignment must react to edits, so similar walks alone do not establish a removable duplicate. |
| forms values/validation/submission | Retain one native current-value authority, selection metadata, file IDs/immutable byte snapshots, native successful-control construction, constraint validity and user validity. Validation reuses forms' float/calendar validity helpers, so an alleged completely separate scalar grammar is rejected. Shared calendrical numeric helpers could be extracted later without relaxing long-year cases. |
| form_encoding/html_encoding/directionality | Retain pure shared form encoders and encoding_rs; runtime form_encoding is already only a reexport. Retain bounded HTML encoding prescan and generated Unicode17 classification plus generator provenance. These implement distinct byte/HTML semantics; they are not interchangeable with arbitrary text codecs. Existing collision, transport-header, BOM, prescan and directionality fixtures are preservation assets. |
| runtime aggregate / scheduler / harness | Retain browser surface outside VM trait, agent/realm host state, completion status separate from passing assertions, trace evidence and explicit microtask boundaries. R4/R5/R8/R15 are narrower cuts. Harness integration inside HostState is modest coupling; no benefit demonstrated for a wholesale extraction. |
| runtime DOM tree/query/adoption | Retain genuine reflector extraction, OwnedNode ownership resolution, cross-origin refusal and private adoption hooks. These prevent mistaken arena reads after adoption. R1/R2/R11 target query adapters, not the ownership architecture. |
| runtime DOM bootstrap / generated HTML interfaces | Retain WebIDL-derived table, generation drift guard and per-interface reflection metadata. Do not call a generated interface shape a fully implemented interface. Retain prototype/brand/liveness regression tests; shape generation is not proof of semantics. Bootstrap size alone is not a reason to rewrite it. |
| markup insertion / form controls | Retain parser active-stream state, readyState, already-started tracking and native current-value projection. Parser source loading/error boundary is R5/R6. |
| mutation observers / selection / shadow sinks | Retain actual pre-removal ancestry/sibling/old-value facts, checkpoint delivery, JS-owned selection and host geometry projection. Mutation record escaping duplicates the CSS transport pattern and should share a codec or become structured under R3/R4; do not delete factual mutation records or duplicate selection ownership in Rust. |
| runtime forms / FormData | Retain private weak brands, genuine VM extraction, reentrancy guards and native immutable files. FormWire is a strict private schema cursor, not an author-facing general JSON parser. It is a consolidation candidate once the value boundary can extract the required data; do not replace it with author property reads or assume ValueEngine solves this. |
| fetch / websocket | Retain independent request-response and bidirectional protocol seams, deferred completion, abort/demand hooks and fail-closed no-network behavior. They are different protocols, not duplicate FetchHandlers. R4 concerns completion transport, not ownership of networking. XHR reuses fetch surface rather than owning another Rust network stack. |
| streams core/readable/writable/piping | Retain private captured intrinsics, per-agent weak brands, per-realm constructors, native PerformPromiseThen, queue/backpressure, BYOB/transfer and cancellation algorithms. These are substantive behavior with independent regression families, not a fake stub to delete. Similar queue utilities merit later focused consolidation only if semantic getter/order/cancellation rules survive. |
| structured clone / messaging / frames transport | Retain one shared serialization model across same-realm, cross-realm and worker transport, with recipient intrinsics and transfer registry. R12 makes history reuse it. Messaging header comments predate working frames/workers; current frame code implements origin filtering and cannot be judged by those old comments. Cross-agent JSON is a justified transport encoding, unlike same-agent roundtrips used merely because a direct bridge was historically missing. |
| frames Rust/JS / window proxy | Retain one browsing context's stable WindowProxy across navigation, caller-realm security, lifecycle teardown, retained old functions/documents and ordinary same-agent object identity. Existing extensive frame lifecycle/security/adoption tests are preservation requirements. Converting child frames to independent engines would break this architecture. |
| platform / timing / crypto / text_encoding | Retain host storage/provider seam and explicit fallback profile, URL parser reuse, operating-system randomness/fail-closed wasm behavior and encoding_rs streaming decoder state. Runtime random-source documentation saying ChaCha20 default is stale (`lib.rs:2046`); actual crypto.rs uses getrandom natively and refuses an unwired wasm source. R12/R15 target history/clock semantics. |
| webgl all files | Retain optional host-owned graphics factory, real GPU handler tests and external-texture projection. R9/R10 remove working-looking defaults and unbranded ad hoc transport, not the capability seam. |
| worker runtime and wasm WorkerSession | Retain separate native worker agents/channels and browser wasm worker ABI with engine-exclusivity/target guards. WorkerSession snapshots and decimal-u64 event IDs are a legitimate host boundary; its four native unit tests do not qualify browser deployment, browser worker lifecycle or browser-hosted runtime. R7/R8 are native enforceable-lifetime limitations. |
| XPath all modules | Retain distinct generic language implementation (AST, tokenizer, parser, evaluation context, axes, functions, values/node sets). The live DOM adapter is the R11 cut. Generic functions' name/local-name/namespace helpers currently operate through elements (`xpath/src/functions.rs:12-32`); attribute-function parity merits follow-up but was not runtime-tested here. |
| support generators/tools in scope | Retain Unicode hash/provenance generator and scheduler trace converter. Trace converter allowed boundaries (`tools/scheduler_trace_to_tla.py:19-27`) omit the animation-frame boundary now emitted by Runtime, a source-confirmed schema drift to reconcile before calling broad scheduler traces validated. This is a small tool schema fix, not grounds to remove tracing. |

#### Test and qualification limits

Every owned test file was structurally indexed, and selected implementations/assertions were read where cited. There is substantial real test coverage for ownership, adoption, liveness, streams, forms, frame lifecycle/security and working host seams. It would be incorrect to describe the whole runtime as mocks or test scaffolding. Conversely most tests create a bare Runtime or injected handler: that does not qualify physical input editing, real-time scheduling, network policy, browser wasm hosting or GPU presentation. No passing result is newly asserted here.

Suggested cut order is by dependency rather than finding quota: first explicit error/capability/budget outcomes and enforceable worker ownership; then the selector and CSS owners; then retained callable/typed boundaries; finally codec/history/XPath simplification. Preserve the existing semantic/ownership regression families while adding failures that old stubs/defaults hid.

## Layout, style and text evidence


Repository: `/home/markik/Code/repos/genet`, verified HEAD `8fa73497952ecda37cf234c4a845354dab1af599`. Source inspection only. No builds, tests, downloads, source edits, or Git mutations were performed. The only writes are these audit artifacts.

This is a complete architectural pass over the assigned package/module families, not a claim that all 175,371 lines were read. `layout-coverage.json` accounts for all 297 assigned inventory files: 9 full-file reads, 197 targeted structural/body/test-contract inspections, 91 inventory-only files, primarily upstream dependency bulk and examples. Targeted does not mean line-by-line. Fork conclusions concern the maintained local deltas; upstream bulk is not certified. The inventory omits Markdown/patch provenance files, which were additionally read where named below. No dynamic defect or performance measurement is claimed unless explicitly distinguished from source evidence.

Offline `workspace-metadata.json` supplied by the parent audit confirms Livery, Cadency, Buckram, Genet-Livery, Genet-Text, Genet-Parley and Genet-Fontique as workspace members. Genet-Taffy is a maintained patched dependency rather than a workspace member in that no-deps listing; membership is not runtime reachability. No excluded package was classified as unused merely for being outside the 52-member workspace.

#### Ownership map and package decisions

| Package | Present responsibility and module-family coverage | Removal decision |
|---|---|---|
| `livery` | Generated property model (`build.rs`, `properties.toml`, consumed contract); declaration/cascade/custom properties; stylesheet/CSSOM/media/container grammar; selector vocabulary over Cadency; typed lengths/math, colors, logical mappings, layout/typography/background/animation/transform values. | Retain the package and generated catalog. Remove duplicate lexical preprocessing. Consolidate raw-string scanners around a shared token boundary. Do not remove specified/computed/used color distinctions or typed environmental lengths. |
| `cadency` | `ast`, `parse`, `matching`, `attr`, generic `Element`/`PseudoClass`; structural, state, generated, shadow-slot and part selectors. | Retain. This is the single selector grammar/matcher under Livery, not a second cascade. No whole-package deletion justified. Its `cssparser` tokenization is a useful model for Livery's raw scanners. |
| `buckram` | Box generation/provenance, logical flow geometry, fragment identity/reconciliation, block/float/intrinsic/positioned/sticky geometry; all table phases; algorithm scratch tree/backend dispatch; prospective fragmentation. | Retain standards-owned identity and geometry. Taffy fallback ownership and the sealed style seam are retirement debt, not grounds to delete Taffy now. Prospective fragmentation kernel can be isolated from production until connected. |
| `genet-livery` | DOM/Cadency bridge; style scopes/index/plane/counters/hints; retained document, resources, animation, scroll, selection, frame damage; box lowering; block/inline/atomic/table/positioned/query/retained transactions; Parley shaping and paint/placement. | Retain the integration layer. Principal cuts are prospective break-stream work in live shaping, image ownership copies/decode repetition, duplicate DOM dependency classifiers, eager receipt aggregation, and whole-plane frame copies. Keep the publication invariants these currently protect. |
| `genet-text` | Default UAX #29 segments and caret boundaries on an unchanged UTF-8 string; byte/scalar/UTF-16 offsets and pinned Unicode corpus. | Retain. It is deliberately narrower than CSS line breaking, shaping, DOM text collection, or tailored search. Replacing it wholesale with Parley or ICU does not follow from this audit. |
| `genet-taffy` patch | Low-level flex/grid/leaf/block fallback; float exclusion accessor, flex order/content-basis/automatic-minimum fixes, static position/grid callback, physical size containment, cache partitioning. | Retain current runtime dependency and required fork delta. Retire individual patches only after equivalent upstream/owned behavior and consumers are demonstrated. |
| `genet-parley` patch | Shaping/line breaker; join-control coverage/spacing fix, actual-cluster fallback query, trailing-whitespace API, opt-in correlated diagnostics. | Retain current runtime dependency and patches. The redundant speculative projection belongs to the Genet adapter, not to Parley itself. |
| `genet-fontique` patch | Font collection/query/backends; actual-cluster DirectWrite fallback with attribute and complete-range constraints. | Retain paired seam with Parley. Do not restore script-only Windows caching or collapse text-specific results into a script key. |

#### Findings and cut directions

##### L1. Duplicate comment stripping is an active lexical correctness defect

**Status: confirmed by source, high confidence.** `components/livery/src/cascade.rs:139` and `components/livery/src/stylesheet.rs:1108` contain the same character-level `without_comments` algorithm. Neither tracks strings or escapes. A quoted string containing `/*keep*/` loses that content before parsing; this conclusion follows directly from the two bodies. The parent audit supplied an already-confirmed standalone verbatim probe; no new probe or build was run in this domain pass.

The architecture cost is two independent preprocessors before the actual CSS grammar. Consolidating them into the same faulty helper would merely centralize the defect. Remove both raw preprocessors in favor of CSS tokenization or one token-aware traversal preserving string/URL tokens and source locations. Adjacent declaration splitting already has independent quote/depth logic at `cascade.rs:159`; stylesheet braces have another at `stylesheet.rs:1128`, `:1152`, and descriptor splitting another at `:1723`.

Preserve declaration/rule recovery, custom-property token boundaries, CSSOM authored text, strings with comment-looking bytes, and escaped delimiters. Add declaration and stylesheet fixtures for quoted content and URLs, real comments, escaped quotes and nested blocks. Existing `livery/tests/stylesheet.rs` and `custom_properties.rs` cover recovery/substitution but no result from this audit proves the quoted-comment case is protected.

##### L2. The future line-break model performs production work while Parley still owns line selection

**Status: confirmed unnecessary production projection under the current implementation, high confidence.** In `components/genet-livery/src/text.rs:1524`, every shape creates an unbounded clone of the Parley layout, breaks it without width, then constructs `LogicalBreakStream`. The finite layout independently calls `break_inline_lines` at `:1548`. The adapter subsequently indexes clusters, refreshes finite flags, and records selected breaks (`:1557-1601`). This is not the algorithm selecting finite line positions.

The model calls itself dormant S1.1 at `:3268-3284`; its permission assertion is `#[cfg(test)]` at `:3287`. It carries copied paragraph text, cluster glyph/source/font/coordinate metadata, CSS candidates, atomic opportunities, and selected breaks (`:3399-3441`, `:3462-3709`). Results travel through `Shaped.break_stream` (`:3256`) and are parked in `InlineLayout._break_stream` (`:2189`; construction `:674`, `:685`). Repository reader search found production construction and bookkeeping, test assertions/readers, and this underscore-owned retention, not a production finite-break consumer. `icu_line_candidates` at `:3905` is reached by this future projection; it is not evidence that ICU currently overrides Parley's chosen finite lines.

Cut direction: compile the unbounded projection/candidate ledger behind an explicit diagnostic or experimental breaker feature, or make it test-only while preserving its tests. If S1.2 is intentionally being built now, connect it to a chosen production authority and delete the parallel finite authority at that cutover. Do not maintain both indefinitely.

Preserve Parley's present glyph selection, bidi, tabs, text-indent, float-band retry, finite line metrics, original-to-transformed source offsets, emergency/regular distinction, atom insertion order, hidden/generated text and caret/selection metadata. Compare emitted commands and geometry with the projection disabled, with representative bidi, preserved whitespace, SA-script fallback, atoms, floats and font features. This audit establishes the extra calls/allocations, not their measured wall-clock share.

##### L3. Paint output owns copies of source images and repeatedly decodes the same resource

**Status: confirmed, high confidence.** `LiveryDocument` owns image bytes (`components/genet-livery/src/document.rs:215`); `LiveryPaintList` also owns `HashMap<String, Vec<u8>>` and derives `Clone` (`paint.rs:50-61`). `with_image_sources` clones the entire host ledger even when most resources are not painted (`:497-509`). The emitter creates that output at `:746`. `image_key_for` clones the selected encoded bytes and decodes to owned RGBA (`:517-537`), with deduplication only in that fresh list's `image_keys`.

Independently `layout.rs:2910-2943` clones and fully decodes an image merely to ask dimensions. Atomic/container/positioned/layout passes can revisit that function. `document/frame.rs:301-306` clones a cached list on every cache hit; `:626` clones a freshly emitted list into the cache. Thus fresh paint-list decoding, layout dimension decoding, cached encoded-source cloning, and RGBA-list cloning are separate visible costs. A cached hit does not decode again, but it still clones owned output payloads. That distinction matters.

Cut direction: document/resource owner holds encoded bytes and versioned decoded metadata/pixels once; paint holds immutable resource handles/references or shared payloads, not a second source ledger. Separate the emitter's borrowed resource lookup from the serializable output. Preserve replacement/deletion invalidation (`document/resources.rs:21-57`), data URLs, stable list-local key semantics or a deliberate replacement, absent/decode-failed images, feature-off behavior (`image_decode.rs:23-34`), natural ratio and source updates. Add decode-count/byte-sharing receipts across dimension query, fresh paint, cached scroll and replacement. Existing `replaced_image.rs` checks natural sizes/ratios; `paint.rs` checks output, not this allocation/decode ownership contract.

##### L4. Transform context facts have drifted between box ownership and paint

**Status: source-confirmed predicate mismatch, high confidence; concrete runtime misplacement unexecuted.** `components/genet-livery/src/box_tree.rs:273-281` establishes absolute/fixed containing blocks for `transform`, `contain`, or positioning. Its transformed predicate only reads `computed.transform`. `paint.rs:3142-3147` also recognizes individual `rotate`, `scale`, and `translate`; `element_matrix` composes those at `:3207-3217`. They are implemented properties, for example `livery/properties.toml:1561`, `:1571`, `:1831`. The positioned pipeline reads the box graph's containing-block selection (`layout/positioned.rs:482`), so the discrepancy reaches geometry ownership rather than being an unused predicate.

Cut direction: own supported transform/context facts once and expose the distinct derived facts each consumer needs: establishes containing block, stacking context, paint transform, and affected reference box. Consumers may have different CSS applicability rules; sharing one unqualified boolean would be too crude. Audit perspective alongside individual transforms rather than assume paint support automatically means its containing-block semantics were implemented.

Preserve inline applicability, fixed versus absolute selection, containment, anonymous table wrapper/grid ownership, grid static area and retained-motion admission (`style.rs:752-802`). Add static-ancestor fixtures with each individual property and absolute/fixed descendants, including `none` to non-`none` mutation. Existing transform tests prove parsing/projection and some transform hit behavior, not this source-mismatch case.

##### L5. Stateless geometry and retained text geometry are separate production semantics

**Status: confirmed API divergence, high confidence; not all stateless use is obsolete.** `layout/transaction.rs:15-17` explicitly says the public `layout()` uses deterministic text estimates, while retained sessions use Parley. Its implementation sends `BuildState.text = None` (`:180` onward), unlike `layout_with_text_system` (`:90-166`). `used_value_context` clones styles, clears margins and calls this estimated entry (`:47-78`). That public CSSOM measurement helper has neither the live font ledger nor host image ledger. It can therefore produce a different geometric basis from the visible retained document; the source proves the input/model distinction, not a particular failed CSSOM assertion.

Cut direction: keep a clearly named estimate helper for deliberately synthetic tests if needed; route user-visible geometry and CSSOM used-value queries through the current retained layout or the same text/resource-aware transaction. Do not let tests using estimated geometry qualify live text wrapping. Preserve the special queried-margin exclusion and table principal-grid versus caption-wrapper rule in this helper.

Tests should distinguish synthetic structural arithmetic from live shaped geometry. Existing host `ch` tests, line-box tests, out-of-flow tests using both entries and retained document tests should survive; add a query against a changed registered font, wrapped text, and loaded image, compared with the actual retained fragment basis.

##### L6. DOM dependency classification is duplicated across style invalidation and layout damage

**Status: confirmed duplication, high confidence.** `invalidation.rs:417-487` and `document/frame.rs:708-768` independently recognize radios and attributes affecting radio groups/form association. Aggregate-validity and select-ancestor traversal is likewise separately expressed in `invalidation.rs:364-415`, `:508-535` and `frame.rs:168-270`. The frame mutation switch (`:47-140`) parallels the style invalidator (`invalidation.rs:170-249`).

Cut direction: share DOM dependency facts and affected semantic owners once. Keep style-root selection and formatting-context-root selection separate; they serve different invalidation planes. A common callback/iterator of affected owners is preferable to collapsing these planes. Preserve old `type=radio`, external `form=`, earlier non-form ID blockers, fieldset/select aggregation, former and current parents, removed node handling, and state versus interaction/custom-validity mutation distinctions. Exercise retained geometry and selectors from the same mutation fixture and compare each with fresh layout. Broad style invalidation is sometimes the correctness fallback; do not narrow it merely to reduce root counts.

##### L7. Retained rendering still copies whole style and fragment planes for local work

**Status: source-confirmed copy topology, high confidence; performance priority needs measurements.** `document/frame.rs:487-488` clones previous styles/fragments before attempting a local root; a successful local result clones the prior fragments again at `:506`. `paint_active_layout` clones the active style and fragment planes (`:605-608`) on a repaint. Sticky application clones fragments again to obtain scrollport geometry (`:645`). This is compatible with correctness but limits the benefit of doing local layout.

Cut direction: use borrowed immutable base state and small animation/sticky overlays, or a transaction with changed subtrees and one accepted publication. Do not simply delete clones that currently prevent animation/sticky mutation from corrupting retained geometry. Keep fresh local box IDs, text frames, table paint state, ledger remapping and overflow together; `layout/query.rs:103-110` and the replacement methods explicitly maintain this publication contract. Verify unchanged subtree identities and text/table side planes after local mutation, animation ticks, sticky scroll, fallback and rejection. Existing retained document and retained-motion tests are relevant but were not executed.

##### L8. Per-table diagnostics duplicate authority and rebuild aggregates eagerly

**Status: confirmed source complexity/duplication, high confidence; unbenchmarked impact.** `TableShadowLedger` stores aggregate counters/vectors and `per_table: HashMap<BoxId, Box<TableShadowLedger>>` (`table_shadow.rs:103-125`). Every `record_table` and `take_table` rebuilds the aggregate (`:171-184`); `rebuild_summary` traverses all contributions, cloning/extending detailed vectors (`:233-258`). Successive table publication/verification therefore repeatedly revisits already-completed receipt data. For N growing independent entries, these rebuild walks accumulate quadratically in entry count; no benchmark was run.

Cut direction: one per-table entry type as authority and a derived aggregate view, or delta maintenance/batched summary construction at publication. Public aggregate fields and diagnostics consumers constrain API changes; preserve observable counts and detail order deliberately. Keep retained subtree replacement/remapping (`:145-227`) and table block counters; deleting the ledger would remove explicit unsupported/divergence evidence. Add a many-table operation/count receipt and compare the aggregate after replacement/removal with a fresh run. The name `table_shadow` is stale: its sizing path is live, not an optional comparison backend (`:7-18`, `:358`).

##### L9. Buckram's backend boundary still has competing block styles and genuine fallback debt

**Status: confirmed architecture debt, high confidence; no immediate whole-backend cut.** `buckram/src/taffy_adapter.rs:155-178` carries both a standards `BlockStyle` and generic backend style. The sealed trait's only implementation is Taffy's `Style` (`:130-157`), while Genet directly constructs Taffy styles in `genet-livery/src/layout/taffy_style.rs:16`. The generic name does not make this boundary backend-neutral.

Dispatch is real: `buckram/src/taffy_adapter/run.rs:1755-1784` tries owned block layout, records a named deferral and invokes Taffy block, and dispatches flex/grid directly to Taffy. `BlockDeferral` enumerates concrete unresolved states (`block.rs:470-486`). Float-context entry can force backend sizing mode. Conversely Table dispatch deliberately returns precomputed Buckram geometry and does not recurse (`run.rs:1785-1794`). Removing Taffy today would remove active flex/grid, leaf behavior and deferred blocks; replacing tables with Taffy would reverse a completed ownership transfer.

Cut direction: eliminate dual block-style construction only as each named block deferral is replaced by an owned input/algorithm contract. Either make the remaining boundary honestly Taffy-specific or finish a backend-neutral style contract before promising substitution. Keep the erased measure callback/`AlgorithmCore` split: `run.rs:13-34` and `taffy_adapter.rs:206-214` explain real algorithm monomorphization reduction. Undoing it by merging every generic context into the algorithm core would reintroduce duplication.

The thread-local calc index store (`genet-livery/layout/taffy_style.rs:806-845`, reset `:1064`) is another boundary obligation. It intentionally round-trips opaque tags without dereferencing them and resolves against a real basis. A per-transaction owner would make lifetime/reentrancy constraints clearer, but no dangling-pointer defect was established. Do not replace percentage-bearing math with a zero basis or remove cache sizing-mode partitioning.

##### L10. Fragmentation contains a prospective kernel and live unsupported markers, not working multicol

**Status: confirmed, high confidence.** `buckram/src/fragmentation.rs:13` suppresses dead-code warnings for the module. The ordinary block emission/resume functions (`:304`, `:316`) are crate-private and reference search found their callers in the module's synthetic tests, not the live adapter. `BlockBreakToken.carried_margin` explicitly calls its state provisional (`:216-225`); float/clearance/inline state is typed deferred (`:184-211`).

There is live input lowering for a narrow column shape (`genet-livery/layout/build_block.rs:15-71`, attached `:814-819`), but `buckram/taffy_adapter.rs:1026-1035` records `SequentialMulticolUnsupported`. This marker is an honesty mechanism, not continuation geometry. Six retained block/inline/table contracts remain ignored (`genet-livery/tests/k6_fragmentation_contracts.rs:160`, `:174`, `:190`, `:202`, `:214`, `:238`). A synthetic block resume kernel existing does not make the first ignored end-to-end contract implemented; its ignore message is now imprecise about which layer is absent.

Cut direction: move the disconnected kernel/provisional carried state behind a development feature or keep it explicitly under construction with one owner and end-to-end activation gate. Retain public one-to-many fragment identity and explicit unsupported dispatch as long as those contracts remain part of the API. Do not delete fragmentainer identity merely because the current continuous route uses one fragment. Preserve ignored acceptance fixtures and their distinction from synthetic tests.

##### L11. Logical-axis facts are repeated across value and geometry layers

**Status: confirmed duplication; medium priority.** `livery/src/values/logical.rs:14-108` and `buckram/src/flow.rs:14-116` separately represent writing-mode/direction side mappings; `genet-livery/src/box_tree.rs:292-307` converts between their enums, and `layout/taffy_style.rs:175` adds backend physical flex mappings. These are distinct layers but share CSS axis arithmetic that can drift, especially sideways modes.

Cut direction: share a tiny dependency-neutral flow vocabulary/math module if doing so reduces repeated normative tables without inverting the value/layout dependency. Otherwise retain explicit conversions and add cross-owner mapping parity tests for every direction/writing mode. Do not merge complete computed style and geometry crates to remove a small enum. Both sides have meaningful tests already (`logical.rs:110` onward; Buckram `flow.rs` tests); neither is dead by source inspection.

##### L12. The consumed-property census is historical, not a current runtime reader proof

**Status: confirmed evidence-boundary problem, high confidence.** `livery/consumed_longhands.toml:1-24` calls its 129 entries the current product path, but records an old revision and consumer audit; `tests/consumed_set.rs:31` reads that fixed table. Current implemented properties extend beyond it, and its `order` row labels only `R` while `layout/taffy_style.rs` lowers backend order and the maintained flex algorithm uses it. This contract is useful historical preservation, not evidence that every present consumer was enumerated.

Cut direction: keep the historical baseline named as such or generate/check a current consumed projection from actual code/declared consumer mappings. Do not delete the generated `KnownUnimplemented` catalog as dead code: `livery/build.rs:24-31` and `cascade.rs:109-121` deliberately distinguish recognized-but-unimplemented from invalid names. Preserve partial-property status so parser acceptance cannot masquerade as runtime implementation. Relevant catalog/property-space/consumed-set tests were inventoried, not executed.

##### L13. Feature-enabled font diagnostics build discarded events even outside capture

**Status: confirmed conditional overhead, high confidence; low priority.** With `font-diagnostic` compiled, `parley/src/shape/mod.rs:620-629` builds a cluster String and diagnostic object and fills candidate strings/vectors (`:632-709`), then `font_diagnostic.rs:91-98` checks whether capture is enabled only when recording. An inactive capture therefore avoids retention but not event construction. The feature is opt-in; Genet-Livery's dev dependency enables it (`components/genet-livery/Cargo.toml:44`), so do not describe this as default release overhead without checking the consumer feature graph.

Cut direction: query active capture before collecting event payloads, keeping the existing feature and RAII/thread-local capture design. Retain diagnostics, which qualify the Windows fallback patch and selection behavior. Add an inactive-capture work/allocation assertion only if this optimization is pursued.

#### Important rejected cuts and retained mechanisms

* **Reject “layout damage is unused.”** The comment at `document/frame.rs:30-32` is stale. The frame consumes `last_layout_damage` to select and promote a local formatting root (`:475-548`); style-only transform/background shortcuts and positioned resize/reposition paths occur before that (`:352-472`); full-layout identity/subtree reconciliation follows as fallback (`:550-596`). Keep damage and retained-layout tests, correct the comment. Conservative fallback is active correctness policy.
* **Reject “delete table_shadow/table_sizing as an abandoned shadow adapter.”** The former runs fixed/automatic Buckram columns (`table_shadow.rs:358-542`), the latter lowers CSS constraints, collapsed borders and logical edges consumed by that live route and the block pipeline (`table_sizing.rs:37-146`, `table_block.rs:157`, `:447`). `buckram/table/pipeline.rs` owns phase order. They are not independent competing table engines. Rename stale concepts and reduce duplicate lowering only after tracing the shared `DirectBoxSource`/builders and contracts.
* **Reject “delete separate table wrapper/grid styles.”** `table_wrapper.rs:64-105` intentionally splits property ownership and resolves percentages against the wrapper's containing block. Comments at `:25-33` are stale: clip-path, perspective, individual transforms now exist in the catalog. This is a follow-through audit item for the table property list and context owner, not proof that replacing the wrapper with the grid is safe. Captions, percentage dimensions, positioned-inline and replaced-element guards must survive.
* **Reject “all caches compete.”** Buckram `IntrinsicSizeCache` (`intrinsic.rs:84-151`) answers standards min/max box queries with explicit cycles/deferred results; Taffy `Cache` stores scratch algorithm `LayoutInput`/`LayoutOutput`; `TextSystem` (`text.rs:214-224`) retains fonts and metric probes; `TextFrame` retains current glyph/selection geometry; `StyleSet` owns derived flattened rule/index views (`style.rs:268-283`). Their identities/lifetimes differ. Remove duplicate payloads/recomputations, not distinct semantic caches. Preserve `SizingMode` in `support/patches/taffy/src/tree/cache.rs:89-99` so automatic-minimum measurements cannot reuse another mode.
* **Reject “handwritten grammar means replace all parsers.”** Cadency uses `cssparser` tokens (`parse.rs:11`, `:77`), Livery math/color/content use typed token grammar, while raw lexical boundary scanners are the weak point. HTML legacy color/dimension parsers (`legacy_color.rs:17`, `presentational_hints.rs:1423-1585`) intentionally have different rules from CSS. Text fragment directives (`text_fragment.rs:74-157`) own navigation splitting/percent decoding. Preserve their domain semantics instead of feeding everything through CSS.
* **Reject “generated pseudo text duplicates DOM.”** `style/counters.rs` resolves generated strings, `box_tree.rs` creates pseudo provenance, and `rendered_generated_text` (`document.rs:263`) exposes visible generated text without source DOM nodes. This is intentional projection. Original offsets, generated visibility/suppression, counter scope and invalidation remain essential to paint, selection and accessibility consumers.
* **Reject “font context rebuild is just waste.”** `document/resources.rs:59-127` rebuilds after removal/change because per-blob removal is unavailable. This prevents stale font faces and metric caches. Optimize payload sharing separately; do not retain a removed face to keep a cache warm. Bundled-font deterministic geometry is a requirement, with Windows platform fallback qualified separately.
* **Reject “the placement module is a second transform parser.”** `placement.rs:183-232` calls paint's resolved transform/clip helpers; it does not parse CSS again. It shares order helpers with hit testing (`layout/hit_testing.rs:46-65`) and handles inverse mapping, capture and clipping. There remain separate paint/hit traversal scopes that require parity tests; wholesale deletion would break host/input geometry.
* **Reject “stack growth can go after simplifying recursion.”** `genet-livery/lib.rs:41-79` and Buckram's descent guard exist for deep document recursion and small debug stacks. `tests/deep_nesting.rs` explicitly exercises this. No iterative replacement was built or validated here.

#### Maintained fork assessment

##### Taffy

The authoritative local record is `support/patches/taffy/0000-complete-fork-delta.patch`, not a claim that a few upstream patch files reproduce the tree. Its hunk inventory covers 16 files: `compute/{block,flexbox,float,leaf,mod}`, `compute/grid/{alignment,mod}`, `prelude`, `style/{compact_length,flex,mod}`, `tree/{cache,layout,taffy_tree,traits}`, and `util/print`. `GENET_PATCHES.md:27` onward records the byte-roundtrip rule and release provenance; those historical receipts were not rerun.

Preserve float band APIs (`compute/float.rs:734-781`, `compute/block.rs:180-216`), flex stable order (`compute/flexbox.rs:275-279`, `style/flex.rs:205-215`), content-basis and automatic minimum (`compute/flexbox.rs` local delta; `tree/cache.rs:89-99`), static `Layout` coordinates (`tree/layout.rs:233`), grid static-position callback (`tree/traits.rs:276` onward; Buckram implementation `taffy_adapter/run.rs:2146`), and size containment. The high-level `TaffyTree` and example families are upstream bulk; Buckram uses its own scratch tree with low-level algorithms. They may be absent from a production feature closure without deleting the maintained fork's tests/source or claiming dependency removal.

Unchanged-upstream geometry/style/grid utility families were inventoried, not independently byte-compared with pristine upstream. No downloaded pristine tree was used. Patch prose that refers to retired `genet-layout`, document-order paint, or old version consumers is historical; current paint uses `order_modified_children` and tables are Buckram-owned. Preserve reproducible delta records and retire stale prose independently of runtime code.

##### Parley

Patch log and source confirm join controls are excluded from coverage (`analysis/mod.rs:530-540`) and letter spacing (`layout/data.rs:547`), while shaping continues to carry source characters. Keep both halves. `LineBreakData.trailing_whitespace` is exposed and filled (`layout/line_break.rs:98-102`, `:350-357`), and the Genet float-band retry subtracts it (`text.rs` live `break_inline_lines` family). Do not remove it because the line's total advance is also available.

Actual-cluster fallback is invoked only after authored font candidates fail (`shape/mod.rs:711-717`) and calls Fontique's matching text seam. Diagnostic data is feature-gated and RAII/thread-local; L13 concerns collection overhead, not its evidentiary value. Editing, accessibility, bidi, resolve, font/shape cache and style/builder modules are upstream families inventoried outside local-delta review. Removing unused editing APIs from a consumer feature closure is different from eliminating the shaping dependency. No upstream replacement was established.

##### Fontique

`collection/query.rs:116-146` chooses actual-text fallback on Windows+system and the existing script route elsewhere, clears script-cache identity for text-derived candidates, and keeps explicit fallback families. `backend/dwrite.rs:269-307` requires a complete UTF-16 mapped range and scale 1.0 while passing requested attributes. Preserve this paired policy with Parley; a cache keyed only by script/locale would resurrect the original mismatch.

Fontconfig/CoreText/Android/DWrite enumeration, family/font attributes, source/charmap/matching/fallback/scan/source-cache families remain upstream implementation. Their existence is not redundant font ownership: they discover/select fonts, while Genet retains document source identities and render font resources. Cross-platform code is inventory-only except the named local seams. A Linux source audit cannot qualify Windows DirectWrite behavior or prove a newer upstream version includes the fix.

#### Test and evidence boundary

All integration test families listed in `layout-coverage.json` were inventoried, including anonymous boxes/tables, block admission, Cambium, CSS text/CSSOM, container/tree counting, contextual colors, selector indexing/shadow flat trees, generated content/counters, controls, images, flex/grid/positioning/writing modes, hit/scroll, text fragments/slots, transforms, line boxes and retained invalidation. Selected risk-bearing contracts and ignored markers were read. No test suite passed or failed in this audit because none was run.

Known acceptance gaps retained: six K6 continuation fixtures, nested-table distribution repro (`tests/table_sibling_blocks.rs:268`), contextual-color tests intentionally deferred to retained consumers (`livery/tests/contextual_color.rs:265`, `:284`, `:319`, `:344`), and Windows-only font/line-scroll/text geometry fixtures. Neither ignored tests nor platform cfg bodies count as runtime qualification.

`genet-text/tests/conformance.rs:62-82` loads pinned Grapheme/Word/Sentence corpus data and checks a nontrivial case-count floor. Preserve it and its Unicode-version coupling. It is a distinct conformance contract from CSS whitespace, line breaking and font selection.

The useful cut order follows ownership: first remove duplicate lexical preprocessing and isolate prospective break-stream work; then share versioned image payloads and DOM dependency facts; then unify context classification and route live geometry queries through the retained owner; then reduce frame copies/receipt aggregation; finally retire backend deferrals only as accepted owned contracts replace them. Done means preserved observable geometry, paint, input, offsets, CSSOM and explicit unsupported receipts, backed by focused regressions and the relevant existing suites. No package-wide deletion is justified by file size or a stale name alone.

## Host, rendering and tooling evidence


Snapshot: `/home/markik/Code/repos/genet`, HEAD `8fa73497952ecda37cf234c4a845354dab1af599`. Read-only source audit; no build, test execution, downloads, repository edits, or Git mutations. Only these audit artifacts were written. Paths below are repository-relative unless absolute. Certainty means source evidence, not runtime qualification. Full/targeted/inventory coverage is recorded per file in `host-tool-coverage.json`; targeted structural review is not a claim that every line was read. External Forms evidence is inventoried, with representative execution/provenance machinery traced, rather than all historical transcripts re-audited.

#### Findings and cut directions

##### H1. Delete the obsolete GL capability stub and handle (high confidence, small cut)

`components/genet-compositor/rendering_context_core.rs:79-114` explicitly reinstates `GlCapability` and `GlCapabilityHandle` solely for old import points. `make_current` returns `Ok(())` without a GL context at lines 88-91. A Rust-source search across Genet components/ports, Mere, and Turnstone found only these definitions and associated comments, not a consumer. The header at lines 7-10 even says this trait is deleted. Remove this phantom capability, handle, implementations, and contradictory lineage prose. Keep `RenderingContextCore` and `WgpuCapability`: `components/webgl-wgpu/lib.rs:122-132` actively consume them. Test gap: compile the affected public imports and WebGL capability route after removal, including downstream consumers; no claim about unknown third-party users.

##### H2. Migrate browser compatibility renderer access to checked ownership (high confidence, API migration)

`components/genet-render-host/src/lib.rs:747-753` calls fallible `try_renderer().expect(...)`. The accessor can refuse normally while a shared frame owns attribution or after loss: `browser_frame.rs:23-26,42-47,255-261`. Returning an owned access guard is correct; turning refusal into panic is the compatibility debt. `ports/ortet/src/web.rs:161-190` still rasterizes, acquires via Option, calls this accessor, composes, and presents outside the checked browser-frame transaction. Remove this host's dependence on infallible compatibility access by adopting `try_begin_browser_frame` and its accepted-frame path, preserving staged resources, scopes, queue completion, counters, loss priority, and finite deadlines. Migrate downstream raw getter callers before removing the getter itself; Mere already has checked access in `crates/cambium/cambium-rootstock/src/lib.rs:220-258`. Native access is a different API at render-host `lib.rs:740-744` and cannot be mechanically treated as the same refusal seam. Tests needed: two hosts on one Core, busy/reentry, mid-frame loss, cancellation/drop, and refusal without present. Do not delete the gate, guard, or recovery state to simplify the getter.

##### H3. WPT animation-frame behavior is disconnected from shipped scripted sessions (high confidence)

The only non-test `run_animation_frame_callbacks` call in the searched Genet/Mere/Turnstone Rust sources is `ports/genet-wpt/src/harness.rs:903`. WPT explicitly tracks pending callbacks in its virtual/wall loops at lines 1246 and 1353. Both real document pumps, `components/genet-scripted/document.rs:386-412` and `1434-1456`, run timers/microtasks/workers without running animation callbacks. Its `has_pending_work` / `has_external_work` at `1459-1474` omit pending rAF, and `components/genet-documents/src/engines/scripted.rs:422-441` delegates only timer/external work to the host. This is a confirmed source-level wiring gap, not a fresh browser test result. Move/share the actual rendering-turn scheduling contract with production document/session driving; retain harness virtual time and network completion adaptation. Also remove `unwrap_or(0)` error erasure in harness line 903. WPT `RenderSession::animating` is literally false at `harness.rs:886-888`; lines 891-899 document that this style route lacks the static Livery CSS animation clock. Do not use scripted CSS animation counts to qualify production/static animation behavior. Required regressions: rAF-only pending document wakes, mutation/presentation ordering, callback cancellation, hidden/frozen documents, each engine, and parity with WPT.

##### H4. Browser key input discards the effect needed to gate defaults (high confidence)

`ports/ortet/src/web.rs:216-239` gets the full typed `SessionInputResult` but returns only its `capture` field. The keydown handler at `655-667` discards that return and attempts `scroll_for_key` even when the session handled/cancelled the key or an editable control owns it. Native `shell.rs:1263-1277` correctly requires `!handled && !editable`. Consolidate the input outcome/default-action decision, retaining browser DOM event translation and pointer capture handling. Preserve navigation/form effects and address synchronization. Tests needed: Arrow/Home/End/Space in editable controls, script-cancelled keys, handled control keys, ignored keys, browser `preventDefault`, and no extra page scroll. Existing web unit tests near line 521 cover redirect addressing rather than this keyboard behavior.

##### H5. Native Ortet fetch ignores the public resource budget policy (high confidence)

`components/genet-host-api/lib.rs:230-245` defines fetch concurrency, response-byte, and timeout policy. `ports/ortet/src/fetch.rs:97-113` owns a Tokio runtime and `FetchContext::permissive()` without that policy. At `209-234` it synchronously blocks on fetch, then fully buffers `response.bytes()` with no per-resource byte cap, timeout, or corresponding permit. Searches of first-party Rust consumers find policy definitions/reexports plus browser use, not native enforcement. Browser code imports the default policy at `web.rs:50-54` and checks streamed response lengths at `462,490`; native and browser limits consequently diverge. Remove the dormant policy facade by wiring one configured policy through native and browser adapters, with counted incremental reads, actual elapsed deadline and request admission. Do not remove the policy fields as unused convenience. The global receipt completion deadline cannot interrupt synchronous blocked fetch. Preserve protocol delegation to netfetcher and local/scheme split. Tests needed: chunked body without Content-Length, oversized/slow response, bounded parallel requests, redirects preserving limits, and exact failure reporting. Transport defaults were not executed or qualified.

##### H6. Test262 module/async negative classification can report infrastructure failures as pass (high confidence)

`ports/genet-wpt/src/test262_cmd.rs:91-115` collapses module execution to a boolean. Runtime creation failure at 95-96 and failed preamble at 98-99 become `threw=true`; any negative metadata makes that pass. Unlike the script route at lines 40-66, the module path does not validate negative type/phase. Async route at `135-166` similarly treats failed runtime creation, synchronous throw, and missing successful `$DONE` as the same boolean that passes a negative test. Additionally `run_262` branches on async before module at `34-38`, and async runs `rt.eval` at 146, so async module tests take the classic-script path. Cut this boolean classification duplication. Use typed setup/execution/completion outcomes, preserve infrastructure errors separately, combine module+async flags correctly, and share the negative matcher without accepting unrelated failures. Keep subprocess isolation and finite timeout. Tests needed: invalid harness, runtime-init failure, wrong error type/phase, module import failure, async module imports, missing/failed DONE, and a proper negative success. No matching classification tests appeared in the inspected WPT tests file.

##### H7. Retire the stale profile-gate paths and make resolution failure fail (high confidence)

`support/profile-gates/check-tiers.sh:25-26` suppresses Cargo stderr and puts `|| true` around the pipeline. With `pipefail`, a failed `cargo tree` is converted to empty success; lines 31-32 print clean. The old `check-static-html.ps1:14-24` neither checks native exit codes nor names the modern engine crates in its short substring blacklist. `README.md:22-23` already says that old script is superseded. Its lines 25-38 claim there is no CI and sibling engine paths block CI, contradicted by `.github/workflows/ci.yml` and immutable-dependency work. Consolidate tier checks into the current `support/ci/check_dependency_cones.py` witness family (main at 754-770), or fix one canonical gate before deleting both obsolete entrances. Keep low-tier cone assertions and add explicit Cargo-failure negative control. Current CI does not call these profile scripts. Done condition: failed metadata/tree resolution is a failing gate, every intended static tier is checked, and docs point at the CI-owned command.

##### H8. Build/receipt wrappers can announce success after failed native commands (high confidence for absent checks; conditional stale-artifact risk)

`support/build-scripted-workers.ps1:17,29-30,35-53,60-75` does not check `$LASTEXITCODE` after metadata/toolchain/build/bindgen native commands, yet prints generated success. `$ErrorActionPreference='Stop'` alone is not a portable native-command exit guarantee. Existing target artifacts can therefore be reused after failed compilation unless the calling PowerShell environment independently enables native error propagation. `support/ci/run_ortet_chromium_receipt.ps1:31-32` has the same missing build/bindgen checks, while the HTTP counterpart explicitly checks at lines 33,35. Introduce one native-command helper and restore changed environment variables (Chromium script overwrites `CARGO_TARGET_DIR` at line 30 without restoring it). Keep the engine/wasm64-vs-wasm32 artifact split and exact wasm-bindgen version requirement. Tests needed: deliberate failing cargo and bindgen with old outputs present, no output success marker or browser launch afterward, restored environment on all exit paths.

##### H9. Remove the dormant Wayland timeline allocation from the same-queue route (medium confidence, bounded cut)

`components/genet-compositor/compositor_wayland/mod.rs:99,165-172` constructs and retains `vk_timeline_sync`; search finds no use of that field beyond construction/storage. Its `sync_mechanism` at `697-702` explicitly reports `None` and says the wrapper is dormant on the smoke route, which submits to the same host queue at 309,473. This makes an unused semaphore prerequisite capable of rejecting construction through `SyncInit`. Remove eager allocation/storage from this route or construct it only for a real multi-queue protocol. Preserve the reusable interop primitive and external-memory release/slot ownership, plus actual DX12 fence and Vulkan synchronization wherever used. Source search proves dormancy, not platform presentation correctness. Required proof before cutting: same-queue synchronization audit including compositor consumption/release, Wayland hardware smoke with slot reuse, and a real multi-queue test if that route is retained.

##### H10. Legacy compositor failure reporting needs an observable outcome (high confidence, redesign boundary)

`components/genet-compositor/compositor.rs:256-261,389-395` logs and skips allocation failure. Platform present methods return no result; Wayland `mod.rs:704-708,732-736` converts actual failures to warnings. The factory's explicit fallback `compositor_factory.rs:214-233` similarly returns capture-only output after logging OS handoff failure. That fallback is intentional and can remain for capture consumers, but it cannot establish visible presentation success. Replace ambiguous successful-frame accounting at its host boundary with outcome/status reporting; do not delete OS backends based on lack of ordinary Ortet use. Mere retains real platform smoke consumers: `mere/ports/pelt/desktop/smoke_windows.rs:120,252`, `mere/ports/pelt/desktop/smoke_wayland.rs:262`, `mere/ports/pelt/desktop/smoke_macos.rs:253`. Tests needed: allocation, unsupported window handle, OS commit/present failure, and fallback capture clearly distinguished from display handoff.

##### H11. Remove stale proposal/lineage commentary from live implementation (high confidence, documentation cut)

`components/genet-render-host/src/surface_capture_layout.rs:1-2` calls itself an unapplied/uncompiled output-only proposal, but `lib.rs:42-53` compiles/exports it under actual diagnostic/test cfgs. `components/genet-compositor/compositor.rs:270-271` calls implemented blit machinery a future milestone. Correct these to current scope and preserve historical detail in existing design/receipt documents. No code deletion follows from the stale wording. Layout row/aggregate budget guards are necessary.

#### Retain/no-cut disposition for every owned family

| Package / family | Disposition and reason |
|---|---|
| `genet-render` (`lib`, `livery_render`, `inspect`, `a11y`, `a11y/name`) | Retain translation, geometry/caret, content and style-owned accessibility projections. Retained Livery session functions and one-shot ScriptedDom convenience functions are distinct uses; do not bulk-delete based on similar names. `livery_render.rs:515-524` scrollbar wrapper merely delegates and is a small compatibility migration candidate, not an architectural blocker. `_with_text_system` retains caller font ownership. `_with_style` deliberately filters current CSS visibility (`a11y.rs:905-948`); collapsing it into simpler projections loses semantics. Accessible-name code owns tree scopes/labels and is not a generic platform bridge. |
| `genet-render-host` core | Retain the target-neutral device/surface boundary, typed acquire/loss/error and checked raster protocols. Native/browser qualification differ intentionally. `WindowSurface::acquire` at `lib.rs:1375-1379` is a lossy compatibility convenience over `acquire_status`; migrate recovery-capable callers, then consider deprecation, never flatten the typed API. |
| render-host `browser_frame`, `recovery`, `readback`, `browser_surface_readback`, `surface_capture_layout` | Retain ownership leases, generations, retirement, cancellation, bounded buffer allocation, reservations across Core replacement, exactly-once collection, queue/scope attribution, and five-second deadlines. `browser_frame.rs:92-95` drops operations outside locks; `readback.rs:96-102,180-187` releases GPU resources outside locks. These are correctness constraints. Small stale comments and panic wrappers are cuts, not the machinery. CPU unit tests and old GPU receipts cannot qualify current browser hardware behavior. |
| render-host receipts / `DEVICE_RECOVERY.md` | Preserve immutable logs/JSON/source identities and explicit native/browser limits. Consolidate an index if needed; deleting negative controls or treating historical receipt HEAD as current is rejected. |
| `genet-winit-host` lib / native AccessKit bridge | Retain single-window SurfaceHost convenience and shared Core construction, logical wheel translation, native adapter lifecycle and queued action identity/data. It keeps OS-specific APIs out of target-neutral render-host. `a11y.rs:18-23` requires Windows install before showing the window; not interchangeable with deferred adapter creation. Platform cfg implementations are structural, not evidence of live accessibility qualification on all OSes. |
| `genet-compositor` facade/core/OS backends | Retain platform export routes, capture backend, host context and backend checks (`compositor.rs:284-291`), destination pools, CALayer/DXGI/Wayland backends, dmabuf modifier selection, slot-release tracking, bake transforms and interop. H1/H9/H10/H11 are narrower. Deprecated `StubCompositor` alias at `compositor.rs:74-82` has no source consumers found and is a low-risk public-name cut once downstream API compatibility is checked. Keep `WgpuMasterCaptureBackend`, which actually captures a texture. |
| `ortet` main/lib/build/args/shell | Retain thin document-session host and engine-feature separation, navigation/form driver, deadline and timing reporting. Windows stack reserve (`build.rs:8-17`) supports retained UI-thread engine execution and is not arbitrary decoration. H3-H5/H8 apply. Shared SessionDriver is a useful outcome seam, not another unnecessary framework. |
| Ortet fetch/web/webgl | Retain native-vs-browser transport adapters and browser streaming limits; consolidate policy and input effects rather than duplicate semantics. Keep same-device WebGL factory/registry and producer image retirement (`webgl.rs:53-76,111-149,242`); this deliberately creates from the render Core device. Native WebGL presentation receipts cover a different path from standalone WPT handlers. |
| Ortet a11y/mutate/timing/receipt | Retain generation/revision-bound action custody (`a11y.rs:7-14,34-38`), explicit mutation recipes, frame phase timings and pixel/semantic receipts. They provide actual scenario evidence. Consolidate receipt schemas/helpers, not assertions or failed artifacts. Examples, fixture generators and native/browser test pages remain fixtures; no cut justified by inventory alone. |
| `genet-wpt` main/args/manifest/expectations/conformance/tests | Retain runner CLI, test-kind/variant identity, strict expectation policy, provenance hashes, coverage counts and regression checks. Expected failures and missing results must remain distinct. `expectations.rs:555` rejects placeholder reasons; generated TODO text is not an accepted baseline. Fixture maps are historical policy inputs, not production implementations. |
| WPT harness/render/reftest/testharness/net/testdriver | Retain virtual/wall scheduling, isolated child timeouts, manifest/reference verification, renderer-specific resource adaptation, finite frame depth, readback, network completions and webdriver action interpretation. Share production semantics where H3/H4 show divergence; do not substitute production transport timing for deterministic harness clocks. Keep disk and server modes because URL/query/redirect semantics differ. Panic capture currently records errors, so removing all catch_unwind is not warranted. |
| WPT test262/test262_cmd | Retain metadata/harness loading, strict variants, isolated worker process and timeout. Replace the false-pass classification and combined flag routing in H6. Missing harness/invalid worker output presently has coarse skip behavior (`test262_cmd.rs:198,226-233`), so richer infrastructure outcomes should also stop hiding missing coverage. |
| WPT webgl_conformance | Retain as a conformance adapter, clearly separate from Ortet's same-device producer path. `webgl_conformance.rs` maintains per-context buffer/program/shader/texture state over shared test device. Do not infer native composition qualification from isolated WebGL calls. Reducing repeated GL dispatch mapping requires parity tests against both adapters. |
| `support/ci` dependency checkers + tests | Retain dependency-cone and Git-source audit tooling; different questions (resolved architecture vs declared/resolved/local/remote drift). `check_git_dependency_state.py:9-25` makes that multi-view purpose explicit. Do not merge them merely for script count. Shared parsing utilities may be extracted after independent tests, preserving no-auto-update behavior. |
| `support/ci` TLA scripts | Retain separate scheduler/postMessage model witnesses with generated actual trace inputs. These are behavioral contract tests, not duplicate Rust test launchers. Pin model-tool download provenance in CI if reproducibility is needed; no network/tool execution occurred here. |
| `support/ci` receipt runners/readback/servers | Retain distinct native/browser/HTTP/WebGL/compositing/arena/realm/accessibility assertions and negative controls. Consolidate repeated process/native-exit/source-identity functions behind one reusable harness. Keep O2 focus vs pointer diagnostic scope distinct: diagnostics alone are not an acceptance pass. Restore explicit source/receipt path bindings when refactoring. |
| `support/wpt` three baseline scripts | Retain local GPU reftest guard and server-fetch baseline guard separately from headless testharness CI. Replace duplicated build/runner-location helpers, keeping exact engine/renderer/subtest checks. GPU reftests intentionally excluded from default CI (`check-reftest-baselines.ps1:14-16`). |
| `support/profile-gates` | H7: retire obsolete entrance/docs, preserve strengthened cone witness. |
| `support/idl-interface-table` lib/parser/tag map/main | Retain deterministic vendored-input generator and explicitly reasoned overrides. `lib.rs:7-12,OUTPUT_PATH:23`, `main.rs:23-36` expose generate/check behavior; runtime dev dependency at `components/script-runtime-api/Cargo.toml:44` makes this test-owned support code, not a second DOM runtime. Handwritten partial IDL parser deliberately skips unsupported members; extending upstream input requires parser/schema tests, not blind table refresh. |
| `support/name-claims` all 11 packages (`cambium`, `frisket`, `genet`, `genet-livery`, `genet-taffy`, `layout-dom-api`, `livery`, `meristem`, `serval-scripted-dom`, `sprigging`, `xilem-serval`) | Retain as publishing/name reservations; manifests have no dependencies and source files explicitly contain no API. These are not dormant production crates. Registry deletion/publication was not checked or authorized. They can be excluded from architectural source counts without erasing their ownership purpose. |
| `support/build-scripted-workers.ps1` | Retain artifact producer with H8 repair; separately configured Nova wasm64 and Boa wasm32 outputs are not accidental duplicate binaries. |
| `scripts/remote-receipt.ps1` | Retain remote real-display attachment and receipt collection/provenance. Its optional ingest binary path keeps Mere out of Genet's dependency graph. Extract generic runner plumbing if reused, but preserve graphical-session checks, source dirty reporting, shell quoting and per-run outputs. Audit was read-only; no SSH/process launch. |
| `.github` / `.cargo` | Retain CI/testharness/MSRV policy workflows and explicit build-dir config. Current general CI is Ubuntu, WPT is Windows; neither establishes macOS/Wayland live qualification. MSRV manifest check is deliberately declarations-only. Do not promote it to compiler proof. |

#### External Forms machinery: preserve evidence, reduce future process duplication

At inspection `/home/markik/Code/testing/genet/forms/phase-c-windows-20261016` held 2,409 files, 255,556,936 bytes, 234 top-level files, and 73 top-level Python scripts. These counts include corpora, immutable receipts, binary/provider artifacts and caches; they are not 2,409 active tools. Most content was inventory-only. Representative source was read without executing it.

- `windows_run.py:15-32,42-60,86-145` binds source/index/config and role-specific target, caps memory, owns a shared lock and verifies descendants/source stability. `borrowed-windows-guard.py:154-176` restricts termination to the exact owned job and verifies bounded membership/drain. Preserve these protections, including negative controls and actual artifact hashes.
- `windows_wpt_capture.py:13-32,42-65,101-105` binds binary, corpus/runtime inputs, tools and exact generated variant identities; compares stdout/map accounting; writes checkpoint incomplete explicitly. Keep this acceptance recipe separate from raw execution success. Source guards are supplied by the enclosing job by contract at line 64. Standalone execution is not equivalent.
- `prepare_control.py:5-10,20-46` reverses a named historical Forms projection while excluding later unrelated changes. This is intentional candidate/control construction, not a stale fork to merge into production. Preserve control snapshots and patch provenance.
- `current-wrapper-name.txt:1` and `next-wrapper-name.txt:1` name different immutable wrapper versions; the ledger at `continuation-ledger.json:2-4` references another candidate head than the historical covering-plan `covering-gate-plan.json:2`. This demonstrates multiple epochs, not proof that current work is wrong. Prefer one authoritative manifest linking phase, role, wrapper hash, source, build, corpus, process and assessment receipts; archive earlier epochs intact. Do not overwrite old wrappers or receipt targets to make filenames simpler.
- Future consolidation: reusable OwnedJob/runner/parser library plus versioned recipe data; retain content-addressed wrapper snapshots for historical verification. Remove `__pycache__` only as optional cache cleanup after checking references, never delete provider binaries, projected source, raw failures or input corpora by age/size.
- No broad transcript re-audit and no claim that Checkpoint C is complete, published, rerun, or current at audit HEAD.

#### Verification limits and next cut order

Start with H1, stale proposal comments, obsolete profile gate replacement, and native-command exit checks. They have small surfaces and explicit done conditions. Then correct test262 result typing before trusting its pass counts. Production scheduling/input/fetch and checked browser frames require cross-host regressions and downstream migration, not mechanical deletion. H9 requires platform synchronization review. No runtime behavior, historical receipt, GPU acceptance or remote state was refreshed in this audit.

Memory context used only to preserve Forms candidate/control and receipt boundaries: `MEMORY.md:246-263`. All code findings above were checked against current source; historical memory results were not promoted to current qualification.

## Independent shader-link and remaining-domain review


HEAD: `8fa73497952ecda37cf234c4a845354dab1af599`. Read-only source/history review; no build, runtime test, network download, or repository edit. The scenarios below are deductions from inspected source, not executed GPU or Fetch probes. Root's existing `origin-helper-probe.txt` supplies the separate isolated origin-helper evidence. Coverage is per-file in `remaining-coverage.json`; inventory-only files are explicitly not semantic review claims.

#### Highest priority: WebGL link needs one shared interface/layout plan

**Retain and strengthen.** `components/webgl-wgpu/shader/lowering.rs:27-35` compiles both stages independently; `:97-121` checks varying names and kinds but does not reconcile their numeric locations. `components/webgl-essl/lower.rs:682-709` assigns vertex output locations in declaration order, while `:1798-1838` independently assigns fragment input locations. Same-type varyings declared `a,b` in the vertex stage and `b,a` in the fragment stage therefore pass the name/kind check with swapped numeric wiring. Different types/order can instead surface as a pipeline validation problem. This is a source-derived counterexample, not an observed draw result.

The uniform problem is stronger than naming cleanup: `components/webgl-essl/lower.rs:1704-1741` starts offset zero for every separately compiled stage and emits binding zero at `:1770-1773`. `components/webgl-wgpu/shader/lowering.rs:168-205` then creates a fresh union, vertex uniforms before fragment uniforms, assigning new offsets. `components/webgl-wgpu/webgl/programs.rs:449-455` writes values at those union offsets, and `components/webgl-wgpu/webgl/pipeline.rs:256-257` binds one buffer. A vertex-only vec4 `u_position` and a fragment-only vec4 `u_color` each compile at byte zero; the merged CPU reflection puts `u_color` at byte sixteen. Both stages read the first member from the same buffer. Identical declarations in identical order hide the issue. Duplicate uniform names are skipped at lowering.rs:175-177 without cross-stage type checks; the comment claiming earlier cross-stage rejection is unsupported by the actual pair check.

**Boundary:** preserve the ESSL parser, diagnostics, shader stages, reflection vectors, and wgpu ownership. Introduce a pair-level interface/uniform layout before final lowering, or preserve stage-local layouts with distinct bindings and matching CPU writes. Merely comparing locations and rejecting otherwise valid programs would be an explicit narrowing, not a complete repair. Done when reordered same-type varyings, disjoint stage uniforms, reordered shared uniforms, and conflicting uniform types have deterministic link behavior and a real draw verifies the accepted layouts.

#### Other retained findings and cut boundaries

- **Repeated shader frontend work:** `webgl/programs.rs:47-60` fully compiles for compile status; `:175-183` invokes cache-key validation then cold translation. `shader/lowering.rs:43-85` makes both operations parse/check/validate/lower each stage, while `webgl-essl/lower.rs:86-92` repeats type checking. Even a warm program-cache hit compiles both source strings to obtain a source-string key. Retain compiled shader artifacts keyed by source revision and stage; link and cache from those artifacts. Preserve compile status, errors, pair checks, and context-loss behavior. This is visible duplicate execution, not a measured performance claim.
- **Obsolete narrow reflection:** `shader/lowering.rs:145-148` requires a first vertex attribute even though the widened reflection is authoritative. The five narrow convenience fields in `shader.rs:62-70` have assignments and test reads, with no production field readers found under webgl-wgpu. Remove those fields and the attribute-required rejection together, after adjusting compatibility tests. Preserve full attributes/uniforms/samplers; broader frontend restrictions still bound attribute-free shader support. Lower priority than the broken link layout.
- **Text processing outside the shader AST:** `webgl/programs.rs:33-39` uses exact substring replacement for `gl_FragData[0]`; equivalent spaced spellings do not take the same path. `shader/lowering.rs:295-317` scans unlexed source for precision and can consume comments. Preserve original source for API/debugging and move compatibility normalization and precision reflection into token/AST ownership. Do not remove supported draw-buffer-zero behavior without replacing it.
- **Netfetcher cache cap is advisory only:** `components/netfetcher/src/fetch/cache_phase.rs:85-88` checks declared length; `:96-106` carries no cap; `:120` appends all decoded bytes and `:131-141` commits them. `fetch/finalize.rs:114,142-149` decodes before the tee. Unknown-length bodies and compressed expansion bypass the configured memory bound. Pass the cap to the tee, abandon and release cache accumulation when decoded bytes exceed it, continue yielding the body, and never commit the abandoned entry. Preserve streaming, clean-EOF commit, partial-body/error exclusion, and the host-configurable cap. SRI's separately documented whole-body buffering is a different path and is not fixed by this change.
- **Browsing-context origin has a second URL authority:** `components/shared/browsing-context/lib.rs:72-116` hand-splits URL strings. `:103,108` converts invalid/out-of-range ports to `None`, then constructs a tuple origin. Percent-encoded hosts, special-scheme backslashes and URL normalization likewise disagree with the existing `url::Url` owner used by Netfetcher. Retain the graph, opaque-origin identity, sandbox flags, activation and navigation model; replace only URL-to-origin derivation with canonical parsed URL data. Preserve inherited `about:blank` and sandboxed opaque-origin policy in their owners. Root's helper probe verifies one collision; this review makes no claim of a demonstrated end-to-end security exploit.
- **Taproot duplicates live semantics/geometry:** `components/taproot/lib.rs:248-259` is a DOM matcher; `:263-284` recomputes Cambium styles with default interactions and runs layout to resolve clicks. Owner-projection matcher exists at `projection.rs:24-59` and documents missing/hidden nodes as authoritative. Live Turnstone callers remain, e.g. `src/workbench_pane.rs:414`, `src/transcript_pane.rs:320`, `src/trail_pane.rs:285`; projected adoption exists at `src/shell/contributed_automation.rs:65`. Route semantic selection through owner accessibility projection and geometry through owner retained layout. Preserve generic class/id/raw-DOM selectors where that is their declared contract, selector order/substring behavior, exact-name opt-in, and test-driver input dispatch. Do not delete Taproot wholesale or silently change all matching semantics.
- **Narrow orphan media cut:** `components/media/media-thread/media_thread.rs:17-23,44-46` retains constellation/GL-player/WeRender external-image plumbing. `media-thread/lib.rs:162` is its own starter; root dependency declaration exists at `Cargo.toml:352`, but supplied workspace metadata contains all fourteen other media packages and excludes this one. Source search found no external `GLPlayerThread` or package consumer in the reviewed family check. Candidate cut is only `components/media/media-thread` plus its root alias, after final consumer verification. Preserve archived history. Do not infer that the whole media family is dead: Turnstone Cargo.toml:526-531 explicitly depends on media traits/player/streams at a pinned Genet revision, and servo-media reexports audio/player/streams/traits/webrtc.

#### Families retained or rejection of overbroad cuts

**Netfetcher:** orchestration remains a useful transport-agnostic Fetch policy owner. Transport traits, WireTransport adapter, request/response streaming, redirect/preflight/finalization phases, cookie/cache/HSTS/Alt-Svc stores, referrer/CORS/SRI/data URL/decode modules and native WebSocket lane are distinct concerns. Surface/symbol scans covered all those modules; targeted body reads concentrated on orchestration, cache, transport and WebSocket policy. No basis to collapse Fetch into a host HTTP client. Built-in empty cache and default transport are interchangeable implementations, not evidence of duplication. Full conformance was not assessed.

**webgl-essl/webgl-wgpu:** keep the standalone frontend and backend separation. Lex/token/span/AST/parse/check/registry/typing/validation/reflection/lowering form a legitimate compiler pipeline; wgpu object/state/program/draw/pipeline/readback split is legitimate. Inventory includes their test families; tests were not executed or all semantically reread. Findings target redundant work and a missing shared link plan, not removal of the frontend or backend state machine.

**Fleece:** retain render-free extraction, canonical text/anchors, metadata, tables, annotations, text fragments, structured data and wire records. `structured.rs:15-22` deliberately uses raw number strings and ordered member vectors; `:26-49` preserves exact DOM text plus invalid JSON status; `:112-164` emits both retained blocks and syntax projections. A generic JSON-value replacement can lose duplicate keys, order, number spelling, or failure evidence. Reject a handwritten-parser removal based only on implementation style. The existing parser/profile has a preservation contract. Helpers with similar names and repeated DOM traversals suggest possible internal consolidation, but no additional concrete defect or measurable cost was established here.

**Media:** reviewed the fourteen package-manifest dependency surfaces and entry seams for servo-media/backend auto selection, player GL context, GStreamer render adapter, audio graph, stream registry and WebRTC traits. Audio processing nodes and individual GStreamer backend implementations were inventoried, not fully reread. Retain audio graph, playback/capture/stream and WebRTC interfaces and dummy fallback. GL-capable rendering adapters and raw-frame fallback are not automatically removable simply because the main renderer uses wgpu. Qualification requires checking which application backend actually supplies frames, not deleting that interchange contract.

**genet-text:** `src/lib.rs:1-109` centralizes unchanged UTF-8 text, explicit byte/scalar/UTF16 coordinates and UAX29 mechanics over unicode-segmentation. Keep this single segmentation owner and the pinned official conformance corpus. Search-specific normalization and DOM range policy are deliberately outside its responsibility; do not fold them into this crate.

**genet-clipboard:** retain service trait, memory backend and long-lived arboard system handle. `system.rs:9-20` introductory prose is stale about text/image/custom coexistence; implementation `:90-114` already uses the fork's `set_data`. A file-list-bearing item intentionally takes the separate path at `:81-84`. Correcting introductory prose is cleanup, not a reason to cut the service. Real OS tests mutate shared state and were not run.

#### Maintained vendor deltas only

Compared every present file also available in the local cached registry copies of ipc-channel 0.22.0, gpu-allocator 0.28.0 and regress 0.11.1. Source matches except regress's stated two-line patch; manifest differences below. This was local comparison, not an upstream release-status check. Upstream code bodies are inventoried rather than claimed as semantic audit coverage.

- **ipc-channel:** `Cargo.toml:33-46,59-66,78-80` defaults to in-process and makes OS deps optional behind native-os-ipc. Registry-source comparison found only Cargo.toml differs among shared present files. Retain until callers migrate from its channel/serde API; do not turn the vestigial OS transport source inventory into a claim it runs. Optional removal of unused platform source is distribution cleanup only and would remove the advertised explicit OS opt-in.
- **gpu-allocator:** sole shared-file difference is `Cargo.toml:167,175`, narrowing windows version ranges to 0.62 for normal/dev dependencies. Retain compatibility pin until a qualified dependency graph eliminates skew. No allocator-algorithm fork found; reject auditing upstream allocator internals as a Genet-maintained delta.
- **sonic-rs 0.5.8:** `Cargo.toml:97-112` documents and implements ahash baseline std/compile-time-rng with default features off, while non-wasm restores default RNG. Root `Cargo.toml:456-460` repeats retirement condition. No cached sonic-rs 0.5.8 source baseline found; history identifies this as a manifest-targeting fork, but exact upstream source identity was not independently verified here. Keep until the Wasm feature graph is qualified without this patch. Do not replace JSON uses merely to delete the vendor directory.
- **regress 0.11.1:** `GENET_PATCHES.md` states native-only UnicodeSets fix and retirement tests. Cached registry diff confirms only source change at `src/parse.rs:1175-1177`: assign the complement returned by `inverted()` for a nested negated set. Retain native-only dependency path and exact UnicodeSets/UTF16 cases until an accepted registry release passes them. No basis to impose this patch on Boa/Vano's separate regex dependencies.

Done conditions for implementation proposals belong to each owner and its consumers. This review does not convert source inspection into hardware/browser/runtime qualification, nor does family coverage mean every inventoried source file received a full semantic read.

## Audit artifact verification

The final assembly checked that every core inventory path has an explicit review depth, all 52 workspace packages appear in the package ledger, all 996 inventoried source hashes still match the audited snapshot, and the report's local Markdown file links resolve. These checks validate the audit artifact and provenance, not product behavior. Product code, tests, historical receipts and build artifacts were not edited.
