# Observers and timing: High Resolution Time, Performance Timeline, the MutationObserver residue and IntersectionObserver

**Status:** in progress, 2026-10-20. Assessment and rulings done; phases H and T implemented and accounted; M attributed; G and I stopped at a fork (F4). Branch `observers-timing` at worktree `Code/worktrees/genet-observers-timing`, from main `16b971188f3`. Not merged.

Ledger rows 4, 5 and 6 of the [standards-to-features ledger](2026-09-07_standards_to_features_ledger.md): Intersection Observer unlocks dwell and attention receipts for the trail pane; High Resolution Time and the Performance Timeline unlock an honest `dwell_ms` and page-load receipts; Mutation Observer unlocks re-extraction on SPA navigation. Mark, 2026-10-20, asked whether the four are briefed as one small lane: "Yes, one bundled lane (Recommended)".

Evidence lives under `Code/testing/genet/observers-timing/`. Baseline counts are from the 2026-09-29 Vano census (`Code/testing/genet/wpt-ledger/2026-09-29_vano_platform_census/disk-fixed/`): disk mode, dev build, four workers. They are a starting picture, not this lane's before map. The before map is a frozen runner built from `16b971188f3` on this box.

## Rulings (Mark's words, unnumbered)

Round 1, 2026-10-20:

| Question | Answer |
|---|---|
| How far the lane goes for IntersectionObserver's geometry | "Build the seam here (Recommended)": one host geometry seam backed by Livery (`getBoundingClientRect`/`getClientRects`, `client*`, `scrollTop`/`scrollLeft` get and set, element `scrollTo`/`scrollBy`, `scrollingElement`). IO computes from the same seam. `css/cssom-view` is measured before and after. |
| Order | "Clock first, IO last (Recommended)": hr-time, then performance-timeline, then the MO residue, then IO. |
| Coarsening | "Spec values, host-configurable (Recommended)": 100 µs, or 5 µs when `crossOriginIsolated`, with no jitter; a host profile may set a coarser resolution. |
| IO timing | "Spec timing (Recommended)": compute after rAF and relayout in each rendering turn, deliver as a queued task, and skip hidden documents. |

Round 2, 2026-10-20:

| Question | Answer |
|---|---|
| Clock owner | "One host clock per process (Recommended)": one shared clock with one epoch anchor, held by the host and handed to every runtime, worker and frame. Each realm's `timeOrigin` is a coarsened reading at navigation start or worker start. |
| Harness `Date` | "Date follows the clock, both engines (Recommended)": the virtual clock is epoch-anchored and drives `Date`. Boa uses its `Clock`. Vano gets an additive `HostHooks` clock hook on its own branch, shown to Mark before anything is pinned. |
| Sleep | "Excludes sleep (Recommended)": the host clock pauses during system suspend on every platform. |

Round 3, 2026-10-20. This was raised because the virtual clock moves only when a timer fires, so a spin on `performance.now()` or `Date.now()` never ends:

| Question | Answer |
|---|---|
| The harness clock | "Real time plus skipped waits (Recommended)": harness time is the shared host clock plus the total of the waits it skipped. Timers still fire in delay order without sleeping, and each fire adds the skip needed to reach its due time. |
| Vano base for the `Date` hook | "Branch from Genet's pin fd9db00 (Recommended)": the repin moves Genet by exactly the hook commit; the branch reaches Vano main separately. |

## Assessment (2026-10-20)

### Where each piece's prerequisites live

- **Timers and the timeline.** `components/script-runtime-api/timing.rs` installs `performance` as a JS bootstrap over two natives: `__performanceClockMode` and `__performanceHostElapsed`. `ElapsedClock` is a per-`Runtime` `Instant`. `timeOrigin` is `Date.now()` sampled when the surface is installed. Nothing is coarsened. Observer delivery is a zero-delay `setTimeout`. The WPT harness selects `PerformanceClockProfile::Virtual` (`ports/genet-wpt/src/harness.rs`, three sites), so `performance.now()` follows the timer queue while `Date.now()` reads wall time.
- **`Date`.** Boa (bole `aef1134`) owns `Rc<dyn Clock>` on its `Context` (`core/engine/src/context/time.rs`), which a host can supply. Vano (`fd9db00`) calls `SystemTime::now()` in `builtins/date/data.rs`; its `HostHooks` trait (`execution/agent.rs`) has no clock.
- **The rendering turn.** Both scripted hosts already run rAF and then relayout in one turn. In the harness this is `RenderSession::turn` (`ports/genet-wpt/src/harness.rs`). In product it is `LiveryScriptedDocument::pump` and `ScriptedDocument::pump` (`components/genet-scripted/document.rs`), and hidden documents skip rAF there. IntersectionObserver's "run the update intersection observations steps" belongs right after the relayout, in both.
- **Geometry.** Livery has what IO needs: `LiveryLayout::fragment_rect` and `content_rect` (`genet-livery/src/document.rs`), `rect_of` / `fragments_for_node` (`layout/query.rs`), and per-element scroll offsets (`document/scrolling.rs`, `element_scroll`). `engine-observables-api`'s `BoxModel` names the border box as `getBoundingClientRect`'s. **Script reaches none of it.** No `Element.getBoundingClientRect`, `getClientRects`, `clientWidth`/`clientHeight`, `scrollTop`/`scrollLeft` or element `scrollTo` exists. Only `window.scrollX`/`scrollY` and the `Range` rects reach script, through `SelectionHandler`, which `genet-scripted/livery.rs` implements over `frame.fragments` minus document scroll. That seam is the model for the geometry seam. `css/cssom-view` is not in the 2026-09-29 census scope.
- **MutationObserver.** Landed 2026-09-07 ([plan](2026-09-07_mutation_observer_plan.md)). The global exists on both engines, so ledger row 6's "9 files fail on the missing global" is stale.

