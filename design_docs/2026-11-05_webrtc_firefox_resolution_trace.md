# Firefox restart resolution and attachment trace

Status: in progress. Follow-up to the closed
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
