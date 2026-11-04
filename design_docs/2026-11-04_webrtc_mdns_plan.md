# WebRTC mDNS candidates

**Status (2026-11-04): remote receiving and the socket-free local responder qualified; native registration and browser privacy adoption open.** The completed E2s plan is
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
collision. The earlier policy rejection of profile/home deletion remains in force;
do not retry or bypass it. The reviewer responsible was not established.

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

## Findings and corrections

- The core already separates the signaled candidate address from its numeric
  endpoint. The trusted insertion API retains names and IDs, and central pair
  and STUN-send guards exclude local TURN routes for remote mDNS candidates,
  including triggered requests and either candidate insertion order.
- Browser responders use multicast as well as legacy unicast. Validate actual
  multicast destination/interface metadata with `socket-pktinfo` 0.4.1 and
  `network-interface` 2.0.5. The provider retains `forbid(unsafe_code)` and does
  not add local platform FFI. Unicast keeps its additional source-subnet check.
  A real socket regression covers multicast sources on overlay subnets.
- Chromium uses zero-ID questionless/cache-flush responses; Firefox can echo
  the pending query ID without a question. Permit those multicast forms while
  retaining strict legacy unicast ID/question validation and bounded parsing.
- Restart preparation must retain an already resolved route without reapplying
  unresolved-name policy or inheriting old validity. Early nomination also
  stranded SCTP acknowledgments on a new Firefox socket while incoming data
  still used the old one. The controlling agent now waits for a full peer's
  current-password authenticated check before nominating away from the retained
  route. ICE-lite peers bypass that wait because they do not initiate checks.
  Old consent authority and the retained route keep their bounded deadlines.
- Windows' UDP owner table can omit shared port-5353 binds, independently
  reproduced with a live socket. Headed checks prove pending-ticket cancellation
  and observable endpoint rebinds; provider tests prove actual multicast
  membership/socket release. Do not count unobserved shared binds as exclusive
  endpoint rebinds.
- Firefox's own responder remains unresolved on this receiving profile. The
  latest independent packet-info sampler saw outgoing selected-interface
  questions and no replies; two earlier observer probes did see replies.
  Existing UDP firewall rules were only read. No firewall, hosts, DNS or browser
  WebRTC preferences were changed, and no firewall cause is asserted.
- The explicit Firefox fixture aliases only ports observed on IPv4 sockets in
  its marked process tree, refreshed per ICE generation. IPv6/unknown sockets
  and Chrome's browser names stay untouched. Controlled and original names are
  counted separately. This cannot qualify Firefox's own name advertisements.

## Qualification (2026-11-04)

Receiving source `993fb879f3139bcd70048e277989f698c41caffc` pins provider
`bd8b6659a29c5e52ebe09b028ec3ef5a455d7e7f`, which pins core
`896ae75de1c2673e98a7ebbf7058b2733786276e`. Earlier failed and incomplete
cohorts remain in the append-only packet; their Chrome passes are not promoted
into this cohort.

- Core: 63 ICE checks pass on working Rust 1.99.1 and MSRV 1.97.1. The nomination
  regression fails with its readiness guard disabled and passes restored.
  Remote-lite transport and public resolved-candidate API checks pass on MSRV;
  strict core/ICE library Clippy passes.
- Supplier: 34 ordinary and three controlled coturn checks pass per toolchain,
  with strict all-target Clippy and host-only DNS compile/normal-build graph
  checks excluding Tokio.
- Genet: all 44 native browser checks pass across Boa/Vano, including eight
  controlled coturn checks. Six Node helper checks, strict adapter Clippy,
  Ortet's scripted Vano all-target check and required repository hooks pass.
- WPT: the same seven files/helpers and all fourteen engine/file assertion maps
  are unchanged. Both engines retain 93/127 passing assertions and the same
  34 media-dependent failures; imported tests were not edited.
- Headed: all 48 Chrome/Firefox cases pass across both engines, routes,
  offerers and close/navigation cleanup: 96 complete restarts, 384 live restart
  probes and 48 interrupted cleanups. Chrome names stay untouched; Firefox uses
  controlled aliases for observed owned IPv4 sockets. Each browser cohort has
  48 successful resolutions; Firefox's own native responder remains unqualified.
- Cleanup: 32 pending mDNS contexts and 16 pending server-DNS contexts cancel.
  All 96 observable exclusive RTC/DNS endpoints rebind, plus three observed
  shared port-5353 endpoints. Other shared binds omitted by the Windows owner
  table are covered by provider socket/drop tests, not invented endpoint counts.