### Failing files by cause (census 2026-09-29, Vano)

**hr-time, 17 files, 13/26.**

| Cause | Files | In lane? |
|---|---|---|
| Virtual `performance` against wall `Date` | `basic.any` ×2 (one subtest each) | yes, H |
| No coarsening | `clamped-time-origin`, `timing-attack` | yes, H |
| Coarsening plus cross-origin isolation (https, headers) | `clamped-time-origin-isolated.https`, `cross-origin-isolated-timing-attack.https`, `raf-coarsened-time.https` (no results) | clock yes; needs server mode to measure |
| Shared clock between window and worker | `timeOrigin.html` (3), `window-worker-timeOrigin` (1) | clock yes; the tests also build workers from `URL.createObjectURL(new Blob(...))`, which does not exist (File API) |
| Child-frame and navigation origin | `test_cross_frame_start`, `navigation-start-post-before-unload` | partly; the origin must come from the navigating host |
| IDL shape | `idlharness.any` ×2 | yes, H/T |

**performance-timeline, 72 files, 70/115.**

| Cause | Files | In lane? |
|---|---|---|
| Resource Timing entries absent (another spec) | `case-sensitivity.any` ×2 (2 of 3 each), `droppedentriescount.any` ×2 (0/5 each), `po-resource`, probably `po-observe.html` | no; residual |
| Coarsening | `webtiming-resolution.any` ×2 (0/2 each) | yes, H |
| `supportedEntryTypes` is not `[SameObject]` | `supportedEntryTypes.any` ×2 (1/2 each) | yes, T |
| Entries are cloneable; they must throw `DataCloneError` | `not-clonable` | yes, T |
| IDL shape | `idlharness.any` ×2 | yes, T |
| `navigationId`, bfcache, not-restored reasons, element/long-task timing (tentative, other specs) | 21 files | no; residual |

**intersection-observer, 119 files, 0/118.** 104 files fail with no `IntersectionObserver` global. 14 report no results: 12 SVG targets, `zoom-scaled-target`, `grow-height-and-scrolled`, and `root-is-table-with-overflow-scroll`. One is skipped (a reftest). 19 files use iframes. Across the tests and their helper, `scrollTop` appears 134 times, `clientWidth`/`clientHeight` 86, `getBoundingClientRect` 23 and `scrollTo` 13.

**The MutationObserver residue.** All the core `MutationObserver-*` files pass. These still fail: `MutationObserver-document` 1/4 (parser-inserted records; residual 7 of the 2026-09-07 plan), `-cross-realm-callback-report-exception` 0/1, `-textContent` 3/4, `ParentNode-replaceChildren` 17/29, `Element-classlist` (errors), and `insertion-removing-steps-iframe` 1/4. Phase M attributes each from a verbose run before changing anything.

## Phases

### Phase H: one host clock (High Resolution Time)

Done when:
- one process-wide `SharedClock` (one epoch anchor taken at startup; a monotonic source that excludes suspend on Windows, Linux and macOS) is owned by the host and handed to every `Runtime`, worker and child realm;
- `timeOrigin` is the coarsened clock reading at navigation start (supplied by the navigating host) or at worker start, and `performance.now()` is the coarsened difference;
- coarsening defaults to 100 µs, or 5 µs when `crossOriginIsolated`, with no jitter, and a host profile can choose a coarser resolution;
- the harness's virtual clock is epoch-anchored, and `Date` reads the same clock on Boa (its `Clock`) and on Vano (a new additive `HostHooks` clock hook, on a Vano branch shown to Mark before any pin moves);
- regressions at the clock layer: monotonicity, coarsening at both resolutions, window and worker origins on one timeline, and `Date` agreeing with the virtual clock, on both engines;
- hr-time and performance-timeline are mapped before and after on both engines, with zero lost passes and every movement attributed.

