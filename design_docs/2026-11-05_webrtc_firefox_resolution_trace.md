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
