# Firefox restart resolution and attachment trace

The multicast follow-up reproduced an interface mismatch with both initial offerers. A separate run reached the candidate parser and fell back. Follow-up to the closed
[restart investigation](2026-11-05_webrtc_firefox_mdns_restart.md).

## Scope and receipts

Trace the installed Firefox build at first restart for each initial offerer.
Keep native names, privacy policy, normal signaling, supplier/core pins and
WebRTC preferences unchanged. Counters and fixed event labels only; do not
retain raw logs, SDP, ICE credentials, hostnames or addresses. Use the stable
Genet target and marked test profiles. No isolated target/home/worktree.

Retain a fresh packet at
`testing/genet/design_docs/archive_docs/receipts/2026-11-05_webrtc_firefox_resolution_trace_receipts.md`.
Total retained receipt cap: 200,000 bytes, including failed controls. Preserve
all older frozen packets. Each process and fixture-origin grant must be
restored. Direct-only cases do not start STUN/TURN services.

Done conditions: identify lookup request/completion and candidate generation
attachment, or record the exact missing observable boundary. Demonstrate initial
success as a positive control. Any correction requires regression checks and
fresh acceptance evidence; diagnostic changes alone do not qualify the eight
failed cases.

## Installed-source boundary

Firefox 159.0.1, build 20261102144711, release source
`7b34a03a910d0385299f4a9f27aa585a786862d8`. The installed application and platform
metadata agree. Exact source was fetched read-only from Mozilla's release
repository. PeerConnectionImpl SHA-256:
`918068eebc2080e170de43acbf3d10c3643f7fe8a82102548e38025622129988`.

The source stamps lookup start, lookup success immediately before forwarding
the queued candidate to MediaTransportHandler, and lookup failure. Its timecard
prints to stdout at peer destruction. Existing runner diagnostics discarded
stdout. The ICE transport logs parsing only after GetStreamForRemoteUfrag
returns a stream; process errors and missing streams have separate messages.
A successful parse log proves generation lookup succeeded, not nomination.

