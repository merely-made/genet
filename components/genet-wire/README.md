# murm-wire

Protocol-level HTTP/1, HTTP/2, HTTP/3 and raw WebSocket carriage, extracted
from Genet's active Netfetcher implementation with its original source history
retained. This workspace is independent of Murm's radio and native peer stacks.

`WireRequest` contains a URL, `http::Method`, header pairs, an optional buffered
body and a caller-selected HTTP/3 hint. `Transport::send` performs one hop and
returns raw status, headers and undecoded chunks, or the original network
failure mapping (`None`). It does not follow redirects, add cookies, discover
Alt-Svc, apply CORS/CSP, decode content or choose browser credentials.
HTTP framing may add protocol-required framing headers and normalize header
field ordering; repeated values are retained.

`hyper::DefaultTransport` preserves Netfetcher's compatibility process pool.
`hyper::HttpTransport` supplies a separate pool with explicit TLS trust, retry
policy and an optional response-header deadline. HTTP/3 on that pool requires
its own explicit QUIC configuration. The caller limits body reads and selects
the partition lifetime. HTTP/3 retains the imported buffered response contract.

`websocket::connect` accepts an assembled handshake and returns the raw
connection and handshake response. `connect_with` accepts a host-selected TLS
connector. Browser refusals must happen before this call, and cookies returned
by the handshake remain the browser policy owner's responsibility.

The engine has no dependency on Genet or Netfetcher. Netfetcher keeps its crate
name, version, default feature set and public Fetch/WebSocket entry points,
using a small adapter to this engine.
