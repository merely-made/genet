# WebRTC remote mDNS candidates

**Status (2026-11-04): E2m in progress.** The completed E2s plan is
[archived whole](archive_docs/2026-11-04/2026-11-04_webrtc_conformance_plan.md).
Earlier rulings and qualification boundaries remain in that archive chain.

## Ownership and baseline

Genet owns signaling, promises/events, document lifetime and web-visible privacy.
The independent `murm-rtc` leaf owns caller-driven native resolution and I/O.
The maintained `rtc` core owns candidate identity, ICE pairing and protocol state.
Applications retain admission, framing and trust. This continues the user's
"Continue!" authorization; it does not create a new doctrine case.

E2s qualifies portless UDP STUN/TURN through SRV/S-NAPTR: provider
`ef2d18fadbbdca3847ad99040af14f2bc7693a47`, core
`479995928668d0db1f8cfb2aa26f818b9f03919f`, Genet product
`ff958976f2a904d250d00ee10335696c62f749b1`, headed recorder
`7abebdda633bac2d91bd05b4976f2bf65ea525cb`. Its closed external Nov 4
conformance packet retains 38 native checks and 48 headed Chrome/Firefox cases,
96 complete restarts and 48 interrupted cleanups. The selected seven WPT maps
remain 93/127 assertions passing per engine with 34 media-dependent failures.
Numeric, explicit-port and portless-server behavior remain regression gates.

## E2m. Receive remote mDNS candidates

Resolve remote host candidates using UUID-v4 `.local` names on the selected
IPv4 interface. This is the RFC 6762 one-shot client profile: an ephemeral
source port, link-local multicast destination, validated unicast replies and
a receive-only port-5353 membership for browser multicast replies, explicit local
subnet validation for unicast, bounded DNS parsing and an owned cancellation ticket. Reuse
the existing hickory wire dependency and resolver contract. Ordinary recursive
DNS must continue refusing `.local`; never fall back to it or change OS DNS.
Do not enable the core's unrelated multicast output queue on the RTC socket.

The core receives a trusted host-resolved candidate separately from its signaled
name. Keep the name and stable candidate identity while storing the numeric
transport endpoint internally. Pairing must never send a resolved remote mDNS
address through a local TURN relay. This rule applies whichever candidate is
added first, including later local candidates and peer-reflexive discovery.
Relay-only does not issue mDNS queries. Local mDNS advertisement/registration,
continuous browsing, shared caching, IPv6 multicast and interface rebinding
remain separate work.

Genet accepts trickled and SDP-embedded candidates without blocking its event
loop. Invalid candidate syntax still fails normally. Unsupported names and
failed/ambiguous resolution are ignored without an ICE-server 701 event. Bound
pending contexts, candidate observations, packet/record counts, retries and time.
Preserve signaled names in remote candidates and descriptions; resolved numeric
addresses must not reach JS or statistics. Provide stable remote statistics for
submitted mDNS candidates independently of resolution success. Restart, changed
remote credentials, close, navigation and drop discard pending work and late
answers. Keep numeric candidates and relay-only privacy unchanged.

Done conditions:

- Core checks prove signaled-name/transport separation, candidate identity and
  relay-pair exclusion in both insertion orders; old ICE/restart tests pass.
- Provider tests use real controlled sockets for multicast query placement,
  reply ID/question/source/subnet and actual destination/interface checks, malformed/oversized data, aliases,
  multiple addresses, wrong family, TTL-zero answers, quotas, timeout and drop.
  Working Rust 1.99.1, MSRV 1.97.1, strict Clippy and host-only graph pass.
- Boa/Vano tests establish encrypted data channels through mDNS candidates,
  including SDP ingestion, original-name/statistics privacy, success-independent
  observations, relay suppression and pending-generation/lifetime cancellation.
  Keep the existing 38 native checks and controlled STUN/TURN coverage.
- Headed Chrome/Firefox exercise both engines, offerers, cleanup modes and
  direct/STUN/relay routes. Distinguish controlled aliases from actual browser
  mDNS advertisements. The final matrix explicitly uses untouched Chrome names
  and controlled aliases for Firefox local test sockets. Firefox's own native
  responder qualification remains open. Require real query/response evidence and socket release;
  do not equate peer-reflexive success with successful name resolution.
- Freeze the same seven WPT files/helpers before product edits and compare exact
  assertion identities/statuses afterward, retaining known media failures.
  Compile Ortet's scripted host against published, exact core/provider pins.
