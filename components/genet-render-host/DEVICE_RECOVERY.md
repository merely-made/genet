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
