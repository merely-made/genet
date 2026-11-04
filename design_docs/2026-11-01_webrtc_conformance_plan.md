# WebRTC conformance and shared transport evaluation

**Status (2026-11-03): E0/E1a and E2h qualified; E2r ICE restart receiving
qualification is in progress; broader E2, carrier cutover and E3 remain open.**
The native receiving path is checked on both engines against selected unmodified
WPT and encrypted direct/UDP-relay transfer. Ortet's scripted host compiles with
the capability. IP-literal UDP STUN/TURN and fixed-server ICE restart are
implemented; DNS, TCP/TLS, browser-guest and media acceptance remain separate gates. Installed headed
Chrome and Firefox qualify the bounded direct/forced-UDP-TURN data-channel
matrix; this is distinct from Genet running inside a browser guest.

**Brief:** Mark asked to evaluate webrtc-rs against the existing stack, inspect
Murm for duplication and dependency direction, and place the implementation
where Genet needs it for WPT conformance. Browser support is an engine feature;
it does not wait for Commons calls or public Graphshell rendezvous.

**Homes:** this plan owns Genet's conformance requirements and the evaluation.
`murm/design_docs/2026-11-02_murm_family_move_plan.md` owns any shared supplier
extraction and consumer cutover. Existing application-carrier qualification is
in `mere/design_docs/mere_docs/implementation_strategy/2026-10-28_browser_carrier_plan.md`;
call/media products remain in its sibling `2026-10-28_calls_and_media_plan.md`.
The existing [Netfetcher wire adoption](archive_docs/2026-11-03/2026-10-14_netfetcher_wire_adoption_plan.md)
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
the application carrier has not changed. Host candidates and IP-literal UDP
STUN/TURN use one caller-driven socket driver. Broader discovery and network
fallbacks remain explicit work.

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
`Code/testing/murm/testing/webrtc-evaluation/receipts/`, **1 MB total retained cap**.
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

### E2h. Automated headed browser interoperability

Use the installed Chrome and Firefox with their own dedicated, marker-owned
profiles and built-in CDP/WebDriver BiDi endpoints, and a native Genet runtime
fixture using the real factory. No microphone/camera permission is needed for
data channels. Reuse the controlled IP-literal UDP coturn fixture. Keep the
application carrier and browser-guest qualification separate.

For a repeat run, start `murm/wire/crates/rtc/tests/coturn.sh` in WSL and use
its printed numeric endpoint below. The runner shuts down its own peers and
browsers; stop the foreground coturn fixture after the run.

```powershell
$env:CARGO_TARGET_DIR = 'C:\t\cargo-targets\genet'
$env:CARGO_BUILD_BUILD_DIR = 'C:\t\cargo-targets\genet\build'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
cargo build -p genet-webrtc --example interop_host --locked --offline -j 1
node support/ci/webrtc_interop.mjs --server NUMERIC_WSL_IP:34790
```

The receiving fixture exposes coturn through a bounded Windows-facing UDP
front end, forwarding unchanged datagrams to the WSL server with one backend
socket per client. Direct cases use no discovery server. Firefox receives an
origin-scoped local/loopback-network permission through BiDi, reset on shutdown;
WebRTC preferences remain at their defaults. This qualifies the named local
network path, not public TURN or arbitrary interface routing.

Done: both Boa and Vano exchange text and fragmented binary with each headed
browser, directly and through forced TURN, in both offer/answer roles. Verify
the selected candidate pair, remote certificate fingerprints, channel modes,
and native close/navigation revocation plus actual UDP socket release. Record
default-browser mDNS refusals explicitly if the successful pair uses another
candidate. Require an OS-visible window for each automation-owned browser;
headless runs do not satisfy this gate. The complete matrix is 32 cases across
engine, browser, route, offerer and cleanup mode, with bounded case deadlines.

Append compact per-case results and source/tool identities to the existing
receipt file, at most 256 KB additional and within its existing 1 MB total cap.
Keep raw SDP, ICE credentials, browser profiles and exploratory logs out of
receipts. Build artifacts reuse the stable Genet target. Automatic approval
review rejected recursive deletion of a verified, exited test profile; retain
the two bounded marker-owned profile caches for reuse, with their browser
owners stopped. Failure remains recorded even when a later fix
passes. DNS, mDNS resolution, server replacement, TURN TCP/TLS, public
deployment and media remain open unless separately qualified.

### E2r. ICE restart on established data channels

**Status (2026-11-03): provider/native/WPT qualified; full headed matrix in progress.** Freeze a fresh native
runner and unchanged WPT case map before implementation. Exercise
`RTCDataChannel-iceRestart.html`, both `RTCPeerConnection-restartIce` files and
the existing constructor/channel/ICE-error controls on Boa and Vano. Retain
media-dependent failures with their actual case identities.

