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