- Restoration: owned native/browser/controller processes and coturn are gone;
  the service is inactive/disabled and Firefox network-permission rows are zero.
  Earlier cleanup commands were rejected as blocked by policy; attribution to
  automatic review was unverified. Retained profiles belong to `genet-rtc-interop`; the
  old `C:/t/cargo-homes/rtc-restart` cache belongs to `genet-rtc-ice-restart`.
  Exact paths and bytes are in the receipt. No isolated target/home/worktree was
  created for this slice.

The [external E2m packet](../../../testing/genet/design_docs/archive_docs/receipts/2026-11-04_webrtc_mdns_plan_receipts.md)
is closed at 232,544 bytes, SHA-256
`80cdf925ad10132aea951c5e50a26b53ad095cd74b38d042c237be45fa12662f`.
It retains exact pins, source/lock/binary hashes, failed diagnostics and the
durable headed case journal. The closed E2s and E2d packets retain their original
hashes. New E2m supplier entries stay below 6 KB and total new evidence below
250 KB; the larger historical family packet is accounted separately.


## Unattended test setup (2026-11-04)

The next local mDNS advertisement slice is authorized. Before its network runs,
use a stable test executable and inspect application firewall rules. Cargo's
hashed integration-test paths otherwise change after dependency revisions.
The native wrapper builds the real Cargo test harness, checks the copy hash,
serializes execution and retains its generated ownership marker at
`C:/t/cargo-targets/genet/rtc-interop/native-browser-tests-owner.json`.
It never substitutes a different implementation or treats enumeration as a pass.

```powershell
# Build/copy and enumerate; these modes open no RTC listeners.
python -B support/ci/webrtc_native.py --list
# Read-only audit of current programs and network profiles.
pwsh -NoProfile -File support/ci/webrtc_permissions.ps1 -Kind All
# Preview the one missing native rule. This changes nothing.
pwsh -NoProfile -File support/ci/webrtc_permissions.ps1 -Kind Native -Install -WhatIf
# Run this once in an elevated PowerShell while present, then audit again.
pwsh -NoProfile -File support/ci/webrtc_permissions.ps1 -Kind Native -Install
# Ordinary tests, or add --server NUMERIC_IPV4:PORT for controlled coturn tests.
python -B support/ci/webrtc_native.py
```

`-FirewallProfile` selects explicit profiles; by default the audit reads current
network categories. Installation adds only missing exact-program UDP rules for
the selected profiles and local-subnet/loopback peers. It neither opens UAC nor
removes existing rules. Existing explicit blocks require separate review. Run test
batches from an ordinary shell; elevation is only for firewall installation. The
audit checks a supported application-rule shape, not packet delivery or every
managed-policy, SmartScreen, antivirus or OS permission boundary. An unattended
native run exits before tests when the rule is missing/unreadable. Its
`--timeout-seconds` bounds execution after compilation (default 600 seconds). The headed
runner also audits its actual host/Node paths before any listener/browser starts.
Firefox test-origin network permissions continue to be granted and restored
through BiDi. The unresolved Firefox responder remains a separate receiving gate.

Current Codex configuration is `approval_policy = "never"`, reviewer `user`, and
`danger-full-access`. Changing that does not grant Windows administrator rights.
Historical frozen packets retain their original auto-review attribution; only
an observed `blocked by policy` command rejection was established. Future runner
messages use the narrower description. Old cleanup is not retried.

Operational validation retains at most **12 KB** in the fresh external
`Code/testing/genet/design_docs/archive_docs/receipts/2026-11-04_webrtc_unattended_receipt.json`.
The closed E2m/E2s/E2d packets remain byte-for-byte unchanged. The prepared native
binary is reusable build output owned by `genet-rtc-native-tests`, retained to
keep its firewall path stable. This setup gate does not claim a new headed or
native network qualification until the missing rule is installed and tests run.

Validation: six runner boundary checks and six existing Node helper checks pass.
The exact copied Cargo artifact enumerates all 44 native tests. Headed
application rules are present; the new stable native path has no rule yet.
The actual native wrapper exits 2 before tests, and `-Install -WhatIf` leaves
firewall rules unchanged. Administrative installation and a network run from
that path remain pending the one-time Windows setup. Prior conformance results
keep their exact original source identities.

Compatibility correction: the setup script supports Windows PowerShell 5.1 and
PowerShell 7. Its original `$IsWindows` guard was undefined under strict mode
in 5.1; the platform check now uses the shared .NET API. Both runtimes pass the
actual native `-Install -WhatIf` preview without changing firewall rules. A
Windows PowerShell 5.1 subprocess regression is part of the runner test suite.
The fresh compatibility receipt shares the operational 12 KB retained cap.