### Phase T: the Performance Timeline residue

Done when `supportedEntryTypes` returns one frozen array per realm, entries refuse structured cloning with `DataCloneError`, `PerformanceEntry` and friends have IDL shape (`idlharness` gets past setup), and performance-timeline's map moves only by attributed gains.

### Phase M: the MutationObserver residue

Done when each of the six files is attributed from a verbose run on both engines, the causes that sit in the observer are fixed, the rest are named here with their owning spec, and `dom`'s map has zero lost passes.

### Phase G: the element geometry seam (CSSOM View subset)

Done when a `GeometryHandler` host seam (modelled on `SelectionHandler`) is installed by the harness `RenderSession` and by `LiveryScriptedDocument`, backed by Livery's fragment and scroll state. Through it, script gets `Element.getBoundingClientRect`/`getClientRects`, `clientTop`/`Left`/`Width`/`Height`, `HTMLElement.offsetParent`/`offsetTop`/`Left`/`Width`/`Height` (IO's test helper computes root bounds from them), `scrollTop`/`scrollLeft` (get and set), element `scrollTo`/`scrollBy`/`scrollWidth`/`scrollHeight`, `document.scrollingElement`, and `window.scrollTo`/`scrollBy` reaching Livery. Done also requires regressions at the seam on both engines, and `css/cssom-view` mapped before and after with zero lost passes.

### Phase I: IntersectionObserver

Done when `IntersectionObserver` and `IntersectionObserverEntry` exist per the IDL. The init must be validated: `rootMargin`/`scrollMargin` parsing and `threshold` range checks with their `TypeError`/`RangeError`. Intersections must be computed per spec over the containing-block chain with clips, the implicit root (the top-level viewport) and an explicit root, using thresholds, `isIntersecting` and the edge-inclusive rule. The update step must run after rAF and relayout in both rendering hosts and skip hidden documents, with delivery as a queued task and `entry.time` from the shared clock. Done also requires regressions at the computation layer (pure geometry, no script) and at the script layer on both engines, and intersection-observer mapped before and after with every movement attributed.

## Findings

- **F1 (2026-10-20).** Main's ignored `Cargo.lock`, copied into this worktree, passes `cargo metadata --locked --offline` at `16b971188f3`. The out-of-step state reported earlier on 2026-10-20 is no longer present.
- **F2 (2026-10-20).** The hr-time text, read at w3c.github.io/hr-time, says three things: the time origin of a Window is when navigation started, a worker's is when it runs, and time origins that can exchange messages share one monotonic clock. Coarsening is "100 microseconds, or a higher implementation-defined value" (5 µs when cross-origin isolated), so a coarser host profile conforms. The spec says nothing about system sleep. Only the WebIDL is vendored (`tests/wpt/tests/interfaces/{hr-time,performance-timeline,intersection-observer}.idl`).
- **F3 (2026-10-20).** The scripted tier has no element geometry API (see Assessment). This is the largest prerequisite for IntersectionObserver, and it is ruled into this lane as phase G.

## Progress

