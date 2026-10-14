# Netfetcher wire adoption

**Status:** Qualified for main integration (2026-10-14). The generic HTTP/1-2-3
engine and raw WebSocket connector are published in Murm; Genet's adapter and
policy/host checks pass through the immutable Git supplier. Mere host adoption
is qualified before publication and receives this Genet commit next.

Canonical migration authority:
`murm/design_docs/2026-10-13_murm_family_move_plan.md`, MOVE5 and R3. This
receiving plan owns Genet's adapter and policy validation, not a second wire
specification.

## Phases and done conditions

1. Separate protocol carriage from the browser contract. `murm-wire` uses
   protocol-level `http` values and raw streams, with no Genet/Netfetcher
   dependency. Genet keeps `Method`, `Transport`, `FetchContext`, Fetch and
   WebSocket APIs, the package name/version and existing default features.
   Done when the adapter delegates its single hop without a shipping duplicate
   connection engine and existing policy cases pass.
2. Preserve host selection. The default process-wide pool remains compatible;
   explicit engines accept separate TLS trust and pool ownership. WebSocket
   HSTS, mixed-content, blocked-port and CSP refusals run before the connector;
   request/response cookie processing stays in Genet. Done when engine receipts
   prove raw HTTP/1-2-3/WebSocket carriage, no hidden web policy, independent
   pools/trust, and feature-disabled Fetch has no wire dependency.
3. Publish coherent source pins. The history-preserving Murm merge retains the
   original Genet source commit as an ancestor. Genet and Mere host checks pass
   before supplier publication; final checks use immutable Git suppliers with
   no cross-worktree path override. Done when source/metadata receipts and
   published dependency identities agree. Registry release remains separate.

## Findings (2026-10-14)

The import source is active Genet commit
`1ec5d08d16e78f936e9ab366af9f36b7d6abc044`, not the archived standalone
Netfetcher clone. The extracted files are `components/netfetcher/src/transport`
and `src/h3_client.rs`; raw WebSocket connection establishment separates from
`src/websocket.rs` while its browser checks and public session vocabulary remain.
`transport/hyper.rs` now adapts Genet's `Method`/response types to the one-hop
engine. The inherited invalid custom-method fallback remains in that adapter;
the protocol engine accepts `http::Method` directly. HTTP/3 still buffers its
response, and default pooling remains process-wide. Neither preservation proves
per-persona partitioning; an explicit host engine has its own configuration and
pool receipt. Native protocol framing can add required framing fields and
normalize field-name order; it does not add credential or content policy.

## Progress (2026-10-14)

The policy library's 83 cases pass through the extracted engine on the working
compiler and MSRV. The independent engine's HTTP/1, HTTP/2 over TLS, HTTP/3
and raw WebSocket tests pass on both compilers. The HTTP/2 fixture verifies one
connection per explicit pool and refusal of a self-signed peer outside the
supplied trust roots. Mere's actual crawler/Fetch host passes its one-hop,
redirect, no-cookie, byte-budget and ranged/session-cookie gates. Genet WPT's
actual JavaScript Fetch host passes after excluding the optional WebGL fixture
from its CPU dependency cone. Final immutable-Git receiving checks follow
supplier publication.

The original broad WPT command reached three existing macOS compositor compile
errors (missing CALayer `errors`/`iosurface` modules and the wgpu 30 Metal import
argument). Its only compositor edge came through the unconditional WebGL dev
fixture. That fixture is now behind `webgl-tests`, explicitly selected for GPU conformance; the existing `image-decode` default
remains unchanged. The focused Fetch host command uses `--no-default-features
--features netfetch`. The existing GPU fixture and its cases remain intact, but running them now
requires `--features webgl-tests`; ordinary runner builds and CPU network gates
keep that dependency outside their cone. No compositor repair or GPU acceptance
is claimed.

Final Git receiving checks pass with `murm-wire` at
`80d9959000fcd913634fd223538dfdbbec1f5e4f`: 83 policy cases on each Rust
compiler, the JavaScript Fetch integration case, and four production WPT net
worker cases (including real initial-document headers, redirect identity and
BOM decoding). Metadata selects one wire supplier, no engine back-edge into
Genet/Netfetcher and no foreign worktree Git manifest. Source hashes and gates:
`design_docs/receipts/2026-10-14_netfetcher_wire/qualification.json`.
