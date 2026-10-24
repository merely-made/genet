# Document session API

Renderer-free contracts implemented by retained Genet document engines and
consumed by Inker/Pelt and Webcue. These types do not own windows, GPU devices,
layout, resource loading or host placement.

`DocumentSession::text_input_state()` observes a focused writable native text
control against current completed layout. `None` revokes the observation when
there is no eligible editor or its layout is unavailable. Observing must not
perform layout. Producers refresh after frame preparation and clear on edits,
hide, viewport changes, blur, replacement and retirement as applicable.

`SessionTextInputState` carries input hints, password/multiline flags, UTF-16
selection offsets and a `SessionTextInputRect`. Coordinates are in the session's
logical viewport after page zoom and scroll. The host adds window placement and
applies device scale once. Field values, password values and document identities
are not part of this observation. Preedit selection uses the independent
UTF-8 byte offsets specified by `SessionIme`.

Webcue reexports these types as `TextInputState` and `LogicalRect`, and publishes
changed observations as ordered, scope-bound `Event::TextInput` messages. A
session that does not support this observation retains the default `None`.