- **2026-10-20.** Worktree and branch created from main `16b971188f3`. Assessment done from the census maps, the WPT sources and the code. Two rounds of rulings recorded above. No code changed yet.
- **2026-10-20, phase H code.** `SharedClock` (`script-runtime-api/timing.rs`) replaces the per-runtime `Instant` and the `PerformanceClockProfile`: one process clock from `QueryUnbiasedInterruptTimePrecise` (Windows) or `CLOCK_MONOTONIC`/`CLOCK_UPTIME_RAW` (Unix), anchored once to the epoch; coarsening at 100 µs (5 µs isolated) with a host-raisable floor; thread workers take the page's clock in `WorkerBoot`, process workers rebuild it from an anchor in `Hello` (protocol version 2). `ScriptEngine::set_date_source` points `Date` at it: Boa through its `Clock`, Vano through a new `HostHooks::system_time_millis` (Vano branch `genet-date-clock`, commit `b1f3d25d`, from the pin `fd9db00`, not pushed or pinned; the lane builds against it with a local `--config patch`). The harness's two drive loops use `SharedClock::skipping()` instead of a private `skipped_ms`. `crossOriginIsolated` is defined as `false`. Two new dependency edges in `Cargo.lock` (`libc`, `windows-sys 0.61.2`), no version moves.
- **2026-10-20, phase H accounting** (frozen before-runner `16b971188f3`, candidate `runners/after-h.freeze.txt`, `--release` without netfetch, disk mode, `--jobs 1`): hr-time subtests 5/19 -> 10/19 and performance-timeline 35/70 -> 38/73 on **both** engines, with **0 lost** and every gain attributed (`Code/testing/genet/observers-timing/accounting-after-h.json`; accountant `account_observers_timing.py`). Gains: coarsening 2+3, now() positive 2, Date and performance on one clock 1, shared-clock time origin 1, real time in the harness clock 2, spins ending on the harness clock 2. Still failing in hr-time: worker origins need `Blob`/`URL.createObjectURL` workers, `crossOriginIsolated` tests need server-mode headers, frame and unload origins need the navigating host's start (those are navigation work), and the idlharness setup fetches WebIDL.
- Passed: 28 `cheap_globals` tests and 4 `SharedClock` tests on both engines under the local Vano patch.
- **2026-10-20, phase T code.** `PerformanceObserver.supportedEntryTypes` is one frozen array per realm; `PerformanceEntry` is registered with a new `__sc_registerUncloneable` hook in the structured-clone walker, so cloning an entry throws `DataCloneError`. Two both-engine regressions added (32 `cheap_globals` tests pass). `not-clonable.html` still needs a `navigation` entry (`getEntriesByType('navigation')[0]`), which is Navigation Timing and stays a residual; `idlharness` needs the harness to fetch WebIDL, which disk mode cannot do. Accounting for T waits on the shared after-map run with M.
- **2026-10-20, phase M findings** (verbose runs of the frozen before-runner, Boa). `MutationObserver-document` 1/4: a both-engine regression (`parser_insertions_reach_a_document_observer`, in `tests/mutation_observer.rs`) shows an observer registered by a script during parsing **does** receive the parser's later insertions at the runtime layer, so the WPT failure is not the observer; it sits in how the harness runs the test's parse and delivery, and stays a residual with that cause named. `MutationObserver-textContent` 3/4: a CDATASection's same-value `textContent` read gives `""` where the test expects `"foo"`: a DOM-node-model defect (CDATA data), not the observer. `-cross-realm-callback-report-exception` 0/1: needs the callback's own realm to report the exception into the frame (cross-realm reporting). `ParentNode-replaceChildren` times out under the harness (not an observer cause).
- **F4 (2026-10-20, found starting phase G).** The scripted tier has no element scrolling at all, not only no script API for it. `LiveryState` (`genet-scripted/livery.rs`) holds one document scroll offset; the per-element offsets (`element_scroll`, `ancestor_scroll`, `nested_scroll`) exist only on the static-document `LiveryDocument` (`genet-livery/src/document/scrolling.rs`), and the retained scripted frame places fragments with no element scroll applied. So `scrollTop`, `scrollTo` and a root that scrolls cannot be served by a query seam alone. Of IntersectionObserver's 99 test files, 64 use no scrolling and 48 use neither scrolling, iframes nor SVG; 26 use `scrollTop` and 34 use some scroll call. This is a design fork, brought to Mark rather than chosen here (see the report).
- **2026-10-20, stopping point.** Phases H and T are implemented and committed; M is attributed (no observer defect found, three causes named); G and I are not started in code. Phase G's ruling ("Build the seam here") assumed a query seam; F4 says the seam needs element scroll state in the scripted layout first.
- **2026-10-20, dom accounting for phase H** (`dom`, 698 files, disk mode, `--jobs 1`, frozen before-runner against `runners/after-h`): **Boa** 46,991 -> 46,992 subtest passes, 0 lost, one gain (`abort-signal-any`, an `AbortSignal.timeout` case, on the real-time clock). **Vano** 45,826 -> 45,280 with 565 lost, **all in one file**, `dom/ranges/Range-mutations-deleteData.html` (564 subtests plus the file). That file takes about 87 s alone against the runner's 120 s file timeout; the after-run shared the machine with the crate gates and was killed. Run alone, **both** runners pass it 564/564 on Vano (86 s after, 89 s before). So the 565 are a load timeout of the external per-file limit, not a clock regression; the isolated re-measurement is the accounting of record, and the loss is not attributed to H. The `dom` after-map is from `after-h`, built before phase T (T touches only `performance` entries and the clone walker).
- **2026-10-20, crate gates** (`--offline`, local Vano patch, debug): script-runtime-api, script-engine-boa, script-engine-api, genet-scripted and genet-wpt, 1,393 passed and 12 failed. Ten are the failures that already fail on main's own sources (`atomic_refusal` ×2, `associated_subtree_boundary` ×2, forms implicit submission ×6, as the WebCrypto lane recorded). **Two are this lane's:** `test_driver_action_sequence_synthesizes_a_click` and `test_driver_get_computed_label_reads_the_projection` time out inside the full parallel run and pass alone (8.4 s and 8.9 s). Cause: Mark's ruling that harness time is real time plus skipped waits makes testharness's own timeouts depend on machine load, where the old virtual clock was immune. This is a fork for Mark (see the report); nothing is changed meanwhile.