Firewall scope correction: Windows rejects explicit IPv6 loopback `::1` in the
remote address filter. The fixture rule now uses `LocalSubnet` and IPv4
`127.0.0.0/8`, retaining the exact executable, selected profile and UDP scope.
Installation passes explicit `-ErrorAction Stop` so provider rejection cannot
continue into a post-install report. The nine runner checks now include the
actual installation scope validated on an unattached Windows `INetFwRule`,
plus a simulated nonterminating rejection against the real installation
pipeline, each under Windows PowerShell 5.1 and PowerShell 7. Neither check
adds a firewall rule. The fresh firewall-scope receipt shares the same 12 KB
operational cap. Administrative installation still needs a successful rerun;
address validation and preview alone do not establish effective rule coverage.

## E2a. Local advertisement

This is the next bounded slice under the user's continued RTC authorization.
It can advance without opening sockets while the Native application firewall
rule is installed separately. E2m receiving evidence stays closed and unchanged.

### Protocol primitive

**Qualified, 2026-11-04.** The caller-driven, bounded responder in `rtc/rtc-mdns`
reuses its existing DNS name/header/record codec. One registration owns one UUID-v4 `.local` name
and one IPv4 address. This follows the ICE candidate draft's unique-name
profile, including its recommendation to skip probing, rather than introducing
a generic DNS-SD registry. The draft remains work in progress:
[draft-ietf-mmusic-mdns-ice-candidates-03 section 3.1.1](https://datatracker.ietf.org/doc/html/draft-ietf-mmusic-mdns-ice-candidates-03#section-3.1.1).
Packet behavior follows [RFC 6762](https://www.rfc-editor.org/rfc/rfc6762.html).

Done conditions: pure tests cover announcements, multicast and requested/legacy
unicast answers, cache-flush and legacy TTL/question rules, known-answer
suppression, bounded decoding/output and pacing, conflict withdrawal, and
close cancellation followed by one goodbye. Use a caller-supplied clock and
response jitter. Keep unsupported multipacket truncated queries explicit.
No socket, timer task, recursive resolver or second packet parser is added.
Run the mDNS unit/integration suite on working Rust 1.99.1 and MSRV 1.97.1,
strict library Clippy and formatting. Additive core API qualification does not
repin the browser's supplier chain before a consumer exists.

### Native registration and browser privacy

Next, Murm's independent provider will own selected-interface, source-port-5353
I/O, registration quotas and cancellable tickets. Genet will own configurable
local-address exposure, SDP/event/statistics projections and document lifetime.
Keep the core's numeric ICE route separate from the exposed name. Registration
failure or conflict must not reveal a numeric address as a fallback. Close,
navigation, restart and drop require explicit receiving tests. These host and
browser phases remain open until implemented and qualified; neither the pure
responder nor the earlier remote receiving matrix establishes them. Native
Firefox responder acceptance remains a distinct open gate.

### Evidence and resources

E2a retains at most 20,000 bytes of new compact evidence in the external
`testing/genet/design_docs/archive_docs/receipts/2026-11-04_webrtc_mdns_advertisement_receipts.md`.
Record source identities, commands, counts and limits; retain no packet captures
or raw interface inventory. Reuse `C:\t\cargo-targets\retinue` for the core.
No isolated Cargo home, target, worktree or running fixture is required.

### Findings and progress

- 2026-11-04: `rtc-mdns/src/proto/mod.rs` already provides a basic responder;
  it lacks the registration and bounded response lifecycle needed here. Reuse
  the private codec instead of duplicating it in Murm or Genet.
- 2026-11-04: Windows PowerShell 5.1 setup compatibility is fixed independently;
  preview verification did not install or modify firewall rules.
- 2026-11-04: `rtc-mdns/src/responder.rs` and its bounded incremental decoder
  implement the protocol primitive. The complete crate suite passes on both
  Rust versions: 68 unit tests, 16 packet-only integration tests and 30 doc
  tests, with eight existing ignored doc examples. These include 26 new
  responder checks and one new public-API client round trip covering standard
  and legacy queries. Strict all-target Clippy passes on both versions after
  small existing example/test style fixes. No example executables ran and no
  sockets were opened; the existing socket-builder tests only inspect settings.
- 2026-11-04: the fresh Native read-only preflight still reports no covering
  rule on the current Public profile. Host registration and browser receiving
  remain open; the earlier supplier pins and closed E2m packet are unchanged.
