# Canvas colour and document scroll

The CSS canvas colour fills the viewport. Moving document content must not move
that fill out of the lower viewport. Livery previously translated its entire
cached paint list, including the viewport-sized canvas rectangle. Turnstone's
hardware Vulkan receiving exposed a transparent lower band after scrolling.

`LiveryPaintList::scrolled` now wraps document paint after the propagated canvas
colour command. Both cached and freshly repainted document frames use it, as
does the scripted Livery owner. Rootstock consumes the same operation. Ordinary
element backgrounds and retained child/text/host slots remain inside document
scroll. `translated` still places the complete surface, including its canvas,
and page zoom still scales the whole result.

Canvas-colour bookkeeping is producer-local and is not added to serialized
paint-list data. Apply document scroll to the canonical freshly emitted list,
before placement or page scale. Background-image emission is unchanged by this
repair; no opaque colour is invented for a transparent canvas.

The [provider receipt](receipts/canvas_scroll_20261023/README.md) preserves the
red pixel checks, initial compiler refusals, final source hashes and validation.
The first reproducer returned transparent lower-viewport pixels where opaque
beige was expected. Final pixel tests cover propagated root/body colours,
two-axis cached scrolling, retained repaint, fractional page scale, transparent
canvases, placement and a child spliced under the scroll transform. The full
Livery suite passes with its explicit ignores, and the scripted lane compiles.

The full run also required two existing test/example compatibility repairs:
an explicit empty byte-map type at a trait-object image-source argument, and
conversion of a corpus placeholder image to its current shared pixel storage.
Neither changes production paint or resource ownership.
