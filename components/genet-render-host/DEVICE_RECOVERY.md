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

Native hosts use the retained recipe. Browser device acquisition needs an
asynchronous host factory; the default synchronous factory reports a typed
failure. Surface Lost requests recreation on the healthy device and Outdated
only reconfigures. Neither is itself a request to replace every device.

The production Cambium owner gate uses two actual hosts, installed callback
notification after Device::destroy, one replacement generation, byte-identical
registered reader images, exact-once capture/readback refusal and a later failed
factory. Its headless distinct-leaf-key scope and fresh native presentation are
recorded separately in Mere's recovery receipt. Controlled destruction does not
claim physical driver-reset recovery or browser automatic replacement.

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
