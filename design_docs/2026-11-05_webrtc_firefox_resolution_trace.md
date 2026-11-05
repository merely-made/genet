# Firefox restart resolution and attachment trace

Status: traced both offerers; normal mDNS resolution path remains unresolved. Follow-up to the closed
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
