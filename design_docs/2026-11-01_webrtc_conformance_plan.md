# WebRTC conformance and shared transport evaluation

**Status (2026-11-01): E0/E1a qualified; native Genet data-channel intake
qualified; application-carrier cutover, broader E2 acceptance and E3 remain open.**
The native receiving path is checked on both engines against selected unmodified
WPT and direct encrypted data transfer. Ortet's scripted host compiles with the
capability. Headed Chrome/Firefox, browser-guest, discovery/relay and media
acceptance remain separate gates.

**Brief:** Mark asked to evaluate webrtc-rs against the existing stack, inspect
Murm for duplication and dependency direction, and place the implementation
where Genet needs it for WPT conformance. Browser support is an engine feature;
it does not wait for Commons calls or public Graphshell rendezvous.

**Homes:** this plan owns Genet's conformance requirements and the evaluation.
`murm/design_docs/2026-10-13_murm_family_move_plan.md` owns any shared supplier
extraction and consumer cutover. Existing application-carrier qualification is
in `mere/design_docs/mere_docs/implementation_strategy/2026-10-28_browser_carrier_plan.md`;
call/media products remain in its sibling `2026-10-28_calls_and_media_plan.md`.
The existing [Netfetcher wire adoption](2026-10-14_netfetcher_wire_adoption_plan.md)
provides the dependency-direction precedent.

## Evaluation and ownership

Distinguish three things: a WebRTC protocol implementation, Genet's web-visible
peer connection and media APIs, and a Merely application session over a data
channel. Sharing the first does not make the last two one API.

Recommended direction, pending the native and browser qualification below:

```text
Genet runtime WebRTC bindings -> host capability -> independent RTC provider
Murm application carrier ----------------------> independent RTC provider
Mere/Turnstone -> Murm application carrier -> admitted application session
```

Arrows denote dependencies or a supplied capability, not network direction.
The reusable provider lives as a separate leaf in Murm's independent `wire/`
workspace. It must not depend on Genet, `murm`, `mere-transport`, Personae,
Notochord, Luggage, storage, the radio root, or Graphshell. Separate package
selection must keep HTTP/QUIC and RTC dependencies independent. A crate name
and a universal transport trait are not prerequisites for this evaluation.

The provider implementation lives in `murm/wire/crates/rtc` (`murm-rtc`): a
shared protocol API plus an optional caller-owned UDP adapter. Genet's host
binding now consumes this exact Git supplier through `components/genet-webrtc`;
the application carrier has not changed. Host-only candidate
gathering is implemented; broader discovery and relaying remain explicit work.

Genet owns WebIDL conversion, exceptions, promises and the operations chain,
stable JS wrapper identities, task/event ordering, per-realm connection
lifetime, navigation teardown, policy and permission checks, transport
observability, and WPT. Use a host capability beside the existing WebSocket
handler rather than putting sockets or an async runtime in `genet-runtime`.
The native Genet/WPT host must provide a real default implementation; injection
is not a reason to leave ordinary browser support unconfigured. The VM crates
remain independent of browser objects.

Murm's generic provider owns peer negotiation and packet processing plus its
explicitly configured I/O adapter, candidate discovery and relay lifecycle.
Genet supplies web policy and the host supplies scheduling, network settings,
resource limits and lifetime. Candidate gathering, STUN/TURN allocation, mDNS,
TCP/TLS fallback and route isolation must have named owners; a protocol core
alone does not perform those network operations.

The existing `murm-webrtc` package retains application framing, invitations,
release references and fingerprint-bound admission transcripts. Its current
single reliable ordered channel is an application profile, not the constraint
of `RTCDataChannel`. Notochord admission belongs above that profile. Ordinary
web peers do not acquire a Personae identity, redeem a Graphshell invitation or
run Notochord as a condition of WebRTC interoperability.

| Requirement | str0m 0.23.1 | rtc 0.21.1 | async webrtc 0.21.1 |
|---|---|---|---|
| Caller-driven protocol and time | Yes | Yes | Core is caller-driven; wrapper owns its driver |
| Browser-shaped pending/current SDP and rollback | Additional browser state adapter required | Public peer-connection operations/state | Async facade over rtc |
| Real Genet JS/WPT implementation | Absent | Absent | Absent |
| Candidate/relay network I/O | Caller supplies discovery and relay | Caller supplies discovery and relay | Built-in driver; TURN/TCP and secure TURN still skipped |
| Existing Merely carrier acceptance | str0m profile has direct/relay/reconnect receipts | No replacement receipt | No replacement receipt |
| Crypto in the measured independent graph | RustCrypto selection still brings AWS-LC certificate generation | ring selected; AWS-LC/custom provider are alternatives | ring selected plus Tokio |

