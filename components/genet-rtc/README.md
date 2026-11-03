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
`3e03d99e5226952faf547b35b07c30ca2fdbaaa4`, based on upstream
`70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38`. It retains the upstream DCEP fix
(#280) and fixes the bounded restart data path and continuing DTLS answer role.
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
peer-close events. DNS, interface discovery, mDNS, TCP/TLS, server replacement
and multiple-interface routing remain open host work. A remote
candidate is not authorization to ignore the consumer's network policy.

After applying a new local ICE generation, republish retained local candidates
and call `restart_gathering()`. It restarts bounded STUN transactions while
retaining TURN allocations, permissions and refresh on the same socket. The
core retains only the previous nominated data route during negotiation, outside
the fresh candidate/check tables, until replacement, failure or a bounded
deadline. Old consent requests are authenticated and answered only on that
retained route; they cannot populate fresh checks or extend its deadline.
This supports fixed-server ICE restart; it does not rebind interfaces
or replace configured servers. Genet owns sticky requests, event ordering and
candidate generation validation at its web boundary.

The core defaults to querying mDNS, which needs a separate multicast route.
For this single-socket adapter, build its SettingEngine with
`with_multicast_dns_mode(MulticastDnsMode::Disabled)` until the host supplies that
route, and explicitly handle unresolved `.local` remote candidates. Route
refusals retain their `TransportContext` for host diagnostics, without packet
contents or credentials. Genet applies this policy in its native factory.

Genet owns JS objects, promises, events, origin/permission policy, navigation
lifetime and WPT. Application consumers own message framing, invitations and
admission. This crate imposes no one-channel, ordered-only or Graphshell
profile. It does not depend on Genet, Personae, Notochord, Luggage, storage,
`mere-transport` or the conversation service.

Canonical receiving and validation plan:
`genet/design_docs/2026-11-01_webrtc_conformance_plan.md`, E1.
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
cargo test --manifest-path wire/Cargo.toml -p murm-rtc --features udp --locked -j 1
cargo +1.97.1 test --manifest-path wire/Cargo.toml -p murm-rtc --features udp --locked -j 1
```

Controlled coturn qualification: run `tests/coturn.sh` in WSL, then set its
printed numeric endpoint as `MURM_RTC_TEST_SERVER`. The fixture contains only
local test credentials and must be stopped after the run. Execute
`cargo test -p murm-rtc --features udp --test turn -- --ignored --test-threads=1`
from `wire/`. These checks cover STUN, relay-only encrypted fragmentation after
allocation refresh, authentication failure and socket release. They do not
qualify public deployment, headed interop, cross-machine routing or packet loss.
