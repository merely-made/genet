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
subnet validation, bounded DNS parsing and an owned cancellation ticket. Reuse
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
  reply ID/question/source/subnet checks, malformed/oversized data, aliases,
  multiple addresses, wrong family, TTL-zero answers, quotas, timeout and drop.
  Working Rust 1.99.1, MSRV 1.97.1, strict Clippy and host-only graph pass.
- Boa/Vano tests establish encrypted data channels through mDNS candidates,
  including SDP ingestion, original-name/statistics privacy, success-independent
  observations, relay suppression and pending-generation/lifetime cancellation.
  Keep the existing 38 native checks and controlled STUN/TURN coverage.
- Headed Chrome/Firefox exercise both engines, offerers, cleanup modes and
  direct/STUN/relay routes. Distinguish controlled aliases from actual browser
  mDNS advertisements. Require real query/response evidence and socket release;
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
Retain at most **250 KB** total new evidence: 220 KB in that Genet packet,
including core receipts, and 30 KB new E2m entries in Murm's existing family
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
- Local mDNS name advertisement and IPv6 multicast require separate ownership,
  privacy, registration/conflict, cancellation and socket qualification.
- TURN TCP/TLS and broader allocation-error cooldown/redirect/authentication
  policy remain separate from the qualified UDP discovery profile.
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
