# Firefox direct mDNS restart investigation

Status: investigating the eight failed direct cases in the closed E2ah packet.
The source and byte identities of that packet remain frozen in the
[mDNS plan](2026-11-04_webrtc_mdns_plan.md#final-e2ah-checkpoint).

## Scope and done conditions

Reproduce one case per restart initiator, then separate signaling/description
handling, candidate resolution, multicast-interface delivery and ICE timing.
Keep native UUID names and per-peer lifetime unchanged. Do not substitute
numeric native candidates or alter browser WebRTC preferences, firewall rules
or interface routes to count a pass. Controlled incoming Firefox aliases remain
separate from native advertisement proof.

Retain a fresh compact packet at
`testing/genet/design_docs/archive_docs/receipts/2026-11-05_webrtc_firefox_mdns_restart_receipts.md`,
with a total 350,000-byte cap including unsuccessful experiments. Preserve the
closed E2ah/WPT/receiving packets. Do not retain full SDP, ICE credentials,
packet captures or unbounded browser logs. Record exact source/binary/lock and
browser identities, diagnostic settings, bounded counters and outcomes.

Use the existing stable Genet target and marked browser profiles. No isolated
target, Cargo home or worktree is needed. Direct-only runs use an unused
loopback server argument because the existing harness requires an endpoint;
their peer configurations contain no STUN/TURN URL, so no coturn service or WSL
fixture is started. Restore origin permissions and owned processes afterward.

If a correction is justified, add regression checks and rerun all eight Firefox
direct cases with both engines, offerers and cleanup modes, plus relevant
controls. Otherwise retain a reproducible diagnosis and distinguish demonstrated
causes from unresolved hypotheses. Update the canonical mDNS status either way.

## Starting evidence

- Four native-initiated restarts fail before Genet receives the browser answer;
  Firefox enters failed ICE and closes channels. Four browser-initiated restarts
  retain live channels but select a peer-reflexive remote route.
- Initial named-host exchange succeeds in all eight. Earlier IPv4 observations
  count one native answer and no watched native question in these cases.
- The installed machine has connected Wi-Fi and WSL IPv4 interfaces. Their
  multicast routes have equal route metrics, with interface metrics 35 and
  5000 respectively. This does not establish which interface Firefox uses.
- Mozilla's current sources queue mDNS resolution asynchronously during transport
  activation. Its mDNS service joins IPv4 interfaces without explicitly selecting
  an outgoing multicast interface. These are investigation leads, not proof of
  the installed Firefox build's failure cause.

Primary source references:
[candidate activation](https://github.com/mozilla-firefox/firefox/blob/main/dom/media/webrtc/jsapi/PeerConnectionImpl.cpp),
[mDNS service](https://github.com/mozilla-firefox/firefox/blob/main/dom/media/webrtc/transport/mdns_service/src/lib.rs),
[end-of-candidates tracking issue](https://bugzilla.mozilla.org/show_bug.cgi?id=1535690).

## Experiments

- Unchanged source `7540712f5b3`, native binary `f8db64d4b082d7831af7804d2b04432de1cd28d540213cadec4a0e9508181797`:
  both Boa/direct/close cases reproduce the original failures. Run
  `349abd68-1db9-44b6-a9c6-0188232a359a`, 0/2 pass, source/binary/lock stable.
- The runner adds explicit `--restart-diagnostic observe|defer-end|trickle`
  controls, restricted to local-mDNS direct cases. `observe` records bounded
  candidate counts, components, generation equality and end-marker timing.
  `defer-end` holds native end markers until fresh ICE connects; `trickle` also
  removes embedded candidates, preserving the real trickled candidate bytes.
  These modify test signaling only and are identified in run metadata. They
  cannot silently qualify the unchanged signaling profile. Unit checks verify
  unchanged browser signals, name/credential-free receipts and bounded storage.
