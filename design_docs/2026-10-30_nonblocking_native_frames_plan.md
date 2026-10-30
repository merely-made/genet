# Nonblocking native frame qualification

**Status (2026-10-30): in progress.** Native hosts need checked GPU frames
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
is created per frame, and no destructor waits for the GPU. A timeout or otherwise unprovable outcome
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
