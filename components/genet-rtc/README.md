# murm-rtc

The shared, pinned `rtc` protocol API and an optional caller-driven native UDP
adapter. This leaf is independent of Murm's radio/native peer workspaces and
of `murm-wire`'s HTTP/QUIC dependencies.

Use `murm_rtc::rtc` for the Sans-I/O connection. `crypto-ring` is the explicit
default provider; `crypto-aws-lc-rs` is an alternative and neither is a claim of
a C/assembly-free build. With both disabled, the caller supplies a provider.
Select `host` for `murm_rtc::udp::UdpPeer::bind_host`: bind a concrete local
address, choose I/O budgets, configure the peer through `core_mut()`, and call
nonblocking `poll` from the host's event loop. This dependency graph contains
no Tokio. `next_deadline` exposes the protocol's next wakeup. Select `udp` for
Tokio binding and async `next`/`next_with_interest`, using the same packet and
timer driver. Dropping the peer releases the socket; it spawns no tasks.
`close` followed by polling flushes protocol close packets.
`next_with_interest` lets a full data consumer keep servicing protocol I/O,
timers and media while leaving data-channel reads under SCTP backpressure.

The engine is pinned to `merely-made/rtc` commit
`896ae75de1c2673e98a7ebbf7058b2733786276e`, based on upstream
`70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38`. It retains the upstream DCEP fix
(#280) and fixes the bounded restart data path, continuing DTLS answer role
and recovery of exhausted controlling checks after a fresh authenticated request.
It also accepts host-resolved mDNS candidates while retaining names/IDs and
refuses local TURN pairs with remote mDNS names.
The published 0.21.1 release failed our
real-packet partial-reliability check; the frozen E0 comparison still describes
that release. This is an immutable maintained source dependency, not a local
registry edit or a root-only Cargo patch that consumers would lose.

`bind_host_with_discovery` / `bind_with_discovery` additionally take resolved
UDP STUN/TURN servers and the same crypto provider as the peer's SettingEngine.
`finish_host_candidates` starts server gathering; completion waits for results
or the caller's gathering timeout. Relay-only suppresses direct candidates and
direct peer packets. TURN authentication, permissions and refresh use the pinned
protocol client. One retained protocol datagram and one socket datagram bound
pending sends, including permission waits. Close/drop attempts authenticated
allocation release without waiting. Server errors are observations, not invented
peer-close events. Interface discovery, local mDNS advertisement, TCP/TLS, server replacement
and multiple-interface routing remain open host work. A remote
candidate is not authorization to ignore the consumer's network policy.

Select `dns` for the optional native DNS resolver, without Tokio. Supply
`DiscoveryConfig::named_servers` and an `Arc<dyn Resolver>` for UDP URLs,
preserving omitted ports and transports with `NamedServer`. Explicit ports use
A/AAAA; portless STUN uses SRV, and portless TURN uses SRV for explicit UDP or
S-NAPTR when transport is omitted. NAPTR delegation and S/A terminals, SRV
priority/weight/ports, initial default fallback and explicit unavailability are
supported. SRV targets cannot be aliases. The `resolve` contract is also available with `host` for caller
implementations. `DnsConfig::system` reads local name-server configuration;
explicit endpoints and `DnsLimits` let hosts choose placement and budgets.
Each request owns nonblocking UDP, or TCP after a truncated DNS reply. Polling
and `next_deadline` drive it; drop cancels its sockets and capacity lease.
No background runtime or public fallback resolver is created. Lookup failure,
empty/wrong-family answers and exhausted address attempts yield one sanitized
701 per original URL. Discovery tries at most 32 distinct route-compatible
addresses within one gathering deadline. Numeric routes do not start DNS.
Service discovery yields one endpoint batch, pauses for the protocol attempt,
and backtracks only on failure. Success/drop releases the cursor. Configurable
query, target and delegation limits share one gathering deadline. Restart cancels
pending names, rechecks STUN and retains existing TURN clients; close/drop cancels
resolution. Local mDNS advertisement, IPv6 multicast, TURN TCP/TLS and broader allocation-error cooldown
and redirect policy remain separate capabilities. Genet's E2s receiving
qualification is tracked in its plan, independently of provider tests.

After applying a new local ICE generation, republish retained local candidates
and call `restart_gathering()`. It restarts bounded STUN transactions while
retaining TURN allocations, permissions and refresh on the same socket. The
core retains only the previous nominated data route during negotiation, outside
the fresh candidate/check tables, until replacement, failure or a bounded
deadline. Old consent requests are authenticated and answered only on that
retained route; they cannot populate fresh checks or extend its deadline.
A controlling agent retains the data route until a full peer authenticates a
check with the new local password, before nominating a replacement. Signaled
ICE-lite peers bypass that wait because they do not initiate checks. Old
consent authority is bounded independently of the replaced data route.
After new peer credentials arrive, the known nominated address is also checked
as a fresh candidate. Its prior validity and transactions are discarded.
This supports fixed-server ICE restart; it does not rebind interfaces
or replace configured servers. Genet owns sticky requests, event ordering and
candidate generation validation at its web boundary.

The core defaults to querying mDNS, which needs a separate multicast route.
For this single-socket adapter, build its SettingEngine with
`with_multicast_dns_mode(MulticastDnsMode::Disabled)` and explicitly handle
unresolved `.local` remote candidates through the optional resolver and the
core host-resolved insertion API. Route
refusals retain their `TransportContext` for host diagnostics, without packet
contents or credentials. Genet applies this policy in its native factory.

Genet owns JS objects, promises, events, origin/permission policy, navigation
lifetime and WPT. Application consumers own message framing, invitations and
admission. This crate imposes no one-channel, ordered-only or Graphshell
profile. It does not depend on Genet, Personae, Notochord, Luggage, storage,
`mere-transport` or the conversation service.

Canonical receiving and validation plan:
`genet/design_docs/2026-11-04_webrtc_mdns_plan.md`, qualified baseline, E2s and
bounded E2m remote receiving.
Supplier integration: `murm/design_docs/2026-11-02_murm_family_move_plan.md`,
RTC ICE restart section. Native UDP tests are not headed-browser or WPT
qualification.

The original five native tests cover both host polling and async waits on working Rust
1.99.1 and MSRV 1.97.1, alongside strict Clippy and the core-only compile gate. The packet test pauses data
delivery while reverse traffic continues, then receives every held message.
It does not measure a saturated receive window or induced packet loss.
E1c adds ordinary discovery demux/deadline checks and the three controlled
coturn checks below. Relay-only candidates conceal the physical related address.
Receipts: `Code/testing/murm/testing/webrtc-evaluation/receipts/provider-qualification.json`
and `provider-boundaries.json`; the host-driver addition is appended to
`design_docs/archive_docs/receipts/2026-10-13_murm_family_move_plan_receipts.md`.
Failed attempts remain in their cited locations. Genet consumes this provider
through its native host adapter; its web/API qualification is recorded in the
receiving plan. The
wire workspace defaults still select only `murm-wire`, so ordinary wire
builds do not acquire RTC. Run this package explicitly:

```powershell
$env:CARGO_TARGET_DIR = 'C:\t\cargo-targets\retinue'
$env:CARGO_BUILD_BUILD_DIR = 'C:\t\cargo-targets\retinue\build'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
cargo test --manifest-path wire/Cargo.toml -p murm-rtc --features dns,udp --locked -j 1
cargo +1.97.1 test --manifest-path wire/Cargo.toml -p murm-rtc --features dns,udp --locked -j 1
```

Controlled coturn qualification: run `tests/coturn.sh` in WSL, then set its
printed numeric endpoint as `MURM_RTC_TEST_SERVER`. The fixture contains only
local test credentials and must be stopped after the run. Execute
`cargo test -p murm-rtc --features dns,udp --test turn -- --ignored --test-threads=1`
from `wire/`. These checks cover STUN, relay-only encrypted fragmentation after
allocation refresh, authentication failure and socket release. They do not
qualify public deployment, headed interop, cross-machine routing or packet loss.

E2r passes nine ordinary tests on working Rust 1.99.1 and MSRV 1.97.1,
the three controlled coturn tests and strict all-target Clippy. Restart-specific
checks cover stale STUN transactions, two bounded relay-refusal generations,
and DTLS answer policy for continued versus changed peer identity. The pinned
core's 58 ICE tests pass on 1.97.1, including authenticated old consent confined
to the retained route and deadline, and a fresh check after an authenticated
restart request revives an exhausted controlling pair. Genet owns the established
data-channel native/WPT/headed restart receiving results in its conformance receipt.

E2d provider `c018fee13f417867cd3bbfe3ebca167257e34dc0` passes 20 ordinary
checks on working Rust/MSRV, three coturn checks, strict Clippy and host-only
compile. Tests cover real A/AAAA wire lookup, spoofed/malformed responses,
aliases, DNS TCP truncation fallback, quotas, bounded address attempts and
cancellation. Immediate resolver refusal reports 701 before gathering completes.
Genet receiving source `d9f402446c7651ddad37014799fdb13c65c9f243` passes 30
native checks, the unchanged selected WPT maps and 48 headed restart cases.
The native side uses explicit-port hostnames; headed browser peers use equivalent
numeric endpoints through the controlled IPv4 fixture. This adds no SRV/NAPTR,
mDNS, TURN TCP/TLS, public-network, browser-guest, media or carrier qualification.

E2s provider `ef2d18fadbbdca3847ad99040af14f2bc7693a47` passes 29 ordinary
checks on working Rust/MSRV and the three controlled coturn checks on each,
strict all-target Clippy, and the host-only DNS compile/graph gate without Tokio.
Nine added resolver checks cover lazy endpoint batches, S-NAPTR delegation and
backtracking, initial default fallback, explicit unavailability, SRV ports and
weighted ordering, alias refusal, IPv6, quotas/cancellation and fragmented SRV
DNS TCP fallback. Genet owns the separate portless-URL native, WPT and headed
receiving results in its current plan and external Nov 4 receipt.

E2s receiving product `ff958976f2a904d250d00ee10335696c62f749b1` passes 38
native checks. Recorder `7abebdda633bac2d91bd05b4976f2bf65ea525cb` passes all
48 headed Chrome/Firefox cases across Boa/Vano, direct/STUN/forced TURN,
offerers and close/navigation. This covers 96 completed restarts, 48 interrupted
cleanups and 16 pending-SRV cancellations with 64 RTC/DNS socket rebinds.
Selected WPT identities/statuses are unchanged at 93/127 assertions per engine,
including the same 34 media-dependent failures. Native portless URLs discover
nondefault ports through controlled DNS; browser peers use equivalent numeric
endpoints. This fixed-interface IPv4 gate does not qualify public NAT, multicast
mDNS, TURN TCP/TLS, interface/server replacement, media or application carriers.
The fixture is stopped and network grants are restored. The external Genet Nov 4
packet is closed; prior failed recorder evidence remains separate from the final
48-case matrix. Provider source stays pinned independently of these doc updates.

E2m provider `bd8b6659a29c5e52ebe09b028ec3ef5a455d7e7f` adds optional
`MdnsConfig` to `DnsResolver`. `start_mdns` is separate from recursive/server
DNS, restricted to UUID-v4 peer names and an explicitly selected IPv4 interface.
The RFC 6762 one-shot profile uses an ephemeral query port and request-owned,
receive-only port-5353 membership. Legacy unicast replies require the echoed
ID/question and an on-link source. Multicast replies require the actual group
destination and selected interface; questionless replies accept zero or the exact
pending query ID. Both paths validate source port, name/class and bounded
records, refuse aliases/ambiguous answers, and release sockets/quota on finish
or drop. No cache, background worker or local-name advertisement is created. Genet owns signaling,
statistics privacy and lifetime/generation cancellation. Its current mDNS plan
owns native/headed qualification. IPv6 multicast and local advertisement remain
separate from the core's existing continuous mDNS client/responder.

The E2m supplier passes 34 ordinary and three controlled coturn checks on each
working/MSRV toolchain, strict all-target Clippy and the host-only DNS compile
and normal/build graph without Tokio. Genet source
`993fb879f3139bcd70048e277989f698c41caffc` passes 44 native checks and 48
controlled headed cases with unchanged selected WPT maps. Chrome names stay
untouched; Firefox uses controlled aliases for observed IPv4 sockets. Firefox's
own responder, local registration/advertisement and IPv6 multicast remain open.
The external Genet E2m packet is closed after fixture and permission restoration;
its plan owns the exact receipt identity and remaining receiving boundaries.

Local IPv4 registration is available through `registration` with `dns` enabled.
Generate a private name with `candidate_name`, create an owned `Registration`
with the selected interface and prefix, and poll it from the caller's loop.
Limit concurrent registrations with the host peer quota. Drop withdraws the
record and releases its sockets; conflicts and I/O failures must not trigger a
numeric address fallback. This adapter accepts multicast queries only; direct
unicast queries and IPv6 registration remain separate work. Genet's E2a plan
owns browser-facing privacy policy and the receiving tests.
