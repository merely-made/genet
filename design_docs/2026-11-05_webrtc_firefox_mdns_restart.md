# Firefox direct mDNS restart investigation

Status: investigation recorded; all eight direct acceptance failures remain open.
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

## Findings

The fresh packet retains 17 runs and 25 failed diagnostic cases, including
unsuccessful controls. These are diagnosis receipts, not a replacement passing
matrix. Firefox is installed version 159.0.1, headed, with its WebRTC preferences
unchanged. The normal supplier/core pins and native UUID privacy policy remain
unchanged.

| Control | Observation | Limit |
| --- | --- | --- |
| Both initial offerers, normal binary | Reproduces channel closure or selected peer-reflexive fallback at first restart | Initial named-host exchange succeeds |
| Add only native trickle capability | Still fails both cases | Missing capability is not a sufficient explanation |
| Hold native end markers; separately use actual trickled candidates | Native-initiated restart still fails | Does not exclude every asynchronous resolution race |
| Observe selected and WSL interfaces | Still fails; one run changes closure into live fallback, others do not | Not a deterministic routing fix |
| Count remote host candidates independently of selected pair | Initial Firefox host count one, named one; after restart host count zero | Stats cannot identify whether resolution failed or a candidate was removed |
| Independent queries after failure | Normal registration answers multicast from ephemeral and shared-5353 sources; direct unicast gets no answer | These probes are not browser advertisement consumption |
| Temporary selected-address socket reader | All three probe paths answer; strict restart still fails | Unicast support alone does not fix acceptance |
| Temporary ICE-lite control | Fresh ICE connection still times out with no host candidate exposed | Changes ICE behavior; not an acceptance profile |

The Firefox counter logs expose ICE checklist failures, but their mDNS/platform
DNS counters are also zero during successful initial resolution. Those zeros
cannot establish that Firefox omitted resolution. Neither a DTLS/SCTP defect nor
a specific Firefox lookup/cache defect is established.

The query-counter erratum in the packet is material: three prototype runs sampled
the selected socket after our independent queries. Their single A query cannot
be attributed to Firefox. Corrected run
`8e48b032-d954-4d84-8dcd-3ea4b3ff1bcc` snapshots zero before probes and one A after
probes, while still failing the named-host route. Two earlier endpoint probes
also serialized candidate objects without explicit address/port fields; the
packet preserves that erratum and subsequent validated endpoint measurements.

The temporary provider, Genet ICE-lite builder and local Cargo patch were
removed. Source and ignored root lockfile were restored byte-for-byte. Normal
`cargo build -p genet-webrtc --example interop_host --locked --offline -j1`
passes; restored executable SHA-256 is
`0c3e031f2efe09d13136743d5828a3c45604cfaf322647b3816634016356d773`.
This rebuilt artifact has build evidence only, not a rerun of the earlier 40/48
matrix. Twenty-five Node helper checks and runner syntax checks pass.

## Next bounded slice

Trace the installed Firefox build's native-name resolution request and completion
across the first restart, then distinguish failure to resolve from failure to
attach the resolved candidate to the new ICE generation. Preserve both initial
offerers as reproductions and use unchanged normal signaling as the acceptance
control. Existing generic logs do not provide that boundary, so more speculative
signaling or routing changes are not justified.

Separately, direct-unicast mDNS handling is a demonstrated provider gap. Any
production correction needs shared socket/demultiplexing ownership, selected
interface/destination checks, bounded processing and multi-peer cleanup tests.
The discarded one-peer socket reader is insufficient evidence for that design.
Do not repin it or count it as resolving these eight failures.

The canonical advertisement acceptance remains Chrome 24/24, Firefox 16/24.
STUN/TURN, WPT, native LAN checks and earlier frozen receipts were not rerun or
requalified by these diagnostic experiments. Cross-machine LAN and IPv6 remain
separate gates.
