// SPDX-License-Identifier: MPL-2.0
//! Raw RFC 6455 connector. The caller assembles its handshake, including any
//! credentials, origin and subprotocols. No browser policy or cookie store is
//! consulted here; a browser must refuse forbidden connections before calling.
pub use tokio_tungstenite::tungstenite;
pub use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};
pub type Connection = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
pub type HandshakeResponse = http::Response<Option<Vec<u8>>>;

pub async fn connect(
    request: http::Request<()>,
) -> Result<(Connection, HandshakeResponse), tungstenite::Error> {
    tokio_tungstenite::connect_async(request).await
}
/// An explicit TLS connector is host selected; `None` uses the same default
/// roots as the compatibility connector. No redirect is followed.
pub async fn connect_with(
    request: http::Request<()>,
    connector: Option<Connector>,
) -> Result<(Connection, HandshakeResponse), tungstenite::Error> {
    tokio_tungstenite::connect_async_tls_with_config(request, None, false, connector).await
}