Done: `restartIce()` and `createOffer({iceRestart:true})` rotate credentials
without closing existing channels; requests survive rollback and schedule
`negotiationneeded` after the operations chain returns to stable. Restart after
close has no effect. Candidate generations reject stale signaling while a
previously authenticated data path remains usable during negotiation. Test
direct UDP, configured STUN and forced TURN against installed headed Chrome
and Firefox, both engines and both initiator roles, including close/navigation.
The provider owns gathering, bounded timers and reuse/release of existing TURN
allocations; Genet owns web semantics and generation-tagged observations.

Append compact baseline/final WPT maps, provider/native gates, headed restart
results and exact source/lock/binary identities to the existing receipt. The
additional E2r retained cap is **220 KB**, within the existing **1 MB** total;
use a delta from the archived full lock. Reuse the approved stable targets.
Core defects are repaired in the supplier rather than duplicating protocol
processing in Genet. DNS/mDNS, server replacement, TCP/TLS, media and carrier
cutover remain separate gates.

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
- The native route gathers host and configured IP-literal UDP STUN/TURN
  candidates. Relay-only construction owns no route when no usable relay is
  configured; otherwise it owns a control socket and relay candidates, suppresses
  direct traffic, and conceals the physical related address in candidates/SDP.
  Gathering waits for server results or a caller-selected deadline, with typed,
  sanitized `icecandidateerror` events and a final null candidate. Local server
  URL/relay protocol metadata stays out of candidate signaling JSON. Route-policy
  changes and ICE-server replacement refuse with `NotSupportedError`.
  Fixed-server restart refreshes STUN transactions and retains TURN allocations.
  DNS, TCP/TLS and mDNS are still open. Custom certificates,
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
  identity: [retained dependency report](https://github.com/merely-made/murm/blob/b0e31de8b581940dcd347968585c401e3ad786ca/testing/webrtc-evaluation/receipts/dependency-report.json).
- The existing carrier's RustCrypto feature does not imply a C/assembly-free
  build. A locked offline reverse tree for `murm-webrtc --features native`
  reaches AWS-LC via `str0m-rust-crypto -> dimpl 0.7.4`'s `rcgen` feature;
  that feature explicitly enables AWS-LC. The isolated graph repeats this at
  dimpl 0.7.6. The old manifest comment claiming no build tooling was corrected;
  provider features and wire/API suitability remain separate decisions. No
  crypto implementation or shipping dependency was changed.

## Headed interoperability findings (2026-11-02)

- `support/ci/webrtc_interop.mjs` controls installed headed browsers through
  CDP/BiDi and checks OS-visible windows. Its JSONL native fixture runs the real
  runtime and factory. Both channel modes, Unicode text, a fragmented binary
  payload, selected UDP route, independently observed remote certificate and
  close/navigation socket release are asserted. Tests retain compact failures
  and source identities in the existing receipt.
- The pinned core enables mDNS queries by default. A multicast query cannot
  use the provider's single unicast UDP route and could retain a refused packet
  ahead of usable traffic. The native factory disables that unsupported path
  and explicitly refuses unresolved `.local` candidates. Working numeric and
  peer-reflexive paths remain usable; mDNS resolution is still open.
- Core candidate-pair references used a different ID namespace from candidate
  stats rows, and internally learned peer-reflexive candidates lacked rows.
  The native adapter now supplies observations from actual ICE candidates and
  resolves those references. Remote stats addresses are conservatively null;
  remote server URL/relay-protocol fields and peer-reflexive candidates in
  `getRemoteCandidates()` are omitted, preserving the web API's privacy boundary.
  See the [stats address rule](https://www.w3.org/TR/webrtc-stats/#dom-rtcicecandidatestats-address)
  and [remote-candidate getter](https://www.w3.org/TR/webrtc/#dom-rtcicetransport-getremotecandidates).
- In this receiving environment Firefox's direct WSL server path timed out.
  The bounded Windows UDP front end makes both browsers reach the same real
  coturn fixture. Scoped browser network permission is recorded separately
  from ICE gathering errors; permission alone did not establish that WSL path.

## ICE restart findings (2026-11-03)

The runtime remembers restart requests across rollback and pending offers,
coalesces `negotiationneeded` after the operations chain, and treats restart
after close as a no-op. Native observations and candidates carry generation
tags; stale signaled candidates refuse before entering the protocol core.
The same fixed UDP socket regathers STUN with new transactions and retains
TURN allocation/permission/refresh state. Server replacement and interface
rebinding remain separate work.

Two core defects required a maintained `merely-made/rtc` fork based on upstream
`70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38`: losing the nominated data route
during restart and choosing a new DTLS answer role when the offerer changes.
The continuing role is retained only for the same remote fingerprint/TLS ID,
as required by [JSEP](https://www.rfc-editor.org/rfc/rfc9429.html#section-5.11).
The old nominated route and its authenticated consent responder are separate
from fresh candidate/check tables, expire under a fixed bounded deadline and
are cleared on replacement, failure or close. Repeated restarts and old consent
cannot extend that deadline. See [ICE restart](https://www.rfc-editor.org/rfc/rfc8445.html#section-9)
and [consent freshness](https://www.rfc-editor.org/rfc/rfc7675.html#section-5.1).

A configured-STUN probe stalled with no fresh checks observed at the native
core after its previously learned numeric address was discarded. Once fresh
remote credentials arrive, the core now checks that known nominated address
as an unchecked candidate. Prior nomination and transactions are discarded;
stale-key responses cannot validate the fresh pair. The passing STUN and
forced-TURN probes qualify this continuing-route behavior. Broader initial
checks before remote credentials arrive still need their own RFC 8445 section
7.3 gate, alongside DNS/mDNS and interface/server replacement.

`support/ci/webrtc_interop.mjs --restart both` exercises both APIs, both
offerers, every SDP step and an interrupted restart before teardown. Use
`--receipt-mode stdout` to retain a compact result after shared Git hooks;
default mode appends within the lane cap. The WPT comparison keeps exact
assertion identities and media failures. Its baseline ERROR completed only the
first data-channel restart assertion; the unchanged source names the second
pending assertion explicitly, so recovery does not silently change the inventory.

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
- 2026-11-01: Murm E1c now feeds IP-literal UDP STUN/TURN into Genet's native
  host. Both engines qualify encrypted forced-relay transfer, private related
  addresses, local-only server metadata, gathering errors/completion and
  allocation lifecycle against controlled coturn. The original selected WPT
  cases retain their identities/results; the additional ICE-error constructor
  case and Ortet receiving check pass. Native discovery remains distinct from
  headed/browser-guest, public TURN, DNS/TCP/TLS, restart and carrier acceptance.
- 2026-11-02: E2h qualifies the installed headed Chrome/Firefox data-channel
  matrix against Boa and Vano through direct and forced-UDP-relay routes,
  both offerer roles, and native close/navigation teardown. The same pass
  fixes an undriven mDNS query path, candidate stats references and remote
  observation privacy. Targeted privacy checks and native regression gates
  accompany the full matrix in the existing receipt; broader E2 and E3 remain open.
- 2026-11-03: E2r implements sticky restart requests, generation filtering,
  continued data/DTLS identity, fixed-server STUN/TURN gathering and teardown
  during a pending restart. Provider checks pass on working Rust and MSRV;
  native Boa/Vano, strict Clippy, Ortet compilation and the unchanged selected
  WPT comparison pass. Both restart WPT assertions and the closed-peer no-op
  pass on both engines; media failures and preliminary ERROR rows remain in
  the receipt. The full 48-case headed restart matrix is in progress.

## UDP STUN and TURN receiving gate

Receive Murm E1c with IP-literal UDP servers, bounded gathering and allocation
refresh. Start discovery on local-description gathering, preserve candidate
server metadata, and surface sanitized gathering errors. Relay-only construction
must retain its no-direct-route guarantee, including refusal and timeout paths.
Changed server configuration must refuse until replacement is supported.
Fixed-server restart uses the same physical socket and relay allocation.
Host name resolution, TCP/TLS, mDNS and carrier cutover remain open.
Headed direct/relay interop is qualified separately by E2h. Done: both engines
exchange encrypted messages using forced relay
against coturn; failure/close paths release their sockets; existing native and
selected WPT gates retain their case identities and results. Append compact
evidence to the existing receipt file within its 1 MB total cap; record supplier
and root-lock deltas rather than duplicating the full lock.

## Decisions

- **webrtc_conformance D2 (2026-11-03, Mark):** "Ok, go ahead with that ICE
  restart slice, including STUN/TURN and the automated tests." Qualify
  established channels on unchanged UDP sockets and fixed configured servers;
  retain the separate DNS/mDNS, interface/server replacement, TCP/TLS, media
  and application-carrier gates.
- **webrtc_conformance D1 (2026-11-01, Mark):** "Okay. So, shall we continue
  with RTC?" Continue with rtc as the selected implementation direction after
  the E0 comparison. This authorizes the provider/Genet work; existing carrier
  and WPT claims still require their named receipts.
  Links: area web-platform · plans E1 E2 E3 · summary Continue with RTC