Sources:
[resolution callback](https://hg.mozilla.org/releases/mozilla-release/file/7b34a03a910d0385299f4a9f27aa585a786862d8/dom/media/webrtc/jsapi/PeerConnectionImpl.cpp#l4237),
[generation lookup and parser](https://hg.mozilla.org/releases/mozilla-release/file/7b34a03a910d0385299f4a9f27aa585a786862d8/dom/media/webrtc/transport/nricemediastream.cpp#l292).

## IPC observation and positive control

Process-local `--firefox-ipc yes` enables Mozilla's documented
`MOZ_REMOTE_ALLOW_SYSTEM_ACCESS=1` only on the owned test Firefox process. The
loopback BiDi connection reads the browser chrome context and starts a bounded
IPC-only profiler with stack sampling disabled. Full profiles are reduced inside
Firefox; only request/completion endpoint counts and relative times leave it.
The profiler is stopped with `discard:true`; no downloaded profile is created.
This does not change WebRTC preferences or apply to the user's ordinary profile.
See [Mozilla's system access documentation](https://firefox-source-docs.mozilla.org/remote/Security.html#system-access).

Both initial offerers show three initial request messages and one completion,
with named candidate parsing. At restart three more requests reach the parent,
then one completion arrives roughly nine seconds later. No new named candidate
parse, missing-stream error or process error appears. A completion IPC by itself
does not expose whether its optional address was present. The interval and
absence of parsing suggest the mDNS timeout path; they do not prove the callback
payload. Timecards remain unavailable on this Windows stdout path.

A separate `--firefox-resolution-replay yes` control runs only after the failed
case and the 12-second completion observation. It re-adds the actual native host
candidate for the current description and sends at most 20 fixture-owned
multicast A answers for that observed native UUID. It never rewrites native
signaling and cannot qualify the native responder. Its purpose is to distinguish
an unanswered lookup from inability to attach a resolved candidate to the new
generation. Record its IPC, parser and host-graph deltas separately.

## Observed result

| Boundary | Browser initial offerer | Native initial offerer |
| --- | --- | --- |
| Initial control | Three requests, one completion; named candidate parser reached | Three requests, one completion; named candidate parser reached |
| Normal first restart | Three new requests reach parent; completion after 9.019 s; no new candidate parse | Three new requests reach parent; completion after 9.036 s; no new candidate parse |
| Post-failure replay with fixture answers | New request/completion in about 154 ms; parser reached | New request/completion in about 157 ms; parser reached |
| Replay transport result | One parse, one redundant result, zero missing-stream/process errors | One parse, one redundant result, zero missing-stream/process errors |

Normal IPC runs: `1c93bfc5-e108-4cad-ad63-0c806791087f` (browser initial offerer)
and `42717152-4644-424e-a1ce-207d6c17ed6a` (native initial offerer).
Corrected replay runs: `e487cc54-9004-45cd-acf7-06456f973473` and
`4745d840-5dab-49d1-a660-5ccc605e2840`, respectively. Source, native executable
and lock identity are stable at every checkpoint. Approximate profiler times
are sufficient to distinguish 0.15 s from the 9 s retry/timeout interval; they
are not a precision latency benchmark.

The replay uses the fresh native username fragment. The exact installed source
only reaches its parser after finding a stream for that fragment. This is
positive evidence that a resolved candidate can reach the matching restarted
generation. The parser reports it redundant after the earlier ICE outcome; both
host graphs remain empty and these failed cases are not rescued or qualified.
The parser log does not prove candidate insertion or nomination. The existing
peer-reflexive route is not promoted to named-host acceptance.

The normal completion IPC does not reveal its optional address payload. Its
roughly nine-second delay and absence of parser/process errors are consistent
with the installed resolver's three-attempt timeout, rather than a demonstrated
successful lookup followed by rejection of the new generation. The fixture
control supplies a successful resolution and observes forwarding. Together,
these narrow the next investigation to the ordinary multicast query/answer
path, including outgoing interface and receive/socket delivery. They do not
assign the defect to Firefox or the native responder, or prove which packet
is missing. Direct-unicast responder support remains a separate gap.

Six failed strict cases are preserved across six runs, including the stdout-only
attempt and the first replay setup failure. That setup failure occurred before
controlled answers were sent and is explicitly corrected in the packet. The
new single-case IPC guard keeps future profiling bounded. Profiler teardown
always requests discard, and a failure to stop it cannot skip browser permission
reset and process shutdown.

No product Rust, supplier source, dependency pin or Cargo.lock changed. The
normal executable remains SHA-256
`0c3e031f2efe09d13136743d5828a3c45604cfaf322647b3816634016356d773`,
with lock SHA-256
`b3a0a9fb35c0a0ab4ba1ed79061ce14ec017dbbc15f78c349affe60428da5030`.
Earlier acceptance remains 40/48, with all eight Firefox direct cases open.
The diagnostic controls are not native advertisement, cross-machine LAN, WPT,
IPv6, STUN/TURN, media or carrier qualification.

The fresh packet is closed at 177618 bytes of its 200,000-byte cap,
SHA-256 `31e8f7ece4f3add924eb5d9c7db7fa5b293a7915ffb2eee91b6f149c059df950`. All 31 Node helper checks, runner syntax and required commit
checks pass. Final audit confirms zero owned test processes and fixture network
grants, inactive/disabled coturn, and unchanged older frozen packets. No isolated
target/home/worktree or downloaded profiler artifact remains. Existing marked
profiles and the stable Genet target remain for reuse.


## Multicast path follow-up

The installed Npcap driver supports non-promiscuous IPv4 observation on the
Wi-Fi and WSL adapters and the loopback capture device. The test runner accepts
`--mdns-wire-interface` with the other adapter's IPv4 address for one explicit
Firefox IPC case. Npcap selects both adapters by address. The capture helper
opens no UDP sockets and changes no multicast memberships. It filters to UDP
5353 and reduces only the test UUID to bounded packet events. The adapter
and source roles are reported separately. The loopback rows show local copies of adapter packets and are
excluded from the transmission counts below. [Npcap loopback details](https://npcap.com/guide/npcap-devguide.html#for-software-that-uses-npcap-loopback-feature).

The capture runs with the pinned Murm registration and the existing fixture
socket. The fixture joins the selected adapter and observes the native name
without answering for it. Firefox's installed source binds a shared wildcard
UDP 5353 socket, joins its supplied interfaces, and sends queries on that socket
without setting an explicit multicast egress interface. A plain Windows UDP
route probe selected Wi-Fi. Firefox packets themselves were observed on both
adapters. [Installed resolver source](https://hg.mozilla.org/releases/mozilla-release/file/7b34a03a910d0385299f4a9f27aa585a786862d8/dom/media/webrtc/transport/mdns_service/src/lib.rs#l441).

In the first two Wi-Fi captures, initial lookups sent a query from WSL and then
consumed a Wi-Fi A record. The native-offerer restart
`697f9f95-e257-4432-a4bc-fc6f32acfa34` sent three A questions on WSL at
about three-second intervals. The Wi-Fi fixture read zero questions for the
native name, and its sole positive A record was from the initial phase. The
adapters and loopback showed no restart answer. Firefox's parent sent completion
9.029 seconds after the restart request, and the named parser count stayed at
three. This locates the failing lookup on the interface boundary for this run.
Murm's registration joins Wi-Fi and checks the received interface and source
prefix before accepting a query.

The first browser-offerer run `d05632c6-8228-47dd-89fc-a5260839856e` sent two
restart questions and received two positive multicast A records on Wi-Fi. The
fixture read those questions and answers. Firefox's first restart completion
followed the parent request by 11 ms, and the transport logged three additional
named parse attempts, including a redundant result. The selected route was
still peer-reflexive. This trace reaches the Firefox receiver and candidate
parser, while nomination remains a separate failure. The first run had a
shorter pcap statistics structure. The subsequent runs use the full Windows
layout.

With the registration and fixture bound to WSL, control
`a452b045-f278-4071-a4aa-43a6a02f7265` saw the WSL query and A record on the
adapter and at the fixture socket. Firefox completed and parsed the name. The
case then hit the initial application exchange deadline. This control establishes
mDNS delivery on the aligned interface through parser entry. It supplies no
restart exchange result.

Adapter capture and fixture reads observe separate boundaries. Firefox IPC
completion paired with named parser entry establishes successful resolution.
The trace does not intercept individual Firefox Winsock reads. A quiet
Firefox mDNS log is expected for those trace-level calls in a release build.
[Mozilla's Rust logging documentation](https://firefox-source-docs.mozilla.org/testing-rust-code/index.html#gecko-logging).


The address-filtered browser-offerer repeat
`e3c8990d-25d5-438e-8e15-5ecf4ec32608` captured the initial Wi-Fi record.
The restart completion took about nine seconds without a new named parse.
Its capture did not locate the restart packets. The final run removed the
source-address predicate and added per-adapter filtered datagram counts.

That final browser-offerer run `9a9506df-2f96-44ed-8606-72e484231627`
saw a Wi-Fi question and answer at initial connection. At restart, all three
queries left through WSL. The fixture remained at one native question and
one native answer across both snapshots. There was no restart answer on
either adapter or loopback, and no new named parse. Completion took 9.039 s.
The complete statistics layout reported zero capture drops and no watched
events omitted. This reproduces the interface mismatch with the browser as
initial offerer too.

| Case | Restart query interface | Restart delivery and outcome |
| --- | --- | --- |
| Wi-Fi registration, native initial offerer | WSL, three queries | No A answer or new named parse. Channel closure |
| Wi-Fi registration, browser initial offerer, first trace | Wi-Fi, two queries | Two A answers and new parser entries. Peer-reflexive fallback |
| Wi-Fi registration, browser initial offerer, final trace | WSL, three queries | No A answer or new named parse. Peer-reflexive fallback |
| WSL registration control, initial exchange only | WSL | Query and A record reach the fixture. Firefox parses the name, then the application exchange stalls |

The interface mismatch accounts for the unanswered lookups in these two
restart captures. The first Wi-Fi restart trace also establishes a failure
after delivery. The evidence leaves the Windows egress decision and the
post-resolution nomination path as separate questions. Earlier acceptance
remains 40/48, with all eight Firefox direct cases open.

The fresh multicast packet is closed at 157,531 bytes of its 240,000-byte
cap. Its SHA-256 is `1895d62e35d0b86e0467376d389a8401a13a79cf4dd42f2e1b1b8ea4b1f4ef62`.
The trace used the prior RTC host and lock identities recorded in the packet.
All 31 Node helper checks, two Python reducer checks, runner syntax and
required commit checks on the trace base pass. The audit found zero owned
trace processes, fixture network grants, raw packet files or profiler exports.
The existing Genet target and marked Firefox profile remain available for
reuse.


## Peer-reflexive nomination after resolution

The browser-offerer trace runs with the installed Firefox and the existing
native executable. The `pair-order` diagnostic supplies a bounded series of
fixture A answers for the owned native UUID during the restart answer. This
lets the candidate resolve independently of the multicast interface mismatch.
The `pair-order-hold` comparison uses the same answers and temporarily holds
browser candidates and end signals going to Genet. It strips embedded browser
candidates and end markers from the first restart offer while keeping the
fresh ICE credentials. The held trickle signals are released after Firefox
logs a nomination. The second native-offerer round uses ordinary signaling.
The reducer records stream and pair ordinals, candidate type and equality
with the native UDP endpoint. It keeps addresses, names and credentials in
process memory. The process-local `nicer:5` module supplies the ICE events.
[Firefox WebRTC logging modules](https://firefox-source-docs.mozilla.org/contributing/debugging/debugging_webrtc_calls.html#standard-logging-modules).

With ordinary browser candidate forwarding, Genet can send a fresh STUN
check before Firefox receives the answer. Firefox queues that check and
processes it when the fresh stream becomes active. Since the named candidate
has not reached the checklist, the source address matches no pair. Firefox
creates and triggers a peer-reflexive pair using the source IP and port.
This is the same native endpoint advertised by the mDNS candidate. Firefox
controls nomination in the traced browser-offerer restart.
[Installed nICEr inbound-check path](https://hg.mozilla.org/releases/mozilla-release/file/7b34a03a910d0385299f4a9f27aa585a786862d8/dom/media/webrtc/transport/third_party/nICEr/src/ice/ice_component.cpp#l920).
[ICE peer-reflexive discovery](https://www.rfc-editor.org/rfc/rfc8445.html#section-7.3.1.3).

Run `22e079f9-c6ca-478c-a36e-42d99e6f9fd7` shows the queued check processed
before the named parser entries. Some entries precede the peer-reflexive
nomination. A later entry is refused in the connected state. The earliest
parser entries alone do not establish that a named pair was checked.

The final repeat `1d3cb14e-0130-448d-ab67-70bdd826613f` records the
peer-reflexive insertion and nomination before every named parser entry.
All three entries hit nICEr's connected-state gate. It returns `R_ALREADY`
before candidate insertion, which the transport reports as redundant.
The redundant label describes stream state in this path.
[Installed trickle state gate](https://hg.mozilla.org/releases/mozilla-release/file/7b34a03a910d0385299f4a9f27aa585a786862d8/dom/media/webrtc/transport/third_party/nICEr/src/ice/ice_peer_ctx.cpp#l295).

Hold control `3ea9eb74-787b-4ef1-9299-c29eff57ea25` defers six browser
signals. Firefox logs no queued fresh check in the first round. It parses
the named candidate, nominates the same UDP endpoint as a host, and keeps
that selection after the six signals are released. That browser-offerer
round completes. The following native-offerer round closes a channel.
The native aggregate check counters also include activity on the retained
route. The Firefox queue events identify the fresh pre-answer check.
Event times reflect controller receipt and serve as an ordering aid.

The first two exploratory runs preserve the coverage steps. The streaming
mtransport log showed parser entry and redundancy. Firefox's internal ring
at default verbosity confirmed a new peer-reflexive nomination while omitting
the debug-level queued check events. The later `nicer:5` runs provide the
ordering evidence. The mDNS fixture answers and signaling hold are diagnostic
controls. Acceptance remains 40/48 with the eight Firefox direct cases open.

The fresh nomination packet is closed at 162,763 bytes of its 240,000-byte
cap. Its SHA-256 is `0c1b2654275ad69de69073870a74e163fbda0cb665c5c8851518fdf80f124169`.
Thirty-three Node checks, two Python checks, syntax and line ceiling pass.
The dependency-bans commit hook waited on a shared Cargo package cache lock.
The trace changes are staged locally. The test audit found zero owned browser,
native or controller processes and zero fixture network grants. The stable
Genet build and marked test profile remain available for reuse.


## Native-offerer channel closure

Run `d33897f7-63ec-40f1-afa8-563c5c132e59` completes the controlled
browser-offerer round and reproduces the channel closure in the following
native-offerer round. The ordinary second-round mDNS questions leave through
WSL and receive no answer. The Firefox parser count stays at six throughout
that round. Firefox applies its local answer and starts the five-second
trickle grace period. Genet still has a local offer and has not applied the
answer because the runner probes the retained data path before forwarding it.

When the grace period expires, Firefox logs checklist exhaustion for the
fresh generation. Its report has zero successful streams and one failed
stream. It then dumps the ICE ring buffer. The named-host nomination in that
dump belongs to the previous generation, identified in memory from the
local username fragment in the stream label. The live `nicer` events and the
`mtransport` replay are now counted separately. The reported nomination from
the earlier failure trace therefore cannot establish that the fresh stream
was nominated. The JavaScript selected-pair event clears the retained pair
as ICE fails.
[Installed failure and ring dump](https://hg.mozilla.org/releases/mozilla-release/file/7b34a03a910d0385299f4a9f27aa585a786862d8/dom/media/webrtc/transport/nricectx.cpp#l407).

The same ICE and DTLS objects are present throughout the failed restart.
Firefox reports the DTLS transport failed, SCTP closed, and then both data
channels closed. The installed ICE layer propagates stream failure to DTLS,
and the data-channel connection closes all channels when the underlying
transport enters error. Genet's channels are still open at this checkpoint.
Its fresh check, fresh response and selected-pair counters have not advanced.
The retained route still exchanges application payload during the answer hold.
[ICE failure signal](https://hg.mozilla.org/releases/mozilla-release/file/7b34a03a910d0385299f4a9f27aa585a786862d8/dom/media/webrtc/transport/transportlayerice.cpp#l139).
[DTLS lower-layer error](https://hg.mozilla.org/releases/mozilla-release/file/7b34a03a910d0385299f4a9f27aa585a786862d8/dom/media/webrtc/transport/transportlayerdtls.cpp#l982).
[Data-channel closure](https://hg.mozilla.org/releases/mozilla-release/file/7b34a03a910d0385299f4a9f27aa585a786862d8/netwerk/sctp/datachannel/DataChannel.cpp#l336).

Run `ace9b3d2-d296-4671-ad12-8c1f60bdf32f` supplies a natural comparison.
A named pair enters the fresh checklist immediately after Firefox's answer.
It remains viable when the same five-second grace period expires. Genet then
receives the answer, Firefox nominates the named pair, and the existing
DTLS and SCTP transports and channels remain connected. Both rounds complete
in this diagnostic case, whose first round holds browser candidates and
supplies fixture answers.
$

Run `d3611da4-7f7e-4503-8ac5-26cd5976dcef` repeats the answer hold and
supplies 32 fixture A answers for the native offer's name. Firefox parses
the name about 12 ms after the answer is available and keeps the fresh
checklist viable through grace expiry. Genet gets the answer 5.78 s after
it is available. Firefox nominates the fresh named pair at the advertised
native endpoint about 230 ms later. The same ICE and DTLS objects and both
channels remain connected. Both restart rounds complete.

The native-offerer packet closed at 189,223 bytes of its 240,000-byte cap.
Its SHA-256 is `c3670b782facd475ab09c622ffd266166403420eb552db54037ca8d0e97c2023`. A shared-checkout source change invalidated one
attempt. The failure and assisted comparison were recorded from an isolated
checkout at `e8ebe2e06da` using the same native executable. All 36 helper
checks passed. The advertisement acceptance matrix remains 40/48, with
eight Firefox direct cases open.$
