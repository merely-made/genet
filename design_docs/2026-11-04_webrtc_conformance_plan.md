# WebRTC conformance continuation

**Status (2026-11-04): E2s UDP service discovery in progress.** The
[previous plan](archive_docs/2026-11-04/2026-11-03_webrtc_conformance_plan.md)
is archived whole, retaining decisions D1-D3 and exact earlier boundaries.

## Ownership and qualified baseline

Genet owns WebIDL, promises/events, origin policy, document lifetime and WPT.
Murm's independent `murm-rtc` leaf supplies caller-driven I/O and generic
resolution; the maintained `rtc` core owns ICE/DTLS/SCTP. Applications retain
framing, admission and identity. Keep the provider independent of Genet,
Personae, Notochord, storage and radio application semantics.

E2d qualifies explicit-port STUN/UDP-TURN hostnames: Genet receiving source
`d9f402446c7651ddad37014799fdb13c65c9f243`, provider
`c018fee13f417867cd3bbfe3ebca167257e34dc0`, core
`479995928668d0db1f8cfb2aa26f818b9f03919f`. Its closed
[packet](../../../testing/genet/design_docs/archive_docs/receipts/2026-11-03_webrtc_conformance_plan_receipts.md)
records 30 native checks and 48 headed cases: 96 completed restarts, 48
interrupted cleanups, including 16 pending-DNS cancellations. Chrome
156.0.8078.80 and Firefox 159.0.1 ran against Boa/Vano on fixed-interface IPv4;
browser peers used equivalent numeric endpoints. Resolver wire tests cover
A/AAAA. Every selected WPT case/status stayed unchanged: 93/127 assertions
per engine pass, with 34 media-dependent failures. Earlier E0/E1/E2h/E2r
remain qualified only at their archived boundaries. No public NAT,
browser-guest, media or application-carrier acceptance follows from these gates.

## E2s. Portless STUN/TURN service discovery

Preserve whether the original URL supplies a port and transport. The core's
`Url` normalizes both, so Genet must pass omission explicitly across the host
boundary. Numeric and explicit-port paths retain E2d behavior.

Implement UDP SRV discovery for portless STUN names, and RFC 5928's UDP-only
supported-transport profile for TURN: explicit `transport=udp` starts SRV;
omitting transport starts S-NAPTR. Support NAPTR order/preference,
nonterminal delegation and terminal S/A records, bounded backtracking, SRV
priority and weighted ordering, target ports, and address-family filtering.
Use default-port A/AAAA fallback only at the specified initial fallback points;
a positive SRV target `.` must not invent a usable default route. SRV targets
must not be CNAME aliases. Treat unsupported schemes/transports as refusals.

Service lookup is an owned, caller-polled candidate cursor. It yields one
bounded target/address batch, pauses while STUN/Allocate tries it, then resumes
only on failure. Stop and discard it on success. Preserve one gathering
deadline across queries and endpoint attempts; bound DNS records, queries,
branches, depth, endpoints and pending I/O. No executor or background worker.
Avoid eager resolution of every alternate before trying the first useful one.

Retain the original credential-free URL in candidate/error metadata, exclude
local metadata from signaling JSON, and keep relay-only address privacy. One
terminal 701 per URL precedes completion. Restart cancels unresolved discovery;
successful TURN allocations retain their endpoint/socket. Close/drop/navigation
release pending query ownership. No machine DNS or hosts changes.

Done conditions:

- Provider tests exercise actual controlled DNS for SRV/NAPTR traversal,
  ordering, target-port fallback, default fallback, explicit unavailability,
  unsupported records, cycles/depth/query limits, malformed/unrelated replies,
  cancellation and DNS TCP truncation; preserve E2d address controls. Working
  Rust/MSRV, host-only graph and strict all-target wire Clippy pass.
- Native Boa/Vano tests exercise real DNS through hostname STUN and forced
  TURN at nondefault discovered ports, fallback, original URL/privacy, error
  ordering and pending restart/close/navigation. Keep numeric and explicit-port
  regressions. Ortet's scripted host compiles against the published pin.
- Headed Chrome/Firefox qualify both engines, offerers, cleanup modes and
  direct/STUN/relay routes. The native side uses portless URLs through controlled
  SRV/NAPTR; browsers may use equivalent numeric endpoints. Check data/channel/
  certificate continuity, selected routes, TURN reuse and owned socket release.
- Freeze the same seven WPT files/helpers before product changes. Compare exact
  identities/statuses afterward; retain both restart assertions and closed-peer
  no-op, and expose existing media failures. No imported corpus edits.
