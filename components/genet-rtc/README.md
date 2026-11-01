# murm-rtc

The shared, pinned `rtc` protocol API and an optional caller-driven native UDP
adapter. This leaf is independent of Murm's radio/native peer workspaces and
of `murm-wire`'s HTTP/QUIC dependencies.

Use `murm_rtc::rtc` for the Sans-I/O connection. `crypto-ring` is the explicit
default provider; `crypto-aws-lc-rs` is an alternative and neither is a claim of
a C/assembly-free build. With both disabled, the caller supplies a provider.
Select `udp` for `murm_rtc::udp::UdpPeer`: bind a concrete local address,
choose I/O budgets, configure the peer through `core_mut()`, and poll `next`
from the host's own event loop. Dropping the peer releases the socket; it
spawns no tasks. `close` followed by polling flushes protocol close packets.
`next_with_interest` lets a full data consumer keep servicing protocol I/O,
timers and media while leaving data-channel reads under SCTP backpressure.

The engine is pinned to upstream commit
`70901c3df6ad92dcb87d97af3d1e2fa8a2e2ae38`, which fixes DCEP reliability on
accepted channels (upstream #280). The published 0.21.1 release failed our
real-packet partial-reliability check; the frozen E0 comparison still describes
that release. This is an immutable upstream source dependency, not a local
registry edit or a root-only Cargo patch that consumers would lose.

The UDP adapter registers host candidates only. Interface discovery, STUN/TURN
allocation, mDNS, TCP/TLS and multiple-interface routing remain additional
host integration work. No remote candidate is an authorization to ignore the
consumer's network policy.

Genet owns JS objects, promises, events, origin/permission policy, navigation
lifetime and WPT. Application consumers own message framing, invitations and
admission. This crate imposes no one-channel, ordered-only or Graphshell
profile. It does not depend on Genet, Personae, Notochord, Luggage, storage,
`mere-transport` or the conversation service.

Canonical receiving and validation plan:
`genet/design_docs/2026-11-01_webrtc_conformance_plan.md`, E1.
Supplier integration: `murm/design_docs/2026-10-13_murm_family_move_plan.md`,
WebRTC provider section. Native UDP tests are not headed-browser or WPT
qualification.

E1a passes four native tests on working Rust 1.99.1 and MSRV 1.97.1, plus
strict Clippy and the core-only compile gate. The packet test pauses data
delivery while reverse traffic continues, then receives every held message.
It does not measure a saturated receive window or induced packet loss.
Receipts: `testing/webrtc-evaluation/receipts/provider-qualification.json`
and `provider-boundaries.json`; failed attempts remain beside them. The
wire workspace defaults still select only `murm-wire`, so ordinary wire
builds do not acquire RTC. Run this package explicitly:

```powershell
$env:CARGO_TARGET_DIR = 'C:\t\cargo-targets\retinue'
$env:CARGO_BUILD_BUILD_DIR = 'C:\t\cargo-targets\retinue\build'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
cargo test --manifest-path wire/Cargo.toml -p murm-rtc --features udp --locked -j 1
cargo +1.97.1 test --manifest-path wire/Cargo.toml -p murm-rtc --features udp --locked -j 1
```