**Recommendation:** use rtc as the first candidate for the shared provider and
Genet conformance work; retain str0m's existing carrier until a replacement has
equal evidence. Avoid constructing a complete browser/JSEP implementation
around the current one-channel answerer merely to preserve its engine choice.
Resolve observed upstream API gaps through adapters or supplier fixes before
calling this a final engine selection. No extraction is implemented by E0.

## Phases and done conditions

### E0. Current-source and dependency evaluation

Compare published `rtc`/`webrtc` 0.21.1 with the installed str0m 0.23.1,
using the features actually needed by each. Record protocol/API differences,
crypto/build inputs, current dependency graphs, and the exact scope of tests.
Keep the old August size comparison historical; do not use it as a current
measurement of the new RTC core. Receipts are compact text/JSON under
`murm/testing/webrtc-evaluation/receipts/`, **1 MB total retained cap**.
Source and lockfile live with that probe; builds reuse the existing approved
`C:\t\cargo-targets\retinue` (Murm's pre-rename target), with one compiler job.

Done when the recommendation identifies the shared mechanisms, application-only
parts, receiving seams and remaining qualification gates. Native API tests are
not WPT, browser interoperability, TURN, media or a shipping provider receipt.

### E1. Independent provider and existing-carrier reuse

Implementation order: E1a establishes `murm-rtc` in Murm's independent `wire/`
workspace and qualifies real data-channel packet exchange with explicit host
lifetime and budgets. E1b wires Genet's host capability and the existing
application carrier through that supplier, retaining their separate semantics.
E1a alone does not close E1 or claim a shipping carrier replacement.

Only after the comparison supports an engine choice, introduce the provider
below both consumers, with explicit commands/events and a caller-owned lifetime.
Preserve all existing invitation and verified-DTLS-fingerprint binding checks;
do not substitute a fingerprint merely read from untrusted SDP. Share the
actual protocol engine and I/O machinery rather than copying the restricted
Graphshell driver into Genet. Keep existing carrier receipts valid at their
original revisions until the replacement is requalified.

Done when the generic crate resolves without application/Genet back-edges,
the core can be driven without sockets, the native adapter owns cancellation
and socket/relay cleanup, and Murm's real carrier passes its composition,
backpressure, refusal and reconnect gates through that provider. Measure the
required crypto configuration on working Rust and the shared MSRV; a custom
crypto provider is implementation work, not a checked feature flag.

### E2. Genet data-channel web surface

This implementation lane retains at most **1 MB** in the single appended
[receipt file](archive_docs/receipts/2026-11-01_webrtc_conformance_plan_receipts.md).
It reuses `C:\t\cargo-targets\genet` and its `build` directory, without an
isolated target/home/worktree. The frozen diagnostic selection is three
unmodified source files at their unvaried URLs: `RTCPeerConnection-constructor`,
`RTCIceCandidate-constructor`, and `RTCPeerConnection-createDataChannel`.
These are matched named assertions, not a manifest-wide coverage denominator.
Native integration additionally requires encrypted JS-to-JS data transfer,
per-realm teardown and operation/event ordering on both engines.


Install `RTCPeerConnection`, session descriptions, candidates, transport objects
and data channels in `genet-runtime`, through engine-neutral host callbacks.
Implement configuration and option validation, offer/answer and rollback,
current/pending descriptions, negotiationneeded, ICE restart and state changes,
multiple ordered/unordered and partially reliable channels, binary/text/Blob
send behavior, bufferedAmount/low events, stats, close and object lifetime.

Done when the selected unmodified vendored `webrtc/` WPT files run through the
real Genet host on both Boa and Vano, with frozen case identity, matched before
and after maps and explicit exclusions. Separately prove Genet-to-Chrome and
Genet-to-Firefox data transfer, direct and forced TURN, ICE restart, close and
navigation cleanup. Use configured servers and bounded deadlines. Headed
interop does not replace WPT accounting, or vice versa.

### E3. Media and remaining WebRTC conformance

Bind RTP sender/receiver/transceiver behavior and MediaStream/MediaStreamTrack;
connect real capture, encoding, decoding and playback through the existing
media/player contracts. Genet owns browser-visible behavior and permission
lifetime; application call policy remains outside the engine. Share codecs and
device backends when qualified rather than building another media organ.
Render decoded video through the host's existing wgpu device.

Done when the media WPT groups and real audio/video interop have their own
receipts, including device loss and capture cancellation. Candidate-address
privacy, DTLS/SCTP transport events, renegotiation and network fallback remain
explicit gaps until exercised. Expand beyond `webrtc/` only against named
interfaces/tests such as `webrtc-stats`, encoded transforms and media capture.

## Findings (2026-11-01)

- `components/script-runtime-api/webrtc.rs` and its bootstrap now install
  Window RTC objects over a per-realm host factory. The runtime contract carries
  no socket or protocol implementation; `components/genet-webrtc` depends on
  that contract and the immutable independent `murm-rtc` Git supplier.
  `genet-scripted` takes the capability before parser scripts and pumps it on
  host turns. WPT's default `webrtc` feature and Ortet's scripted native route
  install the real adapter. Ortet's `--rtc-bind` selects a concrete host address;
  the initial address is loopback. A generic runtime without a host rejects
  peer construction with `NotSupportedError`.
- The selected constructor/candidate/channel files now complete on both engines
  with unchanged assertion identities. Remaining failures in that selection are
  the audio/video cases, which require E3's capture and RTP surface. Exact case
  maps, compiler/lock/binary/source identities, failed attempts and gate commands
  are kept in the [receipt file](archive_docs/receipts/2026-11-01_webrtc_conformance_plan_receipts.md).
  This diagnostic source selection does not represent all WebRTC WPT.
- Genet enforces stream-ID uniqueness and the ignored/non-negotiated ID rule.
  Its web-visible SCTP Connected procedure waits for initial stream-ID
  assignment. UTF-8 string limits are checked in the native host, avoiding
  temporary JS byte arrays for large labels/protocols; USV conversion preserves
  valid pairs and repairs lone surrogates. Local candidate metadata is retained
  when cloned and omitted from the signaled candidate dictionary.
- Native ownership is independent of VM collection: discarded realms revoke
  factories and peer handles, stale callbacks check active document ownership,
  retained wrappers close before an outgoing host cell is reused, and snapshots
  with active peers refuse. Resource ceilings and the polling
  interval are caller settings. JS event delivery uses retained per-realm hooks
  and checkpoints microtasks after each observation. SDP operations run as tasks,
  with signaling events before promise completion. Revisioned observations cannot
  overwrite a newer command result; description getters preserve slot aliases.
- The first native route gathers host candidates. Relay-only construction owns
  no socket and registers no direct candidate; changing route policy after
  construction refuses with `NotSupportedError`. ICE server configuration is
  retained, but discovery/allocation is not yet driven. Custom certificates,
  complete media/IDL, and full stats and transport event qualification remain
  open. `bufferedAmount` currently reflects the core's outstanding send bytes;
  exact web-visible timing and send-buffer saturation need their own coverage.
- `components/media/webrtc/{lib.rs,thread.rs,datachannel.rs}` is the inherited
  controller/backend contract. It spawns its own thread, configures one STUN
  server and a bundle policy, and lacks the full current peer-connection
  observability/configuration contract. GStreamer and dummy backends implement
  it; it is not a current WPT implementation. Preserve real consumers while
  qualifying any replacement or adapter.
- Murm's `native/crates/webrtc-carrier` has no dependency on its sibling
  `murm` or `mere-transport`, but does depend on Luggage through `InviteV1`.
  The native driver accepts one configured reliable ordered channel and
  rejects others. Reusing that entire package as the browser backend would
  import application policy and require removing those profile restrictions.
- `mere-transport` unconditionally combines the peer-stream seam with Iroh,
  blobs, p2panda, Stickleback and Muniment. Genet's RTC provider does not need
  that dependency cone. Do not force a web peer connection through its known-
  peer `Transport` identity/ALPN API or introduce a split merely for naming.
- The vendored `tests/wpt/tests/webrtc/` contains 239 HTML/JavaScript source
  files, including helpers; this is a source inventory, not runnable test or
  subtest coverage. Relevant cases exercise validation, current/pending SDP,
  explicit rollback, stats, ICE restart, multiple channel modes, buffering and
  GC. `interfaces/webrtc.idl` is the local binding reference.
- Upstream comparison is pinned to the published 0.21.1 sources:
  [rtc manifest](https://github.com/webrtc-rs/rtc/blob/v0.21.1/Cargo.toml),
  [crypto features](https://github.com/webrtc-rs/rtc/blob/v0.21.1/rtc-crypto/Cargo.toml),
  [async transport deviations](https://github.com/webrtc-rs/webrtc/blob/v0.21.1/docs/transport-objects.md),
  and [TURN adapter](https://github.com/webrtc-rs/webrtc/blob/v0.21.1/src/peer_connection/transport/turn_relayer.rs).
  [str0m 0.23.1](https://docs.rs/str0m/0.23.1/str0m/) deliberately exposes a
  different Rust API from the standard browser peer connection. Both cores
  separate protocol logic from I/O; that is no longer a unique str0m advantage.
- `rtc` exposes create/set descriptions with current/pending state, rollback,
  restart, channel options, stats and transport handles closer to the WPT
  vocabulary. This is a lower adapter-cost hypothesis, not a conformance score.
  Its bundled crypto choices are ring and AWS-LC, with a custom provider trait.
  Stable JS object identity and transport event delivery still need Genet work.
- `rtc`'s core and async `webrtc` have different relay responsibilities:
  rtc 0.21.1 contains TURN/TCP client support, but the async 0.21.1 relayer
  still skips non-UDP and secure TURN URLs. The core's TCP capability is not a
  working network fallback receipt. Candidate gathering and I/O remain host
  integration work when selecting the core directly.
- The isolated, locked Windows graph selects 133 transitive package versions
  for str0m/RustCrypto, 165 for rtc/ring, and 189 for webrtc/ring/Tokio. Normal
  and build dependencies are included; the probe itself is excluded. The
  separate graphs contain no upper-layer family packages. These are newly
  resolved graphs frozen by the probe lock, not the carrier's shipping lock or
  comparable compiled-byte measurements. Highest declared Rust floors are
  1.88/1.89/1.89 respectively; undeclared requirements and actual MSRV execution
  are not qualified by metadata. Full compact package lists and toolchain
  identity: `murm/testing/webrtc-evaluation/receipts/dependency-report.json`.
- The existing carrier's RustCrypto feature does not imply a C/assembly-free
  build. A locked offline reverse tree for `murm-webrtc --features native`
  reaches AWS-LC via `str0m-rust-crypto -> dimpl 0.7.4`'s `rcgen` feature;
  that feature explicitly enables AWS-LC. The isolated graph repeats this at
  dimpl 0.7.6. The old manifest comment claiming no build tooling was corrected;
  provider features and wire/API suitability remain separate decisions. No
  crypto implementation or shipping dependency was changed.

## Progress

- 2026-11-01: E0 selected rtc and established the independent provider boundary.
- 2026-11-01: E1a qualified `murm-rtc`; application-carrier cutover remains open.
- 2026-11-01: E1b/E2 added Genet's per-document capability, native adapter,
  Window objects, transport observations, document/WPT scheduling and Ortet
  intake. Qualification is in the appended [receipt file](archive_docs/receipts/2026-11-01_webrtc_conformance_plan_receipts.md).
  Both engines pass the native integration gate, including encrypted text and
  binary transfer, operation/event ordering and navigation lifetime. Selected
  WPT is accounted against frozen case identities with its media failures
  retained. Strict native-adapter Clippy and Ortet all-target receiving checks
  pass. Concurrent upstream runtime/runner work was merged before final
  qualification.

## Decisions

- **webrtc_conformance D1 (2026-11-01, Mark):** "Okay. So, shall we continue
  with RTC?" Continue with rtc as the selected implementation direction after
  the E0 comparison. This authorizes the provider/Genet work; existing carrier
  and WPT claims still require their named receipts.
  Links: area web-platform · plans E1 E2 E3 · summary Continue with RTC