- Retain pins, lock deltas, actual binary identities, bounded failures, fixture
  shutdown and permission restoration. Preserve concurrent WIP and prior receipts.

Sources: [RFC 5928 section 3](https://www.rfc-editor.org/rfc/rfc5928.html#section-3),
[RFC 3958](https://www.rfc-editor.org/rfc/rfc3958.html),
[RFC 2782](https://www.rfc-editor.org/rfc/rfc2782.html),
[RFC 8489 section 8](https://www.rfc-editor.org/rfc/rfc8489.html#section-8),
[RFC 7064](https://www.rfc-editor.org/rfc/rfc7064.html) and
[RFC 7065](https://www.rfc-editor.org/rfc/rfc7065.html).
This is the bounded UDP resolution slice, not full TURN transport/authentication
conformance. Broader allocation-error cooldown/redirect policy keeps its own
core/provider audit; do not imply it from DNS traversal acceptance.

## Evidence and workspace budget

Use the new external
[packet](../../../testing/genet/design_docs/archive_docs/receipts/2026-11-04_webrtc_conformance_plan_receipts.md).
Total new retained evidence across both repositories is **250 KB**: at most
**220 KB** in Genet's packet and **30 KB** of new E2s supplier evidence in Murm's
existing family receipt (which remains below 100 KB total). Keep compact
summaries and exact deltas, not raw SDP, credentials, captures or full compiler
logs. The E2d and earlier packets are frozen; new writers refuse those paths.

Reuse `C:\t\cargo-targets\genet` and `C:\t\cargo-targets\retinue` and their
`build` directories, debug=0/jobs=1. No new isolated target/home/worktree is
planned. The marked RTC browser profiles and old MSRV home retain their prior
automatic-approval deletion rejection; do not retry or bypass it.

## Remaining work

- E2 data-channel/WebIDL/statistics/renegotiation conformance, named WPT groups,
  browser-guest and public-network evidence remain separate receiving gates.
- Complete triggered ICE scheduling and requests before remote credentials
  remain a distinct RFC 8445 section 7.3 gate. Existing restart fixes do not
  qualify server replacement, interface discovery/rebinding or multi-interface I/O.
- Multicast mDNS peer candidates and TURN TCP/TLS transports retain separate
  discovery, trust, fallback, cancellation and socket-release gates.
- E3 binds RTP sender/receiver/transceiver, MediaStream/Track, capture/device
  permissions and decoded frames through the host's existing wgpu device.
  Media WPT, real A/V interop, device loss and capture cancellation are unqualified.
- Application carriers must receive the provider through actual admission,
  binding, refusal, backpressure and reconnect tests. Existing str0m receipts
  do not qualify a replacement. Registry publication waits for maintained fixes.

## Findings (2026-11-04)

The E2d host accepts only explicit-port names. `murm-rtc::resolve::Lookup`
yields IP addresses, and discovery adds one fixed URL port. Service discovery
therefore needs endpoint batches and preserved URL omission; it cannot safely
be added by silently replacing the default port. The existing DNS wire codec,
nonblocking sockets and query leases are reusable. No duplicate HTTP resolver
or application transport is needed.

## Progress

- 2026-11-04: archived the completed E2d plan whole and opened E2s; source and
  dependency state rechecked before product changes. The fresh WPT baseline is retained.
- Supplier `ef2d18fadbbdca3847ad99040af14f2bc7693a47` is published and received
  by Genet `ff958976f2a904d250d00ee10335696c62f749b1`. Provider checks pass
  29 ordinary plus three controlled coturn tests on working Rust/MSRV, strict
  Clippy and the host-only graph. Genet passes 38 native tests across Boa/Vano,
  strict adapter Clippy, Ortet scripted-host compilation and required hooks.
  The seven selected WPT files retain every baseline case/status. Headed
  service-discovery qualification is running against that committed source.
- The first headed attempt retained 17 passing cases, then stopped when its
  initial one-shot statistics capture lacked a succeeded pair. Added bounded
  initial resampling, preserving route/certificate/privacy acceptance checks
  and failure diagnostics. Three helper regressions pass; a new complete
  headed matrix must qualify the revised recorder. The first attempt remains
  in the packet and does not count toward that fresh matrix.

## Decisions

The exact earlier design rulings D1-D3 remain in the archived predecessors.
The user authorized this continuation with "Continue!"; this is an operational
continuation, not a new doctrine case.
