# Shared render-device recovery

`RenderCore::shared_recovery` reuses one ownership-group recovery owner and the
initial native boot recipe. Each core has an immutable generation and installs
one actual device-loss callback. Hosts subscribe with wake callbacks, release
their surfaces/GPU bindings and acknowledge the lost generation. A closed token
leaves the barrier. Replacement runs once outside locks after every live host
and admitted/cancellable readback has detached. Failed replacement is terminal
for that transition; stale notices and foreign acknowledgements cannot boot
another core.

`PendingRgbaReadback` owns cancellable buffer/map state and no strong RenderCore.
Loss drops its GPU resources before host replacement, returning one typed loss
notice; later collection returns AlreadyCollected. Resource destruction and all
wake/cancellation callbacks happen outside their protecting locks. Readback
creation admitted before loss holds the barrier until its cancellation is
registered.

Native hosts use the retained synchronous boot recipe. On Wasm,
`RenderCore::shared_recovery` stores an asynchronous boot recipe. After the same
all-host detachment barrier, the browser adapter claims
`begin_async_replacement` once and polls its local, non-Send future outside host
borrows and owner locks. Ordinary synchronous `recover()` leaves that strategy
pending; it does not invoke a synchronous failure. The claim is branded by owner,
lost generation and attempt. Drop, expiry, stale or foreign completion cannot
install a core. A monotonic deadline is checked before factory polling and before
Ready installation, even if the browser timeout callback runs late. Failure is
terminal for that transition. Surface Lost requests recreation on the healthy
device and Outdated only reconfigures; neither replaces every device.

The production Cambium owner gate uses two actual hosts, installed callback
notification after Device::destroy, one replacement generation, byte-identical
registered reader images, exact-once capture/readback refusal and a later failed
factory. Its headless distinct-leaf-key scope and fresh native presentation are
recorded separately in Mere's recovery receipt. Controlled destruction does not
claim physical driver-reset recovery. Browser automatic replacement has the
separate controlled two-mount R6 qualification described below.

The checked raster API returns `RasterizeFailure`: an installed loss notice has
priority over the original provider error. Admission holds the replacement
barrier through target creation and render submission. It rechecks the monitor
immediately before rendering and after rendering; it does not make a physical
GPU immune to concurrent loss. Native checked raster operations also capture
wgpu validation, internal and out-of-memory errors through per-thread scopes,
including target/filter creation and buffer uploads, so an invalid resource
caused by concurrent destruction can return through the checked error path.
Every native scope result is observed before success. The original Vello scope
error is retained on a healthy device; loss routes to the shared owner instead.

These native raster scopes do not qualify every producer upload, composition,
presentation or physical driver-reset path. WebGPU error scopes resolve through
JavaScript promises and need a corresponding asynchronous checked host path;
the synchronous browser API is not qualified for loss inside submission.

Native checked rasterization now qualifies the current Classic frame before
returning its target. Submission alone is insufficient: shader curve-limit
flags arrive through asynchronous counter readback. The checked call observes
all current counter buffers until zero pending bytes/count remain, preserving
any actual provider quality error. Native queue completion and repeated
observation share a five-second deadline inside loss admission and API scopes.
A missing observer, timeout or completion error refuses the frame. This adds
native completion waiting; encode-only timings must not be described as the
complete checked-host call cost.

The actual 32x24 round-cap regression submits a finite width-40,000,000 stroke
that passes aggregate preflight but exceeds the shader's per-curve bound. It
requires CurveFlatteningLimit before a successful checked target could escape,
then zero pending counters and a valid render on the same healthy device.
Browser current-frame shader-counter qualification also requires an async host
seam and remains separate from the native completion guarantee.

## Browser controlled shared replacement R6

The accepted browser R6 consumer uses Genet
`5618326d287e7f7ef0c1cc3545587d64cedb22a7`, Mere
`4682ba99040c234c6410d73125e5dc9b1f91cc03`, Netrender `c74b7cb28090c28b1c6d80b08dcdea18aff48d3e`
and provider `2eae6243881d5b24c984d5ee35dde509ca5db782`. Two actual full Workshop
mounts use identical local preview keys with distinct physical images. Actual
owned `Device::destroy` emits generation 2's Destroyed notice; the production
async owner detaches both hosts and installs shared generation 3 on their same
canvases. One host is application-hidden within a visible browser document.
Runner/source identity, authored state, focus/selection/zoom and subsequent
logical input/resize survive. Typed captures/readback refuse once; both mount
lifetimes and exclusively owned devices complete their distinct cleanup.

Twelve complete root/Reader/Stylesheet image copies and six before/after pairs
pass the unchanged uniform RGB 3 / alpha 2 / mean RGBA 1 limits. Maximum observed
RGB difference is 1, alpha 0 and mean 1.9501730778606602e-5; equality is not
claimed. All twelve PNGs were reviewed. The fixture explicitly maps its checked-in
Roboto bytes to sans-serif, system-ui, serif and monospace in the outer Host and
both retained producer documents. This qualifies face delivery and retention,
not production serif, fixed-width or script coverage. Native shared R2 evidence
uses the earlier G09b/Mcc65 source and does not requalify these font changes.

The compact consumer record is Mere's
`design_docs/cambium_docs/testing/receipts/2026-10-12_flattening_and_device_recovery/browser-r6/`.
Its provenance retains the actual compiler cone/local override, source/package
and driver seals, external raw hashes, numeric/visual review and owned-process
closure. Later immutable-pin integration is a separate coherence check.
Current-frame asynchronous shader counters, GPU error scopes and final canvas
clear/blit remain unqualified: R6 records pending Some((3, 96)) and both current-frame
qualification flags false. Physical driver reset, OS assistive actions and
sustained performance remain outside this controlled replacement receipt.
