# WebRTC conformance continuation

**Status (2026-11-04): E2d qualified for explicit-port STUN and UDP TURN hostnames in the controlled native/headed environment.** The
[previous plan](archive_docs/2026-11-03/2026-11-01_webrtc_conformance_plan.md)
is archived whole. This successor retains its open work and qualification limits.

## Ownership and qualified baseline

Genet owns Window/WebIDL behavior, promises, events, permissions, per-document
lifetime and WPT. Its native adapter uses Murm's independent `murm-rtc` wire
leaf. Murm owns generic packet/timer driving; `rtc` owns ICE/DTLS/SCTP. Host
I/O and network placement remain caller-owned. Application framing, admission,
invitations and identity remain above the provider. Carrier adoption is owned
by `murm/design_docs/2026-11-02_murm_family_move_plan.md` and Mere's carrier plan.

E0/E1a/E1c, E2h and fixed-server E2r are qualified at their recorded boundaries.
The [closed packet](archive_docs/receipts/2026-11-01_webrtc_conformance_plan_receipts.md)
contains selected unmodified WPT maps, native Boa/Vano checks, controlled coturn
and headed Windows Chrome/Firefox data-channel receipts. E2r's receiving source
is `69c926551546fe5d7cdd4efaeb6b89e18e045adc`, provider
`2ff146629e17ce8e77fc56ffa26f7b13b8b8f83a`, core
`479995928668d0db1f8cfb2aa26f818b9f03919f`. That source passed 48 headed cases,
96 completed restarts and 48 interrupted-restart cleanups, with unchanged
interfaces and fixed numeric UDP servers. These counts do not qualify public
NAT traversal, browser-guest execution, media or application-carrier replacement.
The old packet is frozen; fresh results use the external packet below.

## E2d. Nonblocking hostname ICE-server resolution

E2d provides address resolution for `stun:` and UDP `turn:` server URLs with an
explicit port. Numeric addresses retain their current path and default ports.
RFC 5928 section 3 distinguishes A/AAAA address lookup with an explicit port
from SRV/NAPTR discovery for portless hostname URLs; the latter remains a named
follow-up. Unresolved `.local` peer candidates, TCP/TLS TURN and changed
`setConfiguration().iceServers` remain separate capabilities.

The shared host layer supplies a resolver contract and an optional native
caller-polled DNS implementation. It uses established DNS message parsing;
it creates no executor or worker thread. The host can supply its own resolver,
name servers and lookup/resource budgets. System name-server configuration
does not authorize a hardcoded public resolver. Numeric-only peers need no
DNS socket. DNS I/O is distinct from the peer's concrete RTC socket and exists
only while its request is pending.

Gathering remains pending until each configured URL succeeds or exhausts its
bounded lookup/address attempts. Select addresses compatible with the concrete
RTC bind, deduplicate and cap them, then try alternatives within the same
gathering deadline. Keep the original credential-free URL in local metadata
and sanitized errors. Relay-only continues to suppress direct candidates and
peer traffic; resolving the configured TURN server does not resolve remote
candidate names. Failed lookup must not synthesize a usable route.

Restart cancels pending lookups and discards late old-generation results.
STUN gets fresh discovery; existing TURN allocations and their selected
server endpoint stay on the same socket. Pending/failed TURN discovery can
retry, without creating duplicate successful allocations. Close, drop and
navigation cancel lookup ownership immediately. The adapter must keep other
peers, timers and the document event loop responsive while resolution waits.

Done conditions:

- Resolver/provider tests cover A/AAAA, aliases, bounded name-server and
  address fallback, NXDOMAIN, malformed/unrelated replies, timeouts, limits,
  cancellation and truncated-response handling. Exercise the host-only graph
  and working Rust/MSRV; strict all-target provider Clippy passes.
- Native Boa/Vano tests exercise real controlled DNS with hostname STUN and
  forced TURN, original URL/privacy observations, failure/completion ordering,
  restart, close/navigation while pending, and unaffected numeric controls.
- Installed headed Chrome/Firefox exchange verified text/binary data against
  both engines, both offerer roles and both cleanup modes. Genet uses hostname
  ICE URLs through controlled DNS; browser peers may use the equivalent numeric
  endpoint. No machine DNS/hosts configuration change is needed.
- Freeze the existing seven selected WPT files and helpers before product
  changes. Retain exact case names/statuses and media-dependent failures; after
  results lose no prior passes and retain both data-channel restart assertions
  and the closed-peer no-op. Ortet's scripted host compiles with the capability.
- Retain exact source/lock/binary identities, bounded failures and fixture
  shutdown. Verify owned sockets/processes and test-origin permissions are
  released/restored. Preserve concurrent source and prior receipts.

Receipts: [current packet](../../../testing/genet/design_docs/archive_docs/receipts/2026-11-03_webrtc_conformance_plan_receipts.md)
under `Code/testing/genet/`. Total new evidence across Genet and Murm stays below
**250 KB**: at most **220 KB** here and **30 KB** of provider evidence in Murm's
existing family receipt. No raw SDP, generated credentials, packet captures or
full compiler logs are retained. Use exact compact deltas and bounded case rows.
Reuse `C:\t\cargo-targets\genet` and `C:\t\cargo-targets\retinue`, including
their existing `build` directories, debug=0/jobs=1. No new isolated target/home
or worktree is planned. Prior marked caches remain subject to their recorded
automatic-approval deletion rejection; this slice does not retry that cleanup.