- Restore owned fixtures/permissions and publish compact source/lock/binary
  evidence. Preserve shared WIP, archives and all closed packets.

Sources: [RFC 6762 sections 5.1 and 11](https://www.rfc-editor.org/rfc/rfc6762.html),
[ICE mDNS draft sections 3.2 and 3.3](https://datatracker.ietf.org/doc/html/draft-ietf-mmusic-mdns-ice-candidates-03).
The ICE document is an expired Internet-Draft, used as browser interoperability
and privacy guidance, not represented as an adopted RFC or full mDNS conformance.

## Evidence budget

Use the new external `Code/testing/genet/design_docs/archive_docs/receipts/2026-11-04_webrtc_mdns_plan_receipts.md`.
Retain at most **250 KB** total new evidence: 244 KB in that Genet packet,
including core receipts, and 6 KB new E2m entries in Murm's existing family
packet, which remains below 100 KB total. Keep full raw SDP, credentials,
profile data and captures out of public receipts. Every earlier standalone Genet RTC packet is
closed. Reuse stable Genet and Retinue Cargo targets/build directories,
debug=0/jobs=1; create no isolated target, Cargo home or worktree without a real
collision. Prior automatic approval rejection of profile/home deletion remains
in force; do not retry or bypass it.

## Remaining work

- Full E2 WebIDL/data-channel/statistics/renegotiation and broader named WPT
  groups, browser-guest and public-network qualification remain separate.
- Complete triggered ICE scheduling, requests before remote credentials, server
  replacement and interface discovery/rebinding retain their own core/host gates.
- Firefox's own native mDNS responder in this receiving environment remains an
  open interop gate. Controlled Firefox aliases qualify resolver/data-channel
  behavior; they cannot close native advertisement acceptance.
- Local mDNS name advertisement and IPv6 multicast require separate ownership,
  privacy, registration/conflict, cancellation and socket qualification.
- TURN TCP/TLS and broader allocation-error cooldown/redirect/authentication
  policy remain separate from the qualified UDP discovery profile.
- The core UDP demultiplexer still reserves peer port 5353 for its built-in
  mDNS route, even when that mode is disabled. These receiving fixtures use
  ephemeral ICE peer ports; arbitrary peer-port qualification needs that
  separate transport-context audit.
- E3 media tracks/transceivers, capture permissions, decoded frames through the
  shared wgpu device, real A/V interop and device-loss handling remain open.
- Application carriers still need real admission, binding, backpressure, refusal
  and reconnect receiving tests. Registry publication waits for maintained fixes.

## Findings (2026-11-04)

- The core already distinguishes `Candidate::address()` from `addr()` and
  `set_ip()` preserves the signaled address. A small host-resolved insertion
  seam can reuse this rather than rewrite candidates at the web boundary.
- `rtc-mdns` includes a continuous client/responder, but its current ICE route
  assumes a dedicated port-5353 socket and has no host-owned request quota.
  Genet currently disables it and explicitly refuses all `.local` candidates.
  The existing native DNS codec and leases fit the bounded one-shot profile.

## Progress

- 2026-11-04: rechecked clean Genet/Murm/core source and archived E2s whole;
  opened remote-resolution scope and fresh evidence before product edits.

- 2026-11-04: the first full 44-test native run passed. A subsequent relay audit
  found triggered-check/reply paths outside normal pair creation; added central
  pairing and STUN-send guards and requalified core ICE on working Rust/MSRV.
- Chrome diagnostic preflight connected through peer-reflexive checks but had
  zero successful mDNS resolutions, correctly failing acceptance. Chromium's
  responder sends multicast responses. Added request-owned receive-only group
  membership, standard zero-ID/questionless/cache-flush parsing and a real
  multicast response regression. No local advertisement or cache is introduced.

- 2026-11-04: final provider `b76891ed04ac96cfc4789e01697d94044adec8f8`
  pins core `ab0f9d1d531299c30986e415bdcf6c69ee94510f`. Thirty-three ordinary
  and three controlled coturn checks pass per supplier toolchain, strict Clippy
  and host-only graph pass, and Genet's final 44-test native suite passes. The
  multicast Chrome diagnostic resolves untouched browser names and passes.
  The full committed-source headed matrix and WPT comparison remain to run.

- 2026-11-04: provider `1e44d5957b6f4873ba7e645670a1d1a5c31a7f76`
  accepts Firefox's questionless multicast replies with the exact pending query
  ID as well as Chromium's zero-ID replies; legacy unicast stays strict. Both
  supplier toolchains pass 33 ordinary and three coturn checks. Genet
  `48000356a4a95d0c6f21599817e09d1678803ce3` passes all 44 native checks,
  strict adapter Clippy, Ortet's scripted Vano check and repository hooks. The
  same seven WPT identity/status maps remain unchanged at 93/127 per engine.
- Windows' UDP owner table omits some shared port-5353 binds, reproduced with an
  independent live socket. The headed recorder now distinguishes observable
  endpoint rebinds from pending multicast-ticket cancellation; provider tests
  also exercise actual multicast reception and socket/lease release at timeout.
  Failed diagnostics and the deliberately interrupted old-cohort matrix remain
  in the new packet. The fresh final-cohort 48-case matrix is still running.

- 2026-11-04: actual multicast destination/interface metadata is now required
  before bypassing the unicast subnet check, as RFC 6762 section 11 specifies.
  Reuse `socket-pktinfo` 0.4.1 and `network-interface` 2.0.5; preserve the provider's
  unsafe-code prohibition instead of adding local platform FFI. A controlled
  overlay-subnet regression rejects the off-subnet unicast, rejects unsolicited
  unicast at shared 5353, and accepts the same source's actual multicast reply.
- The core also discarded a previously resolved mDNS route while preparing fresh
  restart checks, by reapplying unresolved-name policy. Core
  `c0e8e02dbbc0a4f0a9982842b351f244fae3a01f` preserves its resolved endpoint
  without inheriting validity; 61 ICE checks pass on working Rust and MSRV.
- Evidence allocation is now 244 KB Genet and 6 KB new supplier entries, retaining
  the same 250 KB total. This preserves failed runs and permits one final full
  matrix; later diagnostic probes retain only compact counters, without full
  source-identity maps or raw captures. Earlier packet headers retain the
  original allocation as historical evidence.

- 2026-11-04: later Firefox probes still time out with no successful native name
  resolution, including independent packet-info observation. Selected-interface
  outgoing queries are observed; the latest sampler sees no response packets.
  Two earlier passive-observer probes did receive Firefox replies. Cause remains
  unresolved, and firewall rules/configuration were only read. Both standards
  corrections above retain their own deterministic regressions.
- The final 48-case receiving profile explicitly selects
  `--mdns-alias-browser firefox`: controlled UUID aliases map the local Firefox
  test sockets to the selected IPv4 host, while Chrome names stay untouched.
  Record controlled and native names separately and require controlled answers
  on every Firefox direct/STUN case. Default recorder behavior still preserves
  all native browser names for future advertisement diagnostics.

- Controlled Firefox aliasing now verifies selected IPv4 socket ownership in
  the marked browser process tree per ICE generation. It leaves IPv6/unknown
  sockets unchanged, so the fixture cannot invent an IPv4 route for an
  obfuscated IPv6-only candidate. This profile reproduces the reliable/binary
  restart gap with three successful controlled resolutions.
- Core `23d65c92bbde587f921fe189061f177fcde6ed92` separates old consent
  authority from the retired data path after early nomination. The old exact
  route/password can answer consent until its existing deadline or a request
  authenticated with the new local password proves mutual credentials. It
  cannot revive the old data route, populate fresh checks or extend the grant.
  Sixty-two ICE tests pass on working Rust/MSRV; receiving qualification follows.
  This continues the restart-consent requirements of RFC 7675 section 5.1.

- Core `896ae75de1c2673e98a7ebbf7058b2733786276e` also delays full-peer
  restart nomination until a new-password authenticated peer check proves
  mutual credentials. The headed diagnostic observed old inbound and new
  outbound data routes before answer delivery, with SCTP acknowledgments
  stranded on the new socket. Signaled ICE-lite peers bypass this wait; they
  do not initiate checks. The retained route keeps its existing deadline.
  The readiness regression fails with the guard disabled and passes restored;
  63 ICE checks pass on working Rust/MSRV, plus the remote-lite transport test.

- Provider `bd8b6659a29c5e52ebe09b028ec3ef5a455d7e7f` receives that core.
  All 34 ordinary and three controlled coturn checks pass on each supplier
  toolchain; strict Clippy and host-only graph checks pass. Both controlled
  Firefox direct preflights now pass, with opposite offerers and close/navigation
  cleanup, three resolutions each and all eight live restart probes. The fresh
  full matrix follows a committed-source freeze; Firefox native advertisements
  remain open independently of these receiving checks.