The qualified headed command was
`node support/ci/webrtc_interop.mjs --server 172.28.40.199:34790 --restart both --dns controlled`
from Genet, with `CARGO_TARGET_DIR=C:\t\cargo-targets\genet` and
`CARGO_BUILD_BUILD_DIR=C:\t\cargo-targets\genet\build` explicitly set.
The endpoint belongs to that stopped fixture. A later run must use its own
controlled endpoint and a fresh `--receipt` packet; the completed E2d packet
and predecessor stay frozen. `node --test support/ci/webrtc_receipts.test.mjs`
checks journal resumption independently of browsers.

## Remaining phases and acceptance boundaries

- **E2 data-channel surface:** extend named WPT groups only against frozen case
  identities. Keep native, headed, browser-guest and public-network evidence
  distinct. Continue transport/statistics/privacy and renegotiation coverage;
  passing data transfer alone does not establish each web-visible API contract.
- **ICE completion:** the retained-path restart repair schedules a new check
  after an authenticated request revives a failed controlling pair. Complete
  triggered-check scheduling and requests arriving before remote credentials
  need their own RFC 8445 section 7.3 gate.
- **Names and routing:** SRV/NAPTR server discovery, mDNS peer candidates,
  interface discovery/rebinding, multiple interfaces and server replacement
  require their own ownership and lifecycle tests. Preserve relay-only privacy.
- **TURN transports:** TCP/TLS TURN needs actual transport fallback, trust,
  cancellation and socket-release receipts. UDP acceptance supplies no TCP/TLS
  or public deployment evidence.
- **E3 media:** bind RTP sender/receiver/transceiver and MediaStream/Track,
  using existing media/player, codec/device and permission contracts. Decoded
  video uses the host's wgpu device. Qualify media WPT and real A/V interop,
  device loss and capture cancellation separately; application call policy
  remains outside the engine. Broader stats, encoded transforms and capture
  suites retain their own named case inventories.
- **Application carrier:** receive the independent provider only after its
  actual consumer passes admission, binding, refusal, backpressure and reconnect.
  Existing str0m receipts do not qualify a new provider. Registry publication
  waits for a release containing the pinned DCEP and restart repairs.

## Findings (2026-11-03)

`genet-webrtc::discovery` now separates numeric endpoints, explicit-port
hostnames and unsupported attempts. `murm-rtc::resolve` supplies owned lookup
tickets, and discovery shares the gathering deadline across returned addresses.
The optional native DNS feature uses Hickory's maintained wire codec with
caller-polled nonblocking I/O; no runtime or worker is started. Genet explicitly
refuses `.local` remote candidates and disables undriven core mDNS queries.
Murm's HTTP connector delegates DNS to Hyper's runtime-specific connector;
Errand is a smolweb transport library, not a reusable executor or DNS provider.
The independent host path must retain its runtime-free boundary.

## Progress

- 2026-11-04: E2d receiving source
  `d9f402446c7651ddad37014799fdb13c65c9f243`, provider
  `c018fee13f417867cd3bbfe3ebca167257e34dc0`, core
  `479995928668d0db1f8cfb2aa26f818b9f03919f` qualify the bounded slice.
  Provider: 20 ordinary checks on Rust 1.99.1 and MSRV 1.97.1, three controlled
  coturn checks, strict all-target Clippy and host-only compile pass; its
  production DNS graph has no Tokio. Genet: 24 ordinary and six coturn checks,
  strict adapter/example Clippy with `--no-deps`, required commit checks and
  Ortet scripted-Vano all-target compile pass. Broad strict Clippy retains the
  unrelated paint-type findings recorded in the packet.
- The unchanged seven-file WPT comparison retains every case/status on both
  engines: 93/127 selected assertions pass per engine, including both data-channel
  restart assertions and the closed-peer restart no-op. All 34 existing failures
  per engine require the unbound media surface. This is no full-suite WPT claim.
- Headed run `22490c35-5645-4240-976b-b02980ab5d3e` passes all 48 cases on
  Windows Chrome 156.0.8078.80 and Firefox 159.0.1, against Boa/Vano, both
  offerers and close/navigation: 96 completed restarts and 48 interrupted
  cleanups. All 16 STUN interruptions cancel pending DNS; all 64 observed
  RTC/DNS sockets are released and rebound. Hostname URL/signaling privacy and
  retained TURN allocations pass. Controlled DNS answered 64 A queries; the
  native resolver's separate wire tests cover AAAA. Headed routing is IPv4 on
  the fixed interface, with equivalent numeric endpoints in the browsers.
- Immediate resolver refusal initially let completion overtake queued 701
  events. The provider repair and both-engine regression now pass, preserving
  the failed checks in the packet. A journal envelope field-shadowing bug was
  found during the run; its start record is corrected append-only afterward,
  and the new resume/identity regression passes. Acceptance assertions and the
  native binary stayed unchanged throughout the headed run.
- Owned browsers, native peers, DNS/front-end sockets and foreground coturn
  are stopped. Firefox's fixture network grants are absent. Earlier approval
  review still prevents deleting the marked browser profiles and old RTC MSRV
  Cargo home; no new home, worktree or isolated target was created. The current
  packet retains exact pins, lock deltas, binaries, failures and shutdown audit.
- 2026-11-03: archived the completed predecessor whole and opened E2d under
  decision D3. The previous packet remains frozen at its original path.

## Decisions

- **webrtc_conformance D3 (2026-11-03, Mark):** "Alright, proceed!" Continue
  the proposed nonblocking STUN/TURN hostname-resolution slice, with bounded
  fallback, cancellation/lifetime and native/headed regression tests. mDNS and
  TCP/TLS TURN retain separate gates. D1/D2 and their original wording remain
  in the archived predecessor.
  Links: area web-platform · plans E2d E2 E3 · summary Nonblocking ICE server names
